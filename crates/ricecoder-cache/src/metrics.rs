//! Cache performance monitoring and metrics

use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, SystemTime},
};

use serde::{Deserialize, Serialize};

/// Cache performance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// Total number of cache hits
    pub hits: u64,
    /// Total number of cache misses
    pub misses: u64,
    /// Total number of cache invalidations
    pub invalidations: u64,
    /// Current cache size in bytes
    pub size_bytes: u64,
    /// Number of entries in cache
    pub entry_count: usize,
    /// Average time to retrieve from cache (milliseconds)
    pub avg_retrieval_time_ms: f64,
    /// Average time to store in cache (milliseconds)
    pub avg_store_time_ms: f64,
    /// Total time spent on cache operations (milliseconds)
    pub total_operation_time_ms: f64,
    /// Timestamp of last cache operation
    pub last_operation_time: Option<SystemTime>,
    /// Timestamp of cache creation
    pub created_at: SystemTime,
}

impl CacheStats {
    /// Create new cache statistics
    pub fn new() -> Self {
        Self {
            hits: 0,
            misses: 0,
            invalidations: 0,
            size_bytes: 0,
            entry_count: 0,
            avg_retrieval_time_ms: 0.0,
            avg_store_time_ms: 0.0,
            total_operation_time_ms: 0.0,
            last_operation_time: None,
            created_at: SystemTime::now(),
        }
    }

    /// Calculate hit rate as a percentage (0.0 to 100.0)
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        }
    }

    /// Calculate miss rate as a percentage (0.0 to 100.0)
    pub fn miss_rate(&self) -> f64 {
        100.0 - self.hit_rate()
    }

    /// Calculate invalidation rate (invalidations per total operations)
    pub fn invalidation_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.invalidations as f64 / total as f64) * 100.0
        }
    }

    /// Get cache efficiency score (0.0 to 100.0)
    /// Higher is better: considers hit rate and invalidation rate
    pub fn efficiency_score(&self) -> f64 {
        let hit_rate = self.hit_rate();
        let invalidation_rate = self.invalidation_rate();
        // Efficiency = hit_rate - (invalidation_rate * 0.5)
        // This penalizes frequent invalidations but not as heavily as misses
        (hit_rate - (invalidation_rate * 0.5)).max(0.0)
    }

    /// Get uptime since cache creation
    pub fn uptime(&self) -> Duration {
        SystemTime::now()
            .duration_since(self.created_at)
            .unwrap_or(Duration::from_secs(0))
    }
}

impl Default for CacheStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe cache metrics tracker
#[derive(Debug, Clone)]
pub struct CacheMetrics {
    hits: Arc<AtomicU64>,
    misses: Arc<AtomicU64>,
    invalidations: Arc<AtomicU64>,
    size_bytes: Arc<AtomicU64>,
    entry_count: Arc<AtomicU64>,
    total_retrieval_time_ms: Arc<AtomicU64>,
    total_store_time_ms: Arc<AtomicU64>,
    total_operations: Arc<AtomicU64>,
    created_at: SystemTime,
}

impl CacheMetrics {
    /// Create new cache metrics
    pub fn new() -> Self {
        Self {
            hits: Arc::new(AtomicU64::new(0)),
            misses: Arc::new(AtomicU64::new(0)),
            invalidations: Arc::new(AtomicU64::new(0)),
            size_bytes: Arc::new(AtomicU64::new(0)),
            entry_count: Arc::new(AtomicU64::new(0)),
            total_retrieval_time_ms: Arc::new(AtomicU64::new(0)),
            total_store_time_ms: Arc::new(AtomicU64::new(0)),
            total_operations: Arc::new(AtomicU64::new(0)),
            created_at: SystemTime::now(),
        }
    }

