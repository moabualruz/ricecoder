//! Command execution system with parameter handling, validation, and error management
//!
//! This module provides a comprehensive command execution framework that supports:
//! - Parameter prompting and autocomplete
//! - Command validation and error handling
//! - Asynchronous command execution
//! - Command result processing

use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::RwLock;

/// Result type for command execution
pub type CommandResult<T> = Result<T, CommandError>;

/// Errors that can occur during command execution
#[derive(Debug, Error)]
pub enum CommandError {
    #[error("Command not found: {0}")]
    CommandNotFound(String),

    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),

    #[error("Command execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Command validation failed: {0}")]
    ValidationFailed(String),

    #[error("Command cancelled")]
    Cancelled,

    #[error("Command timeout")]
    Timeout,

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Command parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandParameter {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub param_type: ParameterType,
    /// Human-readable description
    pub description: String,
    /// Whether the parameter is required
    pub required: bool,
    /// Default value (if any)
    pub default_value: Option<String>,
    /// Validation rules
    pub validation: Option<ParameterValidation>,
    /// Autocomplete options (if applicable)
    pub autocomplete: Option<Vec<String>>,
}

/// Parameter type definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    String,
    Integer,
    Float,
    Boolean,
    Choice(Vec<String>),
    FilePath,
    DirectoryPath,
    Url,
}

/// Parameter validation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterValidation {
    /// Minimum length for strings
    pub min_length: Option<usize>,
    /// Maximum length for strings
    pub max_length: Option<usize>,
    /// Regular expression pattern
    pub pattern: Option<String>,
    /// Minimum value for numbers
    pub min_value: Option<f64>,
    /// Maximum value for numbers
    pub max_value: Option<f64>,
    /// Custom validation function name
    pub custom_validator: Option<String>,
}

/// Command definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandDefinition {
    /// Unique command name
    pub name: String,
    /// Human-readable display name
    pub display_name: String,
    /// Command description
    pub description: String,
    /// Command category
    pub category: String,
    /// Command parameters
    pub parameters: Vec<CommandParameter>,
    /// Whether the command requires confirmation
    pub requires_confirmation: bool,
    /// Confirmation message
    pub confirmation_message: Option<String>,
    /// Execution timeout in seconds
    pub timeout_seconds: Option<u64>,
    /// Required permissions
    pub permissions: Vec<String>,
}

/// Command execution context
#[derive(Debug, Clone)]
pub struct CommandContext {
    /// Current working directory
    pub cwd: std::path::PathBuf,
    /// Environment variables
    pub env: HashMap<String, String>,
    /// User information
    pub user: Option<String>,
    /// Session information
    pub session_id: Option<String>,
    /// Additional context data
    pub data: HashMap<String, serde_json::Value>,
}

/// Command execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandExecutionResult {
    /// Command that was executed
    pub command: String,
    /// Parameters used
    pub parameters: HashMap<String, String>,
    /// Execution status
    pub status: ExecutionStatus,
    /// Result output
    pub output: Option<String>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Timestamp of execution
    pub executed_at: chrono::DateTime<chrono::Utc>,
}

/// Execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionStatus {
    Success,
    Failed,
    Cancelled,
    Timeout,
}

/// Command executor trait
#[async_trait::async_trait]
pub trait CommandExecutor: Send + Sync {
    /// Execute a command with the given parameters
    async fn execute(
        &self,
        command: &str,
        parameters: HashMap<String, String>,
        context: &CommandContext,
    ) -> CommandResult<CommandExecutionResult>;

    /// Validate command parameters
    fn validate_parameters(
        &self,
        command: &str,
        parameters: &HashMap<String, String>,
    ) -> CommandResult<()>;

    /// Get autocomplete suggestions for a parameter
    async fn get_autocomplete(
        &self,
        command: &str,
        parameter: &str,
        partial_value: &str,
        context: &CommandContext,
    ) -> CommandResult<Vec<String>>;
}

