//! MCP Testing Infrastructure Enhancement Tests
//!
//! This module contains tests that validate the MCP testing infrastructure enhancements,
//! including reusable fixtures, mock servers, automated test data generation, and
//! enterprise security testing utilities.

use proptest::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use ricecoder_mcp::{
    metadata::ToolMetadata,
    transport::{MCPRequest, MCPResponse, MCPMessage},
    registry::ToolRegistry,
    error::Result,
};

// Import our testing infrastructure
#[path = "mcp_testing_infrastructure.rs"]
mod mcp_testing_infrastructure;
use mcp_testing_infrastructure::*;

// ============================================================================
// Tests for Reusable MCP Test Fixtures
// ============================================================================

#[tokio::test]
async fn test_mcp_test_environment_setup() {
    let env = MCPTestEnvironment::new();

    // Test initial state
    assert_eq!(env.tool_registry.tool_count().await, 0);

    // Register test tools
    env.register_test_tools(5).await.unwrap();
    assert_eq!(env.tool_registry.tool_count().await, 5);

    // Verify tools are registered correctly
    for i in 0..5 {
        let tool_name = format!("test-tool-{}", i);
        let tool = env.tool_registry.get_tool(&tool_name).await.unwrap();
        assert_eq!(tool.name, tool_name);
        assert!(tool.description.contains(&i.to_string()));
    }
}

#[tokio::test]
async fn test_mcp_test_environment_isolation() {
    let env1 = MCPTestEnvironment::new();
    let env2 = MCPTestEnvironment::new();

    // Register tools in env1
    env1.register_test_tools(3).await.unwrap();

    // env2 should remain unaffected
    assert_eq!(env1.tool_registry.tool_count().await, 3);
    assert_eq!(env2.tool_registry.tool_count().await, 0);

    // Register different tools in env2
    env2.register_test_tools(2).await.unwrap();
    assert_eq!(env1.tool_registry.tool_count().await, 3);
    assert_eq!(env2.tool_registry.tool_count().await, 2);
}

// ============================================================================
// Tests for Mock MCP Servers
// ============================================================================

#[tokio::test]
async fn test_mock_mcp_server_basic_functionality() {
    let mut server = MockMCPServer::new("test-server".to_string());

    // Add a test tool
    let tool = ToolMetadata {
        name: "mock-tool".to_string(),
        description: "A mock tool for testing".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {"input": {"type": "string"}}
        }),
        permissions_required: vec![],
        metadata: None,
    };
    server.tools.insert("mock-tool".to_string(), tool);

    // Test tool listing
    let request = MCPRequest {
        id: "list-1".to_string(),
        method: "tools/list".to_string(),
        params: None,
    };

    let response = server.handle_request(&request).await.unwrap();
    assert_eq!(response.id, "list-1");

    let tools = response.result.get("tools").unwrap().as_array().unwrap();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0]["name"], "mock-tool");
}

#[tokio::test]
async fn test_mock_mcp_server_error_handling() {
    let server = MockMCPServer::new("test-server".to_string());

    // Test invalid method
    let request = MCPRequest {
        id: "invalid-1".to_string(),
        method: "invalid/method".to_string(),
        params: None,
    };

    let result = server.handle_request(&request).await;
    assert!(result.is_err());
}

// ============================================================================
// Tests for Automated Test Data Generation
// ============================================================================

#[test]
fn test_protocol_data_generator_basic_scenarios() {
    let scenarios = MCPProtocolDataGenerator::generate_protocol_test_scenarios();

    assert!(!scenarios.is_empty());

    // Check first scenario
    let first_scenario = &scenarios[0];
    assert_eq!(first_scenario.name, "basic_request_response");
    assert_eq!(first_scenario.request.method, "tools/call");
    assert!(!first_scenario.should_fail);
    assert!(first_scenario.expected_response.is_some());
}

#[test]
fn test_protocol_data_generator_scenario_validity() {
    let scenarios = MCPProtocolDataGenerator::generate_protocol_test_scenarios();

    for scenario in scenarios {
        // All scenarios should have valid IDs
        assert!(!scenario.request.id.is_empty());

        // Response ID should match request ID if response exists
        if let Some(ref response) = scenario.expected_response {
            assert_eq!(response.id, scenario.request.id);
        }
    }
}

