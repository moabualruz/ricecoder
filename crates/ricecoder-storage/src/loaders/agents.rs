//! Agent loader for RiceCoder
//!
//! Loads agent definitions from `config/agents/*.md` files.
//! Each agent file has YAML frontmatter with metadata and a markdown body
//! containing the system prompt.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{StorageError, StorageResult};

/// Agent definition loaded from a markdown file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    /// Agent name (derived from filename)
    pub name: String,
    /// Description from frontmatter
    pub description: String,
    /// Optional model override
    #[serde(default)]
    pub model: Option<String>,
    /// System prompt (markdown body)
    pub system_prompt: String,
}

/// Frontmatter structure for agent files
#[derive(Debug, Deserialize)]
struct AgentFrontmatter {
    description: String,
    #[serde(default)]
    model: Option<String>,
}

/// Loader for agent configuration files
pub struct AgentLoader {
    config_dir: PathBuf,
}

impl AgentLoader {
    /// Create a new agent loader with the given config directory
    pub fn new(config_dir: PathBuf) -> Self {
        Self { config_dir }
    }

    /// Create an agent loader using the default config path
    ///
    /// Priority: Project (.rice/agents/) > Global (~/Documents/.ricecoder/config/agents/)
    pub fn with_default_path() -> Self {
        use crate::manager::PathResolver;
        use crate::types::{ConfigSubdirectory, StorageDirectory};

        // First try project-local .rice/agents/
        let project_dir = PathResolver::resolve_project_path()
            .join(ConfigSubdirectory::Agents.dir_name());
        if project_dir.exists() {
            return Self::new(project_dir);
        }

        // Fall back to global ~/Documents/.ricecoder/config/agents/
        if let Ok(global_path) = PathResolver::resolve_global_path() {
            let global_dir = global_path
                .join(StorageDirectory::Config.dir_name())
                .join(ConfigSubdirectory::Agents.dir_name());
            return Self::new(global_dir);
        }

        // Last resort: current directory's config/agents (for backward compat)
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self::new(cwd.join("config").join("agents"))
    }

    /// Create an agent loader that merges global and project agents
    ///
    /// Project agents override global agents with the same name
    pub fn with_merged_paths() -> Self {
        // This returns a loader for the primary path; use load_all_merged() for merging
        Self::with_default_path()
    }

    /// Load agents from both global and project paths, with project overriding global
    pub fn load_all_merged(&self) -> StorageResult<HashMap<String, Agent>> {
        use crate::manager::PathResolver;
        use crate::types::{ConfigSubdirectory, StorageDirectory};

        let mut agents = HashMap::new();

        // Load from global path first
        if let Ok(global_path) = PathResolver::resolve_global_path() {
            let global_dir = global_path
                .join(StorageDirectory::Config.dir_name())
                .join(ConfigSubdirectory::Agents.dir_name());
            if global_dir.exists() {
                let global_loader = AgentLoader::new(global_dir);
                if let Ok(global_agents) = global_loader.load_all() {
                    agents.extend(global_agents);
                }
            }
        }

        // Load from project path (overrides global)
        let project_dir = PathResolver::resolve_project_path()
            .join(ConfigSubdirectory::Agents.dir_name());
        if project_dir.exists() {
            let project_loader = AgentLoader::new(project_dir);
            if let Ok(project_agents) = project_loader.load_all() {
                agents.extend(project_agents);
            }
        }

        Ok(agents)
    }

    /// Load all agents from the config directory
    pub fn load_all(&self) -> StorageResult<HashMap<String, Agent>> {
        let mut agents = HashMap::new();

        if !self.config_dir.exists() {
            return Ok(agents);
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
                if let Ok(agent) = self.load_from_file(&path) {
                    agents.insert(agent.name.clone(), agent);
                }
            }
        }

        Ok(agents)
    }

    /// Load a single agent from a file
    pub fn load_from_file(&self, path: &Path) -> StorageResult<Agent> {
        let content = fs::read_to_string(path).map_err(|e| {
            StorageError::io_error(path.to_path_buf(), crate::error::IoOperation::Read, e)
        })?;

        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        self.parse_agent(&content, name, path)
    }

    /// Load a specific agent by name
    pub fn load(&self, name: &str) -> StorageResult<Agent> {
        let path = self.config_dir.join(format!("{}.md", name));
        self.load_from_file(&path)
    }

    /// Parse agent content with YAML frontmatter
    fn parse_agent(&self, content: &str, name: String, path: &Path) -> StorageResult<Agent> {
        // Split frontmatter and body
        let (frontmatter, body) = Self::split_frontmatter(content)?;

        // Parse frontmatter
        let meta: AgentFrontmatter = serde_yaml::from_str(&frontmatter).map_err(|e| {
            StorageError::parse_error(path.to_path_buf(), "YAML frontmatter", e.to_string())
        })?;

        Ok(Agent {
            name,
            description: meta.description,
            model: meta.model,
            system_prompt: body.trim().to_string(),
        })
    }

    /// Split content into frontmatter and body
    fn split_frontmatter(content: &str) -> StorageResult<(String, String)> {
        let content = content.trim();

        // Check for frontmatter delimiter
        if !content.starts_with("---") {
            return Err(StorageError::Internal(
                "Missing YAML frontmatter delimiter".to_string(),
            ));
        }

        // Find the closing delimiter
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
    fn test_parse_agent() {
        let content = r#"---
description: Test agent for documentation
model: gpt-4
---

You are a helpful assistant.

Be concise and clear.
"#;
        let loader = AgentLoader::with_default_path();
        let agent = loader
            .parse_agent(content, "test".to_string(), Path::new("test.md"))
            .unwrap();

        assert_eq!(agent.name, "test");
        assert_eq!(agent.description, "Test agent for documentation");
        assert_eq!(agent.model, Some("gpt-4".to_string()));
        assert!(agent.system_prompt.contains("helpful assistant"));
    }
}
