/// Pattern extraction from decision history
use crate::error::{LearningError, Result};
use crate::models::{Decision, LearnedPattern, PatternExample};
use std::collections::HashMap;

/// Extracts implicit patterns from repeated user decisions
pub struct PatternCapturer {
    /// Minimum occurrences to consider a pattern valid
    min_occurrences: usize,
    /// Minimum confidence threshold
    min_confidence: f32,
}

impl PatternCapturer {
    /// Create a new pattern capturer with default settings
    pub fn new() -> Self {
        Self {
            min_occurrences: 2,
            min_confidence: 0.5,
        }
    }

    /// Create a new pattern capturer with custom settings
    pub fn with_settings(min_occurrences: usize, min_confidence: f32) -> Self {
        Self {
            min_occurrences,
            min_confidence,
        }
    }

    /// Extract patterns from a list of decisions
    pub fn extract_patterns(&self, decisions: &[Decision]) -> Result<Vec<LearnedPattern>> {
        if decisions.is_empty() {
            return Ok(Vec::new());
        }

        // Group decisions by type
        let mut decisions_by_type: HashMap<String, Vec<&Decision>> = HashMap::new();
        for decision in decisions {
            decisions_by_type
                .entry(decision.decision_type.clone())
                .or_default()
                .push(decision);
        }

        let mut patterns = Vec::new();

        // Extract patterns for each decision type
        for (decision_type, type_decisions) in decisions_by_type {
            if type_decisions.len() >= self.min_occurrences {
                // Try to find similar patterns
                let extracted = self.extract_patterns_for_type(&decision_type, &type_decisions)?;
                patterns.extend(extracted);
            }
        }

        Ok(patterns)
    }

    /// Extract patterns for a specific decision type
    fn extract_patterns_for_type(
        &self,
        decision_type: &str,
        decisions: &[&Decision],
    ) -> Result<Vec<LearnedPattern>> {
        let mut patterns = Vec::new();

        // Group by input similarity using a deterministic approach
        let mut input_groups: HashMap<String, Vec<&Decision>> = HashMap::new();
        for decision in decisions {
            // Use JSON string representation for deterministic grouping
            let input_key = serde_json::to_string(&decision.input)
                .unwrap_or_else(|_| "unknown".to_string());
            input_groups
                .entry(input_key)
                .or_default()
                .push(decision);
        }

        // Create patterns from similar inputs
        for (_input_key, group) in input_groups {
            if group.len() >= self.min_occurrences {
                let pattern = self.create_pattern_from_group(decision_type, &group)?;
                if pattern.confidence >= self.min_confidence {
                    patterns.push(pattern);
                }
            }
        }

        Ok(patterns)
    }

    /// Create a pattern from a group of similar decisions
    fn create_pattern_from_group(
        &self,
        decision_type: &str,
        decisions: &[&Decision],
    ) -> Result<LearnedPattern> {
        if decisions.is_empty() {
            return Err(LearningError::PatternExtractionFailed(
                "Cannot create pattern from empty group".to_string(),
            ));
        }

        // Create a deterministic ID based on the pattern content
        let pattern_content = format!(
            "{}:{}",
            decision_type,
            serde_json::to_string(&decisions[0].input).unwrap_or_default()
        );
        let pattern_id = format!(
            "{:x}",
            md5::compute(pattern_content.as_bytes())
        );

        let mut pattern = LearnedPattern {
            id: pattern_id,
            pattern_type: decision_type.to_string(),
            description: format!("Pattern for {}", decision_type),
            examples: Vec::new(),
            confidence: 0.0,
            occurrences: 0,
            created_at: chrono::Utc::now(),
            last_seen: chrono::Utc::now(),
        };

        // Add examples from the group
        for decision in decisions {
            let example = PatternExample {
                input: decision.input.clone(),
                output: decision.output.clone(),
                context: serde_json::json!({
                    "agent_type": decision.context.agent_type,
                    "file_path": decision.context.file_path.to_string_lossy(),
                    "line_number": decision.context.line_number,
                }),
            };
            pattern.examples.push(example);
        }

        // Calculate confidence based on consistency
        pattern.occurrences = decisions.len();
        pattern.confidence = self.calculate_confidence(decisions)?;
        pattern.last_seen = decisions.last().unwrap().timestamp;

        Ok(pattern)
    }

