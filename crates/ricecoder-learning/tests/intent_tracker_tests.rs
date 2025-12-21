/// Unit tests for IntentTracker component
///
/// Tests architectural pattern detection, evolution tracking, and drift detection
use ricecoder_learning::{
    ArchitecturalDecision, ArchitecturalEvolution, DriftDetection, IntentTracker,
};

#[test]
fn test_intent_tracker_creation() {
    let tracker = IntentTracker::new();
    assert_eq!(tracker.list_decisions().len(), 0);
    assert_eq!(tracker.get_evolution_history().len(), 0);
    assert_eq!(tracker.get_drift_detections().len(), 0);
}

#[test]
fn test_record_single_decision() {
    let mut tracker = IntentTracker::new();
    let decision = tracker
        .record_decision(
            "layering".to_string(),
            "Implement layered architecture".to_string(),
            "Separation of concerns".to_string(),
        )
        .expect("Failed to record decision");

    assert_eq!(decision.decision_type, "layering");
    assert_eq!(decision.description, "Implement layered architecture");
    assert_eq!(decision.rationale, "Separation of concerns");
    assert_eq!(tracker.list_decisions().len(), 1);
}

#[test]
fn test_record_multiple_decisions() {
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
            "modularity".to_string(),
            "Modular design".to_string(),
            "Reusability".to_string(),
        )
        .expect("Failed to record decision");

    tracker
        .record_decision(
            "async_pattern".to_string(),
            "Async/await pattern".to_string(),
            "Non-blocking operations".to_string(),
        )
        .expect("Failed to record decision");

    assert_eq!(tracker.list_decisions().len(), 3);
}

#[test]
fn test_get_decision_by_id() {
    let mut tracker = IntentTracker::new();
    let decision = tracker
        .record_decision(
            "layering".to_string(),
            "Layered architecture".to_string(),
            "Separation of concerns".to_string(),
        )
        .expect("Failed to record decision");

    let retrieved = tracker
        .get_decision(&decision.id)
        .expect("Failed to get decision");

    assert_eq!(retrieved.id, decision.id);
    assert_eq!(retrieved.decision_type, "layering");
}

#[test]
fn test_get_nonexistent_decision() {
    let tracker = IntentTracker::new();
    let result = tracker.get_decision("nonexistent_id");
    assert!(result.is_err());
}

#[test]
fn test_identify_patterns_single_type() {
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
    assert_eq!(patterns[0].occurrences, 2);
}

#[test]
fn test_identify_patterns_multiple_types() {
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
            "modularity".to_string(),
            "Modular design".to_string(),
            "Reusability".to_string(),
        )
        .expect("Failed to record decision");

    tracker
        .record_decision(
            "async_pattern".to_string(),
            "Async/await pattern".to_string(),
            "Non-blocking operations".to_string(),
        )
        .expect("Failed to record decision");

    let patterns = tracker
        .identify_patterns()
        .expect("Failed to identify patterns");

    assert_eq!(patterns.len(), 3);
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
    assert_eq!(evolution.description, "Added async patterns");
    assert_eq!(tracker.get_evolution_history().len(), 1);
}

#[test]
fn test_record_multiple_evolutions() {
    let mut tracker = IntentTracker::new();

    tracker
        .record_evolution(
            "0.1.0".to_string(),
            "0.2.0".to_string(),
            "Added async patterns".to_string(),
        )
        .expect("Failed to record evolution");

    tracker
        .record_evolution(
            "0.2.0".to_string(),
            "0.3.0".to_string(),
            "Refactored modularity".to_string(),
        )
        .expect("Failed to record evolution");

    let history = tracker.get_evolution_history();
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].from_version, "0.1.0");
    assert_eq!(history[1].from_version, "0.2.0");
}

#[test]
fn test_detect_drift_violation() {
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

    assert_eq!(drift.drift_type, "violation");
    assert_eq!(drift.severity, "high");
    assert_eq!(tracker.get_drift_detections().len(), 1);
}

#[test]
fn test_detect_drift_deviation() {
    let mut tracker = IntentTracker::new();
    let decision = tracker
        .record_decision(
            "modularity".to_string(),
            "Modular design".to_string(),
            "Reusability".to_string(),
        )
        .expect("Failed to record decision");

    let drift = tracker
        .detect_drift(
            &decision.id,
            "deviation".to_string(),
            "Slight deviation from pattern".to_string(),
        )
        .expect("Failed to detect drift");

    assert_eq!(drift.drift_type, "deviation");
    assert_eq!(drift.severity, "medium");
}

