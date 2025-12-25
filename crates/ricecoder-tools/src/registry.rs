//! Tool Registry - Enhanced provider registry with replace/override, filtering, and plugin interop
//!
//! **GAP-5, GAP-6, GAP-7, GAP-8, GAP-9 Implementation**
//!
//! Extends ProviderRegistry with:
//! - replace/override methods for runtime tool replacement
//! - Provider-specific tool availability filtering
//! - Agent permission integration via enabled(agent)
//! - Plugin-to-tool conversion via from_plugin()
//! - Surface methods: ids(), tools(), all()

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, warn};

use crate::error::ToolError;
use crate::provider::{Provider, ProviderRegistry};
use crate::tool::{Tool, ToolDefinition, ToolWrapper};

/// Tool metadata for registry operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    /// Tool ID
    pub id: String,
    /// Tool description
    pub description: String,
    /// Provider ID (e.g., "mcp", "builtin", "openai")
    pub provider_id: String,
    /// Whether this tool is enabled
    pub enabled: bool,
    /// Required permissions
    pub required_permissions: Vec<String>,
}

/// Plugin-format tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginTool {
    /// Tool name/ID
    pub name: String,
    /// Tool description
    pub description: String,
    /// Input schema (JSON Schema)
    pub input_schema: serde_json::Value,
    /// Optional provider override
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
}

/// Agent permission context
pub trait AgentPermissions: Send + Sync {
    /// Check if agent has permission for tool
    fn has_permission(&self, tool_id: &str) -> bool;
}

/// Enhanced tool registry with replace/override, filtering, and plugin interop
pub struct ToolRegistry {
    /// Underlying provider registry
    provider_registry: Arc<ProviderRegistry>,
    /// Tool metadata by tool ID
    tool_metadata: Arc<RwLock<HashMap<String, ToolMetadata>>>,
    /// Provider filter (None = all providers allowed)
    provider_filter: Arc<RwLock<Option<HashSet<String>>>>,
    /// Registered tool wrappers
    tools: Arc<RwLock<HashMap<String, Arc<dyn Tool>>>>,
}

