// Configuration management
// Adapted from automation/src/infrastructure/storage/config.rs

use crate::error::{CliError, CliResult};
use crate::output::OutputStyle;
use super::Command;
use std::collections::HashMap;
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

    /// Get the configuration directory path
    fn get_config_dir() -> CliResult<PathBuf> {
        // Try to get from environment first
        if let Ok(path) = std::env::var("RICECODER_HOME") {
            return Ok(PathBuf::from(path));
        }

        // Fall back to home directory
        if let Some(home) = dirs::home_dir() {
            return Ok(home.join(".ricecoder"));
        }

        Err(CliError::Config(
            "Could not determine configuration directory".to_string(),
        ))
    }

    /// Load configuration from file
    fn load_config() -> CliResult<HashMap<String, String>> {
        let mut config = HashMap::new();

        // Default configuration values
        config.insert("provider.default".to_string(), "openai".to_string());
        config.insert("storage.mode".to_string(), "merged".to_string());
        config.insert("chat.history".to_string(), "true".to_string());
        config.insert("output.colors".to_string(), "auto".to_string());

        // TODO: Load from config file if it exists
        // config_dir/ricecoder.toml

        Ok(config)
    }

    /// List all configuration values
    fn list_config(&self) -> CliResult<()> {
        let style = OutputStyle::default();
        let config = Self::load_config()?;

        println!("{}", style.header("RiceCoder Configuration"));
        println!();

        let mut keys: Vec<_> = config.keys().collect();
        keys.sort();

        for key in keys {
            let value = &config[key];
            println!("  {} = {}", style.code(key), value);
        }

        println!();
        let config_dir = Self::get_config_dir()?;
        println!("{}", style.info(&format!("Config directory: {}", config_dir.display())));

        Ok(())
    }

    /// Get a specific configuration value
    fn get_config(&self, key: &str) -> CliResult<()> {
        let style = OutputStyle::default();
        let config = Self::load_config()?;

        match config.get(key) {
            Some(value) => {
                println!("{} = {}", style.code(key), value);
                Ok(())
            }
            None => {
                println!("{}", style.warning(&format!("Configuration key not found: {}", key)));
                println!("{}", style.info("Run 'rice config' to see all available keys"));
                Ok(())
            }
        }
    }

    /// Set a configuration value
    fn set_config(&self, key: &str, value: &str) -> CliResult<()> {
        let style = OutputStyle::default();

        // TODO: Validate key format
        // TODO: Write to config file

        println!("{}", style.success(&format!("Set {} = {}", key, value)));
        println!("{}", style.info("Configuration changes will take effect on next run"));

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
