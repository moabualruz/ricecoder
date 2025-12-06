//! Tool metadata management

use serde_json::Value;

/// Tool metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolMetadata {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub parameters: Vec<ParameterMetadata>,
    pub return_type: String,
    pub source: ToolSource,
    pub server_id: Option<String>,
}

/// Parameter metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ParameterMetadata {
    pub name: String,
    pub type_: String,
    pub description: String,
    pub required: bool,
    pub default: Option<Value>,
}

/// Tool source
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ToolSource {
    BuiltIn,
    Custom,
    Mcp(String),
}

impl ToolMetadata {
    /// Creates a new tool metadata
    pub fn new(
        id: String,
        name: String,
        description: String,
        category: String,
        return_type: String,
        source: ToolSource,
    ) -> Self {
        Self {
            id,
            name,
            description,
            category,
            parameters: Vec::new(),
            return_type,
            source,
            server_id: None,
        }
    }

    /// Adds a parameter to the tool
    pub fn add_parameter(&mut self, parameter: ParameterMetadata) {
        self.parameters.push(parameter);
    }

    /// Sets the server ID for MCP tools
    pub fn set_server_id(&mut self, server_id: String) {
        self.server_id = Some(server_id);
    }

    /// Gets the tool documentation
    pub fn get_documentation(&self) -> String {
        let mut doc = format!("# {}\n\n", self.name);
        doc.push_str(&format!("**Description**: {}\n\n", self.description));
        doc.push_str(&format!("**Category**: {}\n\n", self.category));

        if !self.parameters.is_empty() {
            doc.push_str("## Parameters\n\n");
            for param in &self.parameters {
                doc.push_str(&format!(
                    "- **{}** ({}{}): {}\n",
                    param.name,
                    param.type_,
                    if param.required { ", required" } else { "" },
                    param.description
                ));
                if let Some(default) = &param.default {
                    doc.push_str(&format!("  - Default: {}\n", default));
                }
            }
            doc.push('\n');
        }

        doc.push_str(&format!("## Returns\n\n{}\n", self.return_type));

        doc
    }

    /// Validates the tool metadata
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("Tool ID cannot be empty".to_string());
        }

        if self.name.is_empty() {
            return Err("Tool name cannot be empty".to_string());
        }

        if self.description.is_empty() {
            return Err("Tool description cannot be empty".to_string());
        }

        if self.category.is_empty() {
            return Err("Tool category cannot be empty".to_string());
        }

        if self.return_type.is_empty() {
            return Err("Tool return type cannot be empty".to_string());
        }

        // Validate parameters
        for param in &self.parameters {
            if param.name.is_empty() {
                return Err("Parameter name cannot be empty".to_string());
            }

            if param.type_.is_empty() {
                return Err("Parameter type cannot be empty".to_string());
            }

            if param.description.is_empty() {
                return Err("Parameter description cannot be empty".to_string());
            }
        }

        Ok(())
    }
}

impl ParameterMetadata {
    /// Creates a new parameter metadata
    pub fn new(
        name: String,
        type_: String,
        description: String,
        required: bool,
    ) -> Self {
        Self {
            name,
            type_,
            description,
            required,
            default: None,
        }
    }

    /// Sets the default value for the parameter
    pub fn with_default(mut self, default: Value) -> Self {
        self.default = Some(default);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_tool_metadata() {
        let tool = ToolMetadata::new(
            "test-tool".to_string(),
            "Test Tool".to_string(),
            "A test tool".to_string(),
            "test".to_string(),
            "string".to_string(),
            ToolSource::Custom,
        );

        assert_eq!(tool.id, "test-tool");
        assert_eq!(tool.name, "Test Tool");
        assert_eq!(tool.description, "A test tool");
        assert_eq!(tool.category, "test");
        assert_eq!(tool.return_type, "string");
        assert!(tool.parameters.is_empty());
        assert!(tool.server_id.is_none());
    }

    #[test]
    fn test_add_parameter() {
        let mut tool = ToolMetadata::new(
            "test-tool".to_string(),
            "Test Tool".to_string(),
            "A test tool".to_string(),
            "test".to_string(),
            "string".to_string(),
            ToolSource::Custom,
        );

        let param = ParameterMetadata::new(
            "param1".to_string(),
            "string".to_string(),
            "First parameter".to_string(),
            true,
        );

        tool.add_parameter(param);
        assert_eq!(tool.parameters.len(), 1);
        assert_eq!(tool.parameters[0].name, "param1");
    }

    #[test]
    fn test_set_server_id() {
        let mut tool = ToolMetadata::new(
            "test-tool".to_string(),
            "Test Tool".to_string(),
            "A test tool".to_string(),
            "test".to_string(),
            "string".to_string(),
            ToolSource::Mcp("server-1".to_string()),
        );

        tool.set_server_id("server-1".to_string());
        assert_eq!(tool.server_id, Some("server-1".to_string()));
    }

    #[test]
    fn test_get_documentation() {
        let mut tool = ToolMetadata::new(
            "test-tool".to_string(),
            "Test Tool".to_string(),
            "A test tool".to_string(),
            "test".to_string(),
            "string".to_string(),
            ToolSource::Custom,
        );

        let param = ParameterMetadata::new(
            "param1".to_string(),
            "string".to_string(),
            "First parameter".to_string(),
            true,
        );

        tool.add_parameter(param);

        let doc = tool.get_documentation();
        assert!(doc.contains("Test Tool"));
        assert!(doc.contains("A test tool"));
        assert!(doc.contains("param1"));
        assert!(doc.contains("First parameter"));
    }

    #[test]
    fn test_validate_valid_tool() {
        let tool = ToolMetadata::new(
            "test-tool".to_string(),
            "Test Tool".to_string(),
            "A test tool".to_string(),
            "test".to_string(),
            "string".to_string(),
            ToolSource::Custom,
        );

        assert!(tool.validate().is_ok());
    }

    #[test]
    fn test_validate_empty_id() {
        let tool = ToolMetadata::new(
            "".to_string(),
            "Test Tool".to_string(),
            "A test tool".to_string(),
            "test".to_string(),
            "string".to_string(),
            ToolSource::Custom,
        );

        assert!(tool.validate().is_err());
    }

    #[test]
    fn test_parameter_with_default() {
        let param = ParameterMetadata::new(
            "param1".to_string(),
            "string".to_string(),
            "First parameter".to_string(),
            false,
        )
        .with_default(Value::String("default_value".to_string()));

        assert_eq!(param.default, Some(Value::String("default_value".to_string())));
    }
}
