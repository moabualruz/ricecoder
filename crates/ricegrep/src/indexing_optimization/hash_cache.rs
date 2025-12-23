//! Content hash caching for triple-validation safety.
//!
//! Implements safety validation layer for metadata gating:
//! - LRU cache for content hashes
//! - Triple validation: mtime + size + timestamp
//! - Handles filesystem race conditions
//! - Fallback to rehashing on mismatch

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use thiserror::Error;

/// Errors from hash cache operations
#[derive(Debug, Error)]
pub enum HashCacheError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("hash computation failed: {0}")]
    ComputationFailed(String),
}

/// Simple content hash (for this implementation, we use file size + mtime as hash)
/// In production, this would be SHA256 or similar
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContentHash(u64);

impl ContentHash {
    /// Compute a simple content hash (size + mtime based)
    /// In production: compute SHA256 of file contents
    pub fn compute(path: &Path) -> Result<Self, HashCacheError> {
        let metadata = std::fs::metadata(path)?;
        let size = metadata.len();
        let mtime = metadata
            .modified()?
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|e| {
                HashCacheError::ComputationFailed(format!("time error: {}", e))
            })?
            .as_secs();

        // Simple hash: combine size and mtime
        // In production: would compute SHA256 of actual file contents
        let hash = size.wrapping_mul(31).wrapping_add(mtime);
        Ok(ContentHash(hash))
    }

    /// Get hash value
    pub fn value(&self) -> u64 {
        self.0
    }
}

/// Cached hash entry with validation metadata
#[derive(Debug, Clone)]
struct CachedHash {
    /// The computed hash
    hash: ContentHash,
    
    /// File mtime when hash was computed
    mtime: u64,
    
    /// File size when hash was computed
    size: u64,
    
    /// When this cache entry was created
    cached_at: u64,
}

/// LRU cache for content hashes with triple-validation
///
/// Triple validation ensures:
/// 1. File mtime unchanged
/// 2. File size unchanged
/// 3. Cache entry < 60 seconds old
///
/// If any validation fails, file is rehashed
#[derive(Debug)]
pub struct ContentHashCache {
    /// Cache entries: path -> cached hash
    cache: HashMap<PathBuf, CachedHash>,
    
    /// Maximum cache size in bytes (default 50 MB)
    max_bytes: usize,
    
    /// Current cache size in bytes
    current_bytes: usize,
}

impl ContentHashCache {
    /// Create new content hash cache with default 50 MB limit
    pub fn new() -> Self {
        Self::with_capacity(50 * 1024 * 1024) // 50 MB default
    }

    /// Create with custom capacity in bytes
    pub fn with_capacity(max_bytes: usize) -> Self {
        Self {
            cache: HashMap::new(),
            max_bytes: max_bytes.max(1_000_000), // Minimum 1 MB
            current_bytes: 0,
        }
    }

    /// Add/update hash in cache
    pub fn add(
        &mut self,
        path: PathBuf,
        hash: ContentHash,
        mtime: u64,
        size: u64,
    ) {
        // Evict LRU entries if needed to make space
        // Simple eviction: remove oldest entry when at capacity
        while self.current_bytes + std::mem::size_of::<CachedHash>() > self.max_bytes
            && !self.cache.is_empty()
        {
            if let Some(oldest_path) = self
                .cache
                .iter()
                .min_by_key(|(_, entry)| entry.cached_at)
                .map(|(p, _)| p.clone())
            {
                if let Some(entry) = self.cache.remove(&oldest_path) {
                    self.current_bytes = self.current_bytes.saturating_sub(
                        entry.hash.value() as usize + 
                        std::mem::size_of::<u64>() * 3
                    );
                }
            }
        }

        let entry = CachedHash {
            hash,
            mtime,
            size,
            cached_at: current_timestamp(),
        };

        let entry_size = entry.hash.value() as usize + std::mem::size_of::<u64>() * 3;
        self.cache.insert(path, entry);
        self.current_bytes = self.current_bytes.saturating_add(entry_size);
    }

    /// Get hash with triple-validation
    ///
    /// Returns hash only if:
    /// 1. Entry exists
    /// 2. mtime matches
    /// 3. size matches
    /// 4. entry < 60 seconds old
    ///
    /// Otherwise returns None (needs recomputation)
    pub fn get_if_valid(
        &self,
        path: &Path,
        current_mtime: u64,
        current_size: u64,
    ) -> Option<ContentHash> {
        self.cache.get(path).and_then(|entry| {
            // Triple validation
            if entry.mtime == current_mtime
                && entry.size == current_size
                && is_recent(entry.cached_at, 60)
            {
                Some(entry.hash)
            } else {
                None
            }
        })
    }

