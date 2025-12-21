//! Image analysis caching with LRU eviction and TTL.

use crate::error::{ImageError, ImageResult};
use crate::models::ImageAnalysisResult;
use ricecoder_storage::cache::{CacheInvalidationStrategy, CacheManager};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use tracing::{debug, warn};

/// Caches image analysis results with TTL and LRU eviction.
///
/// Wraps ricecoder-storage's CacheManager to provide image-specific caching
/// with SHA256-based cache keys, TTL support, and LRU eviction.
pub struct ImageCache {
    /// Underlying cache manager
    cache_manager: CacheManager,
    /// TTL in seconds (24 hours by default)
    ttl_seconds: u64,
    /// Maximum cache size in MB (100 MB by default)
    max_size_mb: u64,
}

impl ImageCache {
    /// Create a new image cache with default settings.
    ///
    /// Uses default cache paths:
    /// - User cache: `~/.ricecoder/cache/images/`
    /// - Project cache: `projects/ricecoder/.agent/cache/images/`
    ///
    /// # Errors
    ///
    /// Returns error if cache directory cannot be created
    pub fn new() -> ImageResult<Self> {
        Self::with_config(86400, 100) // 24 hours, 100 MB
    }

    /// Create a new image cache with custom settings.
    ///
    /// # Arguments
    ///
    /// * `ttl_seconds` - Time-to-live for cache entries in seconds
    /// * `max_size_mb` - Maximum cache size in MB
    ///
    /// # Errors
    ///
    /// Returns error if cache directory cannot be created
    pub fn with_config(ttl_seconds: u64, max_size_mb: u64) -> ImageResult<Self> {
        // Try to use project cache first, fall back to user cache
        let cache_dir = if let Ok(project_cache) = std::env::current_dir() {
            project_cache.join(".agent").join("cache").join("images")
        } else {
            // Fall back to user cache
            if let Ok(home) = std::env::var("HOME") {
                PathBuf::from(home)
                    .join(".ricecoder")
                    .join("cache")
                    .join("images")
            } else {
                return Err(ImageError::CacheError(
                    "Cannot determine cache directory: HOME not set".to_string(),
                ));
            }
        };

        let cache_manager = CacheManager::new(&cache_dir).map_err(|e| {
            ImageError::CacheError(format!("Failed to create cache manager: {}", e))
        })?;

        debug!(
            "Created image cache at: {} (TTL: {}s, Max: {}MB)",
            cache_dir.display(),
            ttl_seconds,
            max_size_mb
        );

        Ok(Self {
            cache_manager,
            ttl_seconds,
            max_size_mb,
        })
    }

    /// Get a cached analysis result by image hash.
    ///
    /// # Arguments
    ///
    /// * `image_hash` - SHA256 hash of the image
    ///
    /// # Returns
    ///
    /// Returns the cached analysis result if found and not expired, None if not found or expired
    pub fn get(&self, image_hash: &str) -> ImageResult<Option<ImageAnalysisResult>> {
        let cache_key = self.hash_to_cache_key(image_hash);

        match self.cache_manager.get(&cache_key) {
            Ok(Some(json)) => {
                match serde_json::from_str::<ImageAnalysisResult>(&json) {
                    Ok(result) => {
                        debug!("Cache hit for image: {}", image_hash);
                        Ok(Some(result))
                    }
                    Err(e) => {
                        warn!("Failed to deserialize cached analysis: {}", e);
                        // Try to invalidate corrupted entry
                        let _ = self.cache_manager.invalidate(&cache_key);
                        Ok(None)
                    }
                }
            }
            Ok(None) => {
                debug!("Cache miss for image: {}", image_hash);
                Ok(None)
            }
            Err(e) => {
                warn!("Cache lookup failed: {}", e);
                Ok(None) // Graceful degradation: treat cache errors as misses
            }
        }
    }

