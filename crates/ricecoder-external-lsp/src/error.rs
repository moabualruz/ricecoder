//! Error types for external LSP integration

use thiserror::Error;

/// Errors that can occur in external LSP operations
#[derive(Debug, Error)]
pub enum ExternalLspError {
    #[error("LSP server not found: {executable}")]
    ServerNotFound { executable: String },

    #[error("Failed to spawn LSP server: {0}")]
    SpawnFailed(#[from] std::io::Error),

    #[error("LSP server crashed: {reason}")]
    ServerCrashed { reason: String },

    #[error("Request timeout after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("Initialization failed: {0}")]
    InitializationFailed(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("No LSP server configured for language: {language}")]
    NoServerForLanguage { language: String },

    #[error("JSON path error: {0}")]
    JsonPathError(String),

    #[error("Transformation error: {0}")]
    TransformationError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
}

/// Result type for external LSP operations
pub type Result<T> = std::result::Result<T, ExternalLspError>;
