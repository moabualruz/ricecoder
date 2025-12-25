//! Thread-safe collection access patterns
//!
//! Provides traits for consistent RwLock access patterns used across
//! multiple crates.

use parking_lot::RwLock;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

/// Trait for thread-safe collection access with RwLock patterns
pub trait CollectionAccess<K, V> {
    /// Get a value by key (cloned)
    fn get(&self, key: &K) -> Option<V>;

    /// Insert a value
    fn insert(&self, key: K, value: V) -> Option<V>;

    /// Remove a value by key
    fn remove(&self, key: &K) -> Option<V>;

    /// Check if key exists
    fn contains(&self, key: &K) -> bool;

    /// Get the number of entries
    fn len(&self) -> usize;

    /// Check if empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get all keys (cloned)
    fn keys(&self) -> Vec<K>;

    /// Clear all entries
    fn clear(&self);
}

/// Thread-safe HashMap wrapper implementing CollectionAccess
pub struct SyncMap<K, V> {
    inner: RwLock<HashMap<K, V>>,
}

impl<K, V> SyncMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: RwLock::new(HashMap::with_capacity(capacity)),
        }
    }
}

impl<K, V> Default for SyncMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> CollectionAccess<K, V> for SyncMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn get(&self, key: &K) -> Option<V> {
        self.inner.read().get(key).cloned()
    }

    fn insert(&self, key: K, value: V) -> Option<V> {
        self.inner.write().insert(key, value)
    }

    fn remove(&self, key: &K) -> Option<V> {
        self.inner.write().remove(key)
    }

    fn contains(&self, key: &K) -> bool {
        self.inner.read().contains_key(key)
    }

    fn len(&self) -> usize {
        self.inner.read().len()
    }

    fn keys(&self) -> Vec<K> {
        self.inner.read().keys().cloned().collect()
    }

    fn clear(&self) {
        self.inner.write().clear();
    }
}

/// Arc-wrapped SyncMap for shared ownership
pub type SharedMap<K, V> = Arc<SyncMap<K, V>>;

/// Create a new shared map
pub fn shared_map<K, V>() -> SharedMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    Arc::new(SyncMap::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_map_basic_operations() {
        let map: SyncMap<String, i32> = SyncMap::new();

        assert!(map.is_empty());

        map.insert("one".to_string(), 1);
        map.insert("two".to_string(), 2);

        assert_eq!(map.len(), 2);
        assert_eq!(map.get(&"one".to_string()), Some(1));
        assert!(map.contains(&"two".to_string()));

        map.remove(&"one".to_string());
        assert_eq!(map.len(), 1);

        map.clear();
        assert!(map.is_empty());
    }

    #[test]
    fn test_shared_map() {
        let map = shared_map::<String, i32>();
        let map2 = Arc::clone(&map);

        map.insert("key".to_string(), 42);
        assert_eq!(map2.get(&"key".to_string()), Some(42));
    }
}
