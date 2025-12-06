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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_metadata_creation() {
        let metadata = ResultMetadata::new(100, "builtin");
        assert_eq!(metadata.duration_ms, 100);
        assert_eq!(metadata.provider, "builtin");
        assert!(!metadata.timestamp.is_empty());
    }

    #[test]
    fn test_tool_result_ok() {
        let result = ToolResult::ok("test data", 50, "mcp");
        assert!(result.success);
        assert_eq!(result.data, Some("test data"));
        assert!(result.error.is_none());
        assert_eq!(result.metadata.duration_ms, 50);
        assert_eq!(result.metadata.provider, "mcp");
    }

    #[test]
    fn test_tool_result_err() {
        let error = ToolError::new("TEST_ERROR", "Test message");
        let result = ToolResult::<String>::err(error, 25, "builtin");
        assert!(!result.success);
        assert!(result.data.is_none());
        assert!(result.error.is_some());
        assert_eq!(result.metadata.duration_ms, 25);
    }

    #[test]
    fn test_tool_error_info_conversion() {
        let error = ToolError::new("TEST_ERROR", "Test message")
            .with_details("Details")
            .with_suggestion("Suggestion");
        let error_info = ToolErrorInfo::from(&error);
        assert_eq!(error_info.code, "TEST_ERROR");
        assert_eq!(error_info.message, "Test message");
        assert_eq!(error_info.details, Some("Details".to_string()));
        assert_eq!(error_info.suggestion, Some("Suggestion".to_string()));
    }

    #[test]
    fn test_tool_result_serialization() {
        let result = ToolResult::ok("test data", 50, "mcp");
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"data\":\"test data\""));
    }
}
