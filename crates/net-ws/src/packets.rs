use crate::buf::{angle_byte, pack_block_pos, Decoder, Encoder};
use crate::error::{DecodeError, DecodeResult};
use crate::ids::{cb, sb};
use crate::nbt::{encode_root_unnamed, Nbt};

pub const GAMEMODE_SURVIVAL: u8 = 0;
pub const GAMEMODE_CREATIVE: u8 = 1;

pub const DIG_START_DESTROY: i32 = 0;
pub const DIG_CANCEL: i32 = 1;
pub const DIG_FINISH: i32 = 2;

pub const ENTITY_TYPE_PLAYER: i32 = 147;

pub const GAME_EVENT_START_WAITING_FOR_CHUNKS: u8 = 13;

pub fn encode_packet(id: i32, body: impl FnOnce(&mut Encoder)) -> Vec<u8> {
    let mut e = Encoder::new();
    e.varint(id);
    body(&mut e);
    e.into_bytes()
}

pub fn status_response(json: &str) -> Vec<u8> {
    encode_packet(cb::STATUS_RESPONSE, |e| e.string(json))
}

pub fn status_pong(time: i64) -> Vec<u8> {
    encode_packet(cb::STATUS_PING, |e| e.i64(time))
}

pub fn login_disconnect(reason: &str) -> Vec<u8> {
    let nbt = encode_root_unnamed(&crate::text::text_compound(reason));
    encode_packet(cb::LOGIN_DISCONNECT, |e| e.raw(&nbt))
}

pub fn login_compress(threshold: i32) -> Vec<u8> {
    encode_packet(cb::LOGIN_COMPRESS, |e| e.varint(threshold))
}

pub fn login_success(uuid: &[u8; 16], name: &str, session_id: &[u8; 16]) -> Vec<u8> {
    encode_packet(cb::LOGIN_SUCCESS, |e| {
        e.uuid(uuid);
        e.string(name);
        e.varint(0);
        // Added in 26.2. This identifies the individual login session rather
        // than the player's persistent game profile.
        e.uuid(session_id);
    })
}

pub fn cfg_known_packs() -> Vec<u8> {
    encode_packet(cb::CFG_KNOWN_PACKS, |e| {
        e.varint(1);
        e.string("minecraft");
        e.string("core");
        e.string(crate::MINECRAFT_VERSION);
    })
}

pub fn cfg_registry_data(registry: &str, entries: &[(&str, Option<&[u8]>)]) -> Vec<u8> {
    encode_packet(cb::CFG_REGISTRY_DATA, |e| {
        e.string(registry);
        e.varint(entries.len() as i32);
        for (name, data) in entries {
            e.string(name);
            match data {
                Some(bytes) => {
                    e.bool(true);
                    e.raw(bytes);
                }
                None => e.bool(false),
            }
        }
    })
}

pub fn cfg_tags(encoded_tags: &[u8]) -> Vec<u8> {
    encode_packet(cb::CFG_TAGS, |e| e.raw(encoded_tags))
}

pub fn cfg_custom_payload(channel: &str, data: &[u8]) -> Vec<u8> {
    encode_packet(cb::CFG_CUSTOM_PAYLOAD, |e| {
        e.string(channel);
        e.raw(data);
    })
}

pub fn cfg_finish() -> Vec<u8> {
    encode_packet(cb::CFG_FINISH, |_| {})
}

#[allow(clippy::too_many_arguments)]
pub fn play_login(
    entity_id: i32,
    is_hardcore: bool,
    world_names: &[&str],
    max_players: i32,
    view_distance: i32,
    simulation_distance: i32,
    dimension_index: i32,
    world_name: &str,
    hashed_seed: i64,
    gamemode: u8,
    is_flat: bool,
    sea_level: i32,
) -> Vec<u8> {
    encode_packet(cb::PLAY_LOGIN, |e| {
        e.i32(entity_id);
        e.bool(is_hardcore);
        e.varint(world_names.len() as i32);
        for name in world_names {
            e.string(name);
        }
        e.varint(max_players);
        e.varint(view_distance);
        e.varint(simulation_distance);
        e.bool(false);
        e.bool(true);
        e.bool(false);
        e.varint(dimension_index);
        e.string(world_name);
        e.i64(hashed_seed);
        e.u8(gamemode);
        e.u8(gamemode);
        e.bool(false);
        e.bool(is_flat);
        e.bool(false);
        e.varint(0);
        e.varint(sea_level);
        // Added in 26.2. KiteCraft is offline-mode at the Minecraft layer.
        e.bool(false);
        // Secure chat is not enforced.
        e.bool(false);
    })
}

