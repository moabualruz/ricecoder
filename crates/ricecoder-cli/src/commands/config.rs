// Configuration management
// Adapted from automation/src/infrastructure/storage/config.rs

use super::Command;
use crate::error::{CliError, CliResult};
use crate::output::OutputStyle;
use ricecoder_storage::{ConfigLoader, PathResolver, Config};
use std::path::PathBuf;

/// Manage configuration
pub struct ConfigCommand {
    pub action: ConfigAction,
}

#[derive(Debug, Clone)]
pub enum ConfigAction {
    List,
    Get(String),
    Set(String, String),
}

impl ConfigCommand {
    pub fn new(action: ConfigAction) -> Self {
        Self { action }
    }

    /// Get the configuration directory path using PathResolver
    fn get_config_dir() -> CliResult<PathBuf> {
        PathResolver::resolve_global_path()
            .map_err(|e| CliError::Config(e.to_string()))
    }

    /// Load configuration from files using ConfigLoader
    fn load_config() -> CliResult<Config> {
        ConfigLoader::new().load_merged()
            .map_err(|e| CliError::Config(e.to_string()))
    }

    /// List all configuration values
    fn list_config(&self) -> CliResult<()> {
        let style = OutputStyle::default();
        let config = Self::load_config()?;

        println!("{}", style.header("RiceCoder Configuration"));
        println!();

        // Display provider configuration
        if let Some(provider) = &config.providers.default_provider {
            println!("  {} = {}", style.code("provider.default"), provider);
        } else {
            println!("  {} = {}", style.code("provider.default"), "(not set)");
        }

        // Display API keys (masked)
        for (provider, key) in &config.providers.api_keys {
            let masked = if key.is_empty() { "(not set)" } else { "***masked***" };
            println!(
                "  {} = {}",
                style.code(&format!("providers.{}.api_key", provider)),
                masked
            );
        }

        // Display model configuration
        if let Some(model) = &config.defaults.model {
            println!("  {} = {}", style.code("defaults.model"), model);
        } else {
            println!("  {} = {}", style.code("defaults.model"), "(not set)");
        }

        // Display temperature if set
        if let Some(temp) = config.defaults.temperature {
            println!("  {} = {}", style.code("defaults.temperature"), temp);
        }

        // Display max tokens if set
        if let Some(max_tokens) = config.defaults.max_tokens {
            println!("  {} = {}", style.code("defaults.max_tokens"), max_tokens);
        }

        println!();
        let config_dir = Self::get_config_dir()?;
        println!(
            "{}",
            style.info(&format!("Config directory: {}", config_dir.display()))
        );

        Ok(())
    }

    /// Get a specific configuration value
    fn get_config(&self, key: &str) -> CliResult<()> {
        let style = OutputStyle::default();
        let config = Self::load_config()?;

        let value = match key {
            "provider.default" => config.providers.default_provider.clone(),
            "defaults.model" => config.defaults.model.clone(),
            "defaults.temperature" => config.defaults.temperature.map(|t| t.to_string()),
            "defaults.max_tokens" => config.defaults.max_tokens.map(|t| t.to_string()),
            k if k.starts_with("providers.") && k.ends_with(".api_key") => {
                // Extract provider name from key like "providers.openai.api_key"
                let provider = k
                    .strip_prefix("providers.")
                    .and_then(|s| s.strip_suffix(".api_key"))
                    .unwrap_or("");
                config
                    .providers
                    .api_keys
                    .get(provider)
                    .map(|k| if k.is_empty() { "(not set)".to_string() } else { "***masked***".to_string() })
            }
            _ => None,
        };

        match value {
            Some(v) => {
                println!("{} = {}", style.code(key), v);
                Ok(())
            }
            None => {
                println!(
                    "{}",
                    style.warning(&format!("Configuration key not found: {}", key))
                );
                println!(
                    "{}",
                    style.info("Run 'rice config' to see all available keys")
                );
                Ok(())
            }
        }
    }

    /// Set a configuration value and persist it
    fn set_config(&self, key: &str, value: &str) -> CliResult<()> {
        let style = OutputStyle::default();

        // Load current configuration
        let mut config = Self::load_config()?;

        // Update the configuration based on the key
        match key {
            "provider.default" => {
                config.providers.default_provider = Some(value.to_string());
            }
            "defaults.model" => {
                config.defaults.model = Some(value.to_string());
            }
            "defaults.temperature" => {
                config.defaults.temperature = value.parse::<f32>().ok();
            }
            "defaults.max_tokens" => {
                config.defaults.max_tokens = value.parse::<u32>().ok();
            }
            k if k.starts_with("providers.") && k.ends_with(".api_key") => {
                // Extract provider name from key like "providers.openai.api_key"
                let provider = k
                    .strip_prefix("providers.")
                    .and_then(|s| s.strip_suffix(".api_key"))
                    .unwrap_or("");
                if !provider.is_empty() {
                    config.providers.api_keys.insert(provider.to_string(), value.to_string());
                } else {
                    return Err(CliError::Config(format!("Invalid key format: {}", key)));
                }
            }
            _ => {
                return Err(CliError::Config(format!("Unknown configuration key: {}", key)));
            }
        }

        // Save to global config file
        let config_dir = Self::get_config_dir()?;
        let config_file = config_dir.join("ricecoder.yaml");

        // Ensure config directory exists
        std::fs::create_dir_all(&config_dir)
            .map_err(|e| CliError::Config(format!("Failed to create config directory: {}", e)))?;

        // Save configuration to file
        ConfigLoader::save_to_file(&config, &config_file, ricecoder_storage::ConfigFormat::Yaml)
            .map_err(|e| CliError::Config(e.to_string()))?;

        println!("{}", style.success(&format!("Set {} = {}", key, value)));
        println!(
            "{}",
            style.info("Configuration changes will take effect on next run")
        );

        Ok(())
    }
}

impl Command for ConfigCommand {
    fn execute(&self) -> CliResult<()> {
        match &self.action {
            ConfigAction::List => self.list_config(),
            ConfigAction::Get(key) => self.get_config(key),
            ConfigAction::Set(key, value) => self.set_config(key, value),
        }
    }
}
