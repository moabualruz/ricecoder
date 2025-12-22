//! Custom Tool Executor component

use std::{collections::HashMap, sync::Arc};

use serde_json::{json, Value};
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::{
    config::CustomToolConfig,
    error::{Error, Result},
};

/// Custom Tool Executor for executing custom tools defined in configuration
#[derive(Debug, Clone)]
pub struct CustomToolExecutor {
    tools: Arc<RwLock<HashMap<String, CustomToolConfig>>>,
}

impl CustomToolExecutor {
    /// Creates a new custom tool executor
    pub fn new() -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Registers a custom tool from configuration
    ///
    /// # Arguments
    /// * `tool` - The custom tool configuration to register
    pub async fn register_tool(&self, tool: CustomToolConfig) -> Result<()> {
        debug!("Registering custom tool: {}", tool.id);

        // Validate tool configuration
        if tool.id.is_empty() {
            return Err(Error::ValidationError(
                "Custom tool ID cannot be empty".to_string(),
            ));
        }

        if tool.handler.is_empty() {
            return Err(Error::ValidationError(format!(
                "Custom tool '{}' has no handler",
                tool.id
            )));
        }

        let mut tools = self.tools.write().await;
        tools.insert(tool.id.clone(), tool.clone());

        info!("Custom tool registered: {}", tool.id);
        Ok(())
    }

    /// Registers multiple custom tools
    pub async fn register_tools(&self, tools: Vec<CustomToolConfig>) -> Result<()> {
        for tool in tools {
            self.register_tool(tool).await?;
        }
        Ok(())
    }

    /// Unregisters a custom tool
    pub async fn unregister_tool(&self, tool_id: &str) -> Result<()> {
        debug!("Unregistering custom tool: {}", tool_id);

        let mut tools = self.tools.write().await;
        tools.remove(tool_id);

        info!("Custom tool unregistered: {}", tool_id);
        Ok(())
    }

    /// Executes a custom tool with parameter validation
    ///
    /// # Arguments
    /// * `tool_id` - The ID of the tool to execute
    /// * `parameters` - The parameters to pass to the tool
    ///
    /// # Returns
    /// The result of the tool execution
    pub async fn execute_tool(&self, tool_id: &str, parameters: Value) -> Result<Value> {
        debug!("Executing custom tool: {}", tool_id);

        let tools = self.tools.read().await;
        let tool = tools
            .get(tool_id)
            .ok_or_else(|| Error::ToolNotFound(format!("Custom tool not found: {}", tool_id)))?
            .clone();
        drop(tools);

        // Validate parameters
        self.validate_parameters(&tool, &parameters)?;

        // Execute the tool
        let result = self.execute_handler(&tool, parameters).await?;

        // Validate output
        self.validate_output(&tool, &result)?;

        info!("Custom tool executed successfully: {}", tool_id);
        Ok(result)
    }

    /// Validates tool parameters before execution
    fn validate_parameters(&self, tool: &CustomToolConfig, parameters: &Value) -> Result<()> {
        debug!("Validating parameters for tool: {}", tool.id);

        let params_obj = parameters.as_object().ok_or_else(|| {
            Error::ParameterValidationError("Parameters must be a JSON object".to_string())
        })?;

        // Check required parameters
        for param in &tool.parameters {
            if param.required && !params_obj.contains_key(&param.name) {
                return Err(Error::ParameterValidationError(format!(
                    "Required parameter '{}' is missing",
                    param.name
                )));
            }

            // Validate parameter types if present
            if let Some(value) = params_obj.get(&param.name) {
                self.validate_parameter_type(&param.name, &param.type_, value)?;
            }
        }

        Ok(())
    }

    /// Validates a single parameter type
    fn validate_parameter_type(
        &self,
        name: &str,
        expected_type: &str,
        value: &Value,
    ) -> Result<()> {
        let type_matches = match expected_type {
            "string" => value.is_string(),
            "number" => value.is_number(),
            "integer" => value.is_i64() || value.is_u64(),
            "boolean" => value.is_boolean(),
            "array" => value.is_array(),
            "object" => value.is_object(),
            _ => true, // Unknown types are allowed
        };

        if !type_matches {
            return Err(Error::ParameterValidationError(format!(
                "Parameter '{}' has invalid type. Expected: {}, Got: {}",
                name,
                expected_type,
                value.type_str()
            )));
        }

        Ok(())
    }

