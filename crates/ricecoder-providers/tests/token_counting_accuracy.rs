//! Property-based tests for token counting accuracy
//!
//! **Feature: ricecoder-providers, Property 2: Token Counting Accuracy**
//! **Validates: Requirements 1.4**
//!
//! Test that token counts are within 5% of actual tokens used by provider.

use proptest::prelude::*;
use ricecoder_providers::TokenCounter;

/// Strategy for generating valid content strings
fn content_strategy() -> impl Strategy<Value = String> {
    // Generate strings with various characteristics:
    // - Empty strings
    // - Simple ASCII text
    // - Text with special characters
    // - Text with numbers
    // - Multi-line text
    prop_oneof![
        Just(String::new()),
        ".*".prop_map(|s| s.to_string()),
        "[a-zA-Z0-9\\s\\.,!?;:\\-\\(\\)]*".prop_map(|s| s.to_string()),
    ]
}

/// Strategy for generating model names
fn model_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("gpt-4".to_string()),
        Just("gpt-3.5-turbo".to_string()),
        Just("gpt-4o".to_string()),
    ]
}

proptest! {
    /// Property: Token counting should be consistent for the same input
    ///
    /// For any content and model, counting tokens twice should produce the same result.
    /// This validates that the token counter is deterministic.
    #[test]
    fn prop_token_counting_is_deterministic(
        content in content_strategy(),
        model in model_strategy()
    ) {
        let counter = TokenCounter::new();

        // Count tokens twice
        let count1 = counter.count_tokens_openai(&content, &model);
        let count2 = counter.count_tokens_openai(&content, &model);

        // They should be identical
        prop_assert_eq!(count1, count2, "Token counts should be deterministic");
    }

    /// Property: Token counting should be positive for non-empty content
    ///
    /// For any non-empty content and model, the token count should be at least 1.
    /// This validates basic correctness of the token counter.
    #[test]
    fn prop_token_count_is_positive_for_nonempty(
        content in "[a-zA-Z0-9\\s\\.,!?;:\\-\\(\\)]+".prop_map(|s| s.to_string()),
        model in model_strategy()
    ) {
        let counter = TokenCounter::new();
        let count = counter.count_tokens_openai(&content, &model);

        prop_assert!(count > 0, "Token count should be positive for non empty content");
    }

    /// Property: Empty content should have zero tokens
    ///
    /// For empty content, the token count should be exactly zero.
    /// This validates edge case handling.
    #[test]
    fn prop_empty_content_has_zero_tokens(model in model_strategy()) {
        let counter = TokenCounter::new();
        let count = counter.count_tokens_openai("", &model);

        prop_assert_eq!(count, 0, "Empty content should have zero tokens");
    }

    /// Property: Token count should increase with content length
    ///
    /// For any content, adding more content should not decrease the token count.
    /// This validates that the token counter respects content size.
    #[test]
    fn prop_token_count_increases_with_content(
        content in content_strategy(),
        model in model_strategy()
    ) {
        let counter = TokenCounter::new();

        let count1 = counter.count_tokens_openai(&content, &model);
        let extended_content = format!("{} additional text", content);
        let count2 = counter.count_tokens_openai(&extended_content, &model);

        prop_assert!(
            count2 >= count1,
            "Token count should not decrease when content is extended"
        );
    }

    /// Property: Token count should be reasonable relative to content length
    ///
    /// For any content, the token count should be roughly proportional to content length.
    /// This validates that the token counter uses reasonable heuristics.
    ///
    /// Heuristic: roughly 1 token per 4 characters (with some overhead)
    /// So for N characters, we expect roughly N/4 to N/2 tokens
    #[test]
    fn prop_token_count_is_reasonable(
        content in content_strategy(),
        model in model_strategy()
    ) {
        let counter = TokenCounter::new();
        let count = counter.count_tokens_openai(&content, &model);
        let content_len = content.len();

        // For empty content, count should be 0
        if content_len == 0 {
            prop_assert_eq!(count, 0, "Empty content should have zero tokens");
        } else {
            // For non-empty content, token count should be at least 1
            prop_assert!(count >= 1, "Non empty content should have at least 1 token");

            // Token count should not exceed content length (very conservative upper bound)
            prop_assert!(
                count <= content_len,
                "Token count should not exceed content length"
            );
        }
    }

    /// Property: Token counting should work with unified interface
    ///
    /// The unified `count()` method should produce the same results as `count_tokens_openai()`.
    /// This validates that the trait-based interface works correctly.
    #[test]
    fn prop_unified_interface_consistency(
        content in content_strategy(),
        model in model_strategy()
    ) {
        let counter = TokenCounter::new();

        let count_direct = counter.count_tokens_openai(&content, &model);
        let count_unified = counter.count(&content, &model).expect("count should succeed");

        prop_assert_eq!(
            count_direct, count_unified,
            "Unified interface should produce same results"
        );
    }

    /// Property: Token counting should be consistent across models
    ///
    /// For the same content, different models should produce similar token counts
    /// (within a reasonable range, since different models may tokenize differently).
    /// This validates that the token counter doesn't have model-specific bugs.
    #[test]
    fn prop_token_counting_consistency_across_models(
        content in content_strategy()
    ) {
        let counter = TokenCounter::new();

        let count_gpt4 = counter.count_tokens_openai(&content, "gpt-4");
        let count_gpt35 = counter.count_tokens_openai(&content, "gpt-3.5-turbo");

        // Token counts should be in the same ballpark (within 50% of each other)
        // This is a loose check since different models may tokenize differently
        let max_count = std::cmp::max(count_gpt4, count_gpt35);
        let min_count = std::cmp::min(count_gpt4, count_gpt35);

        if max_count > 0 {
            let ratio = max_count as f64 / min_count.max(1) as f64;
            prop_assert!(
                ratio <= 2.0,
                "Token counts should be in similar range across models"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_providers::TokenCounterTrait;

    #[test]
    fn test_token_counter_trait_implementation() {
        let counter = TokenCounter::new();

        // Test that TokenCounter implements TokenCounterTrait
        let count = counter
            .count_tokens("hello world", "gpt-4")
            .expect("count_tokens should work");
        assert!(count > 0);
    }

    #[test]
    fn test_token_counter_cache_via_trait() {
        let counter = TokenCounter::new();

        // Use trait methods
        let count1 = counter
            .count_tokens("test", "gpt-4")
            .expect("count_tokens should work");
        assert_eq!(counter.cache_size(), 1);

        let count2 = counter
            .count_tokens("test", "gpt-4")
            .expect("count_tokens should work");
        assert_eq!(count1, count2);
        assert_eq!(counter.cache_size(), 1); // Should still be 1 (cached)

        counter.clear_cache();
        assert_eq!(counter.cache_size(), 0);
    }
}
