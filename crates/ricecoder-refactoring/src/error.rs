//! Error types for the refactoring engine

use thiserror::Error;

/// Result type for refactoring operations
pub type Result<T> = std::result::Result<T, RefactoringError>;

/// Errors that can occur during refactoring operations
#[derive(Debug, Error)]
pub enum RefactoringError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    /// Storage error
    #[error("Storage error: {0}")]
    StorageError(String),

    /// Analysis failed
    #[error("Analysis failed: {0}")]
    AnalysisFailed(String),

    /// Refactoring failed
    #[error("Refactoring failed: {0}")]
    RefactoringFailed(String),

    /// Impact analysis failed
    #[error("Impact analysis failed: {0}")]
    ImpactAnalysisFailed(String),

    /// Validation failed
    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    /// Rollback failed
    #[error("Rollback failed: {0}")]
    RollbackFailed(String),

    /// File operation error
    #[error("File operation error: {0}")]
    FileError(String),

    /// Pattern error
    #[error("Pattern error: {0}")]
    PatternError(String),

    /// Provider error
    #[error("Provider error: {0}")]
    ProviderError(String),

    /// LSP error
    #[error("LSP error: {0}")]
    LspError(String),

    /// Other error
    #[error("{0}")]
    Other(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// YAML error
    #[error("YAML error: {0}")]
    YamlError(#[from] serde_yaml::Error),
}
