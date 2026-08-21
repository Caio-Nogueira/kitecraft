mod chunk_wire;
mod storage;
mod vanilla_registries;

use net_ws::ids::sb;
use net_ws::nbt::Nbt;
use net_ws::packets::{
    decode_block_dig, decode_block_place, decode_chat_message, decode_config,
    decode_handshake, decode_held_item_slot, decode_login_start, decode_movement, decode_play_keep_alive,
    decode_play_ping_request, decode_set_creative_slot, decode_status, offset_by_direction,
    play_ack_digging, play_abilities, play_block_change, play_chunk_batch_finished,
    play_chunk_batch_start, play_custom_payload, play_entity_destroy, play_game_event,
    play_head_rotation, play_keep_alive, play_login, play_ping_response, play_player_info_add,
    play_player_remove, play_position, play_spawn_entity_player, play_spawn_position,
    play_sync_entity_position, play_system_chat, play_time_update, play_unload_chunk, status_pong,
    status_response, CreativeSlot, PlayerListEntry, Teleport, GAMEMODE_CREATIVE,
    GAME_EVENT_START_WAITING_FOR_CHUNKS,
};
use net_ws::{decode_frame, encode_frame};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use worker::*;
use world::World;

use crate::storage::SqliteBacking;

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
    time_base_age_ticks: i64,
    time_base_wall_ms: u64,
    alarm_ticks: u64,
    teleport_counter: i32,
}

