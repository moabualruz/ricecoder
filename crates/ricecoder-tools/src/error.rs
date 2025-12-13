//! Error types for ricecoder-tools
//!
//! Provides structured error handling with context and suggestions for all tool operations.

use std::fmt;

/// Structured error type for tool operations
///
/// Includes error code, message, optional details, and suggestions for corrective action.
#[derive(Debug, Clone)]
pub struct ToolError {
    /// Machine-readable error code (e.g., "TIMEOUT", "INVALID_URL")
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// Additional context about the error
    pub details: Option<String>,
    /// Suggested corrective action
    pub suggestion: Option<String>,
}

impl ToolError {
    /// Create a new ToolError
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
            suggestion: None,
        }
    }

    /// Add details to the error
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    /// Add a suggestion to the error
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)?;
        if let Some(details) = &self.details {
            write!(f, " ({})", details)?;
        }
        if let Some(suggestion) = &self.suggestion {
            write!(f, " - Suggestion: {}", suggestion)?;
        }
        Ok(())
    }
}

impl std::error::Error for ToolError {}

/// Conversion from reqwest errors
impl From<reqwest::Error> for ToolError {
    fn from(err: reqwest::Error) -> Self {
        let (code, message) = if err.is_timeout() {
            ("TIMEOUT", "Request timeout")
        } else if err.is_connect() {
            ("NETWORK_ERROR", "Connection failed")
        } else if err.is_request() {
            ("REQUEST_ERROR", "Invalid request")
        } else if err.is_status() {
            ("HTTP_ERROR", "HTTP error response")
        } else {
            ("NETWORK_ERROR", "Network error")
        };

        ToolError::new(code, message)
            .with_details(err.to_string())
            .with_suggestion("Check your network connection and try again")
    }
}

/// Conversion from IO errors
impl From<std::io::Error> for ToolError {
    fn from(err: std::io::Error) -> Self {
        let (code, message) = match err.kind() {
            std::io::ErrorKind::NotFound => ("FILE_NOT_FOUND", "File not found"),
            std::io::ErrorKind::PermissionDenied => ("PERMISSION_DENIED", "Permission denied"),
            std::io::ErrorKind::InvalidInput => ("INVALID_INPUT", "Invalid input"),
            std::io::ErrorKind::TimedOut => ("TIMEOUT", "Operation timeout"),
            _ => ("IO_ERROR", "IO error"),
        };

        ToolError::new(code, message)
            .with_details(err.to_string())
    }
}

/// Conversion from serde_json errors
impl From<serde_json::Error> for ToolError {
    fn from(err: serde_json::Error) -> Self {
        ToolError::new("JSON_ERROR", "JSON parsing error")
            .with_details(err.to_string())
            .with_suggestion("Check the JSON format and try again")
    }
}


