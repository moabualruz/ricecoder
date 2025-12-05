//! Integration with ricecoder-modes for markdown-based mode configuration

use crate::markdown_config::error::MarkdownConfigResult;
use crate::markdown_config::loader::{ConfigFile, ConfigFileType, ConfigurationLoader};
use crate::markdown_config::types::ModeConfig;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Type alias for registration results: (success_count, error_count, errors)
pub type RegistrationResult = (usize, usize, Vec<(String, String)>);

/// Trait for registering mode configurations
///
/// This trait allows ricecoder-storage to register mode configurations without
/// directly depending on ricecoder-modes, avoiding circular dependencies.
pub trait ModeRegistrar: Send + Sync {
    /// Register a mode configuration
    fn register_mode(&mut self, mode: ModeConfig) -> Result<(), String>;
}

/// Integration layer for mode configuration with ricecoder-modes
///
/// This struct provides methods to discover, load, and register mode configurations
/// from markdown files with the ricecoder-modes subsystem.
pub struct ModeConfigIntegration {
    loader: Arc<ConfigurationLoader>,
}

impl ModeConfigIntegration {
    /// Create a new mode configuration integration
    pub fn new(loader: Arc<ConfigurationLoader>) -> Self {
        Self { loader }
    }

    /// Discover mode configuration files in the given paths
    ///
    /// # Arguments
    /// * `paths` - Directories to search for mode markdown files
    ///
    /// # Returns
    /// A vector of discovered mode configuration files
    pub fn discover_mode_configs(&self, paths: &[PathBuf]) -> MarkdownConfigResult<Vec<ConfigFile>> {
        let all_files = self.loader.discover(paths)?;

        // Filter to only mode configuration files
        let mode_files: Vec<ConfigFile> = all_files
            .into_iter()
            .filter(|f| f.config_type == ConfigFileType::Mode)
            .collect();

        debug!("Discovered {} mode configuration files", mode_files.len());
        Ok(mode_files)
    }

    /// Load mode configurations from markdown files
    ///
    /// # Arguments
    /// * `paths` - Directories to search for mode markdown files
    ///
    /// # Returns
    /// A tuple of (loaded_modes, errors)
    pub async fn load_mode_configs(
        &self,
        paths: &[PathBuf],
    ) -> MarkdownConfigResult<(Vec<ModeConfig>, Vec<(PathBuf, String)>)> {
        let files = self.discover_mode_configs(paths)?;

        let mut modes = Vec::new();
        let mut errors = Vec::new();

        for file in files {
            match self.loader.load(&file).await {
                Ok(config) => {
                    match config {
                        crate::markdown_config::loader::LoadedConfig::Mode(mode) => {
                            debug!("Loaded mode configuration: {}", mode.name);
                            modes.push(mode);
                        }
                        _ => {
                            warn!("Expected mode configuration but got different type from {}", file.path.display());
                            errors.push((
                                file.path,
                                "Expected mode configuration but got different type".to_string(),
                            ));
                        }
                    }
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    warn!("Failed to load mode configuration from {}: {}", file.path.display(), error_msg);
                    errors.push((file.path, error_msg));
                }
            }
        }

        info!("Loaded {} mode configurations", modes.len());
        Ok((modes, errors))
    }

    /// Register mode configurations with a registrar
    ///
    /// This method registers mode configurations using a generic registrar trait,
    /// allowing integration with any mode manager implementation.
    ///
    /// # Arguments
    /// * `modes` - Mode configurations to register
    /// * `registrar` - The mode registrar to register with
    ///
    /// # Returns
    /// A tuple of (successful_count, error_count, errors)
    pub fn register_modes(
        &self,
        modes: Vec<ModeConfig>,
        registrar: &mut dyn ModeRegistrar,
    ) -> MarkdownConfigResult<RegistrationResult> {
        let mut success_count = 0;
        let mut error_count = 0;
        let mut errors = Vec::new();

        for mode in modes {
            // Validate mode configuration
            if let Err(e) = mode.validate() {
                error_count += 1;
                let error_msg = format!("Invalid mode configuration: {}", e);
                warn!("Failed to register mode '{}': {}", mode.name, error_msg);
                errors.push((mode.name.clone(), error_msg));
                continue;
            }

            debug!("Registering mode: {}", mode.name);

            // Register the mode using the registrar
            match registrar.register_mode(mode.clone()) {
                Ok(_) => {
                    success_count += 1;
                    info!("Registered mode: {}", mode.name);
                }
                Err(e) => {
                    error_count += 1;
                    warn!("Failed to register mode '{}': {}", mode.name, e);
                    errors.push((mode.name.clone(), e));
                }
            }
        }

        debug!(
            "Mode registration complete: {} successful, {} failed",
            success_count, error_count
        );

        Ok((success_count, error_count, errors))
    }

