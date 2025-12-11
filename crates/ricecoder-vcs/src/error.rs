//! Error types for VCS operations

use thiserror::Error;

/// Result type for VCS operations
pub type Result<T> = std::result::Result<T, VcsError>;

/// Errors that can occur during VCS operations
#[derive(Debug, Error)]
pub enum VcsError {
    /// Git repository error
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    /// Repository not found
    #[error("Repository not found at path: {path}")]
    RepositoryNotFound { path: String },

    /// Invalid repository state
    #[error("Invalid repository state: {message}")]
    InvalidState { message: String },

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Invalid branch name
    #[error("Invalid branch name: {name}")]
    InvalidBranch { name: String },

    /// File not found in repository
    #[error("File not found in repository: {path}")]
    FileNotFound { path: String },

    /// Operation not supported
    #[error("Operation not supported: {operation}")]
    NotSupported { operation: String },
}