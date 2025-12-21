//! Error types for the activity log crate

use thiserror::Error;

/// Result type for activity logging operations
pub type ActivityLogResult<T> = Result<T, ActivityLogError>;

/// Errors that can occur in activity logging operations
#[derive(Error, Debug)]
pub enum ActivityLogError {
    #[error("Storage error: {message}")]
    StorageError { message: String },

    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    #[error("Configuration error: {field} - {message}")]
    ConfigError { field: String, message: String },

    #[error("Validation error: {message}")]
    ValidationError { message: String },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Session error: {0}")]
    SessionError(String),

    #[error("Retention policy violation: {message}")]
    RetentionError { message: String },

    #[error("Performance monitoring error: {message}")]
    MonitoringError { message: String },
}
