//! Configuration types for language-agnostic, configuration-driven architecture

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
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
            return Err(ConfigError::ValidationError(format!(
                "Language '{}' must have at least one file extension",
                self.language
            )));
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
                return Err(ConfigError::ValidationError(format!(
                    "Invalid severity level: {}",
                    self.severity
                )))
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
                return Err(ConfigError::ValidationError(format!(
                    "Invalid action kind: {}",
                    self.kind
                )))
            }
        }

        Ok(())
    }
}

/// Completion configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionConfig {
    /// Enable/disable completion
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Completion timeout in milliseconds
    #[serde(default = "default_completion_timeout")]
    pub timeout_ms: u64,

    /// Maximum number of completions to return
    #[serde(default = "default_max_completions")]
    pub max_completions: usize,

    /// Enable ghost text display
    #[serde(default = "default_true")]
    pub ghost_text_enabled: bool,

    /// Minimum prefix length to trigger completion
    #[serde(default = "default_min_prefix_length")]
    pub min_prefix_length: usize,

    /// Enable fuzzy matching
    #[serde(default = "default_true")]
    pub fuzzy_matching: bool,

    /// Enable frequency-based ranking
    #[serde(default = "default_true")]
    pub frequency_ranking: bool,

    /// Enable recency-based ranking
    #[serde(default = "default_true")]
    pub recency_ranking: bool,

    /// Language-specific completion providers
    #[serde(default)]
    pub providers: HashMap<String, String>,
}

impl Default for CompletionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            timeout_ms: 100,
            max_completions: 50,
            ghost_text_enabled: true,
            min_prefix_length: 1,
            fuzzy_matching: true,
            frequency_ranking: true,
            recency_ranking: true,
            providers: HashMap::new(),
        }
    }
}

impl CompletionConfig {
    /// Validate the completion configuration
    pub fn validate(&self) -> ConfigResult<()> {
        if self.timeout_ms == 0 {
            return Err(ConfigError::ValidationError(
                "Completion timeout must be greater than 0".to_string(),
            ));
        }

        if self.max_completions == 0 {
            return Err(ConfigError::ValidationError(
                "Max completions must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }
}

/// Default value for enabled fields
fn default_true() -> bool {
    true
}

/// Default completion timeout (100ms)
fn default_completion_timeout() -> u64 {
    100
}

/// Default max completions (50)
fn default_max_completions() -> usize {
    50
}

/// Default min prefix length (1)
fn default_min_prefix_length() -> usize {
    1
}

/// Configuration registry for managing multiple languages
pub struct ConfigRegistry {
    /// Language configurations by language identifier
    languages: HashMap<String, LanguageConfig>,

    /// Completion configuration
    completion_config: CompletionConfig,
}

impl ConfigRegistry {
    /// Create a new configuration registry
    pub fn new() -> Self {
        Self {
            languages: HashMap::new(),
            completion_config: CompletionConfig::default(),
        }
    }

    /// Create a new configuration registry with custom completion config
    pub fn with_completion_config(completion_config: CompletionConfig) -> ConfigResult<Self> {
        completion_config.validate()?;
        Ok(Self {
            languages: HashMap::new(),
            completion_config,
        })
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

    /// Get completion configuration
    pub fn completion_config(&self) -> &CompletionConfig {
        &self.completion_config
    }

    /// Get mutable completion configuration
    pub fn completion_config_mut(&mut self) -> &mut CompletionConfig {
        &mut self.completion_config
    }

    /// Set completion configuration
    pub fn set_completion_config(&mut self, config: CompletionConfig) -> ConfigResult<()> {
        config.validate()?;
        self.completion_config = config;
        Ok(())
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

    #[test]
    fn test_completion_config_default() {
        let config = CompletionConfig::default();
        assert!(config.enabled);
        assert_eq!(config.timeout_ms, 100);
        assert_eq!(config.max_completions, 50);
        assert!(config.ghost_text_enabled);
    }

    #[test]
    fn test_completion_config_validation() {
        let config = CompletionConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_completion_config_validation_invalid_timeout() {
        let mut config = CompletionConfig::default();
        config.timeout_ms = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_registry_completion_config() {
        let mut registry = ConfigRegistry::new();
        let completion_config = registry.completion_config();
        assert!(completion_config.enabled);

        let mut new_config = CompletionConfig::default();
        new_config.enabled = false;
        assert!(registry.set_completion_config(new_config).is_ok());
        assert!(!registry.completion_config().enabled);
    }
}
