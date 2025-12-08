//! Issue Operations
//!
//! Handles issue tracking, updates, and PR linking

use crate::errors::{GitHubError, Result};
use crate::models::IssueStatus;
use serde::{Deserialize, Serialize};

/// Comment to post on an issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueComment {
    /// Comment body
    pub body: String,
    /// Comment ID (if existing)
    pub id: Option<u64>,
}

/// Status change for an issue
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatusChange {
    /// Open the issue
    Open,
    /// Mark as in progress
    InProgress,
    /// Close the issue
    Close,
}

/// PR linking information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrLink {
    /// PR number
    pub pr_number: u32,
    /// PR title
    pub pr_title: String,
    /// Link type (closes, relates to, etc.)
    pub link_type: String,
}

/// Issue Operations for tracking and updates
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct IssueOperations {
    /// GitHub token for API access
    token: String,
    /// Owner of the repository
    owner: String,
    /// Repository name
    repo: String,
}

impl IssueOperations {
    /// Create a new IssueOperations
    pub fn new(token: String, owner: String, repo: String) -> Self {
        IssueOperations { token, owner, repo }
    }

    /// Create a comment for posting to an issue
    pub fn create_comment(&self, body: String) -> IssueComment {
        IssueComment { body, id: None }
    }

    /// Format a progress comment
    pub fn format_progress_comment(
        &self,
        current_step: &str,
        total_steps: u32,
        completed_steps: u32,
        details: &str,
    ) -> IssueComment {
        let progress_percentage = if total_steps > 0 {
            (completed_steps as f32 / total_steps as f32 * 100.0) as u32
        } else {
            0
        };

        let progress_bar = self.create_progress_bar(progress_percentage);

        let body = format!(
            "## 🔄 Progress Update\n\n\
             **Current Step:** {}\n\
             **Progress:** {} ({}/{})\n\
             **Status Bar:** {}\n\n\
             ### Details\n\
             {}\n\n\
             _Last updated: {}_",
            current_step,
            progress_percentage,
            completed_steps,
            total_steps,
            progress_bar,
            details,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        self.create_comment(body)
    }

    /// Format a status change comment
    pub fn format_status_change_comment(&self, old_status: IssueStatus, new_status: IssueStatus) -> IssueComment {
        let status_emoji = match new_status {
            IssueStatus::Open => "🔴",
            IssueStatus::InProgress => "🟡",
            IssueStatus::Closed => "🟢",
        };

        let body = format!(
            "{} **Status Changed**\n\n\
             **From:** {:?}\n\
             **To:** {:?}\n\n\
             _Updated at: {}_",
            status_emoji,
            old_status,
            new_status,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        self.create_comment(body)
    }

    /// Format a PR linking comment
    pub fn format_pr_link_comment(&self, pr_link: &PrLink) -> IssueComment {
        let body = format!(
            "## 🔗 PR Linked\n\n\
             **PR:** #{} - {}\n\
             **Link Type:** {}\n\n\
             This issue is now being addressed by the linked pull request.\n\n\
             _Linked at: {}_",
            pr_link.pr_number,
            pr_link.pr_title,
            pr_link.link_type,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        self.create_comment(body)
    }

    /// Create a PR closure link
    pub fn create_pr_closure_link(&self, pr_number: u32, pr_title: String) -> PrLink {
        PrLink {
            pr_number,
            pr_title,
            link_type: "closes".to_string(),
        }
    }

    /// Create a PR relation link
    pub fn create_pr_relation_link(&self, pr_number: u32, pr_title: String) -> PrLink {
        PrLink {
            pr_number,
            pr_title,
            link_type: "relates to".to_string(),
        }
    }

    /// Format a closure message for PR body
    pub fn format_closure_message(&self, issue_number: u32) -> String {
        format!(
            "Closes #{}\n\nThis PR resolves the issue by implementing the required changes.",
            issue_number
        )
    }

    /// Validate a comment
    pub fn validate_comment(&self, comment: &IssueComment) -> Result<()> {
        if comment.body.is_empty() {
            return Err(GitHubError::invalid_input("Comment body cannot be empty"));
        }

        if comment.body.len() > 65536 {
            return Err(GitHubError::invalid_input(
                "Comment body exceeds maximum length of 65536 characters",
            ));
        }

        Ok(())
    }

    /// Create a progress bar string
    fn create_progress_bar(&self, percentage: u32) -> String {
        let filled = (percentage / 10) as usize;
        let empty = 10 - filled;
        format!(
            "[{}{}] {}%",
            "█".repeat(filled),
            "░".repeat(empty),
            percentage
        )
    }

    /// Extract issue number from a closure message
    pub fn extract_issue_number_from_closure(&self, message: &str) -> Result<u32> {
        use regex::Regex;

        let pattern = Regex::new(r"[Cc]loses\s+#(\d+)")
            .map_err(|e| GitHubError::invalid_input(format!("Regex error: {}", e)))?;

        pattern
            .captures(message)
            .and_then(|cap| cap.get(1))
            .and_then(|m| m.as_str().parse::<u32>().ok())
            .ok_or_else(|| {
                GitHubError::invalid_input("No issue number found in closure message")
            })
    }

    /// Post a comment to an issue using the GitHub API
    pub async fn post_comment_to_issue(
        &self,
        issue_number: u32,
        comment: &IssueComment,
    ) -> Result<u64> {
        self.validate_comment(comment)?;

        let client = octocrab::OctocrabBuilder::new()
            .personal_token(self.token.clone())
            .build()
            .map_err(|e| GitHubError::api_error(format!("Failed to create client: {}", e)))?;

        let response = client
            .issues(&self.owner, &self.repo)
            .create_comment(issue_number as u64, &comment.body)
            .await
            .map_err(|e| GitHubError::api_error(format!("Failed to post comment: {}", e)))?;

        Ok(response.id.0)
    }

    /// Update an existing comment on an issue
    pub async fn update_comment_on_issue(
        &self,
        comment_id: u64,
        new_body: &str,
    ) -> Result<()> {
        if new_body.is_empty() {
            return Err(GitHubError::invalid_input("Comment body cannot be empty"));
        }

        let client = octocrab::OctocrabBuilder::new()
            .personal_token(self.token.clone())
            .build()
            .map_err(|e| GitHubError::api_error(format!("Failed to create client: {}", e)))?;

        client
            .issues(&self.owner, &self.repo)
            .update_comment(octocrab::models::CommentId(comment_id), new_body)
            .await
            .map_err(|e| GitHubError::api_error(format!("Failed to update comment: {}", e)))?;

        Ok(())
    }

    /// Link a PR to close an issue
    pub async fn link_pr_to_close_issue(
        &self,
        issue_number: u32,
        pr_number: u32,
        pr_title: &str,
    ) -> Result<u64> {
        let link = self.create_pr_closure_link(pr_number, pr_title.to_string());
        let comment = self.format_pr_link_comment(&link);
        self.post_comment_to_issue(issue_number, &comment).await
    }

    /// Link a PR to relate to an issue
    pub async fn link_pr_to_relate_issue(
        &self,
        issue_number: u32,
        pr_number: u32,
        pr_title: &str,
    ) -> Result<u64> {
        let link = self.create_pr_relation_link(pr_number, pr_title.to_string());
        let comment = self.format_pr_link_comment(&link);
        self.post_comment_to_issue(issue_number, &comment).await
    }

    /// Post a progress update comment to an issue
    pub async fn post_progress_update(
        &self,
        issue_number: u32,
        current_step: &str,
        total_steps: u32,
        completed_steps: u32,
        details: &str,
    ) -> Result<u64> {
        let comment = self.format_progress_comment(current_step, total_steps, completed_steps, details);
        self.post_comment_to_issue(issue_number, &comment).await
    }

    /// Post a status change comment to an issue
    pub async fn post_status_change(
        &self,
        issue_number: u32,
        old_status: IssueStatus,
        new_status: IssueStatus,
    ) -> Result<u64> {
        let comment = self.format_status_change_comment(old_status, new_status);
        self.post_comment_to_issue(issue_number, &comment).await
    }

    /// Update issue status (open, in progress, closed)
    pub async fn update_issue_status(
        &self,
        issue_number: u32,
        new_status: IssueStatus,
    ) -> Result<()> {
        let client = octocrab::OctocrabBuilder::new()
            .personal_token(self.token.clone())
            .build()
            .map_err(|e| GitHubError::api_error(format!("Failed to create client: {}", e)))?;

        let state = match new_status {
            IssueStatus::Open => octocrab::models::IssueState::Open,
            IssueStatus::InProgress => octocrab::models::IssueState::Open, // GitHub doesn't have "in progress" state, use open with label
            IssueStatus::Closed => octocrab::models::IssueState::Closed,
        };

        client
            .issues(&self.owner, &self.repo)
            .update(issue_number as u64)
            .state(state)
            .send()
            .await
            .map_err(|e| GitHubError::api_error(format!("Failed to update issue status: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_operations() -> IssueOperations {
        IssueOperations::new(
            "test_token".to_string(),
            "test_owner".to_string(),
            "test_repo".to_string(),
        )
    }

    #[test]
    fn test_create_comment() {
        let ops = create_test_operations();
        let comment = ops.create_comment("Test comment".to_string());
        assert_eq!(comment.body, "Test comment");
        assert_eq!(comment.id, None);
    }

    #[test]
    fn test_format_progress_comment() {
        let ops = create_test_operations();
        let comment = ops.format_progress_comment("Step 1", 5, 2, "Working on implementation");
        assert!(comment.body.contains("Progress Update"));
        assert!(comment.body.contains("Step 1"));
        assert!(comment.body.contains("2/5"));
    }

    #[test]
    fn test_format_status_change_comment() {
        let ops = create_test_operations();
        let comment = ops.format_status_change_comment(IssueStatus::Open, IssueStatus::InProgress);
        assert!(comment.body.contains("Status Changed"));
        assert!(comment.body.contains("InProgress"));
    }

    #[test]
    fn test_format_pr_link_comment() {
        let ops = create_test_operations();
        let pr_link = PrLink {
            pr_number: 42,
            pr_title: "Implement feature".to_string(),
            link_type: "closes".to_string(),
        };
        let comment = ops.format_pr_link_comment(&pr_link);
        assert!(comment.body.contains("PR Linked"));
        assert!(comment.body.contains("#42"));
    }

    #[test]
    fn test_create_pr_closure_link() {
        let ops = create_test_operations();
        let link = ops.create_pr_closure_link(42, "Implement feature".to_string());
        assert_eq!(link.pr_number, 42);
        assert_eq!(link.link_type, "closes");
    }

    #[test]
    fn test_validate_comment_valid() {
        let ops = create_test_operations();
        let comment = IssueComment {
            body: "Valid comment".to_string(),
            id: None,
        };
        assert!(ops.validate_comment(&comment).is_ok());
    }

    #[test]
    fn test_validate_comment_empty() {
        let ops = create_test_operations();
        let comment = IssueComment {
            body: "".to_string(),
            id: None,
        };
        assert!(ops.validate_comment(&comment).is_err());
    }

    #[test]
    fn test_extract_issue_number_from_closure() {
        let ops = create_test_operations();
        let message = "Closes #123";
        assert_eq!(ops.extract_issue_number_from_closure(message).unwrap(), 123);
    }

    #[test]
    fn test_extract_issue_number_case_insensitive() {
        let ops = create_test_operations();
        let message = "closes #456";
        assert_eq!(ops.extract_issue_number_from_closure(message).unwrap(), 456);
    }

    #[test]
    fn test_link_pr_closure_link_format() {
        let ops = create_test_operations();
        let link = ops.create_pr_closure_link(42, "Implement feature".to_string());
        assert_eq!(link.pr_number, 42);
        assert_eq!(link.link_type, "closes");
        assert_eq!(link.pr_title, "Implement feature");
    }

    #[test]
    fn test_link_pr_relation_link_format() {
        let ops = create_test_operations();
        let link = ops.create_pr_relation_link(42, "Related work".to_string());
        assert_eq!(link.pr_number, 42);
        assert_eq!(link.link_type, "relates to");
        assert_eq!(link.pr_title, "Related work");
    }

    #[test]
    fn test_format_closure_message() {
        let ops = create_test_operations();
        let message = ops.format_closure_message(123);
        assert!(message.contains("Closes #123"));
        assert!(message.contains("resolves the issue"));
    }
}
