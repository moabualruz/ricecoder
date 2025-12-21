/// Architectural intent tracking component
///
/// Tracks architectural decisions and patterns over time, detecting evolution
/// and drift from established architectural patterns.
use crate::error::{LearningError, Result};
use crate::models::{LearnedPattern, PatternExample};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Represents an architectural decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalDecision {
    /// Unique identifier for the decision
    pub id: String,
    /// Type of architectural decision (e.g., "layering", "modularity", "async_pattern")
    pub decision_type: String,
    /// Description of the decision
    pub description: String,
    /// Rationale for the decision
    pub rationale: String,
    /// When the decision was made
    pub created_at: DateTime<Utc>,
    /// When the decision was last observed
    pub last_observed: DateTime<Utc>,
    /// Version when this decision was introduced
    pub introduced_version: String,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
    /// Number of times this pattern has been observed
    pub occurrences: usize,
    /// Additional metadata
    pub metadata: serde_json::Value,
}

impl ArchitecturalDecision {
    /// Create a new architectural decision
    pub fn new(
        decision_type: String,
        description: String,
        rationale: String,
        version: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            decision_type,
            description,
            rationale,
            created_at: Utc::now(),
            last_observed: Utc::now(),
            introduced_version: version,
            confidence: 0.5,
            occurrences: 1,
            metadata: serde_json::json!({}),
        }
    }
}

/// Represents architectural evolution between versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalEvolution {
    /// Unique identifier
    pub id: String,
    /// From version
    pub from_version: String,
    /// To version
    pub to_version: String,
    /// Decisions that were added
    pub added_decisions: Vec<String>,
    /// Decisions that were removed
    pub removed_decisions: Vec<String>,
    /// Decisions that were modified
    pub modified_decisions: Vec<String>,
    /// When this evolution was recorded
    pub recorded_at: DateTime<Utc>,
    /// Description of the evolution
    pub description: String,
}

impl ArchitecturalEvolution {
    /// Create a new architectural evolution record
    pub fn new(from_version: String, to_version: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            from_version,
            to_version,
            added_decisions: Vec::new(),
            removed_decisions: Vec::new(),
            modified_decisions: Vec::new(),
            recorded_at: Utc::now(),
            description: String::new(),
        }
    }
}

/// Represents architectural drift detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftDetection {
    /// Unique identifier
    pub id: String,
    /// Decision that drifted
    pub decision_id: String,
    /// Type of drift
    pub drift_type: String,
    /// Severity level (low, medium, high)
    pub severity: String,
    /// Description of the drift
    pub description: String,
    /// When the drift was detected
    pub detected_at: DateTime<Utc>,
    /// Suggested remediation
    pub remediation: String,
}

impl DriftDetection {
    /// Create a new drift detection
    pub fn new(
        decision_id: String,
        drift_type: String,
        severity: String,
        description: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            decision_id,
            drift_type,
            severity,
            description,
            detected_at: Utc::now(),
            remediation: String::new(),
        }
    }
}

/// Tracks architectural intent and evolution
pub struct IntentTracker {
    /// Stored architectural decisions
    decisions: HashMap<String, ArchitecturalDecision>,
    /// Evolution history
    evolution_history: Vec<ArchitecturalEvolution>,
    /// Drift detections
    drift_detections: Vec<DriftDetection>,
    /// Current version
    current_version: String,
}

impl IntentTracker {
    /// Create a new intent tracker
    pub fn new() -> Self {
        Self {
            decisions: HashMap::new(),
            evolution_history: Vec::new(),
            drift_detections: Vec::new(),
            current_version: "0.1.0".to_string(),
        }
    }

    /// Record an architectural decision
    pub fn record_decision(
        &mut self,
        decision_type: String,
        description: String,
        rationale: String,
    ) -> Result<ArchitecturalDecision> {
        let decision = ArchitecturalDecision::new(
            decision_type,
            description,
            rationale,
            self.current_version.clone(),
        );

        self.decisions.insert(decision.id.clone(), decision.clone());
        Ok(decision)
    }

