//! Application Layer Errors

use std::fmt;
use std::io;
use crate::domain::DomainError;

/// Application-level error types
#[derive(Debug)]
pub enum AppError {
    Domain(DomainError),
    Validation { message: String },
    Io { operation: IoOperation, path: String, source: io::Error },
    Index { operation: String, message: String },
    Search { query: String, message: String },
    Config(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoOperation { Read, Write, Exists, Delete, Create }

impl fmt::Display for IoOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IoOperation::Read => write!(f, "read"),
            IoOperation::Write => write!(f, "write"),
            IoOperation::Exists => write!(f, "check existence"),
            IoOperation::Delete => write!(f, "delete"),
            IoOperation::Create => write!(f, "create"),
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Domain(err) => write!(f, "{}", err),
            AppError::Validation { message } => write!(f, "Validation error: {}", message),
            AppError::Io { operation, path, source } => write!(f, "Failed to {} '{}': {}", operation, path, source),
            AppError::Index { operation, message } => write!(f, "Index {} failed: {}", operation, message),
            AppError::Search { query, message } => write!(f, "Search for '{}' failed: {}", query, message),
            AppError::Config(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::Domain(err) => Some(err),
            AppError::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl From<DomainError> for AppError {
    fn from(err: DomainError) -> Self { AppError::Domain(err) }
}

impl From<io::Error> for AppError {
    fn from(err: io::Error) -> Self {
        AppError::Io { operation: IoOperation::Read, path: String::new(), source: err }
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_error_display() {
        let error = AppError::Config("missing key".to_string());
        assert!(error.to_string().contains("missing key"));
    }
}
