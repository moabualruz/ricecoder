//! Configuration file loader supporting multiple formats and sources
//!
//! This module provides loading of configuration files in YAML, TOML, JSON, and JSONC formats.
//! It supports CLI arguments, environment variables, and schema validation with priority merging:
//! CLI > Environment > Project > User > Global > Defaults

use std::path::{Path, PathBuf};

use super::{CliArgs, Config, ConfigMerger, EnvOverrides};
use crate::{
    error::{StorageError, StorageResult},
    manager::PathResolver,
    types::ConfigFormat,
};

/// Configuration loader for multiple formats and sources
pub struct ConfigLoader {
    /// CLI arguments (highest priority)
    cli_args: Option<CliArgs>,
    /// Skip validation flag
    skip_validation: bool,
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigLoader {
    /// Create a new configuration loader
    pub fn new() -> Self {
        Self {
            cli_args: None,
            skip_validation: false,
        }
    }

    /// Set CLI arguments for highest priority overrides
    pub fn with_cli_args(mut self, args: CliArgs) -> Self {
        self.cli_args = Some(args);
        self
    }

    /// Skip configuration validation
    pub fn skip_validation(mut self, skip: bool) -> Self {
        self.skip_validation = skip;
        self
    }

    /// Load and merge configuration from all sources
    ///
    /// Loads configuration from multiple sources with the following priority:
    /// 1. CLI arguments (highest)
    /// 2. Environment variable overrides (`RICECODER_*`)
    /// 3. Project config (`./.ricecoder/ricecoder.yaml`)
    /// 4. User config (`~/.ricecoder/ricecoder.yaml`)
    /// 5. Global config (`~/Documents/.ricecoder/ricecoder.yaml`)
    /// 6. Built-in defaults (lowest)
    ///
    /// Returns the merged configuration. If no configuration files exist,
    /// returns the built-in defaults.
    pub fn load_merged(self) -> StorageResult<Config> {
        // Start with built-in defaults
        let defaults = Config::default();

        // Load global config if it exists
        let global_config = Self::load_global_config().ok();

        // Load user config if it exists
        let user_config = Self::load_user_config().ok();

        // Load project config if it exists
        let project_config = Self::load_project_config().ok();

        // Parse environment variable overrides
        let mut env_config = Config::default();
        EnvOverrides::apply(&mut env_config);

        // Apply CLI overrides (highest priority)
        let cli_config = self.cli_args.as_ref().map(Self::cli_args_to_config);

        // Merge all configurations with proper precedence: CLI > Env > Project > User > Global > Defaults
        let (mut merged, _decisions) = ConfigMerger::merge_with_cli(
            defaults,
            global_config,
            user_config,
            project_config,
            Some(env_config),
            cli_config,
        );

        // Substitute environment variables in config values
        Self::substitute_env_vars(&mut merged)?;

        // Apply configuration migrations if needed
        // TODO: Load version information from config and apply migrations

        // Validate configuration against schemas
        if !self.skip_validation {
            let mut validator = crate::config::validation::ConfigValidator::new();
            validator.load_builtin_schemas()?;
            validator.validate(&merged)?;
        }

        Ok(merged)
    }

    /// Load global configuration from `~/Documents/.ricecoder/ricecoder.yaml`
    fn load_global_config() -> StorageResult<Config> {
        let global_path = PathResolver::resolve_global_path()?;
        let config_file = global_path.join("ricecoder.yaml");

        if config_file.exists() {
            Self::load_from_file(&config_file)
        } else {
            Ok(Config::default())
        }
    }

    /// Load user configuration from `~/.ricecoder/ricecoder.yaml`
    fn load_user_config() -> StorageResult<Config> {
        let user_config_dir = dirs::home_dir().ok_or_else(|| {
            StorageError::Internal("Could not determine home directory".to_string())
        })?;
        let config_file = user_config_dir.join(".ricecoder").join("ricecoder.yaml");

        if config_file.exists() {
            Self::load_from_file(&config_file)
        } else {
            Ok(Config::default())
        }
    }

    /// Load project configuration from `./.ricecoder/ricecoder.yaml`
    fn load_project_config() -> StorageResult<Config> {
        let project_config_file = PathBuf::from(".ricecoder/ricecoder.yaml");

        if project_config_file.exists() {
            Self::load_from_file(&project_config_file)
        } else {
            Ok(Config::default())
        }
    }

    /// Substitute `${VAR_NAME}` patterns in configuration values with environment variables
    ///
    /// Replaces patterns like `${OPENAI_API_KEY}` with the corresponding environment
    /// variable value. If the environment variable is not set, replaces with empty string.
    pub fn substitute_env_vars(config: &mut Config) -> StorageResult<()> {
        let re = regex::Regex::new(r"\$\{([^}]+)\}")
            .map_err(|e| StorageError::Internal(format!("Invalid regex pattern: {}", e)))?;

        // Substitute in API keys
        for (_, value) in config.providers.api_keys.iter_mut() {
            if re.is_match(value) {
                let substituted = re
                    .replace_all(value, |caps: &regex::Captures| {
                        let var_name = &caps[1];
                        std::env::var(var_name).unwrap_or_default()
                    })
                    .to_string();
                *value = substituted;
            }
        }

        // Substitute in endpoints
        for (_, value) in config.providers.endpoints.iter_mut() {
            if re.is_match(value) {
                let substituted = re
                    .replace_all(value, |caps: &regex::Captures| {
                        let var_name = &caps[1];
                        std::env::var(var_name).unwrap_or_default()
                    })
                    .to_string();
                *value = substituted;
            }
        }

        // Substitute in custom settings
        for (_, value) in config.custom.iter_mut() {
            if let serde_json::Value::String(s) = value {
                if re.is_match(s) {
                    let substituted = re
                        .replace_all(s, |caps: &regex::Captures| {
                            let var_name = &caps[1];
                            std::env::var(var_name).unwrap_or_default()
                        })
                        .to_string();
                    *value = serde_json::Value::String(substituted);
                }
            }
        }

        Ok(())
    }

    /// Convert CLI arguments to configuration
    pub fn cli_args_to_config(cli_args: &CliArgs) -> Config {
        let mut config = Config::default();

        if let Some(ref provider) = cli_args.provider {
            config.providers.default_provider = Some(provider.clone());
        }

        if let Some(ref model) = cli_args.model {
            config.defaults.model = Some(model.clone());
        }

        if let Some(ref api_key) = cli_args.api_key {
            // For CLI args, we assume the provider is the default or specified
            let provider = cli_args.provider.as_deref().unwrap_or("openai");
            config
                .providers
                .api_keys
                .insert(provider.to_string(), api_key.clone());
        }

        if let Some(temp) = cli_args.temperature {
            config.defaults.temperature = Some(temp as f32);
        }

        if let Some(tokens) = cli_args.max_tokens {
            config.defaults.max_tokens = Some(tokens);
        }

        // Theme and other settings would be added here when the config structure supports them

        config
    }

    /// Load configuration from a file
    ///
    /// Automatically detects format based on file extension.
    /// Supports YAML (.yaml, .yml), TOML (.toml), and JSON (.json) formats.
    pub fn load_from_file(path: &Path) -> StorageResult<Config> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            StorageError::io_error(path.to_path_buf(), crate::error::IoOperation::Read, e)
        })?;

        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| {
                StorageError::parse_error(path.to_path_buf(), "unknown", "File has no extension")
            })?;

        let format = ConfigFormat::from_extension(extension).ok_or_else(|| {
            StorageError::parse_error(
                path.to_path_buf(),
                "unknown",
                format!("Unsupported file format: {}", extension),
            )
        })?;

        Self::load_from_string(&content, format, path)
    }

    /// Load configuration from a string with specified format
    pub fn load_from_string(
        content: &str,
        format: ConfigFormat,
        path: &Path,
    ) -> StorageResult<Config> {
        match format {
            ConfigFormat::Yaml => Self::parse_yaml(content, path),
            ConfigFormat::Toml => Self::parse_toml(content, path),
            ConfigFormat::Json => Self::parse_json(content, path),
            ConfigFormat::Jsonc => Self::parse_jsonc(content, path),
        }
    }

    /// Parse YAML content
    fn parse_yaml(content: &str, path: &Path) -> StorageResult<Config> {
        serde_yaml::from_str(content)
            .map_err(|e| StorageError::parse_error(path.to_path_buf(), "YAML", e.to_string()))
    }

    /// Parse TOML content
    fn parse_toml(content: &str, path: &Path) -> StorageResult<Config> {
        toml::from_str(content)
            .map_err(|e| StorageError::parse_error(path.to_path_buf(), "TOML", e.to_string()))
    }

    /// Parse JSON content
    fn parse_json(content: &str, path: &Path) -> StorageResult<Config> {
        serde_json::from_str(content)
            .map_err(|e| StorageError::parse_error(path.to_path_buf(), "JSON", e.to_string()))
    }

    /// Parse JSONC content (JSON with comments)
    /// TODO: Implement proper JSONC parsing with comment stripping
    fn parse_jsonc(content: &str, path: &Path) -> StorageResult<Config> {
        // For now, treat JSONC as regular JSON (comments not supported yet)
        serde_json::from_str(content)
            .map_err(|e| StorageError::parse_error(path.to_path_buf(), "JSONC", e.to_string()))
    }

    /// Serialize configuration to string in specified format
    pub fn serialize(config: &Config, format: ConfigFormat) -> StorageResult<String> {
        match format {
            ConfigFormat::Yaml => serde_yaml::to_string(config)
                .map_err(|e| StorageError::Internal(format!("Failed to serialize to YAML: {}", e))),
            ConfigFormat::Toml => toml::to_string_pretty(config)
                .map_err(|e| StorageError::Internal(format!("Failed to serialize to TOML: {}", e))),
            ConfigFormat::Json => serde_json::to_string_pretty(config)
                .map_err(|e| StorageError::Internal(format!("Failed to serialize to JSON: {}", e))),
            ConfigFormat::Jsonc => serde_json::to_string_pretty(config).map_err(|e| {
                StorageError::Internal(format!("Failed to serialize to JSONC: {}", e))
            }),
        }
    }

    /// Save configuration to a file
    pub fn save_to_file(config: &Config, path: &Path, format: ConfigFormat) -> StorageResult<()> {
        let content = Self::serialize(config, format)?;
        std::fs::write(path, content).map_err(|e| {
            StorageError::io_error(path.to_path_buf(), crate::error::IoOperation::Write, e)
        })
    }
}
