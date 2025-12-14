//! Tool Execution and Result Handling
//!
//! Provides abstractions for executing MCP tools and handling their results,
//! including validation, error handling, and result processing.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::error::{Error, Result, ToolError};
use crate::metadata::{ParameterMetadata, ToolMetadata};
use crate::permissions::{MCPPermissionManager, PermissionLevelConfig};
use crate::transport::{MCPMessage, MCPRequest, MCPResponse, MCPTransport};

/// Tool execution context
#[derive(Debug, Clone)]
pub struct ToolExecutionContext {
    pub tool_name: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub timeout: Duration,
    pub metadata: HashMap<String, String>,
}

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionResult {
    pub tool_name: String,
    pub success: bool,
    pub result: Option<serde_json::Value>,
    pub error: Option<ToolError>,
    pub execution_time_ms: u64,
    pub timestamp: SystemTime,
    pub metadata: HashMap<String, String>,
}

/// Tool executor trait
#[async_trait]
pub trait ToolExecutor: Send + Sync {
    /// Execute a tool with the given context
    async fn execute(&self, context: &ToolExecutionContext) -> Result<ToolExecutionResult>;

    /// Validate tool parameters
    async fn validate_parameters(&self, tool_name: &str, parameters: &HashMap<String, serde_json::Value>) -> Result<()>;

    /// Get tool metadata
    async fn get_tool_metadata(&self, tool_name: &str) -> Result<Option<ToolMetadata>>;

    /// List available tools
    async fn list_tools(&self) -> Result<Vec<ToolMetadata>>;
}

/// MCP tool executor that communicates with MCP servers
pub struct MCPToolExecutor {
    transport: Arc<dyn MCPTransport>,
    permission_manager: Arc<MCPPermissionManager>,
    default_timeout: Duration,
    execution_stats: Arc<RwLock<HashMap<String, ToolExecutionStats>>>,
}

