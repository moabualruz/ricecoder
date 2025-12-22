//! MCP Testing Infrastructure Enhancements
//!
//! This module provides reusable test fixtures, mock servers, automated test data
//! generation, and testing utilities specifically designed for MCP (Model Context Protocol)
//! testing, with particular focus on enterprise security scenarios.

use std::{collections::HashMap, sync::Arc};

use proptest::prelude::*;
// MCP imports
use ricecoder_mcp::{
    error::{Error, Result},
    metadata::ToolMetadata,
    transport::{MCPMessage, MCPRequest, MCPResponse, MCPTransport},
};
use serde_json::{json, Value};

// ============================================================================
// Reusable MCP Test Fixtures
// ============================================================================

/// **Test Fixture: MCP Test Environment**
/// A comprehensive test fixture that sets up a complete MCP environment
pub struct MCPTestEnvironment {
    pub tool_registry: Arc<ricecoder_mcp::registry::ToolRegistry>,
}

impl MCPTestEnvironment {
    pub fn new() -> Self {
        Self {
            tool_registry: Arc::new(ricecoder_mcp::registry::ToolRegistry::new()),
        }
    }

    /// Register a set of test tools
    pub async fn register_test_tools(&self, count: usize) -> Result<()> {
        for i in 0..count {
            let tool = ToolMetadata {
                name: format!("test-tool-{}", i),
                description: format!("Test tool {} for MCP testing", i),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "param1": {"type": "string"}
                    }
                }),
                permissions_required: vec![],
                metadata: None,
            };

            self.tool_registry.register_tool(tool).await?;
        }
        Ok(())
    }
}

// ============================================================================
// Mock MCP Servers
// ============================================================================

/// **Mock MCP Server: Configurable Mock Server**
pub struct MockMCPServer {
    pub server_id: String,
    pub tools: std::sync::Mutex<HashMap<String, ToolMetadata>>,
}

impl MockMCPServer {
    pub fn new(server_id: String) -> Self {
        Self {
            server_id,
            tools: std::sync::Mutex::new(HashMap::new()),
        }
    }

    pub async fn handle_request(&self, request: &MCPRequest) -> Result<MCPResponse> {
        // Handle tool listing
        if request.method == "tools/list" {
            let tools = self.tools.lock().unwrap();
            let tools: Vec<Value> = tools
                .values()
                .map(|tool| {
                    json!({
                        "name": tool.name,
                        "description": tool.description,
                        "inputSchema": tool.input_schema
                    })
                })
                .collect();

            return Ok(MCPResponse {
                id: request.id.clone(),
                result: json!({ "tools": tools }),
            });
        }

        Err(ricecoder_mcp::error::Error::ToolError(
            ricecoder_mcp::error::ToolError::MethodNotFound(request.method.clone()),
        ))
    }
}

// ============================================================================
// Automated Test Data Generation
// ============================================================================

/// **Test Data Generator: MCP Protocol Messages**
pub struct MCPProtocolDataGenerator;

impl MCPProtocolDataGenerator {
    /// Generate a complete set of MCP protocol test scenarios
    pub fn generate_protocol_test_scenarios() -> Vec<MCPProtocolTestScenario> {
        vec![MCPProtocolTestScenario {
            name: "basic_request_response".to_string(),
            request: MCPRequest {
                id: "test-1".to_string(),
                method: "tools/call".to_string(),
                params: Some(json!({"name": "test-tool"})),
            },
            expected_response: Some(MCPResponse {
                id: "test-1".to_string(),
                result: json!({"result": "success"}),
            }),
            should_fail: false,
        }]
    }
}

#[derive(Clone, Debug)]
pub struct MCPProtocolTestScenario {
    pub name: String,
    pub request: MCPRequest,
    pub expected_response: Option<MCPResponse>,
    pub should_fail: bool,
}

// ============================================================================
// MCP-Specific Property-Based Test Generators
// ============================================================================

/// **Generator: Enterprise Tool Metadata**
pub fn arb_enterprise_tool_metadata() -> impl Strategy<Value = ToolMetadata> {
    (
        "[a-zA-Z0-9_-]{1,64}".prop_map(|s| s), // name
        ".{1,256}".prop_map(|s| s),            // description
        arb_enterprise_input_schema(),
        proptest::collection::vec("[a-zA-Z0-9_:.-]{1,128}".prop_map(|s| s), 0..5), // permissions
        proptest::option::of(arb_enterprise_metadata()),
    )
        .prop_map(
            |(name, description, input_schema, permissions_required, metadata)| ToolMetadata {
                name,
                description,
                input_schema,
                permissions_required,
                metadata,
            },
        )
}

fn arb_enterprise_input_schema() -> impl Strategy<Value = Value> {
    Just(json!({
        "type": "object",
        "properties": {
            "target": {"type": "string"},
            "classification": {"type": "string", "enum": ["public", "internal", "confidential"]}
        }
    }))
}

fn arb_enterprise_metadata() -> impl Strategy<Value = Value> {
    Just(json!({"classification": "internal", "compliance": ["GDPR", "SOX"]}))
}

// ============================================================================
// Testing Utilities for Enterprise Security Scenarios
// ============================================================================

/// **Security Testing Utility: Compliance Test Harness**
pub struct ComplianceTestHarness {
    pub environment: MCPTestEnvironment,
}

impl ComplianceTestHarness {
    pub fn new() -> Self {
        Self {
            environment: MCPTestEnvironment::new(),
        }
    }
}

/// **Test Utility: Enhanced Mock Transport**
pub struct EnhancedMockTransport {
    pub request_history: Arc<std::sync::RwLock<Vec<MCPMessage>>>,
}

impl EnhancedMockTransport {
    pub fn new() -> Self {
        Self {
            request_history: Arc::new(std::sync::RwLock::new(Vec::new())),
        }
    }
}

#[async_trait::async_trait]
impl MCPTransport for EnhancedMockTransport {
    async fn send(&self, message: &MCPMessage) -> Result<()> {
        self.request_history.write().unwrap().push(message.clone());
        Ok(())
    }

    async fn receive(&self) -> Result<MCPMessage> {
        Ok(MCPMessage::Response(MCPResponse {
            id: "mock-id".to_string(),
            result: json!({"status": "ok"}),
        }))
    }

    async fn is_connected(&self) -> bool {
        true
    }

    async fn close(&self) -> Result<()> {
        Ok(())
    }
}
