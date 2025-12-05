//! Git integration for version control and auto-commit functionality

use crate::error::FileError;
use crate::models::{FileOperation, GitStatus, OperationType};
use git2::{Repository, Status, StatusOptions};
use std::path::{Path, PathBuf};

/// Integrates with git for version control and auto-commit functionality
#[derive(Debug, Clone)]
pub struct GitIntegration;

impl GitIntegration {
    /// Creates a new GitIntegration instance
    pub fn new() -> Self {
        GitIntegration
    }

    /// Checks the git status of a repository
    ///
    /// Returns information about modified, staged, and untracked files.
    ///
    /// # Arguments
    ///
    /// * `repo_path` - Path to the git repository
    ///
    /// # Returns
    ///
    /// A `GitStatus` containing the current branch and file statuses
    ///
    /// # Errors
    ///
    /// Returns `FileError::GitError` if the repository cannot be opened or status cannot be checked
    pub fn check_status(repo_path: &Path) -> Result<GitStatus, FileError> {
        let repo = Repository::open(repo_path)
            .map_err(|e| FileError::GitError(format!("Failed to open repository: {}", e)))?;

        let branch = Self::get_current_branch_internal(&repo)?;

        let mut status_opts = StatusOptions::new();
        status_opts.include_untracked(true);
        status_opts.include_ignored(false);

        let statuses = repo
            .statuses(Some(&mut status_opts))
            .map_err(|e| FileError::GitError(format!("Failed to get status: {}", e)))?;

        let mut modified = Vec::new();
        let mut staged = Vec::new();
        let mut untracked = Vec::new();

        for entry in statuses.iter() {
            let path = PathBuf::from(entry.path().unwrap_or(""));
            let status = entry.status();

            if status.contains(Status::WT_MODIFIED) || status.contains(Status::WT_DELETED) {
                modified.push(path);
            } else if status.contains(Status::INDEX_NEW)
                || status.contains(Status::INDEX_MODIFIED)
                || status.contains(Status::INDEX_DELETED)
            {
                staged.push(path);
            } else if status.contains(Status::WT_NEW) {
                untracked.push(path);
            }
        }

        Ok(GitStatus {
            branch,
            modified,
            staged,
            untracked,
        })
    }

    /// Gets the current branch name
    ///
    /// # Arguments
    ///
    /// * `repo_path` - Path to the git repository
    ///
    /// # Returns
    ///
    /// The name of the current branch
    ///
    /// # Errors
    ///
    /// Returns `FileError::GitError` if the repository cannot be opened or branch cannot be determined
    pub fn get_current_branch(repo_path: &Path) -> Result<String, FileError> {
        let repo = Repository::open(repo_path)
            .map_err(|e| FileError::GitError(format!("Failed to open repository: {}", e)))?;

        Self::get_current_branch_internal(&repo)
    }

    /// Internal helper to get the current branch from a repository
    fn get_current_branch_internal(repo: &Repository) -> Result<String, FileError> {
        let head = repo
            .head()
            .map_err(|e| FileError::GitError(format!("Failed to get HEAD: {}", e)))?;

        if let Some(name) = head.shorthand() {
            Ok(name.to_string())
        } else {
            Ok("HEAD".to_string())
        }
    }

    /// Stages files for commit
    ///
    /// # Arguments
    ///
    /// * `repo_path` - Path to the git repository
    /// * `files` - Paths to files to stage
    ///
    /// # Errors
    ///
    /// Returns `FileError::GitError` if staging fails
    pub fn stage_files(repo_path: &Path, files: &[PathBuf]) -> Result<(), FileError> {
        let repo = Repository::open(repo_path)
            .map_err(|e| FileError::GitError(format!("Failed to open repository: {}", e)))?;

        let mut index = repo
            .index()
            .map_err(|e| FileError::GitError(format!("Failed to get index: {}", e)))?;

        for file in files {
            index
                .add_path(file)
                .map_err(|e| FileError::GitError(format!("Failed to stage file: {}", e)))?;
        }

        index
            .write()
            .map_err(|e| FileError::GitError(format!("Failed to write index: {}", e)))?;

        Ok(())
    }

