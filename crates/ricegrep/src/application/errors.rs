//! Application Layer Errors
//!
//! Application-level errors that wrap domain errors and add
//! infrastructure concerns like I/O failures.

use std::fmt;
use std::io;
use crate::domain::DomainError;

/// Application-level error types
///
/// These errors represent failures that can occur in the application layer,
/// wrapping domain errors and adding infrastructure concerns.
#[derive(Debug)]
pub enum AppError {
    /// Domain validation/business rule errors
    Domain(DomainError),
    
    /// Input validation errors (before domain processing)
    Validation {
        message: String,
    },
    
    /// File I/O errors with context
    Io {
        operation: IoOperation,
        path: String,
        source: io::Error,
    },
    
    /// Index operation errors
    Index {
        operation: String,
        message: String,
    },
    
    /// Search operation errors
    Search {
        query: String,
        message: String,
    },
    
    /// Configuration errors
    Config(String),
}

/// Types of I/O operations for error context
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoOperation {
    Read,
    Write,
    Exists,
    Delete,
    Create,
}

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
            AppError::Io { operation, path, source } => {
                write!(f, "Failed to {} '{}': {}", operation, path, source)
            }
            AppError::Index { operation, message } => {
                write!(f, "Index {} failed: {}", operation, message)
            }
            AppError::Search { query, message } => {
                write!(f, "Search for '{}' failed: {}", query, message)
            }
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
    fn from(err: DomainError) -> Self {
        AppError::Domain(err)
    }
}

impl From<io::Error> for AppError {
    fn from(err: io::Error) -> Self {
        AppError::Io {
            operation: IoOperation::Read,
            path: String::new(),
            source: err,
        }
    }
}

/// Result type for application operations
pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_error_display() {
        let error = AppError::Config("missing key".to_string());
        assert_eq!(error.to_string(), "Configuration error: missing key");
    }

    #[test]
    fn test_io_operation_display() {
        assert_eq!(IoOperation::Read.to_string(), "read");
        assert_eq!(IoOperation::Write.to_string(), "write");
        assert_eq!(IoOperation::Exists.to_string(), "check existence");
    }

    #[test]
    fn test_app_error_from_domain() {
        let domain_err = DomainError::InvalidFilePath("empty".to_string());
        let app_err: AppError = domain_err.into();
        
        assert!(matches!(app_err, AppError::Domain(_)));
        assert!(app_err.to_string().contains("Invalid file path"));
    }

    #[test]
    fn test_io_error_with_context() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let app_err = AppError::Io {
            operation: IoOperation::Read,
            path: "test.rs".to_string(),
            source: io_err,
        };
        
        assert!(app_err.to_string().contains("read"));
        assert!(app_err.to_string().contains("test.rs"));
    }

    #[test]
    fn test_index_error() {
        let err = AppError::Index {
            operation: "update".to_string(),
            message: "corruption detected".to_string(),
        };
        
        assert!(err.to_string().contains("Index update failed"));
    }

    #[test]
    fn test_search_error() {
        let err = AppError::Search {
            query: "fn main".to_string(),
            message: "timeout".to_string(),
        };
        
        assert!(err.to_string().contains("fn main"));
    }
}