/// Command registry and executor
pub struct CommandRegistry {
    /// Registered commands
    commands: HashMap<String, CommandDefinition>,
    /// Command executors
    executors: HashMap<String, Arc<dyn CommandExecutor>>,
    /// Parameter prompt handler
    prompt_handler: Option<Arc<dyn ParameterPromptHandler>>,
}

impl CommandRegistry {
    /// Create a new command registry
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
            executors: HashMap::new(),
            prompt_handler: None,
        }
    }

    /// Register a command
    pub fn register_command(
        &mut self,
        definition: CommandDefinition,
        executor: Arc<dyn CommandExecutor>,
    ) {
        let command_name = definition.name.clone();
        self.commands.insert(command_name.clone(), definition);
        self.executors.insert(command_name, executor);
    }

    /// Get a command definition
    pub fn get_command(&self, name: &str) -> Option<&CommandDefinition> {
        self.commands.get(name)
    }

    /// List all commands
    pub fn list_commands(&self) -> Vec<&CommandDefinition> {
        self.commands.values().collect()
    }

    /// List commands by category
    pub fn list_commands_by_category(&self, category: &str) -> Vec<&CommandDefinition> {
        self.commands
            .values()
            .filter(|cmd| cmd.category == category)
            .collect()
    }

    /// Set parameter prompt handler
    pub fn set_prompt_handler(&mut self, handler: Arc<dyn ParameterPromptHandler>) {
        self.prompt_handler = Some(handler);
    }

    /// Execute a command with parameter prompting
    pub async fn execute_command(
        &self,
        command_name: &str,
        initial_params: HashMap<String, String>,
        context: &CommandContext,
    ) -> CommandResult<CommandExecutionResult> {
        let definition = self
            .commands
            .get(command_name)
            .ok_or_else(|| CommandError::CommandNotFound(command_name.to_string()))?;

        let executor = self
            .executors
            .get(command_name)
            .ok_or_else(|| CommandError::CommandNotFound(command_name.to_string()))?;

        // Collect all parameters (initial + prompted)
        let mut all_params = initial_params;

        // Prompt for missing required parameters
        for param in &definition.parameters {
            if param.required && !all_params.contains_key(&param.name) {
                if let Some(prompt_handler) = &self.prompt_handler {
                    let value = prompt_handler.prompt_parameter(param, context).await?;
                    all_params.insert(param.name.clone(), value);
                } else {
                    return Err(CommandError::InvalidParameters(format!(
                        "Missing required parameter: {}",
                        param.name
                    )));
                }
            }
        }

        // Validate parameters
        executor.validate_parameters(command_name, &all_params)?;

        // Check confirmation if required
        if definition.requires_confirmation {
            if let Some(prompt_handler) = &self.prompt_handler {
                let default_message = format!("Execute command '{}'?", command_name);
                let message = definition
                    .confirmation_message
                    .as_ref()
                    .unwrap_or(&default_message);

                if !prompt_handler.confirm_execution(message, context).await? {
                    return Ok(CommandExecutionResult {
                        command: command_name.to_string(),
                        parameters: all_params,
                        status: ExecutionStatus::Cancelled,
                        output: None,
                        error: None,
                        execution_time_ms: 0,
                        executed_at: chrono::Utc::now(),
                    });
                }
            }
        }

        // Execute the command
        let start_time = std::time::Instant::now();
        let result = executor
            .execute(command_name, all_params.clone(), context)
            .await;
        let execution_time = start_time.elapsed().as_millis() as u64;

        match result {
            Ok(mut execution_result) => {
                execution_result.execution_time_ms = execution_time;
                Ok(execution_result)
            }
            Err(e) => {
                // Create error result
                Ok(CommandExecutionResult {
                    command: command_name.to_string(),
                    parameters: all_params,
                    status: ExecutionStatus::Failed,
                    output: None,
                    error: Some(e.to_string()),
                    execution_time_ms: execution_time,
                    executed_at: chrono::Utc::now(),
                })
            }
        }
    }

    /// Get autocomplete suggestions
    pub async fn get_autocomplete(
        &self,
        command: &str,
        parameter: &str,
        partial_value: &str,
        context: &CommandContext,
    ) -> CommandResult<Vec<String>> {
        if let Some(executor) = self.executors.get(command) {
            executor
                .get_autocomplete(command, parameter, partial_value, context)
                .await
        } else {
            Ok(Vec::new())
        }
    }
}

