//! External Tool Integration for RiceCoder Agents
//!
//! This module provides application layer integration for external tool execution,
//! allowing agents to execute tools through various backends including MCP servers,
//! external APIs, and custom implementations.

use crate::error::AgentError;
use crate::tool_invokers::{ExtensibleToolInvoker, ToolBackend};
use crate::tool_registry::ToolInvoker;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

#[cfg(feature = "mcp")]
use rust_mcp_sdk::client::McpClient;
#[cfg(feature = "mcp")]
use rust_mcp_sdk::transport::stdio::StdioTransport;

/// External Tool Backend
///
/// Generic backend for executing external tools through various protocols
/// including MCP, HTTP APIs, and custom implementations.
#[derive(Clone)]
pub struct ExternalToolBackend {
    backend_type: String,
    executor: Arc<dyn ToolExecutor + Send + Sync>,
}

impl ExternalToolBackend {
    /// Create a new external tool backend
    pub fn new(backend_type: String, executor: Arc<dyn ToolExecutor + Send + Sync>) -> Self {
        Self {
            backend_type,
            executor,
        }
    }

    /// Create an MCP backend
    #[cfg(feature = "mcp")]
    pub fn mcp(server_command: String, server_args: Vec<String>) -> Self {
        Self::new("mcp".to_string(), Arc::new(MCPToolExecutor::new(server_command, server_args)))
    }

    /// Create an MCP backend with default settings (placeholder)
    pub fn mcp_default() -> Self {
        Self::new("mcp".to_string(), Arc::new(MockToolExecutor::new()))
    }

    /// Create an HTTP API backend
    pub fn http_api(base_url: String) -> Self {
        Self::new("http_api".to_string(), Arc::new(HTTPToolExecutor::new(base_url)))
    }
}

#[async_trait::async_trait]
impl ToolBackend for ExternalToolBackend {
    async fn invoke_tool(&self, input: serde_json::Value) -> Result<serde_json::Value, String> {
        debug!(backend_type = %self.backend_type, "Invoking external tool");

        // Extract tool name and parameters
        let tool_name = input
            .get("tool_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'tool_name' field in input".to_string())?;

        let parameters = input
            .get("parameters")
            .cloned()
            .unwrap_or_else(|| json!({}));

        let backend_config = input
            .get("backend_config")
            .cloned()
            .unwrap_or_else(|| json!({}));

        info!(tool_name = %tool_name, backend_type = %self.backend_type, "Executing external tool");

        // Execute through the backend executor
        let result = self.executor.execute_tool(tool_name, parameters, backend_config).await?;

        Ok(json!({
            "success": result.success,
            "result": result.data,
            "error": result.error,
            "execution_time_ms": result.execution_time_ms,
            "metadata": {
                "provider": "external",
                "backend": self.backend_type,
                "tool_name": tool_name
            }
        }))
    }

    fn backend_metadata(&self) -> serde_json::Value {
        json!({
            "backend_type": self.backend_type,
            "capabilities": ["tool_execution"],
            "executor_type": std::any::type_name::<dyn ToolExecutor>()
        })
    }
}

/// Tool execution result
pub struct ToolExecutionResult {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
}

/// Tool executor trait for external tool execution
#[async_trait::async_trait]
pub trait ToolExecutor: Send + Sync {
    /// Execute a tool with the given parameters
    async fn execute_tool(
        &self,
        tool_name: &str,
        parameters: serde_json::Value,
        config: serde_json::Value,
    ) -> Result<ToolExecutionResult, String>;
}

/// Mock tool executor for development and testing
pub struct MockToolExecutor;

