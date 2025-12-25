//! Error types for PTY operations

use thiserror::Error;

/// Errors that can occur during PTY operations
#[derive(Debug, Error)]
pub enum PtyError {
    /// PTY session not found
    #[error("PTY session not found: {0}")]
    SessionNotFound(String),

    /// PTY backend error
    #[error("PTY backend error: {0}")]
    BackendError(String),

    /// IO error during PTY operations
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Session already exists
    #[error("Session already exists: {0}")]
    SessionExists(String),

    /// Invalid session state for operation
    #[error("Invalid session state: {0}")]
    InvalidState(String),

    /// Resize operation failed
    #[error("Resize failed: {0}")]
    ResizeFailed(String),

    /// Write operation failed
    #[error("Write failed: {0}")]
    WriteFailed(String),

    /// Spawn operation failed
    #[error("Spawn failed: {0}")]
    SpawnFailed(String),
}

/// Result type alias for PTY operations
pub type Result<T> = std::result::Result<T, PtyError>;
