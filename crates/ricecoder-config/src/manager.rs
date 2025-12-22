//! Configuration manager implementation

use std::path::PathBuf;

use config::{Config, Environment, File};

use crate::{
    error::{ConfigError, Result},
    types::{AppConfig, ConfigManager as ConfigManagerTrait},
};

/// Configuration manager
pub struct ConfigManager {
    /// Configuration file path
    config_path: PathBuf,
    /// Environment prefix
    env_prefix: String,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new() -> Self {
        Self {
            config_path: Self::default_config_path(),
            env_prefix: "APP".to_string(),
        }
    }

    /// Create with custom config path
    pub fn with_path(path: PathBuf) -> Self {
        Self {
            config_path: path,
            env_prefix: "APP".to_string(),
        }
    }

    /// Get default config path
    fn default_config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("ricecoder")
            .join("config.toml")
    }
}

impl ConfigManagerTrait for ConfigManager {
    fn load_config(&mut self) -> Result<AppConfig> {
        let builder = Config::builder()
            .add_source(File::from(self.config_path.clone()).required(false))
            .add_source(Environment::with_prefix(&self.env_prefix));

        let config = builder.build()?;
        let app_config: AppConfig = config.try_deserialize()?;
        Ok(app_config)
    }

    fn save_config(&self, config: &AppConfig) -> Result<()> {
        let toml = toml::to_string(config)?;
        std::fs::create_dir_all(self.config_path.parent().unwrap())?;
        std::fs::write(&self.config_path, toml)?;
        Ok(())
    }

    fn validate_config(&self, config: &AppConfig) -> Result<()> {
        // Basic validation
        if config.editor.tab_size == 0 {
            return Err(ConfigError::Validation(
                "Tab size must be greater than 0".to_string(),
            ));
        }
        if config.ui.font_size == 0 {
            return Err(ConfigError::Validation(
                "Font size must be greater than 0".to_string(),
            ));
        }
        Ok(())
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}
