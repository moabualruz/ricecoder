//! Cache-related error types

use thiserror::Error;

/// Cache operation errors
#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Serialization error: {message}")]
    Serialization { message: String },

    #[error("Deserialization error: {message}")]
    Deserialization { message: String },

    #[error("Storage error: {message}")]
    Storage { message: String },

    #[error("Invalid cache key: {key}")]
    InvalidKey { key: String },

    #[error("Cache entry expired")]
    Expired,

    #[error("Cache entry not found: {key}")]
    NotFound { key: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Time conversion error")]
    TimeConversion,

    #[error("Lock acquisition failed")]
    LockError,

    #[error("Compression error: {message}")]
    Compression { message: String },
}

/// Re-export commonly used Result type
pub type Result<T> = std::result::Result<T, CacheError>;
