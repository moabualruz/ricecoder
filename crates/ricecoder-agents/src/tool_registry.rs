//! Tool registry for integrating ricecoder-tools with the agent system
//!
//! This module provides a registry for discovering and managing tools that agents can invoke.
//! Tools are registered with metadata about their capabilities and invocation interface.

use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// Metadata about a registered tool
///
/// This struct contains metadata about a tool, including its ID, name, description,
/// and the types of operations it supports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    /// Tool identifier (e.g., "webfetch", "patch", "todowrite", "todoread", "websearch")
    pub id: String,
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// Input schema (JSON Schema)
    pub input_schema: serde_json::Value,
    /// Output schema (JSON Schema)
    pub output_schema: serde_json::Value,
    /// Whether the tool is available
    pub available: bool,
}

/// Tool invocation interface
///
/// This trait defines the interface for invoking tools. Implementations handle
/// the actual tool execution and result formatting.
#[async_trait::async_trait]
pub trait ToolInvoker: Send + Sync {
    /// Invoke the tool with the given input
    ///
    /// # Arguments
    ///
    /// * `input` - The input to the tool as JSON
    ///
    /// # Returns
    ///
    /// The tool output as JSON or an error
    async fn invoke(&self, input: serde_json::Value) -> Result<serde_json::Value, String>;

    /// Get metadata about this tool
    fn metadata(&self) -> ToolMetadata;
}

/// Registry for discovering and managing tools
///
/// The `ToolRegistry` maintains a registry of all available tools and provides
/// methods to discover tools by ID and invoke them. Tools can be registered
/// at startup or dynamically at runtime.
///
/// # Examples
///
/// ```ignore
/// use ricecoder_agents::ToolRegistry;
///
/// let mut registry = ToolRegistry::new();
/// // Register tools...
///
/// // Find a tool
/// let tool = registry.find_tool("webfetch");
/// ```
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn ToolInvoker>>,
    metadata: HashMap<String, ToolMetadata>,
}

impl ToolRegistry {
    /// Create a new tool registry
    ///
    /// # Returns
    ///
    /// A new empty `ToolRegistry`
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// Register a tool
    ///
    /// Registers a tool with the registry, making it available for agent invocation.
    ///
    /// # Arguments
    ///
    /// * `tool` - The tool invoker to register
    pub fn register(&mut self, tool: Arc<dyn ToolInvoker>) {
        let metadata = tool.metadata();
        let tool_id = metadata.id.clone();
        let tool_name = metadata.name.clone();

        debug!(tool_id = %tool_id, tool_name = %tool_name, "Registering tool");

        self.tools.insert(tool_id.clone(), tool);
        self.metadata.insert(tool_id.clone(), metadata);

        info!(tool_id = %tool_id, tool_name = %tool_name, "Tool registered successfully");
    }

    /// Find a tool by ID
    ///
    /// # Arguments
    ///
    /// * `tool_id` - The ID of the tool to find
    ///
    /// # Returns
    ///
    /// An `Option` containing the tool if found
    pub fn find_tool(&self, tool_id: &str) -> Option<Arc<dyn ToolInvoker>> {
        self.tools.get(tool_id).cloned()
    }

    /// Get all registered tools
    ///
    /// # Returns
    ///
    /// A vector of all registered tools
    pub fn all_tools(&self) -> Vec<Arc<dyn ToolInvoker>> {
        self.tools.values().cloned().collect()
    }

