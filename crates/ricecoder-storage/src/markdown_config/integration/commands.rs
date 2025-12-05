//! Integration with ricecoder-commands for markdown-based command configuration

use crate::markdown_config::error::MarkdownConfigResult;
use crate::markdown_config::loader::{ConfigFile, ConfigFileType, ConfigurationLoader};
use crate::markdown_config::types::CommandConfig;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Type alias for registration results: (success_count, error_count, errors)
pub type RegistrationResult = (usize, usize, Vec<(String, String)>);

/// Trait for registering command configurations
///
/// This trait allows ricecoder-storage to register command configurations without
/// directly depending on ricecoder-commands, avoiding circular dependencies.
pub trait CommandRegistrar: Send + Sync {
    /// Register a command configuration
    fn register_command(&mut self, command: CommandConfig) -> Result<(), String>;
}

/// Integration layer for command configuration with ricecoder-commands
///
/// This struct provides methods to discover, load, and register command configurations
/// from markdown files with the ricecoder-commands subsystem.
pub struct CommandConfigIntegration {
    loader: Arc<ConfigurationLoader>,
}

impl CommandConfigIntegration {
    /// Create a new command configuration integration
    pub fn new(loader: Arc<ConfigurationLoader>) -> Self {
        Self { loader }
    }

    /// Discover command configuration files in the given paths
    ///
    /// # Arguments
    /// * `paths` - Directories to search for command markdown files
    ///
    /// # Returns
    /// A vector of discovered command configuration files
    pub fn discover_command_configs(&self, paths: &[PathBuf]) -> MarkdownConfigResult<Vec<ConfigFile>> {
        let all_files = self.loader.discover(paths)?;

        // Filter to only command configuration files
        let command_files: Vec<ConfigFile> = all_files
            .into_iter()
            .filter(|f| f.config_type == ConfigFileType::Command)
            .collect();

        debug!("Discovered {} command configuration files", command_files.len());
        Ok(command_files)
    }

    /// Load command configurations from markdown files
    ///
    /// # Arguments
    /// * `paths` - Directories to search for command markdown files
    ///
    /// # Returns
    /// A tuple of (loaded_commands, errors)
    pub async fn load_command_configs(
        &self,
        paths: &[PathBuf],
    ) -> MarkdownConfigResult<(Vec<CommandConfig>, Vec<(PathBuf, String)>)> {
        let files = self.discover_command_configs(paths)?;

        let mut commands = Vec::new();
        let mut errors = Vec::new();

        for file in files {
            match self.loader.load(&file).await {
                Ok(config) => {
                    match config {
                        crate::markdown_config::loader::LoadedConfig::Command(command) => {
                            debug!("Loaded command configuration: {}", command.name);
                            commands.push(command);
                        }
                        _ => {
                            warn!("Expected command configuration but got different type from {}", file.path.display());
                            errors.push((
                                file.path,
                                "Expected command configuration but got different type".to_string(),
                            ));
                        }
                    }
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    warn!("Failed to load command configuration from {}: {}", file.path.display(), error_msg);
                    errors.push((file.path, error_msg));
                }
            }
        }

        info!("Loaded {} command configurations", commands.len());
        Ok((commands, errors))
    }

    /// Register command configurations with a registrar
    ///
    /// This method registers command configurations using a generic registrar trait,
    /// allowing integration with any command registry implementation.
    ///
    /// # Arguments
    /// * `commands` - Command configurations to register
    /// * `registrar` - The command registrar to register with
    ///
    /// # Returns
    /// A tuple of (successful_count, error_count, errors)
    pub fn register_commands(
        &self,
        commands: Vec<CommandConfig>,
        registrar: &mut dyn CommandRegistrar,
    ) -> MarkdownConfigResult<RegistrationResult> {
        let mut success_count = 0;
        let mut error_count = 0;
        let mut errors = Vec::new();

        for command in commands {
            // Validate command configuration
            if let Err(e) = command.validate() {
                error_count += 1;
                let error_msg = format!("Invalid command configuration: {}", e);
                warn!("Failed to register command '{}': {}", command.name, error_msg);
                errors.push((command.name.clone(), error_msg));
                continue;
            }

            debug!("Registering command: {}", command.name);

            // Register the command using the registrar
            match registrar.register_command(command.clone()) {
                Ok(_) => {
                    success_count += 1;
                    info!("Registered command: {}", command.name);
                }
                Err(e) => {
                    error_count += 1;
                    warn!("Failed to register command '{}': {}", command.name, e);
                    errors.push((command.name.clone(), e));
                }
            }
        }

        debug!(
            "Command registration complete: {} successful, {} failed",
            success_count, error_count
        );

        Ok((success_count, error_count, errors))
    }

