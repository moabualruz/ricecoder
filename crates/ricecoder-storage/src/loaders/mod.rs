//! Configuration and asset loaders for RiceCoder
//!
//! This module provides loaders for various configuration files and assets:
//! - Agents: AI agent definitions with system prompts
//! - Auth: Provider authentication credentials
//! - Commands: Slash commands with frontmatter configuration
//! - Themes: JSON theme files with color definitions
//! - Prompts: Template prompts organized by category
//! - Tips: User tips displayed in the UI

pub mod agents;
pub mod auth;
pub mod commands;
pub mod prompts;
pub mod themes;
pub mod tips;

pub use agents::{Agent, AgentLoader};
pub use auth::{AuthLoader, ProviderAuth, ProvidersAuth};
pub use commands::{Command, CommandLoader};
pub use prompts::{PromptCategory, PromptLoader};
pub use themes::{Theme, ThemeLoader};
pub use tips::TipsLoader;
