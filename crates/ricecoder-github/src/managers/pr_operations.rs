//! PR Operations - Handles PR updates, comments, and reviews

use crate::errors::{GitHubError, Result};
use crate::models::{PullRequest, PrStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// PR comment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrComment {
    /// Comment ID
    pub id: u64,
    /// Comment body
    pub body: String,
    /// Author
    pub author: String,
    /// Created at timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Updated at timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl PrComment {
    /// Create a new PR comment
    pub fn new(body: impl Into<String>, author: impl Into<String>) -> Self {
        Self {
            id: 0,
            body: body.into(),
            author: author.into(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }
}

/// PR review state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ReviewState {
    /// Approved
    Approved,
    /// Changes requested
    ChangesRequested,
    /// Commented
    Commented,
    /// Dismissed
    Dismissed,
    /// Pending
    Pending,
}

/// PR review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrReview {
    /// Review ID
    pub id: u64,
    /// Reviewer
    pub reviewer: String,
    /// Review state
    pub state: ReviewState,
    /// Review body
    pub body: String,
    /// Created at timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl PrReview {
    /// Create a new PR review
    pub fn new(reviewer: impl Into<String>, state: ReviewState, body: impl Into<String>) -> Self {
        Self {
            id: 0,
            reviewer: reviewer.into(),
            state,
            body: body.into(),
            created_at: chrono::Utc::now(),
        }
    }

    /// Create an approval review
    pub fn approval(reviewer: impl Into<String>) -> Self {
        Self::new(reviewer, ReviewState::Approved, "Approved")
    }

    /// Create a changes requested review
    pub fn changes_requested(reviewer: impl Into<String>, body: impl Into<String>) -> Self {
        Self::new(reviewer, ReviewState::ChangesRequested, body)
    }

    /// Create a comment review
    pub fn comment(reviewer: impl Into<String>, body: impl Into<String>) -> Self {
        Self::new(reviewer, ReviewState::Commented, body)
    }
}

/// PR update options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrUpdateOptions {
    /// New title (optional)
    pub title: Option<String>,
    /// New body (optional)
    pub body: Option<String>,
    /// New state (optional)
    pub state: Option<PrStatus>,
    /// Draft status (optional)
    pub draft: Option<bool>,
}

impl Default for PrUpdateOptions {
    fn default() -> Self {
        Self {
            title: None,
            body: None,
            state: None,
            draft: None,
        }
    }
}

impl PrUpdateOptions {
    /// Create new update options
    pub fn new() -> Self {
        Self::default()
    }

    /// Set title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set body
    pub fn with_body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    /// Set state
    pub fn with_state(mut self, state: PrStatus) -> Self {
        self.state = Some(state);
        self
    }

    /// Set draft status
    pub fn with_draft(mut self, draft: bool) -> Self {
        self.draft = Some(draft);
        self
    }

    /// Check if any updates are specified
    pub fn has_updates(&self) -> bool {
        self.title.is_some() || self.body.is_some() || self.state.is_some() || self.draft.is_some()
    }
}

/// Progress update for PR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdate {
    /// Update title
    pub title: String,
    /// Update description
    pub description: String,
    /// Status (e.g., "In Progress", "Completed", "Blocked")
    pub status: String,
    /// Percentage complete (0-100)
    pub progress_percent: u32,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl ProgressUpdate {
    /// Create a new progress update
    pub fn new(title: impl Into<String>, status: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: String::new(),
            status: status.into(),
            progress_percent: 0,
            metadata: HashMap::new(),
        }
    }

    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Set progress percentage
    pub fn with_progress(mut self, percent: u32) -> Self {
        self.progress_percent = percent.min(100);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Format as comment body
    pub fn format_as_comment(&self) -> String {
        let mut comment = format!("## {}\n\n", self.title);
        comment.push_str(&format!("**Status**: {}\n", self.status));
        comment.push_str(&format!("**Progress**: {}%\n\n", self.progress_percent));

        if !self.description.is_empty() {
            comment.push_str(&format!("{}\n\n", self.description));
        }

        if !self.metadata.is_empty() {
            comment.push_str("### Details\n\n");
            for (key, value) in &self.metadata {
                comment.push_str(&format!("- **{}**: {}\n", key, value));
            }
        }

        comment
    }
}

/// PR Operations Manager
pub struct PrOperations;

