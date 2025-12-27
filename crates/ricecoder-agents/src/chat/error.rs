//! Chat service error types
//!
//! Ported from crustly's agent error handling.

use thiserror::Error;
use uuid::Uuid;

/// Result type for chat service operations
pub type Result<T> = std::result::Result<T, ChatError>;

/// Errors that can occur during chat service operations
#[derive(Debug, Error)]
pub enum ChatError {
    /// Session not found
    #[error("Session not found: {0}")]
    SessionNotFound(Uuid),

    /// Provider error during LLM communication
    #[error("Provider error: {0}")]
    Provider(String),

    /// Tool execution error
    #[error("Tool execution failed: {0}")]
    ToolExecution(String),

    /// Maximum tool iterations exceeded
    #[error("Maximum tool iterations ({0}) exceeded")]
    MaxIterationsExceeded(usize),

    /// Tool not found in registry
    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    /// Tool approval denied by user
    #[error("Tool approval denied: {0}")]
    ApprovalDenied(String),

    /// Context window exceeded
    #[error("Context window exceeded: {current} tokens (max: {max})")]
    ContextExceeded { current: usize, max: usize },

    /// Database/persistence error
    #[error("Database error: {0}")]
    Database(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),

    /// Timeout error
    #[error("Operation timed out after {0} seconds")]
    Timeout(u64),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl From<serde_json::Error> for ChatError {
    fn from(e: serde_json::Error) -> Self {
        ChatError::Serialization(e.to_string())
    }
}

impl From<ricecoder_providers::error::ProviderError> for ChatError {
    fn from(e: ricecoder_providers::error::ProviderError) -> Self {
        ChatError::Provider(e.to_string())
    }
}
