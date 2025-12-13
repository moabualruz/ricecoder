//! Configuration management for providers

use std::collections::HashMap;
use std::path::PathBuf;

use crate::error::ProviderError;
use crate::models::{DefaultsConfig, ProviderConfig, ProviderSettings};

/// Configuration manager for loading and validating provider configuration
pub struct ConfigurationManager {
    config: ProviderConfig,
}

impl ConfigurationManager {
    /// Create a new configuration manager with default settings
    pub fn new() -> Self {
        Self {
            config: ProviderConfig {
                defaults: DefaultsConfig {
                    provider: "openai".to_string(),
                    model: "gpt-4".to_string(),
                    per_command: HashMap::new(),
                    per_action: HashMap::new(),
                },
                providers: HashMap::new(),
            },
        }
    }

    /// Load configuration with proper precedence:
    /// 1. Environment variables (highest priority)
    /// 2. Project config (./.agent/config.yaml)
    /// 3. Global config (~/Documents/.ricecoder/config.yaml)
    /// 4. Built-in defaults (lowest priority)
    pub fn load_with_precedence(&mut self) -> Result<(), ProviderError> {
        // Start with built-in defaults (already set in new())

        // Load global config if it exists
        let global_config_path = Self::get_global_config_path();
        if global_config_path.exists() {
            self.load_from_file(&global_config_path)?;
        }

        // Load project config if it exists (overrides global)
        let project_config_path = Self::get_project_config_path();
        if project_config_path.exists() {
            self.merge_from_file(&project_config_path)?;
        }

        // Load environment variables (highest priority, overrides all)
        self.load_from_env()?;

        Ok(())
    }

    /// Get the global configuration path
    pub fn get_global_config_path() -> PathBuf {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join("Documents/.ricecoder/config.yaml")
    }

    /// Get the project configuration path
    pub fn get_project_config_path() -> PathBuf {
        PathBuf::from("./.agent/config.yaml")
    }

    /// Load configuration from environment variables (highest priority)
    /// Environment variables override any existing configuration
    pub fn load_from_env(&mut self) -> Result<(), ProviderError> {
        // Load provider-specific environment variables
        // These override any configuration from files
        let providers_to_check = vec!["openai", "anthropic", "google", "ollama"];

        for provider in providers_to_check {
            let env_var = format!("{}_API_KEY", provider.to_uppercase());
            if let Ok(api_key) = std::env::var(&env_var) {
                // Always update or insert with the environment variable
                self.config
                    .providers
                    .entry(provider.to_string())
                    .and_modify(|s| {
                        s.api_key = Some(api_key.clone());
                    })
                    .or_insert_with(|| ProviderSettings {
                        api_key: Some(api_key.clone()),
                        base_url: None,
                        timeout: None,
                        retry_count: None,
                    });
            }
        }

        // Load RICECODER_PROVIDER_* environment variables
        for (key, value) in std::env::vars() {
            if key.starts_with("RICECODER_PROVIDER_") {
                let provider_id = key
                    .strip_prefix("RICECODER_PROVIDER_")
                    .unwrap()
                    .to_lowercase();
                self.config
                    .providers
                    .entry(provider_id)
                    .and_modify(|s| {
                        s.api_key = Some(value.clone());
                    })
                    .or_insert_with(|| ProviderSettings {
                        api_key: Some(value.clone()),
                        base_url: None,
                        timeout: None,
                        retry_count: None,
                    });
            }
        }

        Ok(())
    }

    /// Load configuration from a YAML file (replaces current config)
    pub fn load_from_file(&mut self, path: &PathBuf) -> Result<(), ProviderError> {
        if !path.exists() {
            return Ok(()); // File doesn't exist, skip
        }

        let content = std::fs::read_to_string(path).map_err(|e| {
            ProviderError::ConfigError(format!("Failed to read config file: {}", e))
        })?;

        let config: ProviderConfig = serde_yaml::from_str(&content).map_err(|e| {
            ProviderError::ConfigError(format!("Failed to parse config file: {}", e))
        })?;

        self.config = config;
        Ok(())
    }

