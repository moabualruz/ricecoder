//! Property-based tests for semantic features
//!
//! **Feature: ricecoder-external-lsp, Property 3: Graceful Degradation**
//! **Validates: Requirements ELSP-4.3, ELSP-5.4, ELSP-6.5**

use proptest::prelude::*;
use ricecoder_completion::types::{CompletionItem, CompletionItemKind};
use ricecoder_external_lsp::merger::{CompletionMerger, DiagnosticsMerger, HoverMerger};
use ricecoder_external_lsp::types::MergeConfig;
use ricecoder_lsp::types::{Diagnostic, DiagnosticSeverity, Position, Range};

/// Strategy for generating completion items
fn arb_completion_item() -> impl Strategy<Value = CompletionItem> {
    (
        "[a-z][a-z0-9_]{0,10}",
        0.0f32..1.0f32,
        prop::option::of("[a-z0-9 ]{0,50}"),
    )
        .prop_map(|(label, score, detail)| {
            let mut item = CompletionItem::new(
                label,
                CompletionItemKind::Variable,
                "insert_text".to_string(),
            )
            .with_score(score);

            if let Some(d) = detail {
                item = item.with_detail(d);
            }

            item
        })
}

/// Strategy for generating completion item vectors
fn arb_completion_items() -> impl Strategy<Value = Vec<CompletionItem>> {
    prop::collection::vec(arb_completion_item(), 0..10)
}

/// Strategy for generating diagnostics
fn arb_diagnostic() -> impl Strategy<Value = Diagnostic> {
    (0u32..100, 0u32..100, "[a-z ]{5,50}").prop_map(|(line, char, message)| {
        Diagnostic::new(
            Range::new(Position::new(line, char), Position::new(line, char + 5)),
            DiagnosticSeverity::Error,
            message,
        )
    })
}

/// Strategy for generating diagnostic vectors
fn arb_diagnostics() -> impl Strategy<Value = Vec<Diagnostic>> {
    prop::collection::vec(arb_diagnostic(), 0..10)
}

/// Property 3: Graceful Degradation - Completion
///
/// For any external LSP completion failure, the system SHALL fall back to internal
/// completions without user-visible errors (other than reduced functionality).
///
/// This property tests that:
/// 1. When external completions are None, internal completions are used
/// 2. Merging never panics or returns invalid results
#[test]
fn prop_graceful_degradation_completion() {
    proptest!(|(
        external in prop::option::of(arb_completion_items()),
        internal in arb_completion_items(),
    )| {
        let config = MergeConfig {
            include_internal: true,
            deduplicate: true,
        };

        // This should never panic
        let result = CompletionMerger::merge(external.clone(), internal.clone(), &config);

        // Verify graceful degradation:
        // If external is None, we should have internal items (possibly deduplicated)
        if external.is_none() {
            // Internal items may be deduplicated, so result.len() <= internal.len()
            prop_assert!(result.len() <= internal.len(), "Should use internal completions when external unavailable");
        }

        // All items should be valid
        for item in &result {
            prop_assert!(!item.label.is_empty(), "Completion label should not be empty");
            prop_assert!(!item.insert_text.is_empty(), "Completion insert_text should not be empty");
        }

        // Results should be sorted by score
        for i in 1..result.len() {
            prop_assert!(
                result[i - 1].score >= result[i].score,
                "Results should be sorted by score (descending)"
            );
        }
    });
}

/// Property 3: Graceful Degradation - Diagnostics
///
/// For any external LSP diagnostics failure, the system SHALL fall back to internal
/// diagnostics without user-visible errors (other than reduced functionality).
///
/// This property tests that:
/// 1. When external diagnostics are None, internal diagnostics are used
/// 2. Merging never panics or returns invalid results
#[test]
fn prop_graceful_degradation_diagnostics() {
    proptest!(|(
        external in prop::option::of(arb_diagnostics()),
        internal in arb_diagnostics(),
    )| {
        let config = MergeConfig {
            include_internal: true,
            deduplicate: true,
        };

        // This should never panic
        let result = DiagnosticsMerger::merge(external.clone(), internal.clone(), &config);

        // Verify graceful degradation:
        // If external is None, we should have internal items (possibly deduplicated)
        if external.is_none() {
            // Internal items may be deduplicated, so result.len() <= internal.len()
            prop_assert!(result.len() <= internal.len(), "Should use internal diagnostics when external unavailable");
        }

        // All items should be valid
        for diag in &result {
            prop_assert!(!diag.message.is_empty(), "Diagnostic message should not be empty");
            prop_assert!(diag.range.start.line <= diag.range.end.line, "Range should be valid");
        }
    });
}

