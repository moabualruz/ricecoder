use crate::error::{CommandError, Result};
use crate::registry::CommandRegistry;
use crate::types::CommandDefinition;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Configuration file format for commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandsConfig {
    /// List of command definitions
    pub commands: Vec<CommandDefinition>,
}

/// Command configuration manager
pub struct ConfigManager;

impl ConfigManager {
    /// Load commands from a YAML file
    pub fn load_from_yaml<P: AsRef<Path>>(path: P) -> Result<CommandRegistry> {
        let content = fs::read_to_string(path)
            .map_err(|e| CommandError::ConfigError(format!("Failed to read config file: {}", e)))?;

        let config: CommandsConfig = serde_yaml::from_str(&content)?;

        let mut registry = CommandRegistry::new();
        for command in config.commands {
            registry.register(command)?;
        }

        Ok(registry)
    }

    /// Load commands from a JSON file
    pub fn load_from_json<P: AsRef<Path>>(path: P) -> Result<CommandRegistry> {
        let content = fs::read_to_string(path)
            .map_err(|e| CommandError::ConfigError(format!("Failed to read config file: {}", e)))?;

        let config: CommandsConfig = serde_json::from_str(&content)?;

        let mut registry = CommandRegistry::new();
        for command in config.commands {
            registry.register(command)?;
        }

        Ok(registry)
    }

    /// Load commands from a file (auto-detect format)
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<CommandRegistry> {
        let path = path.as_ref();
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        match extension {
            "yaml" | "yml" => Self::load_from_yaml(path),
            "json" => Self::load_from_json(path),
            _ => Err(CommandError::ConfigError(
                "Unsupported file format. Use .yaml, .yml, or .json".to_string(),
            )),
        }
    }

    /// Save commands to a YAML file
    pub fn save_to_yaml<P: AsRef<Path>>(registry: &CommandRegistry, path: P) -> Result<()> {
        let config = CommandsConfig {
            commands: registry.list_all(),
        };

        let content = serde_yaml::to_string(&config)?;
        fs::write(path, content)
            .map_err(|e| CommandError::ConfigError(format!("Failed to write config file: {}", e)))?;

        Ok(())
    }

    /// Save commands to a JSON file
    pub fn save_to_json<P: AsRef<Path>>(registry: &CommandRegistry, path: P) -> Result<()> {
        let config = CommandsConfig {
            commands: registry.list_all(),
        };

        let content = serde_json::to_string_pretty(&config)?;
        fs::write(path, content)
            .map_err(|e| CommandError::ConfigError(format!("Failed to write config file: {}", e)))?;

        Ok(())
    }

    /// Save commands to a file (auto-detect format)
    pub fn save_to_file<P: AsRef<Path>>(registry: &CommandRegistry, path: P) -> Result<()> {
        let path = path.as_ref();
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        match extension {
            "yaml" | "yml" => Self::save_to_yaml(registry, path),
            "json" => Self::save_to_json(registry, path),
            _ => Err(CommandError::ConfigError(
                "Unsupported file format. Use .yaml, .yml, or .json".to_string(),
            )),
        }
    }

    /// Merge multiple registries
    pub fn merge_registries(registries: Vec<CommandRegistry>) -> Result<CommandRegistry> {
        let mut merged = CommandRegistry::new();

        for registry in registries {
            for command in registry.list_all() {
                merged.register(command)?;
            }
        }

        Ok(merged)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ArgumentType, CommandArgument};
    use tempfile::NamedTempFile;

    fn create_test_registry() -> CommandRegistry {
        let mut registry = CommandRegistry::new();
        let cmd = CommandDefinition::new("test", "Test Command", "echo test")
            .with_description("A test command")
            .with_argument(
                CommandArgument::new("name", ArgumentType::String)
                    .with_description("User name")
                    .with_required(true),
            );
        registry.register(cmd).ok();
        registry
    }

    #[test]
    fn test_save_and_load_yaml() {
        let registry = create_test_registry();
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().with_extension("yaml");

        ConfigManager::save_to_yaml(&registry, &path).unwrap();
        let loaded = ConfigManager::load_from_yaml(&path).unwrap();

        assert_eq!(loaded.count(), 1);
        assert!(loaded.exists("test"));
    }

    #[test]
    fn test_save_and_load_json() {
        let registry = create_test_registry();
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().with_extension("json");

        ConfigManager::save_to_json(&registry, &path).unwrap();
        let loaded = ConfigManager::load_from_json(&path).unwrap();

        assert_eq!(loaded.count(), 1);
        assert!(loaded.exists("test"));
    }

    #[test]
    fn test_auto_detect_yaml() {
        let registry = create_test_registry();
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().with_extension("yaml");

        ConfigManager::save_to_file(&registry, &path).unwrap();
        let loaded = ConfigManager::load_from_file(&path).unwrap();

        assert_eq!(loaded.count(), 1);
    }

    #[test]
    fn test_auto_detect_json() {
        let registry = create_test_registry();
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().with_extension("json");

        ConfigManager::save_to_file(&registry, &path).unwrap();
        let loaded = ConfigManager::load_from_file(&path).unwrap();

        assert_eq!(loaded.count(), 1);
    }

    #[test]
    fn test_unsupported_format() {
        let registry = create_test_registry();
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().with_extension("txt");

        assert!(ConfigManager::save_to_file(&registry, &path).is_err());
    }

    #[test]
    fn test_merge_registries() {
        let mut registry1 = CommandRegistry::new();
        registry1
            .register(CommandDefinition::new("cmd1", "Cmd1", "echo 1"))
            .ok();

        let mut registry2 = CommandRegistry::new();
        registry2
            .register(CommandDefinition::new("cmd2", "Cmd2", "echo 2"))
            .ok();

        let merged = ConfigManager::merge_registries(vec![registry1, registry2]).unwrap();
        assert_eq!(merged.count(), 2);
        assert!(merged.exists("cmd1"));
        assert!(merged.exists("cmd2"));
    }
}