/// Parameter prompt handler trait
#[async_trait::async_trait]
pub trait ParameterPromptHandler: Send + Sync {
    /// Prompt user for a parameter value
    async fn prompt_parameter(
        &self,
        parameter: &CommandParameter,
        context: &CommandContext,
    ) -> CommandResult<String>;

    /// Ask for execution confirmation
    async fn confirm_execution(
        &self,
        message: &str,
        context: &CommandContext,
    ) -> CommandResult<bool>;
}

/// Default parameter validation
pub fn validate_parameter(param: &CommandParameter, value: &str) -> CommandResult<()> {
    // Check required
    if param.required && value.trim().is_empty() {
        return Err(CommandError::InvalidParameters(format!(
            "Parameter '{}' is required",
            param.name
        )));
    }

    // Type-specific validation
    match &param.param_type {
        ParameterType::Integer => {
            if value.parse::<i64>().is_err() {
                return Err(CommandError::InvalidParameters(format!(
                    "Parameter '{}' must be an integer",
                    param.name
                )));
            }
        }
        ParameterType::Float => {
            if value.parse::<f64>().is_err() {
                return Err(CommandError::InvalidParameters(format!(
                    "Parameter '{}' must be a number",
                    param.name
                )));
            }
        }
        ParameterType::Boolean => {
            let lower = value.to_lowercase();
            if !matches!(lower.as_str(), "true" | "false" | "1" | "0" | "yes" | "no") {
                return Err(CommandError::InvalidParameters(format!(
                    "Parameter '{}' must be a boolean (true/false)",
                    param.name
                )));
            }
        }
        ParameterType::Choice(options) => {
            if !options.contains(&value.to_string()) {
                return Err(CommandError::InvalidParameters(format!(
                    "Parameter '{}' must be one of: {:?}",
                    param.name, options
                )));
            }
        }
        ParameterType::Url => {
            if !value.starts_with("http://") && !value.starts_with("https://") {
                return Err(CommandError::InvalidParameters(format!(
                    "Parameter '{}' must be a valid URL",
                    param.name
                )));
            }
        }
        _ => {} // Other types are validated as strings
    }

    // Custom validation rules
    if let Some(validation) = &param.validation {
        if let Some(min_len) = validation.min_length {
            if value.len() < min_len {
                return Err(CommandError::InvalidParameters(format!(
                    "Parameter '{}' must be at least {} characters",
                    param.name, min_len
                )));
            }
        }

        if let Some(max_len) = validation.max_length {
            if value.len() > max_len {
                return Err(CommandError::InvalidParameters(format!(
                    "Parameter '{}' must be at most {} characters",
                    param.name, max_len
                )));
            }
        }

        if let Some(pattern) = &validation.pattern {
            let regex = regex::Regex::new(pattern).map_err(|_| {
                CommandError::ValidationFailed(format!(
                    "Invalid regex pattern for parameter '{}'",
                    param.name
                ))
            })?;

            if !regex.is_match(value) {
                return Err(CommandError::InvalidParameters(format!(
                    "Parameter '{}' does not match required pattern",
                    param.name
                )));
            }
        }

        // Numeric validation
        if matches!(
            param.param_type,
            ParameterType::Integer | ParameterType::Float
        ) {
            if let Ok(num) = value.parse::<f64>() {
                if let Some(min_val) = validation.min_value {
                    if num < min_val {
                        return Err(CommandError::InvalidParameters(format!(
                            "Parameter '{}' must be at least {}",
                            param.name, min_val
                        )));
                    }
                }

                if let Some(max_val) = validation.max_value {
                    if num > max_val {
                        return Err(CommandError::InvalidParameters(format!(
                            "Parameter '{}' must be at most {}",
                            param.name, max_val
                        )));
                    }
                }
            }
        }
    }

    Ok(())
}