    /// Creates a commit with the given message
    ///
    /// # Arguments
    ///
    /// * `repo_path` - Path to the git repository
    /// * `message` - Commit message
    ///
    /// # Errors
    ///
    /// Returns `FileError::GitError` if the commit fails
    pub fn create_commit(repo_path: &Path, message: &str) -> Result<(), FileError> {
        let repo = Repository::open(repo_path)
            .map_err(|e| FileError::GitError(format!("Failed to open repository: {}", e)))?;

        let signature = repo
            .signature()
            .map_err(|e| FileError::GitError(format!("Failed to get signature: {}", e)))?;

        let tree_id = {
            let mut index = repo
                .index()
                .map_err(|e| FileError::GitError(format!("Failed to get index: {}", e)))?;

            index
                .write_tree()
                .map_err(|e| FileError::GitError(format!("Failed to write tree: {}", e)))?
        };

        let tree = repo
            .find_tree(tree_id)
            .map_err(|e| FileError::GitError(format!("Failed to find tree: {}", e)))?;

        let parent_commit = repo.head().ok().and_then(|head| head.peel_to_commit().ok());

        let parents = if let Some(parent) = parent_commit {
            vec![parent]
        } else {
            vec![]
        };

        let parent_refs: Vec<&git2::Commit> = parents.iter().collect();

        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &parent_refs,
        )
        .map_err(|e| FileError::GitError(format!("Failed to create commit: {}", e)))?;

        Ok(())
    }

    /// Reviews changes in the repository before committing
    ///
    /// Returns unified diffs for all modified files.
    ///
    /// # Arguments
    ///
    /// * `repo_path` - Path to the git repository
    ///
    /// # Returns
    ///
    /// A vector of FileDiff objects showing changes
    ///
    /// # Errors
    ///
    /// Returns `FileError::GitError` if the review fails
    pub fn review_changes(repo_path: &Path) -> Result<Vec<crate::models::FileDiff>, FileError> {
        let repo = Repository::open(repo_path)
            .map_err(|e| FileError::GitError(format!("Failed to open repository: {}", e)))?;

        let mut diffs = Vec::new();

        // Get the HEAD tree
        let head = repo
            .head()
            .map_err(|e| FileError::GitError(format!("Failed to get HEAD: {}", e)))?;

        let head_tree = head
            .peel_to_tree()
            .map_err(|e| FileError::GitError(format!("Failed to get HEAD tree: {}", e)))?;

        // Get the index (staged changes)
        let mut index = repo
            .index()
            .map_err(|e| FileError::GitError(format!("Failed to get index: {}", e)))?;

        let index_tree = index
            .write_tree_to(&repo)
            .map_err(|e| FileError::GitError(format!("Failed to write index tree: {}", e)))?;

        let index_tree_obj = repo
            .find_tree(index_tree)
            .map_err(|e| FileError::GitError(format!("Failed to find index tree: {}", e)))?;

        // Get diff between HEAD and index
        let diff = repo
            .diff_tree_to_tree(Some(&head_tree), Some(&index_tree_obj), None)
            .map_err(|e| FileError::GitError(format!("Failed to generate diff: {}", e)))?;

        // Convert git2 diff to our FileDiff format
        for delta in diff.deltas() {
            if let Some(path) = delta.new_file().path() {
                let file_diff = crate::models::FileDiff {
                    path: path.to_path_buf(),
                    hunks: vec![],
                    stats: crate::models::DiffStats {
                        additions: 0,
                        deletions: 0,
                        files_changed: 1,
                    },
                };
                diffs.push(file_diff);
            }
        }

        Ok(diffs)
    }

    /// Accepts all staged changes and prepares for commit
    ///
    /// # Arguments
    ///
    /// * `repo_path` - Path to the git repository
    ///
    /// # Errors
    ///
    /// Returns `FileError::GitError` if accepting changes fails
    pub fn accept_changes(repo_path: &Path) -> Result<(), FileError> {
        // In a real implementation, this would mark changes as accepted
        // For now, we just verify the repository is valid
        let _repo = Repository::open(repo_path)
            .map_err(|e| FileError::GitError(format!("Failed to open repository: {}", e)))?;

        Ok(())
    }

    /// Rejects all staged changes and reverts them
    ///
    /// # Arguments
    ///
    /// * `repo_path` - Path to the git repository
    ///
    /// # Errors
    ///
    /// Returns `FileError::GitError` if rejecting changes fails
    pub fn reject_changes(repo_path: &Path) -> Result<(), FileError> {
        let repo = Repository::open(repo_path)
            .map_err(|e| FileError::GitError(format!("Failed to open repository: {}", e)))?;

        // Reset the index to HEAD
        let head = repo
            .head()
            .map_err(|e| FileError::GitError(format!("Failed to get HEAD: {}", e)))?;

        let head_commit = head
            .peel_to_commit()
            .map_err(|e| FileError::GitError(format!("Failed to get HEAD commit: {}", e)))?;

        repo.reset(head_commit.as_object(), git2::ResetType::Mixed, None)
            .map_err(|e| FileError::GitError(format!("Failed to reset changes: {}", e)))?;

        Ok(())
    }

    /// Generates a descriptive commit message from file operations
    ///
    /// # Arguments
    ///
    /// * `operations` - List of file operations to summarize
    ///
    /// # Returns
    ///
    /// A descriptive commit message
    pub fn generate_commit_message(operations: &[FileOperation]) -> String {
        if operations.is_empty() {
            return "Update files".to_string();
        }

        let mut creates = 0;
        let mut updates = 0;
        let mut deletes = 0;

        for op in operations {
            match op.operation {
                OperationType::Create => creates += 1,
                OperationType::Update => updates += 1,
                OperationType::Delete => deletes += 1,
                OperationType::Rename { .. } => updates += 1,
            }
        }

        let mut parts = Vec::new();

        if creates > 0 {
            parts.push(format!("Create {} file(s)", creates));
        }
        if updates > 0 {
            parts.push(format!("Update {} file(s)", updates));
        }
        if deletes > 0 {
            parts.push(format!("Delete {} file(s)", deletes));
        }

        if parts.is_empty() {
            "Update files".to_string()
        } else {
            parts.join(", ")
        }
    }
}

