//! Tool Registry for managing available tools

use std::collections::HashMap;

use crate::{error::Result, metadata::ToolMetadata};

/// Tool Registry for managing all available tools
#[derive(Debug, Clone)]
pub struct ToolRegistry {
    tools: HashMap<String, ToolMetadata>,
}

impl ToolRegistry {
    /// Creates a new tool registry
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Registers a tool in the registry
    pub fn register_tool(&mut self, tool: ToolMetadata) -> Result<()> {
        // Check for naming conflicts
        if self.tools.contains_key(&tool.id) {
            return Err(crate::error::Error::NamingConflict(format!(
                "Tool with ID '{}' already exists",
                tool.id
            )));
        }

        self.tools.insert(tool.id.clone(), tool);
        Ok(())
    }

    /// Gets a tool by ID
    pub fn get_tool(&self, id: &str) -> Option<&ToolMetadata> {
        self.tools.get(id)
    }

    /// Lists all tools
    pub fn list_tools(&self) -> Vec<&ToolMetadata> {
        self.tools.values().collect()
    }

    /// Lists tools by category
    pub fn list_tools_by_category(&self, category: &str) -> Vec<&ToolMetadata> {
        self.tools
            .values()
            .filter(|t| t.category == category)
            .collect()
    }

    /// Lists tools by server
    pub fn list_tools_by_server(&self, server_id: &str) -> Vec<&ToolMetadata> {
        self.tools
            .values()
            .filter(|t| t.server_id.as_deref() == Some(server_id))
            .collect()
    }

    /// Gets the number of registered tools
    pub fn tool_count(&self) -> usize {
        self.tools.len()
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
    use crate::metadata::ToolSource;

    #[test]
    fn test_register_tool() {
        let mut registry = ToolRegistry::new();
        let tool = ToolMetadata {
            id: "test-tool".to_string(),
            name: "Test Tool".to_string(),
            description: "A test tool".to_string(),
            category: "test".to_string(),
            parameters: vec![],
            return_type: "string".to_string(),
            source: ToolSource::Custom,
            server_id: None,
        };

        let result = registry.register_tool(tool);
        assert!(result.is_ok());
        assert_eq!(registry.tool_count(), 1);
    }

    #[test]
    fn test_register_duplicate_tool() {
        let mut registry = ToolRegistry::new();
        let tool = ToolMetadata {
            id: "test-tool".to_string(),
            name: "Test Tool".to_string(),
            description: "A test tool".to_string(),
            category: "test".to_string(),
            parameters: vec![],
            return_type: "string".to_string(),
            source: ToolSource::Custom,
            server_id: None,
        };

        registry.register_tool(tool.clone()).unwrap();
        let result = registry.register_tool(tool);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_tool() {
        let mut registry = ToolRegistry::new();
        let tool = ToolMetadata {
            id: "test-tool".to_string(),
            name: "Test Tool".to_string(),
            description: "A test tool".to_string(),
            category: "test".to_string(),
            parameters: vec![],
            return_type: "string".to_string(),
            source: ToolSource::Custom,
            server_id: None,
        };

        registry.register_tool(tool).unwrap();
        let retrieved = registry.get_tool("test-tool");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Test Tool");
    }

    #[test]
    fn test_list_tools_by_category() {
        let mut registry = ToolRegistry::new();
        let tool1 = ToolMetadata {
            id: "tool1".to_string(),
            name: "Tool 1".to_string(),
            description: "Tool 1".to_string(),
            category: "math".to_string(),
            parameters: vec![],
            return_type: "number".to_string(),
            source: ToolSource::Custom,
            server_id: None,
        };

        let tool2 = ToolMetadata {
            id: "tool2".to_string(),
            name: "Tool 2".to_string(),
            description: "Tool 2".to_string(),
            category: "string".to_string(),
            parameters: vec![],
            return_type: "string".to_string(),
            source: ToolSource::Custom,
            server_id: None,
        };

        registry.register_tool(tool1).unwrap();
        registry.register_tool(tool2).unwrap();

        let math_tools = registry.list_tools_by_category("math");
        assert_eq!(math_tools.len(), 1);
        assert_eq!(math_tools[0].id, "tool1");
    }
}
