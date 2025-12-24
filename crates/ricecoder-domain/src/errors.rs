//! Domain errors for RiceCoder

use thiserror::Error;

/// Core domain errors
#[derive(Error, Debug, Clone, PartialEq)]
pub enum DomainError {
    #[error("Invalid project name: {reason}")]
    InvalidProjectName { reason: String },

    #[error("Invalid file path: {reason}")]
    InvalidFilePath { reason: String },

    #[error("Invalid session state: {reason}")]
    InvalidSessionState { reason: String },

    #[error("Invalid provider configuration: {reason}")]
    InvalidProviderConfig { reason: String },

    #[error("Analysis failed: {reason}")]
    AnalysisFailed { reason: String },

    #[error("Validation error: {field} - {reason}")]
    ValidationError { field: String, reason: String },

    #[error("Business rule violation: {rule}")]
    BusinessRuleViolation { rule: String },

    #[error("Entity not found: {entity_type} with id {id}")]
    EntityNotFound { entity_type: String, id: String },

    #[error("Concurrency conflict: {resource}")]
    ConcurrencyConflict { resource: String },

    // Event-related errors (REQ-DOMAIN-000.8)
    #[error("Event dispatch failed: {reason}")]
    EventDispatchFailed { reason: String },

    #[error("Event serialization failed: {reason}")]
    EventSerializationFailed { reason: String },

    #[error("Event deserialization failed: {reason}")]
    EventDeserializationFailed { reason: String },

    #[error("Invalid event: {event_type} - {reason}")]
    InvalidEvent { event_type: String, reason: String },

    #[error("Event handler error: {handler} - {reason}")]
    EventHandlerError { handler: String, reason: String },
}

/// Result type alias for domain operations
pub type DomainResult<T> = Result<T, DomainError>;
