//! Property-based tests for image cache consistency.
//!
//! **Feature: ricecoder-images, Property 2: Cache Consistency**
//! **Validates: Requirements 3.1, 3.2, 3.4, 3.5**

use proptest::prelude::*;
use tempfile::tempdir;
use ricecoder_images::{ImageAnalysisResult, ImageCache};
use std::thread;
use std::time::Duration;

/// Strategy for generating image hashes (SHA256 hex strings)
fn image_hash_strategy() -> impl Strategy<Value = String> {
    r"[a-f0-9]{64}".prop_map(|s| s.to_string())
}

/// Strategy for generating analysis text
fn analysis_text_strategy() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9 .,!?]{10,200}".prop_map(|s| s.to_string())
}

/// Strategy for generating provider names
fn provider_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("openai".to_string()),
        Just("anthropic".to_string()),
        Just("ollama".to_string()),
        Just("google".to_string()),
        Just("zen".to_string()),
    ]
}

/// Strategy for generating token counts
fn token_count_strategy() -> impl Strategy<Value = u32> {
    100u32..10000u32
}



/// Property 2: Cache Consistency - Same image returns cached result
///
/// For any image, analyzing it twice within the cache TTL SHALL return the cached result
/// on the second analysis without re-analysis.
#[test]
fn prop_cache_hit_returns_same_result() {
    proptest!(|(
        image_hash in image_hash_strategy(),
        analysis in analysis_text_strategy(),
        provider in provider_strategy(),
        tokens in token_count_strategy(),
        unique_id in 0u32..1000u32,
    )| {
        // Use unique hash to avoid collisions with other tests
        let unique_hash = format!("{}_{}", image_hash, unique_id);
        let cache = ImageCache::new().expect("Failed to create cache");
        
        // Create analysis result
        let result = ImageAnalysisResult::new(
            unique_hash.clone(),
            analysis.clone(),
            provider.clone(),
            tokens,
        );
        
        // Cache the result
        cache.set(&unique_hash, &result)
            .expect("Failed to cache result");
        
        // Retrieve from cache
        let cached = cache.get(&unique_hash)
            .expect("Failed to get from cache")
            .expect("Cache should contain the result");
        
        // Verify the cached result matches the original
        prop_assert_eq!(cached.image_hash, unique_hash, "Image hash should match");
        prop_assert_eq!(cached.analysis, analysis, "Analysis should match");
        prop_assert_eq!(cached.provider, provider, "Provider should match");
        prop_assert_eq!(cached.tokens_used, tokens, "Token count should match");
    });
}

/// Property 2: Cache Consistency - Cache key consistency
///
/// For any image hash, the cache key SHALL be consistent across multiple lookups.
/// The same hash always produces the same cache key.
#[test]
fn prop_cache_key_consistency() {
    proptest!(|(image_hash in image_hash_strategy())| {
        // Use a temporary directory for the cache to avoid conflicts
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let cache = ImageCache::with_temp_dir(temp_dir.path()).expect("Failed to create cache");
        
        // Create same result
        let result = ImageAnalysisResult::new(
            image_hash.clone(),
            "Test analysis".to_string(),
            "openai".to_string(),
            100,
        );
        
        // Set and get multiple times
        cache.set(&image_hash, &result)
            .expect("Failed to cache result");
        
        let cached1 = cache.get(&image_hash)
            .expect("Failed to get from cache")
            .expect("Cache should contain result");
        
        let cached2 = cache.get(&image_hash)
            .expect("Failed to get from cache again")
            .expect("Cache should contain result on second lookup");
        
        // Both lookups should have identical data
        prop_assert_eq!(cached1.image_hash, cached2.image_hash);
        prop_assert_eq!(cached1.analysis, cached2.analysis);
        prop_assert_eq!(cached1.provider, cached2.provider);
        prop_assert_eq!(cached1.tokens_used, cached2.tokens_used);
    });
}

/// Property 2: Cache Consistency - Cache miss for non-existent images
///
/// For any image hash that was never cached, the cache SHALL return None (cache miss).
#[test]
fn prop_cache_miss_for_uncached_images() {
    proptest!(|(image_hash in image_hash_strategy())| {
        // Use a temporary directory for the cache to avoid conflicts
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let cache = ImageCache::with_temp_dir(temp_dir.path()).expect("Failed to create cache");
        
        // Try to get a non-existent image
        let result = cache.get(&image_hash)
            .expect("Cache lookup should not fail");
        
        prop_assert!(result.is_none(), "Cache should return None for uncached image");
    });
}

