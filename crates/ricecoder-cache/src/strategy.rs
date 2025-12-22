//! Cache invalidation strategies

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::SystemTime,
};

use async_trait::async_trait;

use crate::{CacheError, Result};

/// Cache invalidation strategy trait
#[async_trait]
pub trait CacheStrategy: Send + Sync {
    /// Check if a cache entry should be invalidated
    async fn should_invalidate(
        &self,
        key: &str,
        created_at: SystemTime,
        metadata: &HashMap<String, serde_json::Value>,
    ) -> Result<bool>;

    /// Update strategy state after cache operation
    async fn on_cache_hit(&self, key: &str) -> Result<()> {
        // Default: no-op
        Ok(())
    }

    /// Update strategy state after cache miss
    async fn on_cache_miss(&self, key: &str) -> Result<()> {
        // Default: no-op
        Ok(())
    }

    /// Get strategy name for debugging
    fn name(&self) -> &str;
}

/// Time-to-live (TTL) based invalidation strategy
pub struct TtlStrategy {
    ttl: std::time::Duration,
}

impl TtlStrategy {
    /// Create new TTL strategy
    pub fn new(ttl: std::time::Duration) -> Self {
        Self { ttl }
    }

    /// Create TTL strategy with seconds
    pub fn with_seconds(seconds: u64) -> Self {
        Self::new(std::time::Duration::from_secs(seconds))
    }

    /// Create TTL strategy with minutes
    pub fn with_minutes(minutes: u64) -> Self {
        Self::new(std::time::Duration::from_secs(minutes * 60))
    }

    /// Create TTL strategy with hours
    pub fn with_hours(hours: u64) -> Self {
        Self::new(std::time::Duration::from_secs(hours * 3600))
    }
}

#[async_trait]
impl CacheStrategy for TtlStrategy {
    async fn should_invalidate(
        &self,
        _key: &str,
        created_at: SystemTime,
        _metadata: &HashMap<String, serde_json::Value>,
    ) -> Result<bool> {
        let elapsed = SystemTime::now()
            .duration_since(created_at)
            .map_err(|_| CacheError::TimeConversion)?;

        Ok(elapsed > self.ttl)
    }

    fn name(&self) -> &str {
        "ttl"
    }
}

/// File change detection strategy
pub struct FileChangeStrategy {
    file_mtimes: HashMap<PathBuf, SystemTime>,
}

impl FileChangeStrategy {
    /// Create new file change strategy
    pub fn new() -> Self {
        Self {
            file_mtimes: HashMap::new(),
        }
    }

