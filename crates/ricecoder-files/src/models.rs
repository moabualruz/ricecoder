//! Data models for file management operations

use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a single file operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOperation {
    /// Path to the file being operated on
    pub path: PathBuf,
    /// Type of operation (create, update, delete, rename)
    pub operation: OperationType,
    /// Content of the file (if applicable)
    pub content: Option<String>,
    /// Path to the backup file (if created)
    pub backup_path: Option<PathBuf>,
    /// SHA-256 hash of the content for verification
    pub content_hash: Option<String>,
}

/// Types of file operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OperationType {
    /// Create a new file
    Create,
    /// Update an existing file
    Update,
    /// Delete a file
    Delete,
    /// Rename a file
    Rename {
        /// New path for the file
        to: PathBuf,
    },
}

/// Information about a detected conflict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictInfo {
    /// Path where the conflict was detected
    pub path: PathBuf,
    /// Content of the existing file
    pub existing_content: String,
    /// Content of the new file
    pub new_content: String,
}

/// Strategy for resolving conflicts
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ConflictResolution {
    /// Skip the write operation, leave existing file unchanged
    Skip,
    /// Overwrite the existing file with new content
    Overwrite,
    /// Merge the changes (combine both versions)
    Merge,
}

/// Represents a transaction containing multiple file operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTransaction {
    /// Unique identifier for the transaction
    pub id: Uuid,
    /// List of operations in the transaction
    pub operations: Vec<FileOperation>,
    /// Current status of the transaction
    pub status: TransactionStatus,
    /// When the transaction was created
    pub created_at: DateTime<Utc>,
    /// When the transaction was completed (if applicable)
    pub completed_at: Option<DateTime<Utc>>,
}

/// Status of a transaction
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransactionStatus {
    /// Transaction is pending execution
    Pending,
    /// Transaction has been committed
    Committed,
    /// Transaction has been rolled back
    RolledBack,
}

/// Entry in the audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// When the operation occurred
    pub timestamp: DateTime<Utc>,
    /// Path of the file affected
    pub path: PathBuf,
    /// Type of operation
    pub operation_type: OperationType,
    /// SHA-256 hash of the content
    pub content_hash: String,
    /// Transaction ID if part of a transaction
    pub transaction_id: Option<Uuid>,
}

/// Metadata about a backup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    /// Original path of the file
    pub original_path: PathBuf,
    /// Path where the backup is stored
    pub backup_path: PathBuf,
    /// When the backup was created
    pub timestamp: DateTime<Utc>,
    /// SHA-256 hash of the backup content for integrity verification
    pub content_hash: String,
}

/// Represents a diff between two file versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDiff {
    /// Path of the file
    pub path: PathBuf,
    /// List of hunks (sections of changes)
    pub hunks: Vec<DiffHunk>,
    /// Statistics about the diff
    pub stats: DiffStats,
}

/// A hunk is a section of changes in a diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHunk {
    /// Starting line number in the old file
    pub old_start: usize,
    /// Number of lines in the old file
    pub old_count: usize,
    /// Starting line number in the new file
    pub new_start: usize,
    /// Number of lines in the new file
    pub new_count: usize,
    /// Lines in this hunk
    pub lines: Vec<DiffLine>,
}

/// A single line in a diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiffLine {
    /// Context line (unchanged)
    Context(String),
    /// Added line
    Added(String),
    /// Removed line
    Removed(String),
}

/// Statistics about a diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffStats {
    /// Number of lines added
    pub additions: usize,
    /// Number of lines deleted
    pub deletions: usize,
    /// Number of files changed
    pub files_changed: usize,
}

/// Git repository status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatus {
    /// Current branch name
    pub branch: String,
    /// Modified files
    pub modified: Vec<PathBuf>,
    /// Staged files
    pub staged: Vec<PathBuf>,
    /// Untracked files
    pub untracked: Vec<PathBuf>,
}
