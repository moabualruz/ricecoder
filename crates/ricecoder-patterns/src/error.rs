//! Error types for pattern detection

use thiserror::Error;

/// Result type for pattern detection operations
pub type PatternResult<T> = Result<T, PatternError>;

/// Errors that can occur during pattern detection
#[derive(Debug, Error)]
pub enum PatternError {
    /// I/O error during file operations
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Parsing error from underlying parser
    #[error("Parsing error: {0}")]
    Parsing(String),

    /// Analysis error during pattern detection
    #[error("Analysis error: {0}")]
    Analysis(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Cache error
    #[error("Cache error: {0}")]
    Cache(String),

    /// Pattern detection timeout
    #[error("Pattern detection timed out")]
    Timeout,

    /// Invalid pattern data
    #[error("Invalid pattern data: {0}")]
    InvalidData(String),
}
