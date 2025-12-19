//! Error types for RiceGrep

use std::io;
use thiserror::Error;

/// Main error type for RiceGrep operations
#[derive(Debug, Error)]
pub enum RiceGrepError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Invalid regex pattern: {0}")]
    Regex(#[from] regex::Error),

    #[error("Invalid UTF-8 in file: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("File walking error: {0}")]
    Walk(#[from] ignore::Error),

    #[error("Search operation failed: {message}")]
    Search { message: String },

    #[error("AI processing failed: {message}")]
    Ai { message: String },

    #[error("Index operation failed: {message}")]
    Index { message: String },

    #[error("Spelling correction failed: {message}")]
    Spelling { message: String },

    #[error("Configuration error: {message}")]
    Config { message: String },

    #[error("File watching error: {0}")]
    Watch(#[from] notify::Error),
}