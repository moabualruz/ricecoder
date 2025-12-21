//! Property-based tests for GitHub Actions Integration
//!
//! **Feature: ricecoder-github, Property 11-15: Workflow Triggering, Status Tracking, CI Failure Diagnostics, Workflow Retry Logic, CI Result Summarization**

use proptest::prelude::*;
use ricecoder_github::{
    ActionsIntegration, ActionsOperations, CiResultSummary, WorkflowStatus, WorkflowTriggerRequest,
};
use std::collections::HashMap;

// Strategy for generating valid workflow names
fn workflow_name_strategy() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9_\-\.]{1,50}\.ya?ml".prop_map(|s| s.to_string())
}

// Strategy for generating valid branch names
fn branch_name_strategy() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9_\-/]{1,50}".prop_map(|s| s.to_string())
}

// Strategy for generating valid run IDs
fn run_id_strategy() -> impl Strategy<Value = u64> {
    1u64..=u64::MAX
}

// Strategy for generating valid PR numbers
fn pr_number_strategy() -> impl Strategy<Value = u32> {
    1u32..=u32::MAX
}

// Property 11: Workflow Triggering
// **Validates: Requirements 3.1**
// *For any* workflow trigger request, the GitHub Actions API SHALL be called with correct parameters.
proptest! {
    #[test]
    fn prop_workflow_triggering_with_valid_inputs(
        workflow in workflow_name_strategy(),
        branch in branch_name_strategy(),
    ) {
        let actions = ActionsIntegration::new(
            "token".to_string(),
            "owner".to_string(),
            "repo".to_string(),
        );

        let request = WorkflowTriggerRequest {
            workflow: workflow.clone(),
            ref_branch: branch.clone(),
            inputs: HashMap::new(),
        };

        // Use a blocking runtime for the async call
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(actions.trigger_workflow(request));

        // Property: Triggering with valid inputs should succeed
        prop_assert!(result.is_ok());

        let trigger_result = result.unwrap();
        // Property: Result should have a valid run ID
        prop_assert!(trigger_result.run_id > 0);
        // Property: Result should have a valid run number
        prop_assert!(trigger_result.run_number > 0);
        // Property: Result should have a valid HTML URL
        prop_assert!(!trigger_result.html_url.is_empty());
        // Property: Status should be Queued initially
        prop_assert_eq!(trigger_result.status, WorkflowStatus::Queued);
    }
}

// Property 12: Workflow Status Tracking
// **Validates: Requirements 3.2**
// *For any* workflow execution, the system SHALL query and report the workflow status correctly.
proptest! {
    #[test]
    fn prop_workflow_status_tracking_with_valid_run_id(run_id in run_id_strategy()) {
        let actions = ActionsIntegration::new(
            "token".to_string(),
            "owner".to_string(),
            "repo".to_string(),
        );

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(actions.track_workflow_status(run_id));

        // Property: Tracking with valid run ID should succeed
        prop_assert!(result.is_ok());

        let status_result = result.unwrap();
        // Property: Run ID should match the requested ID
        prop_assert_eq!(status_result.run_id, run_id);
        // Property: Progress should be between 0 and 100
        prop_assert!(status_result.progress <= 100);
        // Property: Status should be one of the valid statuses
        prop_assert!(matches!(
            status_result.status,
            WorkflowStatus::Queued
                | WorkflowStatus::InProgress
                | WorkflowStatus::Completed
                | WorkflowStatus::Failed
                | WorkflowStatus::Cancelled
                | WorkflowStatus::Skipped
        ));
    }
}

// Property 13: CI Failure Diagnostics
// **Validates: Requirements 3.3**
// *For any* CI failure, the system SHALL generate diagnostic information including error logs and failed steps.
proptest! {
    #[test]
    fn prop_ci_failure_diagnostics_with_valid_run_id(run_id in run_id_strategy()) {
        let actions = ActionsIntegration::new(
            "token".to_string(),
            "owner".to_string(),
            "repo".to_string(),
        );

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(actions.diagnose_ci_failure(run_id));

        // Property: Diagnostics with valid run ID should succeed
        prop_assert!(result.is_ok());

        let diagnostics = result.unwrap();
        // Property: Diagnostics should have recommendations
        prop_assert!(!diagnostics.recommendations.is_empty());
        // Property: Diagnosed timestamp should be recent
        prop_assert!(diagnostics.diagnosed_at <= chrono::Utc::now());
    }
}

// Property 14: Workflow Retry Logic
// **Validates: Requirements 3.4**
// *For any* failed workflow, the system SHALL support re-running the workflow after fixes are applied.
proptest! {
    #[test]
    fn prop_workflow_retry_logic_with_valid_run_id(run_id in run_id_strategy()) {
        let actions = ActionsIntegration::new(
            "token".to_string(),
            "owner".to_string(),
            "repo".to_string(),
        );

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(actions.retry_workflow(run_id));

        // Property: Retry with valid run ID should succeed
        prop_assert!(result.is_ok());

        let retry_result = result.unwrap();
        // Property: New run ID should be different from original
        prop_assert_ne!(retry_result.new_run_id, run_id);
        // Property: New run ID should be greater than original
        prop_assert!(retry_result.new_run_id > run_id);
        // Property: New run number should be valid
        prop_assert!(retry_result.new_run_number > 0);
        // Property: Status should be Queued for new run
        prop_assert_eq!(retry_result.status, WorkflowStatus::Queued);
    }
}

