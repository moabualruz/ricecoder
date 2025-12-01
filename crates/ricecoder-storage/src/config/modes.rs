//! Storage mode handling
//!
//! This module provides handling for different storage modes:
//! - GlobalOnly: Only use global storage
//! - ProjectOnly: Only use project storage
//! - Merged: Use both global and project with project overriding global

use super::Config;
use crate::types::StorageMode;
use std::path::Path;

/// Storage mode handler
pub struct StorageModeHandler;

impl StorageModeHandler {
    /// Load configuration based on storage mode
    ///
    /// - GlobalOnly: Load only from global path
    /// - ProjectOnly: Load only from project path
    /// - Merged: Load from both, with project overriding global
    pub fn load_for_mode(
        mode: StorageMode,
        global_path: Option<&Path>,
        project_path: Option<&Path>,
    ) -> crate::error::StorageResult<Config> {
        match mode {
            StorageMode::GlobalOnly => {
                Self::load_global_only(global_path)
            }
            StorageMode::ProjectOnly => {
                Self::load_project_only(project_path)
            }
            StorageMode::Merged => {
                Self::load_merged(global_path, project_path)
            }
        }
    }

    /// Load configuration from global storage only
    fn load_global_only(global_path: Option<&Path>) -> crate::error::StorageResult<Config> {
        if let Some(path) = global_path {
            let config_file = path.join("config.yaml");
            if config_file.exists() {
                return super::loader::ConfigLoader::load_from_file(&config_file);
            }
        }
        Ok(Config::default())
    }

    /// Load configuration from project storage only
    fn load_project_only(project_path: Option<&Path>) -> crate::error::StorageResult<Config> {
        if let Some(path) = project_path {
            let config_file = path.join("config.yaml");
            if config_file.exists() {
                return super::loader::ConfigLoader::load_from_file(&config_file);
            }
        }
        Ok(Config::default())
    }

    /// Load configuration from both global and project, with project overriding global
    fn load_merged(
        global_path: Option<&Path>,
        project_path: Option<&Path>,
    ) -> crate::error::StorageResult<Config> {
        let global_config = if let Some(path) = global_path {
            let config_file = path.join("config.yaml");
            if config_file.exists() {
                super::loader::ConfigLoader::load_from_file(&config_file).ok()
            } else {
                None
            }
        } else {
            None
        };

        let project_config = if let Some(path) = project_path {
            let config_file = path.join("config.yaml");
            if config_file.exists() {
                super::loader::ConfigLoader::load_from_file(&config_file).ok()
            } else {
                None
            }
        } else {
            None
        };

        let (merged, _) = super::merge::ConfigMerger::merge(
            Config::default(),
            global_config,
            project_config,
            None,
        );

        Ok(merged)
    }

