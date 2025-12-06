/// Drift detection component for identifying architectural deviations
///
/// Detects deviations from established architectural patterns and warns
/// about architectural drift.

use crate::error::{LearningError, Result};
use crate::intent_tracker::{ArchitecturalDecision, DriftDetection};
use crate::models::LearnedPattern;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for drift detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftDetectionConfig {
    /// Confidence threshold for detecting drift (0.0 to 1.0)
    pub confidence_threshold: f32,
    /// Minimum occurrences before considering a pattern established
    pub min_occurrences_for_pattern: usize,
    /// Enable strict mode (more sensitive to deviations)
    pub strict_mode: bool,
}

impl Default for DriftDetectionConfig {
    fn default() -> Self {
        Self {
            confidence_threshold: 0.7,
            min_occurrences_for_pattern: 3,
            strict_mode: false,
        }
    }
}

/// Drift detection engine
pub struct DriftDetector {
    /// Configuration for drift detection
    config: DriftDetectionConfig,
    /// Established patterns
    patterns: HashMap<String, LearnedPattern>,
    /// Detected drifts
    drifts: Vec<DriftDetection>,
}

impl DriftDetector {
    /// Create a new drift detector with default configuration
    pub fn new() -> Self {
        Self::with_config(DriftDetectionConfig::default())
    }

    /// Create a new drift detector with custom configuration
    pub fn with_config(config: DriftDetectionConfig) -> Self {
        Self {
            config,
            patterns: HashMap::new(),
            drifts: Vec::new(),
        }
    }

    /// Register an established pattern
    pub fn register_pattern(&mut self, pattern: LearnedPattern) -> Result<()> {
        if pattern.occurrences < self.config.min_occurrences_for_pattern {
            return Err(LearningError::PatternExtractionFailed(
                format!(
                    "Pattern must have at least {} occurrences",
                    self.config.min_occurrences_for_pattern
                ),
            ));
        }

        self.patterns.insert(pattern.id.clone(), pattern);
        Ok(())
    }

