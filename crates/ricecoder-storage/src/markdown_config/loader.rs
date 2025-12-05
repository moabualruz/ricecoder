//! Configuration loader for discovering and loading markdown configurations
//!
//! This module provides the [`ConfigurationLoader`] which discovers and loads
//! markdown configuration files from standard locations.
//!
//! # Discovery Locations
//!
//! Configuration files are discovered in the following locations (in priority order):
//!
//! 1. **Project-level**: `projects/ricecoder/.agent/`
//! 2. **User-level**: `~/.ricecoder/agents/`, `~/.ricecoder/modes/`, `~/.ricecoder/commands/`
//! 3. **System-level**: `/etc/ricecoder/agents/` (Linux/macOS)
//!
//! # File Patterns
//!
//! - `*.agent.md` - Agent configurations
//! - `*.mode.md` - Mode configurations
//! - `*.command.md` - Command configurations
//!
//! # Usage
//!
//! ```ignore
//! use ricecoder_storage::markdown_config::{ConfigurationLoader, ConfigRegistry};
//! use std::sync::Arc;
//! use std::path::PathBuf;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let registry = Arc::new(ConfigRegistry::new());
//!     let loader = ConfigurationLoader::new(registry.clone());
//!
//!     // Discover and load configurations
//!     let paths = vec![
//!         PathBuf::from("~/.ricecoder/agents"),
//!         PathBuf::from("projects/ricecoder/.agent"),
//!     ];
//!
//!     loader.load_all(&paths).await?;
//!
//!     // Query loaded configurations
//!     if let Some(agent) = registry.get_agent("code-review") {
//!         println!("Found agent: {}", agent.name);
//!     }
//!
//!     Ok(())
//! }
//! ```

use crate::markdown_config::error::{MarkdownConfigError, MarkdownConfigResult};
use crate::markdown_config::parser::MarkdownParser;
use crate::markdown_config::registry::ConfigRegistry;
use crate::markdown_config::types::{AgentConfig, CommandConfig, ModeConfig};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{debug, warn};

/// Configuration file type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFileType {
    /// Agent configuration file (*.agent.md)
    Agent,
    /// Mode configuration file (*.mode.md)
    Mode,
    /// Command configuration file (*.command.md)
    Command,
}

impl ConfigFileType {
    /// Get the file pattern for this configuration type
    pub fn pattern(&self) -> &'static str {
        match self {
            ConfigFileType::Agent => "*.agent.md",
            ConfigFileType::Mode => "*.mode.md",
            ConfigFileType::Command => "*.command.md",
        }
    }

    /// Detect configuration type from file path
    pub fn from_path(path: &Path) -> Option<Self> {
        let file_name = path.file_name()?.to_str()?;

        if file_name.ends_with(".agent.md") {
            Some(ConfigFileType::Agent)
        } else if file_name.ends_with(".mode.md") {
            Some(ConfigFileType::Mode)
        } else if file_name.ends_with(".command.md") {
            Some(ConfigFileType::Command)
        } else {
            None
        }
    }
}

/// Discovered configuration file
#[derive(Debug, Clone)]
pub struct ConfigFile {
    /// Path to the configuration file
    pub path: PathBuf,
    /// Type of configuration
    pub config_type: ConfigFileType,
}

impl ConfigFile {
    /// Create a new configuration file reference
    pub fn new(path: PathBuf, config_type: ConfigFileType) -> Self {
        Self { path, config_type }
    }
}

/// Configuration loader for discovering and loading markdown configurations
#[derive(Debug)]
pub struct ConfigurationLoader {
    parser: MarkdownParser,
    registry: Arc<ConfigRegistry>,
}

impl ConfigurationLoader {
    /// Create a new configuration loader
    pub fn new(registry: Arc<ConfigRegistry>) -> Self {
        Self {
            parser: MarkdownParser::new(),
            registry,
        }
    }

    /// Discover configuration files in the given paths
    ///
    /// # Arguments
    /// * `paths` - Directories to search for configuration files
    ///
    /// # Returns
    /// A vector of discovered configuration files
    pub fn discover(&self, paths: &[PathBuf]) -> MarkdownConfigResult<Vec<ConfigFile>> {
        let mut discovered = Vec::new();

        for path in paths {
            if !path.exists() {
                debug!("Configuration path does not exist: {}", path.display());
                continue;
            }

            if !path.is_dir() {
                debug!("Configuration path is not a directory: {}", path.display());
                continue;
            }

            self.discover_in_directory(path, &mut discovered)?;
        }

        debug!("Discovered {} configuration files", discovered.len());
        Ok(discovered)
    }

