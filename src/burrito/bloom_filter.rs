use std::{collections::{hash_map::DefaultHasher, HashSet}, hash::{Hash, Hasher}};

#[derive(Clone)]
pub struct BloomFilter {
    hashes: HashSet<u64>,
}

impl Default for BloomFilter {
    fn default() -> Self {
        Self {
            hashes: HashSet::new(),
        }
    }
}

impl BloomFilter {
    pub fn insert<T: Hash>(&mut self, item: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        item.hash(&mut hasher);
        let hash = hasher.finish();
        self.hashes.insert(hash);
        hash
    }
    
    pub fn probably_contains<T: Hash>(&self, item: &T) -> bool {
        let mut hasher = DefaultHasher::new();
        item.hash(&mut hasher);
        let hash = hasher.finish();
        self.hashes.contains(&hash)
    }

    pub fn clear(&mut self) {
        self.hashes.clear();
    }

    pub fn new() -> Self {
        BloomFilter::default()
    }
}

#[cfg(test)]
mod tests {
    use super::BloomFilter;

    #[test]
    fn test_bloom_filter() {
        let mut uut = BloomFilter::new();
        assert!(!uut.probably_contains(&"ur mom".to_owned()));
        uut.insert(&"ur mom".to_owned());
        assert!(uut.probably_contains(&"ur mom".to_owned()));
        assert!(!uut.probably_contains(&"ur dad".to_owned()));
    }

}
