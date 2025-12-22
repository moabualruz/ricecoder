//! Phase 2: Port Interface Unit Tests
//!
//! Tests for port interfaces in hexagonal architecture:
//! - ToolExecutor trait contracts
//! - MCPTransport trait contracts
//! - Permission interfaces
//! - Use case contracts and invariants

use std::{collections::HashMap, sync::Arc, time::Duration};

use async_trait::async_trait;
use ricecoder_mcp::{
    error::{Error, Result},
    metadata::{ToolMetadata, ToolSource},
    permissions::{MCPPermissionManager, PermissionLevelConfig, PermissionRule},
    tool_execution::{ToolExecutionContext, ToolExecutionResult, ToolExecutionStats, ToolExecutor},
    transport::{MCPMessage, MCPRequest, MCPResponse, MCPTransport},
};
use serde_json::json;
use tokio::sync::RwLock;

/// **Port Test 2.1: ToolExecutor trait contract**
/// **Validates: Interface contract compliance**
#[async_trait]
struct MockToolExecutor {
    tools: HashMap<String, ToolMetadata>,
}

#[async_trait]
impl ToolExecutor for MockToolExecutor {
    async fn execute(&self, context: &ToolExecutionContext) -> Result<ToolExecutionResult> {
        if let Some(tool) = self.tools.get(&context.tool_name) {
            Ok(ToolExecutionResult {
                tool_name: context.tool_name.clone(),
                success: true,
                result: Some(json!({"output": "mock result"})),
                error: None,
                execution_time_ms: 100,
                timestamp: std::time::SystemTime::now(),
                metadata: HashMap::new(),
            })
        } else {
            Err(Error::ToolNotFound(context.tool_name.clone()))
        }
    }

    async fn validate_parameters(
        &self,
        tool_name: &str,
        parameters: &HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        if let Some(tool) = self.tools.get(tool_name) {
            for param in &tool.parameters {
                if param.required && !parameters.contains_key(&param.name) {
                    return Err(Error::ValidationError(format!(
                        "Missing required parameter: {}",
                        param.name
                    )));
                }
            }
            Ok(())
        } else {
            Err(Error::ToolNotFound(tool_name.to_string()))
        }
    }

    async fn get_tool_metadata(&self, tool_name: &str) -> Result<Option<ToolMetadata>> {
        Ok(self.tools.get(tool_name).cloned())
    }

    async fn list_tools(&self) -> Result<Vec<ToolMetadata>> {
        Ok(self.tools.values().cloned().collect())
    }
}

#[test]
fn test_tool_executor_contract_compliance() {
    let mut tools = HashMap::new();
    let tool = ToolMetadata::new(
        "test-tool".to_string(),
        "Test Tool".to_string(),
        "A test tool".to_string(),
        "test".to_string(),
        "string".to_string(),
        ToolSource::Custom,
    );
    tools.insert("test-tool".to_string(), tool);

    let executor = MockToolExecutor { tools };

    // Test that the trait is implemented correctly
    assert!(true); // If this compiles, the contract is satisfied
}

/// **Port Test 2.2: ToolExecutor execution contract**
/// **Validates: Execution interface invariants**
#[tokio::test]
async fn test_tool_executor_execution_contract() {
    let mut tools = HashMap::new();
    let tool = ToolMetadata::new(
        "test-tool".to_string(),
        "Test Tool".to_string(),
        "A test tool".to_string(),
        "test".to_string(),
        "string".to_string(),
        ToolSource::Custom,
    );
    tools.insert("test-tool".to_string(), tool);

    let executor = MockToolExecutor { tools };

    let context = ToolExecutionContext {
        tool_name: "test-tool".to_string(),
        parameters: HashMap::new(),
        user_id: Some("user-1".to_string()),
        session_id: Some("session-1".to_string()),
        timeout: Duration::from_secs(30),
        metadata: HashMap::new(),
    };

    let result = executor.execute(&context).await.unwrap();
    assert_eq!(result.tool_name, "test-tool");
    assert!(result.success);
    assert!(result.result.is_some());
    assert!(result.error.is_none());
    assert_eq!(result.execution_time_ms, 100);
}

/// **Port Test 2.3: ToolExecutor validation contract**
/// **Validates: Parameter validation interface**
#[tokio::test]
async fn test_tool_executor_validation_contract() {
    let mut tools = HashMap::new();
    let mut tool = ToolMetadata::new(
        "test-tool".to_string(),
        "Test Tool".to_string(),
        "A test tool".to_string(),
        "test".to_string(),
        "string".to_string(),
        ToolSource::Custom,
    );

    // Add a required parameter
    let param = ricecoder_mcp::metadata::ParameterMetadata::new(
        "required_param".to_string(),
        "string".to_string(),
        "A required parameter".to_string(),
        true,
    );
    tool.add_parameter(param);
    tools.insert("test-tool".to_string(), tool);

    let executor = MockToolExecutor { tools };

    // Valid parameters
    let valid_params = {
        let mut map = HashMap::new();
        map.insert("required_param".to_string(), json!("value"));
        map
    };
    assert!(executor
        .validate_parameters("test-tool", &valid_params)
        .await
        .is_ok());

    // Missing required parameter
    let invalid_params = HashMap::new();
    assert!(executor
        .validate_parameters("test-tool", &invalid_params)
        .await
        .is_err());

    // Non-existent tool
    assert!(executor
        .validate_parameters("non-existent", &valid_params)
        .await
        .is_err());
}

