use thiserror::Error;

/// Errors that can occur in the modes system
#[derive(Debug, Error)]
pub enum ModeError {
    /// Mode not found with the given ID
    #[error("Mode not found: {0}")]
    NotFound(String),

    /// Invalid mode transition attempted
    #[error("Invalid mode transition: {0} -> {1}")]
    InvalidTransition(String, String),

    /// Capability not available in the specified mode
    #[error("Capability not available in {mode}: {capability}")]
    CapabilityNotAvailable {
        /// The mode name
        mode: String,
        /// The capability name
        capability: String,
    },

    /// Operation not allowed in the specified mode
    #[error("Operation not allowed in {mode}: {operation}")]
    OperationNotAllowed {
        /// The mode name
        mode: String,
        /// The operation name
        operation: String,
    },

    /// File operation blocked in Ask Mode
    #[error("File operation blocked in Ask Mode")]
    FileOperationBlocked,

    /// Processing failed with the given reason
    #[error("Processing failed: {0}")]
    ProcessingFailed(String),

    /// Think More processing timed out
    #[error("Think More timeout after {0}ms")]
    ThinkMoreTimeout(u64),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Session context error
    #[error("Session context error: {0}")]
    ContextError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// Result type for mode operations
pub type Result<T> = std::result::Result<T, ModeError>;
