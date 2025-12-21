//! Error types for session operations

use base64;
use std::io;
use thiserror::Error;

/// Result type for session operations
pub type SessionResult<T> = Result<T, SessionError>;

/// Errors that can occur during session operations
#[derive(Debug, Error)]
pub enum SessionError {
    /// Session not found
    #[error("Session not found: {0}")]
    NotFound(String),

    /// Session already exists
    #[error("Session already exists: {0}")]
    AlreadyExists(String),

    /// Invalid session state or data
    #[error("Invalid session: {0}")]
    Invalid(String),

    /// Session limit reached
    #[error("Session limit reached: maximum {max} sessions allowed")]
    LimitReached { max: usize },

    /// Storage error
    #[error("Storage error: {0}")]
    StorageError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Share not found
    #[error("Share not found: {0}")]
    ShareNotFound(String),

    /// Share expired
    #[error("Share expired: {0}")]
    ShareExpired(String),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Background agent error
    #[error("Background agent error: {0}")]
    AgentError(String),

    /// Token estimation error
    #[error("Token estimation error: {0}")]
    TokenEstimation(String),
}

impl From<ricecoder_security::SecurityError> for SessionError {
    fn from(err: ricecoder_security::SecurityError) -> Self {
        SessionError::StorageError(format!("Security error: {}", err))
    }
}

impl From<base64::DecodeError> for SessionError {
    fn from(err: base64::DecodeError) -> Self {
        SessionError::Invalid(format!("Base64 decode error: {}", err))
    }
}