    /// Get a decision by ID
    pub fn get_decision(&self, decision_id: &str) -> Result<ArchitecturalDecision> {
        self.decisions
            .get(decision_id)
            .cloned()
            .ok_or_else(|| LearningError::PatternNotFound(decision_id.to_string()))
    }

    /// List all architectural decisions
    pub fn list_decisions(&self) -> Vec<ArchitecturalDecision> {
        self.decisions.values().cloned().collect()
    }

    /// Identify architectural patterns from decisions
    pub fn identify_patterns(&self) -> Result<Vec<LearnedPattern>> {
        let mut patterns = Vec::new();

        // Group decisions by type
        let mut decisions_by_type: HashMap<String, Vec<&ArchitecturalDecision>> = HashMap::new();
        for decision in self.decisions.values() {
            decisions_by_type
                .entry(decision.decision_type.clone())
                .or_default()
                .push(decision);
        }

        // Create patterns for each decision type
        for (decision_type, decisions) in decisions_by_type {
            if !decisions.is_empty() {
                let mut pattern = LearnedPattern::new(
                    format!("architectural_{}", decision_type),
                    format!("Architectural pattern: {}", decision_type),
                );

                // Add examples from decisions
                for decision in &decisions {
                    let example = PatternExample {
                        input: serde_json::json!({
                            "decision_type": &decision.decision_type,
                            "description": &decision.description,
                        }),
                        output: serde_json::json!({
                            "rationale": &decision.rationale,
                            "version": &decision.introduced_version,
                        }),
                        context: serde_json::json!({
                            "occurrences": decision.occurrences,
                            "confidence": decision.confidence,
                        }),
                    };
                    pattern.examples.push(example);
                }

                // Calculate confidence based on occurrences
                let avg_occurrences = decisions.iter().map(|d| d.occurrences).sum::<usize>() as f32
                    / decisions.len() as f32;
                pattern.confidence = (avg_occurrences / 10.0).min(1.0);
                pattern.occurrences = decisions.len();

                patterns.push(pattern);
            }
        }

        Ok(patterns)
    }

    /// Recognize architectural evolution between versions
    pub fn record_evolution(
        &mut self,
        from_version: String,
        to_version: String,
        description: String,
    ) -> Result<ArchitecturalEvolution> {
        let mut evolution = ArchitecturalEvolution::new(from_version, to_version);
        evolution.description = description;

        self.evolution_history.push(evolution.clone());
        self.current_version = evolution.to_version.clone();

        Ok(evolution)
    }

    /// Get evolution history
    pub fn get_evolution_history(&self) -> Vec<ArchitecturalEvolution> {
        self.evolution_history.clone()
    }

    /// Detect architectural drift
    pub fn detect_drift(
        &mut self,
        decision_id: &str,
        drift_type: String,
        description: String,
    ) -> Result<DriftDetection> {
        // Verify the decision exists
        let _decision = self.get_decision(decision_id)?;

        // Determine severity based on drift type
        let severity = match drift_type.as_str() {
            "violation" => "high".to_string(),
            "deviation" => "medium".to_string(),
            "inconsistency" => "low".to_string(),
            _ => "medium".to_string(),
        };

        let mut drift =
            DriftDetection::new(decision_id.to_string(), drift_type, severity, description);

        // Suggest remediation based on drift type
        drift.remediation = match drift.drift_type.as_str() {
            "violation" => {
                "Immediately address the violation to restore architectural integrity".to_string()
            }
            "deviation" => "Review and align with established architectural patterns".to_string(),
            "inconsistency" => {
                "Standardize implementation to match architectural intent".to_string()
            }
            _ => "Review and address the drift".to_string(),
        };

        self.drift_detections.push(drift.clone());
        Ok(drift)
    }

    /// Get all drift detections
    pub fn get_drift_detections(&self) -> Vec<DriftDetection> {
        self.drift_detections.clone()
    }

