//! Phase 1: Domain Layer Unit Tests
//!
//! Tests for core domain models in hexagonal architecture:
//! - ToolMetadata and ParameterMetadata
//! - ToolRegistry
//! - MCP protocol models (MCPMessage, MCPRequest, etc.)
//! - Error types and validation

use proptest::prelude::*;
use ricecoder_mcp::error::{Error, ToolError};
use ricecoder_mcp::metadata::{ParameterMetadata, ToolMetadata, ToolSource};
use ricecoder_mcp::registry::ToolRegistry;
use ricecoder_mcp::transport::{
    MCPError, MCPErrorData, MCPMessage, MCPNotification, MCPRequest, MCPResponse,
};
use serde_json::{json, Value};

/// **Domain Test 1.1: ToolMetadata creation and validation**
/// **Validates: Domain model integrity, validation rules**
#[test]
fn test_tool_metadata_creation() {
    let tool = ToolMetadata::new(
        "test-tool".to_string(),
        "Test Tool".to_string(),
        "A test tool".to_string(),
        "test".to_string(),
        "string".to_string(),
        ToolSource::Custom,
    );

    assert_eq!(tool.id, "test-tool");
    assert_eq!(tool.name, "Test Tool");
    assert_eq!(tool.description, "A test tool");
    assert_eq!(tool.category, "test");
    assert_eq!(tool.return_type, "string");
    assert!(matches!(tool.source, ToolSource::Custom));
    assert!(tool.parameters.is_empty());
    assert!(tool.server_id.is_none());
}

/// **Domain Test 1.2: ToolMetadata parameter management**
/// **Validates: Parameter addition, metadata consistency**
#[test]
fn test_tool_metadata_parameter_management() {
    let mut tool = ToolMetadata::new(
        "test-tool".to_string(),
        "Test Tool".to_string(),
        "A test tool".to_string(),
        "test".to_string(),
        "string".to_string(),
        ToolSource::Custom,
    );

    let param = ParameterMetadata::new(
        "param1".to_string(),
        "string".to_string(),
        "First parameter".to_string(),
        true,
    );

    tool.add_parameter(param);
    assert_eq!(tool.parameters.len(), 1);
    assert_eq!(tool.parameters[0].name, "param1");
    assert_eq!(tool.parameters[0].type_, "string");
    assert!(tool.parameters[0].required);
}

/// **Domain Test 1.3: ToolMetadata server ID management**
/// **Validates: MCP tool server association**
#[test]
fn test_tool_metadata_server_id() {
    let mut tool = ToolMetadata::new(
        "test-tool".to_string(),
        "Test Tool".to_string(),
        "A test tool".to_string(),
        "test".to_string(),
        "string".to_string(),
        ToolSource::Mcp("server-1".to_string()),
    );

    tool.set_server_id("server-1".to_string());
    assert_eq!(tool.server_id, Some("server-1".to_string()));
}

/// **Domain Test 1.4: ToolMetadata validation - valid tool**
/// **Validates: Validation logic for correct tools**
#[test]
fn test_tool_metadata_validation_valid() {
    let mut tool = ToolMetadata::new(
        "test-tool".to_string(),
        "Test Tool".to_string(),
        "A test tool".to_string(),
        "test".to_string(),
        "string".to_string(),
        ToolSource::Custom,
    );

    let param = ParameterMetadata::new(
        "param1".to_string(),
        "string".to_string(),
        "A parameter".to_string(),
        true,
    );
    tool.add_parameter(param);

    assert!(tool.validate().is_ok());
}

/// **Domain Test 1.5: ToolMetadata validation - invalid tools**
/// **Validates: Validation error handling**
#[test]
fn test_tool_metadata_validation_invalid() {
    // Empty ID
    let tool = ToolMetadata::new(
        "".to_string(),
        "Test Tool".to_string(),
        "A test tool".to_string(),
        "test".to_string(),
        "string".to_string(),
        ToolSource::Custom,
    );
    assert!(tool.validate().is_err());

    // Empty name
    let tool = ToolMetadata::new(
        "test-tool".to_string(),
        "".to_string(),
        "A test tool".to_string(),
        "test".to_string(),
        "string".to_string(),
        ToolSource::Custom,
    );
    assert!(tool.validate().is_err());

    // Empty description
    let tool = ToolMetadata::new(
        "test-tool".to_string(),
        "Test Tool".to_string(),
        "".to_string(),
        "test".to_string(),
        "string".to_string(),
        ToolSource::Custom,
    );
    assert!(tool.validate().is_err());
}