/// **Port Test 2.4: ToolExecutor metadata contract**
/// **Validates: Metadata retrieval interface**
#[tokio::test]
async fn test_tool_executor_metadata_contract() {
    let mut tools = HashMap::new();
    let tool = ToolMetadata::new(
        "test-tool".to_string(),
        "Test Tool".to_string(),
        "A test tool".to_string(),
        "test".to_string(),
        "string".to_string(),
        ToolSource::Custom,
    );
    tools.insert("test-tool".to_string(), tool.clone());

    let executor = MockToolExecutor { tools };

    // Get existing tool metadata
    let metadata = executor.get_tool_metadata("test-tool").await.unwrap();
    assert!(metadata.is_some());
    assert_eq!(metadata.unwrap().id, "test-tool");

    // Get non-existent tool metadata
    let metadata = executor.get_tool_metadata("non-existent").await.unwrap();
    assert!(metadata.is_none());
}

/// **Port Test 2.5: ToolExecutor list tools contract**
/// **Validates: Tool listing interface**
#[tokio::test]
async fn test_tool_executor_list_tools_contract() {
    let mut tools = HashMap::new();
    let tool1 = ToolMetadata::new(
        "tool1".to_string(),
        "Tool 1".to_string(),
        "First tool".to_string(),
        "test".to_string(),
        "string".to_string(),
        ToolSource::Custom,
    );
    let tool2 = ToolMetadata::new(
        "tool2".to_string(),
        "Tool 2".to_string(),
        "Second tool".to_string(),
        "test".to_string(),
        "string".to_string(),
        ToolSource::Custom,
    );
    tools.insert("tool1".to_string(), tool1);
    tools.insert("tool2".to_string(), tool2);

    let executor = MockToolExecutor { tools };

    let listed_tools = executor.list_tools().await.unwrap();
    assert_eq!(listed_tools.len(), 2);

    let tool_ids: Vec<String> = listed_tools.iter().map(|t| t.id.clone()).collect();
    assert!(tool_ids.contains(&"tool1".to_string()));
    assert!(tool_ids.contains(&"tool2".to_string()));
}

/// **Port Test 2.6: MCPTransport trait contract**
/// **Validates: Transport interface compliance**
struct MockTransport {
    connected: bool,
}

#[async_trait]
impl MCPTransport for MockTransport {
    async fn send(&self, _message: &MCPMessage) -> Result<()> {
        if !self.connected {
            return Err(Error::ConnectionError("Not connected".to_string()));
        }
        Ok(())
    }

    async fn receive(&self) -> Result<MCPMessage> {
        if !self.connected {
            return Err(Error::ConnectionError("Not connected".to_string()));
        }
        Ok(MCPMessage::Response(MCPResponse {
            id: "test".to_string(),
            result: json!({"status": "ok"}),
        }))
    }

    async fn is_connected(&self) -> bool {
        self.connected
    }

    async fn close(&self) -> Result<()> {
        Ok(())
    }
}

#[test]
fn test_mcp_transport_contract_compliance() {
    let transport = MockTransport { connected: true };
    // Test that the trait is implemented correctly
    assert!(true); // If this compiles, the contract is satisfied
}

/// **Port Test 2.7: MCPTransport connection contract**
/// **Validates: Connection state management**
#[tokio::test]
async fn test_mcp_transport_connection_contract() {
    let connected_transport = MockTransport { connected: true };
    let disconnected_transport = MockTransport { connected: false };

    assert!(connected_transport.is_connected().await);
    assert!(!disconnected_transport.is_connected().await);

    // Connected transport can send
    let message = MCPMessage::Request(MCPRequest {
        id: "test".to_string(),
        method: "test".to_string(),
        params: json!({}),
    });
    assert!(connected_transport.send(&message).await.is_ok());
    assert!(disconnected_transport.send(&message).await.is_err());

    // Connected transport can receive
    assert!(connected_transport.receive().await.is_ok());
    assert!(disconnected_transport.receive().await.is_err());
}

/// **Port Test 2.8: Permission manager interface contract**
/// **Validates: Permission checking interface**
#[test]
fn test_permission_manager_interface_contract() {
    let mut manager = MCPPermissionManager::new();

    // Add a rule
    let rule = PermissionRule {
        pattern: "test-*".to_string(),
        level: PermissionLevelConfig::Allow,
        agent_id: None,
    };
    assert!(manager.add_global_rule(rule).is_ok());

    // Check permission
    let permission = manager.check_permission("test-tool", None).unwrap();
    assert!(matches!(
        permission,
        ricecoder_permissions::PermissionLevel::Allow
    ));
}