    /// Load and register mode configurations in one operation
    ///
    /// # Arguments
    /// * `paths` - Directories to search for mode markdown files
    /// * `registrar` - The mode registrar to register with
    ///
    /// # Returns
    /// A tuple of (successful_count, error_count, errors)
    pub async fn load_and_register_modes(
        &self,
        paths: &[PathBuf],
        registrar: &mut dyn ModeRegistrar,
    ) -> MarkdownConfigResult<(usize, usize, Vec<(String, String)>)> {
        let (modes, load_errors) = self.load_mode_configs(paths).await?;

        let (success, errors, mut reg_errors) = self.register_modes(modes, registrar)?;

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

    fn create_test_mode_file(dir: &PathBuf, name: &str, content: &str) -> PathBuf {
        let path = dir.join(format!("{}.mode.md", name));
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn test_discover_mode_configs() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().to_path_buf();

        // Create test mode files
        create_test_mode_file(&dir_path, "mode1", "---\nname: mode1\n---\nTest");
        create_test_mode_file(&dir_path, "mode2", "---\nname: mode2\n---\nTest");

        // Create a non-mode file
        fs::write(dir_path.join("agent1.agent.md"), "---\nname: agent1\n---\nTest").unwrap();

        let registry = Arc::new(ConfigRegistry::new());
        let loader = Arc::new(ConfigurationLoader::new(registry));
        let integration = ModeConfigIntegration::new(loader);

        let discovered = integration.discover_mode_configs(&[dir_path]).unwrap();

        assert_eq!(discovered.len(), 2);
        assert!(discovered.iter().all(|f| f.config_type == ConfigFileType::Mode));
    }

    #[tokio::test]
    async fn test_load_mode_configs() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().to_path_buf();

        let mode_content = r#"---
name: focus-mode
description: Focus mode
keybinding: C-f
enabled: true
---
Focus on the task"#;

        create_test_mode_file(&dir_path, "focus-mode", mode_content);

        let registry = Arc::new(ConfigRegistry::new());
        let loader = Arc::new(ConfigurationLoader::new(registry));
        let integration = ModeConfigIntegration::new(loader);

        let (modes, errors) = integration.load_mode_configs(&[dir_path]).await.unwrap();

        assert_eq!(modes.len(), 1);
        assert_eq!(errors.len(), 0);
        assert_eq!(modes[0].name, "focus-mode");
        assert_eq!(modes[0].keybinding, Some("C-f".to_string()));
    }

    #[tokio::test]
    async fn test_load_mode_configs_with_errors() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().to_path_buf();

        // Create a valid mode file
        let valid_content = r#"---
name: valid-mode
---
Valid mode"#;
        create_test_mode_file(&dir_path, "valid-mode", valid_content);

        // Create an invalid mode file (missing frontmatter)
        fs::write(dir_path.join("invalid.mode.md"), "# No frontmatter\nJust markdown").unwrap();

        let registry = Arc::new(ConfigRegistry::new());
        let loader = Arc::new(ConfigurationLoader::new(registry));
        let integration = ModeConfigIntegration::new(loader);

        let (modes, errors) = integration.load_mode_configs(&[dir_path]).await.unwrap();

        assert_eq!(modes.len(), 1);
        assert_eq!(errors.len(), 1);
        assert_eq!(modes[0].name, "valid-mode");
    }

    #[test]
    fn test_register_with_mode_manager() {
        let registry = Arc::new(ConfigRegistry::new());
        let loader = Arc::new(ConfigurationLoader::new(registry));
        let integration = ModeConfigIntegration::new(loader);

        let modes = vec![
            ModeConfig {
                name: "mode1".to_string(),
                description: Some("Test mode 1".to_string()),
                prompt: "You are mode 1".to_string(),
                keybinding: Some("C-m".to_string()),
                enabled: true,
            },
            ModeConfig {
                name: "mode2".to_string(),
                description: Some("Test mode 2".to_string()),
                prompt: "You are mode 2".to_string(),
                keybinding: None,
                enabled: false,
            },
        ];

        struct MockRegistrar;
        impl ModeRegistrar for MockRegistrar {
            fn register_mode(&mut self, _mode: ModeConfig) -> Result<(), String> {
                Ok(())
            }
        }

        let mut registrar = MockRegistrar;
        let (success, errors, error_list) = integration
            .register_modes(modes, &mut registrar)
            .unwrap();

        assert_eq!(success, 2);
        assert_eq!(errors, 0);
        assert_eq!(error_list.len(), 0);
    }

    #[test]
    fn test_register_invalid_mode() {
        let registry = Arc::new(ConfigRegistry::new());
        let loader = Arc::new(ConfigurationLoader::new(registry));
        let integration = ModeConfigIntegration::new(loader);

        let modes = vec![
            ModeConfig {
                name: String::new(), // Invalid: empty name
                description: None,
                prompt: "Test".to_string(),
                keybinding: None,
                enabled: true,
            },
        ];

        struct MockRegistrar;
        impl ModeRegistrar for MockRegistrar {
            fn register_mode(&mut self, _mode: ModeConfig) -> Result<(), String> {
                Ok(())
            }
        }

        let mut registrar = MockRegistrar;
        let (success, errors, error_list) = integration
            .register_modes(modes, &mut registrar)
            .unwrap();

        assert_eq!(success, 0);
        assert_eq!(errors, 1);
        assert_eq!(error_list.len(), 1);
    }
}
