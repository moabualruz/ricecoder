//! Embedded default configurations
//!
//! This module provides default configurations that are baked into the binary.
//! On first run, these are copied to the user's config folder as templates.
//! Users can then edit these files, and their changes will override the defaults.

use std::fs;
use std::path::Path;

use crate::error::{IoOperation, StorageError, StorageResult};
use crate::manager::PathResolver;
use crate::types::{ConfigSubdirectory, RuntimeStorageType, StorageDirectory};

/// Default main configuration file content
pub const DEFAULT_CONFIG: &str = r#"# RiceCoder Configuration
# This file is auto-generated on first run. Edit freely.

# Default provider and model
# zen: OpenCode's free provider (no API key required)
# anthropic, openai, ollama, google: Require API keys
provider: zen
model: zen-gpt4

# Provider endpoints (uncomment to customize)
# providers:
#   zen:
#     base_url: https://opencode.ai/zen/v1
#   anthropic:
#     api_key: ${ANTHROPIC_API_KEY}
#   openai:
#     api_key: ${OPENAI_API_KEY}
#   ollama:
#     base_url: http://localhost:11434

# TUI settings
tui:
  theme: dark
  animations: true
  mouse: true
  vim_mode: false

# Accessibility settings
accessibility:
  screen_reader_enabled: false
  high_contrast_mode: false
  animations_disabled: false

# Default LLM parameters
defaults:
  temperature: 0.7
  max_tokens: 4096
"#;

/// Default tips content
pub const DEFAULT_TIPS: &str = r#"# RiceCoder Tips - One per line, # for comments
Type @ followed by a filename to fuzzy search and attach files to your prompt.
Start a message with ! to run shell commands directly (e.g., !ls -la).
Press Tab to cycle between Build (full access) and Plan (read-only) agents.
Use /undo to revert the last message and any file changes made by RiceCoder.
Use /redo to restore previously undone messages and file changes.
Run /share to create a public link to your conversation.
Press Ctrl+V to paste images from your clipboard directly into the prompt.
Press Ctrl+X E or /editor to compose messages in your external editor.
Run /init to auto-generate project rules based on your codebase structure.
Run /models or Ctrl+X M to see and switch between available AI models.
Use /theme or Ctrl+X T to preview and switch between 50+ built-in themes.
Press Ctrl+X N or /new to start a fresh conversation session.
Use /sessions or Ctrl+X L to list and continue previous conversations.
Press Ctrl+C to interrupt the current AI response.
Use /clear to clear the current conversation history.
Type /help to see all available commands.
Press ? in the prompt for context-aware help.
Use Ctrl+R to search through command history.
Press Escape to cancel the current operation.
"#;

/// Default providers auth file content
pub const DEFAULT_PROVIDERS_AUTH: &str = r#"# RiceCoder Provider Authentication
# Store your API keys here. This file is NOT committed to git.
# Use environment variables for better security in production.

providers:
  # Zen (OpenCode's free provider - no API key required)
  zen:
    base_url: https://opencode.ai/zen/v1
  
  # Uncomment and add your API keys for other providers:
  # anthropic:
  #   api_key: sk-ant-...
  # openai:
  #   api_key: sk-...
  # ollama:
  #   base_url: http://localhost:11434
  # google:
  #   api_key: ...
"#;

/// Default build agent
pub const DEFAULT_AGENT_BUILD: &str = r#"---
name: build
description: Full access agent for development work - can read, write, and execute
model: claude-sonnet-4-20250514
tools:
  - read
  - write
  - execute
  - search
  - git
---

You are a software development agent with full access to the codebase.
You can read files, write files, execute commands, search code, and interact with git.

When given a task:
1. Analyze the requirements carefully
2. Search the codebase for relevant context
3. Plan your approach before making changes
4. Make changes incrementally with proper error handling
5. Verify changes work as expected

Always explain your reasoning and ask for clarification when needed.
"#;

/// Default plan agent
pub const DEFAULT_AGENT_PLAN: &str = r#"---
name: plan
description: Read-only agent for analysis and exploration
model: claude-sonnet-4-20250514
tools:
  - read
  - search
---

You are an analysis agent with read-only access to the codebase.
You can read files and search code, but cannot make any modifications.

Your role is to:
1. Analyze code structure and patterns
2. Answer questions about the codebase
3. Suggest improvements and refactoring opportunities
4. Help plan implementation approaches

Be thorough in your analysis and provide clear explanations.
"#;

/// Manager for embedded defaults
pub struct DefaultsManager {
    base_path: std::path::PathBuf,
}

impl DefaultsManager {
    /// Create a new defaults manager for the given base path
    pub fn new(base_path: std::path::PathBuf) -> Self {
        Self { base_path }
    }

    /// Create a defaults manager using the default global path
    pub fn with_default_path() -> StorageResult<Self> {
        let base_path = PathResolver::resolve_global_path()?;
        Ok(Self { base_path })
    }

    /// Check if this is the first run (no config directory exists)
    pub fn is_first_run(&self) -> bool {
        !self.base_path.exists()
    }

