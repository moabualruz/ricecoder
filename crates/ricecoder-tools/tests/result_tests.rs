//! Tests for ricecoder-tools result handling
//!
//! Tests for ToolResult, ResultMetadata, and ToolErrorInfo structures.

use ricecoder_tools::{
    error::ToolError,
    result::{ResultMetadata, ToolErrorInfo, ToolResult},
};

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
