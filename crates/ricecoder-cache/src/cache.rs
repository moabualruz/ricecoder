//! Main cache implementation with multi-level support


use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::{CacheError, Result};
use crate::metrics::{CacheMetrics, OperationTimer};
use crate::storage::{CacheEntry, CacheStorage};
use crate::strategy::CacheStrategy;

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Default TTL for cache entries
    pub default_ttl: Option<std::time::Duration>,
    /// Maximum number of entries
    pub max_entries: Option<usize>,
    /// Maximum total size in bytes
    pub max_size_bytes: Option<u64>,
    /// Enable metrics collection
    pub enable_metrics: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            default_ttl: Some(std::time::Duration::from_secs(3600)), // 1 hour
            max_entries: None,
            max_size_bytes: None,
            enable_metrics: true,
        }
    }
}

/// Multi-level cache implementation
pub struct Cache {
    /// Primary storage (usually memory)
    primary_storage: Arc<dyn CacheStorage>,
    /// Secondary storage (usually disk)
    secondary_storage: Option<Arc<dyn CacheStorage>>,
    /// Tertiary storage (usually remote)
    tertiary_storage: Option<Arc<dyn CacheStorage>>,
    /// Cache invalidation strategy
    strategy: Option<Arc<dyn CacheStrategy>>,
    /// Configuration
    config: CacheConfig,
    /// Metrics collector
    metrics: Arc<CacheMetrics>,
    /// Cache entries metadata for invalidation
    entries: Arc<RwLock<HashMap<String, HashMap<String, serde_json::Value>>>>,
}

impl Cache {
    /// Create a new cache with primary storage only
    pub fn new(primary_storage: Arc<dyn CacheStorage>) -> Self {
        Self::with_config(primary_storage, CacheConfig::default())
    }

