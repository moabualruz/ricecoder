//! Data types for markdown configuration
//!
//! This module defines the core data types for markdown-based configuration:
//! - [`ParsedMarkdown`]: Markdown content with separated frontmatter and body
//! - [`AgentConfig`]: Agent configuration with model and parameters
//! - [`ModeConfig`]: Mode configuration with keybindings
//! - [`CommandConfig`]: Command configuration with parameters and templates
//! - [`Parameter`]: Command parameter definition

use serde::{Deserialize, Serialize};

/// Parsed markdown content with separated frontmatter and body
///
/// This type represents the result of parsing a markdown file with YAML frontmatter.
/// The frontmatter (YAML between `---` delimiters) is separated from the markdown body.
///
/// # Example
///
/// ```ignore
/// let markdown = r#"---
/// name: example
/// description: Example configuration
/// ---
///
/// # Example
/// This is the markdown content.
/// "#;
///
/// let parsed = ParsedMarkdown {
///     frontmatter: Some("name: example\ndescription: Example configuration".to_string()),
///     content: "# Example\nThis is the markdown content.".to_string(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedMarkdown {
    /// YAML frontmatter content (if present)
    ///
    /// Contains the YAML metadata between the `---` delimiters.
    /// If no frontmatter is present, this is `None`.
    pub frontmatter: Option<String>,
    /// Markdown body content
    ///
    /// Contains the markdown content after the frontmatter.
    /// This is used as documentation or prompt text.
    pub content: String,
}

impl ParsedMarkdown {
    /// Create a new ParsedMarkdown
    pub fn new(frontmatter: Option<String>, content: String) -> Self {
        Self { frontmatter, content }
    }
}

/// Agent configuration from markdown
///
/// Defines an AI agent with specific capabilities and parameters.
/// Agents are loaded from `*.agent.md` files and can be queried by name.
///
/// # Fields
///
/// - `name`: Unique identifier for the agent (required)
/// - `description`: Human-readable description (optional)
/// - `prompt`: System prompt or instructions (optional, from markdown body)
/// - `model`: LLM model to use, e.g., "gpt-4" (optional)
/// - `temperature`: Model temperature (0.0-2.0, optional)
/// - `max_tokens`: Maximum response tokens (optional)
/// - `tools`: List of available tools (optional)
///
/// # Example
///
/// ```ignore
/// let agent = AgentConfig {
///     name: "code-review".to_string(),
///     description: Some("Code review agent".to_string()),
///     prompt: "You are a code review expert...".to_string(),
///     model: Some("gpt-4".to_string()),
///     temperature: Some(0.7),
///     max_tokens: Some(2000),
///     tools: vec!["syntax-analyzer".to_string()],
/// };
/// ```
///
/// # Validation
///
/// Use [`AgentConfig::validate`] to validate the configuration:
///
/// ```ignore
/// agent.validate()?;  // Returns error if invalid
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentConfig {
    /// Agent name (required)
    ///
    /// Unique identifier for the agent. Must be lowercase alphanumeric with hyphens.
    pub name: String,
    /// Agent description (optional)
    ///
    /// Human-readable description of the agent's purpose and capabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Agent prompt/system message (from markdown body or frontmatter)
    ///
    /// System prompt or instructions for the agent. Used to guide the agent's behavior.
    #[serde(default)]
    pub prompt: String,
    /// Model to use (optional)
    ///
    /// LLM model identifier, e.g., "gpt-4", "gpt-3.5-turbo".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Temperature parameter (optional, 0.0-2.0)
    ///
    /// Controls randomness: lower values (0.0) are more deterministic,
    /// higher values (2.0) are more creative.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Maximum tokens (optional)
    ///
    /// Maximum number of tokens in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// Tools available to agent (optional)
    ///
    /// List of tool names that the agent can use.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<String>,
}

