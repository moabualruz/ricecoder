//! Property-based tests for risk scoring
//! **Feature: ricecoder-workflows, Property 13-17: Risk Scoring and Assessment**

use proptest::prelude::*;
use crate::models::{WorkflowStep, StepType, StepConfig, ErrorAction, AgentStep, RiskFactors, WorkflowState, WorkflowStatus, StepResult, StepStatus};
use crate::risk_scoring::RiskScorer;
use crate::safety_constraints::SafetyConstraints;
use chrono::Utc;
use std::collections::HashMap;

// Strategy for generating risk factors
fn risk_factors_strategy() -> impl Strategy<Value = RiskFactors> {
    (0u8..=100, 0u8..=100, 0u8..=100).prop_map(|(impact, reversibility, complexity)| {
        RiskFactors {
            impact,
            reversibility,
            complexity,
        }
    })
}

// Strategy for generating workflow steps
fn workflow_step_strategy() -> impl Strategy<Value = WorkflowStep> {
    (
        "step_[a-z0-9]{1,10}",
        "Step [A-Z][a-z]{2,10}",
        risk_factors_strategy(),
    )
        .prop_map(|(id, name, risk_factors)| WorkflowStep {
            id,
            name,
            step_type: StepType::Agent(AgentStep {
                agent_id: "test-agent".to_string(),
                task: "test-task".to_string(),
            }),
            config: StepConfig {
                config: serde_json::json!({}),
            },
            dependencies: Vec::new(),
            approval_required: false,
            on_error: ErrorAction::Fail,
            risk_score: None,
            risk_factors,
        })
}

// Property 13: Risk Score Calculation
// **Validates: Requirements 3.1**
// *For any* workflow step, the calculated risk score SHALL be between 0 and 100 inclusive.
proptest! {
    #[test]
    fn prop_risk_score_in_valid_range(step in workflow_step_strategy()) {
        let scorer = RiskScorer::new();
        let score = scorer.calculate_risk_score(&step);
        prop_assert!(score <= 100, "Risk score {} exceeds maximum of 100", score);
    }
}

// Property 14: Risk Factor Consideration
// **Validates: Requirements 3.2**
// *For any* two workflow steps where one has higher impact, reversibility, or complexity than the other,
// the risk score SHALL reflect this difference.
proptest! {
    #[test]
    fn prop_risk_factors_affect_score(
        impact1 in 0u8..=100,
        reversibility1 in 0u8..=100,
        complexity1 in 0u8..=100,
        impact2 in 0u8..=100,
        reversibility2 in 0u8..=100,
        complexity2 in 0u8..=100,
    ) {
        let scorer = RiskScorer::new();

        let step1 = WorkflowStep {
            id: "step1".to_string(),
            name: "Step 1".to_string(),
            step_type: StepType::Agent(AgentStep {
                agent_id: "agent".to_string(),
                task: "task".to_string(),
            }),
            config: StepConfig {
                config: serde_json::json!({}),
            },
            dependencies: Vec::new(),
            approval_required: false,
            on_error: ErrorAction::Fail,
            risk_score: None,
            risk_factors: RiskFactors {
                impact: impact1,
                reversibility: reversibility1,
                complexity: complexity1,
            },
        };

        let step2 = WorkflowStep {
            id: "step2".to_string(),
            name: "Step 2".to_string(),
            step_type: StepType::Agent(AgentStep {
                agent_id: "agent".to_string(),
                task: "task".to_string(),
            }),
            config: StepConfig {
                config: serde_json::json!({}),
            },
            dependencies: Vec::new(),
            approval_required: false,
            on_error: ErrorAction::Fail,
            risk_score: None,
            risk_factors: RiskFactors {
                impact: impact2,
                reversibility: reversibility2,
                complexity: complexity2,
            },
        };

        let score1 = scorer.calculate_risk_score(&step1);
        let score2 = scorer.calculate_risk_score(&step2);

        // If factors are identical, scores should be identical
        if impact1 == impact2 && reversibility1 == reversibility2 && complexity1 == complexity2 {
            prop_assert_eq!(score1, score2, "Identical factors should produce identical scores");
        }

        // If step1 has significantly higher impact and significantly lower reversibility, 
        // it should have higher or equal risk (accounting for complexity differences)
        if impact1 > impact2 + 20 && reversibility1 + 20 < reversibility2 {
            prop_assert!(
                score1 >= score2,
                "Significantly higher impact and lower reversibility should result in higher or equal risk score"
            );
        }
    }
}

