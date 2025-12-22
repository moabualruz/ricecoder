//! Property-based tests for risk scoring
//!
//! **Feature: ricecoder-execution, Property 1: Risk Score Consistency**
//! **Validates: Requirements 1.1, 1.2**

use std::time::Duration;

use proptest::prelude::*;
use ricecoder_execution::{
    ExecutionPlan, ExecutionRiskScorer, ExecutionStep, RiskLevel, StepAction, StepStatus,
};
use uuid::Uuid;

/// Strategy for generating valid file paths
fn file_path_strategy() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9_\-./]{1,50}\.rs".prop_map(|s| s.to_string())
}

/// Strategy for generating execution steps
fn execution_step_strategy() -> impl Strategy<Value = ExecutionStep> {
    (
        "[a-zA-Z0-9 ]{1,50}",
        prop_oneof![
            (file_path_strategy(), ".*").prop_map(|(path, diff)| StepAction::ModifyFile {
                path,
                diff: diff.to_string(),
            }),
            file_path_strategy().prop_map(|path| StepAction::DeleteFile { path }),
            (file_path_strategy(), ".*").prop_map(|(path, content)| StepAction::CreateFile {
                path,
                content: content.to_string(),
            }),
        ],
    )
        .prop_map(|(desc, action)| ExecutionStep {
            id: Uuid::new_v4().to_string(),
            description: desc.to_string(),
            action,
            risk_score: Default::default(),
            dependencies: Vec::new(),
            rollback_action: None,
            status: StepStatus::Pending,
        })
}

/// Strategy for generating execution plans
fn execution_plan_strategy() -> impl Strategy<Value = ExecutionPlan> {
    prop::collection::vec(execution_step_strategy(), 0..10).prop_map(|steps| ExecutionPlan {
        id: Uuid::new_v4().to_string(),
        name: "Test Plan".to_string(),
        steps,
        risk_score: Default::default(),
        estimated_duration: Duration::from_secs(0),
        estimated_complexity: Default::default(),
        requires_approval: false,
        editable: true,
    })
}

proptest! {
    /// Property 1: Risk Score Consistency
    /// For any execution plan, risk scoring SHALL produce consistent results for the same plan.
    ///
    /// **Feature: ricecoder-execution, Property 1: Risk Score Consistency**
    /// **Validates: Requirements 1.1, 1.2**
    #[test]
    fn prop_risk_score_consistency(plan in execution_plan_strategy()) {
        let scorer = ExecutionRiskScorer::new();

        // Score the same plan multiple times
        let score1 = scorer.score_plan(&plan);
        let score2 = scorer.score_plan(&plan);
        let score3 = scorer.score_plan(&plan);

        // All scores should be identical
        prop_assert_eq!(score1.score, score2.score, "Risk scores should be consistent");
        prop_assert_eq!(score2.score, score3.score, "Risk scores should be consistent");
        prop_assert_eq!(score1.level, score2.level, "Risk levels should be consistent");
        prop_assert_eq!(score2.level, score3.level, "Risk levels should be consistent");
    }

    /// Property: Risk Score is Non-Negative
    /// For any execution plan, the risk score should never be negative.
    #[test]
    fn prop_risk_score_non_negative(plan in execution_plan_strategy()) {
        let scorer = ExecutionRiskScorer::new();
        let score = scorer.score_plan(&plan);

        prop_assert!(score.score >= 0.0, "Risk score should be non-negative");
    }

    /// Property: Risk Score Increases with File Count
    /// For any execution plan, adding more file modifications should not decrease the risk score.
    #[test]
    fn prop_risk_score_increases_with_files(
        mut plan in execution_plan_strategy(),
        additional_steps in prop::collection::vec(execution_step_strategy(), 1..5),
    ) {
        let scorer = ExecutionRiskScorer::new();

        let score_before = scorer.score_plan(&plan);
        plan.steps.extend(additional_steps);
        let score_after = scorer.score_plan(&plan);

        prop_assert!(
            score_after.score >= score_before.score,
            "Adding steps should not decrease risk score"
        );
    }

    /// Property: Risk Level Matches Score
    /// For any execution plan, the risk level should match the score thresholds.
    #[test]
    fn prop_risk_level_matches_score(plan in execution_plan_strategy()) {
        let scorer = ExecutionRiskScorer::new();
        let score = scorer.score_plan(&plan);

        match score.level {
            RiskLevel::Low => prop_assert!(score.score < 0.5),
            RiskLevel::Medium => prop_assert!(score.score >= 0.5 && score.score < 1.5),
            RiskLevel::High => prop_assert!(score.score >= 1.5 && score.score < 2.5),
            RiskLevel::Critical => prop_assert!(score.score >= 2.5),
        }
    }

    /// Property: Risk Factors Sum to Score
    /// For any execution plan, the sum of risk factor weights should equal the total score.
    #[test]
    fn prop_risk_factors_sum_to_score(plan in execution_plan_strategy()) {
        let scorer = ExecutionRiskScorer::new();
        let score = scorer.score_plan(&plan);

        let factors_sum: f32 = score.factors.iter().map(|f| f.weight).sum();
        prop_assert!(
            (factors_sum - score.score).abs() < 0.01,
            "Risk factors should sum to total score"
        );
    }

    /// Property: Approval Threshold is Consistent
    /// For any execution plan, the approval requirement should be consistent.
    #[test]
    fn prop_approval_threshold_consistent(plan in execution_plan_strategy()) {
        let scorer = ExecutionRiskScorer::new();
        let score = scorer.score_plan(&plan);

        let requires_approval_1 = scorer.requires_approval(&score);
        let requires_approval_2 = scorer.requires_approval(&score);

        prop_assert_eq!(
            requires_approval_1, requires_approval_2,
            "Approval requirement should be consistent"
        );
    }
}