// Property 15: CI Result Summarization
// **Validates: Requirements 3.5**
// *For any* workflow completion, the system SHALL generate a summary and post it as a PR comment.
proptest! {
    #[test]
    fn prop_ci_result_summarization(
        run_id in run_id_strategy(),
        pr_number in pr_number_strategy(),
        total_jobs in 1u32..=100u32,
        passed_jobs in 0u32..=100u32,
        failed_jobs in 0u32..=100u32,
    ) {
        let summary = CiResultSummary {
            run_id,
            status: WorkflowStatus::Completed,
            conclusion: Some("completed".to_string()),
            total_jobs,
            passed_jobs,
            failed_jobs,
            skipped_jobs: 0,
            duration_seconds: 120,
            key_findings: vec!["Test finding".to_string()],
            recommendations: vec!["Test recommendation".to_string()],
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(ActionsOperations::summarize_in_pr_comment(pr_number, summary));

        // Property: Summarization should succeed
        prop_assert!(result.is_ok());

        let comment = result.unwrap();
        // Property: Comment body should not be empty
        prop_assert!(!comment.body.is_empty());
        // Property: Comment body should contain CI Results header
        prop_assert!(comment.body.contains("CI Results"));
        // Property: Comment body should contain job counts
        prop_assert!(comment.body.contains("total"));
        // Property: Comment body should contain duration
        prop_assert!(comment.body.contains("Duration"));
    }
}

// Additional property: Workflow triggering with invalid inputs should fail
proptest! {
    #[test]
    fn prop_workflow_triggering_rejects_empty_workflow(branch in branch_name_strategy()) {
        let actions = ActionsIntegration::new(
            "token".to_string(),
            "owner".to_string(),
            "repo".to_string(),
        );

        let request = WorkflowTriggerRequest {
            workflow: String::new(),
            ref_branch: branch,
            inputs: HashMap::new(),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(actions.trigger_workflow(request));

        // Property: Empty workflow should be rejected
        prop_assert!(result.is_err());
    }
}

// Additional property: Workflow triggering with invalid branch should fail
proptest! {
    #[test]
    fn prop_workflow_triggering_rejects_empty_branch(workflow in workflow_name_strategy()) {
        let actions = ActionsIntegration::new(
            "token".to_string(),
            "owner".to_string(),
            "repo".to_string(),
        );

        let request = WorkflowTriggerRequest {
            workflow,
            ref_branch: String::new(),
            inputs: HashMap::new(),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(actions.trigger_workflow(request));

        // Property: Empty branch should be rejected
        prop_assert!(result.is_err());
    }
}

// Additional property: Status tracking with zero run ID should fail
#[test]
fn prop_workflow_status_tracking_rejects_zero_run_id() {
    let actions =
        ActionsIntegration::new("token".to_string(), "owner".to_string(), "repo".to_string());

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(actions.track_workflow_status(0));

    // Property: Zero run ID should be rejected
    assert!(result.is_err());
}

// Additional property: Retry with zero run ID should fail
#[test]
fn prop_workflow_retry_rejects_zero_run_id() {
    let actions =
        ActionsIntegration::new("token".to_string(), "owner".to_string(), "repo".to_string());

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(actions.retry_workflow(0));

    // Property: Zero run ID should be rejected
    assert!(result.is_err());
}

// Additional property: Fix and retry with empty fixes should fail
proptest! {
    #[test]
    fn prop_fix_and_retry_rejects_empty_fixes(run_id in run_id_strategy()) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(ActionsOperations::fix_and_retry(run_id, vec![]));

        // Property: Empty fixes should be rejected
        prop_assert!(result.is_err());
    }
}

// Additional property: Fix and retry with zero run ID should fail
#[test]
fn prop_fix_and_retry_rejects_zero_run_id() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(ActionsOperations::fix_and_retry(0, vec!["fix".to_string()]));

    // Property: Zero run ID should be rejected
    assert!(result.is_err());
}

// Additional property: PR comment with zero PR number should fail
#[test]
fn prop_pr_comment_rejects_zero_pr_number() {
    let summary = CiResultSummary {
        run_id: 12345,
        status: WorkflowStatus::Completed,
        conclusion: Some("success".to_string()),
        total_jobs: 5,
        passed_jobs: 5,
        failed_jobs: 0,
        skipped_jobs: 0,
        duration_seconds: 120,
        key_findings: vec![],
        recommendations: vec![],
    };

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(ActionsOperations::summarize_in_pr_comment(0, summary));

    // Property: Zero PR number should be rejected
    assert!(result.is_err());
}

// Additional property: Load workflow config with empty path should fail
#[test]
fn prop_load_workflow_config_rejects_empty_path() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(ActionsOperations::load_workflow_config(""));

    // Property: Empty path should be rejected
    assert!(result.is_err());
}

// Additional property: Generate detailed report with zero run ID should fail
#[test]
fn prop_generate_detailed_report_rejects_zero_run_id() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(ActionsOperations::generate_detailed_report(
        0,
        WorkflowStatus::Completed,
        vec![],
    ));

    // Property: Zero run ID should be rejected
    assert!(result.is_err());
}
