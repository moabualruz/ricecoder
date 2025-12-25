//! Error types for process management

use std::io;
use thiserror::Error;

/// Process management errors
#[derive(Debug, Error)]
pub enum ProcessError {
    /// Failed to spawn process
    #[error("Failed to spawn process: {0}")]
    SpawnFailed(#[from] io::Error),

    /// Process timed out
    #[error("Process timed out after {seconds}s")]
    Timeout { seconds: u64 },

    /// Process crashed or exited unexpectedly
    #[error("Process crashed: {reason}")]
    Crashed { reason: String },

    /// Failed to kill process
    #[error("Failed to kill process: {0}")]
    KillFailed(String),

    /// Invalid configuration
    #[error("Invalid process configuration: {0}")]
    InvalidConfig(String),

    /// Process not found
    #[error("Process not found (PID: {pid})")]
    NotFound { pid: u32 },
}

/// Result type for process operations
pub type Result<T> = std::result::Result<T, ProcessError>;
