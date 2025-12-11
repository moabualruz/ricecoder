//! Repository status and file modification tracking

use crate::types::{Branch, ModifiedFile};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Overall repository status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryStatus {
    /// Current branch information
    pub current_branch: Branch,
    /// Number of uncommitted changes
    pub uncommitted_changes: usize,
    /// Number of untracked files
    pub untracked_files: usize,
    /// Number of staged files
    pub staged_files: usize,
    /// Whether the repository is clean (no changes)
    pub is_clean: bool,
    /// Whether there are conflicts
    pub has_conflicts: bool,
    /// Last commit information
    pub last_commit: Option<CommitInfo>,
    /// Repository root path
    pub repository_root: String,
}

impl RepositoryStatus {
    /// Create a new repository status
    pub fn new(current_branch: Branch, repository_root: impl Into<String>) -> Self {
        Self {
            current_branch,
            uncommitted_changes: 0,
            untracked_files: 0,
            staged_files: 0,
            is_clean: true,
            has_conflicts: false,
            last_commit: None,
            repository_root: repository_root.into(),
        }
    }

    /// Update status with file counts
    pub fn with_counts(
        mut self,
        uncommitted: usize,
        untracked: usize,
        staged: usize,
        has_conflicts: bool,
    ) -> Self {
        self.uncommitted_changes = uncommitted;
        self.untracked_files = untracked;
        self.staged_files = staged;
        self.is_clean = uncommitted == 0 && untracked == 0 && staged == 0;
        self.has_conflicts = has_conflicts;
        self
    }

    /// Set last commit information
    pub fn with_last_commit(mut self, commit: CommitInfo) -> Self {
        self.last_commit = Some(commit);
        self
    }

    /// Get a summary string for display
    pub fn summary(&self) -> String {
        if self.is_clean {
            "Clean".to_string()
        } else {
            let mut parts = Vec::new();
            if self.staged_files > 0 {
                parts.push(format!("{}S", self.staged_files));
            }
            if self.uncommitted_changes > 0 {
                parts.push(format!("{}M", self.uncommitted_changes));
            }
            if self.untracked_files > 0 {
                parts.push(format!("{}U", self.untracked_files));
            }
            if self.has_conflicts {
                parts.push("C".to_string());
            }
            parts.join(" ")
        }
    }
}

/// Information about a commit
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitInfo {
    /// Commit hash (short)
    pub hash: String,
    /// Commit message (first line)
    pub message: String,
    /// Author name
    pub author: String,
    /// Commit timestamp
    pub timestamp: DateTime<Utc>,
}

impl CommitInfo {
    /// Create a new commit info
    pub fn new(
        hash: impl Into<String>,
        message: impl Into<String>,
        author: impl Into<String>,
        timestamp: DateTime<Utc>,
    ) -> Self {
        Self {
            hash: hash.into(),
            message: message.into(),
            author: author.into(),
            timestamp,
        }
    }
}

/// File status indicators for display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModificationIndicator {
    /// File is clean (no changes)
    Clean,
    /// File has been modified
    Modified,
    /// File has been added
    Added,
    /// File has been deleted
    Deleted,
    /// File is untracked
    Untracked,
    /// File is staged
    Staged,
    /// File has conflicts
    Conflicted,
}

impl ModificationIndicator {
    /// Get the display character for this indicator
    pub fn display_char(&self) -> char {
        match self {
            ModificationIndicator::Clean => ' ',
            ModificationIndicator::Modified => 'M',
            ModificationIndicator::Added => 'A',
            ModificationIndicator::Deleted => 'D',
            ModificationIndicator::Untracked => '?',
            ModificationIndicator::Staged => 'S',
            ModificationIndicator::Conflicted => 'U',
        }
    }

    /// Get the display color for this indicator (as ANSI color code)
    pub fn display_color(&self) -> &'static str {
        match self {
            ModificationIndicator::Clean => "\x1b[0m",      // Reset
            ModificationIndicator::Modified => "\x1b[33m",  // Yellow
            ModificationIndicator::Added => "\x1b[32m",     // Green
            ModificationIndicator::Deleted => "\x1b[31m",   // Red
            ModificationIndicator::Untracked => "\x1b[36m", // Cyan
            ModificationIndicator::Staged => "\x1b[32m",    // Green
            ModificationIndicator::Conflicted => "\x1b[35m", // Magenta
        }
    }

    /// Create from a modified file
    pub fn from_modified_file(file: &ModifiedFile) -> Self {
        if file.staged {
            ModificationIndicator::Staged
        } else {
            match file.status {
                crate::types::FileStatus::Modified => ModificationIndicator::Modified,
                crate::types::FileStatus::Added => ModificationIndicator::Added,
                crate::types::FileStatus::Deleted => ModificationIndicator::Deleted,
                crate::types::FileStatus::Untracked => ModificationIndicator::Untracked,
                crate::types::FileStatus::Conflicted => ModificationIndicator::Conflicted,
                _ => ModificationIndicator::Modified,
            }
        }
    }
}

/// Re-export FileStatus from types module
pub use crate::types::FileStatus;