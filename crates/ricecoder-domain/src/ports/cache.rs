//! Cache port interfaces and value objects
//!
//! Cache Repository
//!
//! This module contains the contracts for caching operations.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::errors::*;

// ============================================================================
// Cache Value Objects
// ============================================================================

/// Cache entry metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntryInfo {
    /// Cache key
    pub key: String,
    /// Size in bytes
    pub size_bytes: u64,
    /// Creation time
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Time-to-live (if set)
    pub ttl: Option<std::time::Duration>,
    /// Whether entry is expired
    pub is_expired: bool,
}

/// Cache statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStatistics {
    /// Number of cache hits
    pub hits: u64,
    /// Number of cache misses
    pub misses: u64,
    /// Total number of entries
    pub entry_count: usize,
    /// Total size in bytes
    pub total_size_bytes: u64,
    /// Number of evictions
    pub evictions: u64,
    /// Average retrieval time in milliseconds
    pub avg_retrieval_time_ms: f64,
}

impl CacheStatistics {
    /// Calculate hit rate as a percentage
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        }
    }
}

// ============================================================================
// Cache Repository Ports (ISP-Compliant Split)
// ============================================================================

/// Read-only cache operations (ISP: 5 methods max)
///
///  Cache Repository
#[async_trait]
pub trait CacheReader: Send + Sync {
    /// Retrieve a value from the cache
    ///
    /// Returns None if key doesn't exist or is expired
    async fn get(&self, key: &str) -> DomainResult<Option<Vec<u8>>>;

    /// Check if a key exists and is not expired
    async fn contains(&self, key: &str) -> DomainResult<bool>;

    /// Get information about a specific cache entry
    async fn entry_info(&self, key: &str) -> DomainResult<Option<CacheEntryInfo>>;

    /// Get all cache keys (for debugging/monitoring)
    async fn keys(&self) -> DomainResult<Vec<String>>;

    /// Get cache statistics
    fn statistics(&self) -> CacheStatistics;
}

/// Write cache operations (ISP: 3 methods)
///
///  Cache Repository
#[async_trait]
pub trait CacheWriter: Send + Sync {
    /// Store a value in the cache
    ///
    /// # Arguments
    /// * `key` - Cache key
    /// * `value` - Serialized value as bytes
    /// * `ttl` - Optional time-to-live
    async fn set(&self, key: &str, value: Vec<u8>, ttl: Option<std::time::Duration>) -> DomainResult<()>;

    /// Remove a key from the cache
    async fn remove(&self, key: &str) -> DomainResult<bool>;

    /// Clear all entries from the cache
    async fn clear(&self) -> DomainResult<()>;
}

/// Combined cache repository (Reader + Writer)
///
/// Clients that need full cache operations can depend on this trait.
/// Clients with more focused needs should depend on role-specific traits:
/// - Read-only: `CacheReader`
/// - Write-only: `CacheWriter`
///
///  Cache Repository
pub trait CacheRepository: CacheReader + CacheWriter {}

/// Blanket implementation: Any type implementing Reader + Writer gets CacheRepository
impl<T: CacheReader + CacheWriter> CacheRepository for T {}

// ============================================================================
// File Watcher Port (Interface)
// ============================================================================

/// Port interface for file watching
///
/// File Watching for Cache Invalidation
#[async_trait]
pub trait FileWatcher: Send + Sync {
    /// Start watching a path for changes
    async fn watch(&self, path: &PathBuf) -> DomainResult<()>;

    /// Stop watching a path
    async fn unwatch(&self, path: &PathBuf) -> DomainResult<()>;

    /// Check if any watched files have changed since last check
    async fn has_changes(&self) -> DomainResult<bool>;

    /// Get list of changed files and clear the change list
    async fn get_changes(&self) -> DomainResult<Vec<PathBuf>>;
}