    /// Discover configuration files in a specific directory
    fn discover_in_directory(
        &self,
        dir: &Path,
        discovered: &mut Vec<ConfigFile>,
    ) -> MarkdownConfigResult<()> {
        match std::fs::read_dir(dir) {
            Ok(entries) => {
                for entry in entries {
                    match entry {
                        Ok(entry) => {
                            let path = entry.path();

                            // Skip directories
                            if path.is_dir() {
                                continue;
                            }

                            // Check if file matches any configuration pattern
                            if let Some(config_type) = ConfigFileType::from_path(&path) {
                                discovered.push(ConfigFile::new(path, config_type));
                            }
                        }
                        Err(e) => {
                            warn!("Failed to read directory entry: {}", e);
                        }
                    }
                }
                Ok(())
            }
            Err(e) => {
                warn!(
                    "Failed to read configuration directory {}: {}",
                    dir.display(),
                    e
                );
                Ok(())
            }
        }
    }

    /// Load a configuration file
    ///
    /// # Arguments
    /// * `file` - The configuration file to load
    ///
    /// # Returns
    /// The loaded configuration (as an enum to support different types)
    pub async fn load(&self, file: &ConfigFile) -> MarkdownConfigResult<LoadedConfig> {
        // Read file content
        let content = tokio::fs::read_to_string(&file.path)
            .await
            .map_err(|e| {
                MarkdownConfigError::load_error(
                    &file.path,
                    format!("Failed to read file: {}", e),
                )
            })?;

        // Parse markdown
        let parsed = self
            .parser
            .parse_with_context(&content, Some(&file.path))?;

        // Extract frontmatter
        let frontmatter = parsed.frontmatter.ok_or_else(|| {
            MarkdownConfigError::load_error(
                &file.path,
                "Configuration file must have YAML frontmatter",
            )
        })?;

        // Parse YAML frontmatter
        let yaml_value: serde_yaml::Value = serde_yaml::from_str(&frontmatter).map_err(|e| {
            MarkdownConfigError::load_error(
                &file.path,
                format!("Failed to parse YAML frontmatter: {}", e),
            )
        })?;

        // Load configuration based on type
        let config = match file.config_type {
            ConfigFileType::Agent => {
                let mut agent_config: AgentConfig =
                    serde_yaml::from_value(yaml_value).map_err(|e| {
                        MarkdownConfigError::load_error(
                            &file.path,
                            format!("Failed to deserialize agent configuration: {}", e),
                        )
                    })?;

                // Use markdown body as prompt if not provided in frontmatter
                if agent_config.prompt.is_empty() {
                    agent_config.prompt = parsed.content;
                }

                LoadedConfig::Agent(agent_config)
            }
            ConfigFileType::Mode => {
                let mut mode_config: ModeConfig =
                    serde_yaml::from_value(yaml_value).map_err(|e| {
                        MarkdownConfigError::load_error(
                            &file.path,
                            format!("Failed to deserialize mode configuration: {}", e),
                        )
                    })?;

                // Use markdown body as prompt if not provided in frontmatter
                if mode_config.prompt.is_empty() {
                    mode_config.prompt = parsed.content;
                }

                LoadedConfig::Mode(mode_config)
            }
            ConfigFileType::Command => {
                let mut command_config: CommandConfig =
                    serde_yaml::from_value(yaml_value).map_err(|e| {
                        MarkdownConfigError::load_error(
                            &file.path,
                            format!("Failed to deserialize command configuration: {}", e),
                        )
                    })?;

                // Use markdown body as template if not provided in frontmatter
                if command_config.template.is_empty() {
                    command_config.template = parsed.content;
                }

                LoadedConfig::Command(command_config)
            }
        };

        Ok(config)
    }

    /// Register a loaded configuration with the registry
    ///
    /// # Arguments
    /// * `config` - The loaded configuration to register
    pub fn register(&self, config: LoadedConfig) -> MarkdownConfigResult<()> {
        match config {
            LoadedConfig::Agent(agent) => self.registry.register_agent(agent),
            LoadedConfig::Mode(mode) => self.registry.register_mode(mode),
            LoadedConfig::Command(command) => self.registry.register_command(command),
        }
    }