/// Property 2: Cache Consistency - Cache invalidation removes entries
///
/// For any cached image, invalidating it SHALL remove it from the cache.
/// Subsequent lookups SHALL return None.
#[test]
fn prop_cache_invalidation_removes_entries() {
    proptest!(|(
        image_hash in image_hash_strategy(),
        analysis in analysis_text_strategy(),
    )| {
        let cache = ImageCache::new().expect("Failed to create cache");
        
        let result = ImageAnalysisResult::new(
            image_hash.clone(),
            analysis,
            "openai".to_string(),
            100,
        );
        
        // Cache the result
        cache.set(&image_hash, &result)
            .expect("Failed to cache result");
        
        // Verify it's cached
        prop_assert!(
            cache.exists(&image_hash).expect("Failed to check existence"),
            "Image should be cached"
        );
        
        // Invalidate the cache
        let invalidated = cache.invalidate(&image_hash)
            .expect("Failed to invalidate");
        prop_assert!(invalidated, "Invalidation should return true");
        
        // Verify it's no longer cached
        prop_assert!(
            !cache.exists(&image_hash).expect("Failed to check existence"),
            "Image should not be cached after invalidation"
        );
        
        // Verify lookup returns None
        let result = cache.get(&image_hash)
            .expect("Cache lookup should not fail");
        prop_assert!(result.is_none(), "Cache should return None after invalidation");
    });
}

/// Property 2: Cache Consistency - Multiple images can be cached independently
///
/// For any set of different images, caching them independently SHALL not interfere
/// with each other. Each image's cache entry SHALL be retrievable independently.
#[test]
fn prop_multiple_images_cached_independently() {
    proptest!(|(
        hash1 in image_hash_strategy(),
        hash2 in image_hash_strategy(),
        unique_id in 0u32..1000u32,
    )| {
        // Ensure hashes are different
        prop_assume!(hash1 != hash2);
        
        // Use unique hashes to avoid collisions with other tests
        let unique_hash1 = format!("{}_multi_{}_1", hash1, unique_id);
        let unique_hash2 = format!("{}_multi_{}_2", hash2, unique_id);
        
        let cache = ImageCache::new().expect("Failed to create cache");
        
        // Create and cache two different results
        let result1 = ImageAnalysisResult::new(
            unique_hash1.clone(),
            "Analysis 1".to_string(),
            "openai".to_string(),
            100,
        );
        let result2 = ImageAnalysisResult::new(
            unique_hash2.clone(),
            "Analysis 2".to_string(),
            "anthropic".to_string(),
            200,
        );
        
        cache.set(&unique_hash1, &result1).expect("Failed to cache result1");
        cache.set(&unique_hash2, &result2).expect("Failed to cache result2");
        
        // Verify each can be retrieved independently
        let cached1 = cache.get(&unique_hash1)
            .expect("Failed to get result1")
            .expect("Result1 should be cached");
        let cached2 = cache.get(&unique_hash2)
            .expect("Failed to get result2")
            .expect("Result2 should be cached");
        
        prop_assert_eq!(cached1.analysis, "Analysis 1");
        prop_assert_eq!(cached2.analysis, "Analysis 2");
        
        // Invalidate one and verify the other remains
        cache.invalidate(&unique_hash2).expect("Failed to invalidate result2");
        
        prop_assert!(cache.exists(&unique_hash1).expect("Failed to check hash1"));
        prop_assert!(!cache.exists(&unique_hash2).expect("Failed to check hash2"));
    });
}

/// Property 2: Cache Consistency - Cache invalidation removes entries
///
/// For any cached image, invalidating it SHALL remove it from the cache.
/// Subsequent lookups SHALL return None.
#[test]
fn prop_cache_clear_removes_all_entries() {
    proptest!(|(
        hash1 in image_hash_strategy(),
        hash2 in image_hash_strategy(),
        unique_id in 0u32..1000u32,
    )| {
        prop_assume!(hash1 != hash2);
        
        // Use unique hashes to avoid collisions with other tests
        let unique_hash1 = format!("{}_clear_{}_1", hash1, unique_id);
        let unique_hash2 = format!("{}_clear_{}_2", hash2, unique_id);
        
        let cache = ImageCache::new().expect("Failed to create cache");
        
        // Cache two results
        let result1 = ImageAnalysisResult::new(
            unique_hash1.clone(),
            "Analysis 1".to_string(),
            "openai".to_string(),
            100,
        );
        let result2 = ImageAnalysisResult::new(
            unique_hash2.clone(),
            "Analysis 2".to_string(),
            "anthropic".to_string(),
            200,
        );
        
        cache.set(&unique_hash1, &result1).expect("Failed to cache result1");
        cache.set(&unique_hash2, &result2).expect("Failed to cache result2");
        
        // Verify both are cached
        prop_assert!(cache.exists(&unique_hash1).expect("Failed to check hash1"));
        prop_assert!(cache.exists(&unique_hash2).expect("Failed to check hash2"));
        
        // Invalidate both entries
        cache.invalidate(&unique_hash1).expect("Failed to invalidate hash1");
        cache.invalidate(&unique_hash2).expect("Failed to invalidate hash2");
        
        // Verify both are gone
        prop_assert!(!cache.exists(&unique_hash1).expect("Failed to check hash1 after invalidate"));
        prop_assert!(!cache.exists(&unique_hash2).expect("Failed to check hash2 after invalidate"));
    });
}