    /// Calculate confidence score for a pattern
    fn calculate_confidence(&self, decisions: &[&Decision]) -> Result<f32> {
        if decisions.is_empty() {
            return Ok(0.0);
        }

        // Base confidence on consistency of outputs
        let output_consistency = if decisions.len() > 1 {
            // Count how many decisions have the same output as the majority
            let mut output_counts: HashMap<String, usize> = HashMap::new();
            for decision in decisions {
                let output_str = serde_json::to_string(&decision.output)
                    .unwrap_or_else(|_| "unknown".to_string());
                *output_counts.entry(output_str).or_insert(0) += 1;
            }

            // Find the most common output
            let max_count = output_counts.values().max().copied().unwrap_or(0);
            max_count as f32 / decisions.len() as f32
        } else {
            0.5 // Single example gets moderate confidence
        };

        // Confidence increases with more occurrences (up to a point)
        let occurrence_factor = (decisions.len() as f32 / 10.0).min(1.0);

        // Combined confidence
        let confidence = (output_consistency * 0.7) + (occurrence_factor * 0.3);

        Ok(confidence.clamp(0.0, 1.0))
    }

    /// Check if two outputs are similar
    #[allow(dead_code)]
    fn outputs_are_similar(&self, output1: &serde_json::Value, output2: &serde_json::Value) -> bool {
        // For now, use exact equality
        // In a more sophisticated implementation, this could use fuzzy matching
        output1 == output2
    }

    /// Compute a hash of the input for grouping
    #[allow(dead_code)]
    fn compute_input_hash(&self, input: &serde_json::Value) -> String {
        // Use JSON string representation as hash
        // In production, this could use a more sophisticated hashing strategy
        format!("{:?}", input)
    }

    /// Validate a pattern against historical decisions
    pub fn validate_pattern(
        &self,
        pattern: &LearnedPattern,
        decisions: &[Decision],
    ) -> Result<f32> {
        if decisions.is_empty() {
            return Ok(0.0);
        }

        let mut matching_count = 0;

        for decision in decisions {
            if decision.decision_type == pattern.pattern_type {
                // Check if this decision matches the pattern
                for example in &pattern.examples {
                    if self.decision_matches_example(decision, example) {
                        matching_count += 1;
                        break;
                    }
                }
            }
        }

        let validation_score = matching_count as f32 / decisions.len() as f32;
        Ok(validation_score)
    }

    /// Check if a decision matches a pattern example
    fn decision_matches_example(&self, decision: &Decision, example: &PatternExample) -> bool {
        // Check if input and output match
        decision.input == example.input && decision.output == example.output
    }

    /// Update pattern confidence based on validation results
    pub fn update_confidence(
        &self,
        pattern: &mut LearnedPattern,
        validation_score: f32,
    ) -> Result<()> {
        // Update confidence using exponential moving average
        let alpha = 0.3; // Smoothing factor
        pattern.confidence = (alpha * validation_score) + ((1.0 - alpha) * pattern.confidence);

        Ok(())
    }

    /// Extract patterns with detailed analysis
    pub fn extract_patterns_with_analysis(
        &self,
        decisions: &[Decision],
    ) -> Result<Vec<(LearnedPattern, PatternAnalysis)>> {
        let patterns = self.extract_patterns(decisions)?;

        let mut results = Vec::new();
        for pattern in patterns {
            let analysis = self.analyze_pattern(&pattern, decisions)?;
            results.push((pattern, analysis));
        }

        Ok(results)
    }

    /// Analyze a pattern in detail
    fn analyze_pattern(
        &self,
        pattern: &LearnedPattern,
        decisions: &[Decision],
    ) -> Result<PatternAnalysis> {
        let validation_score = self.validate_pattern(pattern, decisions)?;

        let mut matching_decisions = 0;
        for decision in decisions {
            if decision.decision_type == pattern.pattern_type {
                for example in &pattern.examples {
                    if self.decision_matches_example(decision, example) {
                        matching_decisions += 1;
                        break;
                    }
                }
            }
        }

        Ok(PatternAnalysis {
            pattern_id: pattern.id.clone(),
            validation_score,
            matching_decisions,
            total_decisions: decisions.len(),
            confidence: pattern.confidence,
            occurrences: pattern.occurrences,
        })
    }
}

