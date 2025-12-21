//! Industry tool abstractions and registry

use crate::error::{IndustryError, IndustryResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;

/// Industry tool interface
#[async_trait]
pub trait IndustryTool: Send + Sync {
    /// Get tool name
    fn name(&self) -> &str;

    /// Get tool description
    fn description(&self) -> &str;

    /// Get tool capabilities
    fn capabilities(&self) -> Vec<ToolCapability>;

    /// Execute tool with given parameters
    async fn execute(
        &self,
        params: HashMap<String, serde_json::Value>,
    ) -> IndustryResult<serde_json::Value>;

    /// Get tool configuration schema
    fn config_schema(&self) -> serde_json::Value;

    /// Validate tool parameters
    fn validate_params(&self, params: &HashMap<String, serde_json::Value>) -> IndustryResult<()>;
}

/// Tool capability
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ToolCapability {
    /// Code analysis and understanding
    CodeAnalysis,
    /// Code generation and completion
    CodeGeneration,
    /// Code refactoring
    CodeRefactoring,
    /// Testing and validation
    Testing,
    /// Documentation generation
    Documentation,
    /// Version control operations
    VersionControl,
    /// Project management
    ProjectManagement,
    /// CI/CD operations
    CiCd,
    /// Security analysis
    SecurityAnalysis,
    /// Performance analysis
    PerformanceAnalysis,
    /// Custom capability
    Custom(String),
}

/// Tool metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    /// Tool name
    pub name: String,
    /// Tool version
    pub version: String,
    /// Tool description
    pub description: String,
    /// Tool author/organization
    pub author: String,
    /// Tool capabilities
    pub capabilities: Vec<ToolCapability>,
    /// Tool configuration requirements
    pub config_requirements: Vec<ConfigRequirement>,
    /// Tool dependencies
    pub dependencies: Vec<String>,
}

/// Configuration requirement for a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigRequirement {
    /// Configuration key
    pub key: String,
    /// Configuration type
    pub value_type: ConfigValueType,
    /// Whether the configuration is required
    pub required: bool,
    /// Default value (if any)
    pub default_value: Option<serde_json::Value>,
    /// Description of the configuration
    pub description: String,
}