#[durable_object]
pub struct MinecraftServer {
    state: State,
    inner: RefCell<Option<Inner>>,
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

impl MinecraftServer {
    fn ensure_inner(&self) -> Result<()> {
        if self.inner.borrow().is_some() {
            return Ok(());
        }
        let storage = self.state.storage();
        let backing = SqliteBacking::new(storage.sql());
        backing.init_schema()?;

        let now_ms = Date::now().as_millis();
        let (base_age, base_wall) = match backing.meta_pair("time_base") {
            Some(pair) => pair,
            None => {
                backing.set_meta_pair("time_base", 1000, now_ms as i64);
                (1000, now_ms as i64)
            }
        };

        self.inner.borrow_mut().replace(Inner {
            world: World::with_cache_size(backing, 256),
            time_base_age_ticks: base_age,
            time_base_wall_ms: base_wall as u64,
            alarm_ticks: 0,
            teleport_counter: 1,
        });
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
                        Some(vanilla_registries::REGISTRY_BLOB.get(e.offset..e.offset + e.len).unwrap_or(&[])),
                    )
                })
                .collect();
            Self::send_payload(ws, &net_ws::packets::cfg_registry_data(registry.id, &entries))?;
        }

        let mut brand = net_ws::Encoder::new();
        brand.string(BRAND);
        let brand_bytes = brand.into_bytes();
        Self::send_payload(ws, &play_custom_payload("minecraft:brand", &brand_bytes))?;
        Self::send_payload(ws, &net_ws::packets::cfg_finish())?;
        Ok(())
    }

    fn enter_play(&self, ws: &WebSocket, name: &str, uuid_hex: String) -> Result<()> {
        let uuid = uuid_from_hex(&uuid_hex);

        let saved = {
            let guard = self.inner.borrow();
            load_saved_position(guard.as_ref().expect("inner"), &uuid_hex)
        };
        let (px, py, pz, pyaw, ppitch) =
            saved.unwrap_or((SPAWN_X, SPAWN_Y, SPAWN_Z, SPAWN_YAW, 0.0));

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
            0,
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
            hotbar: Vec::new(),
            held_slot: 0,
            pending_keepalive: None,
            missed_keepalives: 0,
        };

        let batch_frames = self.build_chunk_batch(&mut pd);
        for frame in batch_frames {
            Self::send_payload(ws, &frame)?;
        }

        let join_text = format!("§e{name} joined the game");
        let join_chat =
            play_system_chat(&Nbt::compound(vec![("text", Nbt::str(join_text))]), false);
        for (other_ws, _) in self.playing_sockets() {
            let _ = Self::send_payload(&other_ws, &join_chat);
        }

        ws.serialize_attachment(ConnPhase::Playing(pd))?;
        Ok(())
    }

    fn next_teleport_id(&self) -> i32 {
        let mut guard = self.inner.borrow_mut();
        let inner = guard.as_mut().expect("inner");
        let id = inner.teleport_counter;
        inner.teleport_counter += 1;
        id
    }

    fn build_chunk_batch(&self, pd: &mut PlayerData) -> Vec<Vec<u8>> {
        let center = pd.chunk_pos();
        let needed: Vec<(i32, i32)> = (-VIEW_DISTANCE..=VIEW_DISTANCE)
            .flat_map(|dx| (-VIEW_DISTANCE..=VIEW_DISTANCE).map(move |dz| (center.0 + dx, center.1 + dz)))
            .filter(|c| !pd.sent_chunks.contains(c))
            .collect();

        let mut frames = vec![play_chunk_batch_start()];
        {
            let mut guard = self.inner.borrow_mut();
            let inner = guard.as_mut().expect("inner");
            for &(cx, cz) in &needed {
                let col = inner.world.chunk(cx, cz);
                frames.push(chunk_wire::encode_chunk_packet(cx, cz, col));
            }
        }
        frames.push(play_chunk_batch_finished(needed.len() as i32));
        pd.sent_chunks.extend(needed);
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
                    self.apply_movement(ws, pd, movement)?;
                }
            }
            sb::PLAY_CHAT_MESSAGE => {
                if let Ok(msg) = decode_chat_message(payload) {
                    self.handle_chat(&pd, &msg.message).await?;
                }
            }
            sb::PLAY_BLOCK_DIG => match decode_block_dig(payload) {
                Ok(dig) if dig.status == 0 => {
                    self.apply_block_change(dig.x, dig.y, dig.z, 0);
                    let change = play_block_change(dig.x, dig.y, dig.z, 0);
                    let ack = play_ack_digging(dig.sequence);
                    for (other_ws, _) in self.playing_sockets() {
                        let _ = Self::send_payload(&other_ws, &change);
                        let _ = Self::send_payload(&other_ws, &ack);
                    }
                }
                Ok(dig) => {
                    let ack = play_ack_digging(dig.sequence);
                    let _ = Self::send_payload(ws, &ack);
                }
                Err(_) => {}
            },
            sb::PLAY_BLOCK_PLACE => {
                let Ok(place) = decode_block_place(payload) else {
                    return Ok(());
                };
                let held_state = pd
                    .hotbar
                    .iter()
                    .find(|(slot, _)| *slot == pd.held_slot || *slot == pd.held_slot + 36)
                    .map(|(_, state)| *state);
                let (tx, ty, tz) =
                    offset_by_direction(place.x, place.y, place.z, place.direction);
                let mut placed = None;
                if let Some(state) = held_state {
                    if (world::MIN_Y..world::MIN_Y + world::WORLD_HEIGHT).contains(&ty) {
                        self.apply_block_change(tx, ty, tz, state);
                        placed = Some((tx, ty, tz, state));
                    }
                }
                let ack = play_ack_digging(place.sequence);
                let change = placed.map(|(x, y, z, s)| play_block_change(x, y, z, s));
                for (other_ws, _) in self.playing_sockets() {
                    let _ = Self::send_payload(&other_ws, &ack);
                    if let Some(f) = &change {
                        let _ = Self::send_payload(&other_ws, f);
                    }
                }
            }
            sb::PLAY_SET_CREATIVE_SLOT => {
                if let Ok(Some(CreativeSlot { slot, block_state, count })) =
                    decode_set_creative_slot(payload)
                {
                    pd.hotbar.retain(|(s, _)| *s != slot as u8);
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

    fn apply_movement(
        &self,
        ws: &WebSocket,
        mut pd: PlayerData,
        movement: net_ws::packets::Movement,
    ) -> Result<()> {
        match movement {
            net_ws::packets::Movement::Position { x, y, z, on_ground } => {
                pd.x = x;
                pd.y = y;
                pd.z = z;
                pd.on_ground = on_ground;
            }
            net_ws::packets::Movement::PositionLook { x, y, z, yaw, pitch, on_ground } => {
                pd.x = x;
                pd.y = y;
                pd.z = z;
                pd.yaw = yaw;
                pd.pitch = pitch;
                pd.on_ground = on_ground;
            }
            net_ws::packets::Movement::Look { yaw, pitch, on_ground } => {
                pd.yaw = yaw;
                pd.pitch = pitch;
                pd.on_ground = on_ground;
            }
            net_ws::packets::Movement::Flying { on_ground } => {
                pd.on_ground = on_ground;
            }
        }

        if pd.y < -70.0 {
            let tid = self.next_teleport_id();
            pd.x = SPAWN_X;
            pd.y = SPAWN_Y;
            pd.z = SPAWN_Z;
            let tp = play_position(&Teleport {
                id: tid,
                x: pd.x,
                y: pd.y,
                z: pd.z,
                yaw: pd.yaw,
                pitch: pd.pitch,
            });
            Self::send_payload(ws, &tp)?;
        }

        let center = pd.chunk_pos();
        let needs_streaming = pd.sent_chunks.is_empty()
            || pd
                .sent_chunks
                .iter()
                .any(|&(cx, cz)| chebyshev(cx, cz, center.0, center.1) > VIEW_DISTANCE + 1);

        if needs_streaming {
            let keep: Vec<(i32, i32)> = pd
                .sent_chunks
                .iter()
                .copied()
                .filter(|&(cx, cz)| chebyshev(cx, cz, center.0, center.1) <= VIEW_DISTANCE)
                .collect();
            let unload: Vec<Vec<u8>> = pd
                .sent_chunks
                .iter()
                .filter(|c| !keep.contains(*c))
                .map(|&(cx, cz)| play_unload_chunk(cx, cz))
                .collect();
            pd.sent_chunks = keep;
            let mut frames = unload;
            frames.extend(self.build_chunk_batch(&mut pd));
            for frame in frames {
                Self::send_payload(ws, &frame)?;
            }
        }
        for (other_ws, other_pd) in self.playing_sockets() {
            if other_pd.uuid_hex == pd.uuid_hex {
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
            let _ = Self::send_payload(&other_ws, &sync);
            let _ = Self::send_payload(&other_ws, &head);
        }

        ws.serialize_attachment(ConnPhase::Playing(pd))?;
        Ok(())
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
            inner.world.backing().set_meta_pair("time_base", target, now as i64);
        }
        let tick = Self::current_tick(self.inner.borrow().as_ref().expect("inner"), Date::now().as_millis());
        let frame = play_time_update(tick, tick.rem_euclid(24000));
        for (ws, _) in self.playing_sockets() {
            let _ = Self::send_payload(&ws, &frame);
        }
        Ok(())
    }

    fn apply_block_change(&self, x: i32, y: i32, z: i32, state: u16) {
        let mut guard = self.inner.borrow_mut();
        guard
            .as_mut()
            .expect("inner")
            .world
            .set_block(x, y, z, state);
    }

    fn broadcast_join_leave(&self, text: &str) {
        let frame =
            play_system_chat(&Nbt::compound(vec![("text", Nbt::str(text.to_string()))]), false);
        for (ws, _) in self.playing_sockets() {
            let _ = Self::send_payload(&ws, &frame);
        }
    }

    async fn cleanup_connection(&self, ws: &WebSocket) -> Result<()> {
        let phase: Option<ConnPhase> = ws.deserialize_attachment().ok().flatten();
        if let Some(ConnPhase::Playing(pd)) = phase {
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
        let time_frame = play_time_update(tick, tick.rem_euclid(24000));

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
    let blob = inner.world.backing().get_blob(&format!("player:{uuid_hex}"))?;
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
            inner: RefCell::new(None),
        }
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        let upgrade = req
            .headers()
            .get("Upgrade")?
            .map(|v| v.eq_ignore_ascii_case("websocket"))
            .unwrap_or(false);

        if !upgrade {
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

    async fn websocket_message(&self, ws: WebSocket, message: WebSocketIncomingMessage) -> Result<()> {
        let data = match message {
            WebSocketIncomingMessage::Binary(bytes) => bytes,
            WebSocketIncomingMessage::String(_) => return Ok(()),
        };

        self.ensure_inner()?;        let phase: ConnPhase = ws
            .deserialize_attachment::<ConnPhase>()
            .unwrap_or({
                None
            })
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
            ConnPhase::Fresh => if let Err(_e) = decode_handshake(&payload) {
                let _ = ws.close(Some(1000), Some("bad handshake"));
            } else if let Ok(hs) = decode_handshake(&payload) {
                let next = if hs.next_state == 1 {
                    ConnPhase::Status
                } else if hs.next_state == 2 {
                    if hs.protocol_version != net_ws::PROTOCOL_VERSION {
                        let kick =
                            net_ws::packets::login_disconnect("KiteCraft requires MC 1.21.4");
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
            },

            ConnPhase::Status => if let Ok(req) = decode_status(&payload) { match req.ping_time {
                None => {
                    let resp = status_response(&self.status_json());
                    let _ = ws.send_with_bytes(encode_frame(&resp, None).as_slice());
                }
                Some(t) => {
                    let pong = status_pong(t);
                    let _ = ws.send_with_bytes(encode_frame(&pong, None).as_slice());
                    let _ = ws.close(Some(1000), Some("done"));
                }
            } },

            ConnPhase::AwaitingLoginStart => if let Ok(start) = decode_login_start(&payload) {
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
                let success =
                    net_ws::packets::login_success(&uuid_from_hex(&uuid_hex), &start.name);
                let _ = ws.send_with_bytes(encode_frame(&success, THRESH).as_slice());
                ws.serialize_attachment(&ConnPhase::AwaitingLoginAck {
                    name: start.name,
                    uuid_hex,
                })?;
            },

            ConnPhase::AwaitingLoginAck { name, uuid_hex } => {
                let mut d = net_ws::Decoder::new(&payload);
                if d.varint() == Ok(sb::LOGIN_ACKNOWLEDGED) {
                    self.enter_configuration(&ws, &name, uuid_hex)?;
                }
            }

            ConnPhase::ConfigKnownPacksSent { name, uuid_hex } => {
                if let Ok(net_ws::packets::IncomingConfig::KnownPacks(_)) =
                    decode_config(&payload)
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

    async fn websocket_close(&self, ws: WebSocket, _code: usize, _reason: String, _was_clean: bool) -> Result<()> {
        self.cleanup_connection(&ws).await
    }

    async fn websocket_error(&self, ws: WebSocket, _error: Error) -> Result<()> {
        self.cleanup_connection(&ws).await
    }
}