impl Default for PatternCapturer {
    fn default() -> Self {
        Self::new()
    }
}

/// Analysis results for a pattern
#[derive(Debug, Clone)]
pub struct PatternAnalysis {
    /// ID of the pattern
    pub pattern_id: String,
    /// Validation score (0.0 to 1.0)
    pub validation_score: f32,
    /// Number of decisions matching this pattern
    pub matching_decisions: usize,
    /// Total number of decisions analyzed
    pub total_decisions: usize,
    /// Pattern confidence score
    pub confidence: f32,
    /// Number of occurrences
    pub occurrences: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::DecisionContext;
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
    fn test_pattern_capturer_creation() {
        let capturer = PatternCapturer::new();
        assert_eq!(capturer.min_occurrences, 2);
        assert_eq!(capturer.min_confidence, 0.5);
    }

    #[test]
    fn test_pattern_capturer_with_settings() {
        let capturer = PatternCapturer::with_settings(3, 0.7);
        assert_eq!(capturer.min_occurrences, 3);
        assert_eq!(capturer.min_confidence, 0.7);
    }

    #[test]
    fn test_extract_patterns_empty() {
        let capturer = PatternCapturer::new();
        let patterns = capturer.extract_patterns(&[]).unwrap();
        assert!(patterns.is_empty());
    }

    #[test]
    fn test_extract_patterns_single_decision() {
        let capturer = PatternCapturer::new();

        let decision = create_test_decision(
            "code_generation",
            serde_json::json!({"input": "test"}),
            serde_json::json!({"output": "result"}),
        );

        let patterns = capturer.extract_patterns(&[decision]).unwrap();
        // Single decision should not create a pattern (min_occurrences = 2)
        assert!(patterns.is_empty());
    }

    #[test]
    fn test_extract_patterns_multiple_decisions() {
        let capturer = PatternCapturer::new();

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

        let patterns = capturer.extract_patterns(&[decision1, decision2]).unwrap();
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].pattern_type, "code_generation");
        assert_eq!(patterns[0].occurrences, 2);
    }

    #[test]
    fn test_extract_patterns_different_types() {
        let capturer = PatternCapturer::new();

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
            "refactoring",
            serde_json::json!({"input": "test"}),
            serde_json::json!({"output": "result"}),
        );

        let patterns = capturer
            .extract_patterns(&[decision1, decision2, decision3])
            .unwrap();
        // Only code_generation should have enough occurrences
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].pattern_type, "code_generation");
    }

    #[test]
    fn test_calculate_confidence() {
        let capturer = PatternCapturer::new();

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

        let confidence = capturer.calculate_confidence(&[&decision1, &decision2]).unwrap();
        assert!(confidence > 0.0);
        assert!(confidence <= 1.0);
    }

    #[test]
    fn test_validate_pattern() {
        let capturer = PatternCapturer::new();

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

        let patterns = capturer.extract_patterns(&[decision1.clone(), decision2]).unwrap();
        assert_eq!(patterns.len(), 1);

        let validation_score = capturer
            .validate_pattern(&patterns[0], &[decision1])
            .unwrap();
        assert!(validation_score >= 0.0);
        assert!(validation_score <= 1.0);
    }

    #[test]
    fn test_update_confidence() {
        let capturer = PatternCapturer::new();

        let mut pattern = LearnedPattern::new(
            "code_generation".to_string(),
            "Test pattern".to_string(),
        );

        let initial_confidence = pattern.confidence;
        capturer.update_confidence(&mut pattern, 0.9).unwrap();

        assert_ne!(pattern.confidence, initial_confidence);
        assert!(pattern.confidence > initial_confidence);
    }

    #[test]
    fn test_extract_patterns_with_analysis() {
        let capturer = PatternCapturer::new();

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

        let results = capturer
            .extract_patterns_with_analysis(&[decision1, decision2])
            .unwrap();

        assert_eq!(results.len(), 1);
        let (pattern, analysis) = &results[0];
        assert_eq!(pattern.pattern_type, "code_generation");
        assert!(analysis.validation_score >= 0.0);
        assert!(analysis.validation_score <= 1.0);
    }

    #[test]
    fn test_pattern_examples() {
        let capturer = PatternCapturer::new();

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

        let patterns = capturer.extract_patterns(&[decision1, decision2]).unwrap();
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].examples.len(), 2);
    }
}