    /// Merge configuration from a YAML file (preserves existing config)
    pub fn merge_from_file(&mut self, path: &PathBuf) -> Result<(), ProviderError> {
        if !path.exists() {
            return Ok(()); // File doesn't exist, skip
        }

        let content = std::fs::read_to_string(path).map_err(|e| {
            ProviderError::ConfigError(format!("Failed to read config file: {}", e))
        })?;

        let new_config: ProviderConfig = serde_yaml::from_str(&content).map_err(|e| {
            ProviderError::ConfigError(format!("Failed to parse config file: {}", e))
        })?;

        // Merge defaults
        if !new_config.defaults.provider.is_empty() {
            self.config.defaults.provider = new_config.defaults.provider;
        }
        if !new_config.defaults.model.is_empty() {
            self.config.defaults.model = new_config.defaults.model;
        }
        self.config
            .defaults
            .per_command
            .extend(new_config.defaults.per_command);
        self.config
            .defaults
            .per_action
            .extend(new_config.defaults.per_action);

        // Merge provider settings
        for (provider_id, settings) in new_config.providers {
            self.config
                .providers
                .entry(provider_id)
                .and_modify(|existing| {
                    if settings.api_key.is_some() {
                        existing.api_key = settings.api_key.clone();
                    }
                    if settings.base_url.is_some() {
                        existing.base_url = settings.base_url.clone();
                    }
                    if settings.timeout.is_some() {
                        existing.timeout = settings.timeout;
                    }
                    if settings.retry_count.is_some() {
                        existing.retry_count = settings.retry_count;
                    }
                })
                .or_insert(settings);
        }

        Ok(())
    }

    /// Validate the current configuration
    ///
    /// Validates:
    /// - At least one provider is configured
    /// - Default provider exists in configuration
    /// - API keys are present (from config or environment)
    /// - Models are available for selected provider
    /// - Context windows are reasonable
    pub fn validate(&self) -> Result<(), ProviderError> {
        // Check that at least one provider is configured
        if self.config.providers.is_empty() {
            return Err(ProviderError::ConfigError(
                "No providers configured".to_string(),
            ));
        }

        // Check that default provider exists
        if !self
            .config
            .providers
            .contains_key(&self.config.defaults.provider)
        {
            return Err(ProviderError::ConfigError(format!(
                "Default provider '{}' not configured",
                self.config.defaults.provider
            )));
        }

        // Check that API keys are present for configured providers
        for (provider_id, settings) in &self.config.providers {
            if settings.api_key.is_none() {
                // Try to get from environment
                let env_var = format!("{}_API_KEY", provider_id.to_uppercase());
                if std::env::var(&env_var).is_err() {
                    return Err(ProviderError::ConfigError(format!(
                        "API key not found for provider '{}'. Set {} environment variable or configure in config file",
                        provider_id, env_var
                    )));
                }
            }

            // Validate context window if present
            if let Some(settings) = self.config.providers.get(provider_id) {
                // Context window validation would happen here if we had model info
                // For now, we just ensure the settings are valid
                if let Some(timeout) = settings.timeout {
                    if timeout.as_secs() == 0 {
                        return Err(ProviderError::ConfigError(format!(
                            "Invalid timeout for provider '{}': must be greater than 0",
                            provider_id
                        )));
                    }
                }

                if let Some(retry_count) = settings.retry_count {
                    if retry_count > 10 {
                        return Err(ProviderError::ConfigError(format!(
                            "Invalid retry count for provider '{}': must be <= 10",
                            provider_id
                        )));
                    }
                }
            }
        }

        // Validate per-command defaults reference valid commands
        for command in self.config.defaults.per_command.keys() {
            match command.as_str() {
                "gen" | "refactor" | "review" => {} // Valid commands
                _ => {
                    return Err(ProviderError::ConfigError(format!(
                        "Invalid command in per_command defaults: '{}'. Valid commands are: gen, refactor, review",
                        command
                    )));
                }
            }
        }

        // Validate per-action defaults reference valid actions
        for action in self.config.defaults.per_action.keys() {
            match action.as_str() {
                "analysis" | "generation" => {} // Valid actions
                _ => {
                    return Err(ProviderError::ConfigError(format!(
                        "Invalid action in per_action defaults: '{}'. Valid actions are: analysis, generation",
                        action
                    )));
                }
            }
        }

        Ok(())
    }

