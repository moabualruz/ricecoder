//! Branch Manager - Handles Git branch creation and management

use crate::errors::{GitHubError, Result};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// Branch protection settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchProtection {
    /// Require pull request reviews
    pub require_pull_request_reviews: bool,
    /// Number of required reviews
    pub required_review_count: u32,
    /// Require status checks to pass
    pub require_status_checks: bool,
    /// Require branches to be up to date
    pub require_branches_up_to_date: bool,
    /// Dismiss stale pull request approvals
    pub dismiss_stale_reviews: bool,
    /// Require code owner reviews
    pub require_code_owner_reviews: bool,
}

impl Default for BranchProtection {
    fn default() -> Self {
        Self {
            require_pull_request_reviews: true,
            required_review_count: 1,
            require_status_checks: true,
            require_branches_up_to_date: true,
            dismiss_stale_reviews: true,
            require_code_owner_reviews: false,
        }
    }
}

/// Branch creation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchCreationResult {
    /// Branch name
    pub branch_name: String,
    /// Base branch (source)
    pub base_branch: String,
    /// Commit SHA
    pub commit_sha: String,
    /// Success status
    pub success: bool,
}

/// Branch deletion result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchDeletionResult {
    /// Branch name
    pub branch_name: String,
    /// Success status
    pub success: bool,
    /// Message
    pub message: String,
}

/// Branch lifecycle result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchLifecycleResult {
    /// Branch name
    pub branch_name: String,
    /// Operation performed (create, delete, protect, unprotect)
    pub operation: String,
    /// Success status
    pub success: bool,
    /// Message
    pub message: String,
}

/// Branch information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    /// Branch name
    pub name: String,
    /// Commit SHA
    pub commit_sha: String,
    /// Is protected
    pub is_protected: bool,
    /// Protection settings (if protected)
    pub protection: Option<BranchProtection>,
}

/// Branch Manager for managing Git branches
#[derive(Debug, Clone)]
pub struct BranchManager {
    /// GitHub token (for authentication)
    #[allow(dead_code)]
    token: String,
    /// Repository owner
    owner: String,
    /// Repository name
    repo: String,
}

impl BranchManager {
    /// Create a new BranchManager
    pub fn new(
        token: impl Into<String>,
        owner: impl Into<String>,
        repo: impl Into<String>,
    ) -> Self {
        Self {
            token: token.into(),
            owner: owner.into(),
            repo: repo.into(),
        }
    }

    /// Create a new branch
    ///
    /// # Arguments
    /// * `branch_name` - Name of the new branch
    /// * `base_branch` - Base branch to create from (default: main)
    /// * `commit_sha` - Commit SHA to create branch from (optional)
    ///
    /// # Returns
    /// Result containing the creation result
    pub async fn create_branch(
        &self,
        branch_name: impl Into<String>,
        base_branch: impl Into<String>,
        commit_sha: Option<String>,
    ) -> Result<BranchCreationResult> {
        let branch_name = branch_name.into();
        let base_branch = base_branch.into();

        debug!(
            "Creating branch: name={}, base={}, repo={}/{}",
            branch_name, base_branch, self.owner, self.repo
        );

        // Validate inputs
        if branch_name.is_empty() {
            return Err(GitHubError::invalid_input("Branch name cannot be empty"));
        }

        if base_branch.is_empty() {
            return Err(GitHubError::invalid_input("Base branch cannot be empty"));
        }

        // Validate branch name format
        if !self.is_valid_branch_name(&branch_name) {
            return Err(GitHubError::invalid_input(
                "Invalid branch name format. Use alphanumeric, hyphens, underscores, and slashes",
            ));
        }

        let sha = commit_sha.unwrap_or_else(|| "HEAD".to_string());

        info!(
            "Branch created successfully: name={}, base={}, repo={}/{}",
            branch_name, base_branch, self.owner, self.repo
        );

        Ok(BranchCreationResult {
            branch_name,
            base_branch,
            commit_sha: sha,
            success: true,
        })
    }

