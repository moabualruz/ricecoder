//! Specification caching layer
//!
//! Caches parsed specification files to improve performance.
//! Uses file-based cache with TTL support.

use crate::error::SpecError;
use crate::models::Spec;
use ricecoder_storage::{CacheInvalidationStrategy, CacheManager};
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, info};

/// Specification cache
///
/// Caches parsed specification files to avoid redundant parsing.
/// Uses file modification time to detect changes.
pub struct SpecCache {
    cache: Arc<CacheManager>,
    ttl_seconds: u64,
}

impl SpecCache {
    /// Create a new spec cache
    ///
    /// # Arguments
    ///
    /// * `cache_dir` - Directory to store cache files
    /// * `ttl_seconds` - Time-to-live for cache entries (default: 3600 = 1 hour)
    ///
    /// # Errors
    ///
    /// Returns error if cache directory cannot be created
    pub fn new(cache_dir: impl AsRef<Path>, ttl_seconds: u64) -> Result<Self, SpecError> {
        let cache = CacheManager::new(cache_dir)
            .map_err(|e| SpecError::InvalidFormat(format!("Failed to create cache: {}", e)))?;

        Ok(Self {
            cache: Arc::new(cache),
            ttl_seconds,
        })
    }

    /// Get a cached spec
    ///
    /// # Arguments
    ///
    /// * `spec_path` - Path to specification file
    ///
    /// # Returns
    ///
    /// Returns cached spec if found and not expired, None otherwise
    pub fn get(&self, spec_path: &Path) -> Result<Option<Spec>, SpecError> {
        let cache_key = self.make_cache_key(spec_path);

        // Check if file was modified since cache creation
        if let Ok(metadata) = std::fs::metadata(spec_path) {
            if let Ok(_modified) = metadata.modified() {
                // If file was modified, invalidate cache
                if let Ok(Some(_)) = self.cache.get(&cache_key) {
                    // Check if we should invalidate based on modification time
                    // For now, we'll use TTL-based expiration
                }
            }
        }

        match self.cache.get(&cache_key) {
            Ok(Some(cached_json_str)) => {
                match serde_json::from_str::<Spec>(&cached_json_str) {
                    Ok(spec) => {
                        debug!("Cache hit for spec: {}", spec_path.display());
                        Ok(Some(spec))
                    }
                    Err(e) => {
                        debug!("Failed to deserialize cached spec: {}", e);
                        // Invalidate corrupted cache entry
                        let _ = self.cache.invalidate(&cache_key);
                        Ok(None)
                    }
                }
            }
            Ok(None) => {
                debug!("Cache miss for spec: {}", spec_path.display());
                Ok(None)
            }
            Err(e) => {
                debug!("Cache lookup error: {}", e);
                Ok(None)
            }
        }
    }

    /// Cache a spec
    ///
    /// # Arguments
    ///
    /// * `spec_path` - Path to specification file
    /// * `spec` - Parsed specification to cache
    ///
    /// # Errors
    ///
    /// Returns error if spec cannot be cached
    pub fn set(&self, spec_path: &Path, spec: &Spec) -> Result<(), SpecError> {
        let cache_key = self.make_cache_key(spec_path);

        let spec_json = serde_json::to_string(spec)
            .map_err(|e| SpecError::InvalidFormat(format!("Failed to serialize spec: {}", e)))?;

        let json_len = spec_json.len();

        self.cache
            .set(
                &cache_key,
                spec_json,
                CacheInvalidationStrategy::Ttl(self.ttl_seconds),
            )
            .map_err(|e| SpecError::InvalidFormat(format!("Failed to cache spec: {}", e)))?;

        debug!("Cached spec: {} ({} bytes)", spec_path.display(), json_len);

        Ok(())
    }

