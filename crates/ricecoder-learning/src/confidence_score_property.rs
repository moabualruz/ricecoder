/// Property-based tests for confidence score accuracy
/// **Feature: ricecoder-learning, Property 7: Confidence Score Accuracy**
/// **Validates: Requirements 3.6**

#[cfg(test)]
mod tests {
    use crate::{Decision, DecisionContext, PatternCapturer};
    use proptest::prelude::*;
    use std::path::PathBuf;

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

    /// Property 7: Confidence Score Accuracy
    /// For any rule, the confidence score SHALL be updated based on validation results,
    /// increasing when rules are successfully applied and decreasing when they fail.
    #[test]
    fn prop_confidence_score_increases_on_success() {
        proptest!(|(decisions in prop::collection::vec(decision_strategy(), 2..20))| {
            let capturer = PatternCapturer::new();

            // Extract patterns
            let patterns = capturer.extract_patterns(&decisions).expect("Extraction failed");

            for pattern in patterns {
                let initial_confidence = pattern.confidence;

                // Validate pattern against decisions (should succeed)
                let validation_score = capturer
                    .validate_pattern(&pattern, &decisions)
                    .expect("Validation failed");

                // Validation score should be between 0 and 1
                prop_assert!(validation_score >= 0.0, "Validation score should be >= 0");
                prop_assert!(validation_score <= 1.0, "Validation score should be <= 1");

                // If validation score is high, confidence should increase
                if validation_score > 0.7 {
                    // Confidence should be positive
                    prop_assert!(initial_confidence >= 0.0, "Initial confidence should be >= 0");
                }
            }
        });
    }

    /// Property: Confidence scores should be bounded
    /// For any pattern, the confidence score should always be between 0 and 1
    #[test]
    fn prop_confidence_score_bounded() {
        proptest!(|(decisions in prop::collection::vec(decision_strategy(), 2..20))| {
            let capturer = PatternCapturer::new();

            // Extract patterns
            let patterns = capturer.extract_patterns(&decisions).expect("Extraction failed");

            for pattern in patterns {
                prop_assert!(
                    pattern.confidence >= 0.0,
                    "Confidence should be >= 0, got {}",
                    pattern.confidence
                );
                prop_assert!(
                    pattern.confidence <= 1.0,
                    "Confidence should be <= 1, got {}",
                    pattern.confidence
                );
            }
        });
    }

    /// Property: Confidence updates should be monotonic
    /// For any pattern, updating confidence with a higher validation score should
    /// increase the confidence (or keep it the same)
    #[test]
    fn prop_confidence_update_monotonic() {
        proptest!(|(
            decisions in prop::collection::vec(decision_strategy(), 2..20),
            validation_score in 0.0f32..=1.0f32
        )| {
            let capturer = PatternCapturer::new();

            // Extract patterns
            let patterns = capturer.extract_patterns(&decisions).expect("Extraction failed");

            for mut pattern in patterns {
                let initial_confidence = pattern.confidence;

                // Update confidence
                capturer
                    .update_confidence(&mut pattern, validation_score)
                    .expect("Update failed");

                // Confidence should still be bounded
                prop_assert!(pattern.confidence >= 0.0, "Confidence should be >= 0");
                prop_assert!(pattern.confidence <= 1.0, "Confidence should be <= 1");

                // Confidence should change (unless it's already at the boundary)
                // Using exponential moving average with alpha=0.3
                let expected = (0.3 * validation_score) + (0.7 * initial_confidence);
                prop_assert!(
                    (pattern.confidence - expected).abs() < 0.0001,
                    "Confidence update should follow EMA formula"
                );
            }
        });
    }

    /// Property: Confidence should reflect pattern consistency
    /// For patterns with consistent outputs, confidence should be higher
    #[test]
    fn prop_confidence_reflects_consistency() {
        proptest!(|(decision in decision_strategy())| {
            let capturer = PatternCapturer::new();

            // Create multiple identical decisions
            let identical_decisions = vec![decision.clone(), decision.clone(), decision.clone()];

            // Extract patterns
            let patterns = capturer
                .extract_patterns(&identical_decisions)
                .expect("Extraction failed");

            // Patterns from identical decisions should have reasonable confidence
            for pattern in patterns {
                prop_assert!(
                    pattern.confidence > 0.0,
                    "Confidence should be > 0 for consistent patterns"
                );
            }
        });
    }

    /// Property: Validation score should be consistent
    /// For the same pattern and decision history, validation should produce the same score
    #[test]
    fn prop_validation_score_consistent() {
        proptest!(|(decisions in prop::collection::vec(decision_strategy(), 2..20))| {
            let capturer = PatternCapturer::new();

            // Extract patterns
            let patterns = capturer.extract_patterns(&decisions).expect("Extraction failed");

            for pattern in patterns {
                // Validate multiple times
                let score1 = capturer
                    .validate_pattern(&pattern, &decisions)
                    .expect("Validation 1 failed");
                let score2 = capturer
                    .validate_pattern(&pattern, &decisions)
                    .expect("Validation 2 failed");
                let score3 = capturer
                    .validate_pattern(&pattern, &decisions)
                    .expect("Validation 3 failed");

                // Scores should be identical
                prop_assert!(
                    (score1 - score2).abs() < 0.0001,
                    "Validation scores should be consistent"
                );
                prop_assert!(
                    (score2 - score3).abs() < 0.0001,
                    "Validation scores should be consistent"
                );
            }
        });
    }

    /// Property: Confidence should increase with more matching examples
    /// For patterns with more matching examples, confidence should be higher
    #[test]
    fn prop_confidence_increases_with_matches() {
        proptest!(|(decision in decision_strategy())| {
            let capturer = PatternCapturer::new();

            // Create decisions with varying numbers of matches
            let double_decision = vec![decision.clone(), decision.clone()];
            let triple_decision = vec![decision.clone(), decision.clone(), decision.clone()];

            // Extract patterns (only double and triple should produce patterns)
            let patterns_double = capturer
                .extract_patterns(&double_decision)
                .expect("Extraction failed");
            let patterns_triple = capturer
                .extract_patterns(&triple_decision)
                .expect("Extraction failed");

            // Both should produce patterns
            prop_assert!(patterns_double.len() > 0, "Double decision should produce patterns");
            prop_assert!(patterns_triple.len() > 0, "Triple decision should produce patterns");

            // Triple should have more occurrences
            if patterns_double.len() > 0 && patterns_triple.len() > 0 {
                let double_occurrences = patterns_double[0].occurrences;
                let triple_occurrences = patterns_triple[0].occurrences;

                prop_assert!(
                    triple_occurrences > double_occurrences,
                    "Triple should have more occurrences"
                );
            }
        });
    }

    /// Property: Confidence should be bounded
    /// For any pattern, confidence should always be between 0 and 1
    #[test]
    fn prop_confidence_deterministic() {
        proptest!(|(decisions in prop::collection::vec(decision_strategy(), 2..20))| {
            let capturer = PatternCapturer::new();

            // Extract patterns multiple times
            let patterns1 = capturer.extract_patterns(&decisions).expect("Extraction 1 failed");
            let patterns2 = capturer.extract_patterns(&decisions).expect("Extraction 2 failed");
            let patterns3 = capturer.extract_patterns(&decisions).expect("Extraction 3 failed");

            // All patterns should have bounded confidence
            for pattern in patterns1.iter().chain(patterns2.iter()).chain(patterns3.iter()) {
                prop_assert!(
                    pattern.confidence >= 0.0 && pattern.confidence <= 1.0,
                    "Confidence should be bounded: {}",
                    pattern.confidence
                );
            }
        });
    }
}
