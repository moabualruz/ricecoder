//! Error types for the orchestration module

use thiserror::Error;

/// Errors that can occur during orchestration operations
#[derive(Debug, Error)]
pub enum OrchestrationError {
    #[error("Project not found: {0}")]
    ProjectNotFound(String),

    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),

    #[error("Dependency validation failed: {0}")]
    DependencyValidationFailed(String),

    #[error("Batch execution failed: {0}")]
    BatchExecutionFailed(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Rules validation failed: {0}")]
    RulesValidationFailed(String),

    #[error("Version constraint violation: {0}")]
    VersionConstraintViolation(String),

    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    #[error("Rollback failed: {0}")]
    RollbackFailed(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Path resolution error: {0}")]
    PathResolutionError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("YAML error: {0}")]
    YamlError(String),
}

impl From<serde_yaml::Error> for OrchestrationError {
    fn from(err: serde_yaml::Error) -> Self {
        OrchestrationError::YamlError(err.to_string())
    }
}

/// Result type for orchestration operations
pub type Result<T> = std::result::Result<T, OrchestrationError>;
