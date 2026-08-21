pub mod chunk;
pub mod lru;
pub mod store;
pub mod superflat;

pub use chunk::{ChunkColumn, Section, SECTIONS_PER_CHUNK};
pub use lru::LruChunkCache;
pub use store::{ChunkBacking, MemoryStore, World};

pub mod blocks {
    pub const AIR: u16 = 0;
    pub const STONE: u16 = 1;
    pub const GRASS_BLOCK: u16 = 9;
    pub const DIRT: u16 = 10;
    pub const BEDROCK: u16 = 85;
}

pub const MIN_Y: i32 = -64;
pub const WORLD_HEIGHT: i32 = 384;