    /// Validates tool output after execution
    fn validate_output(&self, tool: &CustomToolConfig, output: &Value) -> Result<()> {
        debug!("Validating output for tool: {}", tool.id);

        // Validate output type
        self.validate_parameter_type("output", &tool.return_type, output)?;

        Ok(())
    }

    /// Executes the tool handler
    ///
    /// In a real implementation, this would invoke the actual handler function.
    /// For now, we simulate successful execution.
    async fn execute_handler(&self, tool: &CustomToolConfig, parameters: Value) -> Result<Value> {
        debug!("Executing handler for tool: {}", tool.id);

        // Simulate tool execution
        // In a real implementation, this would:
        // 1. Look up the handler function
        // 2. Call it with the parameters
        // 3. Return the result

        // For now, return a success response
        Ok(json!({
            "success": true,
            "tool_id": tool.id,
            "message": format!("Tool '{}' executed successfully", tool.id),
            "parameters": parameters
        }))
    }

    /// Gets a registered custom tool
    pub async fn get_tool(&self, tool_id: &str) -> Result<CustomToolConfig> {
        let tools = self.tools.read().await;
        tools
            .get(tool_id)
            .cloned()
            .ok_or_else(|| Error::ToolNotFound(format!("Custom tool not found: {}", tool_id)))
    }

    /// Lists all registered custom tools
    pub async fn list_tools(&self) -> Vec<CustomToolConfig> {
        let tools = self.tools.read().await;
        tools.values().cloned().collect()
    }

    /// Gets the count of registered custom tools
    pub async fn tool_count(&self) -> usize {
        let tools = self.tools.read().await;
        tools.len()
    }

    /// Checks if a tool is registered
    pub async fn has_tool(&self, tool_id: &str) -> bool {
        let tools = self.tools.read().await;
        tools.contains_key(tool_id)
    }

    /// Clears all registered custom tools
    pub async fn clear_tools(&self) {
        let mut tools = self.tools.write().await;
        tools.clear();
        info!("All custom tools cleared");
    }
}

impl Default for CustomToolExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension trait for Value to get type string
trait ValueTypeStr {
    fn type_str(&self) -> &'static str;
}

