//! Application layer error types
//!
//! These errors represent application-level failures that are suitable
//! for API/UI consumption. They wrap domain errors with additional context.

use thiserror::Error;

/// Application layer result type
pub type ApplicationResult<T> = Result<T, ApplicationError>;

/// Application layer errors
///
/// These errors provide context suitable for external consumers (API, UI)
/// while hiding internal domain implementation details.
#[derive(Error, Debug, Clone)]
pub enum ApplicationError {
    // === Validation Errors ===
    
    /// Input validation failed
    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    /// Required field missing
    #[error("Required field missing: {0}")]
    RequiredFieldMissing(String),

    // === Not Found Errors ===
    
    /// Project not found
    #[error("Project not found: {0}")]
    ProjectNotFound(String),

    /// Session not found
    #[error("Session not found: {0}")]
    SessionNotFound(String),

    /// Specification not found
    #[error("Specification not found: {0}")]
    SpecificationNotFound(String),

    // === Conflict Errors ===
    
    /// Project with this name already exists
    #[error("Project already exists with name: {0}")]
    ProjectAlreadyExists(String),

    /// Specification already exists
    #[error("Specification already exists: {0}")]
    SpecificationAlreadyExists(String),

    // === Business Rule Violations ===
    
    /// Operation not allowed in current state
    #[error("Operation not allowed: {0}")]
    OperationNotAllowed(String),

    /// Business rule violation
    #[error("Business rule violation: {0}")]
    BusinessRuleViolation(String),

    // === Infrastructure Errors ===
    
    /// Repository operation failed
    #[error("Repository error: {0}")]
    RepositoryError(String),

    /// Transaction failed
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    /// Event publication failed
    #[error("Event publication failed: {0}")]
    EventPublicationFailed(String),

    // === Domain Error Wrapper ===
    
    /// Wrapped domain error
    #[error("Domain error: {0}")]
    DomainError(String),
}

impl From<ricecoder_domain::errors::DomainError> for ApplicationError {
    fn from(err: ricecoder_domain::errors::DomainError) -> Self {
        match err {
            ricecoder_domain::errors::DomainError::InvalidProjectName { reason } => {
                ApplicationError::ValidationFailed(format!("Invalid project name: {}", reason))
            }
            ricecoder_domain::errors::DomainError::InvalidFilePath { reason } => {
                ApplicationError::ValidationFailed(format!("Invalid file path: {}", reason))
            }
            ricecoder_domain::errors::DomainError::BusinessRuleViolation { rule } => {
                ApplicationError::BusinessRuleViolation(rule)
            }
            ricecoder_domain::errors::DomainError::ValidationError { field, reason } => {
                ApplicationError::ValidationFailed(format!("{}: {}", field, reason))
            }
            ricecoder_domain::errors::DomainError::InvalidSessionState { reason } => {
                ApplicationError::OperationNotAllowed(format!("Invalid session state: {}", reason))
            }
            other => ApplicationError::DomainError(other.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error_display() {
        let err = ApplicationError::ValidationFailed("name is required".into());
        assert_eq!(err.to_string(), "Validation failed: name is required");
    }

    #[test]
    fn test_not_found_error_display() {
        let err = ApplicationError::ProjectNotFound("proj-123".into());
        assert_eq!(err.to_string(), "Project not found: proj-123");
    }

    #[test]
    fn test_domain_error_conversion() {
        let domain_err = ricecoder_domain::errors::DomainError::InvalidProjectName {
            reason: "too short".into(),
        };
        let app_err: ApplicationError = domain_err.into();
        assert!(matches!(app_err, ApplicationError::ValidationFailed(_)));
    }
}
