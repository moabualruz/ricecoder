//! Generic repository trait and implementations

use crate::error::Result;
use crate::status::RepositoryStatus;
use crate::types::{Branch, ModifiedFile};
use std::path::Path;

/// Generic repository trait for VCS operations
pub trait Repository {
    /// Get the current repository status
    fn get_status(&self) -> Result<RepositoryStatus>;

    /// Get the current branch
    fn get_current_branch(&self) -> Result<Branch>;

    /// Get all branches
    fn get_branches(&self) -> Result<Vec<Branch>>;

    /// Get modified files
    fn get_modified_files(&self) -> Result<Vec<ModifiedFile>>;

    /// Get the repository root path
    fn get_root_path(&self) -> Result<String>;

    /// Check if the repository is clean (no uncommitted changes)
    fn is_clean(&self) -> Result<bool>;

    /// Get the number of uncommitted changes
    fn count_uncommitted_changes(&self) -> Result<usize>;

    /// Get diff for a specific file
    fn get_file_diff(&self, file_path: &Path) -> Result<String>;

    /// Stage a file
    fn stage_file(&self, file_path: &Path) -> Result<()>;

    /// Unstage a file
    fn unstage_file(&self, file_path: &Path) -> Result<()>;

    /// Stage all changes
    fn stage_all(&self) -> Result<()>;

    /// Reset all changes
    fn reset_all(&self) -> Result<()>;
}