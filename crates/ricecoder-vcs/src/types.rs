//! Common types for VCS operations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents a Git branch
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Branch {
    /// Branch name
    pub name: String,
    /// Whether this is the current branch
    pub is_current: bool,
    /// Whether this is a remote branch
    pub is_remote: bool,
    /// Last commit hash (short)
    pub last_commit: Option<String>,
    /// Last commit message
    pub last_commit_message: Option<String>,
    /// Last commit timestamp
    pub last_commit_time: Option<DateTime<Utc>>,
}

impl Branch {
    /// Create a new branch
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            is_current: false,
            is_remote: false,
            last_commit: None,
            last_commit_message: None,
            last_commit_time: None,
        }
    }

    /// Mark this branch as current
    pub fn current(mut self) -> Self {
        self.is_current = true;
        self
    }

    /// Mark this branch as remote
    pub fn remote(mut self) -> Self {
        self.is_remote = true;
        self
    }

    /// Set commit information
    pub fn with_commit(
        mut self,
        hash: impl Into<String>,
        message: impl Into<String>,
        time: DateTime<Utc>,
    ) -> Self {
        self.last_commit = Some(hash.into());
        self.last_commit_message = Some(message.into());
        self.last_commit_time = Some(time);
        self
    }
}

/// Represents a modified file in the repository
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModifiedFile {
    /// File path relative to repository root
    pub path: PathBuf,
    /// File status (modified, added, deleted, etc.)
    pub status: FileStatus,
    /// Whether the file is staged
    pub staged: bool,
    /// Number of lines added
    pub lines_added: Option<usize>,
    /// Number of lines removed
    pub lines_removed: Option<usize>,
}

impl ModifiedFile {
    /// Create a new modified file
    pub fn new(path: impl Into<PathBuf>, status: FileStatus) -> Self {
        Self {
            path: path.into(),
            status,
            staged: false,
            lines_added: None,
            lines_removed: None,
        }
    }

    /// Mark this file as staged
    pub fn staged(mut self) -> Self {
        self.staged = true;
        self
    }

    /// Set line change counts
    pub fn with_changes(mut self, added: usize, removed: usize) -> Self {
        self.lines_added = Some(added);
        self.lines_removed = Some(removed);
        self
    }

    /// Get a display string for the file status
    pub fn status_display(&self) -> &'static str {
        match self.status {
            FileStatus::Modified => "M",
            FileStatus::Added => "A",
            FileStatus::Deleted => "D",
            FileStatus::Renamed => "R",
            FileStatus::Copied => "C",
            FileStatus::Untracked => "?",
            FileStatus::Ignored => "!",
            FileStatus::Conflicted => "U",
        }
    }
}

/// File status in the repository
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileStatus {
    /// File has been modified
    Modified,
    /// File has been added
    Added,
    /// File has been deleted
    Deleted,
    /// File has been renamed
    Renamed,
    /// File has been copied
    Copied,
    /// File is untracked
    Untracked,
    /// File is ignored
    Ignored,
    /// File has conflicts
    Conflicted,
}

impl FileStatus {
    /// Convert from git2::Status flags
    pub fn from_git2_status(status: git2::Status) -> Self {
        if status.contains(git2::Status::CONFLICTED) {
            FileStatus::Conflicted
        } else if status.contains(git2::Status::WT_MODIFIED)
            || status.contains(git2::Status::INDEX_MODIFIED)
        {
            FileStatus::Modified
        } else if status.contains(git2::Status::WT_NEW) || status.contains(git2::Status::INDEX_NEW)
        {
            FileStatus::Added
        } else if status.contains(git2::Status::WT_DELETED)
            || status.contains(git2::Status::INDEX_DELETED)
        {
            FileStatus::Deleted
        } else if status.contains(git2::Status::WT_RENAMED)
            || status.contains(git2::Status::INDEX_RENAMED)
        {
            FileStatus::Renamed
        } else if status.contains(git2::Status::IGNORED) {
            FileStatus::Ignored
        } else {
            FileStatus::Untracked
        }
    }
}
