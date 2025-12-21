//! GitHub Actions Operations
//!
//! Advanced operations for GitHub Actions workflow management and reporting.

use crate::errors::{GitHubError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::actions_integration::{CiResultSummary, WorkflowJob, WorkflowStatus};

/// Workflow configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    /// Workflow file path
    pub file_path: String,
    /// Trigger events
    pub triggers: Vec<String>,
    /// Environment variables
    pub env_vars: HashMap<String, String>,
    /// Timeout in minutes
    pub timeout_minutes: u32,
}

/// Workflow iteration result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowIterationResult {
    /// Original run ID
    pub original_run_id: u64,
    /// New run ID after retry
    pub new_run_id: u64,
    /// Fixes applied
    pub fixes_applied: Vec<String>,
    /// Status after retry
    pub status: WorkflowStatus,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// PR comment for CI results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CiResultComment {
    /// Comment body
    pub body: String,
    /// Comment ID (if already posted)
    pub comment_id: Option<u64>,
    /// Timestamp
    pub created_at: DateTime<Utc>,
}

/// Workflow configuration support result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfigResult {
    /// Configuration loaded successfully
    pub success: bool,
    /// Configuration details
    pub config: Option<WorkflowConfig>,
    /// Error message if failed
    pub error: Option<String>,
}

/// GitHub Actions Operations manager
pub struct ActionsOperations;

impl ActionsOperations {
    /// Fix issues and re-run workflows
    ///
    /// # Arguments
    ///
    /// * `run_id` - Original workflow run ID
    /// * `fixes` - List of fixes to apply
    ///
    /// # Returns
    ///
    /// Result containing the iteration result
    pub async fn fix_and_retry(run_id: u64, fixes: Vec<String>) -> Result<WorkflowIterationResult> {
        if run_id == 0 {
            return Err(GitHubError::invalid_input("Run ID cannot be zero"));
        }
        if fixes.is_empty() {
            return Err(GitHubError::invalid_input(
                "At least one fix must be provided",
            ));
        }

        // In a real implementation, this would:
        // 1. Apply the fixes to the codebase
        // 2. Commit the changes
        // 3. Re-run the workflow
        // For now, we return a mock result
        Ok(WorkflowIterationResult {
            original_run_id: run_id,
            new_run_id: run_id + 1,
            fixes_applied: fixes,
            status: WorkflowStatus::Queued,
            timestamp: Utc::now(),
        })
    }

    /// Summarize CI results in a PR comment
    ///
    /// # Arguments
    ///
    /// * `pr_number` - PR number
    /// * `summary` - CI result summary
    ///
    /// # Returns
    ///
    /// Result containing the PR comment
    pub async fn summarize_in_pr_comment(
        pr_number: u32,
        summary: CiResultSummary,
    ) -> Result<CiResultComment> {
        if pr_number == 0 {
            return Err(GitHubError::invalid_input("PR number cannot be zero"));
        }

        // Build the comment body
        let findings = summary
            .key_findings
            .iter()
            .map(|f| format!("- {}", f))
            .collect::<Vec<_>>()
            .join("\n");
        let recommendations = if summary.recommendations.is_empty() {
            "No recommendations".to_string()
        } else {
            summary
                .recommendations
                .iter()
                .map(|r| format!("- {}", r))
                .collect::<Vec<_>>()
                .join("\n")
        };
        let body = format!(
            "## CI Results\n\n\
             **Status**: {:?}\n\n\
             **Jobs**: {} total, {} passed, {} failed, {} skipped\n\n\
             **Duration**: {} seconds\n\n\
             ### Key Findings\n\
             {}\n\n\
             ### Recommendations\n\
             {}",
            summary.status,
            summary.total_jobs,
            summary.passed_jobs,
            summary.failed_jobs,
            summary.skipped_jobs,
            summary.duration_seconds,
            findings,
            recommendations
        );

        Ok(CiResultComment {
            body,
            comment_id: None,
            created_at: Utc::now(),
        })
    }

    /// Support workflow configuration
    ///
    /// # Arguments
    ///
    /// * `config_path` - Path to workflow configuration file
    ///
    /// # Returns
    ///
    /// Result containing the workflow configuration
    pub async fn load_workflow_config(config_path: &str) -> Result<WorkflowConfigResult> {
        if config_path.is_empty() {
            return Err(GitHubError::invalid_input("Config path cannot be empty"));
        }

        // In a real implementation, this would:
        // 1. Read the configuration file
        // 2. Parse it (YAML or JSON)
        // 3. Validate the configuration
        // For now, we return a mock result
        Ok(WorkflowConfigResult {
            success: true,
            config: Some(WorkflowConfig {
                file_path: config_path.to_string(),
                triggers: vec!["push".to_string(), "pull_request".to_string()],
                env_vars: HashMap::new(),
                timeout_minutes: 60,
            }),
            error: None,
        })
    }

    /// Generate detailed workflow report
    ///
    /// # Arguments
    ///
    /// * `run_id` - Workflow run ID
    /// * `status` - Workflow status
    /// * `jobs` - Jobs in the workflow
    ///
    /// # Returns
    ///
    /// Result containing a detailed report
    pub async fn generate_detailed_report(
        run_id: u64,
        status: WorkflowStatus,
        jobs: Vec<WorkflowJob>,
    ) -> Result<String> {
        if run_id == 0 {
            return Err(GitHubError::invalid_input("Run ID cannot be zero"));
        }

        let mut report = format!(
            "# Workflow Report\n\nRun ID: {}\n\nStatus: {:?}\n\n",
            run_id, status
        );

        report.push_str("## Jobs\n\n");
        for job in jobs {
            report.push_str(&format!(
                "### {}\n- Status: {:?}\n- Conclusion: {}\n\n",
                job.name,
                job.status,
                job.conclusion.as_deref().unwrap_or("N/A")
            ));
        }

        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fix_and_retry_with_zero_id() {
        let result = ActionsOperations::fix_and_retry(0, vec!["fix".to_string()]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fix_and_retry_with_empty_fixes() {
        let result = ActionsOperations::fix_and_retry(12345, vec![]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fix_and_retry_success() {
        let fixes = vec!["fix1".to_string(), "fix2".to_string()];
        let result = ActionsOperations::fix_and_retry(12345, fixes.clone()).await;
        assert!(result.is_ok());
        let iteration = result.unwrap();
        assert_eq!(iteration.original_run_id, 12345);
        assert_eq!(iteration.new_run_id, 12346);
        assert_eq!(iteration.fixes_applied, fixes);
    }

    #[tokio::test]
    async fn test_summarize_in_pr_comment_with_zero_pr() {
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
    async fn test_summarize_in_pr_comment_success() {
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
    }

    #[tokio::test]
    async fn test_load_workflow_config_with_empty_path() {
        let result = ActionsOperations::load_workflow_config("").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_load_workflow_config_success() {
        let result = ActionsOperations::load_workflow_config(".github/workflows/test.yml").await;
        assert!(result.is_ok());
        let config_result = result.unwrap();
        assert!(config_result.success);
        assert!(config_result.config.is_some());
    }

    #[tokio::test]
    async fn test_generate_detailed_report_with_zero_id() {
        let result =
            ActionsOperations::generate_detailed_report(0, WorkflowStatus::Completed, vec![]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_generate_detailed_report_success() {
        let result =
            ActionsOperations::generate_detailed_report(12345, WorkflowStatus::Completed, vec![])
                .await;
        assert!(result.is_ok());
        let report = result.unwrap();
        assert!(report.contains("Workflow Report"));
        assert!(report.contains("12345"));
    }
}
