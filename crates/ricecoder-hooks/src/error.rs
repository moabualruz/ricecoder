//! Error types for the hooks system
//!
//! This module defines comprehensive error types for the hooks system with clear
//! error messages and context. All errors use the `thiserror` crate for ergonomic
//! error handling.
//!
//! # Error Handling Patterns
//!
//! The hooks system uses explicit error types to enable proper error recovery:
//!
//! 1. **Hook Execution Errors**: When a hook fails to execute, the error is logged
//!    but other hooks continue executing (hook isolation).
//!
//! 2. **Configuration Errors**: Invalid configuration is rejected with clear messages
//!    indicating what's wrong and how to fix it.
//!
//! 3. **Timeout Errors**: Long-running hooks are terminated gracefully with a timeout
//!    error indicating how long the hook ran.
//!
//! 4. **Storage Errors**: Configuration loading failures are reported with context
//!    about which configuration file failed and why.
//!
//! # Examples
//!
//! Handling hook execution errors:
//!
//! ```ignore
//! match executor.execute_hook(&hook, &context) {
//!     Ok(result) => println!("Hook executed: {:?}", result),
//!     Err(HooksError::Timeout(ms)) => eprintln!("Hook timed out after {}ms", ms),
//!     Err(HooksError::ExecutionFailed(msg)) => eprintln!("Hook failed: {}", msg),
//!     Err(e) => eprintln!("Error: {}", e),
//! }
//! ```

use thiserror::Error;

/// Errors that can occur in the hooks system
///
/// This enum provides comprehensive error types for all operations in the hooks system.
/// Each variant includes context about what went wrong and how to recover.
#[derive(Debug, Error)]
pub enum HooksError {
    /// Hook not found in the registry
    ///
    /// This error occurs when trying to access a hook that doesn't exist.
    /// The string contains the hook ID that was not found.
    #[error("Hook not found: {0}")]
    HookNotFound(String),

    /// Invalid hook configuration
    ///
    /// This error occurs when hook configuration is invalid or malformed.
    /// The string contains details about what's wrong with the configuration.
    /// Common causes:
    /// - Missing required fields (event, action)
    /// - Invalid action type
    /// - Malformed YAML syntax
    #[error("Invalid hook configuration: {0}")]
    InvalidConfiguration(String),

    /// Hook execution failed
    ///
    /// This error occurs when a hook action fails to execute.
    /// The string contains details about what went wrong.
    /// Note: Hook failures don't affect other hooks (hook isolation).
    #[error("Hook execution failed: {0}")]
    ExecutionFailed(String),

    /// Hook execution timed out
    ///
    /// This error occurs when a hook takes longer than the configured timeout.
    /// The u64 contains the timeout duration in milliseconds.
    /// Recovery: Increase timeout or optimize the hook action.
    #[error("Hook execution timed out after {0}ms")]
    Timeout(u64),

    /// Hook is disabled
    ///
    /// This error occurs when trying to execute a disabled hook.
    /// The string contains the hook ID.
    /// Recovery: Enable the hook using `enable_hook()`.
    #[error("Hook is disabled: {0}")]
    HookDisabled(String),

    /// Storage or registry error
    ///
    /// This error occurs when there's a problem with hook storage or registry operations.
    /// The string contains details about what went wrong.
    /// Common causes:
    /// - Lock poisoning (concurrent access issue)
    /// - File system errors
    /// - Configuration file not found
    #[error("Storage error: {0}")]
    StorageError(String),

    /// Configuration validation error
    ///
    /// This error occurs when configuration validation fails.
    /// The string contains details about what validation failed.
    /// Common causes:
    /// - Missing required fields
    /// - Invalid field values
    /// - Schema validation failure
    #[error("Configuration validation error: {0}")]
    ValidationError(String),

    /// Variable substitution error
    ///
    /// This error occurs when variable substitution in hook parameters fails.
    /// The string contains details about what variable is missing or invalid.
    /// Common causes:
    /// - Missing variable in event context
    /// - Invalid variable syntax
    /// - Type mismatch in substitution
    #[error("Variable substitution error: {0}")]
    SubstitutionError(String),

    /// Condition evaluation error
    ///
    /// This error occurs when evaluating a hook condition fails.
    /// The string contains details about what went wrong.
    /// Common causes:
    /// - Invalid condition expression
    /// - Missing context variables
    /// - Type mismatch in condition
    #[error("Condition evaluation error: {0}")]
    ConditionError(String),

    /// Serialization error
    ///
    /// This error occurs when serializing or deserializing configuration.
    /// Wraps `serde_yaml::Error` for YAML parsing failures.
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_yaml::Error),

    /// IO error
    ///
    /// This error occurs when reading or writing files.
    /// Wraps `std::io::Error` for file system operations.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON error
    ///
    /// This error occurs when working with JSON data.
    /// Wraps `serde_json::Error` for JSON parsing failures.
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// Result type for hooks operations
///
/// This is the standard result type used throughout the hooks system.
/// All public APIs return `Result<T>` where `T` is the success type.
///
/// # Examples
///
/// ```ignore
/// fn register_hook(&mut self, hook: Hook) -> Result<String> {
///     // ... implementation ...
///     Ok(hook_id)
/// }
/// ```
pub type Result<T> = std::result::Result<T, HooksError>;
