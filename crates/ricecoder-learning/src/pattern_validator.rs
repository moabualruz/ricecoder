/// Pattern validation component for verifying pattern correctness
use crate::error::Result;
use crate::models::{Decision, LearnedPattern};

/// Validates patterns against historical decisions
pub struct PatternValidator;

impl PatternValidator {
    /// Create a new pattern validator
    pub fn new() -> Self {
        Self
    }

    /// Validate a pattern against historical decisions
    ///
    /// Tests the pattern against all historical decisions to verify correctness.
    /// Returns a validation score (0.0 to 1.0) indicating how well the pattern
    /// matches the historical decisions.
    pub fn validate_pattern(
        &self,
        pattern: &LearnedPattern,
        decisions: &[Decision],
    ) -> Result<ValidationResult> {
        if decisions.is_empty() {
            return Ok(ValidationResult {
                pattern_id: pattern.id.clone(),
                is_valid: false,
                validation_score: 0.0,
                matching_decisions: 0,
                total_decisions: 0,
                mismatches: Vec::new(),
                confidence_recommendation: 0.0,
            });
        }

        let mut matching_count = 0;
        let mut mismatches = Vec::new();

        // Count how many decisions match this pattern
        for decision in decisions {
            if decision.decision_type == pattern.pattern_type {
                let matches = self.decision_matches_pattern(decision, pattern);
                if matches {
                    matching_count += 1;
                } else {
                    mismatches.push(decision.id.clone());
                }
            }
        }

        // Calculate validation score
        let validation_score = if !decisions.is_empty() {
            matching_count as f32 / decisions.len() as f32
        } else {
            0.0
        };

        // Determine if pattern is valid (>= 70% match rate)
        let is_valid = validation_score >= 0.7;

        // Recommend confidence based on validation score
        let confidence_recommendation = self.calculate_confidence_recommendation(
            validation_score,
            pattern.occurrences,
            matching_count,
        );

        Ok(ValidationResult {
            pattern_id: pattern.id.clone(),
            is_valid,
            validation_score,
            matching_decisions: matching_count,
            total_decisions: decisions.len(),
            mismatches,
            confidence_recommendation,
        })
    }

    /// Check if a decision matches a pattern
    fn decision_matches_pattern(&self, decision: &Decision, pattern: &LearnedPattern) -> bool {
        // Check if decision type matches
        if decision.decision_type != pattern.pattern_type {
            return false;
        }

        // Check if decision matches any of the pattern examples
        for example in &pattern.examples {
            if decision.input == example.input && decision.output == example.output {
                return true;
            }
        }

        false
    }

    /// Calculate a confidence recommendation based on validation results
    fn calculate_confidence_recommendation(
        &self,
        validation_score: f32,
        occurrences: usize,
        matching_count: usize,
    ) -> f32 {
        // Base confidence on validation score
        let validation_factor = validation_score;

        // Increase confidence with more occurrences (up to a point)
        let occurrence_factor = (occurrences as f32 / 10.0).min(1.0);

        // Increase confidence with more matches
        let match_factor = (matching_count as f32 / 5.0).min(1.0);

        // Combined confidence recommendation
        let confidence = (validation_factor * 0.5) + (occurrence_factor * 0.25) + (match_factor * 0.25);

        confidence.clamp(0.0, 1.0)
    }

    /// Validate multiple patterns
    pub fn validate_patterns(
        &self,
        patterns: &[LearnedPattern],
        decisions: &[Decision],
    ) -> Result<Vec<ValidationResult>> {
        let mut results = Vec::new();

        for pattern in patterns {
            let result = self.validate_pattern(pattern, decisions)?;
            results.push(result);
        }

        Ok(results)
    }

    /// Get validation statistics for a set of patterns
    pub fn get_validation_statistics(
        &self,
        validation_results: &[ValidationResult],
    ) -> ValidationStatistics {
        if validation_results.is_empty() {
            return ValidationStatistics {
                total_patterns: 0,
                valid_patterns: 0,
                invalid_patterns: 0,
                average_validation_score: 0.0,
                average_confidence_recommendation: 0.0,
                total_mismatches: 0,
            };
        }

        let valid_count = validation_results.iter().filter(|r| r.is_valid).count();
        let invalid_count = validation_results.len() - valid_count;
        let avg_score: f32 = validation_results.iter().map(|r| r.validation_score).sum::<f32>()
            / validation_results.len() as f32;
        let avg_confidence: f32 =
            validation_results.iter().map(|r| r.confidence_recommendation).sum::<f32>()
                / validation_results.len() as f32;
        let total_mismatches: usize = validation_results.iter().map(|r| r.mismatches.len()).sum();

        ValidationStatistics {
            total_patterns: validation_results.len(),
            valid_patterns: valid_count,
            invalid_patterns: invalid_count,
            average_validation_score: avg_score,
            average_confidence_recommendation: avg_confidence,
            total_mismatches,
        }
    }
}

