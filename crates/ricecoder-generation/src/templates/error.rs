//! Error types for template operations

use thiserror::Error;

/// Errors that can occur during template operations
#[derive(Debug, Error)]
pub enum TemplateError {
    /// Template not found
    #[error("Template not found: {0}")]
    NotFound(String),

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
}

impl PartialEq for TemplateError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TemplateError::NotFound(a), TemplateError::NotFound(b)) => a == b,
            (TemplateError::MissingPlaceholder(a), TemplateError::MissingPlaceholder(b)) => a == b,
            (
                TemplateError::InvalidSyntax {
                    line: line_a,
                    message: msg_a,
                },
                TemplateError::InvalidSyntax {
                    line: line_b,
                    message: msg_b,
                },
            ) => line_a == line_b && msg_a == msg_b,
            (TemplateError::RenderError(a), TemplateError::RenderError(b)) => a == b,
            (TemplateError::ValidationFailed(a), TemplateError::ValidationFailed(b)) => a == b,
            (TemplateError::IoError(_), TemplateError::IoError(_)) => {
                // IO errors can't be compared, so we consider them equal if both are IO errors
                true
            }
            _ => false,
        }
    }
}

/// Errors that can occur during boilerplate operations
#[derive(Debug, Error)]
pub enum BoilerplateError {
    /// Boilerplate not found
    #[error("Boilerplate not found: {0}")]
    NotFound(String),

    /// Invalid boilerplate structure
    #[error("Invalid boilerplate structure: {0}")]
    InvalidStructure(String),

    /// File conflict during scaffolding
    #[error("File conflict at {path}: {reason}")]
    FileConflict {
        /// Path to the conflicting file
        path: String,
        /// Reason for the conflict
        reason: String,
    },

    /// Boilerplate validation failed
    #[error("Boilerplate validation failed: {0}")]
    ValidationFailed(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl PartialEq for BoilerplateError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (BoilerplateError::NotFound(a), BoilerplateError::NotFound(b)) => a == b,
            (BoilerplateError::InvalidStructure(a), BoilerplateError::InvalidStructure(b)) => a == b,
            (
                BoilerplateError::FileConflict {
                    path: path_a,
                    reason: reason_a,
                },
                BoilerplateError::FileConflict {
                    path: path_b,
                    reason: reason_b,
                },
            ) => path_a == path_b && reason_a == reason_b,
            (BoilerplateError::ValidationFailed(a), BoilerplateError::ValidationFailed(b)) => a == b,
            (BoilerplateError::IoError(_), BoilerplateError::IoError(_)) => {
                // IO errors can't be compared, so we consider them equal if both are IO errors
                true
            }
            _ => false,
        }
    }
}
