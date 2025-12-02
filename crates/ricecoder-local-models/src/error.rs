//! Error types for local model operations

use thiserror::Error;

/// Errors that can occur during local model operations
#[derive(Debug, Error)]
pub enum LocalModelError {
    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Model pull failed: {0}")]
    PullFailed(String),

    #[error("Model removal failed: {0}")]
    RemovalFailed(String),

    #[error("Model update failed: {0}")]
    UpdateFailed(String),

    #[error("Invalid model name: {0}")]
    InvalidModelName(String),

    #[error("Insufficient disk space")]
    InsufficientDiskSpace,

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Provider error: {0}")]
    ProviderError(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<reqwest::Error> for LocalModelError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            LocalModelError::Timeout(err.to_string())
        } else if err.is_connect() {
            LocalModelError::NetworkError(err.to_string())
        } else {
            LocalModelError::NetworkError(err.to_string())
        }
    }
}