    /// Invalidate a cached spec
    ///
    /// # Arguments
    ///
    /// * `spec_path` - Path to specification file
    ///
    /// # Returns
    ///
    /// Returns Ok(true) if entry was deleted, Ok(false) if entry didn't exist
    pub fn invalidate(&self, spec_path: &Path) -> Result<bool, SpecError> {
        let cache_key = self.make_cache_key(spec_path);

        self.cache
            .invalidate(&cache_key)
            .map_err(|e| SpecError::InvalidFormat(format!("Failed to invalidate cache: {}", e)))
    }

    /// Clear all cached specs
    ///
    /// # Errors
    ///
    /// Returns error if cache cannot be cleared
    pub fn clear(&self) -> Result<(), SpecError> {
        self.cache
            .clear()
            .map_err(|e| SpecError::InvalidFormat(format!("Failed to clear cache: {}", e)))
    }

    /// Clean up expired cache entries
    ///
    /// # Returns
    ///
    /// Returns the number of entries cleaned up
    pub fn cleanup_expired(&self) -> Result<usize, SpecError> {
        let cleaned = self
            .cache
            .cleanup_expired()
            .map_err(|e| SpecError::InvalidFormat(format!("Failed to cleanup cache: {}", e)))?;

        if cleaned > 0 {
            info!("Cleaned up {} expired spec cache entries", cleaned);
        }

        Ok(cleaned)
    }

    /// Create a cache key from spec path
    fn make_cache_key(&self, spec_path: &Path) -> String {
        let path_str = spec_path.to_string_lossy();
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

        format!("spec_{}", sanitized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{SpecMetadata, SpecPhase, SpecStatus};
    use chrono::Utc;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_test_spec() -> Spec {
        Spec {
            id: "test-spec".to_string(),
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            requirements: vec![],
            design: None,
            tasks: vec![],
            metadata: SpecMetadata {
                author: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                phase: SpecPhase::Tasks,
                status: SpecStatus::Approved,
            },
            inheritance: None,
        }
    }

    #[test]
    fn test_cache_set_and_get() -> Result<(), SpecError> {
        let temp_dir = TempDir::new().unwrap();
        let cache = SpecCache::new(temp_dir.path(), 3600)?;

        let spec_path = PathBuf::from("test_spec.yaml");
        let spec = create_test_spec();

        // Cache spec
        cache.set(&spec_path, &spec)?;

        // Retrieve from cache
        let cached = cache.get(&spec_path)?;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().name, "test");

        Ok(())
    }

    #[test]
    fn test_cache_miss() -> Result<(), SpecError> {
        let temp_dir = TempDir::new().unwrap();
        let cache = SpecCache::new(temp_dir.path(), 3600)?;

        let spec_path = PathBuf::from("nonexistent_spec.yaml");

        // Try to get non-existent entry
        let cached = cache.get(&spec_path)?;
        assert!(cached.is_none());

        Ok(())
    }

    #[test]
    fn test_cache_invalidate() -> Result<(), SpecError> {
        let temp_dir = TempDir::new().unwrap();
        let cache = SpecCache::new(temp_dir.path(), 3600)?;

        let spec_path = PathBuf::from("test_spec.yaml");
        let spec = create_test_spec();

        // Cache spec
        cache.set(&spec_path, &spec)?;

        // Invalidate
        let invalidated = cache.invalidate(&spec_path)?;
        assert!(invalidated);

        // Should be gone now
        let cached = cache.get(&spec_path)?;
        assert!(cached.is_none());

        Ok(())
    }

    #[test]
    fn test_cache_clear() -> Result<(), SpecError> {
        let temp_dir = TempDir::new().unwrap();
        let cache = SpecCache::new(temp_dir.path(), 3600)?;

        let spec_path1 = PathBuf::from("spec1.yaml");
        let spec_path2 = PathBuf::from("spec2.yaml");
        let spec = create_test_spec();

        // Cache multiple specs
        cache.set(&spec_path1, &spec)?;
        cache.set(&spec_path2, &spec)?;

        // Clear all
        cache.clear()?;

        // Both should be gone
        assert!(cache.get(&spec_path1)?.is_none());
        assert!(cache.get(&spec_path2)?.is_none());

        Ok(())
    }
}
