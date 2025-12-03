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

    /// Create a new Internal error
    pub fn internal(reason: impl Into<String>) -> Self {
        Self::Internal(reason.into())
    }
}

/// Result type for agent operations
pub type Result<T> = std::result::Result<T, AgentError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_error_not_found() {
        let error = AgentError::not_found("test-agent");
        assert!(matches!(error, AgentError::NotFound(_)));
        assert_eq!(error.to_string(), "Agent not found: test-agent");
    }

    #[test]
    fn test_agent_error_execution_failed() {
        let error = AgentError::execution_failed("execution error");
        assert!(matches!(error, AgentError::ExecutionFailed(_)));
        assert_eq!(error.to_string(), "Agent execution failed: execution error");
    }

    #[test]
    fn test_agent_error_timeout() {
        let error = AgentError::timeout(5000);
        assert!(matches!(error, AgentError::Timeout(_)));
        assert_eq!(error.to_string(), "Agent timeout after 5000ms");
    }

    #[test]
    fn test_agent_error_config_error() {
        let error = AgentError::config_error("invalid config");
        assert!(matches!(error, AgentError::ConfigError(_)));
        assert_eq!(error.to_string(), "Configuration error: invalid config");
    }

    #[test]
    fn test_agent_error_path_error() {
        let error = AgentError::path_error("path not found");
        assert!(matches!(error, AgentError::PathError(_)));
        assert_eq!(error.to_string(), "Path resolution error: path not found");
    }

    #[test]
    fn test_agent_error_provider_error() {
        let error = AgentError::provider_error("provider unavailable");
        assert!(matches!(error, AgentError::ProviderError(_)));
        assert_eq!(error.to_string(), "Provider error: provider unavailable");
    }

    #[test]
    fn test_agent_error_invalid_input() {
        let error = AgentError::invalid_input("invalid data");
        assert!(matches!(error, AgentError::InvalidInput(_)));
        assert_eq!(error.to_string(), "Invalid input: invalid data");
    }

    #[test]
    fn test_agent_error_internal() {
        let error = AgentError::internal("internal error");
        assert!(matches!(error, AgentError::Internal(_)));
        assert_eq!(error.to_string(), "Internal error: internal error");
    }

    #[test]
    fn test_agent_error_serialization_error() {
        let error = AgentError::SerializationError("invalid json".to_string());
        assert!(matches!(error, AgentError::SerializationError(_)));
    }

    #[test]
    fn test_agent_error_clone() {
        let error = AgentError::not_found("test-agent");
        let cloned = error.clone();
        assert_eq!(error.to_string(), cloned.to_string());
    }

    #[test]
    fn test_error_display_trait() {
        let error = AgentError::not_found("agent-1");
        let display_string = format!("{}", error);
        assert_eq!(display_string, "Agent not found: agent-1");
    }

    #[test]
    fn test_error_debug_trait() {
        let error = AgentError::timeout(1000);
        let debug_string = format!("{:?}", error);
        assert!(debug_string.contains("Timeout"));
    }

    #[test]
    fn test_result_type_ok() {
        let result: Result<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_result_type_err() {
        let result: Result<i32> = Err(AgentError::not_found("test"));
        assert!(result.is_err());
    }
}