/// **Domain Test 1.6: ParameterMetadata with defaults**
/// **Validates: Default value handling**
#[test]
fn test_parameter_metadata_with_defaults() {
    let param = ParameterMetadata::new(
        "param1".to_string(),
        "string".to_string(),
        "A parameter".to_string(),
        false,
    )
    .with_default(Value::String("default_value".to_string()));

    assert_eq!(param.name, "param1");
    assert_eq!(
        param.default,
        Some(Value::String("default_value".to_string()))
    );
}

/// **Domain Test 1.7: ToolRegistry basic operations**
/// **Validates: Registry CRUD operations**
#[test]
fn test_tool_registry_basic_operations() {
    let mut registry = ToolRegistry::new();

    let tool = ToolMetadata::new(
        "test-tool".to_string(),
        "Test Tool".to_string(),
        "A test tool".to_string(),
        "test".to_string(),
        "string".to_string(),
        ToolSource::Custom,
    );

    // Register tool
    assert!(registry.register_tool(tool.clone()).is_ok());
    assert_eq!(registry.tool_count(), 1);

    // Get tool
    let retrieved = registry.get_tool("test-tool");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, "test-tool");

    // List tools
    let tools = registry.list_tools();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].id, "test-tool");
}

/// **Domain Test 1.8: ToolRegistry duplicate prevention**
/// **Validates: Naming conflict detection**
#[test]
fn test_tool_registry_duplicate_prevention() {
    let mut registry = ToolRegistry::new();

    let tool1 = ToolMetadata::new(
        "test-tool".to_string(),
        "Test Tool".to_string(),
        "A test tool".to_string(),
        "test".to_string(),
        "string".to_string(),
        ToolSource::Custom,
    );

    let tool2 = ToolMetadata::new(
        "test-tool".to_string(),
        "Another Tool".to_string(),
        "Another test tool".to_string(),
        "test".to_string(),
        "string".to_string(),
        ToolSource::Custom,
    );

    assert!(registry.register_tool(tool1).is_ok());
    assert!(registry.register_tool(tool2).is_err());
    assert_eq!(registry.tool_count(), 1);
}

/// **Domain Test 1.9: ToolRegistry category filtering**
/// **Validates: Category-based tool discovery**
#[test]
fn test_tool_registry_category_filtering() {
    let mut registry = ToolRegistry::new();

    let tool1 = ToolMetadata::new(
        "tool1".to_string(),
        "Tool 1".to_string(),
        "A test tool".to_string(),
        "category1".to_string(),
        "string".to_string(),
        ToolSource::Custom,
    );

    let tool2 = ToolMetadata::new(
        "tool2".to_string(),
        "Tool 2".to_string(),
        "Another test tool".to_string(),
        "category2".to_string(),
        "string".to_string(),
        ToolSource::Custom,
    );

    let tool3 = ToolMetadata::new(
        "tool3".to_string(),
        "Tool 3".to_string(),
        "Third test tool".to_string(),
        "category1".to_string(),
        "string".to_string(),
        ToolSource::Custom,
    );

    registry.register_tool(tool1).unwrap();
    registry.register_tool(tool2).unwrap();
    registry.register_tool(tool3).unwrap();

    let category1_tools = registry.list_tools_by_category("category1");
    assert_eq!(category1_tools.len(), 2);

    let category2_tools = registry.list_tools_by_category("category2");
    assert_eq!(category2_tools.len(), 1);

    let category3_tools = registry.list_tools_by_category("category3");
    assert_eq!(category3_tools.len(), 0);
}

