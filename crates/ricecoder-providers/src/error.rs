//! Error types for the providers module

use thiserror::Error;

/// Errors that can occur when interacting with providers
#[derive(Debug, Error, PartialEq, Clone)]
pub enum ProviderError {
    /// Provider not found by ID or name
    #[error("Provider not found: {0}")]
    NotFound(String),

    /// Authentication failed (never includes key details)
    #[error("Authentication failed")]
    AuthError,

    /// Rate limited by provider
    #[error("Rate limited, retry after {0} seconds")]
    RateLimited(u64),

    /// Context is too large for the provider
    #[error("Context too large: {0} tokens, max {1}")]
    ContextTooLarge(usize, usize),

    /// Network error occurred
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Generic provider error
    #[error("Provider error: {0}")]
    ProviderError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Invalid model specified
    #[error("Invalid model: {0}")]
    InvalidModel(String),

    /// Model not available in provider
    #[error("Model not available: {0}")]
    ModelNotAvailable(String),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Parse error
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<serde_json::Error> for ProviderError {
    fn from(err: serde_json::Error) -> Self {
        ProviderError::SerializationError(err.to_string())
    }
}

impl From<reqwest::Error> for ProviderError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            ProviderError::ProviderError("Request timeout".to_string())
        } else if err.is_connect() {
            ProviderError::NetworkError(err.to_string())
        } else {
            ProviderError::ProviderError(err.to_string())
        }
    }
}