impl Default for PatternValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of pattern validation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// ID of the validated pattern
    pub pattern_id: String,
    /// Whether the pattern is valid (>= 70% match rate)
    pub is_valid: bool,
    /// Validation score (0.0 to 1.0)
    pub validation_score: f32,
    /// Number of decisions matching this pattern
    pub matching_decisions: usize,
    /// Total number of decisions analyzed
    pub total_decisions: usize,
    /// IDs of decisions that don't match the pattern
    pub mismatches: Vec<String>,
    /// Recommended confidence score based on validation
    pub confidence_recommendation: f32,
}

/// Statistics about pattern validation results
#[derive(Debug, Clone)]
pub struct ValidationStatistics {
    /// Total number of patterns validated
    pub total_patterns: usize,
    /// Number of valid patterns
    pub valid_patterns: usize,
    /// Number of invalid patterns
    pub invalid_patterns: usize,
    /// Average validation score across all patterns
    pub average_validation_score: f32,
    /// Average confidence recommendation across all patterns
    pub average_confidence_recommendation: f32,
    /// Total number of mismatches across all patterns
    pub total_mismatches: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{DecisionContext, PatternExample};
    use std::path::PathBuf;

    fn create_test_decision(
        decision_type: &str,
        input: serde_json::Value,
        output: serde_json::Value,
    ) -> Decision {
        let context = DecisionContext {
            project_path: PathBuf::from("/project"),
            file_path: PathBuf::from("/project/src/main.rs"),
            line_number: 10,
            agent_type: "test_agent".to_string(),
        };

        Decision::new(context, decision_type.to_string(), input, output)
    }

    #[test]
    fn test_pattern_validator_creation() {
        let validator = PatternValidator::new();
        assert_eq!(std::mem::size_of_val(&validator), 0); // Zero-sized type
    }

    #[test]
    fn test_validate_pattern_empty_decisions() {
        let validator = PatternValidator::new();
        let pattern = LearnedPattern::new("test".to_string(), "Test pattern".to_string());

        let result = validator.validate_pattern(&pattern, &[]).unwrap();
        assert!(!result.is_valid);
        assert_eq!(result.validation_score, 0.0);
        assert_eq!(result.matching_decisions, 0);
    }

    #[test]
    fn test_validate_pattern_matching() {
        let validator = PatternValidator::new();

        let decision1 = create_test_decision(
            "code_generation",
            serde_json::json!({"input": "test"}),
            serde_json::json!({"output": "result"}),
        );

        let decision2 = create_test_decision(
            "code_generation",
            serde_json::json!({"input": "test"}),
            serde_json::json!({"output": "result"}),
        );

        let mut pattern = LearnedPattern::new("code_generation".to_string(), "Test pattern".to_string());
        pattern.examples.push(PatternExample {
            input: serde_json::json!({"input": "test"}),
            output: serde_json::json!({"output": "result"}),
            context: serde_json::json!({}),
        });

        let result = validator
            .validate_pattern(&pattern, &[decision1, decision2])
            .unwrap();

        assert!(result.is_valid);
        assert!(result.validation_score > 0.0);
        assert_eq!(result.matching_decisions, 2);
    }

    #[test]
    fn test_validate_pattern_no_matches() {
        let validator = PatternValidator::new();

        let decision = create_test_decision(
            "code_generation",
            serde_json::json!({"input": "test"}),
            serde_json::json!({"output": "result"}),
        );

        let mut pattern = LearnedPattern::new("code_generation".to_string(), "Test pattern".to_string());
        pattern.examples.push(PatternExample {
            input: serde_json::json!({"input": "different"}),
            output: serde_json::json!({"output": "different"}),
            context: serde_json::json!({}),
        });

        let result = validator.validate_pattern(&pattern, &[decision]).unwrap();

        assert!(!result.is_valid);
        assert_eq!(result.validation_score, 0.0);
        assert_eq!(result.matching_decisions, 0);
    }

    #[test]
    fn test_validate_pattern_partial_matches() {
        let validator = PatternValidator::new();

        let decision1 = create_test_decision(
            "code_generation",
            serde_json::json!({"input": "test"}),
            serde_json::json!({"output": "result"}),
        );

        let decision2 = create_test_decision(
            "code_generation",
            serde_json::json!({"input": "different"}),
            serde_json::json!({"output": "different"}),
        );

        let decision3 = create_test_decision(
            "code_generation",
            serde_json::json!({"input": "test"}),
            serde_json::json!({"output": "result"}),
        );

        let mut pattern = LearnedPattern::new("code_generation".to_string(), "Test pattern".to_string());
        pattern.examples.push(PatternExample {
            input: serde_json::json!({"input": "test"}),
            output: serde_json::json!({"output": "result"}),
            context: serde_json::json!({}),
        });

        let result = validator
            .validate_pattern(&pattern, &[decision1, decision2, decision3])
            .unwrap();

        // 2 out of 3 match = 66.7%, which is < 70%, so invalid
        assert!(!result.is_valid);
        assert!(result.validation_score > 0.6 && result.validation_score < 0.7);
        assert_eq!(result.matching_decisions, 2);
    }

