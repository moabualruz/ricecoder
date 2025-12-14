//! Use cases for agent operations
//!
//! This module contains use case implementations that orchestrate
//! complex operations using the application services.

use crate::error::AgentError;
use crate::mcp_integration::{ExternalToolBackend, ExternalToolIntegrationService, ToolExecutionResult};
use serde_json::json;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Process and validate tool execution result
pub(crate) fn process_tool_result(tool_name: &str, raw_result: &serde_json::Value) -> Result<ToolExecutionResult, AgentError> {
    // Validate result structure
    if !raw_result.is_object() {
        return Err(AgentError::ExecutionFailed(
            "Invalid tool result format: expected object".to_string(),
        ));
    }

    let success = raw_result
        .get("success")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| AgentError::ExecutionFailed(
            "Missing or invalid 'success' field in tool result".to_string(),
        ))?;

    let execution_time_ms = raw_result
        .get("execution_time_ms")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    // Validate execution time is reasonable
    if execution_time_ms > 300_000 { // 5 minutes
        warn!(
            execution_time_ms = execution_time_ms,
            "Unusually long tool execution time detected"
        );
    }

    if success {
        // Validate successful result structure
        let data = raw_result.get("result").cloned();

        // Validate metadata if present
        if let Some(metadata) = raw_result.get("metadata") {
            if let Some(provider) = metadata.get("provider").and_then(|p| p.as_str()) {
                debug!(
                    provider = provider,
                    execution_time_ms = execution_time_ms,
                    "Tool executed successfully"
                );
            }
        }

        Ok(ToolExecutionResult {
            success: true,
            data,
            error: None,
            execution_time_ms,
        })
    } else {
        // Process error result
        let error_message = raw_result
            .get("error")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Unknown tool execution error".to_string());

        // Log error details
        error!(
            tool_name = %tool_name,
            error = %error_message,
            execution_time_ms = execution_time_ms,
            "Tool execution failed"
        );

        // Check for specific error types
        let processed_error = if error_message.contains("timeout") {
            format!("Tool execution timed out: {}", error_message)
        } else if error_message.contains("connection") {
            format!("Tool connection error: {}", error_message)
        } else if error_message.contains("authentication") {
            format!("Tool authentication error: {}", error_message)
        } else {
            error_message
        };

        Ok(ToolExecutionResult {
            success: false,
            data: None,
            error: Some(processed_error),
            execution_time_ms,
        })
    }
}

/// Use case for executing external tools
pub struct ExecuteExternalToolUseCase {
    tool_integration: Arc<ExternalToolIntegrationService>,
}

impl ExecuteExternalToolUseCase {
    /// Create a new tool execution use case
    pub fn new(tool_integration: Arc<ExternalToolIntegrationService>) -> Self {
        Self { tool_integration }
    }

    /// Execute a tool with the given parameters
    pub async fn execute(
        &self,
        tool_name: &str,
        parameters: serde_json::Value,
        session_id: Option<String>,
    ) -> Result<ToolExecutionResult, AgentError> {
        debug!(
            tool_name = %tool_name,
            session_id = ?session_id,
            "Executing external tool use case"
        );

        // Check if tool integration is ready
        if !self.tool_integration.is_ready().await {
            warn!("Tool integration service is not ready");
            return Err(AgentError::ExecutionFailed(
                "Tool integration service not configured".to_string(),
            ));
        }

        // Execute the tool through the integration service
        let result = self.tool_integration
            .execute_tool(tool_name, parameters, session_id)
            .await?;

        // Parse and validate the result
        let execution_result = process_tool_result(tool_name, &result)?;

        info!(
            tool_name = %tool_name,
            success = execution_result.success,
            execution_time_ms = execution_result.execution_time_ms,
            "Tool execution completed"
        );

        Ok(execution_result)
    }
}

/// Use case for configuring tool backends
pub struct ConfigureToolBackendUseCase {
    tool_integration: Arc<ExternalToolIntegrationService>,
}

impl ConfigureToolBackendUseCase {
    /// Create a new backend configuration use case
    pub fn new(tool_integration: Arc<ExternalToolIntegrationService>) -> Self {
        Self { tool_integration }
    }

    /// Configure an MCP backend
    #[cfg(feature = "mcp")]
    pub async fn configure_mcp_backend(
        &self,
        server_name: String,
        server_command: String,
        server_args: Vec<String>,
    ) -> Result<(), AgentError> {
        debug!(
            server_name = %server_name,
            server_command = %server_command,
            server_args = ?server_args,
            "Configuring MCP backend"
        );

        let backend = ExternalToolBackend::mcp(server_command, server_args);
        self.tool_integration
            .configure_backend(server_name.clone(), backend)
            .await?;

        info!(server_name = %server_name, "MCP backend configured successfully");
        Ok(())
    }

    /// Configure an MCP backend (mock implementation when MCP feature is disabled)
    #[cfg(not(feature = "mcp"))]
    pub async fn configure_mcp_backend(
        &self,
        server_name: String,
        _server_command: String,
        _server_args: Vec<String>,
    ) -> Result<(), AgentError> {
        debug!(server_name = %server_name, "Configuring mock MCP backend");

        let backend = ExternalToolBackend::mcp_default();
        self.tool_integration
            .configure_backend(server_name.clone(), backend)
            .await?;

        info!(server_name = %server_name, "Mock MCP backend configured successfully");
        Ok(())
    }

    /// Configure an HTTP API backend
    pub async fn configure_http_backend(
        &self,
        name: String,
        base_url: String,
    ) -> Result<(), AgentError> {
        debug!(name = %name, base_url = %base_url, "Configuring HTTP API backend");

        let backend = ExternalToolBackend::http_api(base_url);
        self.tool_integration
            .configure_backend(name.clone(), backend)
            .await?;

        info!(name = %name, "HTTP API backend configured successfully");
        Ok(())
    }

    /// Get current configuration status
    pub async fn get_configuration_status(&self) -> serde_json::Value {
        json!({
            "configured_backends": self.tool_integration.get_configured_backends().await,
            "ready": self.tool_integration.is_ready().await
        })
    }
}

/// Use case for managing tool operations
pub struct ToolManagementUseCase {
    tool_integration: Arc<ExternalToolIntegrationService>,
}

impl ToolManagementUseCase {
    /// Create a new tool management use case
    pub fn new(tool_integration: Arc<ExternalToolIntegrationService>) -> Self {
        Self { tool_integration }
    }

    /// List all configured backends
    pub async fn list_backends(&self) -> Vec<String> {
        self.tool_integration.get_configured_backends().await
    }

    /// Check if tool integration is operational
    pub async fn health_check(&self) -> serde_json::Value {
        let is_ready = self.tool_integration.is_ready().await;
        let backends = self.tool_integration.get_configured_backends().await;

        json!({
            "healthy": is_ready,
            "backends_configured": backends.len(),
            "backend_names": backends,
            "timestamp": chrono::Utc::now().to_rfc3339()
        })
    }

    /// Execute a batch of tools
    pub async fn execute_tool_batch(
        &self,
        tools: Vec<(String, serde_json::Value)>,
        session_id: Option<String>,
    ) -> Result<Vec<ToolExecutionResult>, AgentError> {
        let mut results = Vec::new();

        for (tool_name, parameters) in tools {
            let result = self.tool_integration
                .execute_tool(&tool_name, parameters, session_id.clone())
                .await?;

            // Parse and validate result
            let execution_result = process_tool_result(&tool_name, &result)?;

            results.push(execution_result);
        }

        Ok(results)
    }
}