impl ValueTypeStr for Value {
    fn type_str(&self) -> &'static str {
        match self {
            Value::Null => "null",
            Value::Bool(_) => "boolean",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ParameterConfig;

    fn create_test_tool(id: &str) -> CustomToolConfig {
        CustomToolConfig {
            id: id.to_string(),
            name: format!("Test Tool {}", id),
            description: "A test tool".to_string(),
            category: "test".to_string(),
            parameters: vec![
                ParameterConfig {
                    name: "input".to_string(),
                    type_: "string".to_string(),
                    description: "Input parameter".to_string(),
                    required: true,
                    default: None,
                },
                ParameterConfig {
                    name: "count".to_string(),
                    type_: "integer".to_string(),
                    description: "Count parameter".to_string(),
                    required: false,
                    default: Some(json!(1)),
                },
            ],
            return_type: "object".to_string(),
            handler: "test::handler".to_string(),
        }
    }

    #[tokio::test]
    async fn test_create_executor() {
        let executor = CustomToolExecutor::new();
        assert_eq!(executor.tool_count().await, 0);
    }

    #[tokio::test]
    async fn test_register_tool() {
        let executor = CustomToolExecutor::new();
        let tool = create_test_tool("tool1");

        let result = executor.register_tool(tool).await;
        assert!(result.is_ok());
        assert_eq!(executor.tool_count().await, 1);
    }

    #[tokio::test]
    async fn test_register_tool_empty_id() {
        let executor = CustomToolExecutor::new();
        let mut tool = create_test_tool("tool1");
        tool.id = "".to_string();

        let result = executor.register_tool(tool).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_register_tool_empty_handler() {
        let executor = CustomToolExecutor::new();
        let mut tool = create_test_tool("tool1");
        tool.handler = "".to_string();

        let result = executor.register_tool(tool).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_unregister_tool() {
        let executor = CustomToolExecutor::new();
        let tool = create_test_tool("tool1");

        executor.register_tool(tool).await.unwrap();
        assert_eq!(executor.tool_count().await, 1);

        executor.unregister_tool("tool1").await.unwrap();
        assert_eq!(executor.tool_count().await, 0);
    }

    #[tokio::test]
    async fn test_get_tool() {
        let executor = CustomToolExecutor::new();
        let tool = create_test_tool("tool1");

        executor.register_tool(tool.clone()).await.unwrap();
        let retrieved = executor.get_tool("tool1").await.unwrap();
        assert_eq!(retrieved.id, tool.id);
    }

    #[tokio::test]
    async fn test_get_tool_not_found() {
        let executor = CustomToolExecutor::new();
        let result = executor.get_tool("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_tools() {
        let executor = CustomToolExecutor::new();
        executor
            .register_tool(create_test_tool("tool1"))
            .await
            .unwrap();
        executor
            .register_tool(create_test_tool("tool2"))
            .await
            .unwrap();

        let tools = executor.list_tools().await;
        assert_eq!(tools.len(), 2);
    }

    #[tokio::test]
    async fn test_has_tool() {
        let executor = CustomToolExecutor::new();
        executor
            .register_tool(create_test_tool("tool1"))
            .await
            .unwrap();

        assert!(executor.has_tool("tool1").await);
        assert!(!executor.has_tool("tool2").await);
    }

    #[tokio::test]
    async fn test_clear_tools() {
        let executor = CustomToolExecutor::new();
        executor
            .register_tool(create_test_tool("tool1"))
            .await
            .unwrap();
        executor
            .register_tool(create_test_tool("tool2"))
            .await
            .unwrap();

        assert_eq!(executor.tool_count().await, 2);
        executor.clear_tools().await;
        assert_eq!(executor.tool_count().await, 0);
    }

    #[tokio::test]
    async fn test_execute_tool_valid_parameters() {
        let executor = CustomToolExecutor::new();
        let tool = create_test_tool("tool1");
        executor.register_tool(tool).await.unwrap();

        let params = json!({
            "input": "test",
            "count": 5
        });

        let result = executor.execute_tool("tool1", params).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_execute_tool_missing_required_parameter() {
        let executor = CustomToolExecutor::new();
        let tool = create_test_tool("tool1");
        executor.register_tool(tool).await.unwrap();

        let params = json!({
            "count": 5
        });

        let result = executor.execute_tool("tool1", params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_tool_invalid_parameter_type() {
        let executor = CustomToolExecutor::new();
        let tool = create_test_tool("tool1");
        executor.register_tool(tool).await.unwrap();

        let params = json!({
            "input": 123,
            "count": 5
        });

        let result = executor.execute_tool("tool1", params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_tool_not_found() {
        let executor = CustomToolExecutor::new();
        let params = json!({
            "input": "test"
        });

        let result = executor.execute_tool("nonexistent", params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_register_multiple_tools() {
        let executor = CustomToolExecutor::new();
        let tools = vec![
            create_test_tool("tool1"),
            create_test_tool("tool2"),
            create_test_tool("tool3"),
        ];

        let result = executor.register_tools(tools).await;
        assert!(result.is_ok());
        assert_eq!(executor.tool_count().await, 3);
    }

    #[tokio::test]
    async fn test_validate_parameter_types() {
        let executor = CustomToolExecutor::new();

        // Test string validation
        assert!(executor
            .validate_parameter_type("test", "string", &json!("hello"))
            .is_ok());
        assert!(executor
            .validate_parameter_type("test", "string", &json!(123))
            .is_err());

        // Test number validation
        assert!(executor
            .validate_parameter_type("test", "number", &json!(123.45))
            .is_ok());
        assert!(executor
            .validate_parameter_type("test", "number", &json!("hello"))
            .is_err());

        // Test boolean validation
        assert!(executor
            .validate_parameter_type("test", "boolean", &json!(true))
            .is_ok());
        assert!(executor
            .validate_parameter_type("test", "boolean", &json!("hello"))
            .is_err());

        // Test array validation
        assert!(executor
            .validate_parameter_type("test", "array", &json!([1, 2, 3]))
            .is_ok());
        assert!(executor
            .validate_parameter_type("test", "array", &json!("hello"))
            .is_err());

        // Test object validation
        assert!(executor
            .validate_parameter_type("test", "object", &json!({}))
            .is_ok());
        assert!(executor
            .validate_parameter_type("test", "object", &json!("hello"))
            .is_err());
    }
}
