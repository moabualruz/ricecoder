//! Error types for IDE integration

use thiserror::Error;

/// IDE Integration Error
#[derive(Debug, Error)]
pub enum IdeError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Configuration validation error
    #[error("Configuration validation error: {0}")]
    ConfigValidationError(String),

    /// Path resolution error
    #[error("Path resolution error: {0}")]
    PathResolutionError(String),

    /// LSP server error
    #[error("LSP server error: {0}")]
    LspError(String),

    /// Provider error
    #[error("Provider error: {0}")]
    ProviderError(String),

    /// IDE communication error
    #[error("IDE communication error: {0}")]
    CommunicationError(String),

    /// Timeout error
    #[error("Operation timeout after {0}ms")]
    Timeout(u64),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// YAML parsing error
    #[error("YAML parsing error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    /// JSON Schema validation error
    #[error("JSON Schema validation error: {0}")]
    SchemaValidationError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Generic error
    #[error("{0}")]
    Other(String),
}

impl IdeError {
    /// Create a configuration error with remediation steps
    pub fn config_error(message: impl Into<String>) -> Self {
        IdeError::ConfigError(message.into())
    }

    /// Create a configuration validation error with remediation steps
    pub fn config_validation_error(message: impl Into<String>) -> Self {
        IdeError::ConfigValidationError(message.into())
    }

    /// Create a path resolution error
    pub fn path_resolution_error(message: impl Into<String>) -> Self {
        IdeError::PathResolutionError(message.into())
    }

    /// Create an LSP error
    pub fn lsp_error(message: impl Into<String>) -> Self {
        IdeError::LspError(message.into())
    }

    /// Create a provider error
    pub fn provider_error(message: impl Into<String>) -> Self {
        IdeError::ProviderError(message.into())
    }

    /// Create a communication error
    pub fn communication_error(message: impl Into<String>) -> Self {
        IdeError::CommunicationError(message.into())
    }

    /// Create a timeout error
    pub fn timeout(ms: u64) -> Self {
        IdeError::Timeout(ms)
    }

    /// Create a schema validation error
    pub fn schema_validation_error(message: impl Into<String>) -> Self {
        IdeError::SchemaValidationError(message.into())
    }

    /// Create a generic error
    pub fn other(message: impl Into<String>) -> Self {
        IdeError::Other(message.into())
    }
}

/// Result type for IDE integration operations
pub type IdeResult<T> = Result<T, IdeError>;
