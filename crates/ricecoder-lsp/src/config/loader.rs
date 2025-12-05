//! Configuration loader for loading language configurations from files

use super::types::{ConfigError, ConfigRegistry, ConfigResult, LanguageConfig};
use std::path::Path;

/// Configuration loader
pub struct ConfigLoader;

impl ConfigLoader {
    /// Load configuration from a YAML file
    pub fn load_yaml(path: &Path) -> ConfigResult<LanguageConfig> {
        let content = std::fs::read_to_string(path)?;
        let config: LanguageConfig = serde_yaml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    /// Load configuration from a JSON file
    pub fn load_json(path: &Path) -> ConfigResult<LanguageConfig> {
        let content = std::fs::read_to_string(path)?;
        let config: LanguageConfig = serde_json::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    /// Load configuration from a file (auto-detect format)
    pub fn load(path: &Path) -> ConfigResult<LanguageConfig> {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("yaml") | Some("yml") => Self::load_yaml(path),
            Some("json") => Self::load_json(path),
            _ => Err(ConfigError::ValidationError(
                "Unsupported configuration file format".to_string(),
            )),
        }
    }

    /// Load all configurations from a directory
    pub fn load_directory(path: &Path) -> ConfigResult<ConfigRegistry> {
        let mut registry = ConfigRegistry::new();

        if !path.is_dir() {
            return Err(ConfigError::ValidationError(format!(
                "Path is not a directory: {}",
                path.display()
            )));
        }

        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if ext == "yaml" || ext == "yml" || ext == "json" {
                        match Self::load(&path) {
                            Ok(config) => {
                                registry.register(config)?;
                            }
                            Err(e) => {
                                tracing::warn!(
                                    "Failed to load configuration from {}: {}",
                                    path.display(),
                                    e
                                );
                            }
                        }
                    }
                }
            }
        }

        Ok(registry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_unsupported_format() {
        let result = ConfigLoader::load(Path::new("test.txt"));
        assert!(result.is_err());
    }
}