impl MockToolExecutor {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl ToolExecutor for MockToolExecutor {
    async fn execute_tool(
        &self,
        tool_name: &str,
        _parameters: serde_json::Value,
        _config: serde_json::Value,
    ) -> Result<ToolExecutionResult, String> {
        // Mock implementation - return success for any tool
        Ok(ToolExecutionResult {
            success: true,
            data: Some(json!({
                "tool_executed": tool_name,
                "mock_result": true
            })),
            error: None,
            execution_time_ms: 100,
        })
    }
}

/// HTTP API tool executor
pub struct HTTPToolExecutor {
    base_url: String,
    client: reqwest::Client,
}

impl HTTPToolExecutor {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl ToolExecutor for HTTPToolExecutor {
    async fn execute_tool(
        &self,
        tool_name: &str,
        parameters: serde_json::Value,
        _config: serde_json::Value,
    ) -> Result<ToolExecutionResult, String> {
        let start_time = std::time::Instant::now();

        // Construct API endpoint
        let endpoint = format!("{}/tools/{}", self.base_url.trim_end_matches('/'), tool_name);

        // Make HTTP request
        let response = self.client
            .post(&endpoint)
            .json(&parameters)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        let execution_time = start_time.elapsed().as_millis() as u64;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await
                .map_err(|e| format!("Failed to parse response: {}", e))?;

            Ok(ToolExecutionResult {
                success: true,
                data: Some(result),
                error: None,
                execution_time_ms: execution_time,
            })
        } else {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());

            Ok(ToolExecutionResult {
                success: false,
                data: None,
                error: Some(error_text),
                execution_time_ms: execution_time,
            })
        }
    }
}

/// MCP (Model Context Protocol) tool executor
#[cfg(feature = "mcp")]
pub struct MCPToolExecutor {
    client: Arc<RwLock<Option<McpClient<StdioTransport>>>>,
    server_command: String,
    server_args: Vec<String>,
}

#[cfg(feature = "mcp")]
impl MCPToolExecutor {
    /// Create a new MCP tool executor
    pub fn new(server_command: String, server_args: Vec<String>) -> Self {
        Self {
            client: Arc::new(RwLock::new(None)),
            server_command,
            server_args,
        }
    }

    /// Ensure MCP client is initialized
    async fn ensure_client(&self) -> Result<(), String> {
        let mut client_guard = self.client.write().await;

        if client_guard.is_none() {
            debug!(
                command = %self.server_command,
                args = ?self.server_args,
                "Initializing MCP client"
            );

            // Create stdio transport
            let transport = StdioTransport::new(&self.server_command, &self.server_args)
                .map_err(|e| format!("Failed to create MCP transport: {}", e))?;

            // Create MCP client
            let client = McpClient::new(transport);

            // Initialize the client
            client.initialize().await
                .map_err(|e| format!("Failed to initialize MCP client: {}", e))?;

            *client_guard = Some(client);

            info!("MCP client initialized successfully");
        }

        Ok(())
    }
}

#[cfg(feature = "mcp")]
#[async_trait::async_trait]
impl ToolExecutor for MCPToolExecutor {
    async fn execute_tool(
        &self,
        tool_name: &str,
        parameters: serde_json::Value,
        _config: serde_json::Value,
    ) -> Result<ToolExecutionResult, String> {
        let start_time = std::time::Instant::now();

        // Ensure client is initialized
        self.ensure_client().await?;

        let client_guard = self.client.read().await;
        let client = client_guard.as_ref()
            .ok_or_else(|| "MCP client not initialized".to_string())?;

        debug!(tool_name = %tool_name, "Executing tool via MCP");

        // Execute tool using MCP
        let result = client.call_tool(tool_name, parameters).await
            .map_err(|e| format!("MCP tool execution failed: {}", e))?;

        let execution_time = start_time.elapsed().as_millis() as u64;

        // Convert MCP result to our format
        match result {
            rust_mcp_sdk::schema::CallToolResult::Success { content, .. } => {
                // Extract the first text content if available
                let data = if let Some(first_content) = content.first() {
                    match first_content {
                        rust_mcp_sdk::schema::ToolResultContent::Text { text } => {
                            Some(json!(text))
                        }
                        rust_mcp_sdk::schema::ToolResultContent::Image { .. } => {
                            Some(json!({"type": "image", "content": "Image content not supported"}))
                        }
                        rust_mcp_sdk::schema::ToolResultContent::Resource { .. } => {
                            Some(json!({"type": "resource", "content": "Resource content not supported"}))
                        }
                    }
                } else {
                    Some(json!({"content": []}))
                };

                Ok(ToolExecutionResult {
                    success: true,
                    data,
                    error: None,
                    execution_time_ms: execution_time,
                })
            }
            rust_mcp_sdk::schema::CallToolResult::Error { error } => {
                Ok(ToolExecutionResult {
                    success: false,
                    data: None,
                    error: Some(error.message),
                    execution_time_ms: execution_time,
                })
            }
        }
    }
}