    /// Record a cache hit with timing
    pub fn record_hit(&self, retrieval_time_ms: f64) {
        self.hits.fetch_add(1, Ordering::Relaxed);
        self.total_retrieval_time_ms
            .fetch_add(retrieval_time_ms as u64, Ordering::Relaxed);
        self.total_operations.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a cache miss
    pub fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
        self.total_operations.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a cache store operation with timing
    pub fn record_store(&self, store_time_ms: f64, size_bytes: u64) {
        self.total_store_time_ms
            .fetch_add(store_time_ms as u64, Ordering::Relaxed);
        self.size_bytes.store(size_bytes, Ordering::Relaxed);
    }

    /// Record a cache invalidation
    pub fn record_invalidation(&self) {
        self.invalidations.fetch_add(1, Ordering::Relaxed);
    }

    /// Update entry count
    pub fn set_entry_count(&self, count: usize) {
        self.entry_count.store(count as u64, Ordering::Relaxed);
    }

    /// Get current statistics snapshot
    pub fn snapshot(&self) -> CacheStats {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let invalidations = self.invalidations.load(Ordering::Relaxed);
        let size_bytes = self.size_bytes.load(Ordering::Relaxed);
        let entry_count = self.entry_count.load(Ordering::Relaxed) as usize;
        let total_retrieval_time_ms = self.total_retrieval_time_ms.load(Ordering::Relaxed);
        let total_store_time_ms = self.total_store_time_ms.load(Ordering::Relaxed);
        let total_operations = self.total_operations.load(Ordering::Relaxed);

        let avg_retrieval_time_ms = if total_operations > 0 {
            total_retrieval_time_ms as f64 / total_operations as f64
        } else {
            0.0
        };

        let avg_store_time_ms = if total_operations > 0 {
            total_store_time_ms as f64 / total_operations as f64
        } else {
            0.0
        };

        let total_operation_time_ms = (total_retrieval_time_ms + total_store_time_ms) as f64;

        CacheStats {
            hits,
            misses,
            invalidations,
            size_bytes,
            entry_count,
            avg_retrieval_time_ms,
            avg_store_time_ms,
            total_operation_time_ms,
            last_operation_time: Some(SystemTime::now()),
            created_at: self.created_at,
        }
    }

    /// Reset all metrics
    pub fn reset(&self) {
        self.hits.store(0, Ordering::Relaxed);
        self.misses.store(0, Ordering::Relaxed);
        self.invalidations.store(0, Ordering::Relaxed);
        self.size_bytes.store(0, Ordering::Relaxed);
        self.entry_count.store(0, Ordering::Relaxed);
        self.total_retrieval_time_ms.store(0, Ordering::Relaxed);
        self.total_store_time_ms.store(0, Ordering::Relaxed);
        self.total_operations.store(0, Ordering::Relaxed);
    }

    /// Get a formatted summary of cache metrics
    pub fn summary(&self) -> String {
        let stats = self.snapshot();
        format!(
            "Cache Metrics:\n  Hits: {}\n  Misses: {}\n  Hit Rate: {:.2}%\n  Invalidations: {}\n  Entries: {}\n  Size: {} bytes\n  Avg Retrieval: {:.2}ms\n  Avg Store: {:.2}ms\n  Efficiency Score: {:.2}",
            stats.hits,
            stats.misses,
            stats.hit_rate(),
            stats.invalidations,
            stats.entry_count,
            stats.size_bytes,
            stats.avg_retrieval_time_ms,
            stats.avg_store_time_ms,
            stats.efficiency_score()
        )
    }
}

impl Default for CacheMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Operation timing helper
#[derive(Debug)]
pub struct OperationTimer {
    start_time: std::time::Instant,
}

impl OperationTimer {
    /// Start timing an operation
    pub fn start() -> Self {
        Self {
            start_time: std::time::Instant::now(),
        }
    }

    /// Get elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64() * 1000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_stats_hit_rate() {
        let mut stats = CacheStats::new();
        stats.hits = 75;
        stats.misses = 25;

        assert_eq!(stats.hit_rate(), 75.0);
        assert_eq!(stats.miss_rate(), 25.0);
    }

    #[test]
    fn test_cache_stats_efficiency_score() {
        let mut stats = CacheStats::new();
        stats.hits = 90;
        stats.misses = 10;
        stats.invalidations = 5;

        let efficiency = stats.efficiency_score();
        // hit_rate = 90%, invalidation_rate = 5%
        // efficiency = 90 - (5 * 0.5) = 87.5
        assert!((efficiency - 87.5).abs() < 0.01);
    }

    #[test]
    fn test_cache_metrics_record_hit() {
        let metrics = CacheMetrics::new();
        metrics.record_hit(1.5);

        let stats = metrics.snapshot();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 0);
    }

    #[test]
    fn test_cache_metrics_record_miss() {
        let metrics = CacheMetrics::new();
        metrics.record_miss();

        let stats = metrics.snapshot();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_cache_metrics_record_store() {
        let metrics = CacheMetrics::new();
        metrics.record_store(2.0, 1024);

        let stats = metrics.snapshot();
        assert_eq!(stats.size_bytes, 1024);
    }

    #[test]
    fn test_operation_timer() {
        let timer = OperationTimer::start();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let elapsed = timer.elapsed_ms();
        assert!(elapsed >= 10.0);
    }
}