#[test]
fn test_detect_drift_inconsistency() {
    let mut tracker = IntentTracker::new();
    let decision = tracker
        .record_decision(
            "async_pattern".to_string(),
            "Async/await pattern".to_string(),
            "Non-blocking operations".to_string(),
        )
        .expect("Failed to record decision");

    let drift = tracker
        .detect_drift(
            &decision.id,
            "inconsistency".to_string(),
            "Inconsistent async usage".to_string(),
        )
        .expect("Failed to detect drift");

    assert_eq!(drift.drift_type, "inconsistency");
    assert_eq!(drift.severity, "low");
}

#[test]
fn test_get_drift_for_decision() {
    let mut tracker = IntentTracker::new();
    let decision = tracker
        .record_decision(
            "layering".to_string(),
            "Layered architecture".to_string(),
            "Separation of concerns".to_string(),
        )
        .expect("Failed to record decision");

    tracker
        .detect_drift(
            &decision.id,
            "violation".to_string(),
            "Violation 1".to_string(),
        )
        .expect("Failed to detect drift");

    tracker
        .detect_drift(
            &decision.id,
            "deviation".to_string(),
            "Deviation 1".to_string(),
        )
        .expect("Failed to detect drift");

    let drifts = tracker.get_drift_for_decision(&decision.id);
    assert_eq!(drifts.len(), 2);
}

#[test]
fn test_update_decision_confidence() {
    let mut tracker = IntentTracker::new();
    let decision = tracker
        .record_decision(
            "layering".to_string(),
            "Layered architecture".to_string(),
            "Separation of concerns".to_string(),
        )
        .expect("Failed to record decision");

    let initial_confidence = tracker
        .get_decision(&decision.id)
        .expect("Failed to get decision")
        .confidence;

    tracker
        .update_decision_confidence(&decision.id, 0.9)
        .expect("Failed to update confidence");

    let updated = tracker
        .get_decision(&decision.id)
        .expect("Failed to get decision");

    assert_ne!(updated.confidence, initial_confidence);
    assert_eq!(updated.confidence, 0.9);
}