/// Property 3: Graceful Degradation - Hover
///
/// For any external LSP hover failure, the system SHALL fall back to internal
/// hover information without user-visible errors (other than reduced functionality).
///
/// This property tests that:
/// 1. When external hover is None, internal hover is used
/// 2. Merging never panics or returns invalid results
#[test]
fn prop_graceful_degradation_hover() {
    proptest!(|(
        external in prop::option::of("[a-z ]{5,100}"),
        internal in prop::option::of("[a-z ]{5,100}"),
    )| {
        let config = MergeConfig {
            include_internal: true,
            deduplicate: true,
        };

        // This should never panic
        let result = HoverMerger::merge(external.clone(), internal.clone(), &config);

        // Verify graceful degradation:
        // If external is None, we should have internal hover
        if external.is_none() {
            prop_assert_eq!(result, internal, "Should use internal hover when external unavailable");
        } else {
            prop_assert_eq!(result, external, "Should prefer external hover when available");
        }
    });
}

/// Property 3: Graceful Degradation - Completion with disabled internal
///
/// When internal provider is disabled, the system should still work but with
/// reduced functionality (only external completions available).
#[test]
fn prop_graceful_degradation_completion_no_internal() {
    proptest!(|(
        external in prop::option::of(arb_completion_items()),
        internal in arb_completion_items(),
    )| {
        let config = MergeConfig {
            include_internal: false,
            deduplicate: true,
        };

        // This should never panic
        let result = CompletionMerger::merge(external.clone(), internal, &config);

        // Should only have external items (possibly deduplicated)
        if let Some(ext) = external {
            // Result should be <= external items (deduplication may reduce count)
            prop_assert!(result.len() <= ext.len(), "Should only use external completions");
            // All result items should come from external
            for result_item in &result {
                let found = ext.iter().any(|e| e.label == result_item.label);
                prop_assert!(found, "Result item should come from external");
            }
        } else {
            prop_assert_eq!(result.len(), 0, "Should have no completions when external unavailable and internal disabled");
        }
    });
}

/// Property 3: Graceful Degradation - Diagnostics with disabled internal
///
/// When internal provider is disabled, the system should still work but with
/// reduced functionality (only external diagnostics available).
#[test]
fn prop_graceful_degradation_diagnostics_no_internal() {
    proptest!(|(
        external in prop::option::of(arb_diagnostics()),
        internal in arb_diagnostics(),
    )| {
        let config = MergeConfig {
            include_internal: false,
            deduplicate: true,
        };

        // This should never panic
        let result = DiagnosticsMerger::merge(external.clone(), internal, &config);

        // Should only have external items
        if let Some(ext) = external {
            prop_assert_eq!(result.len(), ext.len(), "Should only use external diagnostics");
        } else {
            prop_assert_eq!(result.len(), 0, "Should have no diagnostics when external unavailable and internal disabled");
        }
    });
}

/// Property 3: Graceful Degradation - Deduplication doesn't lose data
///
/// Deduplication should never lose data - it should only remove exact duplicates.
#[test]
fn prop_graceful_degradation_deduplication_preserves_data() {
    proptest!(|(
        external in arb_completion_items(),
        internal in arb_completion_items(),
    )| {
        let config = MergeConfig {
            include_internal: true,
            deduplicate: true,
        };

        let result = CompletionMerger::merge(Some(external.clone()), internal.clone(), &config);

        // Total items should be <= external + internal
        prop_assert!(result.len() <= external.len() + internal.len(), "Deduplication should not create new items");

        // All result items should come from either external or internal
        for result_item in &result {
            let found_in_external = external.iter().any(|e| e.label == result_item.label);
            let found_in_internal = internal.iter().any(|i| i.label == result_item.label);
            prop_assert!(found_in_external || found_in_internal, "Result item should come from external or internal");
        }
    });
}

/// Property 3: Graceful Degradation - Merging is idempotent
///
/// Merging the same results multiple times should produce the same output.
#[test]
fn prop_graceful_degradation_merge_idempotent() {
    proptest!(|(
        external in prop::option::of(arb_completion_items()),
        internal in arb_completion_items(),
    )| {
        let config = MergeConfig {
            include_internal: true,
            deduplicate: true,
        };

        let result1 = CompletionMerger::merge(external.clone(), internal.clone(), &config);
        let result2 = CompletionMerger::merge(external, internal, &config);

        prop_assert_eq!(result1.len(), result2.len(), "Merging should be idempotent");

        for (item1, item2) in result1.iter().zip(result2.iter()) {
            prop_assert_eq!(&item1.label, &item2.label, "Merged items should be identical");
            prop_assert_eq!(item1.score, item2.score, "Merged scores should be identical");
        }
    });
}