impl ToolRegistry {
    /// Create a new tool registry
    pub fn new(provider_registry: Arc<ProviderRegistry>) -> Self {
        Self {
            provider_registry,
            tool_metadata: Arc::new(RwLock::new(HashMap::new())),
            provider_filter: Arc::new(RwLock::new(None)),
            tools: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// **GAP-5**: Replace a tool at runtime
    ///
    /// Replaces an existing tool with a new implementation, preserving permissions.
    /// Returns the old tool if it existed.
    pub async fn replace(
        &self,
        tool_id: impl Into<String>,
        tool: Arc<dyn Tool>,
    ) -> Option<Arc<dyn Tool>> {
        let tool_id = tool_id.into();
        debug!("Replacing tool: {}", tool_id);

        // Replace in tools registry
        let old_tool = self.tools.write().await.insert(tool_id.clone(), tool.clone());

        // Update metadata if tool already existed, otherwise create new
        let mut metadata_lock = self.tool_metadata.write().await;
        if let Some(old_meta) = metadata_lock.get(&tool_id) {
            // Preserve permissions and enabled state
            let new_meta = ToolMetadata {
                id: tool_id.clone(),
                description: "Replaced tool".to_string(), // Will be updated on next init()
                provider_id: old_meta.provider_id.clone(),
                enabled: old_meta.enabled,
                required_permissions: old_meta.required_permissions.clone(),
            };
            metadata_lock.insert(tool_id, new_meta);
        }

        old_tool
    }

    /// **GAP-5**: Override a tool with fallback behavior
    ///
    /// Adds a new tool that takes priority, but keeps the old tool as fallback.
    /// Unlike replace(), this allows graceful degradation if the new tool fails.
    pub async fn override_tool(
        &self,
        tool_id: impl Into<String>,
        new_tool: Arc<dyn Tool>,
        fallback_to_old: bool,
    ) -> Result<(), ToolError> {
        let tool_id = tool_id.into();
        debug!("Overriding tool: {} (fallback={})", tool_id, fallback_to_old);

        if fallback_to_old {
            // Keep old tool in a "fallback" namespace
            if let Some(old_tool) = self.tools.read().await.get(&tool_id) {
                let fallback_id = format!("{}_fallback", tool_id);
                self.tools.write().await.insert(fallback_id, old_tool.clone());
            }
        }

        // Install new tool
        self.tools.write().await.insert(tool_id.clone(), new_tool);
        Ok(())
    }

    /// **GAP-6**: Set provider filter for tool availability
    ///
    /// Only tools from the specified providers will be available.
    /// Pass None to allow all providers.
    pub async fn set_provider_filter(&self, provider_ids: Option<Vec<String>>) {
        let filter = provider_ids.map(|ids| ids.into_iter().collect());
        debug!("Setting provider filter: {:?}", filter);
        *self.provider_filter.write().await = filter;
    }

    /// **GAP-6**: Check if tool is available for given provider
    pub async fn is_tool_available_for_provider(&self, tool_id: &str, provider_id: &str) -> bool {
        let filter = self.provider_filter.read().await;
        
        // If no filter, all tools available
        if filter.is_none() {
            return true;
        }

        // Check if tool's provider matches filter
        if let Some(metadata) = self.tool_metadata.read().await.get(tool_id) {
            if let Some(allowed_providers) = filter.as_ref() {
                return allowed_providers.contains(&metadata.provider_id) 
                    || allowed_providers.contains(provider_id);
            }
        }

        false
    }

    /// **GAP-7**: Check if tool is enabled for agent
    ///
    /// Integrates with Agent.permission system to check tool availability.
    pub async fn enabled<P>(&self, tool_id: &str, agent: &P) -> bool
    where
        P: AgentPermissions,
    {
        // Check if tool exists
        if !self.tools.read().await.contains_key(tool_id) {
            return false;
        }

        // Check metadata enabled flag
        if let Some(metadata) = self.tool_metadata.read().await.get(tool_id) {
            if !metadata.enabled {
                return false;
            }
        }

        // Check agent permissions
        agent.has_permission(tool_id)
    }

    /// **GAP-8**: Convert plugin-format tool to RiceCoder tool
    ///
    /// Supports loading tools from external plugin systems (MCP, etc).
    pub async fn from_plugin(&self, plugin_tool: PluginTool) -> Result<Arc<dyn Tool>, ToolError> {
        debug!("Converting plugin tool: {}", plugin_tool.name);

        // Create adapter that wraps plugin tool
        let adapter = PluginToolAdapter {
            id: plugin_tool.name.clone(),
            description: plugin_tool.description,
            input_schema: plugin_tool.input_schema,
            provider_id: plugin_tool.provider.unwrap_or_else(|| "plugin".to_string()),
        };

        let tool: Arc<dyn Tool> = Arc::new(adapter);
        
        // Register in tools registry
        self.tools.write().await.insert(plugin_tool.name.clone(), tool.clone());

        // Create metadata
        let metadata = ToolMetadata {
            id: plugin_tool.name,
            description: tool.id().to_string(),
            provider_id: "plugin".to_string(),
            enabled: true,
            required_permissions: Vec::new(),
        };
        self.tool_metadata.write().await.insert(metadata.id.clone(), metadata);

        Ok(tool)
    }

    /// **GAP-9**: Get all tool IDs
    pub async fn ids(&self) -> Vec<String> {
        self.tools.read().await.keys().cloned().collect()
    }

    /// **GAP-9**: Get all tool wrappers
    pub async fn tools(&self) -> Vec<ToolWrapper> {
        let tools = self.tools.read().await;
        tools
            .values()
            .map(|tool| ToolWrapper::new(tool.clone()))
            .collect()
    }

    /// **GAP-9**: Get all tool definitions (ID + metadata)
    pub async fn all(&self) -> Vec<(String, ToolMetadata)> {
        let metadata = self.tool_metadata.read().await;
        metadata
            .iter()
            .map(|(id, meta)| (id.clone(), meta.clone()))
            .collect()
    }

    /// Register a tool with metadata
    pub async fn register_tool(
        &self,
        tool: Arc<dyn Tool>,
        metadata: ToolMetadata,
    ) -> Result<(), ToolError> {
        let tool_id = tool.id().to_string();
        debug!("Registering tool: {}", tool_id);

        self.tools.write().await.insert(tool_id.clone(), tool);
        self.tool_metadata.write().await.insert(tool_id, metadata);

        Ok(())
    }

    /// Get a tool by ID
    pub async fn get_tool(&self, tool_id: &str) -> Option<Arc<dyn Tool>> {
        self.tools.read().await.get(tool_id).cloned()
    }

    /// Get tool metadata
    pub async fn get_metadata(&self, tool_id: &str) -> Option<ToolMetadata> {
        self.tool_metadata.read().await.get(tool_id).cloned()
    }
}

/// Plugin tool adapter that implements Tool trait
struct PluginToolAdapter {
    id: String,
    description: String,
    input_schema: serde_json::Value,
    provider_id: String,
}

#[async_trait]
impl Tool for PluginToolAdapter {
    fn id(&self) -> &str {
        &self.id
    }

    async fn init(
        &self,
        _ctx: Option<&crate::context::ToolContext>,
    ) -> Result<ToolDefinition, ToolError> {
        // Convert JSON Schema to ToolParameters
        let parameters = if let Some(props) = self.input_schema.get("properties") {
            if let Some(obj) = props.as_object() {
                obj.iter()
                    .map(|(name, schema)| {
                        let param = crate::tool::ParameterSchema {
                            type_: schema
                                .get("type")
                                .and_then(|t| t.as_str())
                                .unwrap_or("string")
                                .to_string(),
                            description: schema
                                .get("description")
                                .and_then(|d| d.as_str())
                                .unwrap_or("")
                                .to_string(),
                            required: false,
                            default: None,
                            properties: None,
                            items: None,
                        };
                        (name.clone(), param)
                    })
                    .collect()
            } else {
                HashMap::new()
            }
        } else {
            HashMap::new()
        };

        Ok(ToolDefinition {
            description: self.description.clone(),
            parameters,
            format_validation_error: None,
        })
    }

    async fn execute(
        &self,
        _args: HashMap<String, serde_json::Value>,
        _ctx: &crate::context::ToolContext,
    ) -> Result<crate::tool::ToolExecutionResult, ToolError> {
        // Plugin tools should be executed via provider registry
        Err(ToolError::new(
            "PLUGIN_EXECUTION_NOT_SUPPORTED",
            format!("Plugin tool '{}' must be executed via provider registry", self.id),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::ProviderRegistry;
    use crate::tool::Tool;

    struct MockTool {
        id: String,
    }

    #[async_trait]
    impl Tool for MockTool {
        fn id(&self) -> &str {
            &self.id
        }

        async fn init(
            &self,
            _ctx: Option<&crate::context::ToolContext>,
        ) -> Result<ToolDefinition, ToolError> {
            Ok(ToolDefinition {
                description: "Mock tool".to_string(),
                parameters: HashMap::new(),
                format_validation_error: None,
            })
        }

        async fn execute(
            &self,
            _args: HashMap<String, serde_json::Value>,
            _ctx: &crate::context::ToolContext,
        ) -> Result<crate::tool::ToolExecutionResult, ToolError> {
            Ok(crate::tool::ToolExecutionResult {
                title: "Mock".to_string(),
                metadata: HashMap::new(),
                output: "mock output".to_string(),
                attachments: Vec::new(),
            })
        }
    }

    struct MockAgentPermissions {
        allowed_tools: Vec<String>,
    }

    impl AgentPermissions for MockAgentPermissions {
        fn has_permission(&self, tool_id: &str) -> bool {
            self.allowed_tools.contains(&tool_id.to_string())
        }
    }

    #[tokio::test]
    async fn test_replace_tool() {
        let provider_registry = Arc::new(ProviderRegistry::new());
        let registry = ToolRegistry::new(provider_registry);

        let tool1 = Arc::new(MockTool { id: "test".to_string() });
        let tool2 = Arc::new(MockTool { id: "test".to_string() });

        // Register initial tool
        let metadata = ToolMetadata {
            id: "test".to_string(),
            description: "Test tool".to_string(),
            provider_id: "builtin".to_string(),
            enabled: true,
            required_permissions: vec!["read".to_string()],
        };
        registry.register_tool(tool1, metadata).await.unwrap();

        // Replace tool
        let old_tool = registry.replace("test", tool2).await;
        assert!(old_tool.is_some());

        // Verify replacement
        let current_tool = registry.get_tool("test").await.unwrap();
        assert_eq!(current_tool.id(), "test");
    }

    #[tokio::test]
    async fn test_provider_filter() {
        let provider_registry = Arc::new(ProviderRegistry::new());
        let registry = ToolRegistry::new(provider_registry);

        // Set filter for "mcp" provider only
        registry.set_provider_filter(Some(vec!["mcp".to_string()])).await;

        // Register tool with "builtin" provider
        let tool = Arc::new(MockTool { id: "test".to_string() });
        let metadata = ToolMetadata {
            id: "test".to_string(),
            description: "Test tool".to_string(),
            provider_id: "builtin".to_string(),
            enabled: true,
            required_permissions: Vec::new(),
        };
        registry.register_tool(tool, metadata).await.unwrap();

        // Tool should not be available for builtin provider
        assert!(!registry.is_tool_available_for_provider("test", "builtin").await);
        
        // Tool should be available for mcp provider
        assert!(registry.is_tool_available_for_provider("test", "mcp").await);
    }

    #[tokio::test]
    async fn test_enabled_with_permissions() {
        let provider_registry = Arc::new(ProviderRegistry::new());
        let registry = ToolRegistry::new(provider_registry);

        let tool = Arc::new(MockTool { id: "test".to_string() });
        let metadata = ToolMetadata {
            id: "test".to_string(),
            description: "Test tool".to_string(),
            provider_id: "builtin".to_string(),
            enabled: true,
            required_permissions: Vec::new(),
        };
        registry.register_tool(tool, metadata).await.unwrap();

        // Agent with permission
        let agent_allowed = MockAgentPermissions {
            allowed_tools: vec!["test".to_string()],
        };
        assert!(registry.enabled("test", &agent_allowed).await);

        // Agent without permission
        let agent_denied = MockAgentPermissions {
            allowed_tools: Vec::new(),
        };
        assert!(!registry.enabled("test", &agent_denied).await);
    }

    #[tokio::test]
    async fn test_from_plugin() {
        let provider_registry = Arc::new(ProviderRegistry::new());
        let registry = ToolRegistry::new(provider_registry);

        let plugin_tool = PluginTool {
            name: "plugin_test".to_string(),
            description: "Plugin tool".to_string(),
            input_schema: serde_json::json!({
                "properties": {
                    "param1": {
                        "type": "string",
                        "description": "Test parameter"
                    }
                }
            }),
            provider: Some("custom".to_string()),
        };

        let tool = registry.from_plugin(plugin_tool).await.unwrap();
        assert_eq!(tool.id(), "plugin_test");

        // Verify tool is registered
        assert!(registry.get_tool("plugin_test").await.is_some());
    }

    #[tokio::test]
    async fn test_registry_surface_methods() {
        let provider_registry = Arc::new(ProviderRegistry::new());
        let registry = ToolRegistry::new(provider_registry);

        // Register multiple tools
        for i in 1..=3 {
            let tool = Arc::new(MockTool {
                id: format!("tool{}", i),
            });
            let metadata = ToolMetadata {
                id: format!("tool{}", i),
                description: format!("Tool {}", i),
                provider_id: "builtin".to_string(),
                enabled: true,
                required_permissions: Vec::new(),
            };
            registry.register_tool(tool, metadata).await.unwrap();
        }

        // Test ids()
        let ids = registry.ids().await;
        assert_eq!(ids.len(), 3);
        assert!(ids.contains(&"tool1".to_string()));

        // Test tools()
        let tools = registry.tools().await;
        assert_eq!(tools.len(), 3);

        // Test all()
        let all = registry.all().await;
        assert_eq!(all.len(), 3);
    }
}
