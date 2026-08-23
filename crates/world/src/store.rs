use crate::chunk::{deserialize_column, serialize_column, ChunkColumn};
use crate::lru::LruChunkCache;
use crate::superflat::Superflat;
use std::collections::HashSet;

pub trait ChunkGenerator {
    fn generate(&mut self, x: i32, z: i32) -> ChunkColumn;
}

pub trait ChunkBacking {
    fn read(&self, x: i32, z: i32) -> Option<Vec<u8>>;
    fn write(&mut self, x: i32, z: i32, data: &[u8]);
}

pub struct MemoryStore {
    map: std::collections::HashMap<(i32, i32), Vec<u8>>,
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            map: std::collections::HashMap::new(),
        }
    }
}

impl ChunkBacking for MemoryStore {
    fn read(&self, x: i32, z: i32) -> Option<Vec<u8>> {
        self.map.get(&(x, z)).cloned()
    }

    fn write(&mut self, x: i32, z: i32, data: &[u8]) {
        self.map.insert((x, z), data.to_vec());
    }
}

pub struct World<B: ChunkBacking> {
    backing: B,
    cache: LruChunkCache<ChunkColumn>,
    dirty: HashSet<(i32, i32)>,
    generator: Box<dyn ChunkGenerator>,
    cache_capacity: usize,
}

pub const DEFAULT_CACHE_CHUNKS: usize = 512;

impl<B: ChunkBacking> World<B> {
    pub fn new(backing: B) -> Self {
        Self::with_cache_size(backing, DEFAULT_CACHE_CHUNKS)
    }

    pub fn with_cache_size(backing: B, cache_chunks: usize) -> Self {
        Self::with_generator_and_cache_size(backing, Box::new(Superflat::classic()), cache_chunks)
    }

    pub fn with_generator(backing: B, generator: Box<dyn ChunkGenerator>) -> Self {
        Self::with_generator_and_cache_size(backing, generator, DEFAULT_CACHE_CHUNKS)
    }

    pub fn with_generator_and_cache_size(
        backing: B,
        generator: Box<dyn ChunkGenerator>,
        cache_chunks: usize,
    ) -> Self {
        let cap = cache_chunks.max(1);
        Self {
            backing,
            cache: LruChunkCache::new(cap),
            dirty: HashSet::new(),
            generator,
            cache_capacity: cap,
        }
    }

    pub fn backing(&self) -> &B {
        &self.backing
    }

    pub fn backing_mut(&mut self) -> &mut B {
        &mut self.backing
    }

    pub fn dirty_count(&self) -> usize {
        self.dirty.len()
    }

    pub fn chunk(&mut self, x: i32, z: i32) -> &ChunkColumn {
        if self.cache.get(x, z).is_none() {
            let col = match self
                .backing
                .read(x, z)
                .and_then(|blob| deserialize_column(&blob))
            {
                Some(col) => col,
                None => self.generator.generate(x, z),
            };
            if self.cache.len() >= self.cache_capacity {
                self.evict_one();
            }
            self.cache.insert(x, z, col);
        }
        self.cache.get(x, z).expect("just inserted")
    }

    pub fn chunk_mut(&mut self, x: i32, z: i32) -> &mut ChunkColumn {
        self.chunk(x, z);
        self.dirty.insert((x, z));
        self.cache.get_mut(x, z).expect("loaded above")
    }

    pub fn set_block(&mut self, bx: i32, by: i32, bz: i32, state: u16) -> bool {
        let (cx, cz) = (bx >> 4, bz >> 4);
        let lx = bx & 15;
        let lz = bz & 15;
        let col = self.chunk_mut(cx, cz);
        col.set_block(lx, by, lz, state)
    }

    pub fn get_block(&mut self, bx: i32, by: i32, bz: i32) -> Option<u16> {
        let (cx, cz) = (bx >> 4, bz >> 4);
        let lx = bx & 15;
        let lz = bz & 15;
        self.chunk(cx, cz).get_block(lx, by, lz)
    }

    fn evict_one(&mut self) {
        let victim = self
            .cache
            .lru_front()
            .expect("cache at capacity has an LRU entry");
        if let Some(col) = self.cache.get(victim.0, victim.1).cloned() {
            if self.dirty.remove(&victim) {
                let blob = serialize_column(&col);
                self.backing.write(victim.0, victim.1, &blob);
            }
        }
        self.cache.remove(victim.0, victim.1);
    }

    pub fn flush_dirty(&mut self) -> usize {
        let mut flushed = 0;
        for key in std::mem::take(&mut self.dirty) {
            if let Some(col) = self.cache.get(key.0, key.1) {
                let blob = serialize_column(col);
                self.backing.write(key.0, key.1, &blob);
                flushed += 1;
            }
        }
        flushed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks;

    struct MarkerGenerator;

    impl ChunkGenerator for MarkerGenerator {
        fn generate(&mut self, x: i32, z: i32) -> ChunkColumn {
            let mut column = ChunkColumn::default();
            column.set_block(0, crate::MIN_Y, 0, (x.wrapping_add(z) as u16).max(1));
            column
        }
    }

    #[test]
    fn generates_then_persists_edits() {
        let mut world = World::new(MemoryStore::new());
        assert_eq!(world.get_block(5, -61, 5), Some(blocks::GRASS_BLOCK));
        assert!(world.set_block(5, -60, 5, blocks::STONE));
        assert_eq!(world.dirty_count(), 1);
        assert_eq!(world.flush_dirty(), 1);

        let mut reopened = World::new(world.backing);
        assert_eq!(reopened.get_block(5, -60, 5), Some(blocks::STONE));
        assert_eq!(reopened.get_block(6, -60, 5), Some(blocks::AIR));
    }

    #[test]
    fn flush_without_edits_is_noop() {
        let mut world = World::new(MemoryStore::new());
        world.chunk(1, 1);
        assert_eq!(world.flush_dirty(), 0);
    }

    #[test]
    fn eviction_writes_dirty_chunks() {
        let mut world = World::with_cache_size(MemoryStore::new(), 2);
        world.set_block(0, -60, 0, blocks::DIRT);
        let _ = world.chunk(1000, 0);
        let _ = world.chunk(2000, 0);
        world.flush_dirty();
        let mut reopened = World::new(world.backing);
        assert_eq!(reopened.get_block(0, -60, 0), Some(blocks::DIRT));
    }

    #[test]
    fn injected_generator_handles_missing_chunks() {
        let mut world = World::with_generator(MemoryStore::new(), Box::new(MarkerGenerator));
        assert_eq!(world.get_block(0, crate::MIN_Y, 0), Some(1));
        assert_eq!(world.get_block(16, crate::MIN_Y, 0), Some(1));
        assert_eq!(world.get_block(32, crate::MIN_Y, 0), Some(2));
    }
}