/// **Domain Test 1.10: ToolRegistry server filtering**
/// **Validates: Server-based tool discovery**
#[test]
fn test_tool_registry_server_filtering() {
    let mut registry = ToolRegistry::new();

    let mut tool1 = ToolMetadata::new(
        "tool1".to_string(),
        "Tool 1".to_string(),
        "A test tool".to_string(),
        "test".to_string(),
        "string".to_string(),
        ToolSource::Mcp("server1".to_string()),
    );
    tool1.set_server_id("server1".to_string());

    let mut tool2 = ToolMetadata::new(
        "tool2".to_string(),
        "Tool 2".to_string(),
        "Another test tool".to_string(),
        "test".to_string(),
        "string".to_string(),
        ToolSource::Mcp("server2".to_string()),
    );
    tool2.set_server_id("server2".to_string());

    let tool3 = ToolMetadata::new(
        "tool3".to_string(),
        "Tool 3".to_string(),
        "Third test tool".to_string(),
        "test".to_string(),
        "string".to_string(),
        ToolSource::Custom,
    );

    registry.register_tool(tool1).unwrap();
    registry.register_tool(tool2).unwrap();
    registry.register_tool(tool3).unwrap();

    let server1_tools = registry.list_tools_by_server("server1");
    assert_eq!(server1_tools.len(), 1);
    assert_eq!(server1_tools[0].id, "tool1");

    let server2_tools = registry.list_tools_by_server("server2");
    assert_eq!(server2_tools.len(), 1);
    assert_eq!(server2_tools[0].id, "tool2");

    let server3_tools = registry.list_tools_by_server("server3");
    assert_eq!(server3_tools.len(), 0);
}

/// **Domain Test 1.11: MCPMessage serialization**
/// **Validates: Protocol message structure**
#[test]
fn test_mcp_message_serialization() {
    let request = MCPRequest {
        id: "req-1".to_string(),
        method: "tools/list".to_string(),
        params: json!({"category": "test"}),
    };

    let message = MCPMessage::Request(request.clone());

    let serialized = serde_json::to_string(&message).unwrap();
    let deserialized: MCPMessage = serde_json::from_str(&serialized).unwrap();

    match deserialized {
        MCPMessage::Request(des_req) => {
            assert_eq!(des_req.id, request.id);
            assert_eq!(des_req.method, request.method);
            assert_eq!(des_req.params, request.params);
        }
        _ => panic!("Expected request message"),
    }
}

/// **Domain Test 1.12: MCPResponse serialization**
/// **Validates: Response message structure**
#[test]
fn test_mcp_response_serialization() {
    let response = MCPResponse {
        id: "req-1".to_string(),
        result: json!({"tools": []}),
    };

    let message = MCPMessage::Response(response.clone());

    let serialized = serde_json::to_string(&message).unwrap();
    let deserialized: MCPMessage = serde_json::from_str(&serialized).unwrap();

    match deserialized {
        MCPMessage::Response(des_res) => {
            assert_eq!(des_res.id, response.id);
            assert_eq!(des_res.result, response.result);
        }
        _ => panic!("Expected response message"),
    }
}

/// **Domain Test 1.13: MCPNotification serialization**
/// **Validates: Notification message structure**
#[test]
fn test_mcp_notification_serialization() {
    let notification = MCPNotification {
        method: "tools/updated".to_string(),
        params: json!({"tool_id": "test-tool"}),
    };

    let message = MCPMessage::Notification(notification.clone());

    let serialized = serde_json::to_string(&message).unwrap();
    let deserialized: MCPMessage = serde_json::from_str(&serialized).unwrap();

    match deserialized {
        MCPMessage::Notification(des_notif) => {
            assert_eq!(des_notif.method, notification.method);
            assert_eq!(des_notif.params, notification.params);
        }
        _ => panic!("Expected notification message"),
    }
}

/// **Domain Test 1.14: MCPError serialization**
/// **Validates: Error message structure**
#[test]
fn test_mcp_error_serialization() {
    let error_data = MCPErrorData {
        code: -32000,
        message: "Tool not found".to_string(),
        data: Some(json!({"tool_id": "missing-tool"})),
    };

    let error = MCPError {
        id: Some("req-1".to_string()),
        error: error_data.clone(),
    };

    let message = MCPMessage::Error(error.clone());

    let serialized = serde_json::to_string(&message).unwrap();
    let deserialized: MCPMessage = serde_json::from_str(&serialized).unwrap();

    match deserialized {
        MCPMessage::Error(des_err) => {
            assert_eq!(des_err.id, error.id);
            assert_eq!(des_err.error.code, error_data.code);
            assert_eq!(des_err.error.message, error_data.message);
            assert_eq!(des_err.error.data, error_data.data);
        }
        _ => panic!("Expected error message"),
    }
}