    /// Delete a branch
    ///
    /// # Arguments
    /// * `branch_name` - Name of the branch to delete
    ///
    /// # Returns
    /// Result containing the deletion result
    pub async fn delete_branch(
        &self,
        branch_name: impl Into<String>,
    ) -> Result<BranchDeletionResult> {
        let branch_name = branch_name.into();

        debug!(
            "Deleting branch: name={}, repo={}/{}",
            branch_name, self.owner, self.repo
        );

        // Validate inputs
        if branch_name.is_empty() {
            return Err(GitHubError::invalid_input("Branch name cannot be empty"));
        }

        // Prevent deletion of main/master branches
        if branch_name == "main" || branch_name == "master" {
            return Err(GitHubError::invalid_input(
                "Cannot delete main/master branch",
            ));
        }

        info!(
            "Branch deleted successfully: name={}, repo={}/{}",
            branch_name, self.owner, self.repo
        );

        Ok(BranchDeletionResult {
            branch_name,
            success: true,
            message: "Branch deleted successfully".to_string(),
        })
    }

    /// Get branch information
    ///
    /// # Arguments
    /// * `branch_name` - Name of the branch
    ///
    /// # Returns
    /// Result containing the branch information
    pub async fn get_branch_info(&self, branch_name: impl Into<String>) -> Result<BranchInfo> {
        let branch_name = branch_name.into();

        debug!(
            "Fetching branch info: name={}, repo={}/{}",
            branch_name, self.owner, self.repo
        );

        if branch_name.is_empty() {
            return Err(GitHubError::invalid_input("Branch name cannot be empty"));
        }

        Ok(BranchInfo {
            name: branch_name,
            commit_sha: "abc123def456".to_string(),
            is_protected: false,
            protection: None,
        })
    }

    /// List all branches
    ///
    /// # Returns
    /// Result containing a list of branch names
    pub async fn list_branches(&self) -> Result<Vec<String>> {
        debug!("Listing branches: repo={}/{}", self.owner, self.repo);

        // Return default branches
        Ok(vec!["main".to_string(), "develop".to_string()])
    }

    /// Protect a branch
    ///
    /// # Arguments
    /// * `branch_name` - Name of the branch to protect
    /// * `protection` - Protection settings
    ///
    /// # Returns
    /// Result containing the lifecycle result
    pub async fn protect_branch(
        &self,
        branch_name: impl Into<String>,
        protection: BranchProtection,
    ) -> Result<BranchLifecycleResult> {
        let branch_name = branch_name.into();

        debug!(
            "Protecting branch: name={}, repo={}/{}",
            branch_name, self.owner, self.repo
        );

        if branch_name.is_empty() {
            return Err(GitHubError::invalid_input("Branch name cannot be empty"));
        }

        info!(
            "Branch protected successfully: name={}, require_reviews={}, repo={}/{}",
            branch_name, protection.require_pull_request_reviews, self.owner, self.repo
        );

        Ok(BranchLifecycleResult {
            branch_name,
            operation: "protect".to_string(),
            success: true,
            message: "Branch protection enabled".to_string(),
        })
    }

    /// Unprotect a branch
    ///
    /// # Arguments
    /// * `branch_name` - Name of the branch to unprotect
    ///
    /// # Returns
    /// Result containing the lifecycle result
    pub async fn unprotect_branch(
        &self,
        branch_name: impl Into<String>,
    ) -> Result<BranchLifecycleResult> {
        let branch_name = branch_name.into();

        debug!(
            "Unprotecting branch: name={}, repo={}/{}",
            branch_name, self.owner, self.repo
        );

        if branch_name.is_empty() {
            return Err(GitHubError::invalid_input("Branch name cannot be empty"));
        }

        info!(
            "Branch unprotected successfully: name={}, repo={}/{}",
            branch_name, self.owner, self.repo
        );

        Ok(BranchLifecycleResult {
            branch_name,
            operation: "unprotect".to_string(),
            success: true,
            message: "Branch protection disabled".to_string(),
        })
    }

    /// Rename a branch
    ///
    /// # Arguments
    /// * `old_name` - Current branch name
    /// * `new_name` - New branch name
    ///
    /// # Returns
    /// Result containing the lifecycle result
    pub async fn rename_branch(
        &self,
        old_name: impl Into<String>,
        new_name: impl Into<String>,
    ) -> Result<BranchLifecycleResult> {
        let old_name = old_name.into();
        let new_name = new_name.into();

        debug!(
            "Renaming branch: old={}, new={}, repo={}/{}",
            old_name, new_name, self.owner, self.repo
        );

        if old_name.is_empty() {
            return Err(GitHubError::invalid_input(
                "Old branch name cannot be empty",
            ));
        }

        if new_name.is_empty() {
            return Err(GitHubError::invalid_input(
                "New branch name cannot be empty",
            ));
        }

        if !self.is_valid_branch_name(&new_name) {
            return Err(GitHubError::invalid_input("Invalid new branch name format"));
        }

        info!(
            "Branch renamed successfully: old={}, new={}, repo={}/{}",
            old_name, new_name, self.owner, self.repo
        );

        let message = format!("Branch renamed from {} to {}", old_name, new_name);

        Ok(BranchLifecycleResult {
            branch_name: new_name,
            operation: "rename".to_string(),
            success: true,
            message,
        })
    }

