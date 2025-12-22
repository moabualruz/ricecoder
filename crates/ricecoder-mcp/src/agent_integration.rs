//! Integration with ricecoder-agents framework
//!
//! This module provides integration between the MCP tool system and the ricecoder-agents
//! framework, enabling agents to discover and invoke MCP tools within their workflows.

use std::{collections::HashMap, sync::Arc};

use serde_json::Value;

use crate::{error::Result, metadata::ToolMetadata, registry::ToolRegistry};

/// Tool invocation capability for agents
///
/// This trait allows agents to invoke MCP tools with parameter validation
/// and result handling.
pub trait ToolInvoker: Send + Sync {
    /// Invoke a tool with the given parameters
    ///
    /// # Arguments
    ///
    /// * `tool_id` - The ID of the tool to invoke
    /// * `parameters` - The parameters to pass to the tool
    ///
    /// # Returns
    ///
    /// The result of the tool invocation
    fn invoke_tool(&self, tool_id: &str, parameters: HashMap<String, Value>) -> Result<Value>;
}

/// Tool discovery capability for agents
///
/// This trait allows agents to discover available tools and their capabilities.
pub trait ToolDiscovery: Send + Sync {
    /// Get all available tools
    fn get_all_tools(&self) -> Vec<ToolMetadata>;

    /// Get tools by category
    fn get_tools_by_category(&self, category: &str) -> Vec<ToolMetadata>;

    /// Get tools by server
    fn get_tools_by_server(&self, server_id: &str) -> Vec<ToolMetadata>;

    /// Get a specific tool by ID
    fn get_tool(&self, tool_id: &str) -> Option<ToolMetadata>;

    /// Search for tools by name or description
    fn search_tools(&self, query: &str) -> Vec<ToolMetadata>;
}

/// Agent tool capabilities
///
/// This struct provides agents with access to MCP tools through a unified interface.
/// It combines tool invocation and discovery capabilities.
pub struct AgentToolCapabilities {
    registry: Arc<ToolRegistry>,
    invoker: Arc<dyn ToolInvoker>,
}

impl AgentToolCapabilities {
    /// Creates a new agent tool capabilities instance
    ///
    /// # Arguments
    ///
    /// * `registry` - The tool registry
    /// * `invoker` - The tool invoker
    pub fn new(registry: Arc<ToolRegistry>, invoker: Arc<dyn ToolInvoker>) -> Self {
        Self { registry, invoker }
    }

    /// Invokes a tool with the given parameters
    ///
    /// # Arguments
    ///
    /// * `tool_id` - The ID of the tool to invoke
    /// * `parameters` - The parameters to pass to the tool
    ///
    /// # Returns
    ///
    /// The result of the tool invocation
    pub fn invoke_tool(&self, tool_id: &str, parameters: HashMap<String, Value>) -> Result<Value> {
        self.invoker.invoke_tool(tool_id, parameters)
    }

    /// Gets all available tools
    pub fn get_all_tools(&self) -> Vec<ToolMetadata> {
        self.registry.list_tools().into_iter().cloned().collect()
    }

    /// Gets tools by category
    pub fn get_tools_by_category(&self, category: &str) -> Vec<ToolMetadata> {
        self.registry
            .list_tools_by_category(category)
            .into_iter()
            .cloned()
            .collect()
    }

    /// Gets tools by server
    pub fn get_tools_by_server(&self, server_id: &str) -> Vec<ToolMetadata> {
        self.registry
            .list_tools_by_server(server_id)
            .into_iter()
            .cloned()
            .collect()
    }

    /// Gets a specific tool by ID
    pub fn get_tool(&self, tool_id: &str) -> Option<ToolMetadata> {
        self.registry.get_tool(tool_id).cloned()
    }

    /// Searches for tools by name or description
    pub fn search_tools(&self, query: &str) -> Vec<ToolMetadata> {
        let query_lower = query.to_lowercase();
        self.registry
            .list_tools()
            .into_iter()
            .filter(|tool| {
                tool.name.to_lowercase().contains(&query_lower)
                    || tool.description.to_lowercase().contains(&query_lower)
            })
            .cloned()
            .collect()
    }

    /// Gets the number of available tools
    pub fn tool_count(&self) -> usize {
        self.registry.tool_count()
    }

    /// Gets tool documentation
    pub fn get_tool_documentation(&self, tool_id: &str) -> Option<String> {
        self.registry
            .get_tool(tool_id)
            .map(|tool| tool.get_documentation())
    }
}

