use std::collections::{BTreeMap, HashMap};

use serde_derive::{Deserialize, Serialize};
use serde_with::serde_as;

use super::systems::{Distance, SystemId};

#[serde_as]
#[derive(Clone, Debug, Default, Eq, Deserialize, PartialEq, Serialize)]
pub struct PathCache {
    #[serde_as(as = "HashMap<serde_with::json::JsonString, _>")]
    path_cache: HashMap<(SystemId, SystemId), PathCacheEntry>,
    strategy: PathCacheStrategy,
}

impl PathCache {
    pub fn search(&mut self, key: &(SystemId, SystemId)) -> Option<Distance> {
        let opposite_key = (key.1, key.0);
        let mut new_key = key.to_owned();
        let mut new_entry:PathCacheEntry = Default::default();
        if let Some(entry) = self.path_cache.get(&opposite_key) {
            // Old entry with opposite key exists. Update it
            new_key = opposite_key;
            new_entry = entry.clone();
            new_entry.last_updated_ms = chrono::offset::Utc::now().timestamp_millis();
        }
        else {
            // Either there is no old entry or it needs to be updated
            if let Some(entry) = self.path_cache.get(key) {
                new_entry = entry.clone();
                new_entry.last_updated_ms = chrono::offset::Utc::now().timestamp_millis();
            }
            else {
                return None;
            }
        }
        self.update(new_key, new_entry.clone());
        return Some(new_entry.distance);
    }

    pub fn insert(&mut self, key: (SystemId, SystemId), value: Distance) {
        let entry = PathCacheEntry {
            distance: value,
            last_updated_ms: chrono::offset::Utc::now().timestamp_millis(),
        };
        self.update(key, entry);
    }

    fn update(&mut self, key: (SystemId, SystemId), value: PathCacheEntry) {
        match self.strategy {
            PathCacheStrategy::Persistent => {
                self.path_cache.insert(key, value);
            },
            PathCacheStrategy::MostRecentlyUsed(n) => {
                let mut map: BTreeMap<PathCacheEntry, (SystemId, SystemId)> = BTreeMap::new();
                self.path_cache.iter().for_each(|e| {
                    map.insert(e.1.clone(), e.0.clone());
                });
                if map.len() >= n {
                    if map.len() > n {
                        // Shrink cache by removing everything older than most recent c entries
                        map.iter().take(self.path_cache.len() - n).for_each(|e| {
                            self.path_cache.remove(e.1).expect("Failed to remove 1");
                        });
                    }
                    // Reverse map to take most recent c items and then get the least recent of those
                    if let Some(lru) = map.iter().rev().take(n).next() {
                        self.path_cache.remove(lru.1).expect("Failed to remove 2");
                    }
                }
                self.path_cache.insert(key, value);
            },
            PathCacheStrategy::None => {
                self.path_cache.clear();
            },
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, Default, Deserialize, Ord, PartialEq, PartialOrd, Serialize)]
pub enum PathCacheStrategy {
    #[default]
    Persistent,
    MostRecentlyUsed(usize),
    None,
}

#[derive(Clone, Debug, Default, Eq, Deserialize, PartialEq, Serialize)]
pub struct PathCacheEntry {
    distance: Distance,
    last_updated_ms: i64,
}

impl Ord for PathCacheEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.last_updated_ms.cmp(&other.last_updated_ms)
    }
}

impl PartialOrd for PathCacheEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.last_updated_ms.partial_cmp(&other.last_updated_ms)
    }
}