    /// Load and register all configurations from the given paths
    ///
    /// # Arguments
    /// * `paths` - Directories to search for configuration files
    ///
    /// # Returns
    /// A tuple of (successful_count, error_count, errors)
    pub async fn load_all(
        &self,
        paths: &[PathBuf],
    ) -> MarkdownConfigResult<(usize, usize, Vec<(PathBuf, String)>)> {
        let files = self.discover(paths)?;

        let mut success_count = 0;
        let mut error_count = 0;
        let mut errors = Vec::new();

        for file in files {
            match self.load(&file).await {
                Ok(config) => {
                    match self.register(config) {
                        Ok(_) => {
                            success_count += 1;
                            debug!("Registered configuration from {}", file.path.display());
                        }
                        Err(e) => {
                            error_count += 1;
                            let error_msg = e.to_string();
                            warn!(
                                "Failed to register configuration from {}: {}",
                                file.path.display(),
                                error_msg
                            );
                            errors.push((file.path, error_msg));
                        }
                    }
                }
                Err(e) => {
                    error_count += 1;
                    let error_msg = e.to_string();
                    warn!(
                        "Failed to load configuration from {}: {}",
                        file.path.display(),
                        error_msg
                    );
                    errors.push((file.path, error_msg));
                }
            }
        }

        debug!(
            "Configuration loading complete: {} successful, {} failed",
            success_count, error_count
        );

        Ok((success_count, error_count, errors))
    }

    /// Get the registry
    pub fn registry(&self) -> Arc<ConfigRegistry> {
        self.registry.clone()
    }
}

/// Loaded configuration (can be any of the three types)
#[derive(Debug)]
pub enum LoadedConfig {
    /// Loaded agent configuration
    Agent(AgentConfig),
    /// Loaded mode configuration
    Mode(ModeConfig),
    /// Loaded command configuration
    Command(CommandConfig),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_agent_file(dir: &Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(format!("{}.agent.md", name));
        fs::write(&path, content).unwrap();
        path
    }

    fn create_test_mode_file(dir: &Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(format!("{}.mode.md", name));
        fs::write(&path, content).unwrap();
        path
    }

    fn create_test_command_file(dir: &Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(format!("{}.command.md", name));
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn test_config_file_type_detection() {
        let agent_path = PathBuf::from("test.agent.md");
        assert_eq!(ConfigFileType::from_path(&agent_path), Some(ConfigFileType::Agent));

        let mode_path = PathBuf::from("test.mode.md");
        assert_eq!(ConfigFileType::from_path(&mode_path), Some(ConfigFileType::Mode));

        let command_path = PathBuf::from("test.command.md");
        assert_eq!(
            ConfigFileType::from_path(&command_path),
            Some(ConfigFileType::Command)
        );

        let other_path = PathBuf::from("test.md");
        assert_eq!(ConfigFileType::from_path(&other_path), None);
    }

    #[test]
    fn test_discover_configuration_files() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        // Create test files
        create_test_agent_file(dir_path, "agent1", "---\nname: agent1\n---\nTest");
        create_test_mode_file(dir_path, "mode1", "---\nname: mode1\n---\nTest");
        create_test_command_file(dir_path, "command1", "---\nname: command1\n---\nTest");
        fs::write(dir_path.join("other.md"), "Not a config").unwrap();

        let registry = Arc::new(ConfigRegistry::new());
        let loader = ConfigurationLoader::new(registry);

        let discovered = loader.discover(&[dir_path.to_path_buf()]).unwrap();

        assert_eq!(discovered.len(), 3);
        assert!(discovered.iter().any(|f| f.config_type == ConfigFileType::Agent));
        assert!(discovered.iter().any(|f| f.config_type == ConfigFileType::Mode));
        assert!(discovered.iter().any(|f| f.config_type == ConfigFileType::Command));
    }

    #[test]
    fn test_discover_nonexistent_directory() {
        let registry = Arc::new(ConfigRegistry::new());
        let loader = ConfigurationLoader::new(registry);

        let nonexistent = PathBuf::from("/nonexistent/path");
        let discovered = loader.discover(&[nonexistent]).unwrap();

        assert_eq!(discovered.len(), 0);
    }

