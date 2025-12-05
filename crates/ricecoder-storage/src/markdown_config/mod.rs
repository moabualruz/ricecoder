//! Markdown Configuration Module
//!
//! Provides markdown-based definition of agents, modes, and commands with YAML frontmatter.
//! Enables users to create custom configurations using familiar markdown syntax.

pub mod error;
pub mod loader;
pub mod parser;
pub mod registry;
pub mod types;
pub mod validation;
pub mod yaml_parser;

// Re-export commonly used types
pub use error::MarkdownConfigError;
pub use loader::{ConfigFile, ConfigFileType, ConfigurationLoader, LoadedConfig};
pub use parser::MarkdownParser;
pub use registry::ConfigRegistry;
pub use types::{AgentConfig, CommandConfig, ModeConfig, ParsedMarkdown, Parameter};
pub use validation::{validate_agent_config, validate_command_config, validate_mode_config};
pub use yaml_parser::YamlParser;