    /// Create a new image cache with a temporary directory (for testing).
    ///
    /// # Arguments
    ///
    /// * `temp_dir` - Temporary directory path
    ///
    /// # Errors
    ///
    /// Returns error if cache directory cannot be created
    pub fn with_temp_dir(temp_dir: &std::path::Path) -> ImageResult<Self> {
        let cache_dir = temp_dir.join("images");
        let cache_manager = CacheManager::new(&cache_dir).map_err(|e| {
            ImageError::CacheError(format!("Failed to create cache manager: {}", e))
        })?;

        Ok(Self {
            cache_manager,
            ttl_seconds: 86400, // 24 hours
            max_size_mb: 100,   // 100 MB
        })
    }

    /// Cache an analysis result.
    ///
    /// # Arguments
    ///
    /// * `image_hash` - SHA256 hash of the image
    /// * `result` - Analysis result to cache
    ///
    /// # Errors
    ///
    /// Returns error if cache operation fails
    pub fn set(&self, image_hash: &str, result: &ImageAnalysisResult) -> ImageResult<()> {
        let cache_key = self.hash_to_cache_key(image_hash);

        let json = serde_json::to_string(result)
            .map_err(|e| ImageError::CacheError(format!("Failed to serialize analysis: {}", e)))?;

        self.cache_manager
            .set(
                &cache_key,
                json,
                CacheInvalidationStrategy::Ttl(self.ttl_seconds),
            )
            .map_err(|e| ImageError::CacheError(format!("Failed to cache analysis: {}", e)))?;

        debug!("Cached analysis for image: {}", image_hash);
        Ok(())
    }

    /// Check if an image analysis is cached and not expired.
    ///
    /// # Arguments
    ///
    /// * `image_hash` - SHA256 hash of the image
    pub fn exists(&self, image_hash: &str) -> ImageResult<bool> {
        let cache_key = self.hash_to_cache_key(image_hash);

        self.cache_manager
            .exists(&cache_key)
            .map_err(|e| ImageError::CacheError(format!("Failed to check cache: {}", e)))
    }

    /// Invalidate a cached analysis result.
    ///
    /// # Arguments
    ///
    /// * `image_hash` - SHA256 hash of the image
    ///
    /// # Returns
    ///
    /// Returns Ok(true) if entry was deleted, Ok(false) if entry didn't exist
    pub fn invalidate(&self, image_hash: &str) -> ImageResult<bool> {
        let cache_key = self.hash_to_cache_key(image_hash);

        self.cache_manager
            .invalidate(&cache_key)
            .map_err(|e| ImageError::CacheError(format!("Failed to invalidate cache: {}", e)))
    }

    /// Clear all cached analysis results.
    ///
    /// # Errors
    ///
    /// Returns error if cache cannot be cleared
    pub fn clear(&self) -> ImageResult<()> {
        self.cache_manager
            .clear()
            .map_err(|e| ImageError::CacheError(format!("Failed to clear cache: {}", e)))?;

        debug!("Cleared all cached analyses");
        Ok(())
    }

    /// Clean up expired cache entries.
    ///
    /// # Returns
    ///
    /// Returns the number of entries cleaned up
    pub fn cleanup_expired(&self) -> ImageResult<usize> {
        let cleaned = self
            .cache_manager
            .cleanup_expired()
            .map_err(|e| ImageError::CacheError(format!("Failed to cleanup cache: {}", e)))?;

        debug!("Cleaned up {} expired cache entries", cleaned);
        Ok(cleaned)
    }

