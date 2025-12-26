//! Command loader for RiceCoder
//!
//! Loads slash command definitions from `config/commands/*.md` files.
//! Each command file has YAML frontmatter with metadata and a markdown body
//! containing the command instructions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{StorageError, StorageResult};

/// Command definition loaded from a markdown file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    /// Command name (derived from filename, used as /name)
    pub name: String,
    /// Description from frontmatter
    pub description: String,
    /// Optional model override for this command
    #[serde(default)]
    pub model: Option<String>,
    /// Whether this command runs as a subtask
    #[serde(default)]
    pub subtask: bool,
    /// Optional argument hint for the command
    #[serde(default)]
    pub argument_hint: Option<String>,
    /// Command instructions (markdown body)
    pub instructions: String,
}

/// Frontmatter structure for command files
#[derive(Debug, Deserialize)]
struct CommandFrontmatter {
    description: String,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    subtask: bool,
    #[serde(default, rename = "argument-hint")]
    argument_hint: Option<String>,
}

/// Loader for command configuration files
pub struct CommandLoader {
    config_dir: PathBuf,
}

impl CommandLoader {
    /// Create a new command loader with the given config directory
    pub fn new(config_dir: PathBuf) -> Self {
        Self { config_dir }
    }

    /// Create a command loader using the default config path
    ///
    /// Priority: Project (.rice/commands/) > Global (~/Documents/.ricecoder/config/commands/)
    pub fn with_default_path() -> Self {
        use crate::manager::PathResolver;
        use crate::types::{ConfigSubdirectory, StorageDirectory};

        // First try project-local .rice/commands/
        let project_dir = PathResolver::resolve_project_path()
            .join(ConfigSubdirectory::Commands.dir_name());
        if project_dir.exists() {
            return Self::new(project_dir);
        }

        // Fall back to global ~/Documents/.ricecoder/config/commands/
        if let Ok(global_path) = PathResolver::resolve_global_path() {
            let global_dir = global_path
                .join(StorageDirectory::Config.dir_name())
                .join(ConfigSubdirectory::Commands.dir_name());
            return Self::new(global_dir);
        }

        // Last resort: current directory's config/commands (for backward compat)
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self::new(cwd.join("config").join("commands"))
    }

    /// Load commands from both global and project paths, with project overriding global
    pub fn load_all_merged(&self) -> StorageResult<HashMap<String, Command>> {
        use crate::manager::PathResolver;
        use crate::types::{ConfigSubdirectory, StorageDirectory};

        let mut commands = HashMap::new();

        // Load from global path first
        if let Ok(global_path) = PathResolver::resolve_global_path() {
            let global_dir = global_path
                .join(StorageDirectory::Config.dir_name())
                .join(ConfigSubdirectory::Commands.dir_name());
            if global_dir.exists() {
                let global_loader = CommandLoader::new(global_dir);
                if let Ok(global_commands) = global_loader.load_all() {
                    commands.extend(global_commands);
                }
            }
        }

        // Load from project path (overrides global)
        let project_dir = PathResolver::resolve_project_path()
            .join(ConfigSubdirectory::Commands.dir_name());
        if project_dir.exists() {
            let project_loader = CommandLoader::new(project_dir);
            if let Ok(project_commands) = project_loader.load_all() {
                commands.extend(project_commands);
            }
        }

        Ok(commands)
    }

    /// Load all commands from the config directory
    pub fn load_all(&self) -> StorageResult<HashMap<String, Command>> {
        let mut commands = HashMap::new();

        if !self.config_dir.exists() {
            return Ok(commands);
        }

        let entries = fs::read_dir(&self.config_dir).map_err(|e| {
            StorageError::io_error(
                self.config_dir.clone(),
                crate::error::IoOperation::Read,
                e,
            )
        })?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "md") {
                if let Ok(command) = self.load_from_file(&path) {
                    commands.insert(command.name.clone(), command);
                }
            }
        }

        Ok(commands)
    }

    /// Load a single command from a file
    pub fn load_from_file(&self, path: &Path) -> StorageResult<Command> {
        let content = fs::read_to_string(path).map_err(|e| {
            StorageError::io_error(path.to_path_buf(), crate::error::IoOperation::Read, e)
        })?;

        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        self.parse_command(&content, name, path)
    }

    /// Load a specific command by name
    pub fn load(&self, name: &str) -> StorageResult<Command> {
        let path = self.config_dir.join(format!("{}.md", name));
        self.load_from_file(&path)
    }

    /// Parse command content with YAML frontmatter
    fn parse_command(&self, content: &str, name: String, path: &Path) -> StorageResult<Command> {
        let (frontmatter, body) = Self::split_frontmatter(content)?;

        let meta: CommandFrontmatter = serde_yaml::from_str(&frontmatter).map_err(|e| {
            StorageError::parse_error(path.to_path_buf(), "YAML frontmatter", e.to_string())
        })?;

        Ok(Command {
            name,
            description: meta.description,
            model: meta.model,
            subtask: meta.subtask,
            argument_hint: meta.argument_hint,
            instructions: body.trim().to_string(),
        })
    }

    /// Split content into frontmatter and body
    fn split_frontmatter(content: &str) -> StorageResult<(String, String)> {
        let content = content.trim();

        if !content.starts_with("---") {
            return Err(StorageError::Internal(
                "Missing YAML frontmatter delimiter".to_string(),
            ));
        }

        let rest = &content[3..];
        if let Some(end_idx) = rest.find("\n---") {
            let frontmatter = rest[..end_idx].trim().to_string();
            let body = rest[end_idx + 4..].to_string();
            Ok((frontmatter, body))
        } else {
            Err(StorageError::Internal(
                "Missing closing frontmatter delimiter".to_string(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_command() {
        let content = r#"---
description: git commit and push
model: opencode/glm-4.6
subtask: true
---

commit and push

make sure it includes a prefix
"#;
        let loader = CommandLoader::with_default_path();
        let cmd = loader
            .parse_command(content, "commit".to_string(), Path::new("commit.md"))
            .unwrap();

        assert_eq!(cmd.name, "commit");
        assert_eq!(cmd.description, "git commit and push");
        assert_eq!(cmd.model, Some("opencode/glm-4.6".to_string()));
        assert!(cmd.subtask);
        assert!(cmd.instructions.contains("commit and push"));
    }
}