/// Configuration value types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigValueType {
    /// String value
    String,
    /// Integer value
    Integer,
    /// Boolean value
    Boolean,
    /// Float value
    Float,
    /// JSON object
    Object,
    /// Array of values
    Array,
}

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionResult {
    /// Execution success
    pub success: bool,
    /// Result data
    pub data: Option<serde_json::Value>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
    /// Execution metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Tool registry for managing industry tools
pub struct ToolRegistry {
    tools: RwLock<HashMap<String, Box<dyn IndustryTool>>>,
    metadata: RwLock<HashMap<String, ToolMetadata>>,
}

impl ToolRegistry {
    /// Create a new tool registry
    pub fn new() -> Self {
        Self {
            tools: RwLock::new(HashMap::new()),
            metadata: RwLock::new(HashMap::new()),
        }
    }

    /// Register a tool
    pub async fn register_tool(
        &self,
        tool: Box<dyn IndustryTool>,
        metadata: ToolMetadata,
    ) -> IndustryResult<()> {
        let name = tool.name().to_string();

        // Validate metadata matches tool
        if metadata.name != name {
            return Err(IndustryError::ConfigError {
                field: "metadata.name".to_string(),
                message: format!(
                    "Metadata name '{}' does not match tool name '{}'",
                    metadata.name, name
                ),
            });
        }

        self.tools.write().await.insert(name.clone(), tool);
        self.metadata.write().await.insert(name, metadata);

        Ok(())
    }

    /// Unregister a tool
    pub async fn unregister_tool(&self, name: &str) -> IndustryResult<()> {
        let tool_removed = self.tools.write().await.remove(name).is_some();
        let metadata_removed = self.metadata.write().await.remove(name).is_some();

        if !tool_removed && !metadata_removed {
            return Err(IndustryError::ConfigError {
                field: "tool_name".to_string(),
                message: format!("Tool '{}' not found in registry", name),
            });
        }

        Ok(())
    }

    /// Get a tool by name (removes it from the registry)
    pub async fn take_tool(&self, name: &str) -> Option<Box<dyn IndustryTool>> {
        self.tools.write().await.remove(name)
    }

    /// Get tool metadata
    pub async fn get_metadata(&self, name: &str) -> Option<ToolMetadata> {
        self.metadata.read().await.get(name).cloned()
    }

    /// List all registered tools
    pub async fn list_tools(&self) -> Vec<String> {
        self.tools.read().await.keys().cloned().collect()
    }

    /// List tools by capability
    pub async fn list_tools_by_capability(&self, capability: &ToolCapability) -> Vec<String> {
        let metadata = self.metadata.read().await;
        metadata
            .iter()
            .filter(|(_, meta)| meta.capabilities.contains(capability))
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Execute a tool
    pub async fn execute_tool(
        &self,
        tool_name: &str,
        params: HashMap<String, serde_json::Value>,
    ) -> IndustryResult<ToolExecutionResult> {
        let tools = self.tools.read().await;
        let tool = tools
            .get(tool_name)
            .ok_or_else(|| IndustryError::ConfigError {
                field: "tool_name".to_string(),
                message: format!("Tool '{}' not found", tool_name),
            })?;

        // Validate parameters
        tool.validate_params(&params)?;

        let start_time = std::time::Instant::now();

        // Execute the tool
        let result = tool.execute(params).await;
        let duration_ms = start_time.elapsed().as_millis() as u64;

        match result {
            Ok(data) => Ok(ToolExecutionResult {
                success: true,
                data: Some(data),
                error: None,
                duration_ms,
                metadata: HashMap::new(),
            }),
            Err(e) => Ok(ToolExecutionResult {
                success: false,
                data: None,
                error: Some(e.to_string()),
                duration_ms,
                metadata: HashMap::new(),
            }),
        }
    }

    /// Validate tool configuration
    pub async fn validate_tool_config(
        &self,
        tool_name: &str,
        config: &HashMap<String, serde_json::Value>,
    ) -> IndustryResult<Vec<ConfigValidationError>> {
        let metadata = self.metadata.read().await;
        let tool_meta = metadata
            .get(tool_name)
            .ok_or_else(|| IndustryError::ConfigError {
                field: "tool_name".to_string(),
                message: format!("Tool '{}' not found", tool_name),
            })?;

        let mut errors = Vec::new();

        // Check required configurations
        for req in &tool_meta.config_requirements {
            if req.required {
                if !config.contains_key(&req.key) {
                    errors.push(ConfigValidationError {
                        field: req.key.clone(),
                        error_type: ValidationErrorType::MissingRequired,
                        message: format!("Required configuration '{}' is missing", req.key),
                    });
                    continue;
                }
            }

            // Check value type if present
            if let Some(value) = config.get(&req.key) {
                if let Some(error) = Self::validate_value_type(&req.key, value, &req.value_type) {
                    errors.push(error);
                }
            }
        }

        Ok(errors)
    }

    /// Validate a single configuration value type
    fn validate_value_type(
        field: &str,
        value: &serde_json::Value,
        expected_type: &ConfigValueType,
    ) -> Option<ConfigValidationError> {
        let type_matches = match expected_type {
            ConfigValueType::String => value.is_string(),
            ConfigValueType::Integer => value.is_i64() || value.is_u64(),
            ConfigValueType::Boolean => value.is_boolean(),
            ConfigValueType::Float => value.is_f64() || value.is_i64() || value.is_u64(),
            ConfigValueType::Object => value.is_object(),
            ConfigValueType::Array => value.is_array(),
        };

        if !type_matches {
            Some(ConfigValidationError {
                field: field.to_string(),
                error_type: ValidationErrorType::InvalidType,
                message: format!(
                    "Field '{}' has invalid type. Expected {:?}, got {}",
                    field,
                    expected_type,
                    Self::get_value_type_name(value)
                ),
            })
        } else {
            None
        }
    }

    /// Get human-readable type name for a JSON value
    fn get_value_type_name(value: &serde_json::Value) -> &'static str {
        if value.is_string() {
            "string"
        } else if value.is_i64() || value.is_u64() {
            "integer"
        } else if value.is_f64() {
            "float"
        } else if value.is_boolean() {
            "boolean"
        } else if value.is_object() {
            "object"
        } else if value.is_array() {
            "array"
        } else if value.is_null() {
            "null"
        } else {
            "unknown"
        }
    }
}

/// Configuration validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigValidationError {
    /// Field that failed validation
    pub field: String,
    /// Type of validation error
    pub error_type: ValidationErrorType,
    /// Error message
    pub message: String,
}

/// Types of validation errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationErrorType {
    /// Required field is missing
    MissingRequired,
    /// Field has invalid type
    InvalidType,
    /// Field value is invalid
    InvalidValue,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