    /// Check if a decision deviates from established patterns
    pub fn check_deviation(
        &mut self,
        decision: &ArchitecturalDecision,
        pattern_type: &str,
    ) -> Result<Option<DriftDetection>> {
        // Find matching pattern
        let pattern = self
            .patterns
            .values()
            .find(|p| p.pattern_type.contains(pattern_type));

        if let Some(pattern) = pattern {
            // Check if decision confidence is significantly lower than pattern confidence
            if decision.confidence < pattern.confidence * self.config.confidence_threshold {
                let drift_type = if self.config.strict_mode {
                    "violation"
                } else {
                    "deviation"
                };

                let drift = DriftDetection::new(
                    decision.id.clone(),
                    drift_type.to_string(),
                    "medium".to_string(),
                    format!(
                        "Decision confidence ({:.2}) is below pattern confidence ({:.2})",
                        decision.confidence, pattern.confidence
                    ),
                );

                self.drifts.push(drift.clone());
                Ok(Some(drift))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Detect inconsistencies in decision application
    pub fn detect_inconsistency(
        &mut self,
        decision_id: &str,
        expected_behavior: &str,
        actual_behavior: &str,
    ) -> Result<DriftDetection> {
        if expected_behavior == actual_behavior {
            return Err(LearningError::PatternExtractionFailed(
                "No inconsistency detected".to_string(),
            ));
        }

        let drift = DriftDetection::new(
            decision_id.to_string(),
            "inconsistency".to_string(),
            "low".to_string(),
            format!(
                "Expected: {}, Actual: {}",
                expected_behavior, actual_behavior
            ),
        );

        self.drifts.push(drift.clone());
        Ok(drift)
    }

    /// Detect pattern violations
    pub fn detect_violation(
        &mut self,
        decision_id: &str,
        violation_description: &str,
    ) -> Result<DriftDetection> {
        let drift = DriftDetection::new(
            decision_id.to_string(),
            "violation".to_string(),
            "high".to_string(),
            violation_description.to_string(),
        );

        self.drifts.push(drift.clone());
        Ok(drift)
    }

    /// Get all detected drifts
    pub fn get_drifts(&self) -> Vec<DriftDetection> {
        self.drifts.clone()
    }

    /// Get drifts by severity
    pub fn get_drifts_by_severity(&self, severity: &str) -> Vec<DriftDetection> {
        self.drifts
            .iter()
            .filter(|d| d.severity == severity)
            .cloned()
            .collect()
    }

    /// Get drifts for a specific decision
    pub fn get_drifts_for_decision(&self, decision_id: &str) -> Vec<DriftDetection> {
        self.drifts
            .iter()
            .filter(|d| d.decision_id == decision_id)
            .cloned()
            .collect()
    }

    /// Clear all detected drifts
    pub fn clear_drifts(&mut self) {
        self.drifts.clear();
    }

    /// Get drift statistics
    pub fn get_statistics(&self) -> DriftStatistics {
        let total_drifts = self.drifts.len();
        let high_severity = self
            .drifts
            .iter()
            .filter(|d| d.severity == "high")
            .count();
        let medium_severity = self
            .drifts
            .iter()
            .filter(|d| d.severity == "medium")
            .count();
        let low_severity = self
            .drifts
            .iter()
            .filter(|d| d.severity == "low")
            .count();

        let violations = self
            .drifts
            .iter()
            .filter(|d| d.drift_type == "violation")
            .count();
        let deviations = self
            .drifts
            .iter()
            .filter(|d| d.drift_type == "deviation")
            .count();
        let inconsistencies = self
            .drifts
            .iter()
            .filter(|d| d.drift_type == "inconsistency")
            .count();

        DriftStatistics {
            total_drifts,
            high_severity,
            medium_severity,
            low_severity,
            violations,
            deviations,
            inconsistencies,
        }
    }
}

impl Default for DriftDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about detected drifts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftStatistics {
    /// Total number of drifts detected
    pub total_drifts: usize,
    /// Number of high-severity drifts
    pub high_severity: usize,
    /// Number of medium-severity drifts
    pub medium_severity: usize,
    /// Number of low-severity drifts
    pub low_severity: usize,
    /// Number of violations
    pub violations: usize,
    /// Number of deviations
    pub deviations: usize,
    /// Number of inconsistencies
    pub inconsistencies: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intent_tracker::ArchitecturalDecision;
    use crate::models::LearnedPattern;

    #[test]
    fn test_drift_detector_creation() {
        let detector = DriftDetector::new();
        assert_eq!(detector.get_drifts().len(), 0);
    }

    #[test]
    fn test_register_pattern() {
        let mut detector = DriftDetector::new();
        let mut pattern = LearnedPattern::new(
            "layering".to_string(),
            "Layered architecture pattern".to_string(),
        );
        pattern.occurrences = 5;
        pattern.confidence = 0.9;

        assert!(detector.register_pattern(pattern).is_ok());
    }

    #[test]
    fn test_register_pattern_insufficient_occurrences() {
        let mut detector = DriftDetector::new();
        let pattern = LearnedPattern::new(
            "layering".to_string(),
            "Layered architecture pattern".to_string(),
        );

        assert!(detector.register_pattern(pattern).is_err());
    }

    #[test]
    fn test_detect_inconsistency() {
        let mut detector = DriftDetector::new();
        let drift = detector
            .detect_inconsistency(
                "decision_1",
                "async_pattern",
                "sync_pattern",
            )
            .expect("Failed to detect inconsistency");

        assert_eq!(drift.drift_type, "inconsistency");
        assert_eq!(drift.severity, "low");
        assert_eq!(detector.get_drifts().len(), 1);
    }

    #[test]
    fn test_detect_violation() {
        let mut detector = DriftDetector::new();
        let drift = detector
            .detect_violation(
                "decision_1",
                "Direct layer bypass detected",
            )
            .expect("Failed to detect violation");

        assert_eq!(drift.drift_type, "violation");
        assert_eq!(drift.severity, "high");
        assert_eq!(detector.get_drifts().len(), 1);
    }

    #[test]
    fn test_get_drifts_by_severity() {
        let mut detector = DriftDetector::new();
        detector
            .detect_violation("decision_1", "Violation")
            .expect("Failed to detect violation");
        detector
            .detect_inconsistency("decision_2", "expected", "actual")
            .expect("Failed to detect inconsistency");

        let high_severity = detector.get_drifts_by_severity("high");
        let low_severity = detector.get_drifts_by_severity("low");

        assert_eq!(high_severity.len(), 1);
        assert_eq!(low_severity.len(), 1);
    }

    #[test]
    fn test_get_drifts_for_decision() {
        let mut detector = DriftDetector::new();
        detector
            .detect_violation("decision_1", "Violation 1")
            .expect("Failed to detect violation");
        detector
            .detect_violation("decision_1", "Violation 2")
            .expect("Failed to detect violation");
        detector
            .detect_violation("decision_2", "Violation 3")
            .expect("Failed to detect violation");

        let drifts_for_decision_1 = detector.get_drifts_for_decision("decision_1");
        assert_eq!(drifts_for_decision_1.len(), 2);
    }

    #[test]
    fn test_clear_drifts() {
        let mut detector = DriftDetector::new();
        detector
            .detect_violation("decision_1", "Violation")
            .expect("Failed to detect violation");

        assert_eq!(detector.get_drifts().len(), 1);
        detector.clear_drifts();
        assert_eq!(detector.get_drifts().len(), 0);
    }

    #[test]
    fn test_get_statistics() {
        let mut detector = DriftDetector::new();
        detector
            .detect_violation("decision_1", "Violation")
            .expect("Failed to detect violation");
        detector
            .detect_inconsistency("decision_2", "expected", "actual")
            .expect("Failed to detect inconsistency");

        let stats = detector.get_statistics();
        assert_eq!(stats.total_drifts, 2);
        assert_eq!(stats.high_severity, 1);
        assert_eq!(stats.low_severity, 1);
        assert_eq!(stats.violations, 1);
        assert_eq!(stats.inconsistencies, 1);
    }

    #[test]
    fn test_check_deviation() {
        let mut detector = DriftDetector::with_config(DriftDetectionConfig {
            confidence_threshold: 0.7,
            min_occurrences_for_pattern: 1,
            strict_mode: false,
        });

        let mut pattern = LearnedPattern::new(
            "layering".to_string(),
            "Layered architecture pattern".to_string(),
        );
        pattern.occurrences = 5;
        pattern.confidence = 0.9;

        detector
            .register_pattern(pattern)
            .expect("Failed to register pattern");

        let decision = ArchitecturalDecision::new(
            "layering".to_string(),
            "Layered architecture".to_string(),
            "Separation of concerns".to_string(),
            "0.1.0".to_string(),
        );

        let drift = detector
            .check_deviation(&decision, "layering")
            .expect("Failed to check deviation");

        // Decision confidence (0.5) is below pattern confidence (0.9) * threshold (0.7)
        assert!(drift.is_some());
    }
}
