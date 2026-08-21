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

pub fn login_success(uuid: &[u8; 16], name: &str) -> Vec<u8> {
    encode_packet(cb::LOGIN_SUCCESS, |e| {
        e.uuid(uuid);
        e.string(name);
        e.varint(0);
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
        e.i64(pack_block_pos(x, y, z));
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

pub fn play_time_update(age: i64, time_of_day: i64) -> Vec<u8> {
    encode_packet(cb::PLAY_UPDATE_TIME, |e| {
        e.i64(age);
        e.i64(time_of_day);
        e.bool(true);
    })
}

pub fn play_chunk_batch_start() -> Vec<u8> {
    encode_packet(cb::PLAY_CHUNK_BATCH_START, |_| {})
}

pub fn play_chunk_batch_finished(batch_size: i32) -> Vec<u8> {
    encode_packet(cb::PLAY_CHUNK_BATCH_FINISHED, |e| e.varint(batch_size))
}

pub struct LightPayload {
    pub sky_light_mask: Vec<i64>,
    pub block_light_mask: Vec<i64>,
    pub empty_sky_light_mask: Vec<i64>,
    pub empty_block_light_mask: Vec<i64>,
    pub sky_light: Vec<Vec<u8>>,
    pub block_light: Vec<Vec<u8>>,
}

pub fn play_chunk_data(
    x: i32,
    z: i32,
    heightmaps: &Nbt,
    chunk_data: &[u8],
    light: &LightPayload,
) -> Vec<u8> {
    encode_packet(cb::PLAY_CHUNK_DATA, |e| {
        e.i32(x);
        e.i32(z);
        e.raw(&encode_root_unnamed(heightmaps));
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
        e.i8(angle_byte(pitch));
        e.i8(angle_byte(yaw));
        e.i8(angle_byte(yaw));
        e.varint(0);
        e.i16(0);
        e.i16(0);
        e.i16(0);
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
        e.i8(angle_byte(yaw));
        e.i8(angle_byte(pitch));
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
        return Err(DecodeError::UnknownPacket { state: "handshake", id });
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
        sb::STATUS_PING => Ok(IncomingStatusRequest { ping_time: Some(d.i64()?) }),
        other => Err(DecodeError::UnknownPacket { state: "status", id: other }),
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
    Ok(IncomingLoginStart { name: d.string()?, uuid: d.uuid()? })
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
    Position { x: f64, y: f64, z: f64, on_ground: bool },
    PositionLook { x: f64, y: f64, z: f64, yaw: f32, pitch: f32, on_ground: bool },
    Look { yaw: f32, pitch: f32, on_ground: bool },
    Flying { on_ground: bool },
}

pub fn decode_movement(payload: &[u8]) -> DecodeResult<Movement> {
    let mut d = Decoder::new(payload);
    match d.varint()? {
        sb::PLAY_POSITION => Ok(Movement::Position {
            x: d.f64()?,
            y: d.f64()?,
            z: d.f64()?,
            on_ground: d.bool()?,
        }),
        sb::PLAY_POSITION_LOOK => Ok(Movement::PositionLook {
            x: d.f64()?,
            y: d.f64()?,
            z: d.f64()?,
            yaw: d.f32()?,
            pitch: d.f32()?,
            on_ground: d.bool()?,
        }),
        sb::PLAY_LOOK => Ok(Movement::Look { yaw: d.f32()?, pitch: d.f32()?, on_ground: d.bool()? }),
        sb::PLAY_FLYING => Ok(Movement::Flying { on_ground: d.bool()? }),
        other => Err(DecodeError::UnknownPacket { state: "play", id: other }),
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
    Ok(IncomingBlockDig { status, x, y, z, sequence })
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
    Ok(IncomingBlockPlace { x, y, z, direction, cursor_x, cursor_y, cursor_z, sequence })
}

pub fn offset_by_direction(
    x: i32,
    y: i32,
    z: i32,
    direction: i32,
) -> (i32, i32, i32) {
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
    Ok(IncomingChatMessage { message: d.string()? })
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

pub struct CreativeSlot {
    pub slot: i16,
    pub block_state: Option<u16>,
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
        return Ok(Some(CreativeSlot { slot, block_state: None, count: 0 }));
    }
    let item_id = d.varint()?;
    let added_components = d.varint()?;
    let removed_components = d.varint()?;
    if added_components != 0 || removed_components != 0 {
        return Ok(None);
    }
    let block_state = crate::itemmap::lookup_block_state(item_id);
    Ok(Some(CreativeSlot { slot, block_state, count }))
}

pub fn decode_held_item_slot(payload: &[u8]) -> DecodeResult<Option<i16>> {
    let mut d = Decoder::new(payload);
    let id = d.varint()?;
    if id != sb::PLAY_HELD_ITEM_SLOT {
        return Err(DecodeError::UnknownPacket { state: "play", id });
    }
    Ok(Some(d.i16()?))
}
