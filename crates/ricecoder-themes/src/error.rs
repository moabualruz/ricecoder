//! Error types for the themes module

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ThemeError {
    #[error("Theme not found: {0}")]
    NotFound(String),

    #[error("Invalid theme format: {0}")]
    InvalidFormat(String),

    #[error("Theme validation failed: {0}")]
    ValidationError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_yaml::Error),

    #[error("Theme already exists: {0}")]
    AlreadyExists(String),

    #[error("Theme is currently in use: {0}")]
    InUse(String),
}

pub type Result<T> = std::result::Result<T, ThemeError>;
