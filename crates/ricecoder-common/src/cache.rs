//! Common cache operation traits
//!
//! Provides a unified caching interface used across multiple crates.

use std::future::Future;
use std::time::Duration;

/// Trait for cache operations
pub trait CacheOperations<K, V> {
    /// Get a cached value
    fn get(&self, key: &K) -> Option<V>;

    /// Set a cached value with optional TTL
    fn set(&self, key: K, value: V, ttl: Option<Duration>);

    /// Remove a cached value
    fn remove(&self, key: &K) -> Option<V>;

    /// Check if a key is cached
    fn contains(&self, key: &K) -> bool;

    /// Clear all cached values
    fn clear(&self);

    /// Get the number of cached entries
    fn len(&self) -> usize;

    /// Check if cache is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Async cache operations trait
pub trait AsyncCacheOperations<K, V>: Send + Sync {
    /// Get a cached value asynchronously
    fn get(&self, key: &K) -> impl Future<Output = Option<V>> + Send;

    /// Set a cached value asynchronously
    fn set(&self, key: K, value: V, ttl: Option<Duration>) -> impl Future<Output = ()> + Send;

    /// Remove a cached value asynchronously
    fn remove(&self, key: &K) -> impl Future<Output = Option<V>> + Send;

    /// Get or compute a value
    fn get_or_insert<F, Fut>(
        &self,
        key: K,
        f: F,
        ttl: Option<Duration>,
    ) -> impl Future<Output = V> + Send
    where
        K: Clone,
        F: FnOnce() -> Fut + Send,
        Fut: Future<Output = V> + Send;
}

/// Cache statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub size: usize,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

/// Trait for caches that track statistics
pub trait CacheWithStats<K, V>: CacheOperations<K, V> {
    fn stats(&self) -> CacheStats;
    fn reset_stats(&self);
}
