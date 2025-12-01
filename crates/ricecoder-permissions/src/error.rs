//! Error types for the permissions system

use thiserror::Error;

/// Result type for permissions operations
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur in the permissions system
#[derive(Error, Debug)]
pub enum Error {
    #[error("Permission denied for tool: {tool}")]
    PermissionDenied { tool: String },

    #[error("Invalid permission level: {0}")]
    InvalidPermissionLevel(String),

    #[error("Invalid glob pattern: {0}")]
    InvalidGlobPattern(String),

    #[error("Pattern matching error: {0}")]
    PatternMatchError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid audit log entry: {0}")]
    InvalidAuditEntry(String),

    #[error("Prompt error: {0}")]
    PromptError(String),

    #[error("Internal error: {0}")]
    Internal(String),
}
