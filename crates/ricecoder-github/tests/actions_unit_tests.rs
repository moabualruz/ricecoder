//! Unit tests for GitHub Actions Integration

use ricecoder_github::{
    ActionsIntegration, ActionsOperations, CiResultSummary, WorkflowStatus, WorkflowTriggerRequest,
};
use std::collections::HashMap;

#[test]
fn test_actions_integration_creation() {
    let actions = ActionsIntegration::new(
        "test_token".to_string(),
        "test_owner".to_string(),
        "test_repo".to_string(),
    );
    // Verify the manager was created successfully
    assert_eq!(actions.owner, "test_owner");
    assert_eq!(actions.repo, "test_repo");
}

#[tokio::test]
async fn test_trigger_workflow_with_valid_inputs() {
    let actions = ActionsIntegration::new(
        "token".to_string(),
        "owner".to_string(),
        "repo".to_string(),
    );

    let request = WorkflowTriggerRequest {
        workflow: "test.yml".to_string(),
        ref_branch: "main".to_string(),
        inputs: HashMap::new(),
    };

    let result = actions.trigger_workflow(request).await;
    assert!(result.is_ok());

    let trigger_result = result.unwrap();
    assert!(trigger_result.run_id > 0);
    assert!(trigger_result.run_number > 0);
    assert!(!trigger_result.html_url.is_empty());
    assert_eq!(trigger_result.status, WorkflowStatus::Queued);
}

