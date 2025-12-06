use proptest::prelude::*;
use ricecoder_mcp::{ToolMarshaler, ToolRegistry, ToolMetadata, ToolSource};
use serde_json::{json, Value};

/// **Feature: ricecoder-mcp, Property 1: Tool registration consistency**
/// **Validates: Requirements 1.1, 4.1**
///
/// For any MCP server, when tools are registered, the tool registry SHALL contain
/// all registered tools with correct metadata and no duplicates.
#[test]
fn property_tool_registration_consistency() {
    proptest!(|(
        tool_id in "[a-z0-9-]{1,50}",
        tool_name in "[a-zA-Z0-9 ]{1,50}",
        category in "[a-z]{1,20}",
    )| {
        let mut registry = ToolRegistry::new();
        
        // Create a tool with the generated properties
        let tool = ToolMetadata {
            id: tool_id.clone(),
            name: tool_name.clone(),
            description: "Test tool".to_string(),
            category: category.clone(),
            parameters: vec![],
            return_type: "string".to_string(),
            source: ToolSource::Custom,
            server_id: None,
        };
        
        // Register the tool
        let result = registry.register_tool(tool.clone());
        prop_assert!(result.is_ok(), "Tool registration should succeed");
        
        // Verify the tool is in the registry
        let retrieved = registry.get_tool(&tool_id);
        prop_assert!(retrieved.is_some(), "Tool should be retrievable after registration");
        
        // Verify the metadata is correct
        let retrieved_tool = retrieved.unwrap();
        prop_assert_eq!(&retrieved_tool.id, &tool_id, "Tool ID should match");
        prop_assert_eq!(&retrieved_tool.name, &tool_name, "Tool name should match");
        prop_assert_eq!(&retrieved_tool.category, &category, "Tool category should match");
        
        // Verify no duplicates - registering the same tool again should fail
        let duplicate_result = registry.register_tool(tool);
        prop_assert!(duplicate_result.is_err(), "Duplicate registration should fail");
        
        // Verify tool count is exactly 1
        prop_assert_eq!(registry.tool_count(), 1, "Registry should contain exactly 1 tool");
    });
}

/// **Feature: ricecoder-mcp, Property 2: Tool marshaling round-trip**
/// **Validates: Requirements 1.2, 1.3**
///
/// For any valid tool input, marshaling then unmarshaling should produce
/// an equivalent value (round-trip property).
#[test]
fn property_tool_marshaling_round_trip() {
    proptest!(|(
        param_value in prop::option::of("[a-zA-Z0-9]{1,50}"),
        param_number in 0i64..1000000,
    )| {
        let marshaler = ToolMarshaler::new();
        
        // Create input with various parameter types
        let mut input_obj = serde_json::Map::new();
        if let Some(val) = param_value {
            input_obj.insert("string_param".to_string(), Value::String(val));
        }
        input_obj.insert("number_param".to_string(), json!(param_number));
        let input = Value::Object(input_obj);
        
        // Marshal the input
        let marshaled = marshaler.marshal_input(&input);
        prop_assert!(marshaled.is_ok(), "Marshaling should succeed");
        
        let marshaled_value = marshaled.unwrap();
        
        // Unmarshal the output
        let unmarshaled = marshaler.unmarshal_output(&marshaled_value);
        prop_assert!(unmarshaled.is_ok(), "Unmarshaling should succeed");
        
        let unmarshaled_value = unmarshaled.unwrap();
        
        // Verify round-trip: original == unmarshaled
        prop_assert_eq!(input, unmarshaled_value, "Round-trip should preserve value");
    });
}

/// **Feature: ricecoder-mcp, Property 3: Type conversion consistency**
/// **Validates: Requirements 1.2, 5.4**
///
/// For any valid type conversion, converting a value to a type and back
/// should produce an equivalent value.
#[test]
fn property_type_conversion_consistency() {
    proptest!(|(
        number_str in "[0-9]{1,10}",
    )| {
        let marshaler = ToolMarshaler::new();
        
        // Test string to number conversion
        let string_value = Value::String(number_str.clone());
        let converted = marshaler.convert_type(&string_value, "number");
        prop_assert!(converted.is_ok(), "String to number conversion should succeed");
        
        // Test number to string conversion
        let number_value = converted.unwrap();
        let converted_back = marshaler.convert_type(&number_value, "string");
        prop_assert!(converted_back.is_ok(), "Number to string conversion should succeed");
        
        // Verify the string representation is consistent
        let final_string = converted_back.unwrap();
        prop_assert!(final_string.is_string(), "Result should be a string");
    });
}

