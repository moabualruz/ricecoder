//! Generic repository trait and implementations
//!
//! This module provides ISP-compliant traits for VCS operations:
//! - `RepositoryQuery`: Read-only status queries (6 methods)
//! - `RepositoryFileInspection`: File inspection operations (2 methods)
//! - `RepositoryMutation`: Write operations (4 methods)
//!
//! The original `Repository` trait is deprecated but maintained as a backward-compatible
//! super-trait with blanket implementation.

use std::path::Path;

use crate::{
    error::Result,
    status::RepositoryStatus,
    types::{Branch, ModifiedFile},
};

/// Read-only repository status queries
///
/// This trait provides methods for querying repository state without modification.
/// Ideal for read-only clients like status displays, monitoring tools, or analytics.
///
/// # Examples
///
/// ```ignore
/// use ricecoder_vcs::{GitRepository, RepositoryQuery};
///
/// let repo = GitRepository::discover(".")?;
/// let status = repo.get_status()?;
/// println!("Branch: {}, Clean: {}", status.current_branch.name, repo.is_clean()?);
/// ```
pub trait RepositoryQuery {
    /// Get the current repository status
    fn get_status(&self) -> Result<RepositoryStatus>;

    /// Get the current branch
    fn get_current_branch(&self) -> Result<Branch>;

    /// Get all branches
    fn get_branches(&self) -> Result<Vec<Branch>>;

    /// Check if the repository is clean (no uncommitted changes)
    fn is_clean(&self) -> Result<bool>;

    /// Get the number of uncommitted changes
    fn count_uncommitted_changes(&self) -> Result<usize>;

    /// Get the repository root path
    fn get_root_path(&self) -> Result<String>;
}

/// File inspection operations
///
/// This trait provides methods for inspecting file changes and diffs.
/// Separates file-level operations from broader repository queries.
///
/// # Examples
///
/// ```ignore
/// use ricecoder_vcs::{GitRepository, RepositoryFileInspection};
/// use std::path::Path;
///
/// let repo = GitRepository::discover(".")?;
/// let modified = repo.get_modified_files()?;
/// let diff = repo.get_file_diff(Path::new("src/main.rs"))?;
/// ```
pub trait RepositoryFileInspection {
    /// Get modified files
    fn get_modified_files(&self) -> Result<Vec<ModifiedFile>>;

    /// Get diff for a specific file
    fn get_file_diff(&self, file_path: &Path) -> Result<String>;
}

/// Write operations for repository changes
///
/// This trait provides methods for modifying repository state (staging, unstaging).
/// Separated from read operations to enable read-only clients and better security boundaries.
///
/// # Examples
///
/// ```ignore
/// use ricecoder_vcs::{GitRepository, RepositoryMutation};
/// use std::path::Path;
///
/// let repo = GitRepository::discover(".")?;
/// repo.stage_file(Path::new("src/main.rs"))?;
/// repo.stage_all()?;
/// ```
pub trait RepositoryMutation {
    /// Stage a file
    fn stage_file(&self, file_path: &Path) -> Result<()>;

    /// Unstage a file
    fn unstage_file(&self, file_path: &Path) -> Result<()>;

    /// Stage all changes
    fn stage_all(&self) -> Result<()>;

    /// Reset all changes
    fn reset_all(&self) -> Result<()>;
}

/// Generic repository trait for VCS operations
///
/// # Deprecation Notice
///
/// This trait is deprecated in favor of the more focused traits:
/// - `RepositoryQuery` for read-only status queries
/// - `RepositoryFileInspection` for file inspection
/// - `RepositoryMutation` for write operations
///
/// The trait remains as a super-trait for backward compatibility via blanket implementation.
/// Any type implementing all three focused traits automatically implements `Repository`.
///
/// # Migration Guide
///
/// **Before:**
/// ```ignore
/// fn process<R: Repository>(repo: &R) {
///     let status = repo.get_status()?;
///     repo.stage_file(path)?;
/// }
/// ```
///
/// **After (recommended):**
/// ```ignore
/// // If you need read-only access:
/// fn read_status<R: RepositoryQuery>(repo: &R) {
///     let status = repo.get_status()?;
/// }
///
/// // If you need write access:
/// fn stage_changes<R: RepositoryMutation>(repo: &R) {
///     repo.stage_file(path)?;
/// }
///
/// // If you need both (compose traits):
/// fn process<R: RepositoryQuery + RepositoryMutation>(repo: &R) {
///     let status = repo.get_status()?;
///     repo.stage_file(path)?;
/// }
/// ```
#[deprecated(
    since = "0.2.0",
    note = "Use RepositoryQuery, RepositoryFileInspection, or RepositoryMutation instead"
)]
pub trait Repository: RepositoryQuery + RepositoryFileInspection + RepositoryMutation {}

/// Blanket implementation: any type implementing all three focused traits
/// automatically implements the deprecated `Repository` trait for backward compatibility.
#[allow(deprecated)]
impl<T> Repository for T where T: RepositoryQuery + RepositoryFileInspection + RepositoryMutation {}
