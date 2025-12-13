//! Tests for ricecoder-tools error handling
//!
//! Tests for ToolError creation, formatting, and conversions.

use ricecoder_tools::error::ToolError;

#[test]
fn test_tool_error_creation() {
    let err = ToolError::new("TEST_ERROR", "Test message");
    assert_eq!(err.code, "TEST_ERROR");
    assert_eq!(err.message, "Test message");
    assert!(err.details.is_none());
    assert!(err.suggestion.is_none());
}

#[test]
fn test_tool_error_with_details() {
    let err = ToolError::new("TEST_ERROR", "Test message")
        .with_details("Additional context");
    assert_eq!(err.details, Some("Additional context".to_string()));
}

#[test]
fn test_tool_error_with_suggestion() {
    let err = ToolError::new("TEST_ERROR", "Test message")
        .with_suggestion("Try this instead");
    assert_eq!(err.suggestion, Some("Try this instead".to_string()));
}

#[test]
fn test_tool_error_display() {
    let err = ToolError::new("TEST_ERROR", "Test message")
        .with_details("Details")
        .with_suggestion("Suggestion");
    let display = format!("{}", err);
    assert!(display.contains("TEST_ERROR"));
    assert!(display.contains("Test message"));
    assert!(display.contains("Details"));
    assert!(display.contains("Suggestion"));
}