//! RiceCoder VCS Integration
//!
//! This crate provides VCS (Version Control System) integration for RiceCoder TUI, including:
//! - Git repository detection and status reading
//! - Current branch and uncommitted changes tracking
//! - Modified files tracking with modification indicators
//! - Diff viewing and staging changes support
//!
//! # Examples
//!
//! ```ignore
//! use ricecoder_vcs::{GitRepository, RepositoryStatus};
//!
//! // Detect and open a Git repository
//! let repo = GitRepository::discover(".")?;
//!
//! // Get repository status
//! let status = repo.get_status()?;
//! println!("Branch: {}", status.current_branch.name);
//! println!("Uncommitted changes: {}", status.uncommitted_changes);
//!
//! // Get modified files
//! let modified_files = repo.get_modified_files()?;
//! for file in modified_files {
//!     println!("{}: {}", file.path.display(), file.status_display());
//! }
//! ```

pub mod di;
pub mod error;
pub mod git;
pub mod repository;
pub mod status;
pub mod tui_integration;
pub mod types;

pub use error::{Result, VcsError};
pub use git::GitRepository;
#[allow(deprecated)]
pub use repository::Repository;
pub use repository::{RepositoryFileInspection, RepositoryMutation, RepositoryQuery};
pub use status::{FileStatus, ModificationIndicator, RepositoryStatus};
pub use tui_integration::{VcsIntegration, VcsStatus};
pub use types::{Branch, ModifiedFile};

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn test_modification_indicator_display() {
        assert_eq!(ModificationIndicator::Clean.display_char(), ' ');
        assert_eq!(ModificationIndicator::Modified.display_char(), 'M');
        assert_eq!(ModificationIndicator::Added.display_char(), 'A');
        assert_eq!(ModificationIndicator::Deleted.display_char(), 'D');
        assert_eq!(ModificationIndicator::Untracked.display_char(), '?');
        assert_eq!(ModificationIndicator::Staged.display_char(), 'S');
        assert_eq!(ModificationIndicator::Conflicted.display_char(), 'U');
    }

    #[test]
    fn test_file_status_display() {
        let file = ModifiedFile::new("test.txt", FileStatus::Modified);
        assert_eq!(file.status_display(), "M");

        let file = ModifiedFile::new("test.txt", FileStatus::Added);
        assert_eq!(file.status_display(), "A");

        let file = ModifiedFile::new("test.txt", FileStatus::Untracked);
        assert_eq!(file.status_display(), "?");
    }

    #[test]
    fn test_repository_status_summary() {
        let branch = Branch::new("main").current();
        let status = RepositoryStatus::new(branch.clone(), "/test/repo");
        assert_eq!(status.summary(), "Clean");
        assert!(status.is_clean);

        let status = RepositoryStatus::new(branch.clone(), "/test/repo").with_counts(2, 1, 1, false);
        assert_eq!(status.summary(), "1S 2M 1U");
        assert!(!status.is_clean);

        // Test that conflicts alone make the repository not clean
        let status = RepositoryStatus::new(branch, "/test/repo").with_counts(0, 0, 0, true);
        assert_eq!(status.summary(), "C");
        assert!(!status.is_clean); // Should NOT be clean when there are conflicts
    }

    #[test]
    fn test_branch_creation() {
        let branch = Branch::new("main").current();
        assert_eq!(branch.name, "main");
        assert!(branch.is_current);
        assert!(!branch.is_remote);

        let remote_branch = Branch::new("origin/main").remote();
        assert_eq!(remote_branch.name, "origin/main");
        assert!(!remote_branch.is_current);
        assert!(remote_branch.is_remote);
    }

    #[test]
    fn test_modified_file_creation() {
        let file = ModifiedFile::new("test.txt", FileStatus::Modified);
        assert_eq!(file.path, Path::new("test.txt"));
        assert_eq!(file.status, FileStatus::Modified);
        assert!(!file.staged);

        let staged_file = file.staged();
        assert!(staged_file.staged);
    }

    #[test]
    fn test_file_status_from_git2() {
        use git2::Status;

        assert_eq!(
            FileStatus::from_git2_status(Status::WT_MODIFIED),
            FileStatus::Modified
        );
        assert_eq!(
            FileStatus::from_git2_status(Status::WT_NEW),
            FileStatus::Added
        );
        assert_eq!(
            FileStatus::from_git2_status(Status::WT_DELETED),
            FileStatus::Deleted
        );
        assert_eq!(
            FileStatus::from_git2_status(Status::CONFLICTED),
            FileStatus::Conflicted
        );
        assert_eq!(
            FileStatus::from_git2_status(Status::IGNORED),
            FileStatus::Ignored
        );
    }

    #[test]
    fn test_modification_indicator_from_file() {
        let file = ModifiedFile::new("test.txt", FileStatus::Modified);
        let indicator = ModificationIndicator::from_modified_file(&file);
        assert_eq!(indicator, ModificationIndicator::Modified);

        let staged_file = file.staged();
        let staged_indicator = ModificationIndicator::from_modified_file(&staged_file);
        assert_eq!(staged_indicator, ModificationIndicator::Staged);
    }

    #[test]
    fn test_repository_status_ahead_behind() {
        let branch = Branch::new("feature").current();
        let status = RepositoryStatus::new(branch, "/test/repo");
        
        // Default values
        assert_eq!(status.ahead, 0);
        assert_eq!(status.behind, 0);
        
        // With ahead/behind set
        let status = status.with_ahead_behind(3, 1);
        assert_eq!(status.ahead, 3);
        assert_eq!(status.behind, 1);
    }
}
