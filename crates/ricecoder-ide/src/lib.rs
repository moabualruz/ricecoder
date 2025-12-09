//! IDE Integration for RiceCoder
//!
//! This crate provides IDE integration for RiceCoder, enabling seamless integration with
//! popular IDEs and editors (VS Code, vim, neovim, emacs). It implements an LSP-first
//! provider chain that queries external LSP servers for semantic intelligence and falls
//! back through configured rules, built-in providers, and generic features.
//!
//! # Architecture
//!
//! The IDE integration follows a provider chain pattern:
//! 1. External LSP Servers (rust-analyzer, typescript-language-server, pylsp, etc.)
//! 2. Configured IDE Rules (YAML/JSON configuration)
//! 3. Built-in Language Providers (Rust, TypeScript, Python)
//! 4. Generic Text-based Features (fallback for any language)
//!
//! # Configuration
//!
//! IDE integration is configured through YAML/JSON files with support for:
//! - IDE-specific settings (VS Code, vim, neovim, emacs)
//! - Provider chain configuration
//! - LSP server configuration
//! - Custom IDE rules
//!
//! # Example
//!
//! ```ignore
//! use ricecoder_ide::config::ConfigManager;
//! use ricecoder_ide::manager::IdeIntegrationManager;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Load configuration
//!     let config = ConfigManager::load_from_file("config/ide-integration.yaml").await?;
//!
//!     // Create IDE integration manager
//!     let manager = IdeIntegrationManager::new(config).await?;
//!
//!     // Handle IDE requests
//!     // ...
//!
//!     Ok(())
//! }
//! ```

pub mod builtin_provider;
pub mod config;
pub mod config_hot_reload;
pub mod configured_rules_provider;
pub mod error;
pub mod external_lsp_provider;
pub mod generic_provider;
pub mod hot_reload;
pub mod lsp_monitor;
pub mod manager;
pub mod provider;
pub mod provider_chain;
pub mod types;

pub use builtin_provider::{PythonProvider, RustProvider, TypeScriptProvider};
pub use config::ConfigManager;
pub use config_hot_reload::ConfigHotReloadCoordinator;
pub use configured_rules_provider::ConfiguredRulesProvider;
pub use error::IdeError;
pub use external_lsp_provider::ExternalLspProvider;
pub use generic_provider::GenericProvider;
pub use hot_reload::{HotReloadManager, ConfigChangeCallback, ProviderAvailabilityCallback};
pub use lsp_monitor::{LspMonitor, LspHealthStatus};
pub use manager::IdeIntegrationManager;
pub use provider::{IdeProvider, ProviderChain};
pub use provider_chain::{ProviderChainManager, ProviderRegistry};
pub use types::*;
