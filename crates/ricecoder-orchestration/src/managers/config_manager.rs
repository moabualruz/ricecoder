//! Configuration management for workspace orchestration
//!
//! Handles loading and applying workspace-level configuration with support for
//! configuration hierarchy (workspace → project → user → defaults) and validation.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{
    error::{OrchestrationError, Result},
    models::{RuleType, WorkspaceConfig, WorkspaceRule},
};

/// Configuration manager for workspace orchestration
///
/// Loads and applies workspace-level configuration with support for:
/// - Configuration hierarchy (workspace → project → user → defaults)
/// - Configuration validation against schema
/// - Runtime configuration updates
#[derive(Debug, Clone)]
pub struct ConfigManager {
    /// Workspace root path
    workspace_root: PathBuf,

    /// Loaded configuration
    config: WorkspaceConfig,

    /// Configuration schema for validation
    schema: ConfigSchema,
}

/// Schema for validating workspace configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSchema {
    /// Required configuration keys
    pub required_keys: Vec<String>,

    /// Optional configuration keys with defaults
    pub optional_keys: HashMap<String, Value>,

    /// Validation rules for configuration values
    pub validation_rules: HashMap<String, ValidationRule>,
}

/// Validation rule for a configuration value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    /// Rule type (e.g., "string", "number", "array")
    pub rule_type: String,

    /// Minimum value (for numbers)
    pub min: Option<f64>,

    /// Maximum value (for numbers)
    pub max: Option<f64>,

    /// Allowed values (for enums)
    pub allowed_values: Option<Vec<String>>,

    /// Pattern for string validation (regex)
    pub pattern: Option<String>,
}

/// Configuration source priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConfigSource {
    /// Built-in defaults (lowest priority)
    Defaults = 0,

    /// User-level configuration (~/.ricecoder/config.yaml)
    User = 1,

    /// Project-level configuration (.ricecoder/project.yaml)
    Project = 2,

    /// Workspace-level configuration (.ricecoder/workspace.yaml)
    Workspace = 3,

    /// Runtime overrides (highest priority)
    Runtime = 4,
}

/// Configuration load result with source tracking
#[derive(Debug, Clone)]
pub struct ConfigLoadResult {
    /// Loaded configuration
    pub config: WorkspaceConfig,

    /// Source of each configuration value
    pub sources: HashMap<String, ConfigSource>,

    /// Warnings during configuration loading
    pub warnings: Vec<String>,
}

impl ConfigManager {
    /// Creates a new configuration manager
    pub fn new(workspace_root: PathBuf) -> Self {
        Self {
            workspace_root,
            config: WorkspaceConfig::default(),
            schema: ConfigSchema::default(),
        }
    }

    /// Loads configuration from the configuration hierarchy
    ///
    /// Configuration is loaded in priority order:
    /// 1. Built-in defaults
    /// 2. User-level configuration (~/.ricecoder/config.yaml)
    /// 3. Project-level configuration (.ricecoder/project.yaml)
    /// 4. Workspace-level configuration (.ricecoder/workspace.yaml)
    ///
    /// Later sources override earlier sources.
    pub async fn load_configuration(&mut self) -> Result<ConfigLoadResult> {
        let mut config = WorkspaceConfig::default();
        let mut sources = HashMap::new();
        let warnings = Vec::new();

        // Load defaults
        let defaults = self.load_defaults();
        config = self.merge_configs(config, defaults.clone());
        for key in defaults
            .settings
            .as_object()
            .unwrap_or(&Default::default())
            .keys()
        {
            sources.insert(key.clone(), ConfigSource::Defaults);
        }

        // Load user-level configuration
        if let Ok(user_config) = self.load_user_config().await {
            config = self.merge_configs(config, user_config.clone());
            for key in user_config
                .settings
                .as_object()
                .unwrap_or(&Default::default())
                .keys()
            {
                sources.insert(key.clone(), ConfigSource::User);
            }
        }

        // Load project-level configuration
        if let Ok(project_config) = self.load_project_config().await {
            config = self.merge_configs(config, project_config.clone());
            for key in project_config
                .settings
                .as_object()
                .unwrap_or(&Default::default())
                .keys()
            {
                sources.insert(key.clone(), ConfigSource::Project);
            }
        }

        // Load workspace-level configuration
        if let Ok(workspace_config) = self.load_workspace_config().await {
            config = self.merge_configs(config, workspace_config.clone());
            for key in workspace_config
                .settings
                .as_object()
                .unwrap_or(&Default::default())
                .keys()
            {
                sources.insert(key.clone(), ConfigSource::Workspace);
            }
        }

        // Validate configuration
        self.validate_config(&config)?;

        self.config = config.clone();

        Ok(ConfigLoadResult {
            config,
            sources,
            warnings,
        })
    }

