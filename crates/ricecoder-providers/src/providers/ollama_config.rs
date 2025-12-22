//! Ollama-specific configuration management
//!
//! Handles loading and validating Ollama provider configuration from:
//! 1. Environment variables (highest priority)
//! 2. Project config file (.ricecoder/config.yaml)
//! 3. Global config file (~/.ricecoder/config.yaml)
//! 4. Built-in defaults (lowest priority)

use std::{path::PathBuf, time::Duration};

use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use crate::error::ProviderError;

/// Ollama provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    /// Base URL for Ollama API (default: http://localhost:11434)
    pub base_url: String,
    /// Default model to use (default: mistral)
    pub default_model: String,
    /// Request timeout in seconds (default: 30)
    pub timeout_secs: u64,
    /// Cache TTL for model listings in seconds (default: 300)
    pub cache_ttl_secs: u64,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:11434".to_string(),
            default_model: "mistral".to_string(),
            timeout_secs: 30,
            cache_ttl_secs: 300,
        }
    }
}

impl OllamaConfig {
    /// Load Ollama configuration with proper precedence:
    /// 1. Environment variables (highest priority)
    /// 2. Project config (.ricecoder/config.yaml)
    /// 3. Global config (~/.ricecoder/config.yaml)
    /// 4. Built-in defaults (lowest priority)
    pub fn load_with_precedence() -> Result<Self, ProviderError> {
        let mut config = Self::default();

        // Load global config if it exists
        let global_config_path = Self::get_global_config_path();
        if global_config_path.exists() {
            debug!("Loading global Ollama config from {:?}", global_config_path);
            config.merge_from_file(&global_config_path)?;
        }

        // Load project config if it exists (overrides global)
        let project_config_path = Self::get_project_config_path();
        if project_config_path.exists() {
            debug!(
                "Loading project Ollama config from {:?}",
                project_config_path
            );
            config.merge_from_file(&project_config_path)?;
        }

        // Load environment variables (highest priority, overrides all)
        config.load_from_env();

        // Validate the final configuration
        config.validate()?;

        Ok(config)
    }

