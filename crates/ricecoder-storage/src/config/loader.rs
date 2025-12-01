//! Configuration file loader supporting multiple formats
//!
//! This module provides loading of configuration files in YAML, TOML, and JSON formats.
//! It automatically detects the format based on file extension.

use crate::error::{StorageError, StorageResult};
use crate::types::ConfigFormat;
use std::path::Path;
use super::Config;

/// Configuration loader for multiple formats
pub struct ConfigLoader;

impl ConfigLoader {
    /// Load configuration from a file
    ///
    /// Automatically detects format based on file extension.
    /// Supports YAML (.yaml, .yml), TOML (.toml), and JSON (.json) formats.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> StorageResult<Config> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path).map_err(|e| {
            StorageError::io_error(
                path.to_path_buf(),
                crate::error::IoOperation::Read,
                e,
            )
        })?;

        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| {
                StorageError::parse_error(
                    path.to_path_buf(),
                    "unknown",
                    "File has no extension",
                )
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
    pub fn load_from_string<P: AsRef<Path>>(
        content: &str,
        format: ConfigFormat,
        path: P,
    ) -> StorageResult<Config> {
        let path = path.as_ref();
        match format {
            ConfigFormat::Yaml => Self::parse_yaml(content, path),
            ConfigFormat::Toml => Self::parse_toml(content, path),
            ConfigFormat::Json => Self::parse_json(content, path),
        }
    }

    /// Parse YAML content
    fn parse_yaml<P: AsRef<Path>>(content: &str, path: P) -> StorageResult<Config> {
        let path = path.as_ref();
        serde_yaml::from_str(content).map_err(|e| {
            StorageError::parse_error(
                path.to_path_buf(),
                "YAML",
                e.to_string(),
            )
        })
    }

    /// Parse TOML content
    fn parse_toml<P: AsRef<Path>>(content: &str, path: P) -> StorageResult<Config> {
        let path = path.as_ref();
        toml::from_str(content).map_err(|e| {
            StorageError::parse_error(
                path.to_path_buf(),
                "TOML",
                e.to_string(),
            )
        })
    }

    /// Parse JSON content
    fn parse_json<P: AsRef<Path>>(content: &str, path: P) -> StorageResult<Config> {
        let path = path.as_ref();
        serde_json::from_str(content).map_err(|e| {
            StorageError::parse_error(
                path.to_path_buf(),
                "JSON",
                e.to_string(),
            )
        })
    }

    /// Serialize configuration to string in specified format
    pub fn serialize(config: &Config, format: ConfigFormat) -> StorageResult<String> {
        match format {
            ConfigFormat::Yaml => serde_yaml::to_string(config).map_err(|e| {
                StorageError::Internal(format!("Failed to serialize to YAML: {}", e))
            }),
            ConfigFormat::Toml => toml::to_string_pretty(config).map_err(|e| {
                StorageError::Internal(format!("Failed to serialize to TOML: {}", e))
            }),
            ConfigFormat::Json => serde_json::to_string_pretty(config).map_err(|e| {
                StorageError::Internal(format!("Failed to serialize to JSON: {}", e))
            }),
        }
    }

    /// Save configuration to a file
    pub fn save_to_file<P: AsRef<Path>>(
        config: &Config,
        path: P,
        format: ConfigFormat,
    ) -> StorageResult<()> {
        let path = path.as_ref();
        let content = Self::serialize(config, format)?;
        std::fs::write(path, content).map_err(|e| {
            StorageError::io_error(
                path.to_path_buf(),
                crate::error::IoOperation::Write,
                e,
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_load_yaml_config() {
        let yaml_content = r#"
providers:
  default_provider: openai
  api_keys:
    openai: test-key
defaults:
  model: gpt-4
  temperature: 0.7
steering: []
"#;
        let config = ConfigLoader::load_from_string(yaml_content, ConfigFormat::Yaml, "test.yaml")
            .expect("Failed to parse YAML");
        assert_eq!(config.providers.default_provider, Some("openai".to_string()));
        assert_eq!(config.defaults.model, Some("gpt-4".to_string()));
    }

    #[test]
    fn test_load_toml_config() {
        let toml_content = r#"[providers]
default_provider = "openai"
api_keys = { openai = "test-key" }
endpoints = {}

[defaults]
model = "gpt-4"
temperature = 0.7

steering = []
custom = {}
"#;
        let config = ConfigLoader::load_from_string(toml_content, ConfigFormat::Toml, "test.toml")
            .expect("Failed to parse TOML");
        assert_eq!(config.providers.default_provider, Some("openai".to_string()));
        assert_eq!(config.defaults.model, Some("gpt-4".to_string()));
    }

    #[test]
    fn test_load_json_config() {
        let json_content = r#"{
  "providers": {
    "default_provider": "openai",
    "api_keys": {
      "openai": "test-key"
    },
    "endpoints": {}
  },
  "defaults": {
    "model": "gpt-4",
    "temperature": 0.7
  },
  "steering": []
}"#;
        let config = ConfigLoader::load_from_string(json_content, ConfigFormat::Json, "test.json")
            .expect("Failed to parse JSON");
        assert_eq!(config.providers.default_provider, Some("openai".to_string()));
        assert_eq!(config.defaults.model, Some("gpt-4".to_string()));
    }

    #[test]
    fn test_serialize_yaml() {
        let config = Config::default();
        let yaml = ConfigLoader::serialize(&config, ConfigFormat::Yaml)
            .expect("Failed to serialize to YAML");
        assert!(yaml.contains("providers:"));
        assert!(yaml.contains("defaults:"));
    }

    #[test]
    fn test_serialize_toml() {
        let config = Config::default();
        let toml = ConfigLoader::serialize(&config, ConfigFormat::Toml)
            .expect("Failed to serialize to TOML");
        assert!(toml.contains("providers") || toml.contains("[providers]"));
        assert!(toml.contains("defaults") || toml.contains("[defaults]"));
    }

    #[test]
    fn test_serialize_json() {
        let config = Config::default();
        let json = ConfigLoader::serialize(&config, ConfigFormat::Json)
            .expect("Failed to serialize to JSON");
        assert!(json.contains("\"providers\""));
        assert!(json.contains("\"defaults\""));
    }

    #[test]
    fn test_save_and_load_yaml() {
        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("config.yaml");
        let config = Config::default();
        
        ConfigLoader::save_to_file(&config, &file_path, ConfigFormat::Yaml)
            .expect("Failed to save config");
        
        let loaded = ConfigLoader::load_from_file(&file_path)
            .expect("Failed to load config");
        
        assert_eq!(config, loaded);
    }

    #[test]
    fn test_save_and_load_toml() {
        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("config.toml");
        let config = Config::default();
        
        ConfigLoader::save_to_file(&config, &file_path, ConfigFormat::Toml)
            .expect("Failed to save config");
        
        let loaded = ConfigLoader::load_from_file(&file_path)
            .expect("Failed to load config");
        
        assert_eq!(config, loaded);
    }

    #[test]
    fn test_save_and_load_json() {
        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("config.json");
        let config = Config::default();
        
        ConfigLoader::save_to_file(&config, &file_path, ConfigFormat::Json)
            .expect("Failed to save config");
        
        let loaded = ConfigLoader::load_from_file(&file_path)
            .expect("Failed to load config");
        
        assert_eq!(config, loaded);
    }
}
