//! Persistence Layer Error Types
//!
//! Error mapping to domain types

use thiserror::Error;

/// Errors that can occur during persistence operations
#[derive(Debug, Error)]
pub enum PersistenceError {
    /// Entity not found
    #[error("Entity not found: {entity_type} with id {id}")]
    NotFound {
        entity_type: &'static str,
        id: String,
    },

    /// Concurrency conflict (optimistic locking)
    #[error("Concurrency conflict: {0}")]
    ConcurrencyConflict(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Deserialization error
    #[error("Deserialization error: {0}")]
    Deserialization(String),

    /// Connection error
    #[error("Connection error: {0}")]
    Connection(String),

    /// Database error
    #[error("Database error: {0}")]
    Database(String),

    /// Lock acquisition failed
    #[error("Failed to acquire lock: {0}")]
    LockError(String),
}

impl PersistenceError {
    /// Create a not found error
    pub fn not_found(entity_type: &'static str, id: impl Into<String>) -> Self {
        Self::NotFound {
            entity_type,
            id: id.into(),
        }
    }

    /// Create a concurrency conflict error
    pub fn concurrency_conflict(message: impl Into<String>) -> Self {
        Self::ConcurrencyConflict(message.into())
    }
}

/// Convert persistence errors to domain errors
impl From<PersistenceError> for ricecoder_domain::errors::DomainError {
    fn from(err: PersistenceError) -> Self {
        match err {
            PersistenceError::NotFound { entity_type, id } => {
                ricecoder_domain::errors::DomainError::EntityNotFound {
                    entity_type: entity_type.to_string(),
                    id,
                }
            }
            PersistenceError::ConcurrencyConflict(msg) => {
                ricecoder_domain::errors::DomainError::ConcurrencyConflict { resource: msg }
            }
            PersistenceError::Serialization(msg) => {
                ricecoder_domain::errors::DomainError::EventSerializationFailed { reason: msg }
            }
            PersistenceError::Deserialization(msg) => {
                ricecoder_domain::errors::DomainError::EventDeserializationFailed { reason: msg }
            }
            PersistenceError::Connection(msg) | PersistenceError::Database(msg) => {
                ricecoder_domain::errors::DomainError::BusinessRuleViolation { 
                    rule: format!("Infrastructure error: {}", msg) 
                }
            }
            PersistenceError::LockError(msg) => {
                ricecoder_domain::errors::DomainError::BusinessRuleViolation { 
                    rule: format!("Lock error: {}", msg) 
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_found_error() {
        let err = PersistenceError::not_found("Project", "proj-123");
        assert!(err.to_string().contains("Project"));
        assert!(err.to_string().contains("proj-123"));
    }

    #[test]
    fn test_concurrency_error() {
        let err = PersistenceError::concurrency_conflict("version mismatch");
        assert!(err.to_string().contains("version mismatch"));
    }

    #[test]
    fn test_error_conversion() {
        let err = PersistenceError::not_found("Project", "123");
        let domain_err: ricecoder_domain::errors::DomainError = err.into();
        assert!(matches!(domain_err, ricecoder_domain::errors::DomainError::EntityNotFound { .. }));
    }
}
