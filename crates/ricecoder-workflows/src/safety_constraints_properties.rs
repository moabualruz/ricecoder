//! Property-based tests for safety constraints
//! **Feature: ricecoder-workflows, Property 16: Safety Constraints Application**

use proptest::prelude::*;
use crate::models::{WorkflowStep, StepType, StepConfig, ErrorAction, CommandStep, RiskFactors};
use crate::safety_constraints::SafetyConstraints;

// Strategy for generating command steps
fn command_step_strategy() -> impl Strategy<Value = WorkflowStep> {
    (100_000u64..=600_000).prop_map(|timeout| WorkflowStep {
        id: "step".to_string(),
        name: "Test Step".to_string(),
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
    })
}

// Property 16: Safety Constraints Application
// **Validates: Requirements 3.4**
// *For any* approved high-risk operation, the system SHALL enforce execution timeouts,
// resource limits, and maintain rollback capability.
proptest! {
    #[test]
    fn prop_timeout_enforcement(step in command_step_strategy()) {
        let constraints = SafetyConstraints::new();
        let enforced_timeout = constraints.enforce_timeout(&step);
        let enforced_ms = enforced_timeout.as_millis() as u64;

        // Enforced timeout should not exceed maximum
        prop_assert!(
            enforced_ms <= constraints.max_timeout_ms,
            "Enforced timeout {} ms exceeds maximum {} ms",
            enforced_ms,
            constraints.max_timeout_ms
        );

        // If original timeout is within limit, it should be preserved
        if let StepType::Command(cmd) = &step.step_type {
            if cmd.timeout <= constraints.max_timeout_ms {
                prop_assert_eq!(
                    enforced_ms,
                    cmd.timeout,
                    "Timeout within limit should be preserved"
                );
            }
        }
    }

    #[test]
    fn prop_rollback_capability_maintained(step in command_step_strategy()) {
        let constraints = SafetyConstraints::new();
        let has_rollback = constraints.has_rollback_capability(&step);

        // Step is created with Rollback error action, so should have capability
        prop_assert!(has_rollback, "Rollback capability should be maintained");
    }

    #[test]
    fn prop_safety_violations_detected(timeout_ms in 300_001u64..=600_000) {
        let constraints = SafetyConstraints::new();
        let step = WorkflowStep {
            id: "step".to_string(),
            name: "Test Step".to_string(),
            step_type: StepType::Command(CommandStep {
                command: "test".to_string(),
                args: vec![],
                timeout: timeout_ms,
            }),
            config: StepConfig {
                config: serde_json::json!({}),
            },
            dependencies: Vec::new(),
            approval_required: false,
            on_error: ErrorAction::Rollback,
            risk_score: None,
            risk_factors: RiskFactors::default(),
        };

        let violations = constraints.apply_to_step(&step, 80);

        // Should detect timeout violation
        prop_assert!(
            !violations.is_empty(),
            "Should detect timeout violation for timeout {} ms > max {} ms",
            timeout_ms,
            constraints.max_timeout_ms
        );

        // Violation should be timeout_exceeded
        prop_assert_eq!(
            &violations[0].violation_type,
            "timeout_exceeded",
            "Violation type should be timeout_exceeded"
        );
    }
}
