//! Error types for domain-specific agents

use thiserror::Error;

/// Errors that can occur during domain agent operations
#[derive(Debug, Clone, Error)]
pub enum DomainError {
    /// Domain agent not found
    #[error("Domain agent not found: {0}")]
    AgentNotFound(String),

    /// Domain not found
    #[error("Domain not found: {0}")]
    DomainNotFound(String),

    /// Knowledge not found
    #[error("Knowledge not found: {0}")]
    KnowledgeNotFound(String),

    /// Context error
    #[error("Context error: {0}")]
    ContextError(String),

    /// Conflict error
    #[error("Conflict error: {0}")]
    ConflictError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Coordination error
    #[error("Coordination error: {0}")]
    CoordinationError(String),

    /// Invalid input
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl DomainError {
    /// Create a new AgentNotFound error
    pub fn agent_not_found(domain: impl Into<String>) -> Self {
        Self::AgentNotFound(domain.into())
    }

    /// Create a new DomainNotFound error
    pub fn domain_not_found(domain: impl Into<String>) -> Self {
        Self::DomainNotFound(domain.into())
    }

    /// Create a new KnowledgeNotFound error
    pub fn knowledge_not_found(knowledge: impl Into<String>) -> Self {
        Self::KnowledgeNotFound(knowledge.into())
    }

    /// Create a new ContextError
    pub fn context_error(reason: impl Into<String>) -> Self {
        Self::ContextError(reason.into())
    }

    /// Create a new ConflictError
    pub fn conflict_error(reason: impl Into<String>) -> Self {
        Self::ConflictError(reason.into())
    }

    /// Create a new ConfigError
    pub fn config_error(reason: impl Into<String>) -> Self {
        Self::ConfigError(reason.into())
    }

    /// Create a new CoordinationError
    pub fn coordination_error(reason: impl Into<String>) -> Self {
        Self::CoordinationError(reason.into())
    }

    /// Create a new InvalidInput error
    pub fn invalid_input(reason: impl Into<String>) -> Self {
        Self::InvalidInput(reason.into())
    }

    /// Create a new SerializationError
    pub fn serialization_error(reason: impl Into<String>) -> Self {
        Self::SerializationError(reason.into())
    }

    /// Create a new Internal error
    pub fn internal(reason: impl Into<String>) -> Self {
        Self::Internal(reason.into())
    }
}

/// Result type for domain agent operations
pub type DomainResult<T> = std::result::Result<T, DomainError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_error_agent_not_found() {
        let error = DomainError::agent_not_found("web");
        assert!(matches!(error, DomainError::AgentNotFound(_)));
        assert_eq!(error.to_string(), "Domain agent not found: web");
    }

    #[test]
    fn test_domain_error_domain_not_found() {
        let error = DomainError::domain_not_found("mobile");
        assert!(matches!(error, DomainError::DomainNotFound(_)));
        assert_eq!(error.to_string(), "Domain not found: mobile");
    }

    #[test]
    fn test_domain_error_knowledge_not_found() {
        let error = DomainError::knowledge_not_found("react");
        assert!(matches!(error, DomainError::KnowledgeNotFound(_)));
        assert_eq!(error.to_string(), "Knowledge not found: react");
    }

    #[test]
    fn test_domain_error_context_error() {
        let error = DomainError::context_error("invalid context");
        assert!(matches!(error, DomainError::ContextError(_)));
        assert_eq!(error.to_string(), "Context error: invalid context");
    }

    #[test]
    fn test_domain_error_conflict_error() {
        let error = DomainError::conflict_error("conflicting recommendations");
        assert!(matches!(error, DomainError::ConflictError(_)));
        assert_eq!(
            error.to_string(),
            "Conflict error: conflicting recommendations"
        );
    }

    #[test]
    fn test_domain_error_config_error() {
        let error = DomainError::config_error("invalid config");
        assert!(matches!(error, DomainError::ConfigError(_)));
        assert_eq!(error.to_string(), "Configuration error: invalid config");
    }

    #[test]
    fn test_domain_error_coordination_error() {
        let error = DomainError::coordination_error("coordination failed");
        assert!(matches!(error, DomainError::CoordinationError(_)));
        assert_eq!(error.to_string(), "Coordination error: coordination failed");
    }

    #[test]
    fn test_domain_error_invalid_input() {
        let error = DomainError::invalid_input("invalid data");
        assert!(matches!(error, DomainError::InvalidInput(_)));
        assert_eq!(error.to_string(), "Invalid input: invalid data");
    }

    #[test]
    fn test_domain_error_serialization_error() {
        let error = DomainError::serialization_error("invalid json");
        assert!(matches!(error, DomainError::SerializationError(_)));
        assert_eq!(error.to_string(), "Serialization error: invalid json");
    }

    #[test]
    fn test_domain_error_internal() {
        let error = DomainError::internal("internal error");
        assert!(matches!(error, DomainError::Internal(_)));
        assert_eq!(error.to_string(), "Internal error: internal error");
    }

    #[test]
    fn test_domain_error_clone() {
        let error = DomainError::agent_not_found("web");
        let cloned = error.clone();
        assert_eq!(error.to_string(), cloned.to_string());
    }

    #[test]
    fn test_result_type_ok() {
        let result: DomainResult<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_result_type_err() {
        let result: DomainResult<i32> = Err(DomainError::agent_not_found("web"));
        assert!(result.is_err());
    }
}
