//! HTTP client error types

use thiserror::Error;

/// Result type for HTTP operations
pub type Result<T> = std::result::Result<T, HttpError>;

/// HTTP client errors
#[derive(Debug, Error)]
pub enum HttpError {
    /// Network request failed
    #[error("Network request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    /// Request timeout
    #[error("Request timed out after {0:?}")]
    Timeout(std::time::Duration),

    /// Invalid URL
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// Invalid proxy configuration
    #[error("Invalid proxy configuration: {0}")]
    InvalidProxy(String),

    /// HTTP error status
    #[error("HTTP {status}: {message}")]
    HttpStatus {
        status: reqwest::StatusCode,
        message: String,
    },

    /// Retry limit exceeded
    #[error("Retry limit exceeded after {attempts} attempts")]
    RetryLimitExceeded { attempts: u32 },

    /// Client build error
    #[error("Failed to build HTTP client: {0}")]
    BuildError(String),
}

impl HttpError {
    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            HttpError::RequestFailed(e) => {
                // Retry on network errors, not client errors
                e.is_timeout() || e.is_connect() || e.is_request()
            }
            HttpError::Timeout(_) => true,
            HttpError::HttpStatus { status, .. } => {
                // Retry on 5xx server errors and 429 rate limit
                status.is_server_error() || *status == reqwest::StatusCode::TOO_MANY_REQUESTS
            }
            _ => false,
        }
    }
}