pub fn play_game_event(reason: u8, value: f32) -> Vec<u8> {
    encode_packet(cb::PLAY_GAME_EVENT, |e| {
        e.u8(reason);
        e.f32(value);
    })
}

pub fn play_spawn_position(x: i32, y: i32, z: i32) -> Vec<u8> {
    encode_packet(cb::PLAY_SPAWN_POSITION, |e| {
        e.string("minecraft:overworld");
        e.i64(pack_block_pos(x, y, z));
        e.f32(0.0);
        e.f32(0.0);
    })
}

pub struct Teleport {
    pub id: i32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
}

pub fn play_position(t: &Teleport) -> Vec<u8> {
    encode_packet(cb::PLAY_POSITION, |e| {
        e.varint(t.id);
        e.f64(t.x);
        e.f64(t.y);
        e.f64(t.z);
        e.f64(0.0);
        e.f64(0.0);
        e.f64(0.0);
        e.f32(t.yaw);
        e.f32(t.pitch);
        e.u32(0);
    })
}

pub fn play_keep_alive(id: i64) -> Vec<u8> {
    encode_packet(cb::PLAY_KEEP_ALIVE, |e| e.i64(id))
}

pub fn play_time_update(age: i64, time_of_day: i64, clock_id: i32) -> Vec<u8> {
    encode_packet(cb::PLAY_UPDATE_TIME, |e| {
        e.i64(age);
        // Since 26.1, time is a list of synchronized clock updates.
        e.varint(1);
        e.varint(clock_id);
        e.varlong(time_of_day);
        e.f32(0.0);
        e.f32(1.0);
    })
}

pub fn play_chunk_batch_start() -> Vec<u8> {
    encode_packet(cb::PLAY_CHUNK_BATCH_START, |_| {})
}

pub fn play_chunk_batch_finished(batch_size: i32) -> Vec<u8> {
    encode_packet(cb::PLAY_CHUNK_BATCH_FINISHED, |e| e.varint(batch_size))
}

pub fn play_set_chunk_cache_center(chunk_x: i32, chunk_z: i32) -> Vec<u8> {
    encode_packet(cb::PLAY_SET_CHUNK_CACHE_CENTER, |e| {
        e.varint(chunk_x);
        e.varint(chunk_z);
    })
}

pub struct LightPayload {
    pub sky_light_mask: Vec<i64>,
    pub block_light_mask: Vec<i64>,
    pub empty_sky_light_mask: Vec<i64>,
    pub empty_block_light_mask: Vec<i64>,
    pub sky_light: Vec<Vec<u8>>,
    pub block_light: Vec<Vec<u8>>,
}

pub struct HeightmapPayload {
    pub id: i32,
    pub data: Vec<i64>,
}

pub fn play_chunk_data(
    x: i32,
    z: i32,
    heightmaps: &[HeightmapPayload],
    chunk_data: &[u8],
    light: &LightPayload,
) -> Vec<u8> {
    encode_packet(cb::PLAY_CHUNK_DATA, |e| {
        e.i32(x);
        e.i32(z);
        // Heightmaps became a registry-indexed map in 1.21.5.
        e.varint(heightmaps.len() as i32);
        for heightmap in heightmaps {
            e.varint(heightmap.id);
            e.varint(heightmap.data.len() as i32);
            for value in &heightmap.data {
                e.i64(*value);
            }
        }
        e.byte_array(chunk_data);
        e.varint(0);
        for mask in [
            &light.sky_light_mask,
            &light.block_light_mask,
            &light.empty_sky_light_mask,
            &light.empty_block_light_mask,
        ] {
            e.varint(mask.len() as i32);
            for v in mask.iter() {
                e.i64(*v);
            }
        }
        for arrays in [&light.sky_light, &light.block_light] {
            e.varint(arrays.len() as i32);
            for arr in arrays {
                e.byte_array(arr);
            }
        }
    })
}

pub fn play_block_change(x: i32, y: i32, z: i32, state: u16) -> Vec<u8> {
    encode_packet(cb::PLAY_BLOCK_CHANGE, |e| {
        e.i64(pack_block_pos(x, y, z));
        e.varint(state as i32);
    })
}

