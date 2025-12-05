//! Storage error types for RiceCoder

use std::path::PathBuf;
use thiserror::Error;

/// Result type for storage operations
pub type StorageResult<T> = Result<T, StorageError>;

/// Storage error types
#[derive(Error, Debug)]
pub enum StorageError {
    /// Directory creation failed
    #[error("Directory creation failed for {path}: {source}")]
    DirectoryCreationFailed {
        path: PathBuf,
        source: std::io::Error,
    },

    /// File read/write failed
    #[error("IO error on {path} ({operation}): {source}")]
    IoError {
        path: PathBuf,
        operation: IoOperation,
        source: std::io::Error,
    },

    /// Configuration parsing failed
    #[error("Failed to parse {path} as {format}: {message}")]
    ParseError {
        path: PathBuf,
        format: String,
        message: String,
    },

    /// Invalid configuration value
    #[error("Invalid configuration value for {field}: {message}")]
    ValidationError { field: String, message: String },

    /// Path resolution failed
    #[error("Path resolution failed: {message}")]
    PathResolutionError { message: String },

    /// Environment variable error
    #[error("Environment variable error for {var_name}: {message}")]
    EnvVarError { var_name: String, message: String },

    /// Relocation failed
    #[error("Failed to relocate storage from {from} to {to}: {message}")]
    RelocationError {
        from: PathBuf,
        to: PathBuf,
        message: String,
    },

    /// Offline mode - storage unavailable
    #[error("Storage unavailable at {path}: {message}")]
    StorageUnavailable { path: PathBuf, message: String },

    /// First-run confirmation required
    #[error("First-run confirmation required. Suggested path: {suggested_path}")]
    FirstRunConfirmationRequired { suggested_path: PathBuf },

    /// Generic IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

/// IO operation type for error context
#[derive(Debug, Clone, Copy)]
pub enum IoOperation {
    Read,
    Write,
    Delete,
    Move,
}

impl std::fmt::Display for IoOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IoOperation::Read => write!(f, "read"),
            IoOperation::Write => write!(f, "write"),
            IoOperation::Delete => write!(f, "delete"),
            IoOperation::Move => write!(f, "move"),
        }
    }
}

impl StorageError {
    /// Create a directory creation failed error
    pub fn directory_creation_failed(path: PathBuf, source: std::io::Error) -> Self {
        StorageError::DirectoryCreationFailed { path, source }
    }

    /// Create an IO error
    pub fn io_error(path: PathBuf, operation: IoOperation, source: std::io::Error) -> Self {
        StorageError::IoError {
            path,
            operation,
            source,
        }
    }

    /// Create a parse error
    pub fn parse_error(
        path: PathBuf,
        format: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        StorageError::ParseError {
            path,
            format: format.into(),
            message: message.into(),
        }
    }

    /// Create a validation error
    pub fn validation_error(field: impl Into<String>, message: impl Into<String>) -> Self {
        StorageError::ValidationError {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Create a path resolution error
    pub fn path_resolution_error(message: impl Into<String>) -> Self {
        StorageError::PathResolutionError {
            message: message.into(),
        }
    }

    /// Create an environment variable error
    pub fn env_var_error(var_name: impl Into<String>, message: impl Into<String>) -> Self {
        StorageError::EnvVarError {
            var_name: var_name.into(),
            message: message.into(),
        }
    }

    /// Create a relocation error
    pub fn relocation_error(from: PathBuf, to: PathBuf, message: impl Into<String>) -> Self {
        StorageError::RelocationError {
            from,
            to,
            message: message.into(),
        }
    }

    /// Create a storage unavailable error
    pub fn storage_unavailable(path: PathBuf, message: impl Into<String>) -> Self {
        StorageError::StorageUnavailable {
            path,
            message: message.into(),
        }
    }

    /// Create a first-run confirmation required error
    pub fn first_run_confirmation_required(suggested_path: PathBuf) -> Self {
        StorageError::FirstRunConfirmationRequired { suggested_path }
    }

    /// Create an internal error
    pub fn internal(message: impl Into<String>) -> Self {
        StorageError::Internal(message.into())
    }
}
