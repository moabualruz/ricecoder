//! Error types for the industry crate

use thiserror::Error;

/// Result type for industry operations
pub type IndustryResult<T> = Result<T, IndustryError>;

/// Errors that can occur in industry integrations
#[derive(Error, Debug)]
pub enum IndustryError {
    #[error("URL parsing error: {0}")]
    UrlParseError(#[from] url::ParseError),
    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),
    #[error("OAuth authentication failed: {message}")]
    OAuthError { message: String },

    #[error("Tool connection failed: {tool} - {message}")]
    ConnectionError { tool: String, message: String },

    #[error("Security validation failed: {message}")]
    SecurityError { message: String },

    #[error("Compliance violation: {violation}")]
    ComplianceError { violation: String },

    #[error("Provider integration error: {provider} - {message}")]
    ProviderError { provider: String, message: String },

    #[error("Configuration error: {field} - {message}")]
    ConfigError { field: String, message: String },

    #[error("Network error: {message}")]
    NetworkError { message: String },

    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    #[error("Authentication expired for {resource}")]
    AuthExpired { resource: String },

    #[error("Rate limit exceeded for {resource}")]
    RateLimitExceeded { resource: String },

    #[error("Permission denied: {permission} for {resource}")]
    PermissionDenied { permission: String, resource: String },
}