// ============================================================================
// Tests for MCP-Specific Property-Based Generators
// ============================================================================

proptest! {
    #[test]
    fn test_enterprise_tool_metadata_generator(
        tool in arb_enterprise_tool_metadata()
    ) {
        // Generated tools should have valid names
        prop_assert!(!tool.name.is_empty());
        prop_assert!(tool.name.len() <= 64);

        // Should have descriptions
        prop_assert!(!tool.description.is_empty());
        prop_assert!(tool.description.len() <= 256);

        // Should have valid input schema
        prop_assert!(tool.input_schema.is_object());

        // Permissions should be reasonable
        prop_assert!(tool.permissions_required.len() <= 5);
        for perm in &tool.permissions_required {
            prop_assert!(!perm.is_empty());
            prop_assert!(perm.len() <= 128);
        }
    }

    #[test]
    fn test_enterprise_tool_metadata_schema_compliance(
        tool in arb_enterprise_tool_metadata()
    ) {
        // Schema should be a valid JSON object
        let schema = &tool.input_schema;
        prop_assert!(schema.is_object());

        let obj = schema.as_object().unwrap();

        // Should have type field
        prop_assert!(obj.contains_key("type"));
        prop_assert_eq!(obj["type"], "object");

        // Should have properties
        prop_assert!(obj.contains_key("properties"));
        let properties = obj["properties"].as_object().unwrap();
        prop_assert!(!properties.is_empty());
    }
}

// ============================================================================
// Tests for Enterprise Security Testing Utilities
// ============================================================================

#[test]
fn test_compliance_test_harness_creation() {
    let harness = ComplianceTestHarness::new();

    // Should create environment successfully
    assert_eq!(harness.environment.tool_registry.tool_count().unwrap(), 0);
}

#[tokio::test]
async fn test_enhanced_mock_transport_request_history() {
    let transport = EnhancedMockTransport::new();

    // Initially empty
    assert_eq!(transport.request_history.read().unwrap().len(), 0);

    // Send a message
    let message = MCPMessage::Request(MCPRequest {
        id: "test-1".to_string(),
        method: "test".to_string(),
        params: None,
    });

    transport.send(&message).await.unwrap();

    // Should record the request
    let history = transport.request_history.read().unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0], message);
}

#[tokio::test]
async fn test_enhanced_mock_transport_receive() {
    let transport = EnhancedMockTransport::new();

    // Should return a mock response
    let response = transport.receive().await.unwrap();

    match response {
        MCPMessage::Response(resp) => {
            assert_eq!(resp.id, "mock-id");
            assert_eq!(resp.result["status"], "ok");
        }
        _ => panic!("Expected response message"),
    }
}

// ============================================================================
// Integration Tests Using Testing Infrastructure
// ============================================================================

#[tokio::test]
async fn test_complete_mcp_workflow_with_test_infrastructure() {
    // Setup test environment
    let env = MCPTestEnvironment::new();
    env.register_test_tools(3).await.unwrap();

    // Create mock server
    let mut server = MockMCPServer::new("integration-server".to_string());

    // Add tools to server that match environment
    for i in 0..3 {
        let tool_name = format!("test-tool-{}", i);
        let tool = env.tool_registry.get_tool(&tool_name).await.unwrap();
        server.tools.insert(tool_name, tool.clone());
    }

    // Test tool discovery workflow
    let list_request = MCPRequest {
        id: "workflow-1".to_string(),
        method: "tools/list".to_string(),
        params: None,
    };

    let list_response = server.handle_request(&list_request).await.unwrap();
    let tools_list = list_response.result["tools"].as_array().unwrap();

    // Should discover all 3 tools
    assert_eq!(tools_list.len(), 3);

    // Verify each tool is discoverable
    for tool in tools_list {
        let tool_name = tool["name"].as_str().unwrap();
        assert!(env.tool_registry.get_tool(tool_name).await.is_some());
    }
}