    /// Load and register command configurations in one operation
    ///
    /// # Arguments
    /// * `paths` - Directories to search for command markdown files
    /// * `registrar` - The command registrar to register with
    ///
    /// # Returns
    /// A tuple of (successful_count, error_count, errors)
    pub async fn load_and_register_commands(
        &self,
        paths: &[PathBuf],
        registrar: &mut dyn CommandRegistrar,
    ) -> MarkdownConfigResult<(usize, usize, Vec<(String, String)>)> {
        let (commands, load_errors) = self.load_command_configs(paths).await?;

        let (success, errors, mut reg_errors) = self.register_commands(commands, registrar)?;

        // Combine load and registration errors
        for (path, msg) in load_errors {
            reg_errors.push((path.display().to_string(), msg));
        }

        Ok((success, errors, reg_errors))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown_config::registry::ConfigRegistry;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_command_file(dir: &PathBuf, name: &str, content: &str) -> PathBuf {
        let path = dir.join(format!("{}.command.md", name));
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn test_discover_command_configs() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().to_path_buf();

        // Create test command files
        create_test_command_file(&dir_path, "cmd1", "---\nname: cmd1\n---\nTest");
        create_test_command_file(&dir_path, "cmd2", "---\nname: cmd2\n---\nTest");

        // Create a non-command file
        fs::write(dir_path.join("agent1.agent.md"), "---\nname: agent1\n---\nTest").unwrap();

        let registry = Arc::new(ConfigRegistry::new());
        let loader = Arc::new(ConfigurationLoader::new(registry));
        let integration = CommandConfigIntegration::new(loader);

        let discovered = integration.discover_command_configs(&[dir_path]).unwrap();

        assert_eq!(discovered.len(), 2);
        assert!(discovered.iter().all(|f| f.config_type == ConfigFileType::Command));
    }

    #[tokio::test]
    async fn test_load_command_configs() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().to_path_buf();

        let command_content = r#"---
name: test-command
description: A test command
parameters:
  - name: message
    description: Message to echo
    required: true
keybinding: C-t
---
echo {{message}}"#;

        create_test_command_file(&dir_path, "test-command", command_content);

        let registry = Arc::new(ConfigRegistry::new());
        let loader = Arc::new(ConfigurationLoader::new(registry));
        let integration = CommandConfigIntegration::new(loader);

        let (commands, errors) = integration.load_command_configs(&[dir_path]).await.unwrap();

        assert_eq!(commands.len(), 1);
        assert_eq!(errors.len(), 0);
        assert_eq!(commands[0].name, "test-command");
        assert_eq!(commands[0].parameters.len(), 1);
    }

    #[tokio::test]
    async fn test_load_command_configs_with_errors() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().to_path_buf();

        // Create a valid command file
        let valid_content = r#"---
name: valid-command
---
echo test"#;
        create_test_command_file(&dir_path, "valid-command", valid_content);

        // Create an invalid command file (missing frontmatter)
        fs::write(dir_path.join("invalid.command.md"), "# No frontmatter\nJust markdown").unwrap();

        let registry = Arc::new(ConfigRegistry::new());
        let loader = Arc::new(ConfigurationLoader::new(registry));
        let integration = CommandConfigIntegration::new(loader);

        let (commands, errors) = integration.load_command_configs(&[dir_path]).await.unwrap();

        assert_eq!(commands.len(), 1);
        assert_eq!(errors.len(), 1);
        assert_eq!(commands[0].name, "valid-command");
    }

    #[test]
    fn test_register_with_command_registry() {
        let registry = Arc::new(ConfigRegistry::new());
        let loader = Arc::new(ConfigurationLoader::new(registry));
        let integration = CommandConfigIntegration::new(loader);

        let commands = vec![
            CommandConfig {
                name: "cmd1".to_string(),
                description: Some("Test command 1".to_string()),
                template: "echo {{message}}".to_string(),
                parameters: vec![crate::markdown_config::types::Parameter {
                    name: "message".to_string(),
                    description: Some("Message to echo".to_string()),
                    required: true,
                    default: None,
                }],
                keybinding: Some("C-1".to_string()),
            },
            CommandConfig {
                name: "cmd2".to_string(),
                description: Some("Test command 2".to_string()),
                template: "ls -la".to_string(),
                parameters: vec![],
                keybinding: None,
            },
        ];

        struct MockRegistrar;
        impl CommandRegistrar for MockRegistrar {
            fn register_command(&mut self, _command: CommandConfig) -> Result<(), String> {
                Ok(())
            }
        }

        let mut registrar = MockRegistrar;
        let (success, errors, error_list) = integration
            .register_commands(commands, &mut registrar)
            .unwrap();

        assert_eq!(success, 2);
        assert_eq!(errors, 0);
        assert_eq!(error_list.len(), 0);
    }

    #[test]
    fn test_register_invalid_command() {
        let registry = Arc::new(ConfigRegistry::new());
        let loader = Arc::new(ConfigurationLoader::new(registry));
        let integration = CommandConfigIntegration::new(loader);

        let commands = vec![
            CommandConfig {
                name: String::new(), // Invalid: empty name
                description: None,
                template: "echo test".to_string(),
                parameters: vec![],
                keybinding: None,
            },
        ];

        struct MockRegistrar;
        impl CommandRegistrar for MockRegistrar {
            fn register_command(&mut self, _command: CommandConfig) -> Result<(), String> {
                Ok(())
            }
        }

        let mut registrar = MockRegistrar;
        let (success, errors, error_list) = integration
            .register_commands(commands, &mut registrar)
            .unwrap();

        assert_eq!(success, 0);
        assert_eq!(errors, 1);
        assert_eq!(error_list.len(), 1);
    }
}
