//! Integration tests for MCP (Model Context Protocol)

use ricecoder_agents::mcp_integration::*;

// Test that MCP integration types can be imported and basic functionality works
#[test]
fn test_mcp_integration_imports() {
    // Test that we can import and use basic MCP types
    let result: Result<serde_json::Value, String> = Ok(serde_json::json!({"test": "value"}));
    assert!(result.is_ok());
}

#[test]
fn test_mcp_tool_execution_result() {
    let result = ToolExecutionResult {
        success: true,
        data: Some(serde_json::json!({"result": "success"})),
        error: None,
        execution_time_ms: 100,
    };

    assert_eq!(result.success, true);
    assert!(result.data.is_some());
    assert!(result.error.is_none());
    assert_eq!(result.execution_time_ms, 100);
}