#[tokio::test]
async fn test_enterprise_security_workflow_with_test_infrastructure() {
    // Setup compliance test harness
    let harness = ComplianceTestHarness::new();
    harness.environment.register_test_tools(5).await.unwrap();

    // Create enhanced mock transport for audit logging
    let transport = EnhancedMockTransport::new();

    // Simulate enterprise workflow with multiple requests
    let requests = vec![
        MCPMessage::Request(MCPRequest {
            id: "sec-1".to_string(),
            method: "tools/call".to_string(),
            params: Some(serde_json::json!({"name": "test-tool-0", "security_level": "internal"})),
        }),
        MCPMessage::Request(MCPRequest {
            id: "sec-2".to_string(),
            method: "tools/list".to_string(),
            params: None,
        }),
    ];

    // Send requests through transport
    for request in &requests {
        transport.send(request).await.unwrap();
    }

    // Verify audit trail
    let history = transport.request_history.read().unwrap();
    assert_eq!(history.len(), 2);

    // Verify request details are preserved
    match &history[0] {
        MCPMessage::Request(req) => {
            assert_eq!(req.id, "sec-1");
            assert_eq!(req.method, "tools/call");
        }
        _ => panic!("Expected request"),
    }
}

// ============================================================================
// Property-Based Tests Using Generated Data
// ============================================================================

proptest! {
    #[test]
    fn test_protocol_scenarios_with_generated_data(
        scenario in prop::sample::select(MCPProtocolDataGenerator::generate_protocol_test_scenarios())
    ) {
        // Test that scenarios are well-formed
        prop_assert!(!scenario.name.is_empty());
        prop_assert!(!scenario.request.id.is_empty());
        prop_assert!(!scenario.request.method.is_empty());

        if let Some(ref expected) = scenario.expected_response {
            prop_assert_eq!(expected.id, scenario.request.id);
        }
    }

    #[test]
    fn test_mock_server_with_generated_tools(
        tool in arb_enterprise_tool_metadata()
    ) {
        let mut server = MockMCPServer::new("prop-test-server".to_string());
        server.tools.insert(tool.name.clone(), tool.clone());

        // Server should contain the tool
        prop_assert!(server.tools.contains_key(&tool.name));

        // Retrieved tool should match
        let retrieved = server.tools.get(&tool.name).unwrap();
        prop_assert_eq!(retrieved.name, tool.name);
        prop_assert_eq!(retrieved.description, tool.description);
    }
}

// ============================================================================
// Performance Tests Using Test Infrastructure
// ============================================================================

#[tokio::test]
async fn test_concurrent_mcp_operations_with_test_infrastructure() {
    let env = Arc::new(MCPTestEnvironment::new());
    env.register_test_tools(10).await.unwrap();

    let mut handles = vec![];

    // Spawn multiple concurrent operations
    for i in 0..5 {
        let env_clone = Arc::clone(&env);
        let handle = tokio::spawn(async move {
            for j in 0..10 {
                let tool_name = format!("test-tool-{}", (i + j) % 10);
                let _tool = env_clone.tool_registry.get_tool(&tool_name).await.unwrap();
            }
        });
        handles.push(handle);
    }

    // Wait for all operations to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Environment should still be in consistent state
    assert_eq!(env.tool_registry.tool_count().await, 10);
}

#[tokio::test]
async fn test_mock_server_concurrent_requests() {
    let server = Arc::new(MockMCPServer::new("concurrent-server".to_string()));
    let mut handles = vec![];

    // Add some tools to the server
    {
        let mut tools = server.tools.lock().unwrap();
        tools.insert(
            "concurrent-tool".to_string(),
            ToolMetadata {
                name: "concurrent-tool".to_string(),
                description: "Tool for concurrent testing".to_string(),
                input_schema: serde_json::json!({"type": "object"}),
                permissions_required: vec![],
                metadata: None,
            }
        );
    }

    // Spawn concurrent requests
    for i in 0..10 {
        let server_clone = Arc::clone(&server);
        let handle = tokio::spawn(async move {
            let request = MCPRequest {
                id: format!("concurrency-{}", i),
                method: "tools/list".to_string(),
                params: None,
            };

            server_clone.handle_request(&request).await.unwrap()
        });
        handles.push(handle);
    }

    // Collect results
    let mut success_count = 0;
    for handle in handles {
        if let Ok(_) = handle.await {
            success_count += 1;
        }
    }

    // All requests should succeed
    assert_eq!(success_count, 10);
}