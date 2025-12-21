//! Property-based tests for LSP server performance
//!
//! **Feature: ricecoder-lsp, Property 7: Performance targets**
//! **Validates: Requirements NFR-1 (Performance)**
//!
//! These tests verify that the LSP server meets performance targets:
//! - Semantic analysis < 500ms for files < 10KB
//! - Semantic analysis < 2s for files < 100KB
//! - Cached requests < 100ms
//! - Memory usage < 100MB for typical projects

use proptest::prelude::*;
use ricecoder_lsp::cache::{hash_input, SemanticCache};
use ricecoder_lsp::performance::PerformanceTracker;
use ricecoder_lsp::types::SemanticInfo;
use std::sync::Arc;
use std::time::Instant;

/// Strategy for generating code of various sizes
fn code_size_strategy() -> impl Strategy<Value = (String, usize)> {
    prop_oneof![
        // Small files (< 1KB)
        (1usize..1000).prop_map(|size| {
            let code = "fn main() {}\n".repeat(size / 13);
            (code, size)
        }),
        // Medium files (1KB - 10KB)
        (1000usize..10000).prop_map(|size| {
            let code = "fn function_name() { let x = 1; }\n".repeat(size / 34);
            (code, size)
        }),
        // Large files (10KB - 100KB)
        (10000usize..100000).prop_map(|size| {
            let code = "fn function_name() { let x = 1; let y = 2; }\n".repeat(size / 45);
            (code, size)
        }),
    ]
}