pub fn play_ack_digging(sequence: i32) -> Vec<u8> {
    encode_packet(cb::PLAY_ACK_DIGGING, |e| e.varint(sequence))
}

pub fn play_system_chat(content: &Nbt, action_bar: bool) -> Vec<u8> {
    encode_packet(cb::PLAY_SYSTEM_CHAT, |e| {
        e.raw(&encode_root_unnamed(content));
        e.bool(action_bar);
    })
}

pub fn play_kick(reason: &str) -> Vec<u8> {
    let nbt = encode_root_unnamed(&crate::text::text_compound(reason));
    encode_packet(cb::PLAY_KICK_DISCONNECT, |e| e.raw(&nbt))
}

pub fn play_ping_response(id: i64) -> Vec<u8> {
    encode_packet(cb::PLAY_PING_RESPONSE, |e| e.i64(id))
}

pub fn play_attack_animation(entity_id: i32) -> Vec<u8> {
    encode_packet(cb::PLAY_ANIMATION, |e| {
        e.varint(entity_id);
        e.u8(0);
    })
}

pub fn play_damage_event(
    target_entity_id: i32,
    attacker_entity_id: i32,
    damage_type_id: i32,
) -> Vec<u8> {
    encode_packet(cb::PLAY_DAMAGE_EVENT, |e| {
        e.varint(target_entity_id);
        e.varint(damage_type_id);
        e.varint(attacker_entity_id + 1);
        e.varint(attacker_entity_id + 1);
        e.bool(false);
    })
}

pub fn play_hurt_animation(entity_id: i32, yaw: f32) -> Vec<u8> {
    encode_packet(cb::PLAY_HURT_ANIMATION, |e| {
        e.varint(entity_id);
        e.f32(yaw);
    })
}

pub fn play_set_health(health: f32) -> Vec<u8> {
    encode_packet(cb::PLAY_SET_HEALTH, |e| {
        e.f32(health);
        e.varint(20);
        e.f32(5.0);
    })
}

pub fn play_set_entity_motion(entity_id: i32, x: f64, y: f64, z: f64) -> Vec<u8> {
    encode_packet(cb::PLAY_SET_ENTITY_MOTION, |e| {
        e.varint(entity_id);
        write_packed_velocity(e, x, y, z);
    })
}

