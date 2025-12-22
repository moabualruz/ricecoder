/// Property-based tests for pattern extraction consistency
/// **Feature: ricecoder-learning, Property 5: Pattern Extraction Consistency**
/// **Validates: Requirements 3.1, 3.2**

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use proptest::prelude::*;

    use crate::{Decision, DecisionContext, PatternCapturer};

    /// Strategy for generating decision contexts
    fn decision_context_strategy() -> impl Strategy<Value = DecisionContext> {
        ("/project", "/project/src/main.rs", 0u32..1000, "test_agent").prop_map(
            |(project, file, line, agent)| DecisionContext {
                project_path: PathBuf::from(project),
                file_path: PathBuf::from(file),
                line_number: line,
                agent_type: agent.to_string(),
            },
        )
    }

    /// Strategy for generating JSON values
    fn json_value_strategy() -> impl Strategy<Value = serde_json::Value> {
        prop_oneof![
            Just(serde_json::json!({})),
            Just(serde_json::json!({"key": "value"})),
            Just(serde_json::json!({"number": 42})),
            Just(serde_json::json!({"array": [1, 2, 3]})),
            Just(serde_json::json!({"nested": {"inner": "value"}})),
        ]
    }

    /// Strategy for generating decisions
    fn decision_strategy() -> impl Strategy<Value = Decision> {
        (
            decision_context_strategy(),
            "code_generation|refactoring|analysis",
            json_value_strategy(),
            json_value_strategy(),
        )
            .prop_map(|(context, decision_type, input, output)| {
                Decision::new(context, decision_type.to_string(), input, output)
            })
    }

    /// Property 5: Pattern Extraction Consistency
    /// For any set of repeated decisions, the Pattern Capturer SHALL extract identical
    /// patterns when processing the same decision history.
    #[test]
    fn prop_pattern_extraction_consistency() {
        proptest!(|(decisions in prop::collection::vec(decision_strategy(), 2..20))| {
            let capturer = PatternCapturer::new();

            // Extract patterns twice from the same decision history
            let patterns1 = capturer.extract_patterns(&decisions).expect("First extraction failed");
            let patterns2 = capturer.extract_patterns(&decisions).expect("Second extraction failed");

            // Both extractions should produce the same number of patterns
            prop_assert_eq!(
                patterns1.len(),
                patterns2.len(),
                "Pattern count should be consistent"
            );

            // Sort patterns by type and occurrences for comparison
            let mut sorted1 = patterns1.clone();
            let mut sorted2 = patterns2.clone();
            sorted1.sort_by(|a, b| a.pattern_type.cmp(&b.pattern_type).then(b.occurrences.cmp(&a.occurrences)));
            sorted2.sort_by(|a, b| a.pattern_type.cmp(&b.pattern_type).then(b.occurrences.cmp(&a.occurrences)));

            // Patterns should have identical properties
            for (p1, p2) in sorted1.iter().zip(sorted2.iter()) {
                prop_assert_eq!(&p1.pattern_type, &p2.pattern_type, "Pattern types should match");
                prop_assert_eq!(&p1.description, &p2.description, "Pattern descriptions should match");
                prop_assert_eq!(p1.occurrences, p2.occurrences, "Pattern occurrences should match");
                prop_assert_eq!(
                    p1.examples.len(),
                    p2.examples.len(),
                    "Pattern examples count should match"
                );

                // Confidence should be bounded
                prop_assert!(p1.confidence >= 0.0 && p1.confidence <= 1.0, "Confidence should be bounded");
                prop_assert!(p2.confidence >= 0.0 && p2.confidence <= 1.0, "Confidence should be bounded");
            }
        });
    }

    /// Property: Pattern extraction should be deterministic
    /// For any decision history, extracting patterns multiple times should always
    /// produce identical results
    #[test]
    fn prop_pattern_extraction_deterministic() {
        proptest!(|(decisions in prop::collection::vec(decision_strategy(), 2..20))| {
            let capturer = PatternCapturer::new();

            // Extract patterns multiple times
            let patterns1 = capturer.extract_patterns(&decisions).expect("Extraction 1 failed");
            let patterns2 = capturer.extract_patterns(&decisions).expect("Extraction 2 failed");
            let patterns3 = capturer.extract_patterns(&decisions).expect("Extraction 3 failed");

            // All extractions should produce the same patterns
            prop_assert_eq!(patterns1.len(), patterns2.len(), "Extraction 1 and 2 should match");
            prop_assert_eq!(patterns2.len(), patterns3.len(), "Extraction 2 and 3 should match");

            // Sort patterns for comparison
            let mut sorted1 = patterns1.clone();
            let mut sorted2 = patterns2.clone();
            let mut sorted3 = patterns3.clone();
            sorted1.sort_by(|a, b| a.pattern_type.cmp(&b.pattern_type).then(b.occurrences.cmp(&a.occurrences)));
            sorted2.sort_by(|a, b| a.pattern_type.cmp(&b.pattern_type).then(b.occurrences.cmp(&a.occurrences)));
            sorted3.sort_by(|a, b| a.pattern_type.cmp(&b.pattern_type).then(b.occurrences.cmp(&a.occurrences)));

            // Verify pattern properties are identical across all extractions
            for (p1, p2) in sorted1.iter().zip(sorted2.iter()) {
                prop_assert_eq!(&p1.pattern_type, &p2.pattern_type, "Type should match");
                prop_assert_eq!(p1.occurrences, p2.occurrences, "Occurrences should match");
            }

            for (p2, p3) in sorted2.iter().zip(sorted3.iter()) {
                prop_assert_eq!(&p2.pattern_type, &p3.pattern_type, "Type should match");
                prop_assert_eq!(p2.occurrences, p3.occurrences, "Occurrences should match");
            }
        });
    }

    /// Property: Pattern extraction should preserve decision information
    /// For any extracted pattern, all examples should be present in the original decisions
    #[test]
    fn prop_pattern_examples_from_decisions() {
        proptest!(|(decisions in prop::collection::vec(decision_strategy(), 2..20))| {
            let capturer = PatternCapturer::new();
            let patterns = capturer.extract_patterns(&decisions).expect("Extraction failed");

            for pattern in patterns {
                // Each example in the pattern should correspond to a decision
                for example in &pattern.examples {
                    let found = decisions.iter().any(|d| {
                        d.decision_type == pattern.pattern_type
                            && d.input == example.input
                            && d.output == example.output
                    });

                    prop_assert!(
                        found,
                        "Pattern example should come from original decisions"
                    );
                }
            }
        });
    }

    /// Property: Pattern extraction should handle empty input
    /// For an empty decision list, pattern extraction should return an empty list
    #[test]
    fn prop_pattern_extraction_empty_input() {
        let capturer = PatternCapturer::new();
        let patterns = capturer.extract_patterns(&[]).expect("Extraction failed");
        assert!(
            patterns.is_empty(),
            "Empty input should produce empty patterns"
        );
    }

    /// Property: Pattern extraction should respect minimum occurrences
    /// For decisions with fewer occurrences than the minimum, no patterns should be extracted
    #[test]
    fn prop_pattern_extraction_respects_minimum() {
        proptest!(|(decision in decision_strategy())| {
            let capturer = PatternCapturer::with_settings(5, 0.5);

            // Single decision should not produce patterns
            let patterns = capturer.extract_patterns(&[decision]).expect("Extraction failed");
            prop_assert!(patterns.is_empty(), "Single decision should not produce patterns");
        });
    }

    /// Property: Pattern confidence should be bounded
    /// For any pattern, confidence should always be between 0 and 1
    #[test]
    fn prop_pattern_confidence_consistency() {
        proptest!(|(decisions in prop::collection::vec(decision_strategy(), 2..20))| {
            let capturer = PatternCapturer::new();

            let patterns1 = capturer.extract_patterns(&decisions).expect("Extraction 1 failed");
            let patterns2 = capturer.extract_patterns(&decisions).expect("Extraction 2 failed");

            // All patterns should have bounded confidence
            for pattern in patterns1.iter().chain(patterns2.iter()) {
                prop_assert!(
                    pattern.confidence >= 0.0 && pattern.confidence <= 1.0,
                    "Confidence should be bounded: {}",
                    pattern.confidence
                );
            }
        });
    }

    /// Property: Pattern extraction should be order-independent for identical decisions
    /// For a set of identical decisions in different orders, patterns should be the same
    #[test]
    fn prop_pattern_extraction_order_independent() {
        proptest!(|(decision in decision_strategy())| {
            let capturer = PatternCapturer::new();

            // Create multiple copies of the same decision
            let decisions = vec![decision.clone(), decision.clone(), decision.clone()];

            // Extract patterns
            let patterns1 = capturer.extract_patterns(&decisions).expect("Extraction 1 failed");

            // Reverse the order
            let mut reversed = decisions.clone();
            reversed.reverse();
            let patterns2 = capturer.extract_patterns(&reversed).expect("Extraction 2 failed");

            // Should produce the same patterns
            prop_assert_eq!(patterns1.len(), patterns2.len(), "Pattern count should match");

            for (p1, p2) in patterns1.iter().zip(patterns2.iter()) {
                prop_assert_eq!(&p1.pattern_type, &p2.pattern_type, "Pattern type should match");
                prop_assert_eq!(p1.occurrences, p2.occurrences, "Occurrences should match");
            }
        });
    }
}
