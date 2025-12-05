//! Validation functions for markdown configuration

use crate::markdown_config::error::{MarkdownConfigError, MarkdownConfigResult};
use crate::markdown_config::types::{AgentConfig, CommandConfig, ModeConfig, Parameter};

/// Validation result containing all errors found
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationResult {
    /// List of validation errors found
    pub errors: Vec<String>,
}

impl ValidationResult {
    /// Create a new validation result
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
        }
    }

    /// Add an error to the validation result
    pub fn add_error(&mut self, error: impl Into<String>) {
        self.errors.push(error.into());
    }

    /// Check if validation passed (no errors)
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get the number of errors
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Convert to a MarkdownConfigError if there are errors
    pub fn to_error(self) -> Option<MarkdownConfigError> {
        if self.errors.is_empty() {
            None
        } else {
            let message = self.errors.join("; ");
            Some(MarkdownConfigError::validation_error(message))
        }
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

// ============ Agent Validation ============

/// Validate an agent configuration
///
/// Checks all required fields and validates field values.
/// Returns a ValidationResult containing all errors found.
pub fn validate_agent_config(config: &AgentConfig) -> ValidationResult {
    let mut result = ValidationResult::new();

    // Check required fields
    if config.name.is_empty() {
        result.add_error("Agent name is required and cannot be empty");
    }

    if config.prompt.is_empty() {
        result.add_error("Agent prompt is required and cannot be empty");
    }

    // Validate optional fields
    if let Some(temp) = config.temperature {
        if !(0.0..=2.0).contains(&temp) {
            result.add_error(format!(
                "Agent temperature must be between 0.0 and 2.0, got {}",
                temp
            ));
        }
    }

    if let Some(tokens) = config.max_tokens {
        if tokens == 0 {
            result.add_error("Agent max_tokens must be greater than 0");
        }
    }

    result
}

/// Validate an agent configuration and return an error if invalid
pub fn validate_agent_config_strict(config: &AgentConfig) -> MarkdownConfigResult<()> {
    let result = validate_agent_config(config);
    result.to_error().map_or(Ok(()), Err)
}

// ============ Mode Validation ============

/// Validate a mode configuration
///
/// Checks all required fields and validates field values.
/// Returns a ValidationResult containing all errors found.
pub fn validate_mode_config(config: &ModeConfig) -> ValidationResult {
    let mut result = ValidationResult::new();

    // Check required fields
    if config.name.is_empty() {
        result.add_error("Mode name is required and cannot be empty");
    }

    if config.prompt.is_empty() {
        result.add_error("Mode prompt is required and cannot be empty");
    }

    result
}

/// Validate a mode configuration and return an error if invalid
pub fn validate_mode_config_strict(config: &ModeConfig) -> MarkdownConfigResult<()> {
    let result = validate_mode_config(config);
    result.to_error().map_or(Ok(()), Err)
}

// ============ Command Validation ============

/// Validate a command configuration
///
/// Checks all required fields and validates field values.
/// Returns a ValidationResult containing all errors found.
pub fn validate_command_config(config: &CommandConfig) -> ValidationResult {
    let mut result = ValidationResult::new();

    // Check required fields
    if config.name.is_empty() {
        result.add_error("Command name is required and cannot be empty");
    }

    if config.template.is_empty() {
        result.add_error("Command template is required and cannot be empty");
    }

    // Validate parameters
    for (idx, param) in config.parameters.iter().enumerate() {
        let param_errors = validate_parameter(param);
        for error in param_errors.errors {
            result.add_error(format!("Parameter[{}]: {}", idx, error));
        }
    }

    result
}

/// Validate a command configuration and return an error if invalid
pub fn validate_command_config_strict(config: &CommandConfig) -> MarkdownConfigResult<()> {
    let result = validate_command_config(config);
    result.to_error().map_or(Ok(()), Err)
}

// ============ Parameter Validation ============

/// Validate a parameter configuration
///
/// Checks all required fields and validates field values.
/// Returns a ValidationResult containing all errors found.
pub fn validate_parameter(param: &Parameter) -> ValidationResult {
    let mut result = ValidationResult::new();

    // Check required fields
    if param.name.is_empty() {
        result.add_error("Parameter name is required and cannot be empty");
    }

    result
}

/// Validate a parameter configuration and return an error if invalid
pub fn validate_parameter_strict(param: &Parameter) -> MarkdownConfigResult<()> {
    let result = validate_parameter(param);
    result.to_error().map_or(Ok(()), Err)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_agent(name: &str) -> AgentConfig {
        AgentConfig {
            name: name.to_string(),
            description: Some("Test agent".to_string()),
            prompt: "You are a helpful assistant".to_string(),
            model: Some("gpt-4".to_string()),
            temperature: Some(0.7),
            max_tokens: Some(2000),
            tools: vec![],
        }
    }

    fn create_test_mode(name: &str) -> ModeConfig {
        ModeConfig {
            name: name.to_string(),
            description: Some("Test mode".to_string()),
            prompt: "Focus on the task".to_string(),
            keybinding: Some("C-f".to_string()),
            enabled: true,
        }
    }

    fn create_test_command(name: &str) -> CommandConfig {
        CommandConfig {
            name: name.to_string(),
            description: Some("Test command".to_string()),
            template: "echo {{message}}".to_string(),
            parameters: vec![],
            keybinding: Some("C-t".to_string()),
        }
    }

    // ============ ValidationResult Tests ============

    #[test]
    fn test_validation_result_new() {
        let result = ValidationResult::new();
        assert!(result.is_valid());
        assert_eq!(result.error_count(), 0);
    }

    #[test]
    fn test_validation_result_add_error() {
        let mut result = ValidationResult::new();
        result.add_error("Test error");
        assert!(!result.is_valid());
        assert_eq!(result.error_count(), 1);
    }

    #[test]
    fn test_validation_result_multiple_errors() {
        let mut result = ValidationResult::new();
        result.add_error("Error 1");
        result.add_error("Error 2");
        result.add_error("Error 3");
        assert!(!result.is_valid());
        assert_eq!(result.error_count(), 3);
    }

    #[test]
    fn test_validation_result_to_error_valid() {
        let result = ValidationResult::new();
        assert!(result.to_error().is_none());
    }

    #[test]
    fn test_validation_result_to_error_invalid() {
        let mut result = ValidationResult::new();
        result.add_error("Test error");
        assert!(result.to_error().is_some());
    }

    // ============ Agent Validation Tests ============

    #[test]
    fn test_validate_agent_config_valid() {
        let agent = create_test_agent("test-agent");
        let result = validate_agent_config(&agent);
        assert!(result.is_valid());
    }

    #[test]
    fn test_validate_agent_config_empty_name() {
        let mut agent = create_test_agent("test");
        agent.name = String::new();
        let result = validate_agent_config(&agent);
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| e.contains("name")));
    }

    #[test]
    fn test_validate_agent_config_empty_prompt() {
        let mut agent = create_test_agent("test");
        agent.prompt = String::new();
        let result = validate_agent_config(&agent);
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| e.contains("prompt")));
    }

    #[test]
    fn test_validate_agent_config_invalid_temperature_too_high() {
        let mut agent = create_test_agent("test");
        agent.temperature = Some(3.0);
        let result = validate_agent_config(&agent);
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| e.contains("temperature")));
    }

    #[test]
    fn test_validate_agent_config_invalid_temperature_negative() {
        let mut agent = create_test_agent("test");
        agent.temperature = Some(-0.5);
        let result = validate_agent_config(&agent);
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| e.contains("temperature")));
    }

    #[test]
    fn test_validate_agent_config_invalid_max_tokens() {
        let mut agent = create_test_agent("test");
        agent.max_tokens = Some(0);
        let result = validate_agent_config(&agent);
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| e.contains("max_tokens")));
    }

    #[test]
    fn test_validate_agent_config_multiple_errors() {
        let mut agent = create_test_agent("test");
        agent.name = String::new();
        agent.prompt = String::new();
        agent.temperature = Some(3.0);
        let result = validate_agent_config(&agent);
        assert!(!result.is_valid());
        assert_eq!(result.error_count(), 3);
    }

    #[test]
    fn test_validate_agent_config_strict_valid() {
        let agent = create_test_agent("test");
        assert!(validate_agent_config_strict(&agent).is_ok());
    }

    #[test]
    fn test_validate_agent_config_strict_invalid() {
        let mut agent = create_test_agent("test");
        agent.name = String::new();
        assert!(validate_agent_config_strict(&agent).is_err());
    }

    // ============ Mode Validation Tests ============

    #[test]
    fn test_validate_mode_config_valid() {
        let mode = create_test_mode("test-mode");
        let result = validate_mode_config(&mode);
        assert!(result.is_valid());
    }

    #[test]
    fn test_validate_mode_config_empty_name() {
        let mut mode = create_test_mode("test");
        mode.name = String::new();
        let result = validate_mode_config(&mode);
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| e.contains("name")));
    }

    #[test]
    fn test_validate_mode_config_empty_prompt() {
        let mut mode = create_test_mode("test");
        mode.prompt = String::new();
        let result = validate_mode_config(&mode);
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| e.contains("prompt")));
    }

    #[test]
    fn test_validate_mode_config_strict_valid() {
        let mode = create_test_mode("test");
        assert!(validate_mode_config_strict(&mode).is_ok());
    }

    #[test]
    fn test_validate_mode_config_strict_invalid() {
        let mut mode = create_test_mode("test");
        mode.name = String::new();
        assert!(validate_mode_config_strict(&mode).is_err());
    }

    // ============ Command Validation Tests ============

    #[test]
    fn test_validate_command_config_valid() {
        let command = create_test_command("test-command");
        let result = validate_command_config(&command);
        assert!(result.is_valid());
    }

    #[test]
    fn test_validate_command_config_empty_name() {
        let mut command = create_test_command("test");
        command.name = String::new();
        let result = validate_command_config(&command);
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| e.contains("name")));
    }

    #[test]
    fn test_validate_command_config_empty_template() {
        let mut command = create_test_command("test");
        command.template = String::new();
        let result = validate_command_config(&command);
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| e.contains("template")));
    }

    #[test]
    fn test_validate_command_config_with_parameters() {
        let mut command = create_test_command("test");
        command.parameters = vec![
            Parameter {
                name: "param1".to_string(),
                description: Some("First parameter".to_string()),
                required: true,
                default: None,
            },
            Parameter {
                name: "param2".to_string(),
                description: None,
                required: false,
                default: Some("default".to_string()),
            },
        ];
        let result = validate_command_config(&command);
        assert!(result.is_valid());
    }

    #[test]
    fn test_validate_command_config_invalid_parameter() {
        let mut command = create_test_command("test");
        command.parameters = vec![Parameter {
            name: String::new(),
            description: None,
            required: false,
            default: None,
        }];
        let result = validate_command_config(&command);
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| e.contains("Parameter")));
    }

    #[test]
    fn test_validate_command_config_strict_valid() {
        let command = create_test_command("test");
        assert!(validate_command_config_strict(&command).is_ok());
    }

    #[test]
    fn test_validate_command_config_strict_invalid() {
        let mut command = create_test_command("test");
        command.name = String::new();
        assert!(validate_command_config_strict(&command).is_err());
    }

    // ============ Parameter Validation Tests ============

    #[test]
    fn test_validate_parameter_valid() {
        let param = Parameter {
            name: "test-param".to_string(),
            description: Some("Test parameter".to_string()),
            required: true,
            default: None,
        };
        let result = validate_parameter(&param);
        assert!(result.is_valid());
    }

    #[test]
    fn test_validate_parameter_empty_name() {
        let param = Parameter {
            name: String::new(),
            description: None,
            required: false,
            default: None,
        };
        let result = validate_parameter(&param);
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| e.contains("name")));
    }

    #[test]
    fn test_validate_parameter_strict_valid() {
        let param = Parameter {
            name: "test".to_string(),
            description: None,
            required: false,
            default: None,
        };
        assert!(validate_parameter_strict(&param).is_ok());
    }

    #[test]
    fn test_validate_parameter_strict_invalid() {
        let param = Parameter {
            name: String::new(),
            description: None,
            required: false,
            default: None,
        };
        assert!(validate_parameter_strict(&param).is_err());
    }
}