/// MCP Security Configuration
#[derive(Clone, Debug)]
pub struct McpSecurityConfig {
    /// Whether to require authentication for MCP operations
    pub require_authentication: bool,
    /// Allowed server commands (whitelist)
    pub allowed_commands: Vec<String>,
    /// Tool execution permissions
    pub tool_permissions: HashMap<String, ToolPermission>,
    /// Audit logging enabled
    pub audit_logging: bool,
    /// Maximum execution time per tool (seconds)
    pub max_execution_time_secs: u64,
    /// Rate limiting configuration
    pub rate_limit: Option<RateLimitConfig>,
}

/// Tool permission configuration
#[derive(Clone, Debug)]
pub struct ToolPermission {
    /// Whether the tool is allowed to execute
    pub allowed: bool,
    /// Required authentication level
    pub auth_level: AuthLevel,
    /// Parameter validation rules
    pub parameter_validation: ParameterValidation,
}

/// Authentication level required
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuthLevel {
    /// No authentication required
    None,
    /// Basic authentication
    Basic,
    /// Session-based authentication
    Session,
    /// Admin-level authentication
    Admin,
}

/// Parameter validation rules
#[derive(Clone, Debug)]
pub struct ParameterValidation {
    /// Maximum parameter size (bytes)
    pub max_size: usize,
    /// Allowed parameter types
    pub allowed_types: Vec<String>,
    /// Custom validation rules
    pub custom_rules: Vec<String>,
}

/// Rate limiting configuration
#[derive(Clone, Debug)]
pub struct RateLimitConfig {
    /// Maximum requests per minute
    pub requests_per_minute: u32,
    /// Maximum concurrent executions
    pub max_concurrent: u32,
}

/// External Tool Integration Service
///
/// Application service that manages external tool execution backends
/// for agents. This provides the clean architecture application layer interface
/// for tool integration without tight coupling to specific protocols.
pub struct ExternalToolIntegrationService {
    tool_invoker: ExtensibleToolInvoker,
    configured_backends: RwLock<Vec<String>>,
    security_config: McpSecurityConfig,
    audit_log: RwLock<Vec<AuditEntry>>,
}