    /// Loads default configuration
    fn load_defaults(&self) -> WorkspaceConfig {
        WorkspaceConfig {
            rules: vec![
                WorkspaceRule {
                    name: "no-circular-deps".to_string(),
                    rule_type: RuleType::DependencyConstraint,
                    enabled: true,
                },
                WorkspaceRule {
                    name: "naming-convention".to_string(),
                    rule_type: RuleType::NamingConvention,
                    enabled: true,
                },
            ],
            settings: json!({
                "max_parallel_operations": 4,
                "transaction_timeout_ms": 30000,
                "enable_audit_logging": true,
            }),
        }
    }

    /// Loads user-level configuration from ~/.ricecoder/config.yaml
    async fn load_user_config(&self) -> Result<WorkspaceConfig> {
        let user_home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| {
                crate::error::OrchestrationError::ConfigurationError(
                    "Could not determine user home directory".to_string(),
                )
            })?;

        let config_path = PathBuf::from(user_home)
            .join(".ricecoder")
            .join("config.yaml");

        if !config_path.exists() {
            return Err(OrchestrationError::ConfigurationError(format!(
                "User config not found: {}",
                config_path.display()
            )));
        }

        self.load_config_from_file(&config_path).await
    }

    /// Loads project-level configuration from .ricecoder/project.yaml
    async fn load_project_config(&self) -> Result<WorkspaceConfig> {
        let config_path = self.workspace_root.join(".ricecoder").join("project.yaml");

        if !config_path.exists() {
            return Err(crate::error::OrchestrationError::ConfigurationError(
                format!("Project config not found: {}", config_path.display()),
            ));
        }

        self.load_config_from_file(&config_path).await
    }

    /// Loads workspace-level configuration from .ricecoder/workspace.yaml
    async fn load_workspace_config(&self) -> Result<WorkspaceConfig> {
        let config_path = self
            .workspace_root
            .join(".ricecoder")
            .join("workspace.yaml");

        if !config_path.exists() {
            return Err(crate::error::OrchestrationError::ConfigurationError(
                format!("Workspace config not found: {}", config_path.display()),
            ));
        }

        self.load_config_from_file(&config_path).await
    }

    /// Loads configuration from a YAML file
    async fn load_config_from_file(&self, path: &Path) -> Result<WorkspaceConfig> {
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(crate::error::OrchestrationError::IoError)?;

        let config: WorkspaceConfig = serde_yaml::from_str(&content)?;

        Ok(config)
    }

    /// Merges two configurations, with the second overriding the first
    pub fn merge_configs(
        &self,
        mut base: WorkspaceConfig,
        override_config: WorkspaceConfig,
    ) -> WorkspaceConfig {
        // Merge rules
        for rule in override_config.rules {
            if let Some(pos) = base.rules.iter().position(|r| r.name == rule.name) {
                base.rules[pos] = rule;
            } else {
                base.rules.push(rule);
            }
        }

        // Merge settings
        if let (Some(base_obj), Some(override_obj)) = (
            base.settings.as_object_mut(),
            override_config.settings.as_object(),
        ) {
            for (key, value) in override_obj {
                base_obj.insert(key.clone(), value.clone());
            }
        }

        base
    }

    /// Validates configuration against the schema
    fn validate_config(&self, config: &WorkspaceConfig) -> Result<()> {
        // Validate rules
        for rule in &config.rules {
            if rule.name.is_empty() {
                return Err(crate::error::OrchestrationError::ConfigurationError(
                    "Rule name cannot be empty".to_string(),
                ));
            }
        }

        // Validate settings
        if let Some(settings_obj) = config.settings.as_object() {
            for (key, value) in settings_obj {
                if let Some(validation_rule) = self.schema.validation_rules.get(key) {
                    self.validate_value(key, value, validation_rule)?;
                }
            }
        }

        Ok(())
    }

    /// Validates a configuration value against a validation rule
    pub fn validate_value(&self, key: &str, value: &Value, rule: &ValidationRule) -> Result<()> {
        match rule.rule_type.as_str() {
            "number" => {
                if let Some(num) = value.as_f64() {
                    if let Some(min) = rule.min {
                        if num < min {
                            return Err(crate::error::OrchestrationError::ConfigurationError(
                                format!("{} must be >= {}", key, min),
                            ));
                        }
                    }
                    if let Some(max) = rule.max {
                        if num > max {
                            return Err(crate::error::OrchestrationError::ConfigurationError(
                                format!("{} must be <= {}", key, max),
                            ));
                        }
                    }
                } else {
                    return Err(crate::error::OrchestrationError::ConfigurationError(
                        format!("{} must be a number", key),
                    ));
                }
            }
            "string" => {
                if !value.is_string() {
                    return Err(crate::error::OrchestrationError::ConfigurationError(
                        format!("{} must be a string", key),
                    ));
                }
            }
            "boolean" => {
                if !value.is_boolean() {
                    return Err(crate::error::OrchestrationError::ConfigurationError(
                        format!("{} must be a boolean", key),
                    ));
                }
            }
            "array" => {
                if !value.is_array() {
                    return Err(crate::error::OrchestrationError::ConfigurationError(
                        format!("{} must be an array", key),
                    ));
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Gets the current configuration
    pub fn get_config(&self) -> &WorkspaceConfig {
        &self.config
    }

    /// Gets a configuration setting by key
    pub fn get_setting(&self, key: &str) -> Option<&Value> {
        self.config.settings.get(key)
    }

    /// Sets a configuration setting at runtime
    pub fn set_setting(&mut self, key: String, value: Value) -> Result<()> {
        if let Some(obj) = self.config.settings.as_object_mut() {
            obj.insert(key, value);
            Ok(())
        } else {
            Err(crate::error::OrchestrationError::ConfigurationError(
                "Settings is not an object".to_string(),
            ))
        }
    }

    /// Gets all rules
    pub fn get_rules(&self) -> &[WorkspaceRule] {
        &self.config.rules
    }

    /// Gets a rule by name
    pub fn get_rule(&self, name: &str) -> Option<&WorkspaceRule> {
        self.config.rules.iter().find(|r| r.name == name)
    }

    /// Enables a rule by name
    pub fn enable_rule(&mut self, name: &str) -> Result<()> {
        if let Some(rule) = self.config.rules.iter_mut().find(|r| r.name == name) {
            rule.enabled = true;
            Ok(())
        } else {
            Err(crate::error::OrchestrationError::RulesValidationFailed(
                format!("Rule not found: {}", name),
            ))
        }
    }

    /// Disables a rule by name
    pub fn disable_rule(&mut self, name: &str) -> Result<()> {
        if let Some(rule) = self.config.rules.iter_mut().find(|r| r.name == name) {
            rule.enabled = false;
            Ok(())
        } else {
            Err(crate::error::OrchestrationError::RulesValidationFailed(
                format!("Rule not found: {}", name),
            ))
        }
    }
}

impl Default for ConfigSchema {
    fn default() -> Self {
        let mut validation_rules = HashMap::new();

        validation_rules.insert(
            "max_parallel_operations".to_string(),
            ValidationRule {
                rule_type: "number".to_string(),
                min: Some(1.0),
                max: Some(32.0),
                allowed_values: None,
                pattern: None,
            },
        );

        validation_rules.insert(
            "transaction_timeout_ms".to_string(),
            ValidationRule {
                rule_type: "number".to_string(),
                min: Some(1000.0),
                max: Some(300000.0),
                allowed_values: None,
                pattern: None,
            },
        );

        validation_rules.insert(
            "enable_audit_logging".to_string(),
            ValidationRule {
                rule_type: "boolean".to_string(),
                min: None,
                max: None,
                allowed_values: None,
                pattern: None,
            },
        );

        Self {
            required_keys: vec![],
            optional_keys: HashMap::new(),
            validation_rules,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_manager_creation() {
        let manager = ConfigManager::new(PathBuf::from("/workspace"));
        assert_eq!(manager.workspace_root, PathBuf::from("/workspace"));
    }

    #[test]
    fn test_load_defaults() {
        let manager = ConfigManager::new(PathBuf::from("/workspace"));
        let defaults = manager.load_defaults();

        assert!(!defaults.rules.is_empty());
        assert!(defaults.settings.get("max_parallel_operations").is_some());
    }

    #[test]
    fn test_merge_configs() {
        let manager = ConfigManager::new(PathBuf::from("/workspace"));

        let base = WorkspaceConfig {
            rules: vec![WorkspaceRule {
                name: "rule1".to_string(),
                rule_type: RuleType::DependencyConstraint,
                enabled: true,
            }],
            settings: json!({"key1": "value1"}),
        };

        let override_config = WorkspaceConfig {
            rules: vec![WorkspaceRule {
                name: "rule2".to_string(),
                rule_type: RuleType::NamingConvention,
                enabled: false,
            }],
            settings: json!({"key2": "value2"}),
        };

        let merged = manager.merge_configs(base, override_config);

        assert_eq!(merged.rules.len(), 2);
        assert!(merged.settings.get("key1").is_some());
        assert!(merged.settings.get("key2").is_some());
    }

    #[test]
    fn test_validate_config_success() {
        let manager = ConfigManager::new(PathBuf::from("/workspace"));
        let config = WorkspaceConfig {
            rules: vec![WorkspaceRule {
                name: "test-rule".to_string(),
                rule_type: RuleType::DependencyConstraint,
                enabled: true,
            }],
            settings: json!({"max_parallel_operations": 4}),
        };

        assert!(manager.validate_config(&config).is_ok());
    }

    #[test]
    fn test_validate_config_empty_rule_name() {
        let manager = ConfigManager::new(PathBuf::from("/workspace"));
        let config = WorkspaceConfig {
            rules: vec![WorkspaceRule {
                name: "".to_string(),
                rule_type: RuleType::DependencyConstraint,
                enabled: true,
            }],
            settings: json!({}),
        };

        assert!(manager.validate_config(&config).is_err());
    }

    #[test]
    fn test_get_setting() {
        let mut manager = ConfigManager::new(PathBuf::from("/workspace"));
        manager.config = WorkspaceConfig {
            rules: vec![],
            settings: json!({"key1": "value1"}),
        };

        assert_eq!(
            manager.get_setting("key1"),
            Some(&Value::String("value1".to_string()))
        );
        assert_eq!(manager.get_setting("nonexistent"), None);
    }

    #[test]
    fn test_set_setting() {
        let mut manager = ConfigManager::new(PathBuf::from("/workspace"));
        manager.config = WorkspaceConfig {
            rules: vec![],
            settings: json!({}),
        };

        assert!(manager
            .set_setting("key1".to_string(), Value::String("value1".to_string()))
            .is_ok());
        assert_eq!(
            manager.get_setting("key1"),
            Some(&Value::String("value1".to_string()))
        );
    }

    #[test]
    fn test_get_rules() {
        let mut manager = ConfigManager::new(PathBuf::from("/workspace"));
        let rule = WorkspaceRule {
            name: "test-rule".to_string(),
            rule_type: RuleType::DependencyConstraint,
            enabled: true,
        };
        manager.config.rules.push(rule);

        assert_eq!(manager.get_rules().len(), 1);
    }

    #[test]
    fn test_get_rule_by_name() {
        let mut manager = ConfigManager::new(PathBuf::from("/workspace"));
        let rule = WorkspaceRule {
            name: "test-rule".to_string(),
            rule_type: RuleType::DependencyConstraint,
            enabled: true,
        };
        manager.config.rules.push(rule);

        assert!(manager.get_rule("test-rule").is_some());
        assert!(manager.get_rule("nonexistent").is_none());
    }

    #[test]
    fn test_enable_rule() {
        let mut manager = ConfigManager::new(PathBuf::from("/workspace"));
        let rule = WorkspaceRule {
            name: "test-rule".to_string(),
            rule_type: RuleType::DependencyConstraint,
            enabled: false,
        };
        manager.config.rules.push(rule);

        assert!(manager.enable_rule("test-rule").is_ok());
        assert!(manager.get_rule("test-rule").unwrap().enabled);
    }

    #[test]
    fn test_disable_rule() {
        let mut manager = ConfigManager::new(PathBuf::from("/workspace"));
        let rule = WorkspaceRule {
            name: "test-rule".to_string(),
            rule_type: RuleType::DependencyConstraint,
            enabled: true,
        };
        manager.config.rules.push(rule);

        assert!(manager.disable_rule("test-rule").is_ok());
        assert!(!manager.get_rule("test-rule").unwrap().enabled);
    }

    #[test]
    fn test_config_schema_default() {
        let schema = ConfigSchema::default();
        assert!(schema
            .validation_rules
            .contains_key("max_parallel_operations"));
        assert!(schema
            .validation_rules
            .contains_key("transaction_timeout_ms"));
        assert!(schema.validation_rules.contains_key("enable_audit_logging"));
    }

    #[test]
    fn test_validate_number_value() {
        let manager = ConfigManager::new(PathBuf::from("/workspace"));
        let rule = ValidationRule {
            rule_type: "number".to_string(),
            min: Some(1.0),
            max: Some(10.0),
            allowed_values: None,
            pattern: None,
        };

        assert!(manager
            .validate_value("test", &Value::Number(5.into()), &rule)
            .is_ok());
        assert!(manager
            .validate_value("test", &Value::Number(0.into()), &rule)
            .is_err());
        assert!(manager
            .validate_value("test", &Value::Number(11.into()), &rule)
            .is_err());
    }

    #[test]
    fn test_validate_string_value() {
        let manager = ConfigManager::new(PathBuf::from("/workspace"));
        let rule = ValidationRule {
            rule_type: "string".to_string(),
            min: None,
            max: None,
            allowed_values: None,
            pattern: None,
        };

        assert!(manager
            .validate_value("test", &Value::String("value".to_string()), &rule)
            .is_ok());
        assert!(manager
            .validate_value("test", &Value::Number(5.into()), &rule)
            .is_err());
    }

    #[test]
    fn test_validate_boolean_value() {
        let manager = ConfigManager::new(PathBuf::from("/workspace"));
        let rule = ValidationRule {
            rule_type: "boolean".to_string(),
            min: None,
            max: None,
            allowed_values: None,
            pattern: None,
        };

        assert!(manager
            .validate_value("test", &Value::Bool(true), &rule)
            .is_ok());
        assert!(manager
            .validate_value("test", &Value::String("true".to_string()), &rule)
            .is_err());
    }
}