    /// Initialize the storage structure and copy defaults
    pub fn initialize(&self) -> StorageResult<()> {
        // Create all top-level directories
        for dir in StorageDirectory::all() {
            self.create_dir(&self.base_path.join(dir.dir_name()))?;
        }

        // Create config subdirectories
        let config_dir = self.base_path.join(StorageDirectory::Config.dir_name());
        for subdir in ConfigSubdirectory::all() {
            self.create_dir(&config_dir.join(subdir.dir_name()))?;
        }

        // Create storage subdirectories
        let storage_dir = self.base_path.join(StorageDirectory::Storage.dir_name());
        for storage_type in RuntimeStorageType::all() {
            self.create_dir(&storage_dir.join(storage_type.dir_name()))?;
        }

        // Write default files (only if they don't exist)
        self.write_default_if_missing(
            &PathResolver::main_config_path(&self.base_path),
            DEFAULT_CONFIG,
        )?;

        self.write_default_if_missing(&PathResolver::tips_path(&self.base_path), DEFAULT_TIPS)?;

        self.write_default_if_missing(
            &PathResolver::auth_providers_path(&self.base_path),
            DEFAULT_PROVIDERS_AUTH,
        )?;

        // Write default agents
        let agents_dir = PathResolver::config_subdir(&self.base_path, ConfigSubdirectory::Agents);
        self.write_default_if_missing(&agents_dir.join("build.md"), DEFAULT_AGENT_BUILD)?;
        self.write_default_if_missing(&agents_dir.join("plan.md"), DEFAULT_AGENT_PLAN)?;

        Ok(())
    }

    /// Create a directory if it doesn't exist
    fn create_dir(&self, path: &Path) -> StorageResult<()> {
        if !path.exists() {
            fs::create_dir_all(path)
                .map_err(|e| StorageError::directory_creation_failed(path.to_path_buf(), e))?;
        }
        Ok(())
    }

    /// Write a default file only if it doesn't already exist
    fn write_default_if_missing(&self, path: &Path, content: &str) -> StorageResult<()> {
        if !path.exists() {
            // Ensure parent directory exists
            if let Some(parent) = path.parent() {
                self.create_dir(parent)?;
            }

            fs::write(path, content)
                .map_err(|e| StorageError::io_error(path.to_path_buf(), IoOperation::Write, e))?;
        }
        Ok(())
    }

    /// Get embedded default for a specific file (for fallback when file not found)
    pub fn get_embedded_default(file_type: EmbeddedDefault) -> &'static str {
        match file_type {
            EmbeddedDefault::Config => DEFAULT_CONFIG,
            EmbeddedDefault::Tips => DEFAULT_TIPS,
            EmbeddedDefault::ProvidersAuth => DEFAULT_PROVIDERS_AUTH,
            EmbeddedDefault::AgentBuild => DEFAULT_AGENT_BUILD,
            EmbeddedDefault::AgentPlan => DEFAULT_AGENT_PLAN,
        }
    }
}

/// Types of embedded default files
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbeddedDefault {
    /// Main configuration file
    Config,
    /// Tips file
    Tips,
    /// Provider authentication file
    ProvidersAuth,
    /// Build agent definition
    AgentBuild,
    /// Plan agent definition
    AgentPlan,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_is_first_run() {
        let temp_dir = TempDir::new().unwrap();
        let non_existent = temp_dir.path().join("does_not_exist");
        let manager = DefaultsManager::new(non_existent);
        assert!(manager.is_first_run());
    }

    #[test]
    fn test_initialize_creates_structure() {
        let temp_dir = TempDir::new().unwrap();
        let manager = DefaultsManager::new(temp_dir.path().to_path_buf());

        manager.initialize().unwrap();

        // Check directories exist
        assert!(temp_dir.path().join("config").exists());
        assert!(temp_dir.path().join("auth").exists());
        assert!(temp_dir.path().join("storage").exists());
        assert!(temp_dir.path().join("logs").exists());
        assert!(temp_dir.path().join("cache").exists());

        // Check config subdirectories
        assert!(temp_dir.path().join("config/agents").exists());
        assert!(temp_dir.path().join("config/commands").exists());
        assert!(temp_dir.path().join("config/themes").exists());

        // Check default files
        assert!(temp_dir.path().join("config/config.yaml").exists());
        assert!(temp_dir.path().join("config/tips.txt").exists());
        assert!(temp_dir.path().join("auth/providers.yaml").exists());
        assert!(temp_dir.path().join("config/agents/build.md").exists());
        assert!(temp_dir.path().join("config/agents/plan.md").exists());
    }

    #[test]
    fn test_does_not_overwrite_existing() {
        let temp_dir = TempDir::new().unwrap();
        let manager = DefaultsManager::new(temp_dir.path().to_path_buf());

        // Create config dir and custom config
        fs::create_dir_all(temp_dir.path().join("config")).unwrap();
        fs::write(
            temp_dir.path().join("config/config.yaml"),
            "custom: content",
        )
        .unwrap();

        // Initialize
        manager.initialize().unwrap();

        // Check custom content is preserved
        let content = fs::read_to_string(temp_dir.path().join("config/config.yaml")).unwrap();
        assert_eq!(content, "custom: content");
    }
}
