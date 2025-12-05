//! Configuration validation for hooks
//!
//! Validates hook configurations to ensure they are well-formed and contain
//! all required fields with correct types.

use crate::error::{HooksError, Result};
use crate::types::{Action, AiPromptAction, ChainAction, CommandAction, Hook, ToolCallAction};

/// Configuration validator for hooks
///
/// Validates hook configurations to ensure they meet requirements:
/// - All required fields are present
/// - Field types are correct
/// - Event names are non-empty and valid
/// - Action configurations are valid
/// - Condition expressions are valid (if present)
pub struct ConfigValidator;

impl ConfigValidator {
    /// Validate a single hook configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the hook configuration is invalid.
    pub fn validate_hook(hook: &Hook) -> Result<()> {
        // Validate required fields
        if hook.id.is_empty() {
            return Err(HooksError::InvalidConfiguration(
                "Hook ID cannot be empty".to_string(),
            ));
        }

        if hook.name.is_empty() {
            return Err(HooksError::InvalidConfiguration(
                "Hook name cannot be empty".to_string(),
            ));
        }

        // Validate event name
        Self::validate_event_name(&hook.event)?;

        // Validate action
        Self::validate_action(&hook.action)?;

        // Validate condition if present
        if let Some(condition) = &hook.condition {
            Self::validate_condition(condition)?;
        }

        Ok(())
    }

    /// Validate event name
    ///
    /// Event names must be non-empty and follow a valid format.
    fn validate_event_name(event: &str) -> Result<()> {
        if event.is_empty() {
            return Err(HooksError::InvalidConfiguration(
                "Event name cannot be empty".to_string(),
            ));
        }

        // Event names should be lowercase with underscores
        if !event.chars().all(|c| c.is_ascii_lowercase() || c == '_') {
            return Err(HooksError::InvalidConfiguration(format!(
                "Invalid event name format: '{}'. Event names must be lowercase with underscores.",
                event
            )));
        }

        Ok(())
    }

    /// Validate action configuration
    ///
    /// Validates that the action is properly configured based on its type.
    fn validate_action(action: &Action) -> Result<()> {
        match action {
            Action::Command(cmd) => Self::validate_command_action(cmd),
            Action::ToolCall(tool) => Self::validate_tool_call_action(tool),
            Action::AiPrompt(prompt) => Self::validate_ai_prompt_action(prompt),
            Action::Chain(chain) => Self::validate_chain_action(chain),
        }
    }