#[test]
fn test_update_decision_confidence_clamping() {
    let mut tracker = IntentTracker::new();
    let decision = tracker
        .record_decision(
            "layering".to_string(),
            "Layered architecture".to_string(),
            "Separation of concerns".to_string(),
        )
        .expect("Failed to record decision");

    // Test clamping to 1.0
    tracker
        .update_decision_confidence(&decision.id, 1.5)
        .expect("Failed to update confidence");

    let updated = tracker
        .get_decision(&decision.id)
        .expect("Failed to get decision");
    assert_eq!(updated.confidence, 1.0);

    // Test clamping to 0.0
    tracker
        .update_decision_confidence(&decision.id, -0.5)
        .expect("Failed to update confidence");

    let updated = tracker
        .get_decision(&decision.id)
        .expect("Failed to get decision");
    assert_eq!(updated.confidence, 0.0);
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
fn test_increment_occurrence_multiple_times() {
    let mut tracker = IntentTracker::new();
    let decision = tracker
        .record_decision(
            "layering".to_string(),
            "Layered architecture".to_string(),
            "Separation of concerns".to_string(),
        )
        .expect("Failed to record decision");

    for _ in 0..5 {
        tracker
            .increment_occurrence(&decision.id)
            .expect("Failed to increment");
    }

    let updated = tracker
        .get_decision(&decision.id)
        .expect("Failed to get decision");

    assert_eq!(updated.occurrences, 6); // 1 initial + 5 increments
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

    tracker
        .record_decision(
            "modularity".to_string(),
            "Modular design".to_string(),
            "Reusability".to_string(),
        )
        .expect("Failed to record decision");

    let summary = tracker.get_summary();

    assert_eq!(summary.total_decisions, 2);
    assert!(summary.average_confidence > 0.0);
    assert_eq!(summary.evolution_count, 0);
    assert_eq!(summary.drift_count, 0);
}

#[test]
fn test_get_summary_with_drifts() {
    let mut tracker = IntentTracker::new();

    let decision = tracker
        .record_decision(
            "layering".to_string(),
            "Layered architecture".to_string(),
            "Separation of concerns".to_string(),
        )
        .expect("Failed to record decision");

    tracker
        .detect_drift(
            &decision.id,
            "violation".to_string(),
            "Direct layer bypass".to_string(),
        )
        .expect("Failed to detect drift");

    let summary = tracker.get_summary();

    assert_eq!(summary.total_decisions, 1);
    assert_eq!(summary.drift_count, 1);
    assert_eq!(summary.high_severity_drifts, 1);
}

#[test]
fn test_architectural_decision_creation() {
    let decision = ArchitecturalDecision::new(
        "layering".to_string(),
        "Layered architecture".to_string(),
        "Separation of concerns".to_string(),
        "0.1.0".to_string(),
    );

    assert_eq!(decision.decision_type, "layering");
    assert_eq!(decision.introduced_version, "0.1.0");
    assert_eq!(decision.confidence, 0.5);
    assert_eq!(decision.occurrences, 1);
}

#[test]
fn test_architectural_evolution_creation() {
    let evolution = ArchitecturalEvolution::new("0.1.0".to_string(), "0.2.0".to_string());

    assert_eq!(evolution.from_version, "0.1.0");
    assert_eq!(evolution.to_version, "0.2.0");
    assert!(evolution.added_decisions.is_empty());
    assert!(evolution.removed_decisions.is_empty());
}

#[test]
fn test_drift_detection_creation() {
    let drift = DriftDetection::new(
        "decision_1".to_string(),
        "violation".to_string(),
        "high".to_string(),
        "Direct layer bypass detected".to_string(),
    );

    assert_eq!(drift.decision_id, "decision_1");
    assert_eq!(drift.drift_type, "violation");
    assert_eq!(drift.severity, "high");
}

#[test]
fn test_complex_workflow() {
    let mut tracker = IntentTracker::new();

    // Record initial decisions
    let layering_decision = tracker
        .record_decision(
            "layering".to_string(),
            "Implement layered architecture".to_string(),
            "Separation of concerns".to_string(),
        )
        .expect("Failed to record decision");

    let modularity_decision = tracker
        .record_decision(
            "modularity".to_string(),
            "Implement modular design".to_string(),
            "Reusability and maintainability".to_string(),
        )
        .expect("Failed to record decision");

    // Record evolution
    tracker
        .record_evolution(
            "0.1.0".to_string(),
            "0.2.0".to_string(),
            "Added async patterns".to_string(),
        )
        .expect("Failed to record evolution");

    // Detect drifts
    tracker
        .detect_drift(
            &layering_decision.id,
            "violation".to_string(),
            "Direct layer bypass detected".to_string(),
        )
        .expect("Failed to detect drift");

    tracker
        .detect_drift(
            &modularity_decision.id,
            "deviation".to_string(),
            "Slight deviation from pattern".to_string(),
        )
        .expect("Failed to detect drift");

    // Update confidence
    tracker
        .update_decision_confidence(&layering_decision.id, 0.8)
        .expect("Failed to update confidence");

    // Increment occurrences
    tracker
        .increment_occurrence(&modularity_decision.id)
        .expect("Failed to increment");

    // Get summary
    let summary = tracker.get_summary();

    assert_eq!(summary.total_decisions, 2);
    assert_eq!(summary.evolution_count, 1);
    assert_eq!(summary.drift_count, 2);
    assert_eq!(summary.high_severity_drifts, 1);
}

#[test]
fn test_identify_patterns_with_confidence() {
    let mut tracker = IntentTracker::new();

    // Record multiple decisions of the same type
    for i in 0..5 {
        tracker
            .record_decision(
                "layering".to_string(),
                format!("Layered architecture {}", i),
                "Separation of concerns".to_string(),
            )
            .expect("Failed to record decision");
    }

    let patterns = tracker
        .identify_patterns()
        .expect("Failed to identify patterns");

    assert_eq!(patterns.len(), 1);
    assert_eq!(patterns[0].pattern_type, "architectural_layering");
    assert_eq!(patterns[0].occurrences, 5);
    // Confidence should be calculated based on occurrences
    assert!(patterns[0].confidence > 0.0);
}

#[test]
fn test_drift_remediation_suggestions() {
    let mut tracker = IntentTracker::new();
    let decision = tracker
        .record_decision(
            "layering".to_string(),
            "Layered architecture".to_string(),
            "Separation of concerns".to_string(),
        )
        .expect("Failed to record decision");

    let violation_drift = tracker
        .detect_drift(
            &decision.id,
            "violation".to_string(),
            "Direct layer bypass detected".to_string(),
        )
        .expect("Failed to detect drift");

    assert!(!violation_drift.remediation.is_empty());
    assert!(violation_drift
        .remediation
        .to_lowercase()
        .contains("immediately"));
}