impl PrOperations {
    /// Update PR with new information
    pub fn update_pr(pr: &mut PullRequest, options: PrUpdateOptions) -> Result<()> {
        if !options.has_updates() {
            return Ok(());
        }

        debug!("Updating PR with new information");

        if let Some(title) = options.title {
            if title.is_empty() {
                return Err(GitHubError::invalid_input("PR title cannot be empty"));
            }
            pr.title = title;
        }

        if let Some(body) = options.body {
            if body.is_empty() {
                return Err(GitHubError::invalid_input("PR body cannot be empty"));
            }
            pr.body = body;
        }

        if let Some(state) = options.state {
            pr.status = state;
        }

        if let Some(draft) = options.draft {
            pr.status = if draft { PrStatus::Draft } else { PrStatus::Open };
        }

        pr.updated_at = chrono::Utc::now();

        info!(
            pr_number = pr.number,
            "PR updated successfully"
        );

        Ok(())
    }

    /// Add a comment to PR
    pub fn add_comment(pr: &mut PullRequest, comment: PrComment) -> Result<()> {
        if comment.body.is_empty() {
            return Err(GitHubError::invalid_input("Comment body cannot be empty"));
        }

        debug!(
            pr_number = pr.number,
            comment_author = %comment.author,
            "Adding comment to PR"
        );

        // Append comment to PR body (in real implementation, this would be a separate API call)
        pr.body.push_str("\n\n---\n");
        pr.body.push_str(&format!("**Comment from {}**:\n\n{}", comment.author, comment.body));

        info!(
            pr_number = pr.number,
            "Comment added to PR"
        );

        Ok(())
    }

    /// Add a progress update comment to PR
    pub fn add_progress_update(pr: &mut PullRequest, update: ProgressUpdate) -> Result<()> {
        let comment_body = update.format_as_comment();
        let comment = PrComment::new(comment_body, "ricecoder-bot");
        Self::add_comment(pr, comment)
    }

    /// Add a review to PR
    pub fn add_review(pr: &mut PullRequest, review: PrReview) -> Result<()> {
        if review.body.is_empty() && review.state != ReviewState::Approved {
            return Err(GitHubError::invalid_input("Review body cannot be empty"));
        }

        debug!(
            pr_number = pr.number,
            reviewer = %review.reviewer,
            state = ?review.state,
            "Adding review to PR"
        );

        // Append review to PR body (in real implementation, this would be a separate API call)
        let state_str = match review.state {
            ReviewState::Approved => "✅ Approved",
            ReviewState::ChangesRequested => "❌ Changes Requested",
            ReviewState::Commented => "💬 Commented",
            ReviewState::Dismissed => "🚫 Dismissed",
            ReviewState::Pending => "⏳ Pending",
        };

        pr.body.push_str("\n\n---\n");
        pr.body.push_str(&format!(
            "**Review from {} ({})**:\n\n{}",
            review.reviewer, state_str, review.body
        ));

        info!(
            pr_number = pr.number,
            "Review added to PR"
        );

        Ok(())
    }

    /// Validate PR update options
    pub fn validate_update_options(options: &PrUpdateOptions) -> Result<()> {
        if let Some(title) = &options.title {
            if title.is_empty() {
                return Err(GitHubError::invalid_input("PR title cannot be empty"));
            }
            if title.len() > 256 {
                return Err(GitHubError::invalid_input(
                    "PR title cannot exceed 256 characters",
                ));
            }
        }

        if let Some(body) = &options.body {
            if body.is_empty() {
                return Err(GitHubError::invalid_input("PR body cannot be empty"));
            }
        }

        Ok(())
    }

    /// Validate comment
    pub fn validate_comment(comment: &PrComment) -> Result<()> {
        if comment.body.is_empty() {
            return Err(GitHubError::invalid_input("Comment body cannot be empty"));
        }

        if comment.author.is_empty() {
            return Err(GitHubError::invalid_input("Comment author cannot be empty"));
        }

        Ok(())
    }

    /// Validate review
    pub fn validate_review(review: &PrReview) -> Result<()> {
        if review.reviewer.is_empty() {
            return Err(GitHubError::invalid_input("Reviewer cannot be empty"));
        }

        if review.body.is_empty() && review.state != ReviewState::Approved {
            return Err(GitHubError::invalid_input("Review body cannot be empty"));
        }

        Ok(())
    }

    /// Check if PR can be approved
    pub fn can_approve(pr: &PullRequest) -> bool {
        matches!(pr.status, PrStatus::Open | PrStatus::Draft)
    }

    /// Check if PR can be merged
    pub fn can_merge(pr: &PullRequest) -> bool {
        matches!(pr.status, PrStatus::Open)
    }

