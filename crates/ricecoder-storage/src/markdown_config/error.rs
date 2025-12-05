//! Error types for markdown configuration parsing and validation

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during markdown configuration parsing and validation
#[derive(Debug, Error)]
pub enum MarkdownConfigError {
    /// Failed to parse markdown content
    #[error("Failed to parse markdown: {message}")]
    ParseError { message: String },

    /// Invalid YAML frontmatter
    #[error("Invalid YAML frontmatter: {message}")]
    YamlError { message: String },

    /// Schema validation failed
    #[error("Schema validation failed: {message}")]
    ValidationError { message: String },

    /// Failed to load configuration file
    #[error("Failed to load configuration from {path}: {message}")]
    LoadError { path: PathBuf, message: String },

    /// Registration failed
    #[error("Registration failed: {message}")]
    RegistrationError { message: String },

    /// File watch error
    #[error("File watch error: {message}")]
    WatchError { message: String },

    /// Missing required field
    #[error("Missing required field: {field}")]
    MissingField { field: String },

    /// Invalid field value
    #[error("Invalid value for field '{field}': {message}")]
    InvalidFieldValue { field: String, message: String },

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl MarkdownConfigError {
    /// Create a parse error with context
    pub fn parse_error(message: impl Into<String>) -> Self {
        Self::ParseError {
            message: message.into(),
        }
    }

    /// Create a YAML error with context
    pub fn yaml_error(message: impl Into<String>) -> Self {
        Self::YamlError {
            message: message.into(),
        }
    }

    /// Create a validation error with context
    pub fn validation_error(message: impl Into<String>) -> Self {
        Self::ValidationError {
            message: message.into(),
        }
    }

    /// Create a load error with path and message
    pub fn load_error(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::LoadError {
            path: path.into(),
            message: message.into(),
        }
    }

    /// Create a registration error
    pub fn registration_error(message: impl Into<String>) -> Self {
        Self::RegistrationError {
            message: message.into(),
        }
    }

    /// Create a missing field error
    pub fn missing_field(field: impl Into<String>) -> Self {
        Self::MissingField {
            field: field.into(),
        }
    }

    /// Create an invalid field value error
    pub fn invalid_field_value(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::InvalidFieldValue {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Create a watch error
    pub fn watch_error(message: impl Into<String>) -> Self {
        Self::WatchError {
            message: message.into(),
        }
    }
}

/// Result type for markdown configuration operations
pub type MarkdownConfigResult<T> = Result<T, MarkdownConfigError>;