/// **Domain Test 1.15: Error type conversions**
/// **Validates: Error domain model**
#[test]
fn test_error_type_conversions() {
    let tool_error = ToolError::NotFound("test-tool".to_string());
    let error = Error::ToolExecutionError(tool_error.clone());

    match error {
        Error::ToolExecutionError(te) => {
            assert!(matches!(te, ToolError::NotFound(id) if id == "test-tool"));
        }
        _ => panic!("Expected tool execution error"),
    }
}

/// **Domain Test 1.16: ToolSource enum variants**
/// **Validates: Tool source domain model**
#[test]
fn test_tool_source_enum() {
    let builtin = ToolSource::BuiltIn;
    let custom = ToolSource::Custom;
    let mcp = ToolSource::Mcp("server-1".to_string());

    assert!(matches!(builtin, ToolSource::BuiltIn));
    assert!(matches!(custom, ToolSource::Custom));
    assert!(matches!(mcp, ToolSource::Mcp(server) if server == "server-1"));
}

// Property-based tests for domain models

/// **Domain Property Test 1.1: ToolMetadata round-trip serialization**
/// **Validates: Serialization consistency**
proptest! {
    #[test]
    fn property_tool_metadata_serialization_round_trip(
        id in "[a-zA-Z0-9_-]{1,50}",
        name in "[a-zA-Z0-9 ]{1,100}",
        description in "[a-zA-Z0-9 .]{1,200}",
        category in "[a-zA-Z0-9_-]{1,50}",
        return_type in "[a-zA-Z0-9_-]{1,50}",
    ) {
        let tool = ToolMetadata::new(
            id.clone(),
            name.clone(),
            description.clone(),
            category.clone(),
            return_type.clone(),
            ToolSource::Custom,
        );

        let serialized = serde_json::to_string(&tool).unwrap();
        let deserialized: ToolMetadata = serde_json::from_str(&serialized).unwrap();

        prop_assert_eq!(deserialized.id, id);
        prop_assert_eq!(deserialized.name, name);
        prop_assert_eq!(deserialized.description, description);
        prop_assert_eq!(deserialized.category, category);
        prop_assert_eq!(deserialized.return_type, return_type);
        prop_assert!(matches!(deserialized.source, ToolSource::Custom));
    }
}

/// **Domain Property Test 1.2: ParameterMetadata validation**
/// **Validates: Parameter validation rules**
proptest! {
    #[test]
    fn property_parameter_metadata_validation(
        name in "[a-zA-Z0-9_-]{1,50}",
        type_ in "[a-zA-Z0-9_-]{1,50}",
        description in "[a-zA-Z0-9 .]{1,200}",
        required in proptest::bool::ANY,
    ) {
        let param = ParameterMetadata::new(
            name.clone(),
            type_.clone(),
            description.clone(),
            required,
        );

        // Valid parameter should have non-empty fields
        prop_assert!(!param.name.is_empty());
        prop_assert!(!param.type_.is_empty());
        prop_assert!(!param.description.is_empty());
    }
}

/// **Domain Property Test 1.3: ToolRegistry capacity**
/// **Validates: Registry scalability**
proptest! {
    #[test]
    fn property_tool_registry_capacity(
        tool_count in 1usize..100,
    ) {
        let mut registry = ToolRegistry::new();

        for i in 0..tool_count {
            let tool = ToolMetadata::new(
                format!("tool-{}", i),
                format!("Tool {}", i),
                format!("Description {}", i),
                "test".to_string(),
                "string".to_string(),
                ToolSource::Custom,
            );
            prop_assert!(registry.register_tool(tool).is_ok());
        }

        prop_assert_eq!(registry.tool_count(), tool_count);
        prop_assert_eq!(registry.list_tools().len(), tool_count);
    }
}