#[tokio::test]
async fn test_trigger_workflow_with_empty_workflow_name() {
    let actions = ActionsIntegration::new(
        "token".to_string(),
        "owner".to_string(),
        "repo".to_string(),
    );

    let request = WorkflowTriggerRequest {
        workflow: String::new(),
        ref_branch: "main".to_string(),
        inputs: HashMap::new(),
    };

    let result = actions.trigger_workflow(request).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_trigger_workflow_with_empty_branch() {
    let actions = ActionsIntegration::new(
        "token".to_string(),
        "owner".to_string(),
        "repo".to_string(),
    );

    let request = WorkflowTriggerRequest {
        workflow: "test.yml".to_string(),
        ref_branch: String::new(),
        inputs: HashMap::new(),
    };

    let result = actions.trigger_workflow(request).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_trigger_workflow_with_inputs() {
    let actions = ActionsIntegration::new(
        "token".to_string(),
        "owner".to_string(),
        "repo".to_string(),
    );

    let mut inputs = HashMap::new();
    inputs.insert("key1".to_string(), "value1".to_string());
    inputs.insert("key2".to_string(), "value2".to_string());

    let request = WorkflowTriggerRequest {
        workflow: "test.yml".to_string(),
        ref_branch: "main".to_string(),
        inputs,
    };

    let result = actions.trigger_workflow(request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_track_workflow_status_with_valid_run_id() {
    let actions = ActionsIntegration::new(
        "token".to_string(),
        "owner".to_string(),
        "repo".to_string(),
    );

    let result = actions.track_workflow_status(12345).await;
    assert!(result.is_ok());

    let status = result.unwrap();
    assert_eq!(status.run_id, 12345);
    assert!(status.progress <= 100);
    assert!(matches!(
        status.status,
        WorkflowStatus::Queued
            | WorkflowStatus::InProgress
            | WorkflowStatus::Completed
            | WorkflowStatus::Failed
            | WorkflowStatus::Cancelled
            | WorkflowStatus::Skipped
    ));
}

#[tokio::test]
async fn test_track_workflow_status_with_zero_run_id() {
    let actions = ActionsIntegration::new(
        "token".to_string(),
        "owner".to_string(),
        "repo".to_string(),
    );

    let result = actions.track_workflow_status(0).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_diagnose_ci_failure_with_valid_run_id() {
    let actions = ActionsIntegration::new(
        "token".to_string(),
        "owner".to_string(),
        "repo".to_string(),
    );

    let result = actions.diagnose_ci_failure(12345).await;
    assert!(result.is_ok());

    let diagnostics = result.unwrap();
    assert!(!diagnostics.recommendations.is_empty());
}

#[tokio::test]
async fn test_diagnose_ci_failure_with_zero_run_id() {
    let actions = ActionsIntegration::new(
        "token".to_string(),
        "owner".to_string(),
        "repo".to_string(),
    );

    let result = actions.diagnose_ci_failure(0).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_retry_workflow_with_valid_run_id() {
    let actions = ActionsIntegration::new(
        "token".to_string(),
        "owner".to_string(),
        "repo".to_string(),
    );

    let result = actions.retry_workflow(12345).await;
    assert!(result.is_ok());

    let retry_result = result.unwrap();
    assert_eq!(retry_result.new_run_id, 12346);
    assert!(retry_result.new_run_number > 0);
    assert_eq!(retry_result.status, WorkflowStatus::Queued);
    assert!(!retry_result.html_url.is_empty());
}

#[tokio::test]
async fn test_retry_workflow_with_zero_run_id() {
    let actions = ActionsIntegration::new(
        "token".to_string(),
        "owner".to_string(),
        "repo".to_string(),
    );

    let result = actions.retry_workflow(0).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_summarize_ci_results_with_valid_run_id() {
    let actions = ActionsIntegration::new(
        "token".to_string(),
        "owner".to_string(),
        "repo".to_string(),
    );

    let result = actions.summarize_ci_results(12345).await;
    assert!(result.is_ok());

    let summary = result.unwrap();
    assert_eq!(summary.run_id, 12345);
    assert_eq!(summary.total_jobs, 5);
    assert_eq!(summary.passed_jobs, 5);
    assert_eq!(summary.failed_jobs, 0);
}

#[tokio::test]
async fn test_summarize_ci_results_with_zero_run_id() {
    let actions = ActionsIntegration::new(
        "token".to_string(),
        "owner".to_string(),
        "repo".to_string(),
    );

    let result = actions.summarize_ci_results(0).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_fix_and_retry_with_valid_inputs() {
    let fixes = vec!["fix1".to_string(), "fix2".to_string()];
    let result = ActionsOperations::fix_and_retry(12345, fixes.clone()).await;
    assert!(result.is_ok());

    let iteration = result.unwrap();
    assert_eq!(iteration.original_run_id, 12345);
    assert_eq!(iteration.new_run_id, 12346);
    assert_eq!(iteration.fixes_applied, fixes);
    assert_eq!(iteration.status, WorkflowStatus::Queued);
}

#[tokio::test]
async fn test_fix_and_retry_with_zero_run_id() {
    let result = ActionsOperations::fix_and_retry(0, vec!["fix".to_string()]).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_fix_and_retry_with_empty_fixes() {
    let result = ActionsOperations::fix_and_retry(12345, vec![]).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_summarize_in_pr_comment_with_valid_inputs() {
    let summary = CiResultSummary {
        run_id: 12345,
        status: WorkflowStatus::Completed,
        conclusion: Some("success".to_string()),
        total_jobs: 5,
        passed_jobs: 5,
        failed_jobs: 0,
        skipped_jobs: 0,
        duration_seconds: 120,
        key_findings: vec!["All tests passed".to_string()],
        recommendations: vec![],
    };

    let result = ActionsOperations::summarize_in_pr_comment(42, summary).await;
    assert!(result.is_ok());

    let comment = result.unwrap();
    assert!(comment.body.contains("CI Results"));
    assert!(comment.body.contains("5 total"));
    assert!(comment.body.contains("120 seconds"));
}

#[tokio::test]
async fn test_summarize_in_pr_comment_with_zero_pr_number() {
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

    let result = ActionsOperations::summarize_in_pr_comment(0, summary).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_summarize_in_pr_comment_with_failures() {
    let summary = CiResultSummary {
        run_id: 12345,
        status: WorkflowStatus::Failed,
        conclusion: Some("failure".to_string()),
        total_jobs: 5,
        passed_jobs: 3,
        failed_jobs: 2,
        skipped_jobs: 0,
        duration_seconds: 180,
        key_findings: vec!["2 tests failed".to_string()],
        recommendations: vec!["Check test logs".to_string()],
    };

    let result = ActionsOperations::summarize_in_pr_comment(42, summary).await;
    assert!(result.is_ok());

    let comment = result.unwrap();
    assert!(comment.body.contains("CI Results"));
    assert!(comment.body.contains("5 total"));
    assert!(comment.body.contains("2 failed"));
}

#[tokio::test]
async fn test_load_workflow_config_with_valid_path() {
    let result = ActionsOperations::load_workflow_config(".github/workflows/test.yml").await;
    assert!(result.is_ok());

    let config_result = result.unwrap();
    assert!(config_result.success);
    assert!(config_result.config.is_some());

    let config = config_result.config.unwrap();
    assert_eq!(config.file_path, ".github/workflows/test.yml");
    assert!(!config.triggers.is_empty());
}

#[tokio::test]
async fn test_load_workflow_config_with_empty_path() {
    let result = ActionsOperations::load_workflow_config("").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_generate_detailed_report_with_valid_inputs() {
    let result = ActionsOperations::generate_detailed_report(
        12345,
        WorkflowStatus::Completed,
        vec![],
    )
    .await;
    assert!(result.is_ok());

    let report = result.unwrap();
    assert!(report.contains("Workflow Report"));
    assert!(report.contains("12345"));
    assert!(report.contains("Completed"));
}

#[tokio::test]
async fn test_generate_detailed_report_with_zero_run_id() {
    let result = ActionsOperations::generate_detailed_report(0, WorkflowStatus::Completed, vec![])
        .await;
    assert!(result.is_err());
}

#[test]
fn test_workflow_status_enum_values() {
    // Test that all workflow status values are valid
    let statuses = vec![
        WorkflowStatus::Queued,
        WorkflowStatus::InProgress,
        WorkflowStatus::Completed,
        WorkflowStatus::Failed,
        WorkflowStatus::Cancelled,
        WorkflowStatus::Skipped,
    ];

    for status in statuses {
        assert!(matches!(
            status,
            WorkflowStatus::Queued
                | WorkflowStatus::InProgress
                | WorkflowStatus::Completed
                | WorkflowStatus::Failed
                | WorkflowStatus::Cancelled
                | WorkflowStatus::Skipped
        ));
    }
}

#[test]
fn test_ci_result_summary_creation() {
    let summary = CiResultSummary {
        run_id: 12345,
        status: WorkflowStatus::Completed,
        conclusion: Some("success".to_string()),
        total_jobs: 5,
        passed_jobs: 5,
        failed_jobs: 0,
        skipped_jobs: 0,
        duration_seconds: 120,
        key_findings: vec!["Test".to_string()],
        recommendations: vec![],
    };

    assert_eq!(summary.run_id, 12345);
    assert_eq!(summary.total_jobs, 5);
    assert_eq!(summary.passed_jobs, 5);
    assert_eq!(summary.failed_jobs, 0);
}
