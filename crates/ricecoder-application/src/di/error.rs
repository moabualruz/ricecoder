//! DI Container Error Types
//!
//! REQ-ARCH-003.7: Clear error messages for service resolution failures

use thiserror::Error;

/// Errors that can occur during service container operations
#[derive(Debug, Error)]
pub enum ContainerError {
    /// Service was not registered in the container
    #[error("Service not registered: {0}")]
    ServiceNotRegistered(&'static str),

    /// Type mismatch during service resolution (internal error)
    #[error("Type mismatch for service: {0}")]
    TypeMismatch(&'static str),

    /// Circular dependency detected during resolution
    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),

    /// Service factory failed to create instance
    #[error("Factory error for service {service}: {message}")]
    FactoryError {
        service: &'static str,
        message: String,
    },

    /// Scope has been dropped and cannot be used
    #[error("Scope has been disposed")]
    ScopeDisposed,
}

impl ContainerError {
    /// Create a factory error
    pub fn factory_error<S: Into<String>>(service: &'static str, message: S) -> Self {
        Self::FactoryError {
            service,
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ContainerError::ServiceNotRegistered("MyService");
        assert_eq!(err.to_string(), "Service not registered: MyService");

        let err = ContainerError::TypeMismatch("MyService");
        assert_eq!(err.to_string(), "Type mismatch for service: MyService");

        let err = ContainerError::CircularDependency("A -> B -> A".to_string());
        assert_eq!(err.to_string(), "Circular dependency detected: A -> B -> A");

        let err = ContainerError::factory_error("MyService", "construction failed");
        assert_eq!(
            err.to_string(),
            "Factory error for service MyService: construction failed"
        );

        let err = ContainerError::ScopeDisposed;
        assert_eq!(err.to_string(), "Scope has been disposed");
    }
}
