//! Cache statistics and performance metrics

use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

/// Detailed cache statistics with performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedCacheStats {
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

impl DetailedCacheStats {
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

/// Tracks cache operation timings
#[derive(Debug, Clone)]
pub struct CacheOperationTimer {
    /// Start time of the operation
    start_time: SystemTime,
}

impl CacheOperationTimer {
    /// Create a new timer
    pub fn start() -> Self {
        Self {
            start_time: SystemTime::now(),
        }
    }

    /// Get elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> f64 {
        self.start_time
            .elapsed()
            .unwrap_or(Duration::from_secs(0))
            .as_secs_f64()
            * 1000.0
    }
}

/// Thread-safe cache statistics tracker
#[derive(Debug, Clone)]
pub struct CacheStatsTracker {
    stats: Arc<RwLock<DetailedCacheStats>>,
}

impl CacheStatsTracker {
    /// Create a new statistics tracker
    pub fn new() -> Self {
        Self {
            stats: Arc::new(RwLock::new(DetailedCacheStats {
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
            })),
        }
    }

    /// Record a cache hit with timing
    pub fn record_hit(&self, retrieval_time_ms: f64) {
        if let Ok(mut stats) = self.stats.write() {
            stats.hits += 1;
            stats.total_operation_time_ms += retrieval_time_ms;

            // Update average retrieval time
            let total_retrievals = stats.hits + stats.misses;
            if total_retrievals > 0 {
                stats.avg_retrieval_time_ms =
                    stats.total_operation_time_ms / total_retrievals as f64;
            }

            stats.last_operation_time = Some(SystemTime::now());
        }
    }

    /// Record a cache miss
    pub fn record_miss(&self) {
        if let Ok(mut stats) = self.stats.write() {
            stats.misses += 1;
            stats.last_operation_time = Some(SystemTime::now());
        }
    }

    /// Record a cache store operation with timing
    pub fn record_store(&self, store_time_ms: f64, size_bytes: u64) {
        if let Ok(mut stats) = self.stats.write() {
            stats.total_operation_time_ms += store_time_ms;
            stats.size_bytes = size_bytes;

            // Update average store time
            let total_operations = stats.hits + stats.misses;
            if total_operations > 0 {
                stats.avg_store_time_ms = store_time_ms / total_operations as f64;
            }

            stats.last_operation_time = Some(SystemTime::now());
        }
    }

    /// Record a cache invalidation
    pub fn record_invalidation(&self) {
        if let Ok(mut stats) = self.stats.write() {
            stats.invalidations += 1;
            stats.last_operation_time = Some(SystemTime::now());
        }
    }

    /// Update entry count
    pub fn set_entry_count(&self, count: usize) {
        if let Ok(mut stats) = self.stats.write() {
            stats.entry_count = count;
        }
    }

    /// Get current statistics
    pub fn get_stats(&self) -> Option<DetailedCacheStats> {
        self.stats.read().ok().map(|s| s.clone())
    }

    /// Reset all statistics
    pub fn reset(&self) {
        if let Ok(mut stats) = self.stats.write() {
            stats.hits = 0;
            stats.misses = 0;
            stats.invalidations = 0;
            stats.size_bytes = 0;
            stats.entry_count = 0;
            stats.avg_retrieval_time_ms = 0.0;
            stats.avg_store_time_ms = 0.0;
            stats.total_operation_time_ms = 0.0;
            stats.last_operation_time = None;
            stats.created_at = SystemTime::now();
        }
    }

    /// Get a formatted summary of cache statistics
    pub fn summary(&self) -> String {
        if let Some(stats) = self.get_stats() {
            format!(
                "Cache Statistics:\n  Hits: {}\n  Misses: {}\n  Hit Rate: {:.2}%\n  Invalidations: {}\n  Entries: {}\n  Size: {} bytes\n  Avg Retrieval: {:.2}ms\n  Avg Store: {:.2}ms\n  Efficiency Score: {:.2}",
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
        } else {
            "Cache Statistics: (unavailable)".to_string()
        }
    }
}

impl Default for CacheStatsTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detailed_cache_stats_hit_rate() {
        let stats = DetailedCacheStats {
            hits: 75,
            misses: 25,
            invalidations: 0,
            size_bytes: 1024,
            entry_count: 10,
            avg_retrieval_time_ms: 1.5,
            avg_store_time_ms: 2.0,
            total_operation_time_ms: 100.0,
            last_operation_time: None,
            created_at: SystemTime::now(),
        };

