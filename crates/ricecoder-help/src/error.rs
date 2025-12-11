//! Error types for the help system

use thiserror::Error;

/// Result type for help operations
pub type Result<T> = std::result::Result<T, HelpError>;

/// Error types for help system operations
#[derive(Debug, Error)]
pub enum HelpError {
    #[error("Help content not found: {0}")]
    ContentNotFound(String),
    
    #[error("Invalid search query: {0}")]
    InvalidSearchQuery(String),
    
    #[error("Help category not found: {0}")]
    CategoryNotFound(String),
    
    #[error("Help item not found: {0}")]
    ItemNotFound(String),
    
    #[error("Render error: {0}")]
    RenderError(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}