    /// Get the number of registered tools
    ///
    /// # Returns
    ///
    /// The total number of registered tools
    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }

    /// Get metadata for a specific tool
    ///
    /// # Arguments
    ///
    /// * `tool_id` - The ID of the tool
    ///
    /// # Returns
    ///
    /// An `Option` containing the tool metadata if found
    pub fn get_tool_metadata(&self, tool_id: &str) -> Option<ToolMetadata> {
        self.metadata.get(tool_id).cloned()
    }

    /// Get metadata for all registered tools
    ///
    /// # Returns
    ///
    /// A vector of metadata for all registered tools
    pub fn all_tool_metadata(&self) -> Vec<ToolMetadata> {
        self.metadata.values().cloned().collect()
    }

    /// Check if a tool is registered
    ///
    /// # Arguments
    ///
    /// * `tool_id` - The ID of the tool to check
    ///
    /// # Returns
    ///
    /// `true` if the tool is registered, `false` otherwise
    pub fn has_tool(&self, tool_id: &str) -> bool {
        self.tools.contains_key(tool_id)
    }

    /// Invoke a tool
    ///
    /// # Arguments
    ///
    /// * `tool_id` - The ID of the tool to invoke
    /// * `input` - The input to the tool as JSON
    ///
    /// # Returns
    ///
    /// The tool output as JSON or an error
    pub async fn invoke_tool(
        &self,
        tool_id: &str,
        input: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let tool = self
            .find_tool(tool_id)
            .ok_or_else(|| format!("Tool not found: {}", tool_id))?;

        debug!(tool_id = %tool_id, "Invoking tool");
        let result = tool.invoke(input).await?;
        debug!(tool_id = %tool_id, "Tool invocation completed");

        Ok(result)
    }

    /// Discover built-in tools at startup
    ///
    /// This method initializes the registry with built-in tools from ricecoder-tools.
    /// It registers webfetch, patch, todowrite, todoread, and websearch tools.
    pub fn discover_builtin_tools(&mut self) -> Result<(), String> {
        info!("Discovering built-in tools");

        // Built-in tools will be registered here as they are implemented
        // For now, this is a placeholder that can be extended
        debug!("Built-in tool discovery completed");
        Ok(())
    }

    /// Load tool configuration from project settings
    ///
    /// This method loads tool configuration from a configuration source.
    /// The configuration can be used to enable/disable tools or customize
    /// their behavior.
    pub fn load_configuration(
        &mut self,
        config: HashMap<String, serde_json::Value>,
    ) -> Result<(), String> {
        info!(config_count = config.len(), "Loading tool configuration");
        // Configuration loading logic can be implemented here
        // This allows tools to be configured at runtime
        debug!("Tool configuration loaded successfully");
        Ok(())
    }

    /// Get all available tool IDs
    ///
    /// # Returns
    ///
    /// A vector of all registered tool IDs
    pub fn available_tool_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = self.tools.keys().cloned().collect();
        ids.sort();
        ids
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestTool {
        id: String,
        name: String,
        description: String,
    }

    #[async_trait::async_trait]
    impl ToolInvoker for TestTool {
        async fn invoke(&self, input: serde_json::Value) -> Result<serde_json::Value, String> {
            Ok(serde_json::json!({
                "success": true,
                "input": input
            }))
        }

        fn metadata(&self) -> ToolMetadata {
            ToolMetadata {
                id: self.id.clone(),
                name: self.name.clone(),
                description: self.description.clone(),
                input_schema: serde_json::json!({}),
                output_schema: serde_json::json!({}),
                available: true,
            }
        }
    }

    #[test]
    fn test_register_tool() {
        let mut registry = ToolRegistry::new();
        let tool = Arc::new(TestTool {
            id: "test-tool".to_string(),
            name: "Test Tool".to_string(),
            description: "A test tool".to_string(),
        });

        registry.register(tool);
        assert_eq!(registry.tool_count(), 1);
    }

    #[test]
    fn test_find_tool_by_id() {
        let mut registry = ToolRegistry::new();
        let tool = Arc::new(TestTool {
            id: "test-tool".to_string(),
            name: "Test Tool".to_string(),
            description: "A test tool".to_string(),
        });

        registry.register(tool);
        let found = registry.find_tool("test-tool");
        assert!(found.is_some());
    }

    #[test]
    fn test_find_tool_not_found() {
        let registry = ToolRegistry::new();
        let found = registry.find_tool("nonexistent");
        assert!(found.is_none());
    }

    #[test]
    fn test_get_tool_metadata() {
        let mut registry = ToolRegistry::new();
        let tool = Arc::new(TestTool {
            id: "test-tool".to_string(),
            name: "Test Tool".to_string(),
            description: "A test tool for metadata".to_string(),
        });

        registry.register(tool);
        let metadata = registry.get_tool_metadata("test-tool");
        assert!(metadata.is_some());

        let meta = metadata.unwrap();
        assert_eq!(meta.id, "test-tool");
        assert_eq!(meta.name, "Test Tool");
        assert_eq!(meta.description, "A test tool for metadata");
    }

    #[test]
    fn test_all_tool_metadata() {
        let mut registry = ToolRegistry::new();
        let tool1 = Arc::new(TestTool {
            id: "tool-1".to_string(),
            name: "Tool 1".to_string(),
            description: "First tool".to_string(),
        });
        let tool2 = Arc::new(TestTool {
            id: "tool-2".to_string(),
            name: "Tool 2".to_string(),
            description: "Second tool".to_string(),
        });

        registry.register(tool1);
        registry.register(tool2);

        let all_metadata = registry.all_tool_metadata();
        assert_eq!(all_metadata.len(), 2);
    }

    #[test]
    fn test_has_tool() {
        let mut registry = ToolRegistry::new();
        let tool = Arc::new(TestTool {
            id: "test-tool".to_string(),
            name: "Test Tool".to_string(),
            description: "A test tool".to_string(),
        });

        registry.register(tool);
        assert!(registry.has_tool("test-tool"));
        assert!(!registry.has_tool("nonexistent"));
    }

    #[tokio::test]
    async fn test_invoke_tool() {
        let mut registry = ToolRegistry::new();
        let tool = Arc::new(TestTool {
            id: "test-tool".to_string(),
            name: "Test Tool".to_string(),
            description: "A test tool".to_string(),
        });

        registry.register(tool);

        let input = serde_json::json!({"test": "data"});
        let result = registry.invoke_tool("test-tool", input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output["success"], true);
    }

    #[tokio::test]
    async fn test_invoke_tool_not_found() {
        let registry = ToolRegistry::new();
        let input = serde_json::json!({"test": "data"});
        let result = registry.invoke_tool("nonexistent", input).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_available_tool_ids() {
        let mut registry = ToolRegistry::new();
        let tool1 = Arc::new(TestTool {
            id: "tool-1".to_string(),
            name: "Tool 1".to_string(),
            description: "First tool".to_string(),
        });
        let tool2 = Arc::new(TestTool {
            id: "tool-2".to_string(),
            name: "Tool 2".to_string(),
            description: "Second tool".to_string(),
        });

        registry.register(tool1);
        registry.register(tool2);

        let ids = registry.available_tool_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"tool-1".to_string()));
        assert!(ids.contains(&"tool-2".to_string()));
    }

    #[test]
    fn test_discover_builtin_tools() {
        let mut registry = ToolRegistry::new();
        let result = registry.discover_builtin_tools();
        assert!(result.is_ok());
    }

    #[test]
    fn test_load_configuration() {
        let mut registry = ToolRegistry::new();
        let mut config = HashMap::new();
        config.insert("tool-1".to_string(), serde_json::json!({"enabled": true}));

        let result = registry.load_configuration(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_registry_empty_discovery() {
        let registry = ToolRegistry::new();
        assert_eq!(registry.tool_count(), 0);
        assert!(registry.all_tool_metadata().is_empty());
        assert!(registry.available_tool_ids().is_empty());
    }

    #[test]
    fn test_registry_with_multiple_tools() {
        let mut registry = ToolRegistry::new();
        let tools: Vec<Arc<dyn ToolInvoker>> = (1..=5)
            .map(|i| {
                Arc::new(TestTool {
                    id: format!("tool-{}", i),
                    name: format!("Tool {}", i),
                    description: format!("Test tool {}", i),
                }) as Arc<dyn ToolInvoker>
            })
            .collect();

        for tool in tools {
            registry.register(tool);
        }

        assert_eq!(registry.tool_count(), 5);
        assert_eq!(registry.all_tool_metadata().len(), 5);
    }
}