        assert_eq!(stats.hit_rate(), 75.0);
        assert_eq!(stats.miss_rate(), 25.0);
    }

    #[test]
    fn test_detailed_cache_stats_invalidation_rate() {
        let stats = DetailedCacheStats {
            hits: 80,
            misses: 20,
            invalidations: 10,
            size_bytes: 1024,
            entry_count: 10,
            avg_retrieval_time_ms: 1.5,
            avg_store_time_ms: 2.0,
            total_operation_time_ms: 100.0,
            last_operation_time: None,
            created_at: SystemTime::now(),
        };

        assert_eq!(stats.invalidation_rate(), 10.0);
    }

    #[test]
    fn test_detailed_cache_stats_efficiency_score() {
        let stats = DetailedCacheStats {
            hits: 90,
            misses: 10,
            invalidations: 5,
            size_bytes: 1024,
            entry_count: 10,
            avg_retrieval_time_ms: 1.5,
            avg_store_time_ms: 2.0,
            total_operation_time_ms: 100.0,
            last_operation_time: None,
            created_at: SystemTime::now(),
        };

        let efficiency = stats.efficiency_score();
        // hit_rate = 90%, invalidation_rate = 5%
        // efficiency = 90 - (5 * 0.5) = 87.5
        assert!((efficiency - 87.5).abs() < 0.01);
    }

    #[test]
    fn test_cache_operation_timer() {
        let timer = CacheOperationTimer::start();
        std::thread::sleep(Duration::from_millis(10));
        let elapsed = timer.elapsed_ms();
        assert!(elapsed >= 10.0);
    }

    #[test]
    fn test_cache_stats_tracker_record_hit() {
        let tracker = CacheStatsTracker::new();
        tracker.record_hit(1.5);

        let stats = tracker.get_stats().unwrap();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 0);
    }

    #[test]
    fn test_cache_stats_tracker_record_miss() {
        let tracker = CacheStatsTracker::new();
        tracker.record_miss();

        let stats = tracker.get_stats().unwrap();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_cache_stats_tracker_record_store() {
        let tracker = CacheStatsTracker::new();
        tracker.record_store(2.0, 1024);

        let stats = tracker.get_stats().unwrap();
        assert_eq!(stats.size_bytes, 1024);
    }

    #[test]
    fn test_cache_stats_tracker_record_invalidation() {
        let tracker = CacheStatsTracker::new();
        tracker.record_invalidation();

        let stats = tracker.get_stats().unwrap();
        assert_eq!(stats.invalidations, 1);
    }

    #[test]
    fn test_cache_stats_tracker_reset() {
        let tracker = CacheStatsTracker::new();
        tracker.record_hit(1.5);
        tracker.record_miss();
        tracker.record_invalidation();

        let stats_before = tracker.get_stats().unwrap();
        assert!(stats_before.hits > 0 || stats_before.misses > 0);

        tracker.reset();

        let stats_after = tracker.get_stats().unwrap();
        assert_eq!(stats_after.hits, 0);
        assert_eq!(stats_after.misses, 0);
        assert_eq!(stats_after.invalidations, 0);
    }

    #[test]
    fn test_cache_stats_tracker_summary() {
        let tracker = CacheStatsTracker::new();
        tracker.record_hit(1.5);
        tracker.record_miss();

        let summary = tracker.summary();
        assert!(summary.contains("Cache Statistics"));
        assert!(summary.contains("Hits: 1"));
        assert!(summary.contains("Misses: 1"));
    }

    #[test]
    fn test_cache_stats_tracker_set_entry_count() {
        let tracker = CacheStatsTracker::new();
        tracker.set_entry_count(42);

        let stats = tracker.get_stats().unwrap();
        assert_eq!(stats.entry_count, 42);
    }
}