    /// Verify that a mode is properly isolated
    ///
    /// For GlobalOnly mode, ensures no project config is loaded.
    /// For ProjectOnly mode, ensures no global config is loaded.
    pub fn verify_isolation(
        mode: StorageMode,
        global_path: Option<&Path>,
        project_path: Option<&Path>,
    ) -> crate::error::StorageResult<bool> {
        match mode {
            StorageMode::GlobalOnly => {
                // Verify that project config is not loaded
                if let Some(path) = project_path {
                    let config_file = path.join("config.yaml");
                    Ok(!config_file.exists())
                } else {
                    Ok(true)
                }
            }
            StorageMode::ProjectOnly => {
                // Verify that global config is not loaded
                if let Some(path) = global_path {
                    let config_file = path.join("config.yaml");
                    Ok(!config_file.exists())
                } else {
                    Ok(true)
                }
            }
            StorageMode::Merged => {
                // Merged mode doesn't have isolation requirements
                Ok(true)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_global_only_mode_loads_global() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let global_path = temp_dir.path();

        // Create a global config
        let config_file = global_path.join("config.yaml");
        let config_content = r#"
providers:
  default_provider: openai
defaults:
  model: gpt-4
steering: []
"#;
        fs::write(&config_file, config_content).expect("Failed to write config");

        let config = StorageModeHandler::load_for_mode(
            StorageMode::GlobalOnly,
            Some(global_path),
            None,
        ).expect("Failed to load config");

        assert_eq!(config.providers.default_provider, Some("openai".to_string()));
        assert_eq!(config.defaults.model, Some("gpt-4".to_string()));
    }

    #[test]
    fn test_project_only_mode_loads_project() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let project_path = temp_dir.path();

        // Create a project config
        let config_file = project_path.join("config.yaml");
        let config_content = r#"
providers:
  default_provider: anthropic
defaults:
  model: claude-3
steering: []
"#;
        fs::write(&config_file, config_content).expect("Failed to write config");

        let config = StorageModeHandler::load_for_mode(
            StorageMode::ProjectOnly,
            None,
            Some(project_path),
        ).expect("Failed to load config");

        assert_eq!(config.providers.default_provider, Some("anthropic".to_string()));
        assert_eq!(config.defaults.model, Some("claude-3".to_string()));
    }

    #[test]
    fn test_merged_mode_project_overrides_global() {
        let global_dir = TempDir::new().expect("Failed to create temp dir");
        let project_dir = TempDir::new().expect("Failed to create temp dir");

        // Create global config
        let global_config_file = global_dir.path().join("config.yaml");
        let global_content = r#"
providers:
  default_provider: openai
defaults:
  model: gpt-4
steering: []
"#;
        fs::write(&global_config_file, global_content).expect("Failed to write global config");

        // Create project config
        let project_config_file = project_dir.path().join("config.yaml");
        let project_content = r#"
providers:
  default_provider: anthropic
defaults:
  model: claude-3
steering: []
"#;
        fs::write(&project_config_file, project_content).expect("Failed to write project config");

        let config = StorageModeHandler::load_for_mode(
            StorageMode::Merged,
            Some(global_dir.path()),
            Some(project_dir.path()),
        ).expect("Failed to load config");

        // Project should override global
        assert_eq!(config.providers.default_provider, Some("anthropic".to_string()));
        assert_eq!(config.defaults.model, Some("claude-3".to_string()));
    }

    #[test]
    fn test_global_only_isolation() {
        let global_dir = TempDir::new().expect("Failed to create temp dir");
        let project_dir = TempDir::new().expect("Failed to create temp dir");

        // Create both configs
        let global_config_file = global_dir.path().join("config.yaml");
        fs::write(&global_config_file, "providers:\n  default_provider: openai\ndefaults: {}\nsteering: []")
            .expect("Failed to write global config");

        let project_config_file = project_dir.path().join("config.yaml");
        fs::write(&project_config_file, "providers:\n  default_provider: anthropic\ndefaults: {}\nsteering: []")
            .expect("Failed to write project config");

        // Load in GlobalOnly mode
        let config = StorageModeHandler::load_for_mode(
            StorageMode::GlobalOnly,
            Some(global_dir.path()),
            Some(project_dir.path()),
        ).expect("Failed to load config");

        // Should only have global config
        assert_eq!(config.providers.default_provider, Some("openai".to_string()));
    }

    #[test]
    fn test_project_only_isolation() {
        let global_dir = TempDir::new().expect("Failed to create temp dir");
        let project_dir = TempDir::new().expect("Failed to create temp dir");

        // Create both configs
        let global_config_file = global_dir.path().join("config.yaml");
        fs::write(&global_config_file, "providers:\n  default_provider: openai\ndefaults: {}\nsteering: []")
            .expect("Failed to write global config");

        let project_config_file = project_dir.path().join("config.yaml");
        fs::write(&project_config_file, "providers:\n  default_provider: anthropic\ndefaults: {}\nsteering: []")
            .expect("Failed to write project config");

        // Load in ProjectOnly mode
        let config = StorageModeHandler::load_for_mode(
            StorageMode::ProjectOnly,
            Some(global_dir.path()),
            Some(project_dir.path()),
        ).expect("Failed to load config");

        // Should only have project config
        assert_eq!(config.providers.default_provider, Some("anthropic".to_string()));
    }
}