    #[test]
    fn test_validate_pattern_high_match_rate() {
        let validator = PatternValidator::new();

        let decision1 = create_test_decision(
            "code_generation",
            serde_json::json!({"input": "test"}),
            serde_json::json!({"output": "result"}),
        );

        let decision2 = create_test_decision(
            "code_generation",
            serde_json::json!({"input": "test"}),
            serde_json::json!({"output": "result"}),
        );

        let decision3 = create_test_decision(
            "code_generation",
            serde_json::json!({"input": "test"}),
            serde_json::json!({"output": "result"}),
        );

        let decision4 = create_test_decision(
            "code_generation",
            serde_json::json!({"input": "different"}),
            serde_json::json!({"output": "different"}),
        );

        let mut pattern = LearnedPattern::new("code_generation".to_string(), "Test pattern".to_string());
        pattern.examples.push(PatternExample {
            input: serde_json::json!({"input": "test"}),
            output: serde_json::json!({"output": "result"}),
            context: serde_json::json!({}),
        });

        let result = validator
            .validate_pattern(&pattern, &[decision1, decision2, decision3, decision4])
            .unwrap();

        // 3 out of 4 match = 75%, which is >= 70%, so valid
        assert!(result.is_valid);
        assert!(result.validation_score > 0.7);
        assert_eq!(result.matching_decisions, 3);
    }

    #[test]
    fn test_validate_patterns_multiple() {
        let validator = PatternValidator::new();

        let decision1 = create_test_decision(
            "code_generation",
            serde_json::json!({"input": "test"}),
            serde_json::json!({"output": "result"}),
        );

        let decision2 = create_test_decision(
            "refactoring",
            serde_json::json!({"input": "test"}),
            serde_json::json!({"output": "result"}),
        );

        let mut pattern1 = LearnedPattern::new("code_generation".to_string(), "Pattern 1".to_string());
        pattern1.examples.push(PatternExample {
            input: serde_json::json!({"input": "test"}),
            output: serde_json::json!({"output": "result"}),
            context: serde_json::json!({}),
        });

        let mut pattern2 = LearnedPattern::new("refactoring".to_string(), "Pattern 2".to_string());
        pattern2.examples.push(PatternExample {
            input: serde_json::json!({"input": "test"}),
            output: serde_json::json!({"output": "result"}),
            context: serde_json::json!({}),
        });

        let results = validator
            .validate_patterns(&[pattern1, pattern2], &[decision1, decision2])
            .unwrap();

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_get_validation_statistics() {
        let validator = PatternValidator::new();

        let result1 = ValidationResult {
            pattern_id: "pattern1".to_string(),
            is_valid: true,
            validation_score: 0.8,
            matching_decisions: 4,
            total_decisions: 5,
            mismatches: vec!["decision1".to_string()],
            confidence_recommendation: 0.75,
        };

        let result2 = ValidationResult {
            pattern_id: "pattern2".to_string(),
            is_valid: false,
            validation_score: 0.5,
            matching_decisions: 2,
            total_decisions: 4,
            mismatches: vec!["decision2".to_string(), "decision3".to_string()],
            confidence_recommendation: 0.5,
        };

        let stats = validator.get_validation_statistics(&[result1, result2]);

        assert_eq!(stats.total_patterns, 2);
        assert_eq!(stats.valid_patterns, 1);
        assert_eq!(stats.invalid_patterns, 1);
        assert!(stats.average_validation_score > 0.6 && stats.average_validation_score < 0.7);
        assert_eq!(stats.total_mismatches, 3);
    }

    #[test]
    fn test_get_validation_statistics_empty() {
        let validator = PatternValidator::new();
        let stats = validator.get_validation_statistics(&[]);

        assert_eq!(stats.total_patterns, 0);
        assert_eq!(stats.valid_patterns, 0);
        assert_eq!(stats.invalid_patterns, 0);
        assert_eq!(stats.average_validation_score, 0.0);
    }

    #[test]
    fn test_confidence_recommendation_calculation() {
        let validator = PatternValidator::new();

        // Test with high validation score and many occurrences
        let confidence1 = validator.calculate_confidence_recommendation(0.9, 10, 5);
        assert!(confidence1 > 0.7);

        // Test with low validation score
        let confidence2 = validator.calculate_confidence_recommendation(0.3, 1, 1);
        assert!(confidence2 < 0.5);

        // Test with zero values
        let confidence3 = validator.calculate_confidence_recommendation(0.0, 0, 0);
        assert_eq!(confidence3, 0.0);
    }

    #[test]
    fn test_validate_pattern_different_type() {
        let validator = PatternValidator::new();

        let decision = create_test_decision(
            "code_generation",
            serde_json::json!({"input": "test"}),
            serde_json::json!({"output": "result"}),
        );

        let mut pattern = LearnedPattern::new("refactoring".to_string(), "Test pattern".to_string());
        pattern.examples.push(PatternExample {
            input: serde_json::json!({"input": "test"}),
            output: serde_json::json!({"output": "result"}),
            context: serde_json::json!({}),
        });

        let result = validator.validate_pattern(&pattern, &[decision]).unwrap();

        // Pattern type doesn't match decision type, so no matches
        assert!(!result.is_valid);
        assert_eq!(result.matching_decisions, 0);
    }
}
