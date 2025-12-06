//! Error types for the undo/redo system

use thiserror::Error;

/// Errors that can occur in the undo/redo system
#[derive(Debug, Error)]
pub enum UndoRedoError {
    /// Change not found in history
    #[error("Change not found: {0}")]
    ChangeNotFound(String),

    /// Checkpoint not found
    #[error("Checkpoint not found: {0}")]
    CheckpointNotFound(String),

    /// No more undos available
    #[error("No more undos available")]
    NoMoreUndos,

    /// No more redos available
    #[error("No more redos available")]
    NoMoreRedos,

    /// Storage error
    #[error("Storage error: {0}")]
    StorageError(String),

    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl UndoRedoError {
    /// Create a new ChangeNotFound error with context
    pub fn change_not_found(id: impl Into<String>) -> Self {
        Self::ChangeNotFound(id.into())
    }

    /// Create a new CheckpointNotFound error with context
    pub fn checkpoint_not_found(id: impl Into<String>) -> Self {
        Self::CheckpointNotFound(id.into())
    }

    /// Create a new StorageError with context
    pub fn storage_error(msg: impl Into<String>) -> Self {
        Self::StorageError(msg.into())
    }

    /// Create a new ValidationError with context
    pub fn validation_error(msg: impl Into<String>) -> Self {
        Self::ValidationError(msg.into())
    }
}