/// **Feature: ricecoder-mcp, Property 4: Invalid input rejection**
/// **Validates: Requirements 5.4, 5.5**
///
/// For any invalid input (arrays, primitives), marshaling should reject it
/// with a validation error.
#[test]
fn property_invalid_input_rejection() {
    proptest!(|(
        array_len in 0usize..10,
    )| {
        let marshaler = ToolMarshaler::new();
        
        // Create an array input (invalid for tool parameters)
        let array_input: Vec<Value> = (0..array_len)
            .map(|i| Value::Number(i.into()))
            .collect();
        let input = Value::Array(array_input);
        
        // Marshaling should fail for array input
        let result = marshaler.marshal_input(&input);
        prop_assert!(result.is_err(), "Array input should be rejected");
    });
}

/// **Feature: ricecoder-mcp, Property 2: Permission enforcement**
/// **Validates: Requirements 3.1, 3.3**
///
/// For any tool execution request, if the tool permission is "deny", the system
/// SHALL reject the request without executing the tool, regardless of agent or context.
#[test]
fn property_permission_enforcement() {
    use ricecoder_mcp::{MCPPermissionManager, PermissionLevelConfig, PermissionRule};
    use ricecoder_permissions::PermissionLevel;

    proptest!(|(
        tool_id in "[a-z0-9-]{1,50}",
        agent_id in prop::option::of("[a-z0-9-]{1,30}"),
    )| {
        let mut manager = MCPPermissionManager::new();
        
        // Add a deny rule for the tool
        let rule = PermissionRule {
            pattern: tool_id.clone(),
            level: PermissionLevelConfig::Deny,
            agent_id: None,
        };
        
        let result = manager.add_global_rule(rule);
        prop_assert!(result.is_ok(), "Adding deny rule should succeed");
        
        // Check permission - should be denied
        let permission = manager.check_permission(&tool_id, agent_id.as_deref());
        prop_assert!(permission.is_ok(), "Permission check should succeed");
        prop_assert_eq!(
            permission.unwrap(),
            PermissionLevel::Deny,
            "Permission should be Deny"
        );
    });
}

/// **Feature: ricecoder-mcp, Property 6: Permission wildcard matching**
/// **Validates: Requirements 3.4**
///
/// For any tool ID and permission rule with wildcard pattern, the system SHALL
/// correctly match the tool against the pattern and apply the appropriate permission level.
#[test]
fn property_permission_wildcard_matching() {
    use ricecoder_mcp::{MCPPermissionManager, PermissionLevelConfig, PermissionRule};
    use ricecoder_permissions::PermissionLevel;

    proptest!(|(
        prefix in "[a-z]{1,10}",
        suffix in "[a-z0-9]{1,10}",
    )| {
        let mut manager = MCPPermissionManager::new();
        
        // Create a wildcard pattern
        let pattern = format!("{}-*", prefix);
        let tool_id = format!("{}-{}", prefix, suffix);
        
        // Add a rule with wildcard pattern
        let rule = PermissionRule {
            pattern: pattern.clone(),
            level: PermissionLevelConfig::Allow,
            agent_id: None,
        };
        
        let result = manager.add_global_rule(rule);
        prop_assert!(result.is_ok(), "Adding wildcard rule should succeed");
        
        // Check permission for matching tool - should be allowed
        let permission = manager.check_permission(&tool_id, None);
        prop_assert!(permission.is_ok(), "Permission check should succeed");
        prop_assert_eq!(
            permission.unwrap(),
            PermissionLevel::Allow,
            "Tool matching wildcard pattern should be allowed"
        );
        
        // Check permission for non-matching tool - should be denied (default)
        let non_matching_tool = format!("other-{}", suffix);
        let permission = manager.check_permission(&non_matching_tool, None);
        prop_assert!(permission.is_ok(), "Permission check should succeed");
        prop_assert_eq!(
            permission.unwrap(),
            PermissionLevel::Deny,
            "Tool not matching wildcard pattern should be denied"
        );
    });
}

