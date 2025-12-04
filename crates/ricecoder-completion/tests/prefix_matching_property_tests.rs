/// Property-based tests for prefix matching in completion ranking
/// **Feature: ricecoder-completion, Property 5: Prefix matching**
/// **Validates: Requirements Completion-5.1, Completion-5.2**

use proptest::prelude::*;
use ricecoder_completion::{
    BasicCompletionRanker, CompletionContext, CompletionItem, CompletionItemKind, CompletionRanker,
    Position,
};

/// Strategy for generating valid completion labels
fn completion_label_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,20}"
        .prop_map(|s| s.to_string())
}

/// Strategy for generating valid prefixes
fn prefix_strategy() -> impl Strategy<Value = String> {
    "[a-z]{0,5}"
        .prop_map(|s| s.to_string())
}

/// Strategy for generating completion items
fn completion_item_strategy() -> impl Strategy<Value = CompletionItem> {
    completion_label_strategy().prop_map(|label| {
        CompletionItem::new(
            label.clone(),
            CompletionItemKind::Variable,
            label,
        )
    })
}

proptest! {
    /// Property: For any typed prefix, only matching completions are returned
    /// This property ensures that prefix filtering works correctly across all inputs
    #[test]
    fn prop_prefix_matching_filters_correctly(
        items in prop::collection::vec(completion_item_strategy(), 1..20),
        prefix in prefix_strategy()
    ) {
        let ranker = BasicCompletionRanker::default_weights();
        let context = CompletionContext::new(
            "rust".to_string(),
            Position::new(0, 0),
            prefix.clone(),
        );

        let ranked = ranker.rank_completions(items.clone(), &context);

        // All returned items should match the prefix (case-insensitive)
        for item in &ranked {
            let filter_text = item.filter_text.as_ref().unwrap_or(&item.label);
            prop_assert!(
                filter_text.to_lowercase().starts_with(&prefix.to_lowercase()),
                "Item '{}' does not match prefix '{}'",
                filter_text,
                prefix
            );
        }
    }

    /// Property: Prefix matching is case-insensitive
    /// This property ensures that case variations don't affect matching
    #[test]
    fn prop_prefix_matching_case_insensitive(
        label in completion_label_strategy(),
        prefix in prefix_strategy()
    ) {
        let ranker = BasicCompletionRanker::default_weights();
        let item = CompletionItem::new(
            label.clone(),
            CompletionItemKind::Variable,
            label.clone(),
        );

        // Test with lowercase prefix
        let context_lower = CompletionContext::new(
            "rust".to_string(),
            Position::new(0, 0),
            prefix.to_lowercase(),
        );
        let ranked_lower = ranker.rank_completions(vec![item.clone()], &context_lower);

        // Test with uppercase prefix
        let context_upper = CompletionContext::new(
            "rust".to_string(),
            Position::new(0, 0),
            prefix.to_uppercase(),
        );
        let ranked_upper = ranker.rank_completions(vec![item.clone()], &context_upper);

        // Both should have the same number of results
        prop_assert_eq!(
            ranked_lower.len(),
            ranked_upper.len(),
            "Case sensitivity affected matching"
        );
    }

    /// Property: Empty prefix matches all items
    /// This property ensures that empty prefix doesn't filter anything
    #[test]
    fn prop_empty_prefix_matches_all(
        items in prop::collection::vec(completion_item_strategy(), 1..20)
    ) {
        let ranker = BasicCompletionRanker::default_weights();
        let context = CompletionContext::new(
            "rust".to_string(),
            Position::new(0, 0),
            "".to_string(),
        );

        let ranked = ranker.rank_completions(items.clone(), &context);

        // All items should be returned when prefix is empty
        prop_assert_eq!(
            ranked.len(),
            items.len(),
            "Empty prefix should match all items"
        );
    }

    /// Property: Prefix matching is consistent
    /// This property ensures that the same input always produces the same output
    #[test]
    fn prop_prefix_matching_consistency(
        items in prop::collection::vec(completion_item_strategy(), 1..20),
        prefix in prefix_strategy()
    ) {
        let ranker = BasicCompletionRanker::default_weights();
        let context = CompletionContext::new(
            "rust".to_string(),
            Position::new(0, 0),
            prefix.clone(),
        );

        let ranked1 = ranker.rank_completions(items.clone(), &context);
        let ranked2 = ranker.rank_completions(items.clone(), &context);

        // Results should be identical
        prop_assert_eq!(
            ranked1.len(),
            ranked2.len(),
            "Prefix matching is not consistent"
        );

        for (item1, item2) in ranked1.iter().zip(ranked2.iter()) {
            prop_assert_eq!(
                &item1.label,
                &item2.label,
                "Prefix matching order is not consistent"
            );
        }
    }

    /// Property: Prefix matching respects filter_text field
    /// This property ensures that filter_text is used for matching when available
    #[test]
    fn prop_prefix_matching_uses_filter_text(
        label in completion_label_strategy(),
        filter_text in completion_label_strategy(),
        prefix in prefix_strategy()
    ) {
        let ranker = BasicCompletionRanker::default_weights();
        let mut item = CompletionItem::new(
            label.clone(),
            CompletionItemKind::Variable,
            label,
        );
        item.filter_text = Some(filter_text.clone());

        let context = CompletionContext::new(
            "rust".to_string(),
            Position::new(0, 0),
            prefix.clone(),
        );

        let ranked = ranker.rank_completions(vec![item], &context);

        // Check if item was included based on filter_text
        let should_match = filter_text.to_lowercase().starts_with(&prefix.to_lowercase());
        let was_matched = !ranked.is_empty();

        prop_assert_eq!(
            should_match,
            was_matched,
            "filter_text was not used for matching"
        );
    }

    /// Property: Exact prefix matches rank higher than partial matches
    /// This property ensures that exact matches are prioritized
    #[test]
    fn prop_exact_prefix_ranks_higher(
        prefix in "[a-z]{2,5}".prop_map(|s| s.to_string())
    ) {
        let ranker = BasicCompletionRanker::default_weights();

        let exact_item = CompletionItem::new(
            prefix.clone(),
            CompletionItemKind::Variable,
            prefix.clone(),
        );

        let partial_item = CompletionItem::new(
            format!("{}extra", prefix),
            CompletionItemKind::Variable,
            format!("{}extra", prefix),
        );

        let context = CompletionContext::new(
            "rust".to_string(),
            Position::new(0, 0),
            prefix.clone(),
        );

        let ranked = ranker.rank_completions(
            vec![partial_item, exact_item],
            &context,
        );

        // Exact match should be first
        prop_assert_eq!(
            &ranked[0].label,
            &prefix,
            "Exact match should rank higher than partial match"
        );
    }

    /// Property: No false positives in prefix matching
    /// This property ensures that non-matching items are not returned
    #[test]
    fn prop_no_false_positives(
        label in "[a-z][a-z0-9_]{0,20}".prop_map(|s| s.to_string()),
        prefix in "[x-z]{1,3}".prop_map(|s| s.to_string())
    ) {
        // Only test when label doesn't start with prefix
        if label.to_lowercase().starts_with(&prefix.to_lowercase()) {
            return Ok(());
        }

        let ranker = BasicCompletionRanker::default_weights();
        let item = CompletionItem::new(
            label.clone(),
            CompletionItemKind::Variable,
            label,
        );

        let context = CompletionContext::new(
            "rust".to_string(),
            Position::new(0, 0),
            prefix.clone(),
        );

        let ranked = ranker.rank_completions(vec![item], &context);

        // Item should not be returned if it doesn't match prefix
        prop_assert!(
            ranked.is_empty(),
            "Non-matching item was returned"
        );
    }
}

