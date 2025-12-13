//! RiceCoder Storage and Configuration Infrastructure
//!
//! This crate provides the foundational storage and configuration management infrastructure
//! for RiceCoder. It serves as the central point for configuration loading, data persistence,
//! and cross-cutting concerns like caching and preferences.
//!
//! ## Features
//!
//! - **Configuration Management**: YAML/JSONC configuration with environment variable overrides
//! - **Hierarchical Settings**: Project > user > defaults configuration loading
//! - **Hot Reload**: Configuration changes applied without restart
//! - **Type-Safe Config**: Strongly typed configuration structures with validation
//! - **Caching**: High-performance caching with TTL and invalidation strategies
//! - **Data Persistence**: Structured storage and retrieval of application data
//! - **File Watching**: Automatic detection of configuration file changes
//! - **User Preferences**: Persistent user preference management
//!
//! ## Architecture
//!
//! `ricecoder-storage` serves as the infrastructure layer that all other RiceCoder crates
//! can depend on for configuration and data persistence needs. It has no business logic
//! dependencies and focuses solely on storage and configuration concerns.
//!
//! # Modules
//!
//! ## Markdown Configuration
//!
//! The [`markdown_config`] module enables users to define custom agents, modes, and commands
//! using markdown files with YAML frontmatter. This provides a user-friendly way to extend
//! RiceCoder without writing code.
//!
//! **Key Components**:
//! - [`markdown_config::ConfigurationLoader`]: Discovers and loads configuration files
//! - [`markdown_config::ConfigRegistry`]: Central registry for loaded configurations
//! - [`markdown_config::FileWatcher`]: Monitors configuration files for hot-reload
//! - [`markdown_config::MarkdownParser`]: Parses markdown with YAML frontmatter
//! - [`markdown_config::YamlParser`]: Parses and validates YAML metadata
//!
//! **Configuration File Locations**:
//! 1. Project-level: `projects/ricecoder/.agent/`
//! 2. User-level: `~/.ricecoder/agents/`, `~/.ricecoder/modes/`, `~/.ricecoder/commands/`
//! 3. System-level: `/etc/ricecoder/agents/` (Linux/macOS)
//!
//! **File Patterns**:
//! - `*.agent.md` - Agent configurations
//! - `*.mode.md` - Mode configurations
//! - `*.command.md` - Command configurations
//!
//! **Example Usage**:
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
//!     // Load configurations from standard locations
//!     let paths = vec![
//!         std::path::PathBuf::from("~/.ricecoder/agents"),
//!         std::path::PathBuf::from("projects/ricecoder/.agent"),
//!     ];
//!
//!     loader.load_all(&paths).await?;
//!
//!     // Query loaded configurations
//!     if let Some(agent) = registry.get_agent("code-review") {
//!         println!("Agent: {}", agent.name);
//!         println!("Model: {:?}", agent.model);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! See [`markdown_config`] module documentation for detailed information.
//!
//! ## Other Modules
//!
//! - [`cache`]: Caching infrastructure for configuration and analysis results
//! - [`config`]: Configuration loading and merging
//! - [`completion`]: Code completion configurations
//! - [`lsp`]: Language Server Protocol configurations
//! - [`refactoring`]: Code refactoring configurations
//! - [`industry`]: Industry-specific file detection and handling
//! - [`global_store`]: Global knowledge base storage
//! - [`project_store`]: Project-local knowledge base storage
//! - [`manager`]: Storage manager and path resolution
//! - [`offline`]: Offline mode handling
//! - [`first_run`]: First-run initialization

pub mod cache;
pub mod cache_implementations;
pub mod completion;
pub mod config;
pub mod config_cache;
pub mod error;
pub mod first_run;
pub mod global_store;
pub mod industry;
pub mod lsp;
pub mod manager;
pub mod markdown_config;
pub mod offline;
pub mod preferences;
pub mod project_store;
pub mod refactoring;
pub mod relocation;
pub mod theme;
pub mod types;

// Re-export commonly used types
pub use cache::{CacheEntry, CacheInvalidationStrategy, CacheManager};
pub use cache_implementations::{
    CacheStats, ConfigCache as ConfigCacheImpl, ProviderCache, ProjectAnalysisCache, SpecCache,
};
pub use completion::{get_builtin_completion_configs, get_completion_config};
pub use config::{
    CliArgs, Config, ConfigLoader, ConfigMerger, DefaultsConfig, DocumentLoader, EnvOverrides,
    ProvidersConfig, StorageModeHandler, TuiConfig, TuiAccessibilityConfig,
};
pub use config_cache::ConfigCache;
pub use error::{IoOperation, StorageError, StorageResult};
pub use config::hot_reload::{ConfigConflictResolver, HotReloadManager};
pub use first_run::FirstRunHandler;
pub use global_store::GlobalStore;
pub use industry::{FileDetectionResult, IndustryFileAdapter, IndustryFileDetector};
pub use lsp::{get_builtin_language_configs, get_language_config};
pub use manager::{PathResolver, StorageManager};
pub use markdown_config::{
    AgentConfig, CommandConfig, MarkdownConfigError, MarkdownParser, ModeConfig, Parameter,
    ParsedMarkdown, YamlParser,
};
pub use offline::OfflineModeHandler;
pub use preferences::{PreferencesManager, UserPreferences};
pub use project_store::ProjectStore;
pub use refactoring::{get_builtin_refactoring_configs, get_refactoring_config};
pub use relocation::RelocationService;
pub use theme::{ThemePreference, ThemeStorage};
pub use types::{
    ConfigFormat, DocumentFormat, ResourceType, StorageConfig, StorageMode, StorageState,
};