// Property 15: High-Risk Approval Requirement
// **Validates: Requirements 3.3**
// *For any* workflow step with a risk score above the configured threshold,
// execution SHALL be blocked until explicit approval is received.
proptest! {
    #[test]
    fn prop_high_risk_requires_approval(
        impact in 0u8..=100,
        reversibility in 0u8..=100,
        complexity in 0u8..=100,
        threshold in 0u8..=100,
    ) {
        let scorer = RiskScorer::with_threshold(threshold);
        let step = WorkflowStep {
            id: "step".to_string(),
            name: "Test Step".to_string(),
            step_type: StepType::Agent(AgentStep {
                agent_id: "agent".to_string(),
                task: "task".to_string(),
            }),
            config: StepConfig {
                config: serde_json::json!({}),
            },
            dependencies: Vec::new(),
            approval_required: false,
            on_error: ErrorAction::Fail,
            risk_score: None,
            risk_factors: RiskFactors {
                impact,
                reversibility,
                complexity,
            },
        };

        let risk_score = scorer.calculate_risk_score(&step);
        let requires_approval = scorer.requires_approval(risk_score);

        // Approval should be required if and only if risk score > threshold
        if risk_score > threshold {
            prop_assert!(requires_approval, "Risk score {} > threshold {} should require approval", risk_score, threshold);
        } else {
            prop_assert!(!requires_approval, "Risk score {} <= threshold {} should not require approval", risk_score, threshold);
        }
    }
}

// Property 16: Safety Constraints Application
// **Validates: Requirements 3.4**
// *For any* approved high-risk operation, the system SHALL enforce execution timeouts,
// resource limits, and maintain rollback capability.
proptest! {
    #[test]
    fn prop_safety_constraints_enforced(timeout_ms in 100_000u64..=600_000) {
        let constraints = SafetyConstraints::new();
        let step = WorkflowStep {
            id: "step".to_string(),
            name: "Test Step".to_string(),
            step_type: StepType::Command(crate::models::CommandStep {
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

        let enforced_timeout = constraints.enforce_timeout(&step);
        let enforced_ms = enforced_timeout.as_millis() as u64;

        // Enforced timeout should not exceed maximum
        prop_assert!(
            enforced_ms <= constraints.max_timeout_ms,
            "Enforced timeout {} ms exceeds maximum {} ms",
            enforced_ms,
            constraints.max_timeout_ms
        );

        // Rollback capability should be maintained
        prop_assert!(
            constraints.has_rollback_capability(&step),
            "Rollback capability should be maintained for high-risk operations"
        );
    }
}

// Property 17: Risk Assessment Report Generation
// **Validates: Requirements 3.5**
// *For any* completed workflow, the system SHALL generate a risk assessment report
// containing risk scores for all steps and all approval decisions.
proptest! {
    #[test]
    fn prop_risk_assessment_report_complete(
        steps_count in 1usize..=10,
        seed in 0u64..1000,
    ) {
        let scorer = RiskScorer::new();
        // Create a 16-byte seed for XorShift
        let mut seed_bytes = [0u8; 16];
        seed_bytes[0..8].copy_from_slice(&seed.to_le_bytes());
        let mut rng = proptest::test_runner::TestRng::from_seed(proptest::test_runner::RngAlgorithm::XorShift, &seed_bytes);

        // Generate workflow steps
        let mut steps = Vec::new();
        for i in 0..steps_count {
            let impact = (rng.next_u32() % 101) as u8;
            let reversibility = (rng.next_u32() % 101) as u8;
            let complexity = (rng.next_u32() % 101) as u8;

            steps.push(WorkflowStep {
                id: format!("step_{}", i),
                name: format!("Step {}", i),
                step_type: StepType::Agent(AgentStep {
                    agent_id: "agent".to_string(),
                    task: "task".to_string(),
                }),
                config: StepConfig {
                    config: serde_json::json!({}),
                },
                dependencies: Vec::new(),
                approval_required: false,
                on_error: ErrorAction::Fail,
                risk_score: None,
                risk_factors: RiskFactors {
                    impact,
                    reversibility,
                    complexity,
                },
            });
        }

        // Create workflow state
        let mut step_results = HashMap::new();
        for step in &steps {
            step_results.insert(
                step.id.clone(),
                StepResult {
                    status: StepStatus::Completed,
                    output: None,
                    error: None,
                    duration_ms: 1000,
                },
            );
        }

        let state = WorkflowState {
            workflow_id: "workflow".to_string(),
            status: WorkflowStatus::Completed,
            current_step: None,
            completed_steps: steps.iter().map(|s| s.id.clone()).collect(),
            step_results,
            started_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Generate report
        let report = scorer.generate_report("workflow", "Test Workflow", &steps, &state);

        // Verify report contains all steps
        prop_assert_eq!(
            report.step_assessments.len(),
            steps_count,
            "Report should contain assessment for all steps"
        );

        // Verify all step IDs are present
        for step in &steps {
            let found = report.step_assessments.iter().any(|a| a.step_id == step.id);
            prop_assert!(found, "Report should contain assessment for step {}", step.id);
        }

        // Verify overall risk score is in valid range
        prop_assert!(
            report.overall_risk_score <= 100,
            "Overall risk score should be <= 100"
        );

        // Verify report timestamp is recent
        let now = Utc::now();
        let diff = now.signed_duration_since(report.generated_at);
        prop_assert!(
            diff.num_seconds() >= 0 && diff.num_seconds() <= 5,
            "Report timestamp should be recent"
        );
    }
}