/// Tool execution context for agents
///
/// This struct provides context for tool execution within an agent workflow.
pub struct ToolExecutionContext {
    /// The agent ID executing the tool
    pub agent_id: String,
    /// The task ID associated with the tool execution
    pub task_id: String,
    /// Additional metadata about the execution
    pub metadata: HashMap<String, Value>,
}

impl ToolExecutionContext {
    /// Creates a new tool execution context
    pub fn new(agent_id: String, task_id: String) -> Self {
        Self {
            agent_id,
            task_id,
            metadata: HashMap::new(),
        }
    }

    /// Adds metadata to the execution context
    pub fn with_metadata(mut self, key: String, value: Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Tool execution result for agents
///
/// This struct represents the result of a tool execution within an agent workflow.
#[derive(Debug, Clone)]
pub struct ToolExecutionResult {
    /// The tool ID that was executed
    pub tool_id: String,
    /// Whether the execution was successful
    pub success: bool,
    /// The output of the tool execution
    pub output: Value,
    /// Any error message if the execution failed
    pub error: Option<String>,
    /// The duration of the execution in milliseconds
    pub duration_ms: u64,
}

impl ToolExecutionResult {
    /// Creates a successful tool execution result
    pub fn success(tool_id: String, output: Value, duration_ms: u64) -> Self {
        Self {
            tool_id,
            success: true,
            output,
            error: None,
            duration_ms,
        }
    }

    /// Creates a failed tool execution result
    pub fn failure(tool_id: String, error: String, duration_ms: u64) -> Self {
        Self {
            tool_id,
            success: false,
            output: Value::Null,
            error: Some(error),
            duration_ms,
        }
    }
}

/// Tool workflow integration for agents
///
/// This struct provides integration between MCP tools and agent workflows,
/// enabling sequential and parallel tool execution.
pub struct ToolWorkflowIntegration {
    capabilities: Arc<AgentToolCapabilities>,
}

impl ToolWorkflowIntegration {
    /// Creates a new tool workflow integration
    pub fn new(capabilities: Arc<AgentToolCapabilities>) -> Self {
        Self { capabilities }
    }

    /// Executes a single tool in the workflow
    pub async fn execute_tool(
        &self,
        context: &ToolExecutionContext,
        tool_id: &str,
        parameters: HashMap<String, Value>,
    ) -> Result<ToolExecutionResult> {
        let _ = context; // Context available for future use (e.g., logging, permission checks)
        let start = std::time::Instant::now();

        match self.capabilities.invoke_tool(tool_id, parameters) {
            Ok(output) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                Ok(ToolExecutionResult::success(
                    tool_id.to_string(),
                    output,
                    duration_ms,
                ))
            }
            Err(e) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                Ok(ToolExecutionResult::failure(
                    tool_id.to_string(),
                    e.to_string(),
                    duration_ms,
                ))
            }
        }
    }

    /// Executes multiple tools sequentially
    pub async fn execute_tools_sequential(
        &self,
        context: &ToolExecutionContext,
        tools: Vec<(String, HashMap<String, Value>)>,
    ) -> Result<Vec<ToolExecutionResult>> {
        let mut results = Vec::new();

        for (tool_id, parameters) in tools {
            let result = self.execute_tool(context, &tool_id, parameters).await?;
            results.push(result);
        }

        Ok(results)
    }

