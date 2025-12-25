//! Tool definition API (OpenCode-compatible)
//!
//! Provides Tool trait and define() helper matching OpenCode's Tool.Info interface.

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use crate::context::ToolContext;
use crate::error::ToolError;
use crate::result::FileAttachment;

/// Tool parameters schema
pub type ToolParameters = HashMap<String, ParameterSchema>;

/// Parameter schema definition
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ParameterSchema {
    /// Parameter type (string, number, boolean, object, array)
    #[serde(rename = "type")]
    pub type_: String,
    
    /// Human-readable description
    pub description: String,
    
    /// Whether this parameter is required
    #[serde(default)]
    pub required: bool,
    
    /// Default value if not provided
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
    
    /// For object types: nested schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<Box<ToolParameters>>,
    
    /// For array types: item schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<ParameterSchema>>,
}

/// Tool execution result (OpenCode-compatible)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolExecutionResult {
    /// Human-readable title
    pub title: String,
    
    /// Structured metadata about execution
    pub metadata: HashMap<String, Value>,
    
    /// Tool output (text/json/etc)
    pub output: String,
    
    /// Optional file attachments
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub attachments: Vec<FileAttachment>,
}

/// Tool trait (OpenCode Tool.Info equivalent)
///
/// Matches OpenCode pattern:
/// ```typescript
/// interface Info {
///   id: string
///   init(ctx?) => Promise<{
///     description: string
///     parameters: Parameters
///     execute(args, ctx) => Promise<{ title, metadata, output, attachments? }>
///     formatValidationError?(error) => string
///   }>
/// }
/// ```
#[async_trait]
pub trait Tool: Send + Sync {
    /// Tool ID (unique identifier)
    fn id(&self) -> &str;

    /// Initialize the tool (OpenCode-compatible init())
    ///
    /// Returns tool definition with description, parameters, and execute function.
    /// The execute function is handled by the execute() method below.
    async fn init(&self, ctx: Option<&ToolContext>) -> Result<ToolDefinition, ToolError>;

    /// Execute the tool (called by the wrapper after validation)
    async fn execute(
        &self,
        args: HashMap<String, Value>,
        ctx: &ToolContext,
    ) -> Result<ToolExecutionResult, ToolError>;
}

/// Tool definition returned from init()
#[derive(Clone)]
pub struct ToolDefinition {
    /// Tool description
    pub description: String,
    
    /// Parameter schemas
    pub parameters: ToolParameters,
    
    /// Optional custom validation error formatter
    pub format_validation_error: Option<Arc<dyn Fn(&str) -> String + Send + Sync>>,
}

impl fmt::Debug for ToolDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ToolDefinition")
            .field("description", &self.description)
            .field("parameters", &self.parameters)
            .field("has_format_validation_error", &self.format_validation_error.is_some())
            .finish()
    }
}

/// Tool definition helper (OpenCode-compatible)
///
/// Wraps a tool with:
/// 1. Parameter validation before execute
/// 2. Custom or standard error formatting
/// 3. Matching OpenCode define() signature
pub struct ToolWrapper {
    tool: Arc<dyn Tool>,
}

impl ToolWrapper {
    /// Create a new tool wrapper (OpenCode Tool.define equivalent)
    pub fn new(tool: Arc<dyn Tool>) -> Self {
        Self { tool }
    }

    /// Execute tool with automatic validation
    pub async fn execute_with_validation(
        &self,
        args: HashMap<String, Value>,
        ctx: &ToolContext,
    ) -> Result<ToolExecutionResult, ToolError> {
        // Get tool definition
        let def = self.tool.init(Some(ctx)).await?;

        // Validate parameters
        if let Err(errors) = validate_parameters(&args, &def.parameters) {
            let error_msg = if let Some(formatter) = &def.format_validation_error {
                formatter(&errors)
            } else {
                format!(
                    "The {} tool was called with invalid arguments: {}.\nPlease rewrite the input so it satisfies the expected schema.",
                    self.tool.id(),
                    errors
                )
            };
            return Err(ToolError::new("VALIDATION_ERROR", error_msg));
        }

        // Execute tool
        self.tool.execute(args, ctx).await
    }

