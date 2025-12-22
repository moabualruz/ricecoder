//! Error types for the specification system

use std::io;

use thiserror::Error;

/// Errors that can occur in the specification system
#[derive(Debug, Error)]
pub enum SpecError {
    /// Spec not found
    #[error("Spec not found: {0}")]
    NotFound(String),

    /// Invalid spec format
    #[error("Invalid spec format: {0}")]
    InvalidFormat(String),

    /// Validation failed
    #[error("Validation failed")]
    ValidationFailed(Vec<ValidationError>),

    /// Parse error with location information
    #[error("Parse error at {path}:{line}: {message}")]
    ParseError {
        /// File path where error occurred
        path: String,
        /// Line number where error occurred
        line: usize,
        /// Error message
        message: String,
    },

    /// Circular dependency detected
    #[error("Circular dependency detected: {specs:?}")]
    CircularDependency {
        /// Specs involved in the circular dependency
        specs: Vec<String>,
    },

    /// Inheritance conflict
    #[error("Inheritance conflict: {0}")]
    InheritanceConflict(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    /// YAML parsing error
    #[error("YAML error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    /// JSON error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Conversation error
    #[error("Conversation error: {0}")]
    ConversationError(String),
}

/// Validation error with location information
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// File path where error occurred
    pub path: String,
    /// Line number where error occurred
    pub line: usize,
    /// Column number where error occurred
    pub column: usize,
    /// Error message
    pub message: String,
    /// Error severity
    pub severity: Severity,
}

/// Severity level for validation errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Error - must be fixed
    Error,
    /// Warning - should be fixed
    Warning,
    /// Info - informational only
    Info,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} at {}:{}:{}: {}",
            self.severity, self.path, self.line, self.column, self.message
        )
    }
}