/// Audit log entry for MCP operations
#[derive(Clone, Debug)]
pub struct AuditEntry {
    pub timestamp: u64,
    pub operation: String,
    pub server: String,
    pub tool: Option<String>,
    pub user: Option<String>,
    pub session_id: Option<String>,
    pub success: bool,
    pub error_message: Option<String>,
    pub execution_time_ms: Option<u64>,
}

    /// Check if the operation is allowed based on security configuration
    async fn check_security(&self, server: &str, tool: &str, session_id: Option<&str>) -> Result<(), AgentError> {
        // Check if authentication is required
        if self.security_config.require_authentication && session_id.is_none() {
            self.log_audit("authentication_required", server, Some(tool), session_id, false, Some("Authentication required".to_string()), None).await;
            return Err(AgentError::AuthenticationRequired);
        }

        // Check tool permissions
        if let Some(permission) = self.security_config.tool_permissions.get(tool) {
            if !permission.allowed {
                self.log_audit("tool_not_allowed", server, Some(tool), session_id, false, Some("Tool execution not allowed".to_string()), None).await;
                return Err(AgentError::PermissionDenied("Tool execution not allowed".to_string()));
            }

            // Check authentication level
            match permission.auth_level {
                AuthLevel::None => {}
                AuthLevel::Basic | AuthLevel::Session => {
                    if session_id.is_none() {
                        self.log_audit("insufficient_auth", server, Some(tool), session_id, false, Some("Insufficient authentication level".to_string()), None).await;
                        return Err(AgentError::AuthenticationRequired);
                    }
                }
                AuthLevel::Admin => {
                    // TODO: Check admin privileges
                    self.log_audit("admin_required", server, Some(tool), session_id, false, Some("Admin privileges required".to_string()), None).await;
                    return Err(AgentError::PermissionDenied("Admin privileges required".to_string()));
                }
            }
        }

        Ok(())
    }

    /// Validate tool parameters against security rules
    fn validate_parameters(&self, tool: &str, parameters: &serde_json::Value) -> Result<(), AgentError> {
        if let Some(permission) = self.security_config.tool_permissions.get(tool) {
            let param_size = serde_json::to_string(parameters).map(|s| s.len()).unwrap_or(0);

            if param_size > permission.parameter_validation.max_size {
                return Err(AgentError::ValidationError(format!(
                    "Parameter size {} exceeds maximum allowed size {}",
                    param_size, permission.parameter_validation.max_size
                )));
            }

            // Validate parameter types
            self.validate_parameter_types(parameters, &permission.parameter_validation.allowed_types)?;
        }

        Ok(())
    }

    /// Validate parameter types recursively
    fn validate_parameter_types(&self, value: &serde_json::Value, allowed_types: &[String]) -> Result<(), AgentError> {
        let value_type = match value {
            serde_json::Value::Null => "null",
            serde_json::Value::Bool(_) => "boolean",
            serde_json::Value::Number(_) => "number",
            serde_json::Value::String(_) => "string",
            serde_json::Value::Array(_) => "array",
            serde_json::Value::Object(_) => "object",
        };

        if !allowed_types.contains(&value_type.to_string()) {
            return Err(AgentError::ValidationError(format!(
                "Parameter type '{}' is not allowed. Allowed types: {:?}",
                value_type, allowed_types
            )));
        }

        // Recursively validate nested structures
        match value {
            serde_json::Value::Array(arr) => {
                for item in arr {
                    self.validate_parameter_types(item, allowed_types)?;
                }
            }
            serde_json::Value::Object(obj) => {
                for (_, val) in obj {
                    self.validate_parameter_types(val, allowed_types)?;
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Log audit entry
    async fn log_audit(
        &self,
        operation: &str,
        server: &str,
        tool: Option<&str>,
        session_id: Option<&str>,
        success: bool,
        error_message: Option<String>,
        execution_time_ms: Option<u64>,
    ) {
        if !self.security_config.audit_logging {
            return;
        }

        let entry = AuditEntry {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            operation: operation.to_string(),
            server: server.to_string(),
            tool: tool.map(|s| s.to_string()),
            user: None, // TODO: Add user context
            session_id: session_id.map(|s| s.to_string()),
            success,
            error_message,
            execution_time_ms,
        };

        let mut audit_log = self.audit_log.write().await;
        audit_log.push(entry);

        // Keep only last 1000 entries
        if audit_log.len() > 1000 {
            audit_log.drain(0..100);
        }
    }

    /// Get audit log
    pub async fn get_audit_log(&self) -> Vec<AuditEntry> {
        self.audit_log.read().await.clone()
    }

    /// Update security configuration
    pub fn update_security_config(&mut self, config: McpSecurityConfig) {
        self.security_config = config;
    }

    /// Get current security configuration
    pub fn get_security_config(&self) -> &McpSecurityConfig {
        &self.security_config
    }
}
    /// Execute tool with retry logic and error classification
    async fn execute_with_retry(
        &self,
        tool_name: &str,
        input: serde_json::Value,
    ) -> Result<serde_json::Value, AgentError> {
        let max_retries = 3;
        let mut last_error = None;

        for attempt in 1..=max_retries {
            debug!(
                tool_name = %tool_name,
                attempt = attempt,
                max_retries = max_retries,
                "Attempting tool execution"
            );

            // Execute with timeout
            let execution_result = tokio::time::timeout(
                std::time::Duration::from_secs(30),
                self.tool_invoker.invoke(input.clone())
            ).await;

            match execution_result {
                Ok(Ok(result)) => {
                    // Check if the result indicates success
                    if result.get("success") == Some(&json!(true)) {
                        if attempt > 1 {
                            info!(
                                tool_name = %tool_name,
                                attempt = attempt,
                                "Tool execution succeeded after retry"
                            );
                        }
                        return Ok(result);
                    } else {
                        // Tool returned error result
                        let error_msg = result
                            .get("error")
                            .and_then(|e| e.as_str())
                            .unwrap_or("Unknown tool error");

                        // Classify error to determine if retry is appropriate
                        if self.should_retry_error(error_msg) && attempt < max_retries {
                            warn!(
                                tool_name = %tool_name,
                                attempt = attempt,
                                error = %error_msg,
                                "Retrying tool execution due to transient error"
                            );
                            last_error = Some(AgentError::ExecutionFailed(error_msg.to_string()));
                            tokio::time::sleep(std::time::Duration::from_millis(500 * attempt as u64)).await;
                            continue;
                        } else {
                            return Err(AgentError::ExecutionFailed(error_msg.to_string()));
                        }
                    }
                }
                Ok(Err(e)) => {
                    // Tool invoker error
                    if self.should_retry_error(&e) && attempt < max_retries {
                        warn!(
                            tool_name = %tool_name,
                            attempt = attempt,
                            error = %e,
                            "Retrying tool execution due to invoker error"
                        );
                        last_error = Some(AgentError::ExecutionFailed(e));
                        tokio::time::sleep(std::time::Duration::from_millis(500 * attempt as u64)).await;
                        continue;
                    } else {
                        return Err(AgentError::ExecutionFailed(e));
                    }
                }
                Err(_) => {
                    // Timeout
                    if attempt < max_retries {
                        warn!(
                            tool_name = %tool_name,
                            attempt = attempt,
                            "Tool execution timed out, retrying"
                        );
                        last_error = Some(AgentError::Timeout(30000));
                        tokio::time::sleep(std::time::Duration::from_millis(1000 * attempt as u64)).await;
                        continue;
                    } else {
                        return Err(AgentError::Timeout(30000));
                    }
                }
            }
        }

        // All retries exhausted
        error!(
            tool_name = %tool_name,
            max_retries = max_retries,
            "Tool execution failed after all retries"
        );
        Err(last_error.unwrap_or_else(|| AgentError::ExecutionFailed("Unknown error after retries".to_string())))
    }

    /// Determine if an error should trigger a retry
    fn should_retry_error(&self, error_msg: &str) -> bool {
        let error_lower = error_msg.to_lowercase();

        // Retry on transient errors
        error_lower.contains("timeout") ||
        error_lower.contains("connection refused") ||
        error_lower.contains("connection reset") ||
        error_lower.contains("temporary failure") ||
        error_lower.contains("server unavailable") ||
        error_lower.contains("rate limit") ||
        error_lower.contains("too many requests")
    }
}

impl Default for ExternalToolIntegrationService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_external_tool_backend_mock() {
        let backend = ExternalToolBackend::new(
            "test".to_string(),
            Arc::new(MockToolExecutor::new())
        );

        let input = json!({
            "tool_name": "test_tool",
            "parameters": {"param": "value"},
            "backend_config": {}
        });

        let result = backend.invoke_tool(input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output["success"], true);
        assert_eq!(output["result"]["tool_executed"], "test_tool");
    }

    #[tokio::test]
    async fn test_external_tool_backend_missing_tool_name() {
        let backend = ExternalToolBackend::new(
            "test".to_string(),
            Arc::new(MockToolExecutor::new())
        );

        let input = json!({
            "parameters": {"param": "value"}
        });

        let result = backend.invoke_tool(input).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing 'tool_name' field"));
    }

    #[tokio::test]
    async fn test_http_tool_executor() {
        let executor = HTTPToolExecutor::new("https://httpbin.org".to_string());

        let result = executor.execute_tool(
            "get",
            json!({"url": "https://httpbin.org/get"}),
            json!({})
        ).await;

        // This will fail because httpbin.org might not have a /tools/get endpoint,
        // but we're testing that the executor attempts the request
        assert!(result.is_ok()); // The result is Ok, but contains an error response
        let execution_result = result.unwrap();
        assert!(!execution_result.success); // Should fail due to 404
    }

    #[cfg(feature = "mcp")]
    #[tokio::test]
    async fn test_mcp_tool_executor_initialization() {
        // Test that MCP executor can be created
        let executor = MCPToolExecutor::new(
            "echo".to_string(),
            vec!["hello".to_string()]
        );

        // Test client initialization (this would normally connect to a real server)
        // For now, we just test that the structure is created correctly
        assert_eq!(executor.server_command, "echo");
        assert_eq!(executor.server_args, vec!["hello"]);
    }

    #[tokio::test]
    async fn test_external_tool_integration_service() {
        let service = ExternalToolIntegrationService::new();

        // Test initial state
        assert!(!service.is_ready().await);
        assert!(service.get_configured_backends().await.is_empty());

        // Configure a mock backend
        service.configure_backend(
            "test_backend".to_string(),
            ExternalToolBackend::new(
                "mock".to_string(),
                Arc::new(MockToolExecutor::new())
            )
        ).await.unwrap();

        // Test configured state
        assert!(service.is_ready().await);
        assert_eq!(service.get_configured_backends().await, vec!["test_backend"]);

        // Test tool execution
        let result = service.execute_tool(
            "test_tool",
            json!({"param": "value"}),
            Some("session_123".to_string())
        ).await.unwrap();

        assert_eq!(result["success"], true);
        assert_eq!(result["result"]["tool_executed"], "test_tool");
    }

    #[tokio::test]
    async fn test_tool_execution_with_retry() {
        let service = ExternalToolIntegrationService::new();

        // Configure a backend that will fail
        struct FailingToolExecutor;

        #[async_trait::async_trait]
        impl ToolExecutor for FailingToolExecutor {
            async fn execute_tool(
                &self,
                _tool_name: &str,
                _parameters: serde_json::Value,
                _config: serde_json::Value,
            ) -> Result<ToolExecutionResult, String> {
                Ok(ToolExecutionResult {
                    success: false,
                    data: None,
                    error: Some("Connection timeout".to_string()),
                    execution_time_ms: 100,
                })
            }
        }

        service.configure_backend(
            "failing_backend".to_string(),
            ExternalToolBackend::new(
                "failing".to_string(),
                Arc::new(FailingToolExecutor)
            )
        ).await.unwrap();

        // Execute tool - should retry and eventually fail
        let result = service.execute_tool(
            "failing_tool",
            json!({}),
            None
        ).await;

        // The service call should fail when tool execution fails after retries
        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            AgentError::ExecutionFailed(msg) => {
                assert!(msg.contains("Connection timeout"));
            }
            _ => panic!("Expected ExecutionFailed error"),
        }
    }

    #[tokio::test]
    async fn test_tool_parameter_validation() {
        let service = ExternalToolIntegrationService::new();

        service.configure_backend(
            "test_backend".to_string(),
            ExternalToolBackend::new(
                "mock".to_string(),
                Arc::new(MockToolExecutor::new())
            )
        ).await.unwrap();

        // Test empty tool name
        let result = service.execute_tool("", json!({}), None).await;
        assert!(result.is_err());

        // Test non-object parameters
        let result = service.execute_tool("test", json!("string"), None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_tool_batch_execution() {
        let service = ExternalToolIntegrationService::new();

        service.configure_backend(
            "test_backend".to_string(),
            ExternalToolBackend::new(
                "mock".to_string(),
                Arc::new(MockToolExecutor::new())
            )
        ).await.unwrap();

        // Create use case for batch execution
        let tool_management = crate::use_cases::ToolManagementUseCase::new(
            Arc::new(service)
        );

        let tools = vec![
            ("tool1".to_string(), json!({"param": "value1"})),
            ("tool2".to_string(), json!({"param": "value2"})),
        ];

        let results = tool_management.execute_tool_batch(tools, None).await.unwrap();

        assert_eq!(results.len(), 2);
        assert!(results[0].success);
        assert!(results[1].success);
    }
}