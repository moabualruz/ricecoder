//! Prompt loader for RiceCoder
//!
//! Loads prompt templates from `config/prompts/{category}/*.txt` files.
//! Prompts are organized by category (session, agent, tool, command).

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{StorageError, StorageResult};

/// Prompt categories matching the directory structure
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PromptCategory {
    /// Session-level prompts (system prompts, welcome messages)
    Session,
    /// Agent-specific prompts
    Agent,
    /// Tool-related prompts
    Tool,
    /// Command prompts
    Command,
}

impl PromptCategory {
    /// Get the directory name for this category
    pub fn dir_name(&self) -> &'static str {
        match self {
            Self::Session => "session",
            Self::Agent => "agent",
            Self::Tool => "tool",
            Self::Command => "command",
        }
    }

    /// Parse category from directory name
    pub fn from_dir_name(name: &str) -> Option<Self> {
        match name {
            "session" => Some(Self::Session),
            "agent" => Some(Self::Agent),
            "tool" => Some(Self::Tool),
            "command" => Some(Self::Command),
            _ => None,
        }
    }
}

/// Loader for prompt templates
pub struct PromptLoader {
    config_dir: PathBuf,
}

impl PromptLoader {
    /// Create a new prompt loader with the given config directory
    pub fn new(config_dir: PathBuf) -> Self {
        Self { config_dir }
    }

    /// Create a prompt loader using the default config path
    pub fn with_default_path() -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let config_dir = cwd.join("config").join("prompts");
        Self::new(config_dir)
    }

    /// Load all prompts for a specific category
    pub fn load_category(&self, category: PromptCategory) -> StorageResult<HashMap<String, String>> {
        let mut prompts = HashMap::new();
        let category_dir = self.config_dir.join(category.dir_name());

        if !category_dir.exists() {
            return Ok(prompts);
        }

        let entries = fs::read_dir(&category_dir).map_err(|e| {
            StorageError::io_error(category_dir.clone(), crate::error::IoOperation::Read, e)
        })?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "txt") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                        prompts.insert(name.to_string(), content);
                    }
                }
            }
        }

        Ok(prompts)
    }

    /// Load all prompts from all categories
    pub fn load_all(&self) -> StorageResult<HashMap<PromptCategory, HashMap<String, String>>> {
        let mut all_prompts = HashMap::new();

        for category in [
            PromptCategory::Session,
            PromptCategory::Agent,
            PromptCategory::Tool,
            PromptCategory::Command,
        ] {
            let prompts = self.load_category(category)?;
            if !prompts.is_empty() {
                all_prompts.insert(category, prompts);
            }
        }

        Ok(all_prompts)
    }

    /// Load a specific prompt by category and name
    pub fn load(&self, category: PromptCategory, name: &str) -> StorageResult<String> {
        let path = self
            .config_dir
            .join(category.dir_name())
            .join(format!("{}.txt", name));

        fs::read_to_string(&path).map_err(|e| {
            StorageError::io_error(path, crate::error::IoOperation::Read, e)
        })
    }

    /// List all available prompt names in a category
    pub fn list_prompts(&self, category: PromptCategory) -> StorageResult<Vec<String>> {
        let mut names = Vec::new();
        let category_dir = self.config_dir.join(category.dir_name());

        if !category_dir.exists() {
            return Ok(names);
        }

        let entries = fs::read_dir(&category_dir).map_err(|e| {
            StorageError::io_error(category_dir.clone(), crate::error::IoOperation::Read, e)
        })?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "txt") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    names.push(name.to_string());
                }
            }
        }

        names.sort();
        Ok(names)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_dir_names() {
        assert_eq!(PromptCategory::Session.dir_name(), "session");
        assert_eq!(PromptCategory::Agent.dir_name(), "agent");
        assert_eq!(PromptCategory::Tool.dir_name(), "tool");
        assert_eq!(PromptCategory::Command.dir_name(), "command");
    }

    #[test]
    fn test_category_from_dir_name() {
        assert_eq!(
            PromptCategory::from_dir_name("session"),
            Some(PromptCategory::Session)
        );
        assert_eq!(PromptCategory::from_dir_name("unknown"), None);
    }
}
