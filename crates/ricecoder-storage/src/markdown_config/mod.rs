//! Markdown Configuration Module
//!
//! Provides markdown-based definition of agents, modes, and commands with YAML frontmatter.
//! Enables users to create custom configurations using familiar markdown syntax.
//!
//! # Overview
//!
//! The markdown configuration module enables users to define custom agents, modes, and commands
//! using markdown files with YAML frontmatter. This approach combines the readability of markdown
//! with the structure of YAML, making configuration files both human-friendly and machine-parseable.
//!
//! # Configuration File Format
//!
//! All markdown configuration files follow this structure:
//!
//! ```markdown
//! ---
//! name: example-agent
//! description: Example agent configuration
//! model: gpt-4
//! temperature: 0.7
//! max_tokens: 2000
//! ---
//!
//! # Example Agent
//!
//! This is the markdown content that serves as documentation or prompt.
//! ```
//!
//! # Configuration Types
//!
//! ## Agent Configuration
//!
//! Agents are AI-powered components with specific capabilities and parameters.
//!
//! **File Pattern**: `*.agent.md`
//!
//! **Fields**:
//! - `name` (required): Unique identifier for the agent
//! - `description` (optional): Human-readable description
//! - `model` (optional): LLM model to use (e.g., "gpt-4")
//! - `temperature` (optional): Model temperature (0.0-2.0)
//! - `max_tokens` (optional): Maximum response tokens
//! - `tools` (optional): List of available tools
//!
//! ## Mode Configuration
//!
//! Modes define editor behaviors and keybindings for different workflows.
//!
//! **File Pattern**: `*.mode.md`
//!
//! **Fields**:
//! - `name` (required): Unique identifier for the mode
//! - `description` (optional): Human-readable description
//! - `keybinding` (optional): Keyboard shortcut to activate
//! - `enabled` (optional): Whether the mode is enabled (default: true)
//!
//! ## Command Configuration
//!
//! Commands define custom operations with parameters and templates.
//!
//! **File Pattern**: `*.command.md`
//!
//! **Fields**:
//! - `name` (required): Unique identifier for the command
//! - `description` (optional): Human-readable description
//! - `template` (required): Command template with parameter placeholders
//! - `parameters` (optional): List of parameter definitions
//! - `keybinding` (optional): Keyboard shortcut to invoke
//!
//! # Discovery and Loading
//!
//! Configuration files are discovered in the following locations (in priority order):
//!
//! 1. **Project-level**: `projects/ricecoder/.agent/`
//! 2. **User-level**: `~/.ricecoder/agents/`, `~/.ricecoder/modes/`, `~/.ricecoder/commands/`
//! 3. **System-level**: `/etc/ricecoder/agents/` (Linux/macOS)
//!
//! The [`ConfigurationLoader`] automatically discovers and loads all configuration files
//! from these locations, validating each file and registering valid configurations with
//! the [`ConfigRegistry`].
//!
//! # Hot-Reload Support
//!
//! The [`FileWatcher`] monitors configuration directories for changes. When a file is modified:
//!
//! 1. File is detected within 5 seconds
//! 2. Configuration is re-parsed and validated
//! 3. If valid, configuration is updated in the registry
//! 4. If invalid, error is logged and previous configuration is retained
//!
//! This enables runtime configuration updates without requiring application restart.
//!
//! # Usage Examples
//!
//! ## Loading Configurations
//!
//! ```ignore
//! use ricecoder_storage::markdown_config::{ConfigurationLoader, ConfigRegistry};
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let registry = Arc::new(ConfigRegistry::new());
//!     let loader = ConfigurationLoader::new(registry.clone());
//!
//!     // Discover and load configurations
//!     let paths = vec![
//!         std::path::PathBuf::from("~/.ricecoder/agents"),
//!         std::path::PathBuf::from("projects/ricecoder/.agent"),
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
//!
//! ## Parsing Markdown Configuration
//!
//! ```ignore
//! use ricecoder_storage::markdown_config::{MarkdownParser, YamlParser};
//!
//! let markdown_content = r#"---
//! name: example-agent
//! description: Example agent
//! model: gpt-4
//! ---
//!
//! # Example Agent
//! This is the documentation.
//! "#;
//!
//! let parser = MarkdownParser::new();
//! let parsed = parser.parse(markdown_content)?;
//!
//! println!("Frontmatter: {:?}", parsed.frontmatter);
//! println!("Content: {}", parsed.content);
//! ```
//!
//! ## Validating Configuration
//!
//! ```ignore
//! use ricecoder_storage::markdown_config::{AgentConfig, validate_agent_config};
//!
//! let config = AgentConfig {
//!     name: "code-review".to_string(),
//!     description: Some("Code review agent".to_string()),
//!     model: Some("gpt-4".to_string()),
//!     temperature: Some(0.7),
//!     max_tokens: Some(2000),
//!     tools: vec!["syntax-analyzer".to_string()],
//! };
//!
//! validate_agent_config(&config)?;
//! println!("Configuration is valid!");
//! ```
//!
//! # Error Handling
//!
//! The module provides detailed error information for debugging:
//!
//! - **Parsing Errors**: Include file path and line numbers
//! - **Validation Errors**: List all validation failures with field paths
//! - **Loading Errors**: Indicate which files failed and why
//!
//! Invalid configurations are logged but don't prevent loading of other files,
//! enabling graceful degradation when some configurations are malformed.
//!
//! # Configuration File Locations
//!
//! ## Project-Level Configuration
//!
//! Place configuration files in `projects/ricecoder/.agent/`:
//!
//! ```text
//! projects/ricecoder/
//! └── .agent/
//!     ├── examples/
//!     │   ├── code-review.agent.md
//!     │   ├── focus.mode.md
//!     │   └── test.command.md
//!     ├── custom-agent.agent.md
//!     └── custom-mode.mode.md
//! ```
//!
//! ## User-Level Configuration
//!
//! Place configuration files in `~/.ricecoder/`:
//!
//! ```text
//! ~/.ricecoder/
//! ├── agents/
//! │   ├── my-agent.agent.md
//! │   └── code-review.agent.md
//! ├── modes/
//! │   ├── focus.mode.md
//! │   └── debug.mode.md
//! └── commands/
//!     ├── test.command.md
//!     └── build.command.md
//! ```
//!
//! # Best Practices
//!
//! 1. **Use Descriptive Names**: Choose names that clearly indicate purpose
//! 2. **Document Configurations**: Include markdown documentation explaining usage
//! 3. **Validate Early**: Use the validation functions to catch errors early
//! 4. **Organize Files**: Group related configurations in directories
//! 5. **Version Control**: Commit project-level configurations to version control
//! 6. **Test Configurations**: Verify configurations work as expected before deploying
//!
//! # See Also
//!
//! - [`ConfigurationLoader`]: Discovers and loads configuration files
//! - [`ConfigRegistry`]: Central registry for loaded configurations
//! - [`FileWatcher`]: Monitors configuration files for changes
//! - [`MarkdownParser`]: Parses markdown files with YAML frontmatter
//! - [`YamlParser`]: Parses and validates YAML frontmatter

pub mod error;
pub mod integration;
pub mod loader;
pub mod parser;
pub mod registry;
pub mod types;
pub mod validation;
pub mod watcher;
pub mod yaml_parser;

// Re-export commonly used types
pub use error::MarkdownConfigError;
pub use integration::{AgentConfigIntegration, CommandConfigIntegration, ModeConfigIntegration};
pub use loader::{ConfigFile, ConfigFileType, ConfigurationLoader, LoadedConfig};
pub use parser::MarkdownParser;
pub use registry::ConfigRegistry;
pub use types::{AgentConfig, CommandConfig, ModeConfig, ParsedMarkdown, Parameter};
pub use validation::{validate_agent_config, validate_command_config, validate_mode_config};
pub use watcher::FileWatcher;
pub use yaml_parser::YamlParser;
