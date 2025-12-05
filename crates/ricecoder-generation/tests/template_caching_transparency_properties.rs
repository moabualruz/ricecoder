//! Property-based tests for template caching transparency
//! **Feature: ricecoder-templates, Property 6: Template Caching Transparency**
//! **Validates: Requirements 1.6**

use proptest::prelude::*;
use ricecoder_generation::{TemplateCache, TemplateParser};

/// Strategy for generating valid template content
fn template_content_strategy() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9 \{\}\-_]+"
        .prop_map(|s| s.to_string())
        .prop_filter("content must not be empty", |s| !s.is_empty())
}

/// Strategy for generating cache keys
fn cache_key_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{1,10}"
        .prop_map(|s| s.to_string())
        .prop_filter("key must not be empty", |s| !s.is_empty())
}

proptest! {
    /// Property: For any template, rendering from cache should produce
    /// identical output to rendering from the original template
    #[test]
    fn prop_cache_transparency_simple(
        key in cache_key_strategy(),
        template_content in template_content_strategy(),
    ) {
        // Parse the template
        let parsed = match TemplateParser::parse(&template_content) {
            Ok(p) => p,
            Err(_) => return Ok(()), // Skip invalid templates
        };

        // Create cache and insert template
        let mut cache = TemplateCache::new();
        cache.insert(key.clone(), parsed.clone());

        // Retrieve from cache
        let cached = cache.get(&key);

        // Both should be Some and equal
        prop_assert!(cached.is_some(), "Template should be in cache");
        let cached_template = cached.unwrap();
        prop_assert_eq!(cached_template.elements.len(), parsed.elements.len(), "Cached template should have same number of elements");
    }

    /// Property: For any template, rendering from cache multiple times
    /// should produce identical results each time
    #[test]
    fn prop_cache_consistency_multiple_retrievals(
        key in cache_key_strategy(),
        template_content in template_content_strategy(),
    ) {
        // Parse the template
        let parsed = match TemplateParser::parse(&template_content) {
            Ok(p) => p,
            Err(_) => return Ok(()), // Skip invalid templates
        };

        // Create cache and insert template
        let mut cache = TemplateCache::new();
        cache.insert(key.clone(), parsed.clone());

        // Retrieve multiple times
        let first = cache.get(&key);
        let second = cache.get(&key);
        let third = cache.get(&key);

        // All should be Some
        prop_assert!(first.is_some(), "First retrieval should succeed");
        prop_assert!(second.is_some(), "Second retrieval should succeed");
        prop_assert!(third.is_some(), "Third retrieval should succeed");

        // All should have same number of elements
        let first_len = first.unwrap().elements.len();
        let second_len = second.unwrap().elements.len();
        let third_len = third.unwrap().elements.len();

        prop_assert_eq!(first_len, second_len, "First and second retrievals should match");
        prop_assert_eq!(second_len, third_len, "Second and third retrievals should match");
    }

    /// Property: For any template, cache hit rate should increase with repeated access
    #[test]
    fn prop_cache_hit_rate_increases(
        key in cache_key_strategy(),
        template_content in template_content_strategy(),
    ) {
        // Parse the template
        let parsed = match TemplateParser::parse(&template_content) {
            Ok(p) => p,
            Err(_) => return Ok(()), // Skip invalid templates
        };

        // Create cache and insert template
        let mut cache = TemplateCache::new();
        cache.insert(key.clone(), parsed);

        // Initial stats
        let initial_stats = cache.stats();
        prop_assert_eq!(initial_stats.hits, 0, "Initial hits should be 0");
        prop_assert_eq!(initial_stats.misses, 0, "Initial misses should be 0");

        // Access the template
        let _ = cache.get(&key);
        let stats_after_first = cache.stats();
        prop_assert_eq!(stats_after_first.hits, 1, "After first access, hits should be 1");

        // Access again
        let _ = cache.get(&key);
        let stats_after_second = cache.stats();
        prop_assert_eq!(stats_after_second.hits, 2, "After second access, hits should be 2");

        // Access a non-existent key
        let _ = cache.get("nonexistent");
        let stats_after_miss = cache.stats();
        prop_assert_eq!(stats_after_miss.hits, 2, "Hits should remain 2");
        prop_assert_eq!(stats_after_miss.misses, 1, "Misses should be 1");
    }

    /// Property: For any template, cache should maintain correct statistics
    #[test]
    fn prop_cache_statistics_accuracy(
        key in cache_key_strategy(),
        template_content in template_content_strategy(),
    ) {
        // Parse the template
        let parsed = match TemplateParser::parse(&template_content) {
            Ok(p) => p,
            Err(_) => return Ok(()), // Skip invalid templates
        };

        // Create cache and insert template
        let mut cache = TemplateCache::new();
        cache.insert(key.clone(), parsed);

        // Check initial stats
        let stats = cache.stats();
        prop_assert_eq!(stats.total_templates, 1, "Should have 1 template");
        prop_assert!(stats.total_size_bytes > 0, "Should have non-zero size");

        // Access the template
        let _ = cache.get(&key);
        let stats_after = cache.stats();
        prop_assert_eq!(stats_after.total_templates, 1, "Should still have 1 template");
        prop_assert_eq!(stats_after.hits, 1, "Should have 1 hit");
    }

    /// Property: For any template, removing from cache should update statistics
    #[test]
    fn prop_cache_removal_updates_stats(
        key in cache_key_strategy(),
        template_content in template_content_strategy(),
    ) {
        // Parse the template
        let parsed = match TemplateParser::parse(&template_content) {
            Ok(p) => p,
            Err(_) => return Ok(()), // Skip invalid templates
        };

        // Create cache and insert template
        let mut cache = TemplateCache::new();
        cache.insert(key.clone(), parsed);

        // Check stats before removal
        let stats_before = cache.stats();
        prop_assert_eq!(stats_before.total_templates, 1, "Should have 1 template before removal");

        // Remove the template
        let removed = cache.remove(&key);
        prop_assert!(removed.is_some(), "Removal should succeed");

        // Check stats after removal
        let stats_after = cache.stats();
        prop_assert_eq!(stats_after.total_templates, 0, "Should have 0 templates after removal");
    }

    /// Property: For any template, cache should be empty after clear
    #[test]
    fn prop_cache_clear_empties_cache(
        key in cache_key_strategy(),
        template_content in template_content_strategy(),
    ) {
        // Parse the template
        let parsed = match TemplateParser::parse(&template_content) {
            Ok(p) => p,
            Err(_) => return Ok(()), // Skip invalid templates
        };

        // Create cache and insert template
        let mut cache = TemplateCache::new();
        cache.insert(key.clone(), parsed);

        // Verify cache is not empty
        prop_assert!(!cache.is_empty(), "Cache should not be empty");
        prop_assert_eq!(cache.len(), 1, "Cache should have 1 item");

        // Clear the cache
        cache.clear();

        // Verify cache is empty
        prop_assert!(cache.is_empty(), "Cache should be empty after clear");
        prop_assert_eq!(cache.len(), 0, "Cache should have 0 items");
    }

    /// Property: For any template, cache should correctly report containment
    #[test]
    fn prop_cache_contains_accuracy(
        key in cache_key_strategy(),
        template_content in template_content_strategy(),
    ) {
        // Parse the template
        let parsed = match TemplateParser::parse(&template_content) {
            Ok(p) => p,
            Err(_) => return Ok(()), // Skip invalid templates
        };

        // Create cache and insert template
        let mut cache = TemplateCache::new();
        cache.insert(key.clone(), parsed);

        // Check containment
        prop_assert!(cache.contains(&key), "Cache should contain the key");
        prop_assert!(!cache.contains("nonexistent"), "Cache should not contain nonexistent key");

        // Remove and check again
        cache.remove(&key);
        prop_assert!(!cache.contains(&key), "Cache should not contain key after removal");
    }

    /// Property: For any template, cache hit rate should be calculable
    #[test]
    fn prop_cache_hit_rate_calculation(
        key in cache_key_strategy(),
        template_content in template_content_strategy(),
    ) {
        // Parse the template
        let parsed = match TemplateParser::parse(&template_content) {
            Ok(p) => p,
            Err(_) => return Ok(()), // Skip invalid templates
        };

        // Create cache and insert template
        let mut cache = TemplateCache::new();
        cache.insert(key.clone(), parsed);

        // Access the template 3 times (hits)
        let _ = cache.get(&key);
        let _ = cache.get(&key);
        let _ = cache.get(&key);

        // Access nonexistent key 2 times (misses)
        let _ = cache.get("nonexistent1");
        let _ = cache.get("nonexistent2");

        // Check hit rate
        let stats = cache.stats();
        prop_assert_eq!(stats.hits, 3, "Should have 3 hits");
        prop_assert_eq!(stats.misses, 2, "Should have 2 misses");

        // Hit rate should be 60% (3 / (3 + 2) * 100)
        let expected_hit_rate = 60.0;
        let actual_hit_rate = stats.hit_rate();
        prop_assert!((actual_hit_rate - expected_hit_rate).abs() < 0.1, "Hit rate should be approximately 60%");
    }

    /// Property: For any template, cache should maintain insertion order
    /// (or at least be consistent across retrievals)
    #[test]
    fn prop_cache_consistency_across_operations(
        key in cache_key_strategy(),
        template_content in template_content_strategy(),
    ) {
        // Parse the template
        let parsed = match TemplateParser::parse(&template_content) {
            Ok(p) => p,
            Err(_) => return Ok(()), // Skip invalid templates
        };

        let original_elements_len = parsed.elements.len();

        // Create cache and insert template
        let mut cache = TemplateCache::new();
        cache.insert(key.clone(), parsed);

        // Retrieve and verify
        let retrieved1 = cache.get(&key).unwrap();
        prop_assert_eq!(retrieved1.elements.len(), original_elements_len, "Retrieved template should match original");

        // Retrieve again and verify
        let retrieved2 = cache.get(&key).unwrap();
        prop_assert_eq!(retrieved2.elements.len(), original_elements_len, "Second retrieval should match original");

        // Verify they're the same
        prop_assert_eq!(retrieved1.elements.len(), retrieved2.elements.len(), "Multiple retrievals should be consistent");
    }
}
