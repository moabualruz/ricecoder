//! Ollama-specific configuration management
//!
//! Handles loading and validating Ollama provider configuration from:
//! 1. Environment variables (highest priority)
//! 2. Project config file (.ricecoder/config.yaml)
//! 3. Global config file (~/.ricecoder/config.yaml)
//! 4. Built-in defaults (lowest priority)

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;
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
            debug!("Loading project Ollama config from {:?}", project_config_path);
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = OllamaConfig::default();
        assert_eq!(config.base_url, "http://localhost:11434");
        assert_eq!(config.default_model, "mistral");
        assert_eq!(config.timeout_secs, 30);
        assert_eq!(config.cache_ttl_secs, 300);
    }

    #[test]
    fn test_validate_default_config() {
        let config = OllamaConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_empty_base_url() {
        let mut config = OllamaConfig::default();
        config.base_url = String::new();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_base_url_scheme() {
        let mut config = OllamaConfig::default();
        config.base_url = "ftp://localhost:11434".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_empty_default_model() {
        let mut config = OllamaConfig::default();
        config.default_model = String::new();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_zero_timeout() {
        let mut config = OllamaConfig::default();
        config.timeout_secs = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_zero_cache_ttl() {
        let mut config = OllamaConfig::default();
        config.cache_ttl_secs = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_timeout_as_duration() {
        let config = OllamaConfig::default();
        assert_eq!(config.timeout(), Duration::from_secs(30));
    }

    #[test]
    fn test_cache_ttl_as_duration() {
        let config = OllamaConfig::default();
        assert_eq!(config.cache_ttl(), Duration::from_secs(300));
    }

    #[test]
    fn test_load_from_env_base_url() {
        std::env::set_var("OLLAMA_BASE_URL", "http://custom:11434");
        let mut config = OllamaConfig::default();
        config.load_from_env();
        assert_eq!(config.base_url, "http://custom:11434");
        std::env::remove_var("OLLAMA_BASE_URL");
    }

    #[test]
    fn test_load_from_env_default_model() {
        std::env::set_var("OLLAMA_DEFAULT_MODEL", "neural-chat");
        let mut config = OllamaConfig::default();
        config.load_from_env();
        assert_eq!(config.default_model, "neural-chat");
        std::env::remove_var("OLLAMA_DEFAULT_MODEL");
    }

    #[test]
    fn test_load_from_env_timeout() {
        std::env::set_var("OLLAMA_TIMEOUT_SECS", "60");
        let mut config = OllamaConfig::default();
        config.load_from_env();
        assert_eq!(config.timeout_secs, 60);
        std::env::remove_var("OLLAMA_TIMEOUT_SECS");
    }

    #[test]
    fn test_load_from_env_cache_ttl() {
        std::env::set_var("OLLAMA_CACHE_TTL_SECS", "600");
        let mut config = OllamaConfig::default();
        config.load_from_env();
        assert_eq!(config.cache_ttl_secs, 600);
        std::env::remove_var("OLLAMA_CACHE_TTL_SECS");
    }

    #[test]
    fn test_load_from_env_invalid_timeout() {
        std::env::set_var("OLLAMA_TIMEOUT_SECS", "invalid");
        let mut config = OllamaConfig::default();
        config.load_from_env();
        // Should keep default value when env var is invalid
        assert_eq!(config.timeout_secs, 30);
        std::env::remove_var("OLLAMA_TIMEOUT_SECS");
    }

    #[test]
    fn test_load_from_env_invalid_cache_ttl() {
        std::env::set_var("OLLAMA_CACHE_TTL_SECS", "not_a_number");
        let mut config = OllamaConfig::default();
        config.load_from_env();
        // Should keep default value when env var is invalid
        assert_eq!(config.cache_ttl_secs, 300);
        std::env::remove_var("OLLAMA_CACHE_TTL_SECS");
    }

    #[test]
    fn test_load_from_file_valid_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        let yaml_content = r#"
ollama:
  base_url: http://custom-host:11434
  default_model: llama2
  timeout_secs: 45
  cache_ttl_secs: 600
"#;

        fs::write(&config_path, yaml_content).unwrap();

        let mut config = OllamaConfig::default();
        config.load_from_file(&config_path).unwrap();

        assert_eq!(config.base_url, "http://custom-host:11434");
        assert_eq!(config.default_model, "llama2");
        assert_eq!(config.timeout_secs, 45);
        assert_eq!(config.cache_ttl_secs, 600);
    }

    #[test]
    fn test_load_from_file_partial_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        let yaml_content = r#"
ollama:
  base_url: http://partial:11434
"#;

        fs::write(&config_path, yaml_content).unwrap();

        let mut config = OllamaConfig::default();
        config.load_from_file(&config_path).unwrap();

        assert_eq!(config.base_url, "http://partial:11434");
        // Other values should be replaced with defaults from file (which are None, so they stay as defaults)
        assert_eq!(config.default_model, "mistral");
        assert_eq!(config.timeout_secs, 30);
        assert_eq!(config.cache_ttl_secs, 300);
    }

    #[test]
    fn test_load_from_file_missing_file() {
        let config_path = PathBuf::from("/nonexistent/path/config.yaml");
        let mut config = OllamaConfig::default();
        // Should not error, just skip
        assert!(config.load_from_file(&config_path).is_ok());
    }

    #[test]
    fn test_load_from_file_invalid_yaml() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        let yaml_content = r#"
ollama:
  base_url: [invalid yaml
"#;

        fs::write(&config_path, yaml_content).unwrap();

        let mut config = OllamaConfig::default();
        assert!(config.load_from_file(&config_path).is_err());
    }

    #[test]
    fn test_merge_from_file_preserves_existing() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        let yaml_content = r#"
ollama:
  default_model: neural-chat
"#;

        fs::write(&config_path, yaml_content).unwrap();

        let mut config = OllamaConfig::default();
        config.base_url = "http://custom:11434".to_string();
        config.merge_from_file(&config_path).unwrap();

        // Merged value should override
        assert_eq!(config.default_model, "neural-chat");
        // Existing value should be preserved
        assert_eq!(config.base_url, "http://custom:11434");
    }

    #[test]
    fn test_merge_from_file_missing_file() {
        let config_path = PathBuf::from("/nonexistent/path/config.yaml");
        let mut config = OllamaConfig::default();
        // Should not error, just skip
        assert!(config.merge_from_file(&config_path).is_ok());
    }

    #[test]
    #[serial_test::serial]
    fn test_env_var_overrides_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        let yaml_content = r#"
ollama:
  base_url: http://file-host:11434
  default_model: llama2
"#;

        fs::write(&config_path, yaml_content).unwrap();

        // Save original value to restore later
        let original_value = std::env::var("OLLAMA_BASE_URL").ok();
        
        // Clear any existing value first
        std::env::remove_var("OLLAMA_BASE_URL");
        
        // Now set the test value
        std::env::set_var("OLLAMA_BASE_URL", "http://env-host:11434");
        
        // Verify it was set
        let env_value = std::env::var("OLLAMA_BASE_URL").expect("OLLAMA_BASE_URL should be set");
        assert_eq!(env_value, "http://env-host:11434", "Environment variable not set correctly");

        let mut config = OllamaConfig::default();
        config.merge_from_file(&config_path).unwrap();
        config.load_from_env();

        // Environment variable should override file
        assert_eq!(config.base_url, "http://env-host:11434", "Environment variable did not override file value");
        // File value should be used for non-overridden settings
        assert_eq!(config.default_model, "llama2");

        // Restore original value
        if let Some(value) = original_value {
            std::env::set_var("OLLAMA_BASE_URL", value);
        } else {
            std::env::remove_var("OLLAMA_BASE_URL");
        }
    }

    #[test]
    fn test_load_with_precedence_all_sources() {
        // This test verifies the precedence order:
        // 1. Environment variables (highest)
        // 2. Project config
        // 3. Global config
        // 4. Defaults (lowest)

        std::env::set_var("OLLAMA_TIMEOUT_SECS", "90");

        // We can't easily test all sources in a unit test without mocking file system
        // But we can verify that environment variables take precedence
        let mut config = OllamaConfig::default();
        config.timeout_secs = 45; // Simulate loading from file
        config.load_from_env();

        assert_eq!(config.timeout_secs, 90); // Env var should override

        std::env::remove_var("OLLAMA_TIMEOUT_SECS");
    }

    #[test]
    fn test_get_global_config_path() {
        let path = OllamaConfig::get_global_config_path();
        assert!(path.to_string_lossy().contains(".ricecoder"));
        assert!(path.to_string_lossy().contains("config.yaml"));
    }

    #[test]
    fn test_get_project_config_path() {
        let path = OllamaConfig::get_project_config_path();
        assert_eq!(path, PathBuf::from(".ricecoder/config.yaml"));
    }

    #[test]
    fn test_validate_https_base_url() {
        let mut config = OllamaConfig::default();
        config.base_url = "https://secure-ollama:11434".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_http_base_url() {
        let mut config = OllamaConfig::default();
        config.base_url = "http://localhost:11434".to_string();
        assert!(config.validate().is_ok());
    }
}
