//! Tests for ricecoder-workflows safety constraints
//!
//! Tests for SafetyConstraints enforcement and validation.

use ricecoder_workflows::models::{CommandStep, ErrorAction, RiskFactors, StepConfig, StepType, WorkflowStep};
use ricecoder_workflows::safety_constraints::SafetyConstraints;

fn create_command_step(id: &str, timeout: u64) -> WorkflowStep {
    WorkflowStep {
        id: id.to_string(),
        name: format!("Step {}", id),
        step_type: StepType::Command(CommandStep {
            command: "test".to_string(),
            args: vec![],
            timeout,
        }),
        config: StepConfig {
            config: serde_json::json!({}),
        },
        dependencies: Vec::new(),
        approval_required: false,
        on_error: ErrorAction::Rollback,
        risk_score: None,
        risk_factors: RiskFactors::default(),
    }
}

#[test]
fn test_safety_constraints_default() {
    let constraints = SafetyConstraints::new();
    assert_eq!(constraints.max_timeout_ms, 300_000);
    assert_eq!(constraints.max_memory_mb, 1024);
    assert_eq!(constraints.max_cpu_percent, 80);
}

#[test]
fn test_enforce_timeout_within_limit() {
    let constraints = SafetyConstraints::new();
    let step = create_command_step("1", 100_000);
    let timeout = constraints.enforce_timeout(&step);
    assert_eq!(timeout.as_millis(), 100_000);
}

#[test]
fn test_enforce_timeout_exceeds_limit() {
    let constraints = SafetyConstraints::new();
    let step = create_command_step("1", 500_000);
    let timeout = constraints.enforce_timeout(&step);
    assert_eq!(timeout.as_millis(), 300_000);
}

#[test]
fn test_timeout_violation_detection() {
    let constraints = SafetyConstraints::new();
    let step = create_command_step("1", 500_000);
    let violations = constraints.apply_to_step(&step, 80);
    assert!(!violations.is_empty());
    assert_eq!(violations[0].violation_type, "timeout_exceeded");
}

#[test]
fn test_rollback_capability() {
    let constraints = SafetyConstraints::new();
    let step = create_command_step("1", 100_000);
    assert!(constraints.has_rollback_capability(&step));
}