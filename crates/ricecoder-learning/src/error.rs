/// Learning system error types
use thiserror::Error;

/// Errors that can occur in the learning system
#[derive(Debug, Error)]
pub enum LearningError {
    #[error("Decision capture failed: {0}")]
    DecisionCaptureFailed(String),

    #[error("Rule validation failed: {0}")]
    RuleValidationFailed(String),

    #[error("Rule storage failed: {0}")]
    RuleStorageFailed(String),

    #[error("Pattern extraction failed: {0}")]
    PatternExtractionFailed(String),

    #[error("Rule promotion failed: {0}")]
    RulePromotionFailed(String),

    #[error("Conflict resolution failed: {0}")]
    ConflictResolutionFailed(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Path resolution failed: {0}")]
    PathResolutionFailed(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid scope: {0}")]
    InvalidScope(String),

    #[error("Rule not found: {0}")]
    RuleNotFound(String),

    #[error("Pattern not found: {0}")]
    PatternNotFound(String),

    #[error("Decision not found: {0}")]
    DecisionNotFound(String),

    #[error("Rule application failed: {0}")]
    RuleApplicationFailed(String),

    #[error("Analytics error: {0}")]
    AnalyticsError(String),
}

impl From<ricecoder_storage::error::StorageError> for LearningError {
    fn from(err: ricecoder_storage::error::StorageError) -> Self {
        LearningError::StorageError(err.to_string())
    }
}

/// Result type for learning system operations
pub type Result<T> = std::result::Result<T, LearningError>;