    /// Create a new cache with configuration
    pub fn with_config(primary_storage: Arc<dyn CacheStorage>, config: CacheConfig) -> Self {
        Self {
            primary_storage,
            secondary_storage: None,
            tertiary_storage: None,
            strategy: None,
            config,
            metrics: Arc::new(CacheMetrics::new()),
            entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add secondary storage (L2 cache)
    pub fn with_secondary_storage(mut self, storage: Arc<dyn CacheStorage>) -> Self {
        self.secondary_storage = Some(storage);
        self
    }

    /// Add tertiary storage (L3 cache)
    pub fn with_tertiary_storage(mut self, storage: Arc<dyn CacheStorage>) -> Self {
        self.tertiary_storage = Some(storage);
        self
    }

    /// Set invalidation strategy
    pub fn with_strategy(mut self, strategy: Arc<dyn CacheStrategy>) -> Self {
        self.strategy = Some(strategy);
        self
    }

    /// Get cache configuration
    pub fn config(&self) -> &CacheConfig {
        &self.config
    }

    /// Get cache metrics
    pub fn metrics(&self) -> &Arc<CacheMetrics> {
        &self.metrics
    }

    /// Store a value in the cache
    pub async fn set<T: Serialize + Send + Sync + Clone>(
        &self,
        key: &str,
        value: T,
        ttl: Option<std::time::Duration>,
    ) -> Result<()> {
        let timer = OperationTimer::start();

        // Create cache entry
        let mut entry = CacheEntry::new(value, ttl.or(self.config.default_ttl));

        // Add metadata for invalidation strategies
        if let Some(ref strategy) = self.strategy {
            entry.metadata.insert(
                "strategy_name".to_string(),
                serde_json::json!(strategy.name()),
            );
        }

        // Serialize entry to JSON
        let json_value = serde_json::to_value(&entry)
            .map_err(|e| CacheError::Serialization {
                message: e.to_string(),
            })?;

        // Store in primary storage
        self.primary_storage.set(key, &json_value).await?;

        // Store in secondary storage if available
        if let Some(ref secondary) = self.secondary_storage {
            let _ = secondary.set(key, &json_value).await; // Ignore errors for secondary
        }

        // Store in tertiary storage if available
        if let Some(ref tertiary) = self.tertiary_storage {
            let _ = tertiary.set(key, &json_value).await; // Ignore errors for tertiary
        }

        // Update metrics
        if self.config.enable_metrics {
            self.metrics.record_store(timer.elapsed_ms(), entry.size_bytes);
            self.metrics.set_entry_count(self.primary_storage.len().await.unwrap_or(0) as usize);
        }

        // Update entries metadata
        let mut entries = self.entries.write().await;
        entries.insert(key.to_string(), entry.metadata.clone());

        // Notify strategy
        if let Some(ref strategy) = self.strategy {
            let _ = strategy.on_cache_miss(key).await; // New entry is like a miss
        }

        Ok(())
    }

    /// Retrieve a value from the cache
    pub async fn get<T: for<'de> Deserialize<'de> + Clone + Serialize>(&self, key: &str) -> Result<Option<T>> {
        let timer = OperationTimer::start();

        // Try primary storage first
        if let Some(json_value) = self.primary_storage.get(key).await? {
            let entry: CacheEntry<T> = serde_json::from_value(json_value)
                .map_err(|e| CacheError::Deserialization {
                    message: e.to_string(),
                })?;

            // Check invalidation strategy
            if let Some(ref strategy) = self.strategy {
                if strategy.should_invalidate(key, entry.created_at, &entry.metadata).await.unwrap_or(false) {
                    // Entry should be invalidated
                    let _ = self.primary_storage.remove(key).await;
                    let mut entries = self.entries.write().await;
                    entries.remove(key);
                    if self.config.enable_metrics {
                        self.metrics.record_invalidation();
                    }
                    return Ok(None);
                }
            }

            if !entry.is_expired() {
                // Cache hit in primary storage
                if self.config.enable_metrics {
                    self.metrics.record_hit(timer.elapsed_ms());
                }

                // Notify strategy
                if let Some(ref strategy) = self.strategy {
                    let _ = strategy.on_cache_hit(key).await;
                }

                return Ok(Some(entry.data));
            } else {
                // Entry expired, remove it
                let _ = self.primary_storage.remove(key).await;
                let mut entries = self.entries.write().await;
                entries.remove(key);
            }
        }

        // Try secondary storage
        if let Some(ref secondary) = self.secondary_storage {
            if let Some(json_value) = secondary.get(key).await? {
                let entry: CacheEntry<T> = serde_json::from_value(json_value)
                    .map_err(|e| CacheError::Deserialization {
                        message: e.to_string(),
                    })?;

                // Check invalidation strategy
                if let Some(ref strategy) = self.strategy {
                    if strategy.should_invalidate(key, entry.created_at, &entry.metadata).await.unwrap_or(false) {
                        let _ = secondary.remove(key).await;
                        return Ok(None);
                    }
                }

                if !entry.is_expired() {
                    // Cache hit in secondary storage
                    // Promote to primary storage
                    let _ = self.primary_storage.set(key, &serde_json::to_value(&entry).unwrap()).await;

                    if self.config.enable_metrics {
                        self.metrics.record_hit(timer.elapsed_ms());
                    }

                    // Notify strategy
                    if let Some(ref strategy) = self.strategy {
                        let _ = strategy.on_cache_hit(key).await;
                    }

                    return Ok(Some(entry.data));
                } else {
                    // Entry expired, remove it
                    let _ = secondary.remove(key).await;
                }
            }
        }

        // Try tertiary storage
        if let Some(ref tertiary) = self.tertiary_storage {
            if let Some(json_value) = tertiary.get(key).await? {
                let entry: CacheEntry<T> = serde_json::from_value(json_value)
                    .map_err(|e| CacheError::Deserialization {
                        message: e.to_string(),
                    })?;

                // Check invalidation strategy
                if let Some(ref strategy) = self.strategy {
                    if strategy.should_invalidate(key, entry.created_at, &entry.metadata).await.unwrap_or(false) {
                        let _ = tertiary.remove(key).await;
                        return Ok(None);
                    }
                }

                if !entry.is_expired() {
                    // Cache hit in tertiary storage
                    // Promote to primary and secondary storage
                    let primary_value = serde_json::to_value(&entry).unwrap();
                    let _ = self.primary_storage.set(key, &primary_value).await;
                    if let Some(ref secondary) = self.secondary_storage {
                        let _ = secondary.set(key, &primary_value).await;
                    }

                    if self.config.enable_metrics {
                        self.metrics.record_hit(timer.elapsed_ms());
                    }

                    // Notify strategy
                    if let Some(ref strategy) = self.strategy {
                        let _ = strategy.on_cache_hit(key).await;
                    }

                    return Ok(Some(entry.data));
                } else {
                    // Entry expired, remove it
                    let _ = tertiary.remove(key).await;
                }
            }
        }

        // Cache miss
        if self.config.enable_metrics {
            self.metrics.record_miss();
        }

        // Notify strategy
        if let Some(ref strategy) = self.strategy {
            let _ = strategy.on_cache_miss(key).await;
        }

        Ok(None)
    }

    /// Check if a key exists in the cache
    pub async fn contains(&self, key: &str) -> Result<bool> {
        // Check all levels
        if self.primary_storage.contains(key).await? {
            return Ok(true);
        }

        if let Some(ref secondary) = self.secondary_storage {
            if secondary.contains(key).await? {
                return Ok(true);
            }
        }

        if let Some(ref tertiary) = self.tertiary_storage {
            if tertiary.contains(key).await? {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Remove a key from all cache levels
    pub async fn remove(&self, key: &str) -> Result<bool> {
        let mut removed = false;

        // Remove from primary
        if self.primary_storage.remove(key).await? {
            removed = true;
        }

        // Remove from secondary
        if let Some(ref secondary) = self.secondary_storage {
            let _ = secondary.remove(key).await; // Ignore errors
        }

        // Remove from tertiary
        if let Some(ref tertiary) = self.tertiary_storage {
            let _ = tertiary.remove(key).await; // Ignore errors
        }

        // Remove from entries metadata
        let mut entries = self.entries.write().await;
        entries.remove(key);

        if self.config.enable_metrics && removed {
            self.metrics.set_entry_count(self.primary_storage.len().await.unwrap_or(0) as usize);
        }

        Ok(removed)
    }

    /// Clear all cache levels
    pub async fn clear(&self) -> Result<()> {
        self.primary_storage.clear().await?;

        if let Some(ref secondary) = self.secondary_storage {
            let _ = secondary.clear().await; // Ignore errors
        }

        if let Some(ref tertiary) = self.tertiary_storage {
            let _ = tertiary.clear().await; // Ignore errors
        }

        // Clear entries metadata
        let mut entries = self.entries.write().await;
        entries.clear();

        if self.config.enable_metrics {
            self.metrics.set_entry_count(0);
        }

        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&self) -> crate::metrics::CacheStats {
        self.metrics.snapshot()
    }

    /// Get number of entries in primary cache
    pub async fn len(&self) -> Result<usize> {
        self.primary_storage.len().await
    }

    /// Get total size of primary cache in bytes
    pub async fn size_bytes(&self) -> Result<u64> {
        self.primary_storage.size_bytes().await
    }
}

/// Builder pattern for cache construction
pub struct CacheBuilder {
    config: CacheConfig,
    primary_storage: Option<Arc<dyn CacheStorage>>,
    secondary_storage: Option<Arc<dyn CacheStorage>>,
    tertiary_storage: Option<Arc<dyn CacheStorage>>,
    strategy: Option<Arc<dyn CacheStrategy>>,
}

impl CacheBuilder {
    /// Create a new cache builder
    pub fn new() -> Self {
        Self {
            config: CacheConfig::default(),
            primary_storage: None,
            secondary_storage: None,
            tertiary_storage: None,
            strategy: None,
        }
    }

    /// Set cache configuration
    pub fn config(mut self, config: CacheConfig) -> Self {
        self.config = config;
        self
    }

    /// Set primary storage (required)
    pub fn primary_storage(mut self, storage: Arc<dyn CacheStorage>) -> Self {
        self.primary_storage = Some(storage);
        self
    }

    /// Set secondary storage
    pub fn secondary_storage(mut self, storage: Arc<dyn CacheStorage>) -> Self {
        self.secondary_storage = Some(storage);
        self
    }

    /// Set tertiary storage
    pub fn tertiary_storage(mut self, storage: Arc<dyn CacheStorage>) -> Self {
        self.tertiary_storage = Some(storage);
        self
    }

    /// Set invalidation strategy
    pub fn strategy(mut self, strategy: Arc<dyn CacheStrategy>) -> Self {
        self.strategy = Some(strategy);
        self
    }

    /// Build the cache
    pub fn build(self) -> Result<Cache> {
        let primary_storage = self.primary_storage
            .ok_or_else(|| CacheError::Storage {
                message: "Primary storage is required".to_string(),
            })?;

        let mut cache = Cache::with_config(primary_storage, self.config);

        if let Some(secondary) = self.secondary_storage {
            cache = cache.with_secondary_storage(secondary);
        }

        if let Some(tertiary) = self.tertiary_storage {
            cache = cache.with_tertiary_storage(tertiary);
        }

        if let Some(strategy) = self.strategy {
            cache = cache.with_strategy(strategy);
        }

        Ok(cache)
    }
}

impl Default for CacheBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::MemoryStorage;
    use crate::strategy::TtlStrategy;
    use std::time::Duration;

    #[tokio::test]
    async fn test_cache_basic_operations() {
        let storage = Arc::new(MemoryStorage::new());
        let cache = Cache::new(storage);

        // Test set and get
        cache.set("key1", "value1", None).await.unwrap();
        let value: Option<String> = cache.get("key1").await.unwrap();
        assert_eq!(value, Some("value1".to_string()));

        // Test contains
        assert!(cache.contains("key1").await.unwrap());
        assert!(!cache.contains("key2").await.unwrap());

        // Test remove
        assert!(cache.remove("key1").await.unwrap());
        assert!(!cache.contains("key1").await.unwrap());
    }

    #[tokio::test]
    async fn test_cache_with_ttl() {
        let storage = Arc::new(MemoryStorage::new());
        let cache = Cache::new(storage);

        // Set with short TTL
        cache.set("key1", "value1", Some(Duration::from_millis(100))).await.unwrap();

        // Should exist immediately
        let value: Option<String> = cache.get("key1").await.unwrap();
        assert_eq!(value, Some("value1".to_string()));

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Should be expired
        let value: Option<String> = cache.get("key1").await.unwrap();
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_cache_with_strategy() {
        let storage = Arc::new(MemoryStorage::new());
        let strategy = Arc::new(TtlStrategy::with_seconds(1));
        let cache = Cache::new(storage).with_strategy(strategy);

        cache.set("key1", "value1", None).await.unwrap();

        // Should exist initially
        let value: Option<String> = cache.get("key1").await.unwrap();
        assert_eq!(value, Some("value1".to_string()));

        // Wait for strategy TTL to expire
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Should be invalidated by strategy
        let value: Option<String> = cache.get("key1").await.unwrap();
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_cache_metrics() {
        let storage = Arc::new(MemoryStorage::new());
        let cache = Cache::new(storage);

        // Initial state
        let stats = cache.stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);

        // Cache miss
        let _value: Option<String> = cache.get("nonexistent").await.unwrap();
        let stats = cache.stats();
        assert_eq!(stats.misses, 1);

        // Cache hit
        cache.set("key1", "value1", None).await.unwrap();
        let _value: Option<String> = cache.get("key1").await.unwrap();
        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
    }

    #[tokio::test]
    async fn test_cache_builder() {
        let storage = Arc::new(MemoryStorage::new());
        let strategy = Arc::new(TtlStrategy::with_hours(1));

        let cache = CacheBuilder::new()
            .primary_storage(storage)
            .strategy(strategy)
            .build()
            .unwrap();

        cache.set("key1", "value1", None).await.unwrap();
        let value: Option<String> = cache.get("key1").await.unwrap();
        assert_eq!(value, Some("value1".to_string()));
    }

    #[tokio::test]
    async fn test_multi_level_cache() {
        let primary = Arc::new(MemoryStorage::new());
        let secondary = Arc::new(MemoryStorage::new());

        let cache = Cache::new(primary).with_secondary_storage(secondary);

        // Set in primary only initially
        cache.set("key1", "value1", None).await.unwrap();

        // Should find in primary
        let value: Option<String> = cache.get("key1").await.unwrap();
        assert_eq!(value, Some("value1".to_string()));

        // Simulate primary cache miss, should promote from secondary
        // (This is a simplified test - in practice the promotion happens automatically)
    }
}