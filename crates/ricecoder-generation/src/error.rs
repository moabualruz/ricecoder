//! Error types for code generation

use thiserror::Error;

/// Errors that can occur during code generation
#[derive(Debug, Error)]
pub enum GenerationError {
    /// Template not found
    #[error("Template not found: {0}")]
    TemplateNotFound(String),

    /// Missing required placeholder
    #[error("Missing required placeholder: {0}")]
    MissingPlaceholder(String),

    /// Invalid template syntax
    #[error("Invalid template syntax at line {line}: {message}")]
    InvalidSyntax {
        /// Line number where syntax error occurred
        line: usize,
        /// Error message describing the syntax issue
        message: String,
    },

    /// Template rendering error
    #[error("Render error: {0}")]
    RenderError(String),

    /// Validation failed
    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}