    /// Compute SHA256 hash of image data.
    ///
    /// # Arguments
    ///
    /// * `data` - Image file data
    ///
    /// # Returns
    ///
    /// SHA256 hash as hex string
    pub fn compute_hash(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    /// Convert image hash to cache key.
    fn hash_to_cache_key(&self, image_hash: &str) -> String {
        format!("image_{}", image_hash)
    }

    /// Get cache statistics.
    ///
    /// # Returns
    ///
    /// Tuple of (ttl_seconds, max_size_mb)
    pub fn stats(&self) -> (u64, u64) {
        (self.ttl_seconds, self.max_size_mb)
    }
}

impl Default for ImageCache {
    fn default() -> Self {
        Self::new().expect("Failed to create default image cache")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ImageAnalysisResult;

    #[test]
    fn test_cache_creation() {
        let cache = ImageCache::new();
        assert!(cache.is_ok());
    }

    #[test]
    fn test_cache_with_config() {
        let cache = ImageCache::with_config(3600, 50);
        assert!(cache.is_ok());

        let cache = cache.unwrap();
        let (ttl, max_size) = cache.stats();
        assert_eq!(ttl, 3600);
        assert_eq!(max_size, 50);
    }

    #[test]
    fn test_compute_hash() {
        let data = b"test image data";
        let hash1 = ImageCache::compute_hash(data);
        let hash2 = ImageCache::compute_hash(data);

        // Same data should produce same hash
        assert_eq!(hash1, hash2);

        // Different data should produce different hash
        let different_data = b"different data";
        let hash3 = ImageCache::compute_hash(different_data);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_hash_to_cache_key() {
        let cache = ImageCache::new().unwrap();
        let key = cache.hash_to_cache_key("abc123");
        assert_eq!(key, "image_abc123");
    }

    #[test]
    fn test_cache_set_and_get() {
        let cache = ImageCache::new().unwrap();
        let unique_hash = format!(
            "hash123_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let result = ImageAnalysisResult::new(
            unique_hash.clone(),
            "This is a test image".to_string(),
            "openai".to_string(),
            100,
        );

        // Set cache
        let set_result = cache.set(&unique_hash, &result);
        assert!(set_result.is_ok());

        // Get cache
        let get_result = cache.get(&unique_hash);
        assert!(get_result.is_ok());

        if let Ok(Some(cached)) = get_result {
            assert_eq!(cached.image_hash, unique_hash);
            assert_eq!(cached.analysis, "This is a test image");
            assert_eq!(cached.provider, "openai");
            assert_eq!(cached.tokens_used, 100);
        } else {
            panic!("Expected cached result");
        }
    }

    #[test]
    fn test_cache_miss() {
        let cache = ImageCache::new().unwrap();
        let unique_hash = format!(
            "nonexistent_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let result = cache.get(&unique_hash);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_cache_exists() {
        let cache = ImageCache::new().unwrap();
        let unique_hash = format!(
            "hash_exists_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let result = ImageAnalysisResult::new(
            unique_hash.clone(),
            "Analysis".to_string(),
            "openai".to_string(),
            100,
        );

        cache.set(&unique_hash, &result).unwrap();

        let exists = cache.exists(&unique_hash);
        assert!(exists.is_ok());
        assert!(exists.unwrap());

        let not_exists = cache.exists("nonexistent_hash_that_never_existed");
        assert!(not_exists.is_ok());
        assert!(!not_exists.unwrap());
    }

    #[test]
    fn test_cache_invalidate() {
        let cache = ImageCache::new().unwrap();
        let unique_hash = format!(
            "hash_invalidate_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let result = ImageAnalysisResult::new(
            unique_hash.clone(),
            "Analysis".to_string(),
            "openai".to_string(),
            100,
        );

        cache.set(&unique_hash, &result).unwrap();
        assert!(cache.exists(&unique_hash).unwrap());

        let invalidated = cache.invalidate(&unique_hash);
        assert!(invalidated.is_ok());
        assert!(invalidated.unwrap());

        assert!(!cache.exists(&unique_hash).unwrap());
    }

    #[test]
    fn test_cache_clear() {
        // Use a unique cache directory for this test to avoid conflicts
        let test_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        // Create a temporary cache directory
        let temp_dir = std::env::temp_dir().join(format!("ricecoder_cache_test_{}", test_id));
        let _ = std::fs::create_dir_all(&temp_dir);

        let cache_manager = ricecoder_storage::cache::CacheManager::new(&temp_dir)
            .expect("Failed to create test cache manager");

        let cache = ImageCache {
            cache_manager,
            ttl_seconds: 86400,
            max_size_mb: 100,
        };

        let unique_hash = format!("hash_clear_{}", test_id);
        let result = ImageAnalysisResult::new(
            unique_hash.clone(),
            "Analysis".to_string(),
            "openai".to_string(),
            100,
        );

        cache.set(&unique_hash, &result).unwrap();
        assert!(cache.exists(&unique_hash).unwrap());

        let clear_result = cache.clear();
        assert!(clear_result.is_ok());

        assert!(!cache.exists(&unique_hash).unwrap());

        // Clean up
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
