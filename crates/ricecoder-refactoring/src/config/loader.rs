//! Configuration loader for refactoring rules

use std::path::Path;

use crate::{
    error::{RefactoringError, Result},
    types::RefactoringConfig,
};

/// Loads refactoring configuration from files
pub struct ConfigLoader;

impl ConfigLoader {
    /// Load configuration from a YAML file
    pub fn load_from_yaml(path: &Path) -> Result<RefactoringConfig> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            RefactoringError::ConfigError(format!("Failed to read config file: {}", e))
        })?;

        serde_yaml::from_str(&content)
            .map_err(|e| RefactoringError::ConfigError(format!("Failed to parse YAML: {}", e)))
    }

    /// Load configuration from a JSON file
    pub fn load_from_json(path: &Path) -> Result<RefactoringConfig> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            RefactoringError::ConfigError(format!("Failed to read config file: {}", e))
        })?;

        serde_json::from_str(&content)
            .map_err(|e| RefactoringError::ConfigError(format!("Failed to parse JSON: {}", e)))
    }

    /// Load configuration from a file (auto-detect format)
    pub fn load(path: &Path) -> Result<RefactoringConfig> {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("yaml") | Some("yml") => Self::load_from_yaml(path),
            Some("json") => Self::load_from_json(path),
            _ => Err(RefactoringError::ConfigError(
                "Unsupported configuration file format".to_string(),
            )),
        }
    }

    /// Validate configuration
    pub fn validate(config: &RefactoringConfig) -> Result<()> {
        if config.language.is_empty() {
            return Err(RefactoringError::InvalidConfiguration(
                "Language name cannot be empty".to_string(),
            ));
        }

        if config.extensions.is_empty() {
            return Err(RefactoringError::InvalidConfiguration(
                "At least one file extension must be specified".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_yaml_config() -> Result<()> {
        use tempfile::TempDir;
        let dir = TempDir::new()?;
        let file_path = dir.path().join("config.yaml");

        let yaml_content = r#"
language: rust
extensions:
  - .rs
rules:
  - name: "unused_variable"
    pattern: "let \\w+ = .*;"
    refactoring_type: RemoveUnused
    enabled: true
transformations: []
"#;
        std::fs::write(&file_path, yaml_content)?;

        let config = ConfigLoader::load(&file_path)?;
        assert_eq!(config.language, "rust");
        assert_eq!(config.extensions, vec![".rs"]);
        assert_eq!(config.rules.len(), 1);

        Ok(())
    }

    #[test]
    fn test_validate_config() -> Result<()> {
        let config = RefactoringConfig {
            language: "rust".to_string(),
            extensions: vec![".rs".to_string()],
            rules: vec![],
            transformations: vec![],
            provider: None,
        };

        ConfigLoader::validate(&config)?;
        Ok(())
    }

    #[test]
    fn test_validate_config_empty_language() {
        let config = RefactoringConfig {
            language: "".to_string(),
            extensions: vec![".rs".to_string()],
            rules: vec![],
            transformations: vec![],
            provider: None,
        };

        assert!(ConfigLoader::validate(&config).is_err());
    }
}