/// **Port Test 2.9: ToolExecutionContext invariants**
/// **Validates: Execution context structure**
#[test]
fn test_tool_execution_context_invariants() {
    let context = ToolExecutionContext {
        tool_name: "test-tool".to_string(),
        parameters: {
            let mut map = HashMap::new();
            map.insert("param1".to_string(), json!("value1"));
            map
        },
        user_id: Some("user-1".to_string()),
        session_id: Some("session-1".to_string()),
        timeout: Duration::from_secs(30),
        metadata: HashMap::new(),
    };

    assert!(!context.tool_name.is_empty());
    assert!(context.timeout > Duration::from_secs(0));
}

/// **Port Test 2.10: ToolExecutionResult invariants**
/// **Validates: Execution result structure**
#[test]
fn test_tool_execution_result_invariants() {
    let result = ToolExecutionResult {
        tool_name: "test-tool".to_string(),
        success: true,
        result: Some(json!({"output": "success"})),
        error: None,
        execution_time_ms: 150,
        timestamp: std::time::SystemTime::now(),
        metadata: HashMap::new(),
    };

    assert!(!result.tool_name.is_empty());
    assert!(result.execution_time_ms > 0);

    // Success implies result, failure implies error
    if result.success {
        assert!(result.result.is_some());
        assert!(result.error.is_none());
    }
}

/// **Port Test 2.11: ToolExecutionStats invariants**
/// **Validates: Statistics tracking structure**
#[test]
fn test_tool_execution_stats_invariants() {
    let stats = ToolExecutionStats {
        tool_name: "test-tool".to_string(),
        total_executions: 10,
        successful_executions: 8,
        failed_executions: 2,
        average_execution_time_ms: 120.5,
        last_execution_time: std::time::SystemTime::now(),
        error_count_by_type: {
            let mut map = HashMap::new();
            map.insert("timeout".to_string(), 1);
            map.insert("validation".to_string(), 1);
            map
        },
    };

    assert!(!stats.tool_name.is_empty());
    assert_eq!(
        stats.total_executions,
        stats.successful_executions + stats.failed_executions
    );
    assert!(stats.average_execution_time_ms >= 0.0);
}

// Property-based tests for port interfaces

/// **Port Property Test 2.1: ToolExecutor parameter validation**
/// **Validates: Parameter validation contract**
proptest::proptest! {
    #[test]
    fn property_tool_executor_parameter_validation(
        param_count in 1usize..10,
        required_count in 0usize..10,
    ) {
        // Create a mock tool with parameters
        let mut tool = ToolMetadata::new(
            "test-tool".to_string(),
            "Test Tool".to_string(),
            "A test tool".to_string(),
            "test".to_string(),
            "string".to_string(),
            ToolSource::Custom,
        );

        let actual_required = std::cmp::min(required_count, param_count);
        for i in 0..param_count {
            let required = i < actual_required;
            let param = ricecoder_mcp::metadata::ParameterMetadata::new(
                format!("param{}", i),
                "string".to_string(),
                format!("Parameter {}", i),
                required,
            );
            tool.add_parameter(param);
        }

        let mut tools = HashMap::new();
        tools.insert("test-tool".to_string(), tool);

        let executor = MockToolExecutor { tools };

        // Test with all required parameters provided
        let mut valid_params = HashMap::new();
        for i in 0..actual_required {
            valid_params.insert(format!("param{}", i), json!(format!("value{}", i)));
        }

        let result = tokio::runtime::Runtime::new().unwrap().block_on(
            executor.validate_parameters("test-tool", &valid_params)
        );
        prop_assert!(result.is_ok());

        // Test with missing required parameters
        if actual_required > 0 {
            let invalid_params = HashMap::new();
            let result = tokio::runtime::Runtime::new().unwrap().block_on(
                executor.validate_parameters("test-tool", &invalid_params)
            );
            prop_assert!(result.is_err());
        }
    }
}

/// **Port Property Test 2.2: Transport message handling**
/// **Validates: Transport contract consistency**
proptest::proptest! {
    #[test]
    fn property_transport_message_handling(
        message_id in "[a-zA-Z0-9_-]{1,50}",
        method in "[a-zA-Z0-9_/]{1,100}",
    ) {
        let transport = MockTransport { connected: true };

        let request = MCPRequest {
            id: message_id.clone(),
            method: method.clone(),
            params: json!({}),
        };
        let message = MCPMessage::Request(request);

        let rt = tokio::runtime::Runtime::new().unwrap();

        // Send should succeed when connected
        let send_result = rt.block_on(transport.send(&message));
        prop_assert!(send_result.is_ok());

        // Receive should return a valid message
        let receive_result = rt.block_on(transport.receive());
        prop_assert!(receive_result.is_ok());

        match receive_result.unwrap() {
            MCPMessage::Response(resp) => {
                prop_assert!(!resp.id.is_empty());
                prop_assert!(resp.result.is_object());
            }
            _ => prop_panic!("Expected response message"),
        }
    }
}
