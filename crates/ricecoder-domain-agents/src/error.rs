//! Error types for domain-specific agents

use thiserror::Error;

/// Errors that can occur in domain agent operations
#[derive(Debug, Error)]
pub enum DomainAgentError {
    /// Domain not found
    #[error("Domain not found: {0}")]
    DomainNotFound(String),

    /// Agent not found for domain
    #[error("Agent not found for domain: {0}")]
    AgentNotFound(String),

    /// Knowledge base error
    #[error("Knowledge base error: {0}")]
    KnowledgeBaseError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// YAML error
    #[error("YAML error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    /// Storage error
    #[error("Storage error: {0}")]
    StorageError(String),

    /// Agent execution error
    #[error("Agent execution error: {0}")]
    ExecutionError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Invalid domain configuration
    #[error("Invalid domain configuration: {0}")]
    InvalidConfiguration(String),

    /// Domain knowledge not available
    #[error("Domain knowledge not available: {0}")]
    KnowledgeNotAvailable(String),
}

/// Result type for domain agent operations
pub type Result<T> = std::result::Result<T, DomainAgentError>;
