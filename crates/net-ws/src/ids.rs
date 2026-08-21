pub mod cb {
    pub const STATUS_RESPONSE: i32 = 0x00;
    pub const STATUS_PING: i32 = 0x01;

    pub const LOGIN_DISCONNECT: i32 = 0x00;
    pub const LOGIN_SUCCESS: i32 = 0x02;
    pub const LOGIN_COMPRESS: i32 = 0x03;

    pub const CFG_CUSTOM_PAYLOAD: i32 = 0x01;
    pub const CFG_DISCONNECT: i32 = 0x02;
    pub const CFG_FINISH: i32 = 0x03;
    pub const CFG_KEEP_ALIVE: i32 = 0x04;
    pub const CFG_PING: i32 = 0x05;
    pub const CFG_REGISTRY_DATA: i32 = 0x07;
    pub const CFG_KNOWN_PACKS: i32 = 0x0e;

    pub const PLAY_SPAWN_ENTITY: i32 = 0x01;
    pub const PLAY_ANIMATION: i32 = 0x03;
    pub const PLAY_ACK_DIGGING: i32 = 0x05;
    pub const PLAY_BLOCK_CHANGE: i32 = 0x09;
    pub const PLAY_CHUNK_BATCH_FINISHED: i32 = 0x0c;
    pub const PLAY_CHUNK_BATCH_START: i32 = 0x0d;
    pub const PLAY_SYNC_ENTITY_POSITION: i32 = 0x20;
    pub const PLAY_UNLOAD_CHUNK: i32 = 0x22;
    pub const PLAY_GAME_EVENT: i32 = 0x23;
    pub const PLAY_KEEP_ALIVE: i32 = 0x27;
    pub const PLAY_CHUNK_DATA: i32 = 0x28;
    pub const PLAY_LOGIN: i32 = 0x2c;
    pub const PLAY_POSITION: i32 = 0x42;
    pub const PLAY_REL_ENTITY_MOVE: i32 = 0x2f;
    pub const PLAY_ENTITY_MOVE_LOOK: i32 = 0x30;
    pub const PLAY_ENTITY_LOOK: i32 = 0x32;
    pub const PLAY_CUSTOM_PAYLOAD: i32 = 0x19;
    pub const PLAY_KICK_DISCONNECT: i32 = 0x1d;
    pub const PLAY_PLAYER_REMOVE: i32 = 0x3f;
    pub const PLAY_PLAYER_INFO: i32 = 0x40;
    pub const PLAY_ABILITIES: i32 = 0x3a;
    pub const PLAY_ENTITY_HEAD_ROTATION: i32 = 0x4d;
    pub const PLAY_PING_RESPONSE: i32 = 0x38;
    pub const PLAY_SPAWN_POSITION: i32 = 0x5b;
    pub const PLAY_UPDATE_TIME: i32 = 0x6b;
    pub const PLAY_SYSTEM_CHAT: i32 = 0x73;
    pub const PLAY_ENTITY_TELEPORT: i32 = 0x77;
    pub const PLAY_ENTITY_DESTROY: i32 = 0x47;
}

pub mod sb {
    pub const HANDSHAKE: i32 = 0x00;

    pub const STATUS_REQUEST: i32 = 0x00;
    pub const STATUS_PING: i32 = 0x01;

    pub const LOGIN_START: i32 = 0x00;
    pub const LOGIN_ACKNOWLEDGED: i32 = 0x03;

    pub const CFG_SETTINGS: i32 = 0x00;
    pub const CFG_CUSTOM_PAYLOAD: i32 = 0x02;
    pub const CFG_FINISH: i32 = 0x03;
    pub const CFG_KEEP_ALIVE: i32 = 0x04;
    pub const CFG_PONG: i32 = 0x05;
    pub const CFG_KNOWN_PACKS: i32 = 0x07;

    pub const PLAY_TELEPORT_CONFIRM: i32 = 0x00;
    pub const PLAY_CHAT_MESSAGE: i32 = 0x07;
    pub const PLAY_SETTINGS: i32 = 0x0c;
    pub const PLAY_CUSTOM_PAYLOAD: i32 = 0x14;
    pub const PLAY_KEEP_ALIVE: i32 = 0x1a;
    pub const PLAY_POSITION: i32 = 0x1c;
    pub const PLAY_POSITION_LOOK: i32 = 0x1d;
    pub const PLAY_LOOK: i32 = 0x1e;
    pub const PLAY_FLYING: i32 = 0x1f;
    pub const PLAY_PING_REQUEST: i32 = 0x24;
    pub const PLAY_BLOCK_DIG: i32 = 0x27;
    pub const PLAY_PONG: i32 = 0x2b;
    pub const PLAY_HELD_ITEM_SLOT: i32 = 0x33;
    pub const PLAY_SET_CREATIVE_SLOT: i32 = 0x36;
    pub const PLAY_ARM_ANIMATION: i32 = 0x3a;
    pub const PLAY_BLOCK_PLACE: i32 = 0x3c;
    pub const PLAY_USE_ITEM: i32 = 0x3d;
}