impl AgentConfig {
    /// Validate agent configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Agent name cannot be empty".to_string());
        }

        if self.prompt.is_empty() {
            return Err("Agent prompt cannot be empty".to_string());
        }

        if let Some(temp) = self.temperature {
            if !(0.0..=2.0).contains(&temp) {
                return Err(format!(
                    "Temperature must be between 0.0 and 2.0, got {}",
                    temp
                ));
            }
        }

        if let Some(tokens) = self.max_tokens {
            if tokens == 0 {
                return Err("max_tokens must be greater than 0".to_string());
            }
        }

        Ok(())
    }
}

/// Mode configuration from markdown
///
/// Defines an editor mode with specific behaviors and keybindings.
/// Modes are loaded from `*.mode.md` files and can be activated by name or keybinding.
///
/// # Fields
///
/// - `name`: Unique identifier for the mode (required)
/// - `description`: Human-readable description (optional)
/// - `prompt`: Mode prompt or instructions (optional, from markdown body)
/// - `keybinding`: Keyboard shortcut to activate (optional)
/// - `enabled`: Whether the mode is enabled (default: true)
///
/// # Example
///
/// ```ignore
/// let mode = ModeConfig {
///     name: "focus".to_string(),
///     description: Some("Focus mode for distraction-free coding".to_string()),
///     prompt: "Minimize UI elements...".to_string(),
///     keybinding: Some("ctrl+shift+f".to_string()),
///     enabled: true,
/// };
/// ```
///
/// # Keybinding Format
///
/// Keybindings use the format: `[modifier+]key`
///
/// Modifiers: `ctrl`, `shift`, `alt`, `meta`
///
/// Examples: `ctrl+s`, `ctrl+shift+f`, `alt+enter`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModeConfig {
    /// Mode name (required)
    ///
    /// Unique identifier for the mode. Must be lowercase alphanumeric with hyphens.
    pub name: String,
    /// Mode description (optional)
    ///
    /// Human-readable description of the mode's purpose.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Mode prompt/system message (from markdown body or frontmatter)
    ///
    /// Instructions or documentation for the mode.
    #[serde(default)]
    pub prompt: String,
    /// Keybinding (optional)
    ///
    /// Keyboard shortcut to activate the mode, e.g., "ctrl+shift+f".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keybinding: Option<String>,
    /// Whether mode is enabled (default: true)
    ///
    /// If false, the mode is not available for activation.
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

impl ModeConfig {
    /// Validate mode configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Mode name cannot be empty".to_string());
        }

        if self.prompt.is_empty() {
            return Err("Mode prompt cannot be empty".to_string());
        }

        Ok(())
    }
}

/// Command parameter configuration
///
/// Defines a parameter for a command template.
/// Parameters are substituted into command templates using `{{parameter_name}}` syntax.
///
/// # Fields
///
/// - `name`: Parameter identifier (required)
/// - `description`: Human-readable description (optional)
/// - `required`: Whether the parameter must be provided (default: false)
/// - `default`: Default value if not provided (optional)
///
/// # Example
///
/// ```ignore
/// let param = Parameter {
///     name: "test_filter".to_string(),
///     description: Some("Filter for specific tests".to_string()),
///     required: false,
///     default: Some("".to_string()),
/// };
/// ```
///
/// # Template Substitution
///
/// Parameters are substituted into command templates:
///
/// ```text
/// Template: "cargo test {{test_filter}}"
/// Parameter: test_filter = "parser"
/// Result: "cargo test parser"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Parameter {
    /// Parameter name (required)
    ///
    /// Used in template as `{{name}}`. Must be valid identifier (alphanumeric, underscores).
    pub name: String,
    /// Parameter description (optional)
    ///
    /// Human-readable description of what the parameter does.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether parameter is required (default: false)
    ///
    /// If true, the parameter must be provided when invoking the command.
    #[serde(default)]
    pub required: bool,
    /// Default value (optional)
    ///
    /// Used if the parameter is not provided and is not required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
}

impl Parameter {
    /// Validate parameter configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Parameter name cannot be empty".to_string());
        }

        Ok(())
    }
}