    /// Get drift detections for a specific decision
    pub fn get_drift_for_decision(&self, decision_id: &str) -> Vec<DriftDetection> {
        self.drift_detections
            .iter()
            .filter(|d| d.decision_id == decision_id)
            .cloned()
            .collect()
    }

    /// Update decision confidence based on observations
    pub fn update_decision_confidence(&mut self, decision_id: &str, confidence: f32) -> Result<()> {
        if let Some(decision) = self.decisions.get_mut(decision_id) {
            decision.confidence = confidence.clamp(0.0, 1.0);
            decision.last_observed = Utc::now();
            Ok(())
        } else {
            Err(LearningError::PatternNotFound(decision_id.to_string()))
        }
    }

    /// Increment occurrence count for a decision
    pub fn increment_occurrence(&mut self, decision_id: &str) -> Result<()> {
        if let Some(decision) = self.decisions.get_mut(decision_id) {
            decision.occurrences += 1;
            decision.last_observed = Utc::now();
            Ok(())
        } else {
            Err(LearningError::PatternNotFound(decision_id.to_string()))
        }
    }

    /// Get summary of architectural state
    pub fn get_summary(&self) -> ArchitecturalSummary {
        let total_decisions = self.decisions.len();
        let avg_confidence = if total_decisions > 0 {
            self.decisions.values().map(|d| d.confidence).sum::<f32>() / total_decisions as f32
        } else {
            0.0
        };

        let high_drift_count = self
            .drift_detections
            .iter()
            .filter(|d| d.severity == "high")
            .count();

        ArchitecturalSummary {
            total_decisions,
            average_confidence: avg_confidence,
            evolution_count: self.evolution_history.len(),
            drift_count: self.drift_detections.len(),
            high_severity_drifts: high_drift_count,
            current_version: self.current_version.clone(),
        }
    }
}

impl Default for IntentTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of architectural state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalSummary {
    /// Total number of architectural decisions
    pub total_decisions: usize,
    /// Average confidence of decisions
    pub average_confidence: f32,
    /// Number of evolution records
    pub evolution_count: usize,
    /// Total number of drift detections
    pub drift_count: usize,
    /// Number of high-severity drifts
    pub high_severity_drifts: usize,
    /// Current version
    pub current_version: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_tracker_creation() {
        let tracker = IntentTracker::new();
        assert_eq!(tracker.list_decisions().len(), 0);
        assert_eq!(tracker.get_evolution_history().len(), 0);
        assert_eq!(tracker.get_drift_detections().len(), 0);
    }

    #[test]
    fn test_record_decision() {
        let mut tracker = IntentTracker::new();
        let decision = tracker
            .record_decision(
                "layering".to_string(),
                "Implement layered architecture".to_string(),
                "Separation of concerns".to_string(),
            )
            .expect("Failed to record decision");

        assert_eq!(decision.decision_type, "layering");
        assert_eq!(tracker.list_decisions().len(), 1);
    }

    #[test]
    fn test_get_decision() {
        let mut tracker = IntentTracker::new();
        let decision = tracker
            .record_decision(
                "modularity".to_string(),
                "Modular design".to_string(),
                "Reusability".to_string(),
            )
            .expect("Failed to record decision");

        let retrieved = tracker
            .get_decision(&decision.id)
            .expect("Failed to get decision");
        assert_eq!(retrieved.id, decision.id);
        assert_eq!(retrieved.decision_type, "modularity");
    }

    #[test]
    fn test_identify_patterns() {
        let mut tracker = IntentTracker::new();
        tracker
            .record_decision(
                "layering".to_string(),
                "Layered architecture".to_string(),
                "Separation of concerns".to_string(),
            )
            .expect("Failed to record decision");

        tracker
            .record_decision(
                "layering".to_string(),
                "Another layered pattern".to_string(),
                "Consistency".to_string(),
            )
            .expect("Failed to record decision");

        let patterns = tracker
            .identify_patterns()
            .expect("Failed to identify patterns");
        assert!(!patterns.is_empty());
        assert_eq!(patterns[0].pattern_type, "architectural_layering");
    }