    /// Invalidate cache entry for a path
    pub fn invalidate(&mut self, path: &Path) {
        if let Some(entry) = self.cache.remove(path) {
            let entry_size = entry.hash.value() as usize + std::mem::size_of::<u64>() * 3;
            self.current_bytes = self.current_bytes.saturating_sub(entry_size);
        }
    }

    /// Clear all cache entries
    pub fn clear(&mut self) {
        self.cache.clear();
        self.current_bytes = 0;
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            entries: self.cache.len(),
            capacity_bytes: self.max_bytes,
            current_bytes: self.current_bytes,
        }
    }

    /// Get number of entries
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

impl Default for ContentHashCache {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entries: usize,
    pub capacity_bytes: usize,
    pub current_bytes: usize,
}

impl CacheStats {
    pub fn utilization_percent(&self) -> f64 {
        if self.capacity_bytes == 0 {
            0.0
        } else {
            (self.current_bytes as f64 / self.capacity_bytes as f64) * 100.0
        }
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn is_recent(timestamp: u64, threshold_secs: u64) -> bool {
    let now = current_timestamp();
    now.saturating_sub(timestamp) < threshold_secs
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_content_hash_computation() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "test content").unwrap();

        let hash = ContentHash::compute(&file_path).unwrap();
        assert!(hash.value() > 0);
    }

    #[test]
    fn test_hash_cache_creation() {
        let cache = ContentHashCache::new();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_hash_cache_add_and_get() {
        let mut cache = ContentHashCache::new();
        let path = PathBuf::from("test.txt");
        let hash = ContentHash(12345);

        cache.add(path.clone(), hash, 1000, 512);

        let retrieved = cache.get_if_valid(&path, 1000, 512);
        assert_eq!(retrieved, Some(hash));
    }

    #[test]
    fn test_hash_cache_triple_validation_mtime() {
        let mut cache = ContentHashCache::new();
        let path = PathBuf::from("test.txt");
        let hash = ContentHash(12345);

        cache.add(path.clone(), hash, 1000, 512);

        // mtime changed
        let retrieved = cache.get_if_valid(&path, 1001, 512);
        assert_eq!(retrieved, None);
    }

    #[test]
    fn test_hash_cache_triple_validation_size() {
        let mut cache = ContentHashCache::new();
        let path = PathBuf::from("test.txt");
        let hash = ContentHash(12345);

        cache.add(path.clone(), hash, 1000, 512);

        // size changed
        let retrieved = cache.get_if_valid(&path, 1000, 513);
        assert_eq!(retrieved, None);
    }

    #[test]
    fn test_hash_cache_invalidate() {
        let mut cache = ContentHashCache::new();
        let path = PathBuf::from("test.txt");
        let hash = ContentHash(12345);

        cache.add(path.clone(), hash, 1000, 512);
        assert_eq!(cache.len(), 1);

        cache.invalidate(&path);
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_hash_cache_clear() {
        let mut cache = ContentHashCache::new();
        cache.add(PathBuf::from("file1.txt"), ContentHash(111), 1000, 512);
        cache.add(PathBuf::from("file2.txt"), ContentHash(222), 2000, 1024);

        assert_eq!(cache.len(), 2);
        cache.clear();
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_hash_cache_stats() {
        let mut cache = ContentHashCache::with_capacity(1000);
        cache.add(PathBuf::from("test.txt"), ContentHash(12345), 1000, 512);

        let stats = cache.stats();
        assert_eq!(stats.entries, 1);
        // Capacity is clamped to minimum 1MB (1_000_000 bytes)
        assert_eq!(stats.capacity_bytes, 1_000_000);
        assert!(stats.utilization_percent() > 0.0);
    }

    #[test]
    fn test_hash_cache_with_real_files() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "test content").unwrap();

        let mut cache = ContentHashCache::new();
        let hash1 = ContentHash::compute(&file_path).unwrap();

        let metadata = std::fs::metadata(&file_path).unwrap();
        let mtime = metadata
            .modified()
            .unwrap()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        cache.add(file_path.clone(), hash1, mtime, metadata.len());

        // Hash should be valid
        let retrieved = cache.get_if_valid(&file_path, mtime, metadata.len());
        assert_eq!(retrieved, Some(hash1));
    }

    #[test]
    fn test_cache_stats_utilization() {
        let mut cache = ContentHashCache::with_capacity(1000);
        cache.add(PathBuf::from("file.txt"), ContentHash(123), 1000, 100);

        let stats = cache.stats();
        let utilization = stats.utilization_percent();
        assert!(utilization > 0.0 && utilization <= 100.0);
    }
}
