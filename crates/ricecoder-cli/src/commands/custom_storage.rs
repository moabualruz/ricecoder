// Storage integration for custom commands
// Handles loading and saving custom commands to ricecoder-storage

use crate::error::{CliError, CliResult};
use ricecoder_commands::{CommandDefinition, CommandRegistry, ConfigManager};
use ricecoder_storage::PathResolver;
use std::fs;
use std::path::{Path, PathBuf};

/// Custom commands storage manager
pub struct CustomCommandsStorage {
    global_path: PathBuf,
    project_path: Option<PathBuf>,
}

impl CustomCommandsStorage {
    /// Create a new custom commands storage manager
    pub fn new() -> CliResult<Self> {
        let global_path =
            PathResolver::resolve_global_path().map_err(|e| CliError::Internal(e.to_string()))?;

        let project_path = if PathResolver::resolve_project_path().exists() {
            Some(PathResolver::resolve_project_path())
        } else {
            None
        };

        Ok(Self {
            global_path,
            project_path,
        })
    }

    /// Get the commands directory path
    fn commands_dir(&self, use_project: bool) -> PathBuf {
        if use_project {
            if let Some(project_path) = &self.project_path {
                return project_path.join("commands");
            }
        }
        self.global_path.join("commands")
    }

    /// Load all custom commands from storage
    pub fn load_all(&self) -> CliResult<CommandRegistry> {
        let mut registry = CommandRegistry::new();

        // Load from global storage first
        let global_commands_dir = self.commands_dir(false);
        if global_commands_dir.exists() {
            self.load_from_directory(&global_commands_dir, &mut registry)?;
        }

        // Load from project storage (overrides global)
        if let Some(project_path) = &self.project_path {
            let project_commands_dir = project_path.join("commands");
            if project_commands_dir.exists() {
                self.load_from_directory(&project_commands_dir, &mut registry)?;
            }
        }

        Ok(registry)
    }

    /// Load commands from a specific directory
    fn load_from_directory(&self, dir: &Path, registry: &mut CommandRegistry) -> CliResult<()> {
        if !dir.is_dir() {
            return Ok(());
        }

        for entry in fs::read_dir(dir).map_err(CliError::Io)? {
            let entry = entry.map_err(CliError::Io)?;
            let path = entry.path();

            if path.is_file() {
                let file_name = match path.file_name() {
                    Some(name) => name.to_string_lossy(),
                    None => {
                        eprintln!("Warning: Could not get file name for path: {:?}", path);
                        continue;
                    }
                };

                // Try to load as JSON or YAML
                if file_name.ends_with(".json")
                    || file_name.ends_with(".yaml")
                    || file_name.ends_with(".yml")
                {
                    match ConfigManager::load_from_file(&path) {
                        Ok(loaded_registry) => {
                            // Merge loaded commands into registry
                            for cmd in loaded_registry.list_all() {
                                // Ignore duplicates (project overrides global)
                                let _ = registry.register(cmd);
                            }
                        }
                        Err(e) => {
                            // Log warning but continue loading other files
                            eprintln!(
                                "Warning: Failed to load commands from {}: {}",
                                path.display(),
                                e
                            );
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Save a command to storage
    pub fn save_command(&self, cmd: &CommandDefinition) -> CliResult<PathBuf> {
        // Determine target directory (prefer project if available)
        let use_project = self.project_path.is_some();
        let target_dir = self.commands_dir(use_project);

        // Create directory if it doesn't exist
        fs::create_dir_all(&target_dir).map_err(CliError::Io)?;

        // Save as JSON with commands wrapper
        let file_name = format!("{}.json", cmd.id);
        let file_path = target_dir.join(&file_name);

        // Create a wrapper with commands array
        let config = serde_json::json!({
            "commands": [cmd]
        });

        // Serialize to JSON
        let json_str =
            serde_json::to_string_pretty(&config).map_err(|e| CliError::Internal(e.to_string()))?;

        // Write file
        fs::write(&file_path, json_str).map_err(CliError::Io)?;

        Ok(file_path)
    }

    /// Delete a command from storage
    pub fn delete_command(&self, command_id: &str) -> CliResult<()> {
        // Try project storage first
        if let Some(project_path) = &self.project_path {
            let project_commands_dir = project_path.join("commands");
            let file_path = project_commands_dir.join(format!("{}.json", command_id));
            if file_path.exists() {
                fs::remove_file(&file_path).map_err(CliError::Io)?;
                return Ok(());
            }
        }

        // Try global storage
        let global_commands_dir = self.commands_dir(false);
        let file_path = global_commands_dir.join(format!("{}.json", command_id));
        if file_path.exists() {
            fs::remove_file(&file_path).map_err(CliError::Io)?;
            return Ok(());
        }

        Err(CliError::InvalidArgument {
            message: format!("Command '{}' not found in storage", command_id),
        })
    }

    /// Get the global storage path
    pub fn global_path(&self) -> &PathBuf {
        &self.global_path
    }

    /// Get the project storage path if available
    pub fn project_path(&self) -> Option<&PathBuf> {
        self.project_path.as_ref()
    }
}

impl Default for CustomCommandsStorage {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            // Fallback if storage initialization fails
            Self {
                global_path: PathBuf::from("."),
                project_path: None,
            }
        })
    }
}