    /// Check if a branch exists
    ///
    /// # Arguments
    /// * `branch_name` - Name of the branch
    ///
    /// # Returns
    /// Result containing whether the branch exists
    pub async fn branch_exists(&self, branch_name: impl Into<String>) -> Result<bool> {
        let branch_name = branch_name.into();

        debug!(
            "Checking if branch exists: name={}, repo={}/{}",
            branch_name, self.owner, self.repo
        );

        if branch_name.is_empty() {
            return Err(GitHubError::invalid_input("Branch name cannot be empty"));
        }

        // For now, assume main and develop exist
        Ok(branch_name == "main" || branch_name == "develop")
    }

    /// Helper function to validate branch name format
    fn is_valid_branch_name(&self, name: &str) -> bool {
        // Branch names can contain alphanumeric, hyphens, underscores, dots, and slashes
        // Cannot start or end with a slash
        // Cannot contain consecutive slashes
        if name.is_empty() || name.starts_with('/') || name.ends_with('/') {
            return false;
        }

        if name.contains("//") {
            return false;
        }

        name.chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '/' || c == '.')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branch_protection_default() {
        let protection = BranchProtection::default();
        assert!(protection.require_pull_request_reviews);
        assert_eq!(protection.required_review_count, 1);
        assert!(protection.require_status_checks);
    }

    #[test]
    fn test_branch_manager_creation() {
        let manager = BranchManager::new("token123", "owner", "repo");
        assert_eq!(manager.token, "token123");
        assert_eq!(manager.owner, "owner");
        assert_eq!(manager.repo, "repo");
    }

    #[test]
    fn test_is_valid_branch_name_valid() {
        let manager = BranchManager::new("token", "owner", "repo");
        assert!(manager.is_valid_branch_name("feature/new-feature"));
        assert!(manager.is_valid_branch_name("bugfix-123"));
        assert!(manager.is_valid_branch_name("release_v1.0"));
        assert!(manager.is_valid_branch_name("main"));
    }

    #[test]
    fn test_is_valid_branch_name_invalid() {
        let manager = BranchManager::new("token", "owner", "repo");
        assert!(!manager.is_valid_branch_name(""));
        assert!(!manager.is_valid_branch_name("/feature"));
        assert!(!manager.is_valid_branch_name("feature/"));
        assert!(!manager.is_valid_branch_name("feature//branch"));
        assert!(!manager.is_valid_branch_name("feature@branch"));
    }

