//! Data types for markdown configuration

use serde::{Deserialize, Serialize};

/// Parsed markdown content with separated frontmatter and body
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedMarkdown {
    /// YAML frontmatter content (if present)
    pub frontmatter: Option<String>,
    /// Markdown body content
    pub content: String,
}

impl ParsedMarkdown {
    /// Create a new ParsedMarkdown
    pub fn new(frontmatter: Option<String>, content: String) -> Self {
        Self { frontmatter, content }
    }
}

/// Agent configuration from markdown
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentConfig {
    /// Agent name (required)
    pub name: String,
    /// Agent description (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Agent prompt/system message (from markdown body or frontmatter)
    #[serde(default)]
    pub prompt: String,
    /// Model to use (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Temperature parameter (optional, 0.0-2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Maximum tokens (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// Tools available to agent (optional)
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModeConfig {
    /// Mode name (required)
    pub name: String,
    /// Mode description (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Mode prompt/system message (from markdown body or frontmatter)
    #[serde(default)]
    pub prompt: String,
    /// Keybinding (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keybinding: Option<String>,
    /// Whether mode is enabled (default: true)
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Parameter {
    /// Parameter name (required)
    pub name: String,
    /// Parameter description (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether parameter is required (default: false)
    #[serde(default)]
    pub required: bool,
    /// Default value (optional)
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommandConfig {
    /// Command name (required)
    pub name: String,
    /// Command description (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Command template (from markdown body or frontmatter)
    #[serde(default)]
    pub template: String,
    /// Command parameters (optional)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<Parameter>,
    /// Keybinding (optional)
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
