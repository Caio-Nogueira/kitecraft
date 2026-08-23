mod chunk_wire;
mod storage;
mod vanilla_registries;
mod vanilla_tags;

use game_core::{
    AttackResult, BlockActionId, BlockFace, BlockPosition, BuildHeight, GameCore, GameEffect,
    MovementInput, PlayerId, PlayerInput, PlayerState as CorePlayerState, Position, Rotation,
    WorldView,
};
use net_ws::ids::sb;
use net_ws::nbt::Nbt;
use net_ws::packets::{
    decode_attack, decode_block_dig, decode_block_place, decode_chat_message,
    decode_chunk_batch_received, decode_config, decode_handshake, decode_held_item_slot,
    decode_login_start, decode_movement, decode_play_keep_alive, decode_play_ping_request,
    decode_set_creative_slot, decode_status, play_abilities, play_ack_digging,
    play_attack_animation, play_block_change, play_chunk_batch_finished, play_chunk_batch_start,
    play_custom_payload, play_damage_event, play_entity_destroy, play_game_event,
    play_head_rotation, play_hurt_animation, play_keep_alive, play_login, play_ping_response,
    play_player_info_add, play_player_remove, play_position, play_set_chunk_cache_center,
    play_set_entity_motion, play_set_health, play_spawn_entity_player, play_spawn_position,
    play_sync_entity_position, play_system_chat, play_time_update, play_unload_chunk, status_pong,
    status_response, CreativeSlot, PlayerListEntry, Teleport, GAMEMODE_CREATIVE,
    GAME_EVENT_START_WAITING_FOR_CHUNKS,
};
use net_ws::{decode_frame, encode_frame};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, collections::HashMap, rc::Rc, time::Duration};
use worker::*;
use world::{ChunkColumn, ChunkGenerator, World};
use worldgen_core::PumpkinTerrain;

use crate::storage::{SqliteBacking, WorldgenKind, WorldgenMetadata};

const THRESH: Option<usize> = Some(256);
const VIEW_DISTANCE: i32 = 8;
const SIM_DISTANCE: i32 = 8;
const MAX_PLAYERS: i32 = 64;
const WORLD_NAME: &str = "kitecraft:world";
const BRAND: &str = "kitecraft";
const SPAWN_X: f64 = 0.5;
const SPAWN_Y: f64 = -60.0;
const SPAWN_Z: f64 = 0.5;
const SPAWN_YAW: f32 = 180.0;
const MS_PER_TICK: f64 = 50.0;
const KEEP_ALIVE_EVERY_N_ALARMS: u64 = 10;
const TICK_INTERVAL_MS: u64 = 50;
const MAX_CATCH_UP_TICKS: u64 = 4;
const DEFAULT_WORLD_SEED: u64 = 0;
const DEFAULT_CHUNKS_PER_BATCH: usize = 8;
const MAX_CHUNKS_PER_BATCH: usize = 8;

struct PumpkinChunkGenerator {
    terrain: PumpkinTerrain,
}

impl PumpkinChunkGenerator {
    fn new(seed: u64) -> Self {
        Self {
            terrain: PumpkinTerrain::new(seed),
        }
    }
}

impl ChunkGenerator for PumpkinChunkGenerator {
    fn generate(&mut self, chunk_x: i32, chunk_z: i32) -> ChunkColumn {
        let occupancy = self.terrain.generate_occupancy(chunk_x, chunk_z);
        let mut column = ChunkColumn::default();
        for x in 0..16 {
            for y in world::MIN_Y..world::MIN_Y + world::WORLD_HEIGHT {
                for z in 0..16 {
                    if occupancy.is_stone(x, y, z) {
                        column.set_block(x, y, z, world::blocks::STONE);
                    }
                }
            }
        }
        column
    }
}

struct TickRuntime {
    active: bool,
    generation: u64,
    players: usize,
    ticks: u64,
    late_callbacks: u64,
    dropped_ticks: u64,
    last_tick_wall_ms: u64,
    restored: bool,
    core: GameCore,
    sockets: HashMap<PlayerId, WebSocket>,
}

impl Default for TickRuntime {
    fn default() -> Self {
        Self {
            active: false,
            generation: 0,
            players: 0,
            ticks: 0,
            late_callbacks: 0,
            dropped_ticks: 0,
            last_tick_wall_ms: 0,
            restored: false,
            core: GameCore::new(
                core_spawn_state(),
                BuildHeight {
                    min_y: world::MIN_Y,
                    max_y_exclusive: world::MIN_Y + world::WORLD_HEIGHT,
                },
            ),
            sockets: HashMap::new(),
        }
    }
}

#[derive(Serialize)]
struct TickStats {
    active: bool,
    players: usize,
    ticks: u64,
    late_callbacks: u64,
    dropped_ticks: u64,
    last_tick_wall_ms: u64,
    interval_ms: u64,
}

#[derive(Serialize)]
struct WorldgenStats {
    generator: &'static str,
    seed: u64,
    spawn_y: f64,
    cold_init_wall_ms: u64,
    spawn_chunk_wall_ms: u64,
}

#[event(fetch)]
async fn fetch(req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    let namespace = _env.durable_object("SERVER")?;
    let stub = namespace.id_from_name("primary")?.get_stub()?;
    stub.fetch_with_request(req).await
}

#[derive(Serialize, Deserialize, Clone)]
enum ConnPhase {
    Fresh,
    Status,
    AwaitingLoginStart,
    AwaitingLoginAck { name: String, uuid_hex: String },
    ConfigKnownPacksSent { name: String, uuid_hex: String },
    ConfigFinishSent { name: String, uuid_hex: String },
    Playing(PlayerData),
}

#[derive(Serialize, Deserialize, Clone)]
struct PlayerData {
    name: String,
    uuid_hex: String,
    x: f64,
    y: f64,
    z: f64,
    yaw: f32,
    pitch: f32,
    on_ground: bool,
    sent_chunks: Vec<(i32, i32)>,
    pending_chunks: Vec<(i32, i32)>,
    announced_chunk_center: Option<(i32, i32)>,
    chunks_per_batch: usize,
    awaiting_chunk_batch_ack: bool,
    hotbar: Vec<(u8, u16)>,
    held_slot: u8,
    pending_keepalive: Option<i64>,
    missed_keepalives: u8,
}

impl PlayerData {
    fn uuid(&self) -> [u8; 16] {
        uuid_from_hex(&self.uuid_hex)
    }

    fn entity_id(&self) -> i32 {
        entity_id_for(&self.uuid())
    }

    fn chunk_pos(&self) -> (i32, i32) {
        ((self.x.floor() as i32) >> 4, (self.z.floor() as i32) >> 4)
    }
}

