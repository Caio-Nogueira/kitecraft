pub mod chunk;
pub mod lru;
pub mod store;
pub mod superflat;

pub use chunk::{ChunkColumn, Section, SECTIONS_PER_CHUNK};
pub use lru::LruChunkCache;
pub use store::{ChunkBacking, ChunkGenerator, MemoryStore, World};

pub mod blocks {
    pub use block_data::{AIR, BEDROCK, DIRT, GRASS_BLOCK, STONE};
}

pub const MIN_Y: i32 = -64;
pub const WORLD_HEIGHT: i32 = 384;
