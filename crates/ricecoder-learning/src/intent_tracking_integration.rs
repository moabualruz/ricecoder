/// Integration of IntentTracker with LearningManager
///
/// Provides methods for tracking architectural intent and detecting drift
/// as part of the learning system.

use crate::drift_detector::{DriftDetectionConfig, DriftDetector};
use crate::error::Result;
use crate::intent_tracker::{
    ArchitecturalDecision, ArchitecturalEvolution, ArchitecturalSummary, DriftDetection,
    IntentTracker,
};
use crate::models::LearnedPattern;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Intent tracking integration for the learning manager
pub struct IntentTrackingIntegration {
    /// Intent tracker for architectural decisions
    intent_tracker: Arc<RwLock<IntentTracker>>,
    /// Drift detector for identifying deviations
    drift_detector: Arc<RwLock<DriftDetector>>,
}

impl IntentTrackingIntegration {
    /// Create a new intent tracking integration
    pub fn new() -> Self {
        Self {
            intent_tracker: Arc::new(RwLock::new(IntentTracker::new())),
            drift_detector: Arc::new(RwLock::new(DriftDetector::new())),
        }
    }

    /// Create with custom drift detection configuration
    pub fn with_drift_config(config: DriftDetectionConfig) -> Self {
        Self {
            intent_tracker: Arc::new(RwLock::new(IntentTracker::new())),
            drift_detector: Arc::new(RwLock::new(DriftDetector::with_config(config))),
        }
    }

    /// Record an architectural decision
    pub async fn record_decision(
        &self,
        decision_type: String,
        description: String,
        rationale: String,
    ) -> Result<ArchitecturalDecision> {
        let mut tracker = self.intent_tracker.write().await;
        tracker.record_decision(decision_type, description, rationale)
    }

    /// Get a decision by ID
    pub async fn get_decision(&self, decision_id: &str) -> Result<ArchitecturalDecision> {
        let tracker = self.intent_tracker.read().await;
        tracker.get_decision(decision_id)
    }

    /// List all architectural decisions
    pub async fn list_decisions(&self) -> Vec<ArchitecturalDecision> {
        let tracker = self.intent_tracker.read().await;
        tracker.list_decisions()
    }

    /// Identify architectural patterns from decisions
    pub async fn identify_patterns(&self) -> Result<Vec<LearnedPattern>> {
        let tracker = self.intent_tracker.read().await;
        tracker.identify_patterns()
    }

    /// Record architectural evolution between versions
    pub async fn record_evolution(
        &self,
        from_version: String,
        to_version: String,
        description: String,
    ) -> Result<ArchitecturalEvolution> {
        let mut tracker = self.intent_tracker.write().await;
        tracker.record_evolution(from_version, to_version, description)
    }

    /// Get evolution history
    pub async fn get_evolution_history(&self) -> Vec<ArchitecturalEvolution> {
        let tracker = self.intent_tracker.read().await;
        tracker.get_evolution_history()
    }

    /// Detect architectural drift
    pub async fn detect_drift(
        &self,
        decision_id: &str,
        drift_type: String,
        description: String,
    ) -> Result<DriftDetection> {
        let mut tracker = self.intent_tracker.write().await;
        tracker.detect_drift(decision_id, drift_type, description)
    }

    /// Get all drift detections
    pub async fn get_drift_detections(&self) -> Vec<DriftDetection> {
        let tracker = self.intent_tracker.read().await;
        tracker.get_drift_detections()
    }

    /// Get drift detections for a specific decision
    pub async fn get_drift_for_decision(&self, decision_id: &str) -> Vec<DriftDetection> {
        let tracker = self.intent_tracker.read().await;
        tracker.get_drift_for_decision(decision_id)
    }

    /// Update decision confidence based on observations
    pub async fn update_decision_confidence(
        &self,
        decision_id: &str,
        confidence: f32,
    ) -> Result<()> {
        let mut tracker = self.intent_tracker.write().await;
        tracker.update_decision_confidence(decision_id, confidence)
    }

    /// Increment occurrence count for a decision
    pub async fn increment_occurrence(&self, decision_id: &str) -> Result<()> {
        let mut tracker = self.intent_tracker.write().await;
        tracker.increment_occurrence(decision_id)
    }

    /// Get summary of architectural state
    pub async fn get_summary(&self) -> ArchitecturalSummary {
        let tracker = self.intent_tracker.read().await;
        tracker.get_summary()
    }

    /// Register an established pattern for drift detection
    pub async fn register_pattern_for_drift_detection(
        &self,
        pattern: LearnedPattern,
    ) -> Result<()> {
        let mut detector = self.drift_detector.write().await;
        detector.register_pattern(pattern)
    }

    /// Check if a decision deviates from established patterns
    pub async fn check_deviation(
        &self,
        decision: &ArchitecturalDecision,
        pattern_type: &str,
    ) -> Result<Option<DriftDetection>> {
        let mut detector = self.drift_detector.write().await;
        detector.check_deviation(decision, pattern_type)
    }

    /// Detect inconsistencies in decision application
    pub async fn detect_inconsistency(
        &self,
        decision_id: &str,
        expected_behavior: &str,
        actual_behavior: &str,
    ) -> Result<DriftDetection> {
        let mut detector = self.drift_detector.write().await;
        detector.detect_inconsistency(decision_id, expected_behavior, actual_behavior)
    }

