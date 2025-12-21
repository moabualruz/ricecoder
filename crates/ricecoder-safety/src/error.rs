//! Error types for the safety crate

use thiserror::Error;

/// Result type for safety operations
pub type SafetyResult<T> = Result<T, SafetyError>;

/// Errors that can occur in safety and security operations
#[derive(Error, Debug)]
pub enum SafetyError {
    #[error("Security constraint violation: {constraint} - {message}")]
    ConstraintViolation { constraint: String, message: String },

    #[error("Risk threshold exceeded: score {score}, threshold {threshold}")]
    RiskThresholdExceeded { score: u8, threshold: u8 },

    #[error("Validation failed: {field} - {message}")]
    ValidationError { field: String, message: String },

    #[error("Approval required: {reason}")]
    ApprovalRequired { reason: String },

    #[error("Security policy violation: {policy} - {message}")]
    PolicyViolation { policy: String, message: String },

    #[error("Access denied: {resource} - {reason}")]
    AccessDenied { resource: String, reason: String },

    #[error("Configuration error: {field} - {message}")]
    ConfigError { field: String, message: String },

    #[error("Monitoring error: {message}")]
    MonitoringError { message: String },

    #[error("Audit logging error: {0}")]
    AuditError(#[from] ricecoder_activity_log::ActivityLogError),

    #[error("Security error: {0}")]
    SecurityError(String),
}
