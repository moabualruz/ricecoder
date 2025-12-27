//! Configuration and asset loaders for RiceCoder
//!
//! This module provides loaders for various configuration files and assets:
//! - Agents: AI agent definitions with system prompts
//! - Auth: Provider authentication credentials
//! - Commands: Slash commands with frontmatter configuration
//! - LSP: Language Server Protocol server configurations
//! - Models: AI model definitions with pricing and capabilities
//! - Themes: JSON theme files with color definitions
//! - Prompts: Template prompts organized by category
//! - Tips: User tips displayed in the UI
//! - Tools: External tool descriptions from text files

pub mod agents;
pub mod auth;
pub mod commands;
pub mod lsp;
pub mod models;
pub mod prompts;
pub mod themes;
pub mod tips;
pub mod tools;

pub use agents::{Agent, AgentLoader};
pub use auth::{AuthLoader, ProviderAuth, ProvidersAuth};
pub use commands::{Command, CommandLoader};
pub use lsp::{global_lsp_configs, LspConfig, LspConfigLoader};
pub use models::{Model, ModelLoader, ModelPricing, Provider};
pub use prompts::{PromptCategory, PromptLoader};
pub use themes::{Theme, ThemeLoader};
pub use tips::TipsLoader;
pub use tools::{global_tool_descriptions, ToolDescriptionLoader};
