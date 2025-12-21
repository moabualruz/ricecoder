//! Parser error types and results

use std::fmt;
use thiserror::Error;

/// Parser operation errors
#[derive(Debug, Error)]
pub enum ParserError {
    #[error("Language not supported: {language}")]
    UnsupportedLanguage { language: String },

    #[error("Parse error: {message}")]
    ParseError { message: String },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("UTF-8 conversion error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),

    #[error("Tree-sitter error: {message}")]
    TreeSitterError { message: String },

    #[error("Traversal error: {message}")]
    TraversalError { message: String },

    #[error("Cache error: {0}")]
    CacheError(#[from] ricecoder_cache::CacheError),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Invalid node type: {node_type}")]
    InvalidNodeType { node_type: String },

    #[error("Position out of bounds: line {line}, column {column}")]
    PositionOutOfBounds { line: usize, column: usize },
}

/// Result type for parser operations
pub type ParserResult<T> = std::result::Result<T, ParserError>;

/// Warning severity levels
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum WarningSeverity {
    Info,
    Warning,
    Error,
}

/// Parser warnings (non-fatal issues)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ParserWarning {
    pub message: String,
    pub position: Option<crate::types::Position>,
    pub severity: WarningSeverity,
}

impl std::fmt::Display for WarningSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WarningSeverity::Info => write!(f, "INFO"),
            WarningSeverity::Warning => write!(f, "WARN"),
            WarningSeverity::Error => write!(f, "ERROR"),
        }
    }
}

impl fmt::Display for ParserWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.severity, self.message)
    }
}

impl ParserWarning {
    /// Create a new parser warning
    pub fn new(message: String, severity: WarningSeverity) -> Self {
        Self {
            message,
            position: None,
            severity,
        }
    }

    /// Create a warning with position information
    pub fn with_position(
        message: String,
        position: crate::types::Position,
        severity: WarningSeverity,
    ) -> Self {
        Self {
            message,
            position: Some(position),
            severity,
        }
    }
}
