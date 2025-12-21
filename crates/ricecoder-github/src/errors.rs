//! GitHub Integration Error Types

use thiserror::Error;

/// Errors that can occur during GitHub operations
#[derive(Debug, Error)]
pub enum GitHubError {
    /// API error from GitHub
    #[error("GitHub API error: {0}")]
    ApiError(String),

    /// Authentication failed
    #[error("Authentication failed: {0}")]
    AuthError(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    /// Resource not found
    #[error("Not found: {0}")]
    NotFound(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    /// Invalid input
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// YAML parsing error
    #[error("YAML parsing error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Octocrab error
    #[error("GitHub client error: {0}")]
    OctocrabError(String),

    /// Timeout error
    #[error("Operation timed out")]
    Timeout,

    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Storage error
    #[error("Storage error: {0}")]
    StorageError(String),

    /// Generic error
    #[error("{0}")]
    Other(String),
}

impl Clone for GitHubError {
    fn clone(&self) -> Self {
        match self {
            GitHubError::ApiError(s) => GitHubError::ApiError(s.clone()),
            GitHubError::AuthError(s) => GitHubError::AuthError(s.clone()),
            GitHubError::RateLimitExceeded => GitHubError::RateLimitExceeded,
            GitHubError::NotFound(s) => GitHubError::NotFound(s.clone()),
            GitHubError::ConfigError(s) => GitHubError::ConfigError(s.clone()),
            GitHubError::InvalidInput(s) => GitHubError::InvalidInput(s.clone()),
            GitHubError::SerializationError(e) => {
                GitHubError::Other(format!("Serialization error: {}", e))
            }
            GitHubError::YamlError(e) => GitHubError::Other(format!("YAML error: {}", e)),
            GitHubError::IoError(e) => GitHubError::Other(format!("IO error: {}", e)),
            GitHubError::OctocrabError(s) => GitHubError::OctocrabError(s.clone()),
            GitHubError::Timeout => GitHubError::Timeout,
            GitHubError::NetworkError(s) => GitHubError::NetworkError(s.clone()),
            GitHubError::StorageError(s) => GitHubError::StorageError(s.clone()),
            GitHubError::Other(s) => GitHubError::Other(s.clone()),
        }
    }
}

impl GitHubError {
    /// Create a new API error
    pub fn api_error(msg: impl Into<String>) -> Self {
        GitHubError::ApiError(msg.into())
    }

    /// Create a new auth error
    pub fn auth_error(msg: impl Into<String>) -> Self {
        GitHubError::AuthError(msg.into())
    }

    /// Create a new config error
    pub fn config_error(msg: impl Into<String>) -> Self {
        GitHubError::ConfigError(msg.into())
    }

    /// Create a new not found error
    pub fn not_found(msg: impl Into<String>) -> Self {
        GitHubError::NotFound(msg.into())
    }

    /// Create a new invalid input error
    pub fn invalid_input(msg: impl Into<String>) -> Self {
        GitHubError::InvalidInput(msg.into())
    }

    /// Create a new network error
    pub fn network_error(msg: impl Into<String>) -> Self {
        GitHubError::NetworkError(msg.into())
    }

    /// Create a new storage error
    pub fn storage_error(msg: impl Into<String>) -> Self {
        GitHubError::StorageError(msg.into())
    }

    /// Check if this is a rate limit error
    pub fn is_rate_limit(&self) -> bool {
        matches!(self, GitHubError::RateLimitExceeded)
    }

    /// Check if this is an auth error
    pub fn is_auth_error(&self) -> bool {
        matches!(self, GitHubError::AuthError(_))
    }

    /// Check if this is a not found error
    pub fn is_not_found(&self) -> bool {
        matches!(self, GitHubError::NotFound(_))
    }
}

/// Result type for GitHub operations
pub type Result<T> = std::result::Result<T, GitHubError>;