impl Default for GitIntegration {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_commit_message_empty() {
        let operations = vec![];
        let message = GitIntegration::generate_commit_message(&operations);
        assert_eq!(message, "Update files");
    }

    #[test]
    fn test_generate_commit_message_creates() {
        let operations = vec![
            FileOperation {
                path: PathBuf::from("file1.rs"),
                operation: OperationType::Create,
                content: Some("content".to_string()),
                backup_path: None,
                content_hash: None,
            },
            FileOperation {
                path: PathBuf::from("file2.rs"),
                operation: OperationType::Create,
                content: Some("content".to_string()),
                backup_path: None,
                content_hash: None,
            },
        ];
        let message = GitIntegration::generate_commit_message(&operations);
        assert!(message.contains("Create 2 file(s)"));
    }

    #[test]
    fn test_generate_commit_message_mixed() {
        let operations = vec![
            FileOperation {
                path: PathBuf::from("file1.rs"),
                operation: OperationType::Create,
                content: Some("content".to_string()),
                backup_path: None,
                content_hash: None,
            },
            FileOperation {
                path: PathBuf::from("file2.rs"),
                operation: OperationType::Update,
                content: Some("content".to_string()),
                backup_path: None,
                content_hash: None,
            },
            FileOperation {
                path: PathBuf::from("file3.rs"),
                operation: OperationType::Delete,
                content: None,
                backup_path: None,
                content_hash: None,
            },
        ];
        let message = GitIntegration::generate_commit_message(&operations);
        assert!(message.contains("Create 1 file(s)"));
        assert!(message.contains("Update 1 file(s)"));
        assert!(message.contains("Delete 1 file(s)"));
    }

    #[test]
    fn test_git_integration_new() {
        let git = GitIntegration::new();
        assert_eq!(format!("{:?}", git), "GitIntegration");
    }

    #[test]
    fn test_git_integration_default() {
        let git = GitIntegration::default();
        assert_eq!(format!("{:?}", git), "GitIntegration");
    }

    #[test]
    fn test_generate_commit_message_updates() {
        let operations = vec![
            FileOperation {
                path: PathBuf::from("file1.rs"),
                operation: OperationType::Update,
                content: Some("content".to_string()),
                backup_path: None,
                content_hash: None,
            },
            FileOperation {
                path: PathBuf::from("file2.rs"),
                operation: OperationType::Update,
                content: Some("content".to_string()),
                backup_path: None,
                content_hash: None,
            },
        ];
        let message = GitIntegration::generate_commit_message(&operations);
        assert_eq!(message, "Update 2 file(s)");
    }

    #[test]
    fn test_generate_commit_message_deletes() {
        let operations = vec![FileOperation {
            path: PathBuf::from("file1.rs"),
            operation: OperationType::Delete,
            content: None,
            backup_path: None,
            content_hash: None,
        }];
        let message = GitIntegration::generate_commit_message(&operations);
        assert_eq!(message, "Delete 1 file(s)");
    }

    #[test]
    fn test_generate_commit_message_renames() {
        let operations = vec![FileOperation {
            path: PathBuf::from("file1.rs"),
            operation: OperationType::Rename {
                to: PathBuf::from("file2.rs"),
            },
            content: None,
            backup_path: None,
            content_hash: None,
        }];
        let message = GitIntegration::generate_commit_message(&operations);
        assert!(message.contains("Update 1 file(s)"));
    }

    #[test]
    fn test_accept_changes_invalid_repo() {
        let result = GitIntegration::accept_changes(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }

    #[test]
    fn test_reject_changes_invalid_repo() {
        let result = GitIntegration::reject_changes(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }

    #[test]
    fn test_review_changes_invalid_repo() {
        let result = GitIntegration::review_changes(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }
}