struct Inner {
    world: World<SqliteBacking>,
    worldgen: WorldgenMetadata,
    spawn_y: f64,
    worldgen_init_wall_ms: u64,
    spawn_chunk_wall_ms: u64,
    time_base_age_ticks: i64,
    time_base_wall_ms: u64,
    alarm_ticks: u64,
    teleport_counter: i32,
}

struct ServerWorldView<'a> {
    world: &'a mut World<SqliteBacking>,
}

impl WorldView for ServerWorldView<'_> {
    fn block_state(&mut self, position: BlockPosition) -> Option<u16> {
        self.world.get_block(position.x, position.y, position.z)
    }
}

#[durable_object]
pub struct MinecraftServer {
    state: State,
    inner: Rc<RefCell<Option<Inner>>>,
    tick_runtime: Rc<RefCell<TickRuntime>>,
}

fn core_spawn_state() -> CorePlayerState {
    core_spawn_state_at(SPAWN_Y)
}

fn core_spawn_state_at(y: f64) -> CorePlayerState {
    CorePlayerState {
        position: Position {
            x: SPAWN_X,
            y,
            z: SPAWN_Z,
        },
        rotation: Rotation {
            yaw: SPAWN_YAW,
            pitch: 0.0,
        },
        on_ground: false,
    }
}

fn core_state_from_player(pd: &PlayerData) -> CorePlayerState {
    CorePlayerState {
        position: Position {
            x: pd.x,
            y: pd.y,
            z: pd.z,
        },
        rotation: Rotation {
            yaw: pd.yaw,
            pitch: pd.pitch,
        },
        on_ground: pd.on_ground,
    }
}

fn apply_core_state(pd: &mut PlayerData, state: CorePlayerState) {
    pd.x = state.position.x;
    pd.y = state.position.y;
    pd.z = state.position.z;
    pd.yaw = state.rotation.yaw;
    pd.pitch = state.rotation.pitch;
    pd.on_ground = state.on_ground;
}