    /// Add a file to monitor for changes
    pub fn monitor_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref().to_path_buf();
        let metadata = std::fs::metadata(&path)?;
        let mtime = metadata.modified()?;
        self.file_mtimes.insert(path, mtime);
        Ok(())
    }

    /// Add multiple files to monitor
    pub fn monitor_files<P: AsRef<Path>, I: IntoIterator<Item = P>>(
        &mut self,
        paths: I,
    ) -> Result<()> {
        for path in paths {
            self.monitor_file(path)?;
        }
        Ok(())
    }

    /// Check if any monitored files have changed
    pub fn have_files_changed(&self) -> Result<bool> {
        for (path, cached_mtime) in &self.file_mtimes {
            match std::fs::metadata(path) {
                Ok(metadata) => {
                    let current_mtime = metadata.modified()?;
                    if current_mtime > *cached_mtime {
                        return Ok(true);
                    }
                }
                Err(_) => {
                    // File was deleted
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}

impl Default for FileChangeStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CacheStrategy for FileChangeStrategy {
    async fn should_invalidate(
        &self,
        _key: &str,
        _created_at: SystemTime,
        metadata: &HashMap<String, serde_json::Value>,
    ) -> Result<bool> {
        // Check if file modification times are stored in metadata
        if let Some(file_mtimes_value) = metadata.get("file_mtimes") {
            if let Ok(file_mtimes) =
                serde_json::from_value::<HashMap<String, SystemTime>>(file_mtimes_value.clone())
            {
                // Compare current file times with cached times
                for (path_str, cached_mtime) in file_mtimes {
                    let path = PathBuf::from(path_str);
                    match tokio::fs::metadata(&path).await {
                        Ok(metadata) => {
                            if let Ok(current_mtime) = metadata.modified() {
                                if current_mtime > cached_mtime {
                                    return Ok(true);
                                }
                            }
                        }
                        Err(_) => {
                            // File was deleted or inaccessible
                            return Ok(true);
                        }
                    }
                }
            }
        }

        Ok(false)
    }

    fn name(&self) -> &str {
        "file_change"
    }
}

/// Least Recently Used (LRU) eviction strategy
pub struct LruStrategy {
    max_entries: usize,
    access_order: std::sync::Mutex<Vec<String>>,
}

impl LruStrategy {
    /// Create new LRU strategy
    pub fn new(max_entries: usize) -> Self {
        Self {
            max_entries,
            access_order: std::sync::Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl CacheStrategy for LruStrategy {
    async fn should_invalidate(
        &self,
        _key: &str,
        _created_at: SystemTime,
        metadata: &HashMap<String, serde_json::Value>,
    ) -> Result<bool> {
        // LRU strategy doesn't invalidate based on time/metadata
        // It provides eviction hints to the cache manager
        Ok(false)
    }

    async fn on_cache_hit(&self, key: &str) -> Result<()> {
        let mut access_order = self.access_order.lock().unwrap();
        // Move key to end (most recently used)
        access_order.retain(|k| k != key);
        access_order.push(key.to_string());

        // If we exceed max entries, mark oldest for eviction
        if access_order.len() > self.max_entries {
            access_order.remove(0);
        }

        Ok(())
    }

    async fn on_cache_miss(&self, key: &str) -> Result<()> {
        let mut access_order = self.access_order.lock().unwrap();
        access_order.push(key.to_string());

        if access_order.len() > self.max_entries {
            access_order.remove(0);
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "lru"
    }
}

/// Size-based eviction strategy
pub struct SizeLimitStrategy {
    max_size_bytes: u64,
}

impl SizeLimitStrategy {
    /// Create new size limit strategy
    pub fn new(max_size_bytes: u64) -> Self {
        Self { max_size_bytes }
    }

    /// Create with size in megabytes
    pub fn with_mb(size_mb: u64) -> Self {
        Self::new(size_mb * 1024 * 1024)
    }

    /// Create with size in gigabytes
    pub fn with_gb(size_gb: u64) -> Self {
        Self::new(size_gb * 1024 * 1024 * 1024)
    }
}

#[async_trait]
impl CacheStrategy for SizeLimitStrategy {
    async fn should_invalidate(
        &self,
        _key: &str,
        _created_at: SystemTime,
        metadata: &HashMap<String, serde_json::Value>,
    ) -> Result<bool> {
        // Check if current total size exceeds limit
        if let Some(size_value) = metadata.get("total_size_bytes") {
            if let Ok(total_size) = serde_json::from_value::<u64>(size_value.clone()) {
                return Ok(total_size > self.max_size_bytes);
            }
        }
        Ok(false)
    }

    fn name(&self) -> &str {
        "size_limit"
    }
}

/// Composite strategy that combines multiple strategies
pub struct CompositeStrategy {
    strategies: Vec<Box<dyn CacheStrategy>>,
}

impl CompositeStrategy {
    /// Create new composite strategy
    pub fn new() -> Self {
        Self {
            strategies: Vec::new(),
        }
    }

    /// Add a strategy to the composite
    pub fn add_strategy<S: CacheStrategy + 'static>(mut self, strategy: S) -> Self {
        self.strategies.push(Box::new(strategy));
        self
    }

    /// Create composite with multiple strategies
    pub fn with_strategies(strategies: Vec<Box<dyn CacheStrategy>>) -> Self {
        Self { strategies }
    }
}

#[async_trait]
impl CacheStrategy for CompositeStrategy {
    async fn should_invalidate(
        &self,
        key: &str,
        created_at: SystemTime,
        metadata: &HashMap<String, serde_json::Value>,
    ) -> Result<bool> {
        // Invalidate if ANY strategy says to invalidate
        for strategy in &self.strategies {
            if strategy
                .should_invalidate(key, created_at, metadata)
                .await?
            {
                return Ok(true);
            }
        }
        Ok(false)
    }

    async fn on_cache_hit(&self, key: &str) -> Result<()> {
        for strategy in &self.strategies {
            strategy.on_cache_hit(key).await?;
        }
        Ok(())
    }

    async fn on_cache_miss(&self, key: &str) -> Result<()> {
        for strategy in &self.strategies {
            strategy.on_cache_miss(key).await?;
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "composite"
    }
}

#[cfg(test)]
mod tests {
    use tempfile::NamedTempFile;

    use super::*;

    #[tokio::test]
    async fn test_ttl_strategy() {
        let strategy = TtlStrategy::with_seconds(1);

        let past_time = SystemTime::now() - std::time::Duration::from_secs(2);
        let metadata = HashMap::new();

        // Should invalidate old entry
        assert!(strategy
            .should_invalidate("key", past_time, &metadata)
            .await
            .unwrap());

        let recent_time = SystemTime::now() - std::time::Duration::from_millis(500);
        // Should not invalidate recent entry
        assert!(!strategy
            .should_invalidate("key", recent_time, &metadata)
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn test_file_change_strategy() {
        let mut strategy = FileChangeStrategy::new();

        // Create a temporary file
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_path_buf();

        strategy.monitor_file(&file_path).unwrap();

        // Initially no changes
        let metadata = HashMap::new();
        assert!(!strategy
            .should_invalidate("key", SystemTime::now(), &metadata)
            .await
            .unwrap());

        // Modify file
        std::fs::write(&file_path, "modified content").unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(10)).await; // Ensure mtime changes

        // Create metadata with file mtimes
        let mut file_mtimes = HashMap::new();
        file_mtimes.insert(
            file_path.to_string_lossy().to_string(),
            SystemTime::now() - std::time::Duration::from_secs(60),
        );
        let mut metadata = HashMap::new();
        metadata.insert(
            "file_mtimes".to_string(),
            serde_json::to_value(file_mtimes).unwrap(),
        );

        // Should detect file change
        assert!(strategy
            .should_invalidate("key", SystemTime::now(), &metadata)
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn test_lru_strategy() {
        let strategy = LruStrategy::new(2);

        // Simulate cache operations
        strategy.on_cache_miss("key1").await.unwrap();
        strategy.on_cache_miss("key2").await.unwrap();
        strategy.on_cache_hit("key1").await.unwrap(); // key1 becomes most recent
        strategy.on_cache_miss("key3").await.unwrap(); // key2 should be evicted

        // Check access order (this is internal, but we can verify the strategy doesn't error)
        let metadata = HashMap::new();
        assert!(!strategy
            .should_invalidate("key1", SystemTime::now(), &metadata)
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn test_size_limit_strategy() {
        let strategy = SizeLimitStrategy::new(1000); // 1000 bytes limit

        let mut metadata = HashMap::new();
        metadata.insert("total_size_bytes".to_string(), serde_json::json!(500));
        assert!(!strategy
            .should_invalidate("key", SystemTime::now(), &metadata)
            .await
            .unwrap());

        metadata.insert("total_size_bytes".to_string(), serde_json::json!(1500));
        assert!(strategy
            .should_invalidate("key", SystemTime::now(), &metadata)
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn test_composite_strategy() {
        let ttl_strategy = TtlStrategy::with_seconds(1);
        let size_strategy = SizeLimitStrategy::new(1000);

        let strategy = CompositeStrategy::new()
            .add_strategy(ttl_strategy)
            .add_strategy(size_strategy);

        let past_time = SystemTime::now() - std::time::Duration::from_secs(2);
        let metadata = HashMap::new();

        // Should invalidate due to TTL
        assert!(strategy
            .should_invalidate("key", past_time, &metadata)
            .await
            .unwrap());
    }
}