fn write_packed_velocity(e: &mut Encoder, x: f64, y: f64, z: f64) {
    const MAX_VELOCITY: f64 = 1.717_986_918_3E10;
    const MIN_VELOCITY: f64 = 3.051_944_088_384_301E-5;
    const QUANTIZED_MAX: f64 = 32_766.0;

    let clamp = |value: f64| {
        if value.is_nan() {
            0.0
        } else {
            value.clamp(-MAX_VELOCITY, MAX_VELOCITY)
        }
    };
    let [x, y, z] = [clamp(x), clamp(y), clamp(z)];
    let maximum = x.abs().max(y.abs()).max(z.abs());
    if maximum < MIN_VELOCITY {
        e.u8(0);
        return;
    }

    let scale = maximum.ceil() as i64;
    let extended = scale > 3;
    let header = if extended { (scale & 3) | 4 } else { scale };
    let quantize = |value: f64| {
        (((value.mul_add(0.5 / scale as f64, 0.5)) * QUANTIZED_MAX).round() as i64).clamp(0, 32_766)
    };
    let packed = header | (quantize(x) << 3) | (quantize(y) << 18) | (quantize(z) << 33);
    e.raw(&(packed as u16).to_le_bytes());
    e.raw(&((packed >> 16) as i32).to_be_bytes());
    if extended {
        e.varint((scale >> 2) as i32);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn play_spawn_entity_player(
    entity_id: i32,
    uuid: &[u8; 16],
    x: f64,
    y: f64,
    z: f64,
    yaw: f32,
    pitch: f32,
) -> Vec<u8> {
    encode_packet(cb::PLAY_SPAWN_ENTITY, |e| {
        e.varint(entity_id);
        e.uuid(uuid);
        e.varint(ENTITY_TYPE_PLAYER);
        e.f64(x);
        e.f64(y);
        e.f64(z);
        // 1.21.9+ moved packed velocity before rotation.
        write_packed_velocity(e, 0.0, 0.0, 0.0);
        e.i8(angle_byte(pitch));
        e.i8(angle_byte(yaw));
        e.i8(angle_byte(yaw));
        e.varint(0);
    })
}

pub fn play_entity_destroy(entity_ids: &[i32]) -> Vec<u8> {
    encode_packet(cb::PLAY_ENTITY_DESTROY, |e| {
        e.varint(entity_ids.len() as i32);
        for id in entity_ids {
            e.varint(*id);
        }
    })
}

pub fn play_entity_teleport(
    entity_id: i32,
    x: f64,
    y: f64,
    z: f64,
    yaw: f32,
    pitch: f32,
    on_ground: bool,
) -> Vec<u8> {
    encode_packet(cb::PLAY_ENTITY_TELEPORT, |e| {
        e.varint(entity_id);
        e.f64(x);
        e.f64(y);
        e.f64(z);
        e.f64(0.0);
        e.f64(0.0);
        e.f64(0.0);
        e.f32(yaw);
        e.f32(pitch);
        e.u32(0);
        e.bool(on_ground);
    })
}

pub fn play_sync_entity_position(
    entity_id: i32,
    x: f64,
    y: f64,
    z: f64,
    yaw: f32,
    pitch: f32,
    on_ground: bool,
) -> Vec<u8> {
    encode_packet(cb::PLAY_SYNC_ENTITY_POSITION, |e| {
        e.varint(entity_id);
        e.f64(x);
        e.f64(y);
        e.f64(z);
        e.f64(0.0);
        e.f64(0.0);
        e.f64(0.0);
        e.f32(yaw);
        e.f32(pitch);
        e.bool(on_ground);
    })
}

pub fn play_entity_look(entity_id: i32, yaw: f32, pitch: f32, on_ground: bool) -> Vec<u8> {
    encode_packet(cb::PLAY_ENTITY_LOOK, |e| {
        e.varint(entity_id);
        e.i8(angle_byte(yaw));
        e.i8(angle_byte(pitch));
        e.bool(on_ground);
    })
}

pub fn play_head_rotation(entity_id: i32, yaw: f32) -> Vec<u8> {
    encode_packet(cb::PLAY_ENTITY_HEAD_ROTATION, |e| {
        e.varint(entity_id);
        e.i8(angle_byte(yaw));
    })
}

pub fn play_player_remove(uuids: &[[u8; 16]]) -> Vec<u8> {
    encode_packet(cb::PLAY_PLAYER_REMOVE, |e| {
        e.varint(uuids.len() as i32);
        for uuid in uuids {
            e.uuid(uuid);
        }
    })
}

pub struct PlayerListEntry {
    pub uuid: [u8; 16],
    pub name: String,
    pub gamemode: u8,
    pub listed: bool,
    pub ping_ms: i32,
}

const ACTION_ADD_PLAYER: u8 = 0x01;
const ACTION_UPDATE_GAMEMODE: u8 = 0x04;
const ACTION_UPDATE_LISTED: u8 = 0x08;
const ACTION_UPDATE_LATENCY: u8 = 0x10;

pub fn play_player_info_add(entry: &PlayerListEntry) -> Vec<u8> {
    player_info(
        ACTION_ADD_PLAYER | ACTION_UPDATE_GAMEMODE | ACTION_UPDATE_LISTED,
        std::slice::from_ref(entry),
    )
}

pub fn play_player_info_update_gamemode_and_listed(entries: &[PlayerListEntry]) -> Vec<u8> {
    player_info(ACTION_UPDATE_GAMEMODE | ACTION_UPDATE_LISTED, entries)
}

fn player_info(action: u8, entries: &[PlayerListEntry]) -> Vec<u8> {
    encode_packet(cb::PLAY_PLAYER_INFO, |e| {
        e.u8(action);
        e.varint(entries.len() as i32);
        for entry in entries {
            e.uuid(&entry.uuid);
            if action & ACTION_ADD_PLAYER != 0 {
                e.string(&entry.name);
                e.varint(0);
            }
            if action & ACTION_UPDATE_GAMEMODE != 0 {
                e.varint(entry.gamemode as i32);
            }
            if action & ACTION_UPDATE_LISTED != 0 {
                e.bool(entry.listed);
            }
            if action & ACTION_UPDATE_LATENCY != 0 {
                e.varint(entry.ping_ms);
            }
        }
    })
}

pub fn play_abilities(flags: u8) -> Vec<u8> {
    encode_packet(cb::PLAY_ABILITIES, |e| {
        e.u8(flags);
        e.f32(0.05);
        e.f32(0.1);
    })
}

pub fn play_custom_payload(channel: &str, data: &[u8]) -> Vec<u8> {
    encode_packet(cb::PLAY_CUSTOM_PAYLOAD, |e| {
        e.string(channel);
        e.raw(data);
    })
}

pub fn play_unload_chunk(x: i32, z: i32) -> Vec<u8> {
    encode_packet(cb::PLAY_UNLOAD_CHUNK, |e| {
        e.i32(z);
        e.i32(x);
    })
}

pub struct IncomingHandshake {
    pub protocol_version: i32,
    pub server_host: String,
    pub server_port: u16,
    pub next_state: i32,
}

pub fn decode_handshake(payload: &[u8]) -> DecodeResult<IncomingHandshake> {
    let mut d = Decoder::new(payload);
    let id = d.varint()?;
    if id != sb::HANDSHAKE {
        return Err(DecodeError::UnknownPacket {
            state: "handshake",
            id,
        });
    }
    Ok(IncomingHandshake {
        protocol_version: d.varint()?,
        server_host: d.string()?,
        server_port: d.u16()?,
        next_state: d.varint()?,
    })
}

pub struct IncomingStatusRequest {
    pub ping_time: Option<i64>,
}

pub fn decode_status(payload: &[u8]) -> DecodeResult<IncomingStatusRequest> {
    let mut d = Decoder::new(payload);
    match d.varint()? {
        sb::STATUS_REQUEST => Ok(IncomingStatusRequest { ping_time: None }),
        sb::STATUS_PING => Ok(IncomingStatusRequest {
            ping_time: Some(d.i64()?),
        }),
        other => Err(DecodeError::UnknownPacket {
            state: "status",
            id: other,
        }),
    }
}

pub struct IncomingLoginStart {
    pub name: String,
    pub uuid: [u8; 16],
}

pub fn decode_login_start(payload: &[u8]) -> DecodeResult<IncomingLoginStart> {
    let mut d = Decoder::new(payload);
    let id = d.varint()?;
    if id != sb::LOGIN_START {
        return Err(DecodeError::UnknownPacket { state: "login", id });
    }
    Ok(IncomingLoginStart {
        name: d.string()?,
        uuid: d.uuid()?,
    })
}

pub enum IncomingConfig {
    Finish,
    KnownPacks(Vec<(String, String, String)>),
    Other,
}

pub fn decode_config(payload: &[u8]) -> DecodeResult<IncomingConfig> {
    let mut d = Decoder::new(payload);
    match d.varint()? {
        sb::CFG_FINISH => Ok(IncomingConfig::Finish),
        sb::CFG_KNOWN_PACKS => {
            let count = d.varint()?;
            let mut packs = Vec::with_capacity(count.max(0) as usize);
            for _ in 0..count {
                packs.push((d.string()?, d.string()?, d.string()?));
            }
            Ok(IncomingConfig::KnownPacks(packs))
        }
        _ => Ok(IncomingConfig::Other),
    }
}

pub enum Movement {
    Position {
        x: f64,
        y: f64,
        z: f64,
        on_ground: bool,
    },
    PositionLook {
        x: f64,
        y: f64,
        z: f64,
        yaw: f32,
        pitch: f32,
        on_ground: bool,
    },
    Look {
        yaw: f32,
        pitch: f32,
        on_ground: bool,
    },
    Flying {
        on_ground: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IncomingAttack {
    pub entity_id: i32,
}

pub fn decode_attack(payload: &[u8]) -> DecodeResult<IncomingAttack> {
    let mut d = Decoder::new(payload);
    let id = d.varint()?;
    if id != sb::PLAY_ATTACK {
        return Err(DecodeError::UnknownPacket { state: "play", id });
    }
    let attack = IncomingAttack {
        entity_id: d.varint()?,
    };
    if !d.is_empty() {
        return Err(DecodeError::Malformed("trailing attack packet data"));
    }
    Ok(attack)
}

pub fn decode_movement(payload: &[u8]) -> DecodeResult<Movement> {
    let mut d = Decoder::new(payload);
    match d.varint()? {
        sb::PLAY_POSITION => Ok(Movement::Position {
            x: d.f64()?,
            y: d.f64()?,
            z: d.f64()?,
            on_ground: d.u8()? & 0x01 != 0,
        }),
        sb::PLAY_POSITION_LOOK => Ok(Movement::PositionLook {
            x: d.f64()?,
            y: d.f64()?,
            z: d.f64()?,
            yaw: d.f32()?,
            pitch: d.f32()?,
            on_ground: d.u8()? & 0x01 != 0,
        }),
        sb::PLAY_LOOK => Ok(Movement::Look {
            yaw: d.f32()?,
            pitch: d.f32()?,
            on_ground: d.bool()?,
        }),
        sb::PLAY_FLYING => Ok(Movement::Flying {
            on_ground: d.bool()?,
        }),
        other => Err(DecodeError::UnknownPacket {
            state: "play",
            id: other,
        }),
    }
}

pub struct IncomingBlockDig {
    pub status: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub sequence: i32,
}

pub fn decode_block_dig(payload: &[u8]) -> DecodeResult<IncomingBlockDig> {
    use crate::buf::unpack_block_pos;
    let mut d = Decoder::new(payload);
    let id = d.varint()?;
    if id != sb::PLAY_BLOCK_DIG {
        return Err(DecodeError::UnknownPacket { state: "play", id });
    }
    let status = d.varint()?;
    let pos = d.i64()?;
    let _face = d.i8()?;
    let sequence = d.varint()?;
    let (x, y, z) = unpack_block_pos(pos);
    Ok(IncomingBlockDig {
        status,
        x,
        y,
        z,
        sequence,
    })
}

pub struct IncomingBlockPlace {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub direction: i32,
    pub cursor_x: f32,
    pub cursor_y: f32,
    pub cursor_z: f32,
    pub sequence: i32,
}

pub fn decode_block_place(payload: &[u8]) -> DecodeResult<IncomingBlockPlace> {
    use crate::buf::unpack_block_pos;
    let mut d = Decoder::new(payload);
    let id = d.varint()?;
    if id != sb::PLAY_BLOCK_PLACE {
        return Err(DecodeError::UnknownPacket { state: "play", id });
    }
    let _hand = d.varint()?;
    let pos = d.i64()?;
    let direction = d.varint()?;
    let cursor_x = d.f32()?;
    let cursor_y = d.f32()?;
    let cursor_z = d.f32()?;
    let _inside_block = d.bool()?;
    let _world_border_hit = d.bool()?;
    let sequence = d.varint()?;
    let (x, y, z) = unpack_block_pos(pos);
    Ok(IncomingBlockPlace {
        x,
        y,
        z,
        direction,
        cursor_x,
        cursor_y,
        cursor_z,
        sequence,
    })
}

pub fn offset_by_direction(x: i32, y: i32, z: i32, direction: i32) -> (i32, i32, i32) {
    let nx = match direction {
        2 => -1,
        3 => 1,
        _ => 0,
    };
    let ny = match direction {
        0 => -1,
        1 => 1,
        _ => 0,
    };
    let nz = match direction {
        4 => -1,
        5 => 1,
        _ => 0,
    };
    (x + nx, y + ny, z + nz)
}

pub struct IncomingChatMessage {
    pub message: String,
}

pub fn decode_chat_message(payload: &[u8]) -> DecodeResult<IncomingChatMessage> {
    let mut d = Decoder::new(payload);
    let id = d.varint()?;
    if id != sb::PLAY_CHAT_MESSAGE {
        return Err(DecodeError::UnknownPacket { state: "play", id });
    }
    let message = d.string()?;
    let _timestamp = d.i64()?;
    let _salt = d.i64()?;
    if d.bool()? {
        d.take(256)?;
    }
    let _message_count = d.varint()?;
    d.take(3)?;
    let _checksum = d.u8()?;
    if !d.is_empty() {
        return Err(DecodeError::Malformed("trailing chat packet data"));
    }
    Ok(IncomingChatMessage { message })
}

pub fn decode_play_keep_alive(payload: &[u8]) -> DecodeResult<i64> {
    let mut d = Decoder::new(payload);
    let id = d.varint()?;
    if id != sb::PLAY_KEEP_ALIVE {
        return Err(DecodeError::UnknownPacket { state: "play", id });
    }
    d.i64()
}

pub fn decode_play_ping_request(payload: &[u8]) -> DecodeResult<i64> {
    let mut d = Decoder::new(payload);
    let id = d.varint()?;
    if id != sb::PLAY_PING_REQUEST {
        return Err(DecodeError::UnknownPacket { state: "play", id });
    }
    d.i64()
}

pub fn decode_chunk_batch_received(payload: &[u8]) -> DecodeResult<f32> {
    let mut d = Decoder::new(payload);
    let id = d.varint()?;
    if id != sb::PLAY_CHUNK_BATCH_RECEIVED {
        return Err(DecodeError::UnknownPacket { state: "play", id });
    }
    let chunks_per_tick = d.f32()?;
    if !d.is_empty() {
        return Err(DecodeError::Malformed("trailing chunk batch data"));
    }
    Ok(chunks_per_tick)
}

#[derive(Debug, Eq, PartialEq)]
pub struct CreativeSlot {
    pub slot: i16,
    pub item_id: Option<u16>,
    pub count: i32,
}

pub fn decode_set_creative_slot(payload: &[u8]) -> DecodeResult<Option<CreativeSlot>> {
    let mut d = Decoder::new(payload);
    let id = d.varint()?;
    if id != sb::PLAY_SET_CREATIVE_SLOT {
        return Err(DecodeError::UnknownPacket { state: "play", id });
    }
    let slot = d.i16()?;
    let count = d.varint()?;
    if count == 0 {
        return Ok(Some(CreativeSlot {
            slot,
            item_id: None,
            count: 0,
        }));
    }
    let item_id = u16::try_from(d.varint()?).ok();
    let added_components = d.varint()?;
    let removed_components = d.varint()?;
    if added_components != 0 || removed_components != 0 {
        return Ok(None);
    }
    Ok(Some(CreativeSlot {
        slot,
        item_id,
        count,
    }))
}

pub fn decode_held_item_slot(payload: &[u8]) -> DecodeResult<Option<i16>> {
    let mut d = Decoder::new(payload);
    let id = d.varint()?;
    if id != sb::PLAY_HELD_ITEM_SLOT {
        return Err(DecodeError::UnknownPacket { state: "play", id });
    }
    Ok(Some(d.i16()?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creative_slot_decoder_keeps_protocol_item_id_raw() {
        let packet = encode_packet(sb::PLAY_SET_CREATIVE_SLOT, |e| {
            e.i16(36);
            e.varint(1);
            e.varint(55);
            e.varint(0);
            e.varint(0);
        });

        assert_eq!(
            decode_set_creative_slot(&packet).unwrap(),
            Some(CreativeSlot {
                slot: 36,
                item_id: Some(55),
                count: 1,
            })
        );
    }

    #[test]
    fn minecraft_26_2_attack_packet_contains_only_the_target() {
        let packet = encode_packet(sb::PLAY_ATTACK, |e| e.varint(1234));
        assert_eq!(
            decode_attack(&packet),
            Ok(IncomingAttack { entity_id: 1234 })
        );
    }

    #[test]
    fn combat_packets_use_pumpkin_26_2_layouts() {
        let damage = play_damage_event(12, 34, 34);
        let mut d = Decoder::new(&damage);
        assert_eq!(d.varint(), Ok(cb::PLAY_DAMAGE_EVENT));
        assert_eq!(d.varint(), Ok(12));
        assert_eq!(d.varint(), Ok(34));
        assert_eq!(d.varint(), Ok(35));
        assert_eq!(d.varint(), Ok(35));
        assert_eq!(d.bool(), Ok(false));
        assert!(d.is_empty());

        let health = play_set_health(19.0);
        let mut d = Decoder::new(&health);
        assert_eq!(d.varint(), Ok(cb::PLAY_SET_HEALTH));
        assert_eq!(d.f32(), Ok(19.0));
        assert_eq!(d.varint(), Ok(20));
        assert_eq!(d.f32(), Ok(5.0));
        assert!(d.is_empty());

        let velocity = play_set_entity_motion(12, 0.5, 0.4, 0.5);
        let mut d = Decoder::new(&velocity);
        assert_eq!(d.varint(), Ok(cb::PLAY_SET_ENTITY_MOTION));
        assert_eq!(d.varint(), Ok(12));
        assert_eq!(d.remaining(), 6);
    }

    #[test]
    fn movement_position_packets_accept_26_2_collision_flags() {
        let packet = encode_packet(sb::PLAY_POSITION_LOOK, |e| {
            e.f64(1.0);
            e.f64(2.0);
            e.f64(3.0);
            e.f32(90.0);
            e.f32(15.0);
            e.u8(0x03);
        });
        match decode_movement(&packet).unwrap() {
            Movement::PositionLook { on_ground, .. } => assert!(on_ground),
            _ => panic!("wrong movement variant"),
        }
    }

    #[test]
    fn chat_decoder_consumes_the_26_2_fingerprint_layout() {
        let packet = encode_packet(sb::PLAY_CHAT_MESSAGE, |e| {
            e.string("hello");
            e.i64(123);
            e.i64(456);
            e.bool(false);
            e.varint(0);
            e.raw(&[0; 3]);
            e.u8(0);
        });
        assert_eq!(decode_chat_message(&packet).unwrap().message, "hello");

        assert!(matches!(
            decode_chat_message(&packet[..packet.len() - 1]),
            Err(DecodeError::Eof)
        ));
    }

    #[test]
    fn changed_26_2_clientbound_layouts_are_complete() {
        let login = login_success(&[1; 16], "Player", &[2; 16]);
        let mut d = Decoder::new(&login);
        assert_eq!(d.varint(), Ok(cb::LOGIN_SUCCESS));
        assert_eq!(d.uuid(), Ok([1; 16]));
        assert_eq!(d.string().as_deref(), Ok("Player"));
        assert_eq!(d.varint(), Ok(0));
        assert_eq!(d.uuid(), Ok([2; 16]));
        assert!(d.is_empty());

        let tags = cfg_tags(&[0]);
        let mut d = Decoder::new(&tags);
        assert_eq!(d.varint(), Ok(cb::CFG_TAGS));
        assert_eq!(d.varint(), Ok(0));
        assert!(d.is_empty());

        let spawn = play_spawn_entity_player(7, &[3; 16], 1.0, 2.0, 3.0, 90.0, 10.0);
        let mut d = Decoder::new(&spawn);
        assert_eq!(d.varint(), Ok(cb::PLAY_SPAWN_ENTITY));
        assert_eq!(d.varint(), Ok(7));
        assert_eq!(d.uuid(), Ok([3; 16]));
        assert_eq!(d.varint(), Ok(ENTITY_TYPE_PLAYER));
        assert_eq!(
            [d.f64().unwrap(), d.f64().unwrap(), d.f64().unwrap()],
            [1.0, 2.0, 3.0]
        );
        assert_eq!(d.u8(), Ok(0)); // packed zero velocity
        d.take(3).unwrap(); // pitch, yaw, head yaw
        assert_eq!(d.varint(), Ok(0));
        assert!(d.is_empty());

        let spawn_position = play_spawn_position(1, 2, 3);
        let mut d = Decoder::new(&spawn_position);
        assert_eq!(d.varint(), Ok(cb::PLAY_SPAWN_POSITION));
        assert_eq!(d.string().as_deref(), Ok("minecraft:overworld"));
        assert_eq!(d.i64(), Ok(pack_block_pos(1, 2, 3)));
        assert_eq!(d.f32(), Ok(0.0));
        assert_eq!(d.f32(), Ok(0.0));
        assert!(d.is_empty());

        let time = play_time_update(100, 6000, 0);
        let mut d = Decoder::new(&time);
        assert_eq!(d.varint(), Ok(cb::PLAY_UPDATE_TIME));
        assert_eq!(d.i64(), Ok(100));
        assert_eq!(d.varint(), Ok(1));
        assert_eq!(d.varint(), Ok(0));
        assert_eq!(d.varlong(), Ok(6000));
        assert_eq!(d.f32(), Ok(0.0));
        assert_eq!(d.f32(), Ok(1.0));
        assert!(d.is_empty());
    }

    #[test]
    fn chunk_streaming_packets_match_pumpkin_26_2_layouts() {
        let center = play_set_chunk_cache_center(-12, 34);
        let mut d = Decoder::new(&center);
        assert_eq!(d.varint(), Ok(cb::PLAY_SET_CHUNK_CACHE_CENTER));
        assert_eq!(d.varint(), Ok(-12));
        assert_eq!(d.varint(), Ok(34));
        assert!(d.is_empty());

        let acknowledgement = encode_packet(sb::PLAY_CHUNK_BATCH_RECEIVED, |e| e.f32(7.5));
        assert_eq!(decode_chunk_batch_received(&acknowledgement), Ok(7.5));
        assert!(
            decode_chunk_batch_received(&acknowledgement[..acknowledgement.len() - 1]).is_err()
        );
    }
}
