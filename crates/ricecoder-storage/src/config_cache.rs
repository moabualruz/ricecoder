//! Configuration caching layer
//!
//! Caches parsed configuration files to improve performance.
//! Uses file-based cache with TTL support.

use std::{path::Path, sync::Arc};

use serde_json::Value;
use tracing::{debug, info};

use crate::{
    cache::{CacheInvalidationStrategy, CacheManager},
    error::{StorageError, StorageResult},
};

/// Configuration cache
///
/// Caches parsed configuration files to avoid redundant parsing.
/// Supports both global and project-level configuration caching.
pub struct ConfigCache {
    cache: Arc<CacheManager>,
    ttl_seconds: u64,
}

impl ConfigCache {
    /// Create a new config cache
    ///
    /// # Arguments
    ///
    /// * `cache_dir` - Directory to store cache files
    /// * `ttl_seconds` - Time-to-live for cache entries (default: 3600 = 1 hour)
    ///
    /// # Errors
    ///
    /// Returns error if cache directory cannot be created
    pub fn new(cache_dir: impl AsRef<Path>, ttl_seconds: u64) -> StorageResult<Self> {
        let cache = CacheManager::new(cache_dir)?;

        Ok(Self {
            cache: Arc::new(cache),
            ttl_seconds,
        })
    }

    /// Get a cached configuration
    ///
    /// # Arguments
    ///
    /// * `config_path` - Path to configuration file
    ///
    /// # Returns
    ///
    /// Returns cached configuration if found and not expired, None otherwise
    pub fn get(&self, config_path: &Path) -> StorageResult<Option<Value>> {
        let cache_key = self.make_cache_key(config_path);

        match self.cache.get(&cache_key) {
            Ok(Some(cached_json)) => {
                match serde_json::from_str::<Value>(&cached_json) {
                    Ok(config) => {
                        debug!("Cache hit for config: {}", config_path.display());
                        Ok(Some(config))
                    }
                    Err(e) => {
                        debug!("Failed to deserialize cached config: {}", e);
                        // Invalidate corrupted cache entry
                        let _ = self.cache.invalidate(&cache_key);
                        Ok(None)
                    }
                }
            }
            Ok(None) => {
                debug!("Cache miss for config: {}", config_path.display());
                Ok(None)
            }
            Err(e) => {
                debug!("Cache lookup error: {}", e);
                Ok(None)
            }
        }
    }

    /// Cache a configuration
    ///
    /// # Arguments
    ///
    /// * `config_path` - Path to configuration file
    /// * `config` - Parsed configuration to cache
    ///
    /// # Errors
    ///
    /// Returns error if configuration cannot be cached
    pub fn set(&self, config_path: &Path, config: &Value) -> StorageResult<()> {
        let cache_key = self.make_cache_key(config_path);

        let config_json = serde_json::to_string(config)
            .map_err(|e| StorageError::internal(format!("Failed to serialize config: {}", e)))?;

        let json_len = config_json.len();

        self.cache.set(
            &cache_key,
            config_json,
            CacheInvalidationStrategy::Ttl(self.ttl_seconds),
        )?;

        debug!(
            "Cached config: {} ({} bytes)",
            config_path.display(),
            json_len
        );

        Ok(())
    }

    /// Invalidate a cached configuration
    ///
    /// # Arguments
    ///
    /// * `config_path` - Path to configuration file
    ///
    /// # Returns
    ///
    /// Returns Ok(true) if entry was deleted, Ok(false) if entry didn't exist
    pub fn invalidate(&self, config_path: &Path) -> StorageResult<bool> {
        let cache_key = self.make_cache_key(config_path);
        self.cache.invalidate(&cache_key)
    }

    /// Clear all cached configurations
    ///
    /// # Errors
    ///
    /// Returns error if cache cannot be cleared
    pub fn clear(&self) -> StorageResult<()> {
        self.cache.clear()
    }

    /// Clean up expired cache entries
    ///
    /// # Returns
    ///
    /// Returns the number of entries cleaned up
    pub fn cleanup_expired(&self) -> StorageResult<usize> {
        let cleaned = self.cache.cleanup_expired()?;

        if cleaned > 0 {
            info!("Cleaned up {} expired config cache entries", cleaned);
        }

        Ok(cleaned)
    }

    /// Create a cache key from config path
    fn make_cache_key(&self, config_path: &Path) -> String {
        let path_str = config_path.to_string_lossy();
        let sanitized = path_str
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '_' || c == '-' || c == '.' {
                    c
                } else {
                    '_'
                }
            })
            .collect::<String>();

        format!("config_{}", sanitized)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_cache_set_and_get() -> StorageResult<()> {
        let temp_dir = TempDir::new().unwrap();
        let cache = ConfigCache::new(temp_dir.path(), 3600)?;

        let config_path = std::path::PathBuf::from("config.yaml");
        let config = serde_json::json!({
            "key": "value",
            "nested": {
                "setting": 42
            }
        });

        // Cache config
        cache.set(&config_path, &config)?;

        // Retrieve from cache
        let cached = cache.get(&config_path)?;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap()["key"], "value");

        Ok(())
    }

    #[test]
    fn test_cache_miss() -> StorageResult<()> {
        let temp_dir = TempDir::new().unwrap();
        let cache = ConfigCache::new(temp_dir.path(), 3600)?;

        let config_path = std::path::PathBuf::from("nonexistent.yaml");

        // Try to get non-existent entry
        let cached = cache.get(&config_path)?;
        assert!(cached.is_none());

        Ok(())
    }

    #[test]
    fn test_cache_invalidate() -> StorageResult<()> {
        let temp_dir = TempDir::new().unwrap();
        let cache = ConfigCache::new(temp_dir.path(), 3600)?;

        let config_path = std::path::PathBuf::from("config.yaml");
        let config = serde_json::json!({"key": "value"});

        // Cache config
        cache.set(&config_path, &config)?;

        // Invalidate
        let invalidated = cache.invalidate(&config_path)?;
        assert!(invalidated);

        // Should be gone now
        let cached = cache.get(&config_path)?;
        assert!(cached.is_none());

        Ok(())
    }

    #[test]
    fn test_cache_clear() -> StorageResult<()> {
        let temp_dir = TempDir::new().unwrap();
        let cache = ConfigCache::new(temp_dir.path(), 3600)?;

        let config_path1 = std::path::PathBuf::from("config1.yaml");
        let config_path2 = std::path::PathBuf::from("config2.yaml");
        let config = serde_json::json!({"key": "value"});

        // Cache multiple configs
        cache.set(&config_path1, &config)?;
        cache.set(&config_path2, &config)?;

        // Clear all
        cache.clear()?;

        // Both should be gone
        assert!(cache.get(&config_path1)?.is_none());
        assert!(cache.get(&config_path2)?.is_none());

        Ok(())
    }
}
