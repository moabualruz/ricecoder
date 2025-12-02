//! Error types for file management operations

use std::path::PathBuf;

/// Errors that can occur during file operations
#[derive(Debug, thiserror::Error)]
pub enum FileError {
    /// File not found at the specified path
    #[error("File not found: {0}")]
    NotFound(PathBuf),

    /// Permission denied for the operation
    #[error("Permission denied: {0}")]
    PermissionDenied(PathBuf),

    /// Conflict detected at path: file already exists
    #[error("Conflict detected at {0}: file already exists")]
    ConflictDetected(PathBuf),

    /// Invalid content provided
    #[error("Invalid content: {0}")]
    InvalidContent(String),

    /// Content verification failed
    #[error("Content verification failed: {0}")]
    VerificationFailed(String),

    /// Backup operation failed
    #[error("Backup failed: {0}")]
    BackupFailed(String),

    /// Backup integrity check failed: hash mismatch
    #[error("Backup integrity check failed: hash mismatch")]
    BackupCorrupted,

    /// Transaction operation failed
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    /// Rollback operation failed
    #[error("Rollback failed: {0}")]
    RollbackFailed(String),

    /// Git operation failed
    #[error("Git operation failed: {0}")]
    GitError(String),

    /// Diff generation failed
    #[error("Diff generation failed: {0}")]
    DiffError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
