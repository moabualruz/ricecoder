//! Configuration types for language-agnostic, configuration-driven architecture

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Configuration error type
#[derive(Debug, Error)]
pub enum ConfigError {
    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// YAML parsing error
    #[error("YAML parsing error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    /// JSON parsing error
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Missing configuration
    #[error("Missing configuration: {0}")]
    MissingConfig(String),
}

/// Result type for configuration operations
pub type ConfigResult<T> = Result<T, ConfigError>;

/// Language configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageConfig {
    /// Language identifier (e.g., "rust", "typescript", "python")
    pub language: String,

    /// File extensions for this language
    #[serde(default)]
    pub extensions: Vec<String>,

    /// Parser plugin reference (e.g., "tree-sitter-rust")
    pub parser_plugin: Option<String>,

    /// Diagnostic rules for this language
    #[serde(default)]
    pub diagnostic_rules: Vec<DiagnosticRule>,

    /// Code action transformations for this language
    #[serde(default)]
    pub code_actions: Vec<CodeActionTemplate>,
}

impl LanguageConfig {
    /// Validate the language configuration
    pub fn validate(&self) -> ConfigResult<()> {
        if self.language.is_empty() {
            return Err(ConfigError::ValidationError(
                "Language name cannot be empty".to_string(),
            ));
        }

        if self.extensions.is_empty() {
            return Err(ConfigError::ValidationError(
                format!("Language '{}' must have at least one file extension", self.language),
            ));
        }

        Ok(())
    }
}

/// Diagnostic rule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticRule {
    /// Rule name
    pub name: String,

    /// Pattern to match (regex or simple pattern)
    pub pattern: String,

    /// Severity level: "error", "warning", "info"
    pub severity: String,

    /// Diagnostic message
    pub message: String,

    /// Optional fix template
    pub fix_template: Option<String>,

    /// Rule code for identification
    pub code: Option<String>,
}

impl DiagnosticRule {
    /// Validate the diagnostic rule
    pub fn validate(&self) -> ConfigResult<()> {
        if self.name.is_empty() {
            return Err(ConfigError::ValidationError(
                "Rule name cannot be empty".to_string(),
            ));
        }

        if self.pattern.is_empty() {
            return Err(ConfigError::ValidationError(
                "Rule pattern cannot be empty".to_string(),
            ));
        }

        match self.severity.as_str() {
            "error" | "warning" | "info" => {}
            _ => {
                return Err(ConfigError::ValidationError(
                    format!("Invalid severity level: {}", self.severity),
                ))
            }
        }

        Ok(())
    }
}

/// Code action template configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeActionTemplate {
    /// Action name
    pub name: String,

    /// Action title
    pub title: String,

    /// Action kind: "quickfix", "refactor", "source"
    pub kind: String,

    /// Transformation template
    pub transformation: String,
}

impl CodeActionTemplate {
    /// Validate the code action template
    pub fn validate(&self) -> ConfigResult<()> {
        if self.name.is_empty() {
            return Err(ConfigError::ValidationError(
                "Action name cannot be empty".to_string(),
            ));
        }

        if self.title.is_empty() {
            return Err(ConfigError::ValidationError(
                "Action title cannot be empty".to_string(),
            ));
        }

        match self.kind.as_str() {
            "quickfix" | "refactor" | "source" => {}
            _ => {
                return Err(ConfigError::ValidationError(
                    format!("Invalid action kind: {}", self.kind),
                ))
            }
        }

        Ok(())
    }
}

/// Configuration registry for managing multiple languages
pub struct ConfigRegistry {
    /// Language configurations by language identifier
    languages: HashMap<String, LanguageConfig>,
}

impl ConfigRegistry {
    /// Create a new configuration registry
    pub fn new() -> Self {
        Self {
            languages: HashMap::new(),
        }
    }

    /// Register a language configuration
    pub fn register(&mut self, config: LanguageConfig) -> ConfigResult<()> {
        config.validate()?;

        // Validate all diagnostic rules
        for rule in &config.diagnostic_rules {
            rule.validate()?;
        }

        // Validate all code action templates
        for action in &config.code_actions {
            action.validate()?;
        }

        self.languages.insert(config.language.clone(), config);
        Ok(())
    }

    /// Get a language configuration by identifier
    pub fn get(&self, language: &str) -> Option<&LanguageConfig> {
        self.languages.get(language)
    }

    /// Get a language configuration by file extension
    pub fn get_by_extension(&self, extension: &str) -> Option<&LanguageConfig> {
        self.languages
            .values()
            .find(|config| config.extensions.contains(&extension.to_string()))
    }

    /// List all registered languages
    pub fn languages(&self) -> Vec<&str> {
        self.languages.keys().map(|s| s.as_str()).collect()
    }

    /// Check if a language is configured
    pub fn has_language(&self, language: &str) -> bool {
        self.languages.contains_key(language)
    }
}

impl Default for ConfigRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_config_validation() {
        let config = LanguageConfig {
            language: "rust".to_string(),
            extensions: vec!["rs".to_string()],
            parser_plugin: Some("tree-sitter-rust".to_string()),
            diagnostic_rules: vec![],
            code_actions: vec![],
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_language_config_validation_empty_name() {
        let config = LanguageConfig {
            language: String::new(),
            extensions: vec!["rs".to_string()],
            parser_plugin: None,
            diagnostic_rules: vec![],
            code_actions: vec![],
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_diagnostic_rule_validation() {
        let rule = DiagnosticRule {
            name: "unused-import".to_string(),
            pattern: "use .*".to_string(),
            severity: "warning".to_string(),
            message: "Unused import".to_string(),
            fix_template: None,
            code: Some("unused-import".to_string()),
        };

        assert!(rule.validate().is_ok());
    }

    #[test]
    fn test_config_registry() {
        let mut registry = ConfigRegistry::new();

        let config = LanguageConfig {
            language: "rust".to_string(),
            extensions: vec!["rs".to_string()],
            parser_plugin: Some("tree-sitter-rust".to_string()),
            diagnostic_rules: vec![],
            code_actions: vec![],
        };

        assert!(registry.register(config).is_ok());
        assert!(registry.has_language("rust"));
        assert!(registry.get("rust").is_some());
        assert!(registry.get_by_extension("rs").is_some());
    }
}
