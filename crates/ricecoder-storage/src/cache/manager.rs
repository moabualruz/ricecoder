//! Cache manager implementation for RiceCoder storage
//!
//! Provides file-based cache storage with TTL and manual invalidation strategies.
//! Adapted from automation/src/infrastructure/cache/cache_manager.rs

use crate::error::{IoOperation, StorageError, StorageResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, warn};

/// Cache invalidation strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheInvalidationStrategy {
    /// Time-to-live: cache expires after specified duration (in seconds)
    #[serde(rename = "ttl")]
    Ttl(u64),
    /// Manual: cache must be explicitly invalidated
    #[serde(rename = "manual")]
    Manual,
}

/// Cache entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Cached data
    pub data: String,
    /// Timestamp when entry was created
    pub created_at: u64,
    /// Invalidation strategy
    pub strategy: CacheInvalidationStrategy,
}

impl CacheEntry {
    /// Create a new cache entry
    pub fn new(data: String, strategy: CacheInvalidationStrategy) -> Self {
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            data,
            created_at,
            strategy,
        }
    }

    /// Check if the entry has expired
    pub fn is_expired(&self) -> bool {
        match self.strategy {
            CacheInvalidationStrategy::Ttl(ttl_secs) => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                now > self.created_at + ttl_secs
            }
            CacheInvalidationStrategy::Manual => false,
        }
    }
}

/// File-based cache manager
///
/// Stores cache entries as JSON files in a cache directory.
/// Supports TTL and manual invalidation strategies.
pub struct CacheManager {
    /// Cache directory path
    cache_dir: PathBuf,
}

