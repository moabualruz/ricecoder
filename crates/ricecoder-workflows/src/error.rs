//! Error types for workflow operations

use thiserror::Error;

/// Errors that can occur during workflow operations
#[derive(Debug, Error)]
pub enum WorkflowError {
    /// Workflow not found
    #[error("Workflow not found: {0}")]
    NotFound(String),

    /// Invalid workflow definition
    #[error("Invalid workflow: {0}")]
    Invalid(String),

    /// Step execution failed
    #[error("Step failed: {0}")]
    StepFailed(String),

    /// Approval request timed out
    #[error("Approval timeout")]
    ApprovalTimeout,

    /// State management error
    #[error("State error: {0}")]
    StateError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_yaml::Error),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Result type for workflow operations
pub type WorkflowResult<T> = Result<T, WorkflowError>;
