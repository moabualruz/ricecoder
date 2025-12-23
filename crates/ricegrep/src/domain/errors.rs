//! Domain Error Types
//!
//! Pure domain errors with no external dependencies.
//! These represent business rule violations and validation failures.

use std::fmt;

/// Domain-level error types for validation and business rule violations
#[derive(Debug, Clone, PartialEq)]
pub enum DomainError {
    /// File path validation errors
    InvalidFilePath(String),
    /// Edit pattern validation errors  
    InvalidEditPattern(String),
    /// Search query validation errors
    InvalidSearchQuery(String),
    /// File edit business rule violations
    InvalidFileEdit(String),
    /// General validation error
    ValidationError(String),
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainError::InvalidFilePath(msg) => write!(f, "Invalid file path: {}", msg),
            DomainError::InvalidEditPattern(msg) => write!(f, "Invalid edit pattern: {}", msg),
            DomainError::InvalidSearchQuery(msg) => write!(f, "Invalid search query: {}", msg),
            DomainError::InvalidFileEdit(msg) => write!(f, "Invalid file edit: {}", msg),
            DomainError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl std::error::Error for DomainError {}

/// Result type for domain operations
pub type DomainResult<T> = Result<T, DomainError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_error_display() {
        let error = DomainError::InvalidFilePath("not found".to_string());
        assert_eq!(error.to_string(), "Invalid file path: not found");
    }

    #[test]
    fn test_domain_error_clone() {
        let error = DomainError::ValidationError("test".to_string());
        let cloned = error.clone();
        assert_eq!(error, cloned);
    }

    #[test]
    fn test_domain_result_type() {
        let success: DomainResult<i32> = Ok(42);
        let failure: DomainResult<i32> = Err(DomainError::ValidationError("test".to_string()));
        
        assert!(success.is_ok());
        assert!(failure.is_err());
    }
}