/// Command configuration from markdown
///
/// Defines a custom command with parameters and template.
/// Commands are loaded from `*.command.md` files and can be invoked by name or keybinding.
///
/// # Fields
///
/// - `name`: Unique identifier for the command (required)
/// - `description`: Human-readable description (optional)
/// - `template`: Command template with parameter placeholders (required)
/// - `parameters`: List of parameter definitions (optional)
/// - `keybinding`: Keyboard shortcut to invoke (optional)
///
/// # Example
///
/// ```ignore
/// let command = CommandConfig {
///     name: "test".to_string(),
///     description: Some("Run tests".to_string()),
///     template: "cargo test {{test_filter}}".to_string(),
///     parameters: vec![Parameter {
///         name: "test_filter".to_string(),
///         description: Some("Filter for specific tests".to_string()),
///         required: false,
///         default: Some("".to_string()),
///     }],
///     keybinding: Some("ctrl+shift+t".to_string()),
/// };
/// ```
///
/// # Template Substitution
///
/// Parameters are substituted into the template using `{{parameter_name}}` syntax:
///
/// ```text
/// Template: "cargo test {{test_filter}}"
/// Invocation: test parser
/// Result: "cargo test parser"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommandConfig {
    /// Command name (required)
    ///
    /// Unique identifier for the command. Must be lowercase alphanumeric with hyphens.
    pub name: String,
    /// Command description (optional)
    ///
    /// Human-readable description of what the command does.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Command template (from markdown body or frontmatter)
    ///
    /// Template string with parameter placeholders, e.g., "cargo test {{filter}}".
    #[serde(default)]
    pub template: String,
    /// Command parameters (optional)
    ///
    /// List of parameters that can be substituted into the template.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<Parameter>,
    /// Keybinding (optional)
    ///
    /// Keyboard shortcut to invoke the command, e.g., "ctrl+shift+t".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keybinding: Option<String>,
}

impl CommandConfig {
    /// Validate command configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Command name cannot be empty".to_string());
        }

        if self.template.is_empty() {
            return Err("Command template cannot be empty".to_string());
        }

        for param in &self.parameters {
            param.validate()?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsed_markdown_creation() {
        let parsed = ParsedMarkdown::new(
            Some("name: test".to_string()),
            "# Test Content".to_string(),
        );
        assert_eq!(parsed.frontmatter, Some("name: test".to_string()));
        assert_eq!(parsed.content, "# Test Content");
    }

    #[test]
    fn test_agent_config_validation_success() {
        let config = AgentConfig {
            name: "test-agent".to_string(),
            description: Some("Test agent".to_string()),
            prompt: "You are a helpful assistant".to_string(),
            model: Some("gpt-4".to_string()),
            temperature: Some(0.7),
            max_tokens: Some(2000),
            tools: vec!["tool1".to_string()],
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_agent_config_validation_empty_name() {
        let config = AgentConfig {
            name: String::new(),
            description: None,
            prompt: "Test".to_string(),
            model: None,
            temperature: None,
            max_tokens: None,
            tools: vec![],
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_agent_config_validation_invalid_temperature() {
        let config = AgentConfig {
            name: "test".to_string(),
            description: None,
            prompt: "Test".to_string(),
            model: None,
            temperature: Some(3.0),
            max_tokens: None,
            tools: vec![],
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_mode_config_validation_success() {
        let config = ModeConfig {
            name: "focus".to_string(),
            description: Some("Focus mode".to_string()),
            prompt: "Focus on the task".to_string(),
            keybinding: Some("C-f".to_string()),
            enabled: true,
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_command_config_validation_success() {
        let config = CommandConfig {
            name: "test-command".to_string(),
            description: Some("Test command".to_string()),
            template: "echo {{message}}".to_string(),
            parameters: vec![Parameter {
                name: "message".to_string(),
                description: Some("Message to echo".to_string()),
                required: true,
                default: None,
            }],
            keybinding: Some("C-t".to_string()),
        };
        assert!(config.validate().is_ok());
    }
}