    #[tokio::test]
    async fn test_load_agent_configuration() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        let content = r#"---
name: test-agent
description: A test agent
model: gpt-4
temperature: 0.7
max_tokens: 2000
---
You are a helpful assistant"#;

        create_test_agent_file(dir_path, "test-agent", content);

        let registry = Arc::new(ConfigRegistry::new());
        let loader = ConfigurationLoader::new(registry);

        let file = ConfigFile::new(
            dir_path.join("test-agent.agent.md"),
            ConfigFileType::Agent,
        );

        let loaded = loader.load(&file).await.unwrap();

        match loaded {
            LoadedConfig::Agent(agent) => {
                assert_eq!(agent.name, "test-agent");
                assert_eq!(agent.description, Some("A test agent".to_string()));
                assert_eq!(agent.model, Some("gpt-4".to_string()));
                assert_eq!(agent.temperature, Some(0.7));
                assert_eq!(agent.max_tokens, Some(2000));
                assert_eq!(agent.prompt, "You are a helpful assistant");
            }
            _ => panic!("Expected agent configuration"),
        }
    }

    #[tokio::test]
    async fn test_load_mode_configuration() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        let content = r#"---
name: focus-mode
description: Focus mode
keybinding: C-f
enabled: true
---
Focus on the task at hand"#;

        create_test_mode_file(dir_path, "focus-mode", content);

        let registry = Arc::new(ConfigRegistry::new());
        let loader = ConfigurationLoader::new(registry);

        let file = ConfigFile::new(
            dir_path.join("focus-mode.mode.md"),
            ConfigFileType::Mode,
        );

        let loaded = loader.load(&file).await.unwrap();

        match loaded {
            LoadedConfig::Mode(mode) => {
                assert_eq!(mode.name, "focus-mode");
                assert_eq!(mode.description, Some("Focus mode".to_string()));
                assert_eq!(mode.keybinding, Some("C-f".to_string()));
                assert!(mode.enabled);
                assert_eq!(mode.prompt, "Focus on the task at hand");
            }
            _ => panic!("Expected mode configuration"),
        }
    }

    #[tokio::test]
    async fn test_load_command_configuration() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        let content = r#"---
name: test-command
description: A test command
parameters:
  - name: message
    description: Message to echo
    required: true
keybinding: C-t
---
echo {{message}}"#;

        create_test_command_file(dir_path, "test-command", content);

        let registry = Arc::new(ConfigRegistry::new());
        let loader = ConfigurationLoader::new(registry);

        let file = ConfigFile::new(
            dir_path.join("test-command.command.md"),
            ConfigFileType::Command,
        );

        let loaded = loader.load(&file).await.unwrap();

        match loaded {
            LoadedConfig::Command(command) => {
                assert_eq!(command.name, "test-command");
                assert_eq!(command.description, Some("A test command".to_string()));
                assert_eq!(command.template, "echo {{message}}");
                assert_eq!(command.parameters.len(), 1);
                assert_eq!(command.parameters[0].name, "message");
                assert!(command.parameters[0].required);
            }
            _ => panic!("Expected command configuration"),
        }
    }

    #[tokio::test]
    async fn test_load_missing_frontmatter() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        let content = "# No frontmatter\nJust markdown";

        create_test_agent_file(dir_path, "no-frontmatter", content);

        let registry = Arc::new(ConfigRegistry::new());
        let loader = ConfigurationLoader::new(registry);

        let file = ConfigFile::new(
            dir_path.join("no-frontmatter.agent.md"),
            ConfigFileType::Agent,
        );

        let result = loader.load(&file).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_load_all_configurations() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        let agent_content = r#"---
name: agent1
---
Agent prompt"#;

        let mode_content = r#"---
name: mode1
---
Mode prompt"#;

        create_test_agent_file(dir_path, "agent1", agent_content);
        create_test_mode_file(dir_path, "mode1", mode_content);

        let registry = Arc::new(ConfigRegistry::new());
        let loader = ConfigurationLoader::new(registry.clone());

        let (success, errors, error_list) = loader.load_all(&[dir_path.to_path_buf()]).await.unwrap();

        assert_eq!(success, 2);
        assert_eq!(errors, 0);
        assert_eq!(error_list.len(), 0);

        // Verify configurations were registered
        assert!(registry.has_agent("agent1").unwrap());
        assert!(registry.has_mode("mode1").unwrap());
    }
}