    /// Validate command action
    fn validate_command_action(action: &CommandAction) -> Result<()> {
        if action.command.is_empty() {
            return Err(HooksError::InvalidConfiguration(
                "Command action: command cannot be empty".to_string(),
            ));
        }

        // Validate timeout if present
        if let Some(timeout) = action.timeout_ms {
            if timeout == 0 {
                return Err(HooksError::InvalidConfiguration(
                    "Command action: timeout must be greater than 0".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Validate tool call action
    fn validate_tool_call_action(action: &ToolCallAction) -> Result<()> {
        if action.tool_name.is_empty() {
            return Err(HooksError::InvalidConfiguration(
                "Tool call action: tool_name cannot be empty".to_string(),
            ));
        }

        if action.tool_path.is_empty() {
            return Err(HooksError::InvalidConfiguration(
                "Tool call action: tool_path cannot be empty".to_string(),
            ));
        }

        // Validate timeout if present
        if let Some(timeout) = action.timeout_ms {
            if timeout == 0 {
                return Err(HooksError::InvalidConfiguration(
                    "Tool call action: timeout must be greater than 0".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Validate AI prompt action
    fn validate_ai_prompt_action(action: &AiPromptAction) -> Result<()> {
        if action.prompt_template.is_empty() {
            return Err(HooksError::InvalidConfiguration(
                "AI prompt action: prompt_template cannot be empty".to_string(),
            ));
        }

        // Validate temperature if present
        if let Some(temp) = action.temperature {
            if !(0.0..=2.0).contains(&temp) {
                return Err(HooksError::InvalidConfiguration(
                    "AI prompt action: temperature must be between 0.0 and 2.0".to_string(),
                ));
            }
        }

        // Validate max_tokens if present
        if let Some(tokens) = action.max_tokens {
            if tokens == 0 {
                return Err(HooksError::InvalidConfiguration(
                    "AI prompt action: max_tokens must be greater than 0".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Validate chain action
    fn validate_chain_action(action: &ChainAction) -> Result<()> {
        if action.hook_ids.is_empty() {
            return Err(HooksError::InvalidConfiguration(
                "Chain action: hook_ids cannot be empty".to_string(),
            ));
        }

        // Check for duplicate hook IDs
        let mut seen = std::collections::HashSet::new();
        for id in &action.hook_ids {
            if !seen.insert(id) {
                return Err(HooksError::InvalidConfiguration(format!(
                    "Chain action: duplicate hook ID '{}'",
                    id
                )));
            }
        }

        Ok(())
    }

    /// Validate condition expression
    fn validate_condition(condition: &crate::types::Condition) -> Result<()> {
        if condition.expression.is_empty() {
            return Err(HooksError::InvalidConfiguration(
                "Condition: expression cannot be empty".to_string(),
            ));
        }

        if condition.context_keys.is_empty() {
            return Err(HooksError::InvalidConfiguration(
                "Condition: context_keys cannot be empty".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CommandAction, Condition};
    use serde_json::json;

    fn create_test_hook() -> Hook {
        Hook {
            id: "test-hook".to_string(),
            name: "Test Hook".to_string(),
            description: None,
            event: "file_saved".to_string(),
            action: Action::Command(CommandAction {
                command: "echo".to_string(),
                args: vec![],
                timeout_ms: None,
                capture_output: false,
            }),
            enabled: true,
            tags: vec![],
            metadata: json!({}),
            condition: None,
        }
    }

    #[test]
    fn test_validate_hook_valid() {
        let hook = create_test_hook();
        assert!(ConfigValidator::validate_hook(&hook).is_ok());
    }

    #[test]
    fn test_validate_hook_empty_id() {
        let mut hook = create_test_hook();
        hook.id = String::new();
        assert!(ConfigValidator::validate_hook(&hook).is_err());
    }

    #[test]
    fn test_validate_hook_empty_name() {
        let mut hook = create_test_hook();
        hook.name = String::new();
        assert!(ConfigValidator::validate_hook(&hook).is_err());
    }

    #[test]
    fn test_validate_event_name_valid() {
        assert!(ConfigValidator::validate_event_name("file_saved").is_ok());
        assert!(ConfigValidator::validate_event_name("test_passed").is_ok());
        assert!(ConfigValidator::validate_event_name("event").is_ok());
    }

    #[test]
    fn test_validate_event_name_empty() {
        assert!(ConfigValidator::validate_event_name("").is_err());
    }

    #[test]
    fn test_validate_event_name_invalid_format() {
        assert!(ConfigValidator::validate_event_name("FileSaved").is_err());
        assert!(ConfigValidator::validate_event_name("file-saved").is_err());
        assert!(ConfigValidator::validate_event_name("file.saved").is_err());
    }

    #[test]
    fn test_validate_command_action_valid() {
        let action = CommandAction {
            command: "echo".to_string(),
            args: vec!["hello".to_string()],
            timeout_ms: Some(5000),
            capture_output: true,
        };
        assert!(ConfigValidator::validate_command_action(&action).is_ok());
    }

    #[test]
    fn test_validate_command_action_empty_command() {
        let action = CommandAction {
            command: String::new(),
            args: vec![],
            timeout_ms: None,
            capture_output: false,
        };
        assert!(ConfigValidator::validate_command_action(&action).is_err());
    }

    #[test]
    fn test_validate_command_action_zero_timeout() {
        let action = CommandAction {
            command: "echo".to_string(),
            args: vec![],
            timeout_ms: Some(0),
            capture_output: false,
        };
        assert!(ConfigValidator::validate_command_action(&action).is_err());
    }

    #[test]
    fn test_validate_tool_call_action_valid() {
        let action = ToolCallAction {
            tool_name: "formatter".to_string(),
            tool_path: "/usr/bin/prettier".to_string(),
            parameters: crate::types::ParameterBindings {
                bindings: std::collections::HashMap::new(),
            },
            timeout_ms: Some(5000),
        };
        assert!(ConfigValidator::validate_tool_call_action(&action).is_ok());
    }

    #[test]
    fn test_validate_tool_call_action_empty_name() {
        let action = ToolCallAction {
            tool_name: String::new(),
            tool_path: "/usr/bin/prettier".to_string(),
            parameters: crate::types::ParameterBindings {
                bindings: std::collections::HashMap::new(),
            },
            timeout_ms: None,
        };
        assert!(ConfigValidator::validate_tool_call_action(&action).is_err());
    }

    #[test]
    fn test_validate_tool_call_action_empty_path() {
        let action = ToolCallAction {
            tool_name: "formatter".to_string(),
            tool_path: String::new(),
            parameters: crate::types::ParameterBindings {
                bindings: std::collections::HashMap::new(),
            },
            timeout_ms: None,
        };
        assert!(ConfigValidator::validate_tool_call_action(&action).is_err());
    }

    #[test]
    fn test_validate_ai_prompt_action_valid() {
        let action = AiPromptAction {
            prompt_template: "Format this code: {{code}}".to_string(),
            variables: std::collections::HashMap::new(),
            model: Some("gpt-4".to_string()),
            temperature: Some(0.7),
            max_tokens: Some(2000),
            stream: true,
        };
        assert!(ConfigValidator::validate_ai_prompt_action(&action).is_ok());
    }

    #[test]
    fn test_validate_ai_prompt_action_empty_template() {
        let action = AiPromptAction {
            prompt_template: String::new(),
            variables: std::collections::HashMap::new(),
            model: None,
            temperature: None,
            max_tokens: None,
            stream: false,
        };
        assert!(ConfigValidator::validate_ai_prompt_action(&action).is_err());
    }

    #[test]
    fn test_validate_ai_prompt_action_invalid_temperature() {
        let action = AiPromptAction {
            prompt_template: "Format this code".to_string(),
            variables: std::collections::HashMap::new(),
            model: None,
            temperature: Some(3.0),
            max_tokens: None,
            stream: false,
        };
        assert!(ConfigValidator::validate_ai_prompt_action(&action).is_err());
    }

    #[test]
    fn test_validate_ai_prompt_action_zero_max_tokens() {
        let action = AiPromptAction {
            prompt_template: "Format this code".to_string(),
            variables: std::collections::HashMap::new(),
            model: None,
            temperature: None,
            max_tokens: Some(0),
            stream: false,
        };
        assert!(ConfigValidator::validate_ai_prompt_action(&action).is_err());
    }

    #[test]
    fn test_validate_chain_action_valid() {
        let action = ChainAction {
            hook_ids: vec!["hook1".to_string(), "hook2".to_string()],
            pass_output: true,
        };
        assert!(ConfigValidator::validate_chain_action(&action).is_ok());
    }

    #[test]
    fn test_validate_chain_action_empty_ids() {
        let action = ChainAction {
            hook_ids: vec![],
            pass_output: false,
        };
        assert!(ConfigValidator::validate_chain_action(&action).is_err());
    }

    #[test]
    fn test_validate_chain_action_duplicate_ids() {
        let action = ChainAction {
            hook_ids: vec!["hook1".to_string(), "hook1".to_string()],
            pass_output: false,
        };
        assert!(ConfigValidator::validate_chain_action(&action).is_err());
    }

    #[test]
    fn test_validate_condition_valid() {
        let condition = Condition {
            expression: "file_path.ends_with('.rs')".to_string(),
            context_keys: vec!["file_path".to_string()],
        };
        assert!(ConfigValidator::validate_condition(&condition).is_ok());
    }

    #[test]
    fn test_validate_condition_empty_expression() {
        let condition = Condition {
            expression: String::new(),
            context_keys: vec!["file_path".to_string()],
        };
        assert!(ConfigValidator::validate_condition(&condition).is_err());
    }

    #[test]
    fn test_validate_condition_empty_context_keys() {
        let condition = Condition {
            expression: "file_path.ends_with('.rs')".to_string(),
            context_keys: vec![],
        };
        assert!(ConfigValidator::validate_condition(&condition).is_err());
    }

    #[test]
    fn test_validate_hook_with_condition() {
        let mut hook = create_test_hook();
        hook.condition = Some(Condition {
            expression: "file_path.ends_with('.rs')".to_string(),
            context_keys: vec!["file_path".to_string()],
        });
        assert!(ConfigValidator::validate_hook(&hook).is_ok());
    }
}
