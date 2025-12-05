//! Error types for execution module

use thiserror::Error;

/// Errors that can occur during execution planning and execution
#[derive(Debug, Error)]
pub enum ExecutionError {
    /// Plan building failed
    #[error("Plan building failed: {0}")]
    PlanError(String),

    /// Step execution failed
    #[error("Step failed: {0}")]
    StepFailed(String),

    /// Tests failed with specified number of failures
    #[error("Tests failed: {0} failures")]
    TestsFailed(usize),

    /// Rollback failed
    #[error("Rollback failed: {0}")]
    RollbackFailed(String),

    /// Approval was denied
    #[error("Approval denied")]
    ApprovalDenied,

    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Timeout error
    #[error("Timeout after {0}ms")]
    Timeout(u64),

    /// State persistence error
    #[error("State persistence error: {0}")]
    StatePersistenceError(String),
}

impl PartialEq for ExecutionError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ExecutionError::PlanError(a), ExecutionError::PlanError(b)) => a == b,
            (ExecutionError::StepFailed(a), ExecutionError::StepFailed(b)) => a == b,
            (ExecutionError::TestsFailed(a), ExecutionError::TestsFailed(b)) => a == b,
            (ExecutionError::RollbackFailed(a), ExecutionError::RollbackFailed(b)) => a == b,
            (ExecutionError::ApprovalDenied, ExecutionError::ApprovalDenied) => true,
            (ExecutionError::ValidationError(a), ExecutionError::ValidationError(b)) => a == b,
            (ExecutionError::ConfigError(a), ExecutionError::ConfigError(b)) => a == b,
            (ExecutionError::SerializationError(a), ExecutionError::SerializationError(b)) => {
                a == b
            }
            (ExecutionError::Timeout(a), ExecutionError::Timeout(b)) => a == b,
            (
                ExecutionError::StatePersistenceError(a),
                ExecutionError::StatePersistenceError(b),
            ) => a == b,
            // IoError comparison: compare error kinds
            (ExecutionError::IoError(a), ExecutionError::IoError(b)) => a.kind() == b.kind(),
            _ => false,
        }
    }
}

/// Result type for execution operations
pub type ExecutionResult<T> = Result<T, ExecutionError>;