    /// Get the underlying tool
    pub fn tool(&self) -> &Arc<dyn Tool> {
        &self.tool
    }
}

/// Validate parameters against schema
fn validate_parameters(
    args: &HashMap<String, Value>,
    schema: &ToolParameters,
) -> Result<(), String> {
    let mut errors = Vec::new();

    // Check required parameters
    for (name, param) in schema {
        if param.required && !args.contains_key(name) {
            errors.push(format!("Missing required parameter: {}", name));
        }
    }

    // Check parameter types (basic validation)
    for (name, value) in args {
        if let Some(param) = schema.get(name) {
            if !validate_type(value, &param.type_) {
                errors.push(format!(
                    "Parameter '{}' has wrong type (expected {}, got {})",
                    name,
                    param.type_,
                    value_type_name(value)
                ));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

/// Validate a value against a type string
fn validate_type(value: &Value, type_: &str) -> bool {
    match type_ {
        "string" => value.is_string(),
        "number" => value.is_number(),
        "boolean" => value.is_boolean(),
        "object" => value.is_object(),
        "array" => value.is_array(),
        "null" => value.is_null(),
        _ => true, // Unknown types pass validation
    }
}

/// Get type name of a JSON value
fn value_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockTool {
        id: String,
    }

    #[async_trait]
    impl Tool for MockTool {
        fn id(&self) -> &str {
            &self.id
        }

        async fn init(&self, _ctx: Option<&ToolContext>) -> Result<ToolDefinition, ToolError> {
            let mut parameters = HashMap::new();
            parameters.insert(
                "name".to_string(),
                ParameterSchema {
                    type_: "string".to_string(),
                    description: "Name parameter".to_string(),
                    required: true,
                    default: None,
                    properties: None,
                    items: None,
                },
            );

            Ok(ToolDefinition {
                description: "Mock tool for testing".to_string(),
                parameters,
                format_validation_error: None,
            })
        }

        async fn execute(
            &self,
            args: HashMap<String, Value>,
            _ctx: &ToolContext,
        ) -> Result<ToolExecutionResult, ToolError> {
            Ok(ToolExecutionResult {
                title: "Mock Result".to_string(),
                metadata: HashMap::new(),
                output: format!("Executed with: {:?}", args),
                attachments: Vec::new(),
            })
        }
    }

    #[tokio::test]
    async fn test_tool_wrapper_validation() {
        let tool = Arc::new(MockTool {
            id: "mock".to_string(),
        });
        let wrapper = ToolWrapper::new(tool);
        let ctx = ToolContext::default();

        // Valid arguments
        let mut args = HashMap::new();
        args.insert("name".to_string(), Value::String("test".to_string()));

        let result = wrapper.execute_with_validation(args, &ctx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_tool_wrapper_validation_missing_required() {
        let tool = Arc::new(MockTool {
            id: "mock".to_string(),
        });
        let wrapper = ToolWrapper::new(tool);
        let ctx = ToolContext::default();

        // Missing required parameter
        let args = HashMap::new();

        let result = wrapper.execute_with_validation(args, &ctx).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_tool_wrapper_validation_wrong_type() {
        let tool = Arc::new(MockTool {
            id: "mock".to_string(),
        });
        let wrapper = ToolWrapper::new(tool);
        let ctx = ToolContext::default();

        // Wrong type (number instead of string)
        let mut args = HashMap::new();
        args.insert("name".to_string(), Value::Number(123.into()));

        let result = wrapper.execute_with_validation(args, &ctx).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_type() {
        assert!(validate_type(&Value::String("test".to_string()), "string"));
        assert!(validate_type(&Value::Number(123.into()), "number"));
        assert!(validate_type(&Value::Bool(true), "boolean"));
        assert!(validate_type(&Value::Array(vec![]), "array"));
        assert!(validate_type(&Value::Object(serde_json::Map::new()), "object"));
        assert!(validate_type(&Value::Null, "null"));
    }

    #[test]
    fn test_validate_type_mismatch() {
        assert!(!validate_type(&Value::String("test".to_string()), "number"));
        assert!(!validate_type(&Value::Number(123.into()), "string"));
        assert!(!validate_type(&Value::Bool(true), "string"));
    }
}