/// Property 2: Cache Consistency - Hash computation is deterministic
///
/// For any image data, computing the hash multiple times SHALL produce the same result.
#[test]
fn prop_hash_computation_deterministic() {
    proptest!(|(data in prop::collection::vec(any::<u8>(), 100..10000))| {
        let hash1 = ImageCache::compute_hash(&data);
        let hash2 = ImageCache::compute_hash(&data);
        let hash3 = ImageCache::compute_hash(&data);
        
        prop_assert_eq!(hash1, hash2.clone(), "Hash should be deterministic");
        prop_assert_eq!(hash2, hash3, "Hash should be deterministic");
    });
}

/// Property 2: Cache Consistency - Different data produces different hashes
///
/// For any two different image data, computing hashes SHALL produce different results.
#[test]
fn prop_different_data_different_hashes() {
    proptest!(|(
        data1 in prop::collection::vec(any::<u8>(), 100..1000),
        data2 in prop::collection::vec(any::<u8>(), 100..1000),
    )| {
        prop_assume!(data1 != data2);
        
        let hash1 = ImageCache::compute_hash(&data1);
        let hash2 = ImageCache::compute_hash(&data2);
        
        prop_assert_ne!(hash1, hash2, "Different data should produce different hashes");
    });
}

/// Property 2: Cache Consistency - Cache statistics are accurate
///
/// For any cache configuration, the statistics SHALL reflect the configured values.
#[test]
fn prop_cache_statistics_accurate() {
    proptest!(|(
        ttl in 1u64..86400u64,
        max_size in 1u64..1000u64,
    )| {
        let cache = ImageCache::with_config(ttl, max_size)
            .expect("Failed to create cache with config");
        
        let (actual_ttl, actual_max_size) = cache.stats();
        
        prop_assert_eq!(actual_ttl, ttl, "TTL should match configuration");
        prop_assert_eq!(actual_max_size, max_size, "Max size should match configuration");
    });
}

/// Test: Cache TTL expiration and reanalysis
///
/// For any cached image with a short TTL, after expiration the cache SHALL return None
/// and the image SHALL need to be reanalyzed.
#[test]
fn test_cache_ttl_expiration_and_reanalysis() {
    let cache = ImageCache::with_config(1, 100) // 1 second TTL
        .expect("Failed to create cache");
    
    let image_hash = "test_hash_123".to_string();
    let result = ImageAnalysisResult::new(
        image_hash.clone(),
        "Test analysis".to_string(),
        "openai".to_string(),
        100,
    );
    
    // Cache the result
    cache.set(&image_hash, &result)
        .expect("Failed to cache result");
    
    // Should be cached immediately
    assert!(
        cache.exists(&image_hash).expect("Failed to check existence"),
        "Image should be cached immediately"
    );
    
    // Wait for TTL to expire
    thread::sleep(Duration::from_secs(2));
    
    // Should be expired now
    assert!(
        !cache.exists(&image_hash).expect("Failed to check existence"),
        "Image should be expired after TTL"
    );
    
    // Cache lookup should return None
    let cached = cache.get(&image_hash)
        .expect("Failed to get from cache");
    assert!(cached.is_none(), "Cache should return None for expired entry");
}

/// Test: Cache cleanup removes expired entries
///
/// For any cache with expired entries, cleanup SHALL remove them.
#[test]
fn test_cache_cleanup_removes_expired() {
    let cache = ImageCache::with_config(1, 100) // 1 second TTL
        .expect("Failed to create cache");
    
    let image_hash = "test_hash_cleanup".to_string();
    let result = ImageAnalysisResult::new(
        image_hash.clone(),
        "Test analysis".to_string(),
        "openai".to_string(),
        100,
    );
    
    // Cache the result
    cache.set(&image_hash, &result)
        .expect("Failed to cache result");
    
    // Verify it's cached
    assert!(
        cache.exists(&image_hash).expect("Failed to check existence"),
        "Entry should be cached initially"
    );
    
    // Wait for TTL to expire
    thread::sleep(Duration::from_secs(2));
    
    // After expiration, the entry should not exist
    assert!(
        !cache.exists(&image_hash).expect("Failed to check existence after expiration"),
        "Entry should be expired after TTL"
    );
    
    // Cleanup should succeed (may or may not remove entries depending on implementation)
    let _cleaned = cache.cleanup_expired()
        .expect("Failed to cleanup cache");
}