fn uuid_from_hex(hex_str: &str) -> [u8; 16] {
    let mut out = [0u8; 16];
    for (i, slot) in out.iter_mut().enumerate() {
        *slot = u8::from_str_radix(hex_str.get(i * 2..i * 2 + 2).unwrap_or("00"), 16).unwrap_or(0);
    }
    out
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn offline_uuid(name: &str) -> [u8; 16] {
    use md5::{Digest, Md5};
    let mut hasher = Md5::new();
    hasher.update(b"OfflinePlayer:");
    hasher.update(name.as_bytes());
    let digest = hasher.finalize();
    let mut out = [0u8; 16];
    out.copy_from_slice(&digest);
    out[6] = (out[6] & 0x0F) | 0x30;
    out[8] = (out[8] & 0x3F) | 0x80;
    out
}

fn entity_id_for(uuid: &[u8; 16]) -> i32 {
    (i32::from_le_bytes([uuid[0], uuid[1], uuid[2], uuid[3]]) & 0x7FFF_FFFF) | 1
}

fn sanitize(s: &str) -> String {
    s.chars()
        .filter(|&c| c != '\n' && c != '\r' && c != '\0')
        .take(256)
        .collect()
}

fn chebyshev(x1: i32, z1: i32, x2: i32, z2: i32) -> i32 {
    (x1 - x2).abs().max((z1 - z2).abs())
}

fn desired_chunks(center: (i32, i32)) -> Vec<(i32, i32)> {
    let mut chunks: Vec<_> = (-VIEW_DISTANCE..=VIEW_DISTANCE)
        .flat_map(|dx| {
            (-VIEW_DISTANCE..=VIEW_DISTANCE).map(move |dz| (center.0 + dx, center.1 + dz))
        })
        .collect();
    chunks.sort_by_key(|&(x, z)| {
        let dx = x - center.0;
        let dz = z - center.1;
        (dx.abs().max(dz.abs()), dx.abs() + dz.abs(), dx, dz)
    });
    chunks
}

fn reset_chunk_window(pd: &mut PlayerData, center: (i32, i32)) -> Vec<(i32, i32)> {
    let mut unload = Vec::new();
    pd.sent_chunks.retain(|&(x, z)| {
        let keep = chebyshev(x, z, center.0, center.1) <= VIEW_DISTANCE;
        if !keep {
            unload.push((x, z));
        }
        keep
    });
    pd.pending_chunks = desired_chunks(center)
        .into_iter()
        .filter(|chunk| !pd.sent_chunks.contains(chunk))
        .collect();
    pd.announced_chunk_center = Some(center);
    unload
}

fn acknowledged_batch_size(chunks_per_tick: f32) -> usize {
    if !chunks_per_tick.is_finite() || chunks_per_tick <= 0.0 {
        return 1;
    }
    (chunks_per_tick.floor() as usize).clamp(1, MAX_CHUNKS_PER_BATCH)
}

impl MinecraftServer {
    fn ensure_tick_loop(&self) {
        let generation = {
            let mut runtime = self.tick_runtime.borrow_mut();
            if runtime.active || runtime.players == 0 {
                return;
            }
            runtime.active = true;
            runtime.generation = runtime.generation.wrapping_add(1);
            runtime.generation
        };

        let runtime = Rc::clone(&self.tick_runtime);
        let inner = Rc::clone(&self.inner);
        worker::wasm_bindgen_futures::spawn_local(async move {
            let mut next_deadline = Date::now().as_millis() + TICK_INTERVAL_MS;
            loop {
                let delay_ms = next_deadline.saturating_sub(Date::now().as_millis());
                worker::Delay::from(Duration::from_millis(delay_ms)).await;
                let now = Date::now().as_millis();

                let mut state = runtime.borrow_mut();
                if state.generation != generation || state.players == 0 {
                    if state.generation == generation {
                        state.active = false;
                    }
                    break;
                }

                let elapsed_ticks = 1 + now.saturating_sub(next_deadline) / TICK_INTERVAL_MS;
                if now > next_deadline {
                    state.late_callbacks += 1;
                }
                let due_ticks = elapsed_ticks.min(MAX_CATCH_UP_TICKS);
                state.dropped_ticks += elapsed_ticks.saturating_sub(due_ticks);
                let mut effects = Vec::new();
                {
                    let mut inner_guard = inner.borrow_mut();
                    let mut world = ServerWorldView {
                        world: &mut inner_guard.as_mut().expect("inner").world,
                    };
                    for _ in 0..due_ticks {
                        state.ticks += 1;
                        let tick = state.ticks;
                        effects.extend(state.core.tick(tick, &mut world));
                    }
                }
                state.last_tick_wall_ms = now;
                let sockets = state.sockets.clone();
                drop(state);

                for effect in effects {
                    Self::apply_game_effect(&inner, &sockets, effect);
                }

                next_deadline = if elapsed_ticks > MAX_CATCH_UP_TICKS {
                    now + TICK_INTERVAL_MS
                } else {
                    next_deadline + elapsed_ticks * TICK_INTERVAL_MS
                };
            }
        });
    }

    fn player_entered(&self, ws: &WebSocket, pd: &PlayerData) {
        let mut runtime = self.tick_runtime.borrow_mut();
        let player = PlayerId(pd.uuid());
        runtime.core.join(player, core_state_from_player(pd));
        runtime.sockets.insert(player, ws.clone());
        runtime.players = runtime.core.player_count();
        drop(runtime);
        self.ensure_tick_loop();
    }

    fn player_left(&self, player: PlayerId) -> Option<CorePlayerState> {
        let mut runtime = self.tick_runtime.borrow_mut();
        runtime.sockets.remove(&player);
        let player_state = runtime.core.leave(player);
        runtime.players = runtime.core.player_count();
        if runtime.players == 0 {
            runtime.active = false;
            runtime.generation = runtime.generation.wrapping_add(1);
            let ticks = runtime.ticks;
            let last_tick_wall_ms = runtime.last_tick_wall_ms;
            drop(runtime);
            if let Some(inner) = self.inner.borrow().as_ref() {
                inner.world.backing().set_meta_pair(
                    "tick_stats",
                    ticks.min(i64::MAX as u64) as i64,
                    last_tick_wall_ms.min(i64::MAX as u64) as i64,
                );
            }
        }
        player_state
    }

    fn enqueue_movement(&self, player: PlayerId, movement: net_ws::packets::Movement) {
        let movement = match movement {
            net_ws::packets::Movement::Position { x, y, z, on_ground } => MovementInput {
                position: Some(Position { x, y, z }),
                rotation: None,
                on_ground,
            },
            net_ws::packets::Movement::PositionLook {
                x,
                y,
                z,
                yaw,
                pitch,
                on_ground,
            } => MovementInput {
                position: Some(Position { x, y, z }),
                rotation: Some(Rotation { yaw, pitch }),
                on_ground,
            },
            net_ws::packets::Movement::Look {
                yaw,
                pitch,
                on_ground,
            } => MovementInput {
                position: None,
                rotation: Some(Rotation { yaw, pitch }),
                on_ground,
            },
            net_ws::packets::Movement::Flying { on_ground } => MovementInput {
                position: None,
                rotation: None,
                on_ground,
            },
        };
        self.tick_runtime
            .borrow_mut()
            .core
            .enqueue(PlayerInput::Move { player, movement });
    }

    fn enqueue_attack(&self, player: PlayerId, target_entity_id: i32) {
        let mut runtime = self.tick_runtime.borrow_mut();
        let target = runtime
            .sockets
            .keys()
            .copied()
            .find(|target| entity_id_for(&target.0) == target_entity_id);
        if let Some(target) = target {
            runtime.core.enqueue(PlayerInput::Attack { player, target });
        }
    }

    fn tick_stats_response(&self) -> Result<Response> {
        let runtime = self.tick_runtime.borrow();
        Response::from_json(&TickStats {
            active: runtime.active,
            players: runtime.players,
            ticks: runtime.ticks,
            late_callbacks: runtime.late_callbacks,
            dropped_ticks: runtime.dropped_ticks,
            last_tick_wall_ms: runtime.last_tick_wall_ms,
            interval_ms: TICK_INTERVAL_MS,
        })
    }

    fn worldgen_stats_response(&self) -> Result<Response> {
        let guard = self.inner.borrow();
        let inner = guard.as_ref().expect("inner");
        Response::from_json(&WorldgenStats {
            generator: inner.worldgen.kind.name(),
            seed: inner.worldgen.seed,
            spawn_y: inner.spawn_y,
            cold_init_wall_ms: inner.worldgen_init_wall_ms,
            spawn_chunk_wall_ms: inner.spawn_chunk_wall_ms,
        })
    }

    fn ensure_inner(&self) -> Result<()> {
        if self.inner.borrow().is_some() {
            return Ok(());
        }
        let storage = self.state.storage();
        let backing = SqliteBacking::new(storage.sql());
        backing.init_schema()?;

        let worldgen = match backing.worldgen_metadata() {
            Some(metadata) => metadata,
            None if backing.has_chunks() => {
                let metadata = WorldgenMetadata {
                    kind: WorldgenKind::Superflat,
                    seed: 0,
                };
                backing.set_worldgen_metadata(metadata);
                metadata
            }
            None => {
                let metadata = WorldgenMetadata {
                    kind: WorldgenKind::PumpkinTerrain,
                    seed: DEFAULT_WORLD_SEED,
                };
                backing.set_worldgen_metadata(metadata);
                metadata
            }
        };

        let now_ms = Date::now().as_millis();
        let (base_age, base_wall) = match backing.meta_pair("time_base") {
            Some(pair) => pair,
            None => {
                backing.set_meta_pair("time_base", 1000, now_ms as i64);
                (1000, now_ms as i64)
            }
        };

        let persisted_tick_stats = backing.meta_pair("tick_stats");
        let init_started = Date::now().as_millis();
        let generator: Box<dyn ChunkGenerator> = match worldgen.kind {
            WorldgenKind::Superflat => Box::new(world::superflat::Superflat::classic()),
            WorldgenKind::PumpkinTerrain => Box::new(PumpkinChunkGenerator::new(worldgen.seed)),
        };
        let worldgen_init_wall_ms = Date::now().as_millis().saturating_sub(init_started);
        let mut world = World::with_generator_and_cache_size(backing, generator, 256);
        let spawn_started = Date::now().as_millis();
        let spawn_y = match world.backing().meta_pair("spawn") {
            Some((y, _)) => y as f64,
            None => {
                let column = world.chunk(0, 0);
                let y = (world::MIN_Y..world::MIN_Y + world::WORLD_HEIGHT)
                    .rev()
                    .find(|&y| {
                        column.get_block(0, y, 0).unwrap_or(world::blocks::AIR)
                            != world::blocks::AIR
                    })
                    .map_or(SPAWN_Y, |y| f64::from(y + 1));
                world.backing().set_meta_pair("spawn", y as i64, 0);
                y
            }
        };
        let spawn_chunk_wall_ms = Date::now().as_millis().saturating_sub(spawn_started);
        self.inner.borrow_mut().replace(Inner {
            world,
            worldgen,
            spawn_y,
            worldgen_init_wall_ms,
            spawn_chunk_wall_ms,
            time_base_age_ticks: base_age,
            time_base_wall_ms: base_wall as u64,
            alarm_ticks: 0,
            teleport_counter: 1,
        });
        let mut runtime = self.tick_runtime.borrow_mut();
        runtime.core.set_spawn(core_spawn_state_at(spawn_y));
        if !runtime.restored {
            if let Some((ticks, last_tick_wall_ms)) = persisted_tick_stats {
                runtime.ticks = ticks.max(0) as u64;
                runtime.last_tick_wall_ms = last_tick_wall_ms.max(0) as u64;
            }
            runtime.restored = true;
        }
        Ok(())
    }

    fn current_tick(inner: &Inner, now_ms: u64) -> i64 {
        let elapsed = now_ms.saturating_sub(inner.time_base_wall_ms) as f64 / MS_PER_TICK;
        inner.time_base_age_ticks + elapsed.floor() as i64
    }

    fn playing_sockets(&self) -> Vec<(WebSocket, PlayerData)> {
        self.state
            .get_websockets()
            .into_iter()
            .filter_map(|ws| match ws.deserialize_attachment::<ConnPhase>() {
                Ok(Some(ConnPhase::Playing(pd))) => Some((ws, pd)),
                _ => None,
            })
            .collect()
    }

    async fn schedule_alarm_in(&self, ms: i64) -> Result<()> {
        self.state.storage().set_alarm(ms).await
    }

    fn status_json(&self) -> String {
        let online = self.playing_sockets().len();
        format!(
            "{{\"version\":{{\"name\":\"{}\",\"protocol\":{}}},\"players\":{{\"max\":{},\"online\":{}}},\"description\":{{\"text\":\"KiteCraft on Cloudflare Workers\"}},\"enforcesSecureChat\":false,\"preventsChatReports\":true}}",
            net_ws::MINECRAFT_VERSION,
            net_ws::PROTOCOL_VERSION,
            MAX_PLAYERS,
            online
        )
    }

    fn info_response(&self) -> Result<Response> {
        Response::ok(
            "KiteCraft — Minecraft Java Edition over WebSockets (wsmc).\nConnect with the wsmc client mod to wss://<this-host>/<any-path>.\n",
        )
    }

    fn send_payload(ws: &WebSocket, payload: &[u8]) -> Result<()> {
        ws.send_with_bytes(encode_frame(payload, THRESH).as_slice())
    }

    fn enter_configuration(&self, ws: &WebSocket, name: &str, uuid_hex: String) -> Result<()> {
        Self::send_payload(ws, &net_ws::packets::cfg_known_packs())?;
        ws.serialize_attachment(&ConnPhase::ConfigKnownPacksSent {
            name: name.to_string(),
            uuid_hex,
        })?;
        Ok(())
    }

    fn send_registries_and_finish(&self, ws: &WebSocket) -> Result<()> {
        for registry in vanilla_registries::VANILLA_REGISTRIES {
            let entries: Vec<(&str, Option<&[u8]>)> = registry
                .entries
                .iter()
                .map(|e| {
                    (
                        e.name,
                        Some(
                            vanilla_registries::REGISTRY_BLOB
                                .get(e.offset..e.offset + e.len)
                                .unwrap_or(&[]),
                        ),
                    )
                })
                .collect();
            Self::send_payload(
                ws,
                &net_ws::packets::cfg_registry_data(registry.id, &entries),
            )?;
        }

        Self::send_payload(ws, &net_ws::packets::cfg_tags(vanilla_tags::TAG_DATA))?;

        let mut brand = net_ws::Encoder::new();
        brand.string(BRAND);
        let brand_bytes = brand.into_bytes();
        Self::send_payload(
            ws,
            &net_ws::packets::cfg_custom_payload("minecraft:brand", &brand_bytes),
        )?;
        Self::send_payload(ws, &net_ws::packets::cfg_finish())?;
        Ok(())
    }

    fn enter_play(&self, ws: &WebSocket, name: &str, uuid_hex: String) -> Result<()> {
        let uuid = uuid_from_hex(&uuid_hex);

        let saved = {
            let guard = self.inner.borrow();
            load_saved_position(guard.as_ref().expect("inner"), &uuid_hex)
        };
        let spawn_y = self.inner.borrow().as_ref().expect("inner").spawn_y;
        let (px, py, pz, pyaw, ppitch) =
            saved.unwrap_or((SPAWN_X, spawn_y, SPAWN_Z, SPAWN_YAW, 0.0));

        let entity_id = entity_id_for(&uuid);
        let entry = PlayerListEntry {
            uuid,
            name: name.to_string(),
            gamemode: GAMEMODE_CREATIVE,
            listed: true,
            ping_ms: 0,
        };

        let login = play_login(
            entity_id,
            false,
            &[WORLD_NAME],
            MAX_PLAYERS,
            VIEW_DISTANCE,
            SIM_DISTANCE,
            vanilla_registries::DIMENSION_TYPE_OVERWORLD,
            WORLD_NAME,
            1044524699,
            GAMEMODE_CREATIVE,
            true,
            63,
        );
        let waiting = play_game_event(GAME_EVENT_START_WAITING_FOR_CHUNKS, 0.0);
        let abilities = play_abilities(0x01 | 0x04 | 0x08);
        let mut brand_enc = net_ws::Encoder::new();
        brand_enc.string(BRAND);
        let brand_data = brand_enc.into_bytes();
        let brand = play_custom_payload("minecraft:brand", &brand_data);
        let spawn_pos = play_spawn_position(0, -60, 0);

        let teleport_id = self.next_teleport_id();
        let teleport = play_position(&Teleport {
            id: teleport_id,
            x: px,
            y: py,
            z: pz,
            yaw: pyaw,
            pitch: ppitch,
        });
        let self_info = play_player_info_add(&entry);

        for frame in [
            login,
            waiting,
            abilities,
            brand,
            spawn_pos,
            teleport,
            self_info.clone(),
        ] {
            Self::send_payload(ws, &frame)?;
        }

        let others = self.playing_sockets();

        for (other_ws, other_pd) in &others {
            let their_entry = PlayerListEntry {
                uuid: other_pd.uuid(),
                name: other_pd.name.clone(),
                gamemode: GAMEMODE_CREATIVE,
                listed: true,
                ping_ms: 0,
            };
            for frame in [
                play_player_info_add(&their_entry),
                play_spawn_entity_player(
                    other_pd.entity_id(),
                    &other_pd.uuid(),
                    other_pd.x,
                    other_pd.y,
                    other_pd.z,
                    other_pd.yaw,
                    other_pd.pitch,
                ),
            ] {
                Self::send_payload(ws, &frame)?;
            }
            for frame in [
                self_info.clone(),
                play_spawn_entity_player(entity_id, &uuid, px, py, pz, pyaw, ppitch),
            ] {
                Self::send_payload(other_ws, &frame)?;
            }
        }

        let mut pd = PlayerData {
            name: name.to_string(),
            uuid_hex: uuid_hex.clone(),
            x: px,
            y: py,
            z: pz,
            yaw: pyaw,
            pitch: ppitch,
            on_ground: false,
            sent_chunks: Vec::new(),
            pending_chunks: Vec::new(),
            announced_chunk_center: None,
            chunks_per_batch: DEFAULT_CHUNKS_PER_BATCH,
            awaiting_chunk_batch_ack: false,
            hotbar: Vec::new(),
            held_slot: 0,
            pending_keepalive: None,
            missed_keepalives: 0,
        };

        for frame in Self::update_chunk_window_from(&self.inner, &mut pd) {
            Self::send_payload(ws, &frame)?;
        }

        let join_text = format!("§e{name} joined the game");
        let join_chat =
            play_system_chat(&Nbt::compound(vec![("text", Nbt::str(join_text))]), false);
        for (other_ws, _) in self.playing_sockets() {
            let _ = Self::send_payload(&other_ws, &join_chat);
        }

        ws.serialize_attachment(ConnPhase::Playing(pd.clone()))?;
        self.player_entered(ws, &pd);
        Ok(())
    }

    fn next_teleport_id(&self) -> i32 {
        Self::next_teleport_id_from(&self.inner)
    }

    fn next_teleport_id_from(inner_ref: &Rc<RefCell<Option<Inner>>>) -> i32 {
        let mut guard = inner_ref.borrow_mut();
        let inner = guard.as_mut().expect("inner");
        let id = inner.teleport_counter;
        inner.teleport_counter += 1;
        id
    }

    fn build_next_chunk_batch_from(
        inner_ref: &Rc<RefCell<Option<Inner>>>,
        pd: &mut PlayerData,
    ) -> Vec<Vec<u8>> {
        if pd.awaiting_chunk_batch_ack || pd.pending_chunks.is_empty() {
            return Vec::new();
        }
        let batch_size = pd.chunks_per_batch.min(pd.pending_chunks.len());
        let batch: Vec<_> = pd.pending_chunks.drain(..batch_size).collect();

        let mut frames = vec![play_chunk_batch_start()];
        {
            let mut guard = inner_ref.borrow_mut();
            let inner = guard.as_mut().expect("inner");
            for &(cx, cz) in &batch {
                let col = inner.world.chunk(cx, cz);
                frames.push(chunk_wire::encode_chunk_packet(cx, cz, col));
            }
        }
        frames.push(play_chunk_batch_finished(batch.len() as i32));
        pd.sent_chunks.extend(batch);
        pd.awaiting_chunk_batch_ack = true;
        frames
    }

    fn update_chunk_window_from(
        inner_ref: &Rc<RefCell<Option<Inner>>>,
        pd: &mut PlayerData,
    ) -> Vec<Vec<u8>> {
        let center = pd.chunk_pos();
        if pd.announced_chunk_center == Some(center) {
            return Vec::new();
        }

        let unload = reset_chunk_window(pd, center);
        let mut frames = vec![play_set_chunk_cache_center(center.0, center.1)];
        frames.extend(
            unload
                .into_iter()
                .map(|(chunk_x, chunk_z)| play_unload_chunk(chunk_x, chunk_z)),
        );
        frames.extend(Self::build_next_chunk_batch_from(inner_ref, pd));
        frames
    }

    async fn handle_playing_message(
        &self,
        ws: &WebSocket,
        mut pd: PlayerData,
        payload: &[u8],
    ) -> Result<()> {
        let packet_id = net_ws::Decoder::new(payload).varint().unwrap_or(-1);

        match packet_id {
            sb::PLAY_CHUNK_BATCH_RECEIVED => {
                if let Ok(chunks_per_tick) = decode_chunk_batch_received(payload) {
                    pd.chunks_per_batch = acknowledged_batch_size(chunks_per_tick);
                    if pd.awaiting_chunk_batch_ack {
                        pd.awaiting_chunk_batch_ack = false;
                        for frame in Self::build_next_chunk_batch_from(&self.inner, &mut pd) {
                            Self::send_payload(ws, &frame)?;
                        }
                    }
                    ws.serialize_attachment(ConnPhase::Playing(pd))?;
                }
            }
            sb::PLAY_ATTACK => {
                if let Ok(attack) = decode_attack(payload) {
                    self.enqueue_attack(PlayerId(pd.uuid()), attack.entity_id);
                }
            }
            sb::PLAY_KEEP_ALIVE => {
                if decode_play_keep_alive(payload).is_ok() {
                    pd.pending_keepalive = None;
                    pd.missed_keepalives = 0;
                    ws.serialize_attachment(ConnPhase::Playing(pd))?;
                }
            }
            sb::PLAY_PING_REQUEST => {
                if let Ok(id) = decode_play_ping_request(payload) {
                    Self::send_payload(ws, &play_ping_response(id))?;
                }
            }
            sb::PLAY_POSITION | sb::PLAY_POSITION_LOOK | sb::PLAY_LOOK | sb::PLAY_FLYING => {
                if let Ok(movement) = decode_movement(payload) {
                    self.enqueue_movement(PlayerId(pd.uuid()), movement);
                }
            }
            sb::PLAY_CHAT_MESSAGE => {
                if let Ok(msg) = decode_chat_message(payload) {
                    self.handle_chat(&pd, &msg.message).await?;
                }
            }
            sb::PLAY_BLOCK_DIG => {
                if let Ok(dig) = decode_block_dig(payload) {
                    self.tick_runtime
                        .borrow_mut()
                        .core
                        .enqueue(PlayerInput::DigBlock {
                            player: PlayerId(pd.uuid()),
                            position: BlockPosition {
                                x: dig.x,
                                y: dig.y,
                                z: dig.z,
                            },
                            break_block: dig.status == 0,
                            action_id: BlockActionId(dig.sequence),
                        });
                }
            }
            sb::PLAY_BLOCK_PLACE => {
                let Ok(place) = decode_block_place(payload) else {
                    return Ok(());
                };
                let held_state = pd
                    .hotbar
                    .iter()
                    .find(|(slot, _)| *slot == pd.held_slot || *slot == pd.held_slot + 36)
                    .map(|(_, state)| *state);
                self.tick_runtime
                    .borrow_mut()
                    .core
                    .enqueue(PlayerInput::PlaceBlock {
                        player: PlayerId(pd.uuid()),
                        clicked: BlockPosition {
                            x: place.x,
                            y: place.y,
                            z: place.z,
                        },
                        face: BlockFace::try_from(place.direction).ok(),
                        state: held_state,
                        action_id: BlockActionId(place.sequence),
                    });
            }
            sb::PLAY_SET_CREATIVE_SLOT => {
                if let Ok(Some(CreativeSlot {
                    slot,
                    item_id,
                    count,
                })) = decode_set_creative_slot(payload)
                {
                    pd.hotbar.retain(|(s, _)| *s != slot as u8);
                    let block_state = item_id.and_then(block_data::default_state_for_item);
                    if let (Some(state), true) = (block_state, count > 0) {
                        pd.hotbar.push((slot as u8, state));
                        pd.hotbar.sort_by_key(|(s, _)| *s);
                    }
                    ws.serialize_attachment(ConnPhase::Playing(pd))?;
                }
            }
            sb::PLAY_HELD_ITEM_SLOT => {
                if let Ok(Some(slot)) = decode_held_item_slot(payload) {
                    pd.held_slot = slot.rem_euclid(9) as u8;
                    ws.serialize_attachment(ConnPhase::Playing(pd))?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn apply_game_effect(
        inner_ref: &Rc<RefCell<Option<Inner>>>,
        sockets: &HashMap<PlayerId, WebSocket>,
        effect: GameEffect,
    ) {
        match effect {
            GameEffect::PlayerMoved {
                player,
                state,
                correction,
                state_changed,
            } => Self::apply_player_moved(
                inner_ref,
                sockets,
                player,
                state,
                correction.is_some(),
                state_changed,
            ),
            GameEffect::BlockActionProcessed {
                player,
                action_id,
                change,
            } => {
                if let Some(ws) = sockets.get(&player) {
                    let _ = Self::send_payload(ws, &play_ack_digging(action_id.0));
                }
                if let Some(change) = change {
                    {
                        let mut guard = inner_ref.borrow_mut();
                        guard.as_mut().expect("inner").world.set_block(
                            change.position.x,
                            change.position.y,
                            change.position.z,
                            change.state,
                        );
                    }
                    let frame = play_block_change(
                        change.position.x,
                        change.position.y,
                        change.position.z,
                        change.state,
                    );
                    for other_ws in sockets.values() {
                        let _ = Self::send_payload(other_ws, &frame);
                    }
                }
            }
            GameEffect::AttackProcessed {
                attacker,
                target,
                result: AttackResult::Damaged { target: combat, .. },
            } => {
                let attacker_entity_id = entity_id_for(&attacker.0);
                let target_entity_id = entity_id_for(&target.0);
                let swing = play_attack_animation(attacker_entity_id);
                let hurt = play_hurt_animation(target_entity_id, 0.0);
                let damage = play_damage_event(
                    target_entity_id,
                    attacker_entity_id,
                    vanilla_registries::DAMAGE_TYPE_PLAYER_ATTACK,
                );
                let velocity = play_set_entity_motion(
                    target_entity_id,
                    combat.velocity.x,
                    combat.velocity.y,
                    combat.velocity.z,
                );
                for other_ws in sockets.values() {
                    let _ = Self::send_payload(other_ws, &swing);
                    let _ = Self::send_payload(other_ws, &hurt);
                    let _ = Self::send_payload(other_ws, &damage);
                    let _ = Self::send_payload(other_ws, &velocity);
                }
                if let Some(target_ws) = sockets.get(&target) {
                    let _ = Self::send_payload(target_ws, &play_set_health(combat.health));
                }
            }
            GameEffect::AttackProcessed {
                result: AttackResult::Rejected(_),
                ..
            } => {}
        }
    }

    fn apply_player_moved(
        inner_ref: &Rc<RefCell<Option<Inner>>>,
        sockets: &HashMap<PlayerId, WebSocket>,
        player: PlayerId,
        state: CorePlayerState,
        send_correction: bool,
        state_changed: bool,
    ) {
        let Some(ws) = sockets.get(&player) else {
            return;
        };
        let Ok(Some(ConnPhase::Playing(mut pd))) = ws.deserialize_attachment::<ConnPhase>() else {
            return;
        };
        apply_core_state(&mut pd, state);

        if send_correction {
            let tid = Self::next_teleport_id_from(inner_ref);
            let tp = play_position(&Teleport {
                id: tid,
                x: pd.x,
                y: pd.y,
                z: pd.z,
                yaw: pd.yaw,
                pitch: pd.pitch,
            });
            let _ = Self::send_payload(ws, &tp);
        }

        if state_changed {
            for frame in Self::update_chunk_window_from(inner_ref, &mut pd) {
                let _ = Self::send_payload(ws, &frame);
            }
        }
        if state_changed {
            for (other_player, other_ws) in sockets {
                if *other_player == player {
                    continue;
                }
                let sync = play_sync_entity_position(
                    pd.entity_id(),
                    pd.x,
                    pd.y,
                    pd.z,
                    pd.yaw,
                    pd.pitch,
                    pd.on_ground,
                );
                let head = play_head_rotation(pd.entity_id(), pd.yaw);
                let _ = Self::send_payload(other_ws, &sync);
                let _ = Self::send_payload(other_ws, &head);
            }
        }

        let _ = ws.serialize_attachment(ConnPhase::Playing(pd));
    }

    async fn handle_chat(&self, pd: &PlayerData, message: &str) -> Result<()> {
        if let Some(command) = message.strip_prefix('/') {
            self.handle_command(command).await?;
            return Ok(());
        }
        let text = format!("§f<{}> {}", pd.name, sanitize(message));
        let frame = play_system_chat(&Nbt::compound(vec![("text", Nbt::str(text))]), false);
        for (other_ws, _) in self.playing_sockets() {
            let _ = Self::send_payload(&other_ws, &frame);
        }
        Ok(())
    }

    async fn handle_command(&self, command: &str) -> Result<()> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if let ["time", "set", unit @ ("day" | "noon" | "night")] = parts.as_slice() {
            let target = match *unit {
                "day" => 1000,
                "noon" => 6000,
                _ => 13000,
            };
            self.set_day_time(target).await?;
        }
        Ok(())
    }

    async fn set_day_time(&self, day_tick: i64) -> Result<()> {
        let now = Date::now().as_millis();
        {
            let mut guard = self.inner.borrow_mut();
            let inner = guard.as_mut().expect("inner");
            let current = Self::current_tick(inner, now);
            let day_floor = current.div_euclid(24000) * 24000;
            let tod = current.rem_euclid(24000);
            let target = if day_tick <= tod {
                day_floor + 24000 + day_tick
            } else {
                day_floor + day_tick
            };
            inner.time_base_age_ticks = target;
            inner.time_base_wall_ms = now;
            inner
                .world
                .backing()
                .set_meta_pair("time_base", target, now as i64);
        }
        let tick = Self::current_tick(
            self.inner.borrow().as_ref().expect("inner"),
            Date::now().as_millis(),
        );
        let frame = play_time_update(
            tick,
            tick.rem_euclid(24000),
            vanilla_registries::WORLD_CLOCK_OVERWORLD,
        );
        for (ws, _) in self.playing_sockets() {
            let _ = Self::send_payload(&ws, &frame);
        }
        Ok(())
    }

    fn broadcast_join_leave(&self, text: &str) {
        let frame = play_system_chat(
            &Nbt::compound(vec![("text", Nbt::str(text.to_string()))]),
            false,
        );
        for (ws, _) in self.playing_sockets() {
            let _ = Self::send_payload(&ws, &frame);
        }
    }

    async fn cleanup_connection(&self, ws: &WebSocket) -> Result<()> {
        let phase: Option<ConnPhase> = ws.deserialize_attachment().ok().flatten();
        if let Some(ConnPhase::Playing(mut pd)) = phase {
            // Mark this socket as no longer playing before any subsequent close/error callback.
            let _ = ws.serialize_attachment(&ConnPhase::Fresh);
            if let Some(state) = self.player_left(PlayerId(pd.uuid())) {
                apply_core_state(&mut pd, state);
            }
            {
                let guard = self.inner.borrow();
                save_player_position(guard.as_ref().expect("inner"), &pd);
            }

            let remove = play_player_remove(&[pd.uuid()]);
            let destroy = play_entity_destroy(&[pd.entity_id()]);
            for (other_ws, _) in self.playing_sockets() {
                let _ = Self::send_payload(&other_ws, &remove);
                let _ = Self::send_payload(&other_ws, &destroy);
            }
            self.broadcast_join_leave(&format!("§e{} left the game", pd.name));

            {
                let mut guard = self.inner.borrow_mut();
                if let Some(inner) = guard.as_mut() {
                    inner.world.flush_dirty();
                }
            }

            if self.playing_sockets().is_empty() {
                let _ = self.state.storage().delete_alarm().await;
            }
        }
        Ok(())
    }

    async fn run_alarm_work(&self) -> Result<()> {
        self.ensure_inner()?;
        let now = Date::now().as_millis();

        let tick = Self::current_tick(self.inner.borrow().as_ref().expect("inner"), now);
        let time_frame = play_time_update(
            tick,
            tick.rem_euclid(24000),
            vanilla_registries::WORLD_CLOCK_OVERWORLD,
        );

        let players = self.playing_sockets();
        let is_keepalive_tick = {
            let mut guard = self.inner.borrow_mut();
            let inner = guard.as_mut().expect("inner");
            inner.alarm_ticks += 1;
            inner.alarm_ticks.is_multiple_of(KEEP_ALIVE_EVERY_N_ALARMS)
        };

        for (ws, mut pd) in players {
            let _ = Self::send_payload(&ws, &time_frame);

            if is_keepalive_tick {
                if pd.pending_keepalive.is_some() {
                    pd.missed_keepalives += 1;
                }
                if pd.missed_keepalives > 2 {
                    let kick = net_ws::packets::play_kick("Timed out");
                    let _ = Self::send_payload(&ws, &kick);
                    let _ = ws.close(Some(1001), Some("timeout"));
                    continue;
                }
                let id = now as i64;
                pd.pending_keepalive = Some(id);
                let ka = play_keep_alive(id);
                let _ = Self::send_payload(&ws, &ka);
                let _ = ws.serialize_attachment(ConnPhase::Playing(pd));
            }
        }

        {
            let mut guard = self.inner.borrow_mut();
            if let Some(inner) = guard.as_mut() {
                inner.world.flush_dirty();
            }
        }

        for (_, pd) in self.playing_sockets() {
            let guard = self.inner.borrow();
            save_player_position(guard.as_ref().expect("inner"), &pd);
        }

        if self.playing_sockets().is_empty() {
            let _ = self.state.storage().delete_alarm().await;
        } else {
            self.schedule_alarm_in(1000).await?;
        }
        Ok(())
    }
}

fn player_blob(pd: &PlayerData) -> Vec<u8> {
    let mut blob = Vec::with_capacity(32);
    blob.extend_from_slice(&pd.x.to_le_bytes());
    blob.extend_from_slice(&pd.y.to_le_bytes());
    blob.extend_from_slice(&pd.z.to_le_bytes());
    blob.extend_from_slice(&pd.yaw.to_le_bytes());
    blob.extend_from_slice(&pd.pitch.to_le_bytes());
    blob
}

fn save_player_position(inner: &Inner, pd: &PlayerData) {
    inner
        .world
        .backing()
        .set_blob(&format!("player:{}", pd.uuid_hex), &player_blob(pd));
}

fn load_saved_position(inner: &Inner, uuid_hex: &str) -> Option<(f64, f64, f64, f32, f32)> {
    let blob = inner
        .world
        .backing()
        .get_blob(&format!("player:{uuid_hex}"))?;
    if blob.len() < 32 {
        return None;
    }
    Some((
        f64::from_le_bytes(blob[0..8].try_into().ok()?),
        f64::from_le_bytes(blob[8..16].try_into().ok()?),
        f64::from_le_bytes(blob[16..24].try_into().ok()?),
        f32::from_le_bytes(blob[24..28].try_into().ok()?),
        f32::from_le_bytes(blob[28..32].try_into().ok()?),
    ))
}

impl DurableObject for MinecraftServer {
    fn new(state: State, _env: Env) -> Self {
        Self {
            state,
            inner: Rc::new(RefCell::new(None)),
            tick_runtime: Rc::new(RefCell::new(TickRuntime::default())),
        }
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        let upgrade = req
            .headers()
            .get("Upgrade")?
            .map(|v| v.eq_ignore_ascii_case("websocket"))
            .unwrap_or(false);

        if !upgrade {
            if req.path() == "/__kitecraft/tick-stats" {
                self.ensure_inner()?;
                return self.tick_stats_response();
            }
            if req.path() == "/__kitecraft/worldgen-stats" {
                self.ensure_inner()?;
                return self.worldgen_stats_response();
            }
            return self.info_response();
        }

        self.ensure_inner()?;

        let pair = WebSocketPair::new()?;
        self.state
            .accept_websocket_with_tags(&pair.server, &["kitecraft"]);
        pair.server.serialize_attachment(&ConnPhase::Fresh)?;
        self.schedule_alarm_in(1000).await?;
        Response::from_websocket(pair.client)
    }

    async fn alarm(&self) -> Result<Response> {
        self.run_alarm_work().await?;
        Response::empty()
    }

    async fn websocket_message(
        &self,
        ws: WebSocket,
        message: WebSocketIncomingMessage,
    ) -> Result<()> {
        let data = match message {
            WebSocketIncomingMessage::Binary(bytes) => bytes,
            WebSocketIncomingMessage::String(_) => return Ok(()),
        };

        self.ensure_inner()?;
        let phase: ConnPhase = ws
            .deserialize_attachment::<ConnPhase>()
            .unwrap_or(None)
            .unwrap_or(ConnPhase::Fresh);

        let compressed_input = matches!(
            phase,
            ConnPhase::AwaitingLoginAck { .. }
                | ConnPhase::ConfigKnownPacksSent { .. }
                | ConnPhase::ConfigFinishSent { .. }
                | ConnPhase::Playing(_)
        );
        let input_threshold = if compressed_input { THRESH } else { None };

        let Ok(payload) = decode_frame(&data, input_threshold) else {
            return Ok(());
        };

        match phase {
            ConnPhase::Fresh => {
                if let Err(_e) = decode_handshake(&payload) {
                    let _ = ws.close(Some(1000), Some("bad handshake"));
                } else if let Ok(hs) = decode_handshake(&payload) {
                    let next = if hs.next_state == 1 {
                        ConnPhase::Status
                    } else if hs.next_state == 2 {
                        if hs.protocol_version != net_ws::PROTOCOL_VERSION {
                            let kick =
                                net_ws::packets::login_disconnect("KiteCraft requires MC 26.2");
                            let _ = ws.send_with_bytes(encode_frame(&kick, None).as_slice());
                            let _ = ws.close(Some(1000), Some("wrong protocol version"));
                            return Ok(());
                        }
                        ConnPhase::AwaitingLoginStart
                    } else {
                        let _ = ws.close(Some(1000), Some("unsupported handshake"));
                        return Ok(());
                    };
                    ws.serialize_attachment(&next)?;
                }
            }

            ConnPhase::Status => {
                if let Ok(req) = decode_status(&payload) {
                    match req.ping_time {
                        None => {
                            let resp = status_response(&self.status_json());
                            let _ = ws.send_with_bytes(encode_frame(&resp, None).as_slice());
                        }
                        Some(t) => {
                            let pong = status_pong(t);
                            let _ = ws.send_with_bytes(encode_frame(&pong, None).as_slice());
                            let _ = ws.close(Some(1000), Some("done"));
                        }
                    }
                }
            }

            ConnPhase::AwaitingLoginStart => {
                if let Ok(start) = decode_login_start(&payload) {
                    if start.name.len() < 3 || start.name.len() > 16 {
                        let kick = net_ws::packets::login_disconnect("Invalid username length");
                        let _ = ws.send_with_bytes(encode_frame(&kick, None).as_slice());
                        let _ = ws.close(Some(1000), Some("invalid username"));
                        return Ok(());
                    }
                    if self.playing_sockets().len() >= MAX_PLAYERS as usize {
                        let kick = net_ws::packets::login_disconnect("Server full");
                        let _ = ws.send_with_bytes(encode_frame(&kick, THRESH).as_slice());
                        let _ = ws.close(Some(1000), Some("server full"));
                        return Ok(());
                    }
                    let uuid_hex = hex(&offline_uuid(&start.name));
                    let compress = net_ws::packets::login_compress(256);
                    let _ = ws.send_with_bytes(encode_frame(&compress, None).as_slice());
                    let session_id = *uuid::Uuid::new_v4().as_bytes();
                    let success = net_ws::packets::login_success(
                        &uuid_from_hex(&uuid_hex),
                        &start.name,
                        &session_id,
                    );
                    let _ = ws.send_with_bytes(encode_frame(&success, THRESH).as_slice());
                    ws.serialize_attachment(&ConnPhase::AwaitingLoginAck {
                        name: start.name,
                        uuid_hex,
                    })?;
                }
            }

            ConnPhase::AwaitingLoginAck { name, uuid_hex } => {
                let mut d = net_ws::Decoder::new(&payload);
                if d.varint() == Ok(sb::LOGIN_ACKNOWLEDGED) {
                    self.enter_configuration(&ws, &name, uuid_hex)?;
                }
            }

            ConnPhase::ConfigKnownPacksSent { name, uuid_hex } => {
                if let Ok(net_ws::packets::IncomingConfig::KnownPacks(_)) = decode_config(&payload)
                {
                    self.send_registries_and_finish(&ws)?;
                    ws.serialize_attachment(&ConnPhase::ConfigFinishSent { name, uuid_hex })?;
                }
            }

            ConnPhase::ConfigFinishSent { name, uuid_hex } => {
                if let Ok(net_ws::packets::IncomingConfig::Finish) = decode_config(&payload) {
                    self.enter_play(&ws, &name, uuid_hex)?;
                }
            }

            ConnPhase::Playing(pd) => {
                self.handle_playing_message(&ws, pd, &payload).await?;
            }
        }

        Ok(())
    }

    async fn websocket_close(
        &self,
        ws: WebSocket,
        _code: usize,
        _reason: String,
        _was_clean: bool,
    ) -> Result<()> {
        self.cleanup_connection(&ws).await
    }

    async fn websocket_error(&self, ws: WebSocket, _error: Error) -> Result<()> {
        self.cleanup_connection(&ws).await
    }
}

#[cfg(test)]
mod chunk_stream_tests {
    use super::*;
    use std::collections::HashSet;

    fn player_with_sent_chunks(sent_chunks: Vec<(i32, i32)>) -> PlayerData {
        PlayerData {
            name: "Streamer".to_string(),
            uuid_hex: "00000000000000000000000000000000".to_string(),
            x: 0.5,
            y: 64.0,
            z: 0.5,
            yaw: 0.0,
            pitch: 0.0,
            on_ground: true,
            sent_chunks,
            pending_chunks: Vec::new(),
            announced_chunk_center: Some((0, 0)),
            chunks_per_batch: DEFAULT_CHUNKS_PER_BATCH,
            awaiting_chunk_batch_ack: false,
            hotbar: Vec::new(),
            held_slot: 0,
            pending_keepalive: None,
            missed_keepalives: 0,
        }
    }

    #[test]
    fn desired_window_is_complete_unique_and_nearest_first() {
        let chunks = desired_chunks((3, -4));
        assert_eq!(chunks.len(), 17 * 17);
        assert_eq!(chunks.first(), Some(&(3, -4)));
        assert_eq!(
            chunks.iter().copied().collect::<HashSet<_>>().len(),
            chunks.len()
        );
        assert!(chunks
            .windows(2)
            .all(|pair| chebyshev(pair[0].0, pair[0].1, 3, -4)
                <= chebyshev(pair[1].0, pair[1].1, 3, -4)));
    }

    #[test]
    fn crossing_one_chunk_replaces_exactly_one_edge() {
        let mut player = player_with_sent_chunks(desired_chunks((0, 0)));
        let unload = reset_chunk_window(&mut player, (1, 0));

        assert_eq!(unload.len(), 17);
        assert!(unload
            .iter()
            .all(|&(x, z)| x == -8 && (-8..=8).contains(&z)));
        assert_eq!(player.pending_chunks.len(), 17);
        assert!(player
            .pending_chunks
            .iter()
            .all(|&(x, z)| x == 9 && (-8..=8).contains(&z)));
        assert_eq!(player.sent_chunks.len(), 16 * 17);
        assert_eq!(player.announced_chunk_center, Some((1, 0)));
    }

    #[test]
    fn client_batch_rate_is_sanitized_and_bounded() {
        assert_eq!(acknowledged_batch_size(f32::NAN), 1);
        assert_eq!(acknowledged_batch_size(-5.0), 1);
        assert_eq!(acknowledged_batch_size(3.9), 3);
        assert_eq!(acknowledged_batch_size(1000.0), MAX_CHUNKS_PER_BATCH);
    }
}