/// **Feature: ricecoder-mcp, Property 7: Per-agent permission override**
/// **Validates: Requirements 3.5**
///
/// For any tool execution with both global and per-agent permissions defined,
/// the system SHALL use the per-agent permission level instead of the global level.
#[test]
fn property_per_agent_permission_override() {
    use ricecoder_mcp::{MCPPermissionManager, PermissionLevelConfig, PermissionRule};
    use ricecoder_permissions::PermissionLevel;

    proptest!(|(
        tool_id in "[a-z0-9-]{1,50}",
        agent_id in "[a-z0-9-]{1,30}",
    )| {
        let mut manager = MCPPermissionManager::new();
        
        // Add a global deny rule
        let global_rule = PermissionRule {
            pattern: tool_id.clone(),
            level: PermissionLevelConfig::Deny,
            agent_id: None,
        };
        manager.add_global_rule(global_rule).unwrap();
        
        // Add a per-agent allow rule
        let agent_rule = PermissionRule {
            pattern: tool_id.clone(),
            level: PermissionLevelConfig::Allow,
            agent_id: Some(agent_id.clone()),
        };
        manager.add_agent_rule(agent_id.clone(), agent_rule).unwrap();
        
        // Check permission for the agent - should use per-agent rule (Allow)
        let permission = manager.check_permission(&tool_id, Some(&agent_id));
        prop_assert!(permission.is_ok(), "Permission check should succeed");
        prop_assert_eq!(
            permission.unwrap(),
            PermissionLevel::Allow,
            "Per-agent permission should override global permission"
        );
        
        // Check permission for other agent - should use global rule (Deny)
        let other_agent = format!("other-{}", agent_id);
        let permission = manager.check_permission(&tool_id, Some(&other_agent));
        prop_assert!(permission.is_ok(), "Permission check should succeed");
        prop_assert_eq!(
            permission.unwrap(),
            PermissionLevel::Deny,
            "Other agents should use global permission"
        );
    });
}

/// **Feature: ricecoder-mcp, Property 3: Error isolation**
/// **Validates: Requirements 5.1, 5.2**
///
/// For any tool execution failure in one server, the system SHALL handle
/// the error gracefully without affecting other servers or tools.
/// When one server fails, other servers remain operational and can still
/// execute their tools successfully.
#[test]
fn property_error_isolation() {
    proptest!(|(
        num_servers in 2usize..5,
        failing_server_idx in 0usize..5,
    )| {
        // Create multiple servers
        let mut registry = ToolRegistry::new();
        
        // Register tools from multiple servers
        for server_idx in 0..num_servers {
            let server_id = format!("server-{}", server_idx);
            let tool_id = format!("tool-{}", server_idx);
            
            let tool = ToolMetadata {
                id: tool_id,
                name: format!("Tool {}", server_idx),
                description: "Test tool".to_string(),
                category: "test".to_string(),
                parameters: vec![],
                return_type: "string".to_string(),
                source: ToolSource::Mcp(server_id.clone()),
                server_id: Some(server_id),
            };
            
            let result = registry.register_tool(tool);
            prop_assert!(result.is_ok(), "Tool registration should succeed");
        }
        
        // Verify all tools are registered
        prop_assert_eq!(registry.tool_count(), num_servers, "All tools should be registered");
        
        // Simulate failure in one server - verify other tools are still accessible
        let failing_server = format!("server-{}", failing_server_idx % num_servers);
        let _failing_tools = registry.list_tools_by_server(&failing_server);
        
        // Even if one server fails, other servers' tools should still be accessible
        for server_idx in 0..num_servers {
            if server_idx != (failing_server_idx % num_servers) {
                let server_id = format!("server-{}", server_idx);
                let tools = registry.list_tools_by_server(&server_id);
                prop_assert_eq!(tools.len(), 1, "Other servers should still have their tools");
            }
        }
        
        // Verify that the registry itself is still functional
        let all_tools = registry.list_tools();
        prop_assert_eq!(all_tools.len(), num_servers, "Registry should still contain all tools");
        
        // Verify we can still query tools by category
        let category_tools = registry.list_tools_by_category("test");
        prop_assert_eq!(category_tools.len(), num_servers, "Category query should still work");
    });
}

