/// Error types for the team collaboration system
use thiserror::Error;

/// Result type for team operations
pub type Result<T> = std::result::Result<T, TeamError>;

/// Errors that can occur in team operations
#[derive(Debug, Error)]
pub enum TeamError {
    #[error("Team not found: {0}")]
    TeamNotFound(String),

    #[error("Member not found: {0}")]
    MemberNotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Rule validation failed: {0}")]
    RuleValidationFailed(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Learning error: {0}")]
    LearningError(String),

    #[error("Permissions error: {0}")]
    PermissionsError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("YAML error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid role: {0}")]
    InvalidRole(String),

    #[error("Invalid scope: {0}")]
    InvalidScope(String),

    #[error("Concurrent modification detected")]
    ConcurrentModification,

    #[error("Operation timeout")]
    Timeout,

    #[error("Internal error: {0}")]
    Internal(String),
}