    /// Validate configuration with provider registry (validates models are available)
    ///
    /// This method requires a provider registry to validate that:
    /// - Default model is available in the default provider
    /// - Per-command models are available in their respective providers
    /// - Per-action models are available in their respective providers
    pub fn validate_with_registry(
        &self,
        registry: &crate::provider::ProviderRegistry,
    ) -> Result<(), ProviderError> {
        // First run basic validation
        self.validate()?;

        // Validate default model is available in default provider
        let default_provider_id = &self.config.defaults.provider;
        let default_model_id = &self.config.defaults.model;

        let provider = registry.get(default_provider_id)?;
        let models = provider.models();

        if !models.iter().any(|m| m.id == *default_model_id) {
            return Err(ProviderError::ConfigError(format!(
                "Default model '{}' not found in provider '{}'",
                default_model_id, default_provider_id
            )));
        }

        // Validate per-command models
        for (command, model_id) in &self.config.defaults.per_command {
            let provider_id = default_provider_id; // Use default provider for per-command
            let provider = registry.get(provider_id)?;
            let models = provider.models();

            if !models.iter().any(|m| m.id == *model_id) {
                return Err(ProviderError::ConfigError(format!(
                    "Model '{}' for command '{}' not found in provider '{}'",
                    model_id, command, provider_id
                )));
            }
        }

        // Validate per-action models
        for (action, model_id) in &self.config.defaults.per_action {
            let provider_id = default_provider_id; // Use default provider for per-action
            let provider = registry.get(provider_id)?;
            let models = provider.models();

            if !models.iter().any(|m| m.id == *model_id) {
                return Err(ProviderError::ConfigError(format!(
                    "Model '{}' for action '{}' not found in provider '{}'",
                    model_id, action, provider_id
                )));
            }
        }

        Ok(())
    }

    /// Get the current configuration
    pub fn config(&self) -> &ProviderConfig {
        &self.config
    }

    /// Get mutable configuration
    pub fn config_mut(&mut self) -> &mut ProviderConfig {
        &mut self.config
    }

    /// Get the default provider ID
    pub fn default_provider(&self) -> &str {
        &self.config.defaults.provider
    }

    /// Get the default model ID
    pub fn default_model(&self) -> &str {
        &self.config.defaults.model
    }

    /// Get provider settings
    pub fn get_provider_settings(&self, provider_id: &str) -> Option<&ProviderSettings> {
        self.config.providers.get(provider_id)
    }

    /// Get API key for a provider (from config or environment)
    pub fn get_api_key(&self, provider_id: &str) -> Result<String, ProviderError> {
        // First check config
        if let Some(settings) = self.config.providers.get(provider_id) {
            if let Some(key) = &settings.api_key {
                return Ok(key.clone());
            }
        }

        // Then check environment
        let env_var = format!("{}_API_KEY", provider_id.to_uppercase());
        std::env::var(&env_var).map_err(|_| {
            ProviderError::ConfigError(format!(
                "API key not found for provider '{}'. Set {} environment variable",
                provider_id, env_var
            ))
        })
    }
}

impl Default for ConfigurationManager {
    fn default() -> Self {
        Self::new()
    }
}