    /// Detect pattern violations
    pub async fn detect_violation(
        &self,
        decision_id: &str,
        violation_description: &str,
    ) -> Result<DriftDetection> {
        let mut detector = self.drift_detector.write().await;
        detector.detect_violation(decision_id, violation_description)
    }

    /// Get all detected drifts from the detector
    pub async fn get_detector_drifts(&self) -> Vec<DriftDetection> {
        let detector = self.drift_detector.read().await;
        detector.get_drifts()
    }

    /// Get drifts by severity
    pub async fn get_drifts_by_severity(&self, severity: &str) -> Vec<DriftDetection> {
        let detector = self.drift_detector.read().await;
        detector.get_drifts_by_severity(severity)
    }

    /// Get drifts for a specific decision from the detector
    pub async fn get_detector_drifts_for_decision(
        &self,
        decision_id: &str,
    ) -> Vec<DriftDetection> {
        let detector = self.drift_detector.read().await;
        detector.get_drifts_for_decision(decision_id)
    }

    /// Clear all detected drifts
    pub async fn clear_detector_drifts(&self) {
        let mut detector = self.drift_detector.write().await;
        detector.clear_drifts();
    }

    /// Get drift statistics
    pub async fn get_drift_statistics(&self) -> crate::drift_detector::DriftStatistics {
        let detector = self.drift_detector.read().await;
        detector.get_statistics()
    }

    /// Perform comprehensive drift detection on all decisions
    pub async fn perform_comprehensive_drift_detection(&self) -> Result<Vec<DriftDetection>> {
        let tracker = self.intent_tracker.read().await;
        let decisions = tracker.list_decisions();

        let mut all_drifts = Vec::new();

        for decision in decisions {
            // Check for deviations from patterns
            let patterns = tracker.identify_patterns()?;
            for pattern in patterns {
                let mut detector = self.drift_detector.write().await;
                if let Ok(Some(drift)) = detector.check_deviation(&decision, &pattern.pattern_type) {
                    all_drifts.push(drift);
                }
            }
        }

        Ok(all_drifts)
    }

    /// Get a comprehensive report of architectural state
    pub async fn get_comprehensive_report(&self) -> ArchitecturalReport {
        let summary = self.get_summary().await;
        let decisions = self.list_decisions().await;
        let evolution = self.get_evolution_history().await;
        let drifts = self.get_drift_detections().await;
        let detector_drifts = self.get_detector_drifts().await;
        let drift_stats = self.get_drift_statistics().await;

        ArchitecturalReport {
            summary,
            total_decisions: decisions.len(),
            total_evolution_records: evolution.len(),
            total_drifts: drifts.len() + detector_drifts.len(),
            drift_statistics: drift_stats,
        }
    }
}

impl Default for IntentTrackingIntegration {
    fn default() -> Self {
        Self::new()
    }
}

/// Comprehensive report of architectural state
#[derive(Debug, Clone)]
pub struct ArchitecturalReport {
    /// Summary of architectural state
    pub summary: ArchitecturalSummary,
    /// Total number of decisions
    pub total_decisions: usize,
    /// Total number of evolution records
    pub total_evolution_records: usize,
    /// Total number of drifts detected
    pub total_drifts: usize,
    /// Drift statistics
    pub drift_statistics: crate::drift_detector::DriftStatistics,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_intent_tracking_integration_creation() {
        let integration = IntentTrackingIntegration::new();
        let summary = integration.get_summary().await;
        assert_eq!(summary.total_decisions, 0);
    }

    #[tokio::test]
    async fn test_record_decision() {
        let integration = IntentTrackingIntegration::new();
        let decision = integration
            .record_decision(
                "layering".to_string(),
                "Implement layered architecture".to_string(),
                "Separation of concerns".to_string(),
            )
            .await
            .expect("Failed to record decision");

        assert_eq!(decision.decision_type, "layering");
        let decisions = integration.list_decisions().await;
        assert_eq!(decisions.len(), 1);
    }

    #[tokio::test]
    async fn test_record_evolution() {
        let integration = IntentTrackingIntegration::new();
        let evolution = integration
            .record_evolution(
                "0.1.0".to_string(),
                "0.2.0".to_string(),
                "Added async patterns".to_string(),
            )
            .await
            .expect("Failed to record evolution");

        assert_eq!(evolution.from_version, "0.1.0");
        let history = integration.get_evolution_history().await;
        assert_eq!(history.len(), 1);
    }

    #[tokio::test]
    async fn test_detect_drift() {
        let integration = IntentTrackingIntegration::new();
        let decision = integration
            .record_decision(
                "layering".to_string(),
                "Layered architecture".to_string(),
                "Separation of concerns".to_string(),
            )
            .await
            .expect("Failed to record decision");

        let drift = integration
            .detect_drift(
                &decision.id,
                "violation".to_string(),
                "Direct layer bypass detected".to_string(),
            )
            .await
            .expect("Failed to detect drift");

        assert_eq!(drift.severity, "high");
        let drifts = integration.get_drift_detections().await;
        assert_eq!(drifts.len(), 1);
    }

    #[tokio::test]
    async fn test_get_comprehensive_report() {
        let integration = IntentTrackingIntegration::new();
        integration
            .record_decision(
                "layering".to_string(),
                "Layered architecture".to_string(),
                "Separation of concerns".to_string(),
            )
            .await
            .expect("Failed to record decision");

        let report = integration.get_comprehensive_report().await;
        assert_eq!(report.total_decisions, 1);
    }
}