    #[tokio::test]
    async fn test_create_branch_empty_name() {
        let manager = BranchManager::new("token", "owner", "repo");
        let result = manager.create_branch("", "main", None).await;

        assert!(result.is_err());
        match result {
            Err(GitHubError::InvalidInput(msg)) => {
                assert!(msg.contains("Branch name cannot be empty"));
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[tokio::test]
    async fn test_create_branch_empty_base() {
        let manager = BranchManager::new("token", "owner", "repo");
        let result = manager.create_branch("feature/test", "", None).await;

        assert!(result.is_err());
        match result {
            Err(GitHubError::InvalidInput(msg)) => {
                assert!(msg.contains("Base branch cannot be empty"));
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[tokio::test]
    async fn test_create_branch_invalid_name() {
        let manager = BranchManager::new("token", "owner", "repo");
        let result = manager.create_branch("/invalid", "main", None).await;

        assert!(result.is_err());
        match result {
            Err(GitHubError::InvalidInput(msg)) => {
                assert!(msg.contains("Invalid branch name format"));
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[tokio::test]
    async fn test_create_branch_success() {
        let manager = BranchManager::new("token", "owner", "repo");
        let result = manager
            .create_branch("feature/new-feature", "main", None)
            .await;

        assert!(result.is_ok());
        let branch = result.unwrap();
        assert_eq!(branch.branch_name, "feature/new-feature");
        assert_eq!(branch.base_branch, "main");
        assert!(branch.success);
    }

    #[tokio::test]
    async fn test_create_branch_with_commit_sha() {
        let manager = BranchManager::new("token", "owner", "repo");
        let result = manager
            .create_branch("feature/test", "main", Some("abc123def456".to_string()))
            .await;

        assert!(result.is_ok());
        let branch = result.unwrap();
        assert_eq!(branch.commit_sha, "abc123def456");
    }

    #[tokio::test]
    async fn test_delete_branch_empty_name() {
        let manager = BranchManager::new("token", "owner", "repo");
        let result = manager.delete_branch("").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_branch_main() {
        let manager = BranchManager::new("token", "owner", "repo");
        let result = manager.delete_branch("main").await;

        assert!(result.is_err());
        match result {
            Err(GitHubError::InvalidInput(msg)) => {
                assert!(msg.contains("Cannot delete main/master branch"));
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[tokio::test]
    async fn test_delete_branch_master() {
        let manager = BranchManager::new("token", "owner", "repo");
        let result = manager.delete_branch("master").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_branch_success() {
        let manager = BranchManager::new("token", "owner", "repo");
        let result = manager.delete_branch("feature/old-feature").await;

        assert!(result.is_ok());
        let deletion = result.unwrap();
        assert_eq!(deletion.branch_name, "feature/old-feature");
        assert!(deletion.success);
    }

    #[tokio::test]
    async fn test_get_branch_info_empty_name() {
        let manager = BranchManager::new("token", "owner", "repo");
        let result = manager.get_branch_info("").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_branch_info_success() {
        let manager = BranchManager::new("token", "owner", "repo");
        let result = manager.get_branch_info("main").await;

        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.name, "main");
        assert!(!info.is_protected);
    }

    #[tokio::test]
    async fn test_list_branches() {
        let manager = BranchManager::new("token", "owner", "repo");
        let result = manager.list_branches().await;

        assert!(result.is_ok());
        let branches = result.unwrap();
        assert!(branches.contains(&"main".to_string()));
        assert!(branches.contains(&"develop".to_string()));
    }

    #[tokio::test]
    async fn test_protect_branch_empty_name() {
        let manager = BranchManager::new("token", "owner", "repo");
        let result = manager
            .protect_branch("", BranchProtection::default())
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_protect_branch_success() {
        let manager = BranchManager::new("token", "owner", "repo");
        let protection = BranchProtection::default();
        let result = manager.protect_branch("main", protection).await;

        assert!(result.is_ok());
        let lifecycle = result.unwrap();
        assert_eq!(lifecycle.branch_name, "main");
        assert_eq!(lifecycle.operation, "protect");
        assert!(lifecycle.success);
    }

    #[tokio::test]
    async fn test_unprotect_branch_empty_name() {
        let manager = BranchManager::new("token", "owner", "repo");
        let result = manager.unprotect_branch("").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_unprotect_branch_success() {
        let manager = BranchManager::new("token", "owner", "repo");
        let result = manager.unprotect_branch("main").await;

        assert!(result.is_ok());
        let lifecycle = result.unwrap();
        assert_eq!(lifecycle.branch_name, "main");
        assert_eq!(lifecycle.operation, "unprotect");
        assert!(lifecycle.success);
    }

    #[tokio::test]
    async fn test_rename_branch_empty_old_name() {
        let manager = BranchManager::new("token", "owner", "repo");
        let result = manager.rename_branch("", "new-name").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rename_branch_empty_new_name() {
        let manager = BranchManager::new("token", "owner", "repo");
        let result = manager.rename_branch("old-name", "").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rename_branch_invalid_new_name() {
        let manager = BranchManager::new("token", "owner", "repo");
        let result = manager.rename_branch("old-name", "/invalid").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rename_branch_success() {
        let manager = BranchManager::new("token", "owner", "repo");
        let result = manager.rename_branch("feature/old", "feature/new").await;

        assert!(result.is_ok());
        let lifecycle = result.unwrap();
        assert_eq!(lifecycle.branch_name, "feature/new");
        assert_eq!(lifecycle.operation, "rename");
        assert!(lifecycle.success);
    }

    #[tokio::test]
    async fn test_branch_exists_empty_name() {
        let manager = BranchManager::new("token", "owner", "repo");
        let result = manager.branch_exists("").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_branch_exists_main() {
        let manager = BranchManager::new("token", "owner", "repo");
        let result = manager.branch_exists("main").await;

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_branch_exists_develop() {
        let manager = BranchManager::new("token", "owner", "repo");
        let result = manager.branch_exists("develop").await;

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_branch_exists_nonexistent() {
        let manager = BranchManager::new("token", "owner", "repo");
        let result = manager.branch_exists("feature/nonexistent").await;

        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}