impl CacheManager {
    /// Create a new cache manager
    ///
    /// # Arguments
    ///
    /// * `cache_dir` - Directory to store cache files
    ///
    /// # Errors
    ///
    /// Returns error if cache directory cannot be created
    pub fn new(cache_dir: impl AsRef<Path>) -> StorageResult<Self> {
        let cache_dir = cache_dir.as_ref().to_path_buf();

        // Create cache directory if it doesn't exist
        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir)
                .map_err(|e| StorageError::directory_creation_failed(cache_dir.clone(), e))?;
            debug!("Created cache directory: {}", cache_dir.display());
        }

        Ok(Self { cache_dir })
    }

    /// Get a cached value
    ///
    /// # Arguments
    ///
    /// * `key` - Cache key
    ///
    /// # Returns
    ///
    /// Returns the cached data if found and not expired, None if not found or expired
    pub fn get(&self, key: &str) -> StorageResult<Option<String>> {
        let path = self.key_to_path(key);

        if !path.exists() {
            debug!("Cache miss for key: {}", key);
            return Ok(None);
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| StorageError::io_error(path.clone(), IoOperation::Read, e))?;

        let entry: CacheEntry = serde_json::from_str(&content).map_err(|e| {
            StorageError::parse_error(
                path.clone(),
                "JSON",
                format!("Failed to deserialize cache entry: {}", e),
            )
        })?;

        if entry.is_expired() {
            debug!("Cache expired for key: {}", key);
            // Delete expired entry
            let _ = fs::remove_file(&path);
            return Ok(None);
        }

        debug!("Cache hit for key: {}", key);
        Ok(Some(entry.data))
    }

    /// Set a cached value
    ///
    /// # Arguments
    ///
    /// * `key` - Cache key
    /// * `data` - Data to cache
    /// * `strategy` - Invalidation strategy
    ///
    /// # Errors
    ///
    /// Returns error if cache entry cannot be written
    pub fn set(
        &self,
        key: &str,
        data: String,
        strategy: CacheInvalidationStrategy,
    ) -> StorageResult<()> {
        let path = self.key_to_path(key);

        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| {
                    StorageError::directory_creation_failed(parent.to_path_buf(), e)
                })?;
            }
        }

        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let entry = CacheEntry {
            data,
            created_at,
            strategy,
        };

        let json = serde_json::to_string_pretty(&entry).map_err(|e| {
            StorageError::parse_error(
                path.clone(),
                "JSON",
                format!("Failed to serialize cache entry: {}", e),
            )
        })?;

        fs::write(&path, json)
            .map_err(|e| StorageError::io_error(path.clone(), IoOperation::Write, e))?;

        debug!("Cached value for key: {}", key);
        Ok(())
    }

    /// Invalidate a cached value
    ///
    /// # Arguments
    ///
    /// * `key` - Cache key to invalidate
    ///
    /// # Returns
    ///
    /// Returns Ok(true) if entry was deleted, Ok(false) if entry didn't exist
    pub fn invalidate(&self, key: &str) -> StorageResult<bool> {
        let path = self.key_to_path(key);

        if !path.exists() {
            debug!("Cache entry not found for invalidation: {}", key);
            return Ok(false);
        }

        fs::remove_file(&path)
            .map_err(|e| StorageError::io_error(path.clone(), IoOperation::Delete, e))?;

        debug!("Invalidated cache for key: {}", key);
        Ok(true)
    }

    /// Check if a key exists in cache and is not expired
    ///
    /// # Arguments
    ///
    /// * `key` - Cache key
    pub fn exists(&self, key: &str) -> StorageResult<bool> {
        let path = self.key_to_path(key);

        if !path.exists() {
            return Ok(false);
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| StorageError::io_error(path.clone(), IoOperation::Read, e))?;

        let entry: CacheEntry = serde_json::from_str(&content).map_err(|e| {
            StorageError::parse_error(
                path.clone(),
                "JSON",
                format!("Failed to deserialize cache entry: {}", e),
            )
        })?;

        Ok(!entry.is_expired())
    }

    /// Clear all cache entries
    ///
    /// # Errors
    ///
    /// Returns error if cache directory cannot be cleared
    pub fn clear(&self) -> StorageResult<()> {
        if !self.cache_dir.exists() {
            return Ok(());
        }

        fs::remove_dir_all(&self.cache_dir)
            .map_err(|e| StorageError::io_error(self.cache_dir.clone(), IoOperation::Delete, e))?;

        fs::create_dir_all(&self.cache_dir)
            .map_err(|e| StorageError::directory_creation_failed(self.cache_dir.clone(), e))?;

        debug!("Cleared all cache entries");
        Ok(())
    }

    /// Clean up expired entries
    ///
    /// Scans the cache directory and removes all expired entries.
    ///
    /// # Returns
    ///
    /// Returns the number of entries cleaned up
    pub fn cleanup_expired(&self) -> StorageResult<usize> {
        if !self.cache_dir.exists() {
            return Ok(0);
        }

        let mut cleaned = 0;

        for entry in fs::read_dir(&self.cache_dir)
            .map_err(|e| StorageError::io_error(self.cache_dir.clone(), IoOperation::Read, e))?
        {
            let entry = entry.map_err(|e| {
                StorageError::io_error(self.cache_dir.clone(), IoOperation::Read, e)
            })?;

            let path = entry.path();

            if path.is_file() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(cache_entry) = serde_json::from_str::<CacheEntry>(&content) {
                        if cache_entry.is_expired() {
                            if let Err(e) = fs::remove_file(&path) {
                                warn!("Failed to remove expired cache entry: {}", e);
                            } else {
                                cleaned += 1;
                                debug!("Cleaned up expired cache entry: {}", path.display());
                            }
                        }
                    }
                }
            }
        }

        debug!("Cleaned up {} expired cache entries", cleaned);
        Ok(cleaned)
    }

    /// Convert a cache key to a file path
    fn key_to_path(&self, key: &str) -> PathBuf {
        // Sanitize key to create valid filename
        let sanitized = key
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '_' || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect::<String>();

        self.cache_dir.join(format!("{}.json", sanitized))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tempfile::TempDir;

    #[test]
    fn test_cache_set_and_get() -> StorageResult<()> {
        let temp_dir = TempDir::new().unwrap();
        let cache = CacheManager::new(temp_dir.path())?;

        cache.set(
            "test_key",
            "test_data".to_string(),
            CacheInvalidationStrategy::Manual,
        )?;

        let result = cache.get("test_key")?;
        assert_eq!(result, Some("test_data".to_string()));

        Ok(())
    }

    #[test]
    fn test_cache_not_found() -> StorageResult<()> {
        let temp_dir = TempDir::new().unwrap();
        let cache = CacheManager::new(temp_dir.path())?;

        let result = cache.get("nonexistent")?;
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_cache_invalidate() -> StorageResult<()> {
        let temp_dir = TempDir::new().unwrap();
        let cache = CacheManager::new(temp_dir.path())?;

        cache.set(
            "test_key",
            "test_data".to_string(),
            CacheInvalidationStrategy::Manual,
        )?;

        let invalidated = cache.invalidate("test_key")?;
        assert!(invalidated);

        let result = cache.get("test_key")?;
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_cache_exists() -> StorageResult<()> {
        let temp_dir = TempDir::new().unwrap();
        let cache = CacheManager::new(temp_dir.path())?;

        cache.set(
            "test_key",
            "test_data".to_string(),
            CacheInvalidationStrategy::Manual,
        )?;

        assert!(cache.exists("test_key")?);
        assert!(!cache.exists("nonexistent")?);

        Ok(())
    }

    #[test]
    fn test_cache_clear() -> StorageResult<()> {
        let temp_dir = TempDir::new().unwrap();
        let cache = CacheManager::new(temp_dir.path())?;

        cache.set(
            "key1",
            "data1".to_string(),
            CacheInvalidationStrategy::Manual,
        )?;
        cache.set(
            "key2",
            "data2".to_string(),
            CacheInvalidationStrategy::Manual,
        )?;

        cache.clear()?;

        assert!(!cache.exists("key1")?);
        assert!(!cache.exists("key2")?);

        Ok(())
    }

    #[test]
    fn test_cache_ttl_expiration() -> StorageResult<()> {
        let temp_dir = TempDir::new().unwrap();
        let cache = CacheManager::new(temp_dir.path())?;

        // Set cache with very short TTL (1 second)
        cache.set(
            "test_key",
            "test_data".to_string(),
            CacheInvalidationStrategy::Ttl(1),
        )?;

        // Should exist immediately
        assert!(cache.exists("test_key")?);

        // Wait for expiration
        std::thread::sleep(Duration::from_secs(2));

        // Should be expired now
        let result = cache.get("test_key")?;
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_cache_cleanup_expired() -> StorageResult<()> {
        let temp_dir = TempDir::new().unwrap();
        let cache = CacheManager::new(temp_dir.path())?;

        // Set cache with short TTL (1 second)
        cache.set(
            "expired_key",
            "data".to_string(),
            CacheInvalidationStrategy::Ttl(1),
        )?;

        // Set cache with manual invalidation (won't expire)
        cache.set(
            "manual_key",
            "data".to_string(),
            CacheInvalidationStrategy::Manual,
        )?;

        // Wait for first entry to expire
        std::thread::sleep(Duration::from_secs(2));

        // Cleanup should remove only expired entries
        let cleaned = cache.cleanup_expired()?;
        assert_eq!(cleaned, 1);

        // Manual entry should still exist
        assert!(cache.exists("manual_key")?);

        Ok(())
    }
}
