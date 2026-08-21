use std::collections::HashMap;
use std::collections::VecDeque;

pub struct LruChunkCache<V> {
    capacity: usize,
    map: HashMap<(i32, i32), V>,
    order: VecDeque<(i32, i32)>,
}

impl<V> LruChunkCache<V> {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            map: HashMap::new(),
            order: VecDeque::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn get(&mut self, x: i32, z: i32) -> Option<&V> {
        if self.map.contains_key(&(x, z)) {
            self.touch(x, z);
        }
        self.map.get(&(x, z))
    }

    pub fn get_mut(&mut self, x: i32, z: i32) -> Option<&mut V> {
        if self.map.contains_key(&(x, z)) {
            self.touch(x, z);
        }
        self.map.get_mut(&(x, z))
    }

    pub fn insert(&mut self, x: i32, z: i32, value: V) {
        if !self.map.contains_key(&(x, z)) {
            while self.map.len() >= self.capacity {
                let victim = self
                    .order
                    .front()
                    .copied()
                    .expect("order tracks all entries");
                self.order.pop_front();
                self.map.remove(&victim);
            }
            self.order.push_back((x, z));
        }
        self.touch(x, z);
        self.map.insert((x, z), value);
    }

    pub fn remove(&mut self, x: i32, z: i32) -> Option<V> {
        let removed = self.map.remove(&(x, z));
        if removed.is_some() {
            if let Some(pos) = self.order.iter().position(|&k| k == (x, z)) {
                self.order.remove(pos);
            }
        }
        removed
    }

    pub fn lru_front(&self) -> Option<(i32, i32)> {
        self.order.front().copied()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&(i32, i32), &mut V)> {
        self.map.iter_mut()
    }

    fn touch(&mut self, x: i32, z: i32) {
        if let Some(pos) = self.order.iter().position(|&k| k == (x, z)) {
            let key = self.order.remove(pos).unwrap();
            self.order.push_back(key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evicts_least_recently_used() {
        let mut cache = LruChunkCache::new(2);
        cache.insert(0, 0, 'a');
        cache.insert(1, 0, 'b');
        assert_eq!(cache.get(0, 0), Some(&'a'));
        cache.insert(2, 0, 'c');
        assert_eq!(cache.get(1, 0), None);
        assert_eq!(cache.get(0, 0), Some(&'a'));
        assert_eq!(cache.get(2, 0), Some(&'c'));
    }

    #[test]
    fn reinsert_does_not_evict_self() {
        let mut cache = LruChunkCache::new(1);
        cache.insert(5, 5, 1);
        cache.insert(5, 5, 2);
        assert_eq!(cache.get(5, 5), Some(&2));
    }

    #[test]
    fn remove_works() {
        let mut cache = LruChunkCache::new(4);
        cache.insert(1, 1, "x");
        assert_eq!(cache.remove(1, 1), Some("x"));
        assert_eq!(cache.remove(1, 1), None);
        assert_eq!(cache.len(), 0);
    }
}
