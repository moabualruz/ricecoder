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

    /// Spec processing error
    #[error("Spec error: {0}")]
    SpecError(String),

    /// Prompt building error
    #[error("Prompt error: {0}")]
    PromptError(String),

    /// Code generation failed
    #[error("Generation failed: {0}")]
    GenerationFailed(String),

    /// Validation error with details
    #[error("Validation error in {file}:{line}: {message}")]
    ValidationError {
        /// File path where error occurred
        file: String,
        /// Line number where error occurred
        line: usize,
        /// Error message
        message: String,
    },

    /// Linting error
    #[error("Linting error: {0}")]
    LintingError(String),

    /// Type checking error
    #[error("Type checking error: {0}")]
    TypeCheckingError(String),

    /// Syntax error
    #[error("Syntax error: {0}")]
    SyntaxError(String),

    /// Write failed
    #[error("Write failed: {0}")]
    WriteFailed(String),

    /// Rollback failed
    #[error("Rollback failed: {0}")]
    RollbackFailed(String),
}