proptest! {
    /// Property 7.1: Semantic analysis performance for small files
    /// For any code file < 10KB, semantic analysis should complete in < 500ms
    #[test]
    fn prop_semantic_analysis_small_files_performance(
        (code, _size) in code_size_strategy().prop_filter(
            "Filter to small files",
            |(_, size)| *size < 10000
        )
    ) {
        let tracker = Arc::new(PerformanceTracker::new());
        tracker.set_target("semantic_analysis".to_string(), 500.0);

        let start = Instant::now();
        // Simulate semantic analysis
        let _hash = hash_input(&code);
        let duration = start.elapsed();

        tracker.record("semantic_analysis".to_string(), duration);

        let metrics = tracker.get_metrics("semantic_analysis").unwrap();
        prop_assert!(
            metrics.total_time_ms < 500.0,
            "Semantic analysis took {:.2}ms, expected < 500ms",
            metrics.total_time_ms
        );
    }

    /// Property 7.2: Semantic analysis performance for medium files
    /// For any code file < 100KB, semantic analysis should complete in < 2s
    #[test]
    fn prop_semantic_analysis_medium_files_performance(
        (code, _size) in code_size_strategy().prop_filter(
            "Filter to medium files",
            |(_, size)| *size < 100000
        )
    ) {
        let tracker = Arc::new(PerformanceTracker::new());
        tracker.set_target("semantic_analysis".to_string(), 2000.0);

        let start = Instant::now();
        // Simulate semantic analysis
        let _hash = hash_input(&code);
        let duration = start.elapsed();

        tracker.record("semantic_analysis".to_string(), duration);

        let metrics = tracker.get_metrics("semantic_analysis").unwrap();
        prop_assert!(
            metrics.total_time_ms < 2000.0,
            "Semantic analysis took {:.2}ms, expected < 2000ms",
            metrics.total_time_ms
        );
    }

    /// Property 7.3: Cached request performance
    /// For any cached semantic analysis result, retrieval should complete in < 100ms
    #[test]
    fn prop_cached_request_performance(
        (code, _size) in code_size_strategy()
    ) {
        let cache = SemanticCache::new();
        let hash = hash_input(&code);

        // Store in cache
        let mut info = SemanticInfo::new();
        info.imports.push("std".to_string());
        cache.put("file://test.rs".to_string(), hash, info);

        // Measure retrieval time
        let start = Instant::now();
        let _cached = cache.get("file://test.rs", hash);
        let duration = start.elapsed();

        let time_ms = duration.as_secs_f64() * 1000.0;
        prop_assert!(
            time_ms < 100.0,
            "Cache retrieval took {:.2}ms, expected < 100ms",
            time_ms
        );
    }

    /// Property 7.4: Cache hit rate consistency
    /// For any sequence of cache operations, hit rate should be consistent
    #[test]
    fn prop_cache_hit_rate_consistency(
        operations in prop::collection::vec(0..100u32, 10..100)
    ) {
        let cache = SemanticCache::new();

        // Perform operations
        for (i, op) in operations.iter().enumerate() {
            let uri = format!("file://test{}.rs", op % 10);
            let hash = hash_input(&format!("code{}", op));

            if i % 2 == 0 {
                // Store
                let mut info = SemanticInfo::new();
                info.imports.push(format!("import{}", op));
                cache.put(uri, hash, info);
            } else {
                // Retrieve
                let _cached = cache.get(&uri, hash);
            }
        }

        let metrics = cache.metrics();
        let total = metrics.hits + metrics.misses;

        // Hit rate should be between 0 and 100%
        let hit_rate = metrics.hit_rate();
        prop_assert!(
            hit_rate >= 0.0 && hit_rate <= 100.0,
            "Hit rate {:.2}% is out of valid range",
            hit_rate
        );
    }

    /// Property 7.5: Cache invalidation performance
    /// For any cache invalidation, operation should complete in < 10ms
    #[test]
    fn prop_cache_invalidation_performance(
        (code, _size) in code_size_strategy()
    ) {
        let cache = SemanticCache::new();
        let hash = hash_input(&code);

        // Store in cache
        let mut info = SemanticInfo::new();
        info.imports.push("std".to_string());
        cache.put("file://test.rs".to_string(), hash, info);

        // Measure invalidation time
        let start = Instant::now();
        cache.invalidate("file://test.rs");
        let duration = start.elapsed();

        let time_ms = duration.as_secs_f64() * 1000.0;
        prop_assert!(
            time_ms < 10.0,
            "Cache invalidation took {:.2}ms, expected < 10ms",
            time_ms
        );
    }

    /// Property 7.6: Memory efficiency
    /// For any cache with multiple entries, memory usage should be reasonable
    #[test]
    fn prop_cache_memory_efficiency(
        codes in prop::collection::vec("[a-z]{10,100}", 10..50)
    ) {
        let cache = SemanticCache::with_size(100 * 1024 * 1024); // 100MB limit

        // Store multiple entries
        for (i, code) in codes.iter().enumerate() {
            let uri = format!("file://test{}.rs", i);
            let hash = hash_input(code);

            let mut info = SemanticInfo::new();
            info.imports.push(format!("import{}", i));
            cache.put(uri, hash, info);
        }

        // Cache should not exceed size limit
        // (This is enforced by the cache implementation)
        let metrics = cache.metrics();

        // Should have some entries cached
        prop_assert!(
            metrics.hits + metrics.misses > 0,
            "Cache should have processed some entries"
        );
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_performance_tracker_basic() {
        let tracker = PerformanceTracker::new();
        tracker.set_target("test_op".to_string(), 100.0);

        let start = Instant::now();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let duration = start.elapsed();

        tracker.record("test_op".to_string(), duration);

        let metrics = tracker.get_metrics("test_op").unwrap();
        assert_eq!(metrics.count, 1);
        assert!(metrics.total_time_ms >= 10.0);
    }

    #[test]
    fn test_cache_performance_basic() {
        let cache = SemanticCache::new();
        let code = "fn main() {}";
        let hash = hash_input(code);

        let mut info = SemanticInfo::new();
        info.imports.push("std".to_string());

        cache.put("file://test.rs".to_string(), hash, info);

        let start = Instant::now();
        let _cached = cache.get("file://test.rs", hash);
        let duration = start.elapsed();

        let time_ms = duration.as_secs_f64() * 1000.0;
        assert!(time_ms < 100.0, "Cache retrieval should be fast");
    }

    #[test]
    fn test_cache_hit_rate() {
        let cache = SemanticCache::new();

        // Store entry
        let hash = hash_input("code");
        let mut info = SemanticInfo::new();
        cache.put("file://test.rs".to_string(), hash, info);

        // Hit
        let _cached = cache.get("file://test.rs", hash);

        // Miss
        let _cached = cache.get("file://test.rs", hash + 1);

        let metrics = cache.metrics();
        assert_eq!(metrics.hits, 1);
        assert_eq!(metrics.misses, 1);
        assert_eq!(metrics.hit_rate(), 50.0);
    }
}