    #[test]
    fn test_record_evolution() {
        let mut tracker = IntentTracker::new();
        let evolution = tracker
            .record_evolution(
                "0.1.0".to_string(),
                "0.2.0".to_string(),
                "Added async patterns".to_string(),
            )
            .expect("Failed to record evolution");

        assert_eq!(evolution.from_version, "0.1.0");
        assert_eq!(evolution.to_version, "0.2.0");
        assert_eq!(tracker.get_evolution_history().len(), 1);
    }

    #[test]
    fn test_detect_drift() {
        let mut tracker = IntentTracker::new();
        let decision = tracker
            .record_decision(
                "layering".to_string(),
                "Layered architecture".to_string(),
                "Separation of concerns".to_string(),
            )
            .expect("Failed to record decision");

        let drift = tracker
            .detect_drift(
                &decision.id,
                "violation".to_string(),
                "Direct layer bypass detected".to_string(),
            )
            .expect("Failed to detect drift");

        assert_eq!(drift.severity, "high");
        assert_eq!(tracker.get_drift_detections().len(), 1);
    }

    #[test]
    fn test_update_decision_confidence() {
        let mut tracker = IntentTracker::new();
        let decision = tracker
            .record_decision(
                "modularity".to_string(),
                "Modular design".to_string(),
                "Reusability".to_string(),
            )
            .expect("Failed to record decision");

        tracker
            .update_decision_confidence(&decision.id, 0.9)
            .expect("Failed to update confidence");

        let updated = tracker
            .get_decision(&decision.id)
            .expect("Failed to get decision");
        assert_eq!(updated.confidence, 0.9);
    }

    #[test]
    fn test_increment_occurrence() {
        let mut tracker = IntentTracker::new();
        let decision = tracker
            .record_decision(
                "async_pattern".to_string(),
                "Async/await pattern".to_string(),
                "Non-blocking operations".to_string(),
            )
            .expect("Failed to record decision");

        let initial_occurrences = tracker
            .get_decision(&decision.id)
            .expect("Failed to get decision")
            .occurrences;

        tracker
            .increment_occurrence(&decision.id)
            .expect("Failed to increment");

        let updated = tracker
            .get_decision(&decision.id)
            .expect("Failed to get decision");
        assert_eq!(updated.occurrences, initial_occurrences + 1);
    }

    #[test]
    fn test_get_summary() {
        let mut tracker = IntentTracker::new();
        tracker
            .record_decision(
                "layering".to_string(),
                "Layered architecture".to_string(),
                "Separation of concerns".to_string(),
            )
            .expect("Failed to record decision");

        let summary = tracker.get_summary();
        assert_eq!(summary.total_decisions, 1);
        assert!(summary.average_confidence > 0.0);
    }

    #[test]
    fn test_drift_severity_levels() {
        let mut tracker = IntentTracker::new();
        let decision = tracker
            .record_decision(
                "layering".to_string(),
                "Layered architecture".to_string(),
                "Separation of concerns".to_string(),
            )
            .expect("Failed to record decision");

        let violation = tracker
            .detect_drift(
                &decision.id,
                "violation".to_string(),
                "Direct layer bypass".to_string(),
            )
            .expect("Failed to detect drift");
        assert_eq!(violation.severity, "high");

        let deviation = tracker
            .detect_drift(
                &decision.id,
                "deviation".to_string(),
                "Slight deviation".to_string(),
            )
            .expect("Failed to detect drift");
        assert_eq!(deviation.severity, "medium");

        let inconsistency = tracker
            .detect_drift(
                &decision.id,
                "inconsistency".to_string(),
                "Minor inconsistency".to_string(),
            )
            .expect("Failed to detect drift");
        assert_eq!(inconsistency.severity, "low");
    }
}