impl MCPToolExecutor {
    /// Create a new MCP tool executor
    pub fn new(
        transport: Arc<dyn MCPTransport>,
        permission_manager: Arc<MCPPermissionManager>,
    ) -> Self {
        Self {
            transport,
            permission_manager,
            default_timeout: Duration::from_secs(30),
            execution_stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create with custom timeout
    pub fn with_timeout(
        transport: Arc<dyn MCPTransport>,
        permission_manager: Arc<MCPPermissionManager>,
        timeout: Duration,
    ) -> Self {
        Self {
            transport,
            permission_manager,
            default_timeout: timeout,
            execution_stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Update execution statistics
    async fn update_stats(&self, tool_name: &str, success: bool, execution_time_ms: u64) {
        let mut stats = self.execution_stats.write().await;
        let tool_stats = stats.entry(tool_name.to_string()).or_insert_with(|| ToolExecutionStats {
            tool_name: tool_name.to_string(),
            total_calls: 0,
            successful_calls: 0,
            failed_calls: 0,
            total_execution_time_ms: 0,
            average_execution_time_ms: 0.0,
            last_execution: None,
        });

        tool_stats.total_calls += 1;
        tool_stats.total_execution_time_ms += execution_time_ms;
        tool_stats.average_execution_time_ms = tool_stats.total_execution_time_ms as f64 / tool_stats.total_calls as f64;
        tool_stats.last_execution = Some(SystemTime::now());

        if success {
            tool_stats.successful_calls += 1;
        } else {
            tool_stats.failed_calls += 1;
        }
    }

    /// Get execution statistics for a tool
    pub async fn get_tool_stats(&self, tool_name: &str) -> Option<ToolExecutionStats> {
        let stats = self.execution_stats.read().await;
        stats.get(tool_name).cloned()
    }

    /// Get all execution statistics
    pub async fn get_all_stats(&self) -> HashMap<String, ToolExecutionStats> {
        let stats = self.execution_stats.read().await;
        stats.clone()
    }
}

#[async_trait]
impl ToolExecutor for MCPToolExecutor {
    async fn execute(&self, context: &ToolExecutionContext) -> Result<ToolExecutionResult> {
        let start_time = SystemTime::now();

        // Check permissions
        let has_permission = self.permission_manager
            .check_permission(&context.tool_name, context.user_id.as_deref())
            .unwrap_or(false);

        if !has_permission {
            return Ok(ToolExecutionResult {
                tool_name: context.tool_name.clone(),
                success: false,
                result: None,
                error: Some(ToolError::new(
                    context.tool_name.clone(),
                    "Permission denied".to_string(),
                    "permission_denied".to_string(),
                )),
                execution_time_ms: 0,
                timestamp: start_time,
                metadata: context.metadata.clone(),
            });
        }

        // Validate parameters
        self.validate_parameters(&context.tool_name, &context.parameters).await?;

        // Create MCP request
        let request_id = format!("req_{}", uuid::Uuid::new_v4().simple());
        let request = MCPRequest {
            id: request_id.clone(),
            method: format!("tools/{}", context.tool_name),
            params: serde_json::to_value(&context.parameters)
                .map_err(|e| Error::SerializationError(e))?,
        };

        let message = MCPMessage::Request(request);

        // Send request
        debug!("Executing tool '{}' with parameters: {:?}", context.tool_name, context.parameters);
        self.transport.send(&message).await?;

        // Wait for response with timeout
        let timeout_result = tokio::time::timeout(context.timeout, self.transport.receive()).await;

        let execution_time_ms = start_time.elapsed()
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64;

        match timeout_result {
            Ok(Ok(MCPMessage::Response(response))) => {
                if response.id == request_id {
                    // Successful execution
                    let success = true;
                    self.update_stats(&context.tool_name, success, execution_time_ms).await;

                    Ok(ToolExecutionResult {
                        tool_name: context.tool_name.clone(),
                        success: true,
                        result: Some(response.result),
                        error: None,
                        execution_time_ms,
                        timestamp: start_time,
                        metadata: context.metadata.clone(),
                    })
                } else {
                    // Response ID mismatch
                    let success = false;
                    self.update_stats(&context.tool_name, success, execution_time_ms).await;

                    Ok(ToolExecutionResult {
                        tool_name: context.tool_name.clone(),
                        success: false,
                        result: None,
                        error: Some(ToolError::new(
                        context.tool_name.clone(),
                        "Response ID mismatch".to_string(),
                        "execution_error".to_string(),
                    )),
                        execution_time_ms,
                        timestamp: start_time,
                        metadata: context.metadata.clone(),
                    })
                }
            }
            Ok(Ok(MCPMessage::Error(mcp_error))) => {
                // MCP error response
                let success = false;
                self.update_stats(&context.tool_name, success, execution_time_ms).await;

                let tool_error = ToolError::new(
                    context.tool_name.clone(),
                    format!("MCP error {}: {}", mcp_error.error.code, mcp_error.error.message),
                    "execution_error".to_string(),
                );

                Ok(ToolExecutionResult {
                    tool_name: context.tool_name.clone(),
                    success: false,
                    result: None,
                    error: Some(tool_error),
                    execution_time_ms,
                    timestamp: start_time,
                    metadata: context.metadata.clone(),
                })
            }
            Ok(Ok(_)) => {
                // Unexpected message type
                let success = false;
                self.update_stats(&context.tool_name, success, execution_time_ms).await;

                Ok(ToolExecutionResult {
                    tool_name: context.tool_name.clone(),
                    success: false,
                    result: None,
                    error: Some(ToolError::new(
                        context.tool_name.clone(),
                        "Unexpected response type".to_string(),
                        "execution_error".to_string(),
                    )),
                    execution_time_ms,
                    timestamp: start_time,
                    metadata: context.metadata.clone(),
                })
            }
            Ok(Err(e)) => {
                // Transport error
                let success = false;
                self.update_stats(&context.tool_name, success, execution_time_ms).await;

                Ok(ToolExecutionResult {
                    tool_name: context.tool_name.clone(),
                    success: false,
                    result: None,
                    error: Some(ToolError::new(
                        context.tool_name.clone(),
                        format!("Transport error: {}", e),
                        "execution_error".to_string(),
                    )),
                    execution_time_ms,
                    timestamp: start_time,
                    metadata: context.metadata.clone(),
                })
            }
            Err(_) => {
                // Timeout
                let success = false;
                self.update_stats(&context.tool_name, success, execution_time_ms).await;

                Ok(ToolExecutionResult {
                    tool_name: context.tool_name.clone(),
                    success: false,
                    result: None,
                    error: Some(ToolError::new(
                        context.tool_name.clone(),
                        format!("Timeout after {}ms", context.timeout.as_millis()),
                        "timeout".to_string(),
                    )),
                    execution_time_ms,
                    timestamp: start_time,
                    metadata: context.metadata.clone(),
                })
            }
        }
    }

    async fn validate_parameters(&self, tool_name: &str, parameters: &HashMap<String, serde_json::Value>) -> Result<()> {
        // Get tool metadata to validate parameters
        if let Some(metadata) = self.get_tool_metadata(tool_name).await? {
            for param_meta in &metadata.parameters {
                match parameters.get(&param_meta.name) {
                    Some(value) => {
                        // Basic type validation
                        if let Err(e) = self.validate_parameter_value(&param_meta, value) {
                            return Err(Error::ParameterValidationError(format!(
                                "Parameter '{}' validation failed: {}", param_meta.name, e
                            )));
                        }
                    }
                    None => {
                        if param_meta.required {
                            return Err(Error::ParameterValidationError(format!(
                                "Required parameter '{}' is missing", param_meta.name
                            )));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn get_tool_metadata(&self, tool_name: &str) -> Result<Option<ToolMetadata>> {
        // This would typically query the MCP server for tool metadata
        // For now, return None (would be implemented with actual MCP server communication)
        warn!("get_tool_metadata not fully implemented - would query MCP server");
        Ok(None)
    }

    async fn list_tools(&self) -> Result<Vec<ToolMetadata>> {
        // This would query the MCP server for available tools
        // For now, return empty list (would be implemented with actual MCP server communication)
        warn!("list_tools not fully implemented - would query MCP server");
        Ok(Vec::new())
    }
}

impl MCPToolExecutor {
    fn validate_parameter_value(&self, param_meta: &ParameterMetadata, value: &serde_json::Value) -> Result<()> {
        // Basic type validation based on parameter type
        match param_meta.type_.as_str() {
            "string" => {
                if !value.is_string() {
                    return Err(Error::ParameterValidationError("Expected string".to_string()));
                }
            }
            "number" => {
                if !value.is_number() {
                    return Err(Error::ParameterValidationError("Expected number".to_string()));
                }
            }
            "boolean" => {
                if !value.is_boolean() {
                    return Err(Error::ParameterValidationError("Expected boolean".to_string()));
                }
            }
            "object" => {
                if !value.is_object() {
                    return Err(Error::ParameterValidationError("Expected object".to_string()));
                }
            }
            "array" => {
                if !value.is_array() {
                    return Err(Error::ParameterValidationError("Expected array".to_string()));
                }
            }
            _ => {
                // Unknown type, accept any value
            }
        }

        Ok(())
    }
}

/// Tool execution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionStats {
    pub tool_name: String,
    pub total_calls: u64,
    pub successful_calls: u64,
    pub failed_calls: u64,
    pub total_execution_time_ms: u64,
    pub average_execution_time_ms: f64,
    pub last_execution: Option<SystemTime>,
}

impl ToolExecutionStats {
    /// Calculate success rate as a percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_calls == 0 {
            0.0
        } else {
            (self.successful_calls as f64 / self.total_calls as f64) * 100.0
        }
    }

    /// Get failure rate as a percentage
    pub fn failure_rate(&self) -> f64 {
        100.0 - self.success_rate()
    }
}

/// Tool result processor for handling execution results
pub struct ToolResultProcessor;

impl ToolResultProcessor {
    /// Process a tool execution result
    pub fn process_result(result: &ToolExecutionResult) -> Result<ProcessedToolResult> {
        if result.success {
            Self::process_successful_result(result)
        } else {
            Self::process_failed_result(result)
        }
    }

    fn process_successful_result(result: &ToolExecutionResult) -> Result<ProcessedToolResult> {
        // Validate result format and extract useful information
        let processed_data = if let Some(ref data) = result.result {
            Self::extract_result_data(data)?
        } else {
            ProcessedData::Empty
        };

        Ok(ProcessedToolResult {
            tool_name: result.tool_name.clone(),
            status: ExecutionStatus::Success,
            data: processed_data,
            execution_time_ms: result.execution_time_ms,
            metadata: result.metadata.clone(),
        })
    }

    fn process_failed_result(result: &ToolExecutionResult) -> Result<ProcessedToolResult> {
        let error_info = if let Some(ref error) = result.error {
            Self::categorize_error(error)
        } else {
            ErrorCategory::Unknown
        };

        Ok(ProcessedToolResult {
            tool_name: result.tool_name.clone(),
            status: ExecutionStatus::Failed(error_info),
            data: ProcessedData::Empty,
            execution_time_ms: result.execution_time_ms,
            metadata: result.metadata.clone(),
        })
    }

    fn extract_result_data(value: &serde_json::Value) -> Result<ProcessedData> {
        match value {
            serde_json::Value::String(s) => Ok(ProcessedData::Text(s.clone())),
            serde_json::Value::Number(n) => Ok(ProcessedData::Number(n.as_f64().unwrap_or(0.0))),
            serde_json::Value::Bool(b) => Ok(ProcessedData::Boolean(*b)),
            serde_json::Value::Array(arr) => Ok(ProcessedData::Array(arr.len())),
            serde_json::Value::Object(obj) => Ok(ProcessedData::Object(obj.len())),
            serde_json::Value::Null => Ok(ProcessedData::Empty),
        }
    }

    fn categorize_error(error: &ToolError) -> ErrorCategory {
        match error.error_type.as_str() {
            "permission_denied" => ErrorCategory::PermissionDenied,
            "parameter_validation" => ErrorCategory::InvalidParameters,
            "timeout" => ErrorCategory::Timeout,
            "execution_error" => ErrorCategory::ExecutionFailed,
            "not_found" => ErrorCategory::ToolNotFound,
            _ => ErrorCategory::Unknown,
        }
    }
}

/// Processed tool result
#[derive(Debug, Clone)]
pub struct ProcessedToolResult {
    pub tool_name: String,
    pub status: ExecutionStatus,
    pub data: ProcessedData,
    pub execution_time_ms: u64,
    pub metadata: HashMap<String, String>,
}

/// Execution status
#[derive(Debug, Clone)]
pub enum ExecutionStatus {
    Success,
    Failed(ErrorCategory),
}

/// Error categories
#[derive(Debug, Clone)]
pub enum ErrorCategory {
    PermissionDenied,
    InvalidParameters,
    Timeout,
    ExecutionFailed,
    ToolNotFound,
    Unknown,
}

/// Processed result data
#[derive(Debug, Clone)]
pub enum ProcessedData {
    Text(String),
    Number(f64),
    Boolean(bool),
    Array(usize), // Length
    Object(usize), // Number of fields
    Empty,
}