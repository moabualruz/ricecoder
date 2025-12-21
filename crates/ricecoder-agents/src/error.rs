//! Error types for the agent framework

use thiserror::Error;

/// Errors that can occur during agent operations
#[derive(Debug, Clone, Error)]
pub enum AgentError {
    /// Agent not found
    #[error("Agent not found: {0}")]
    NotFound(String),

    /// Agent execution failed
    #[error("Agent execution failed: {0}")]
    ExecutionFailed(String),

    /// Agent timeout
    #[error("Agent timeout after {0}ms")]
    Timeout(u64),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Path resolution error
    #[error("Path resolution error: {0}")]
    PathError(String),

    /// Provider error
    #[error("Provider error: {0}")]
    ProviderError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Invalid input
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Authentication required
    #[error("Authentication required")]
    AuthenticationRequired,

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Access denied
    #[error("Access denied: {0}")]
    AccessDenied(String),

    /// Compliance violation
    #[error("Compliance violation: {0}")]
    ComplianceViolation(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl AgentError {
    /// Create a new NotFound error
    pub fn not_found(agent_id: impl Into<String>) -> Self {
        Self::NotFound(agent_id.into())
    }

    /// Create a new ExecutionFailed error
    pub fn execution_failed(reason: impl Into<String>) -> Self {
        Self::ExecutionFailed(reason.into())
    }

    /// Create a new Timeout error
    pub fn timeout(ms: u64) -> Self {
        Self::Timeout(ms)
    }

    /// Create a new ConfigError
    pub fn config_error(reason: impl Into<String>) -> Self {
        Self::ConfigError(reason.into())
    }

    /// Create a new PathError
    pub fn path_error(reason: impl Into<String>) -> Self {
        Self::PathError(reason.into())
    }

    /// Create a new ProviderError
    pub fn provider_error(reason: impl Into<String>) -> Self {
        Self::ProviderError(reason.into())
    }

    /// Create a new InvalidInput error
    pub fn invalid_input(reason: impl Into<String>) -> Self {
        Self::InvalidInput(reason.into())
    }

    /// Create a new AccessDenied error
    pub fn access_denied(reason: impl Into<String>) -> Self {
        Self::AccessDenied(reason.into())
    }

    /// Create a new ComplianceViolation error
    pub fn compliance_violation(reason: impl Into<String>) -> Self {
        Self::ComplianceViolation(reason.into())
    }

    /// Create a new Internal error
    pub fn internal(reason: impl Into<String>) -> Self {
        Self::Internal(reason.into())
    }
}

/// Result type for agent operations
pub type Result<T> = std::result::Result<T, AgentError>;

impl From<ricecoder_security::SecurityError> for AgentError {
    fn from(error: ricecoder_security::SecurityError) -> Self {
        match error {
            ricecoder_security::SecurityError::AccessDenied { message } => {
                AgentError::AccessDenied(message)
            }
            ricecoder_security::SecurityError::Validation { message } => {
                AgentError::ValidationError(message)
            }
            _ => AgentError::Internal(format!("Security error: {}", error)),
        }
    }
}