    /// Executes multiple tools in parallel
    pub async fn execute_tools_parallel(
        &self,
        _context: &ToolExecutionContext,
        tools: Vec<(String, HashMap<String, Value>)>,
    ) -> Result<Vec<ToolExecutionResult>> {
        let mut handles = Vec::new();

        for (tool_id, parameters) in tools {
            let capabilities = self.capabilities.clone();

            let handle = tokio::spawn(async move {
                let start = std::time::Instant::now();

                match capabilities.invoke_tool(&tool_id, parameters) {
                    Ok(output) => {
                        let duration_ms = start.elapsed().as_millis() as u64;
                        ToolExecutionResult::success(tool_id, output, duration_ms)
                    }
                    Err(e) => {
                        let duration_ms = start.elapsed().as_millis() as u64;
                        ToolExecutionResult::failure(tool_id, e.to_string(), duration_ms)
                    }
                }
            });

            handles.push(handle);
        }

        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(result) => results.push(result),
                Err(e) => {
                    return Err(crate::error::Error::ExecutionError(format!(
                        "Tool execution task failed: {}",
                        e
                    )))
                }
            }
        }

        Ok(results)
    }

    /// Gets available tools for the agent
    pub fn get_available_tools(&self) -> Vec<ToolMetadata> {
        self.capabilities.get_all_tools()
    }

    /// Gets tool documentation
    pub fn get_tool_documentation(&self, tool_id: &str) -> Option<String> {
        self.capabilities.get_tool_documentation(tool_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockToolInvoker;

    impl ToolInvoker for MockToolInvoker {
        fn invoke_tool(&self, tool_id: &str, _parameters: HashMap<String, Value>) -> Result<Value> {
            Ok(serde_json::json!({
                "tool_id": tool_id,
                "result": "success"
            }))
        }
    }

    #[test]
    fn test_agent_tool_capabilities_creation() {
        let registry = Arc::new(ToolRegistry::new());
        let invoker: Arc<dyn ToolInvoker> = Arc::new(MockToolInvoker);
        let capabilities = AgentToolCapabilities::new(registry, invoker);

        assert_eq!(capabilities.tool_count(), 0);
    }

    #[test]
    fn test_tool_execution_context_creation() {
        let context = ToolExecutionContext::new("agent-1".to_string(), "task-1".to_string());

        assert_eq!(context.agent_id, "agent-1");
        assert_eq!(context.task_id, "task-1");
        assert!(context.metadata.is_empty());
    }

    #[test]
    fn test_tool_execution_context_with_metadata() {
        let context = ToolExecutionContext::new("agent-1".to_string(), "task-1".to_string())
            .with_metadata("key1".to_string(), serde_json::json!("value1"));

        assert_eq!(context.metadata.len(), 1);
        assert_eq!(
            context.metadata.get("key1"),
            Some(&serde_json::json!("value1"))
        );
    }

    #[test]
    fn test_tool_execution_result_success() {
        let result = ToolExecutionResult::success(
            "tool-1".to_string(),
            serde_json::json!({"result": "success"}),
            100,
        );

        assert_eq!(result.tool_id, "tool-1");
        assert!(result.success);
        assert_eq!(result.duration_ms, 100);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_tool_execution_result_failure() {
        let result = ToolExecutionResult::failure(
            "tool-1".to_string(),
            "Tool execution failed".to_string(),
            100,
        );

        assert_eq!(result.tool_id, "tool-1");
        assert!(!result.success);
        assert_eq!(result.duration_ms, 100);
        assert_eq!(result.error, Some("Tool execution failed".to_string()));
    }

    #[tokio::test]
    async fn test_tool_workflow_integration_execute_tool() {
        let registry = Arc::new(ToolRegistry::new());
        let invoker: Arc<dyn ToolInvoker> = Arc::new(MockToolInvoker);
        let capabilities = Arc::new(AgentToolCapabilities::new(registry, invoker));
        let workflow = ToolWorkflowIntegration::new(capabilities);

        let context = ToolExecutionContext::new("agent-1".to_string(), "task-1".to_string());
        let mut params = HashMap::new();
        params.insert("param1".to_string(), serde_json::json!("value1"));

        let result = workflow
            .execute_tool(&context, "tool-1", params)
            .await
            .unwrap();

        assert_eq!(result.tool_id, "tool-1");
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_tool_workflow_integration_execute_tools_sequential() {
        let registry = Arc::new(ToolRegistry::new());
        let invoker: Arc<dyn ToolInvoker> = Arc::new(MockToolInvoker);
        let capabilities = Arc::new(AgentToolCapabilities::new(registry, invoker));
        let workflow = ToolWorkflowIntegration::new(capabilities);

        let context = ToolExecutionContext::new("agent-1".to_string(), "task-1".to_string());
        let tools = vec![
            ("tool-1".to_string(), HashMap::new()),
            ("tool-2".to_string(), HashMap::new()),
        ];

        let results = workflow
            .execute_tools_sequential(&context, tools)
            .await
            .unwrap();

        assert_eq!(results.len(), 2);
        assert!(results[0].success);
        assert!(results[1].success);
    }

    #[tokio::test]
    async fn test_tool_workflow_integration_execute_tools_parallel() {
        let registry = Arc::new(ToolRegistry::new());
        let invoker: Arc<dyn ToolInvoker> = Arc::new(MockToolInvoker);
        let capabilities = Arc::new(AgentToolCapabilities::new(registry, invoker));
        let workflow = ToolWorkflowIntegration::new(capabilities);

        let context = ToolExecutionContext::new("agent-1".to_string(), "task-1".to_string());
        let tools = vec![
            ("tool-1".to_string(), HashMap::new()),
            ("tool-2".to_string(), HashMap::new()),
        ];

        let results = workflow
            .execute_tools_parallel(&context, tools)
            .await
            .unwrap();

        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.success));
    }
}
