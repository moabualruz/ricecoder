//! Result types for tool operations
//!
//! Provides structured result handling with metadata about operation execution.

use crate::error::ToolError;
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Metadata about a tool operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultMetadata {
    /// Duration of the operation in milliseconds
    pub duration_ms: u64,
    /// Provider that executed the operation ("mcp" or "builtin")
    pub provider: String,
    /// Timestamp when the operation completed (ISO 8601 format)
    pub timestamp: String,
}

impl ResultMetadata {
    /// Create new result metadata
    pub fn new(duration_ms: u64, provider: impl Into<String>) -> Self {
        Self {
            duration_ms,
            provider: provider.into(),
            timestamp: Utc::now().to_rfc3339(),
        }
    }
}

/// Structured result for tool operations
///
/// Contains success status, optional data, optional error, and metadata about execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult<T> {
    /// Whether the operation succeeded
    pub success: bool,
    /// Operation result data (if successful)
    pub data: Option<T>,
    /// Error information (if failed)
    pub error: Option<ToolErrorInfo>,
    /// Metadata about the operation
    pub metadata: ResultMetadata,
}

/// Serializable error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolErrorInfo {
    /// Machine-readable error code
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// Additional context
    pub details: Option<String>,
    /// Suggested corrective action
    pub suggestion: Option<String>,
}

impl From<&ToolError> for ToolErrorInfo {
    fn from(err: &ToolError) -> Self {
        Self {
            code: err.code.clone(),
            message: err.message.clone(),
            details: err.details.clone(),
            suggestion: err.suggestion.clone(),
        }
    }
}

impl<T> ToolResult<T> {
    /// Create a successful result
    pub fn ok(data: T, duration_ms: u64, provider: impl Into<String>) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            metadata: ResultMetadata::new(duration_ms, provider),
        }
    }

    /// Create a failed result
    pub fn err(error: ToolError, duration_ms: u64, provider: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ToolErrorInfo::from(&error)),
            metadata: ResultMetadata::new(duration_ms, provider),
        }
    }
}