    /// Get the global configuration path (~/.ricecoder/config.yaml)
    pub fn get_global_config_path() -> PathBuf {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".ricecoder/config.yaml")
    }

    /// Get the project configuration path (.ricecoder/config.yaml)
    pub fn get_project_config_path() -> PathBuf {
        PathBuf::from(".ricecoder/config.yaml")
    }

    /// Load configuration from environment variables
    /// Environment variables override any existing configuration
    pub fn load_from_env(&mut self) {
        // OLLAMA_BASE_URL
        if let Ok(url) = std::env::var("OLLAMA_BASE_URL") {
            debug!("Loading OLLAMA_BASE_URL from environment: {}", url);
            self.base_url = url;
        }

        // OLLAMA_DEFAULT_MODEL
        if let Ok(model) = std::env::var("OLLAMA_DEFAULT_MODEL") {
            debug!("Loading OLLAMA_DEFAULT_MODEL from environment: {}", model);
            self.default_model = model;
        }

        // OLLAMA_TIMEOUT_SECS
        if let Ok(timeout_str) = std::env::var("OLLAMA_TIMEOUT_SECS") {
            if let Ok(timeout) = timeout_str.parse::<u64>() {
                debug!("Loading OLLAMA_TIMEOUT_SECS from environment: {}", timeout);
                self.timeout_secs = timeout;
            } else {
                warn!("Invalid OLLAMA_TIMEOUT_SECS value: {}", timeout_str);
            }
        }

        // OLLAMA_CACHE_TTL_SECS
        if let Ok(ttl_str) = std::env::var("OLLAMA_CACHE_TTL_SECS") {
            if let Ok(ttl) = ttl_str.parse::<u64>() {
                debug!("Loading OLLAMA_CACHE_TTL_SECS from environment: {}", ttl);
                self.cache_ttl_secs = ttl;
            } else {
                warn!("Invalid OLLAMA_CACHE_TTL_SECS value: {}", ttl_str);
            }
        }
    }

    /// Load configuration from a YAML file
    /// This replaces the current configuration with values from the file
    pub fn load_from_file(&mut self, path: &PathBuf) -> Result<(), ProviderError> {
        if !path.exists() {
            return Ok(()); // File doesn't exist, skip
        }

        let content = std::fs::read_to_string(path).map_err(|e| {
            ProviderError::ConfigError(format!("Failed to read Ollama config file: {}", e))
        })?;

        let file_config: OllamaFileConfig = serde_yaml::from_str(&content).map_err(|e| {
            ProviderError::ConfigError(format!("Failed to parse Ollama config file: {}", e))
        })?;

        // Replace current config with file config
        if let Some(ollama) = file_config.ollama {
            if let Some(base_url) = ollama.base_url {
                self.base_url = base_url;
            }
            if let Some(default_model) = ollama.default_model {
                self.default_model = default_model;
            }
            if let Some(timeout_secs) = ollama.timeout_secs {
                self.timeout_secs = timeout_secs;
            }
            if let Some(cache_ttl_secs) = ollama.cache_ttl_secs {
                self.cache_ttl_secs = cache_ttl_secs;
            }
        }

        Ok(())
    }

    /// Merge configuration from a YAML file
    /// This preserves existing configuration and only overrides specified values
    pub fn merge_from_file(&mut self, path: &PathBuf) -> Result<(), ProviderError> {
        if !path.exists() {
            return Ok(()); // File doesn't exist, skip
        }

        let content = std::fs::read_to_string(path).map_err(|e| {
            ProviderError::ConfigError(format!("Failed to read Ollama config file: {}", e))
        })?;

        let file_config: OllamaFileConfig = serde_yaml::from_str(&content).map_err(|e| {
            ProviderError::ConfigError(format!("Failed to parse Ollama config file: {}", e))
        })?;

        // Merge file config with current config (file values override current)
        if let Some(ollama) = file_config.ollama {
            if let Some(base_url) = ollama.base_url {
                self.base_url = base_url;
            }
            if let Some(default_model) = ollama.default_model {
                self.default_model = default_model;
            }
            if let Some(timeout_secs) = ollama.timeout_secs {
                self.timeout_secs = timeout_secs;
            }
            if let Some(cache_ttl_secs) = ollama.cache_ttl_secs {
                self.cache_ttl_secs = cache_ttl_secs;
            }
        }

        Ok(())
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), ProviderError> {
        // Validate base URL is not empty
        if self.base_url.is_empty() {
            return Err(ProviderError::ConfigError(
                "Ollama base URL cannot be empty".to_string(),
            ));
        }

        // Validate base URL is a valid URL
        if !self.base_url.starts_with("http://") && !self.base_url.starts_with("https://") {
            return Err(ProviderError::ConfigError(format!(
                "Ollama base URL must start with http:// or https://: {}",
                self.base_url
            )));
        }

        // Validate default model is not empty
        if self.default_model.is_empty() {
            return Err(ProviderError::ConfigError(
                "Ollama default model cannot be empty".to_string(),
            ));
        }

        // Validate timeout is greater than 0
        if self.timeout_secs == 0 {
            return Err(ProviderError::ConfigError(
                "Ollama timeout must be greater than 0 seconds".to_string(),
            ));
        }

        // Validate cache TTL is greater than 0
        if self.cache_ttl_secs == 0 {
            return Err(ProviderError::ConfigError(
                "Ollama cache TTL must be greater than 0 seconds".to_string(),
            ));
        }

        Ok(())
    }

    /// Get timeout as Duration
    pub fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_secs)
    }

    /// Get cache TTL as Duration
    pub fn cache_ttl(&self) -> Duration {
        Duration::from_secs(self.cache_ttl_secs)
    }
}

/// YAML file structure for Ollama configuration
#[derive(Debug, Deserialize)]
struct OllamaFileConfig {
    ollama: Option<OllamaFileSettings>,
}

/// Ollama settings from YAML file (all fields optional)
#[derive(Debug, Deserialize)]
struct OllamaFileSettings {
    base_url: Option<String>,
    default_model: Option<String>,
    timeout_secs: Option<u64>,
    cache_ttl_secs: Option<u64>,
}