    /// Check if PR can be closed
    pub fn can_close(pr: &PullRequest) -> bool {
        matches!(pr.status, PrStatus::Open | PrStatus::Draft)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pr_comment_creation() {
        let comment = PrComment::new("This is a comment", "user123");
        assert_eq!(comment.body, "This is a comment");
        assert_eq!(comment.author, "user123");
    }

    #[test]
    fn test_pr_review_approval() {
        let review = PrReview::approval("reviewer1");
        assert_eq!(review.reviewer, "reviewer1");
        assert_eq!(review.state, ReviewState::Approved);
    }

    #[test]
    fn test_pr_review_changes_requested() {
        let review = PrReview::changes_requested("reviewer1", "Please fix this");
        assert_eq!(review.state, ReviewState::ChangesRequested);
        assert_eq!(review.body, "Please fix this");
    }

    #[test]
    fn test_pr_update_options_creation() {
        let options = PrUpdateOptions::new();
        assert!(!options.has_updates());
    }

    #[test]
    fn test_pr_update_options_with_title() {
        let options = PrUpdateOptions::new().with_title("New Title");
        assert!(options.has_updates());
        assert_eq!(options.title, Some("New Title".to_string()));
    }

    #[test]
    fn test_pr_update_options_with_draft() {
        let options = PrUpdateOptions::new().with_draft(true);
        assert!(options.has_updates());
        assert_eq!(options.draft, Some(true));
    }

    #[test]
    fn test_progress_update_creation() {
        let update = ProgressUpdate::new("Task 1", "In Progress");
        assert_eq!(update.title, "Task 1");
        assert_eq!(update.status, "In Progress");
        assert_eq!(update.progress_percent, 0);
    }

    #[test]
    fn test_progress_update_with_progress() {
        let update = ProgressUpdate::new("Task 1", "In Progress")
            .with_progress(50);
        assert_eq!(update.progress_percent, 50);
    }

    #[test]
    fn test_progress_update_with_progress_capped() {
        let update = ProgressUpdate::new("Task 1", "In Progress")
            .with_progress(150);
        assert_eq!(update.progress_percent, 100);
    }

    #[test]
    fn test_progress_update_format_as_comment() {
        let update = ProgressUpdate::new("Task 1", "In Progress")
            .with_progress(50)
            .with_description("Working on implementation");
        let comment = update.format_as_comment();
        assert!(comment.contains("Task 1"));
        assert!(comment.contains("In Progress"));
        assert!(comment.contains("50%"));
        assert!(comment.contains("Working on implementation"));
    }

    #[test]
    fn test_update_pr_title() {
        let mut pr = PullRequest {
            id: 1,
            number: 1,
            title: "Old Title".to_string(),
            body: "Body".to_string(),
            branch: "feature/test".to_string(),
            base: "main".to_string(),
            status: PrStatus::Open,
            files: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let options = PrUpdateOptions::new().with_title("New Title");
        PrOperations::update_pr(&mut pr, options).unwrap();
        assert_eq!(pr.title, "New Title");
    }

    #[test]
    fn test_update_pr_body() {
        let mut pr = PullRequest {
            id: 1,
            number: 1,
            title: "Title".to_string(),
            body: "Old Body".to_string(),
            branch: "feature/test".to_string(),
            base: "main".to_string(),
            status: PrStatus::Open,
            files: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let options = PrUpdateOptions::new().with_body("New Body");
        PrOperations::update_pr(&mut pr, options).unwrap();
        assert_eq!(pr.body, "New Body");
    }

    #[test]
    fn test_update_pr_state() {
        let mut pr = PullRequest {
            id: 1,
            number: 1,
            title: "Title".to_string(),
            body: "Body".to_string(),
            branch: "feature/test".to_string(),
            base: "main".to_string(),
            status: PrStatus::Open,
            files: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let options = PrUpdateOptions::new().with_state(PrStatus::Closed);
        PrOperations::update_pr(&mut pr, options).unwrap();
        assert_eq!(pr.status, PrStatus::Closed);
    }

    #[test]
    fn test_add_comment() {
        let mut pr = PullRequest {
            id: 1,
            number: 1,
            title: "Title".to_string(),
            body: "Body".to_string(),
            branch: "feature/test".to_string(),
            base: "main".to_string(),
            status: PrStatus::Open,
            files: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let comment = PrComment::new("This is a comment", "user1");
        PrOperations::add_comment(&mut pr, comment).unwrap();
        assert!(pr.body.contains("This is a comment"));
        assert!(pr.body.contains("user1"));
    }

    #[test]
    fn test_add_progress_update() {
        let mut pr = PullRequest {
            id: 1,
            number: 1,
            title: "Title".to_string(),
            body: "Body".to_string(),
            branch: "feature/test".to_string(),
            base: "main".to_string(),
            status: PrStatus::Open,
            files: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let update = ProgressUpdate::new("Task 1", "In Progress")
            .with_progress(50);
        PrOperations::add_progress_update(&mut pr, update).unwrap();
        assert!(pr.body.contains("Task 1"));
        assert!(pr.body.contains("50%"));
    }

    #[test]
    fn test_add_review() {
        let mut pr = PullRequest {
            id: 1,
            number: 1,
            title: "Title".to_string(),
            body: "Body".to_string(),
            branch: "feature/test".to_string(),
            base: "main".to_string(),
            status: PrStatus::Open,
            files: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let review = PrReview::approval("reviewer1");
        PrOperations::add_review(&mut pr, review).unwrap();
        assert!(pr.body.contains("reviewer1"));
        assert!(pr.body.contains("Approved"));
    }

    #[test]
    fn test_validate_update_options_empty_title() {
        let options = PrUpdateOptions::new().with_title("");
        assert!(PrOperations::validate_update_options(&options).is_err());
    }

    #[test]
    fn test_validate_update_options_title_too_long() {
        let long_title = "a".repeat(300);
        let options = PrUpdateOptions::new().with_title(long_title);
        assert!(PrOperations::validate_update_options(&options).is_err());
    }

    #[test]
    fn test_validate_comment_empty_body() {
        let comment = PrComment {
            id: 0,
            body: String::new(),
            author: "user1".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        assert!(PrOperations::validate_comment(&comment).is_err());
    }

    #[test]
    fn test_validate_review_empty_body_with_changes_requested() {
        let review = PrReview {
            id: 0,
            reviewer: "reviewer1".to_string(),
            state: ReviewState::ChangesRequested,
            body: String::new(),
            created_at: chrono::Utc::now(),
        };
        assert!(PrOperations::validate_review(&review).is_err());
    }

    #[test]
    fn test_validate_review_empty_body_with_approval() {
        let review = PrReview {
            id: 0,
            reviewer: "reviewer1".to_string(),
            state: ReviewState::Approved,
            body: String::new(),
            created_at: chrono::Utc::now(),
        };
        assert!(PrOperations::validate_review(&review).is_ok());
    }

    #[test]
    fn test_can_approve_open_pr() {
        let pr = PullRequest {
            id: 1,
            number: 1,
            title: "Title".to_string(),
            body: "Body".to_string(),
            branch: "feature/test".to_string(),
            base: "main".to_string(),
            status: PrStatus::Open,
            files: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        assert!(PrOperations::can_approve(&pr));
    }

    #[test]
    fn test_can_approve_merged_pr() {
        let pr = PullRequest {
            id: 1,
            number: 1,
            title: "Title".to_string(),
            body: "Body".to_string(),
            branch: "feature/test".to_string(),
            base: "main".to_string(),
            status: PrStatus::Merged,
            files: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        assert!(!PrOperations::can_approve(&pr));
    }

    #[test]
    fn test_can_merge_open_pr() {
        let pr = PullRequest {
            id: 1,
            number: 1,
            title: "Title".to_string(),
            body: "Body".to_string(),
            branch: "feature/test".to_string(),
            base: "main".to_string(),
            status: PrStatus::Open,
            files: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        assert!(PrOperations::can_merge(&pr));
    }

    #[test]
    fn test_can_merge_draft_pr() {
        let pr = PullRequest {
            id: 1,
            number: 1,
            title: "Title".to_string(),
            body: "Body".to_string(),
            branch: "feature/test".to_string(),
            base: "main".to_string(),
            status: PrStatus::Draft,
            files: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        assert!(!PrOperations::can_merge(&pr));
    }

    #[test]
    fn test_can_close_open_pr() {
        let pr = PullRequest {
            id: 1,
            number: 1,
            title: "Title".to_string(),
            body: "Body".to_string(),
            branch: "feature/test".to_string(),
            base: "main".to_string(),
            status: PrStatus::Open,
            files: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        assert!(PrOperations::can_close(&pr));
    }

    #[test]
    fn test_can_close_merged_pr() {
        let pr = PullRequest {
            id: 1,
            number: 1,
            title: "Title".to_string(),
            body: "Body".to_string(),
            branch: "feature/test".to_string(),
            base: "main".to_string(),
            status: PrStatus::Merged,
            files: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        assert!(!PrOperations::can_close(&pr));
    }
}