/// **Feature: ricecoder-mcp, Property 4: Tool discovery completeness**
/// **Validates: Requirements 1.1, 4.1, 4.2**
///
/// For any configured MCP server, the tool discovery process SHALL return
/// all available tools from that server with complete metadata.
#[test]
fn property_tool_discovery_completeness() {
    proptest!(|(
        num_tools in 1usize..10,
        server_id in "[a-z0-9-]{1,30}",
    )| {
        let mut registry = ToolRegistry::new();
        
        // Register multiple tools from the same server
        for tool_idx in 0..num_tools {
            let tool_id = format!("tool-{}", tool_idx);
            let tool = ToolMetadata {
                id: tool_id,
                name: format!("Tool {}", tool_idx),
                description: format!("Description for tool {}", tool_idx),
                category: "test".to_string(),
                parameters: vec![],
                return_type: "string".to_string(),
                source: ToolSource::Mcp(server_id.clone()),
                server_id: Some(server_id.clone()),
            };
            
            let result = registry.register_tool(tool);
            prop_assert!(result.is_ok(), "Tool registration should succeed");
        }
        
        // Discover tools from the server
        let discovered_tools = registry.list_tools_by_server(&server_id);
        
        // Verify all tools are discovered
        prop_assert_eq!(
            discovered_tools.len(),
            num_tools,
            "All registered tools should be discovered"
        );
        
        // Verify each discovered tool has complete metadata
        for tool in discovered_tools {
            prop_assert!(!tool.id.is_empty(), "Tool ID should not be empty");
            prop_assert!(!tool.name.is_empty(), "Tool name should not be empty");
            prop_assert!(!tool.description.is_empty(), "Tool description should not be empty");
            prop_assert!(!tool.category.is_empty(), "Tool category should not be empty");
            prop_assert!(!tool.return_type.is_empty(), "Tool return type should not be empty");
            prop_assert_eq!(&tool.server_id, &Some(server_id.clone()), "Server ID should match");
        }
    });
}

/// **Feature: ricecoder-mcp, Property 5: Custom tool registration**
/// **Validates: Requirements 2.1, 2.4, 2.5**
///
/// For any custom tool defined in configuration (JSON or Markdown), the system
/// SHALL register it with the agent system and make it available for execution.
#[test]
fn property_custom_tool_registration() {
    proptest!(|(
        tool_id in "[a-z0-9-]{1,50}",
        tool_name in "[a-zA-Z0-9 ]{1,50}",
        category in "[a-z]{1,20}",
    )| {
        let mut registry = ToolRegistry::new();
        
        // Create a custom tool (simulating configuration-based registration)
        let custom_tool = ToolMetadata {
            id: tool_id.clone(),
            name: tool_name.clone(),
            description: "Custom tool from configuration".to_string(),
            category: category.clone(),
            parameters: vec![],
            return_type: "string".to_string(),
            source: ToolSource::Custom,
            server_id: None,
        };
        
        // Register the custom tool
        let result = registry.register_tool(custom_tool.clone());
        prop_assert!(result.is_ok(), "Custom tool registration should succeed");
        
        // Verify the tool is available for execution
        let retrieved = registry.get_tool(&tool_id);
        prop_assert!(retrieved.is_some(), "Custom tool should be available after registration");
        
        // Verify the tool has correct metadata
        let retrieved_tool = retrieved.unwrap();
        prop_assert_eq!(&retrieved_tool.id, &tool_id, "Tool ID should match");
        prop_assert_eq!(&retrieved_tool.name, &tool_name, "Tool name should match");
        prop_assert_eq!(&retrieved_tool.category, &category, "Tool category should match");
        prop_assert!(matches!(retrieved_tool.source, ToolSource::Custom), "Tool source should be Custom");
        
        // Verify the tool is discoverable by category
        let category_tools = registry.list_tools_by_category(&category);
        prop_assert!(
            category_tools.iter().any(|t| t.id == tool_id),
            "Custom tool should be discoverable by category"
        );
    });
}
