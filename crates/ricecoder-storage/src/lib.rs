//! RiceCoder Pure Storage Infrastructure
//!
//! This crate provides pure persistence and storage infrastructure for RiceCoder.
//! It focuses solely on data storage, retrieval, and caching without any business logic.
//!
//! ## Features
//!
//! - **Configuration Management**: YAML/JSONC configuration with environment variable overrides
//! - **Hierarchical Settings**: Project > user > defaults configuration loading
//! - **Session Management**: Session persistence and state management
//! - **Caching**: High-performance caching with TTL and invalidation strategies
//! - **Data Persistence**: Structured storage and retrieval of application data
//! - **Global Store**: Global knowledge base storage
//! - **Project Store**: Project-local knowledge base storage
//!
//! ## Architecture
//!
//! `ricecoder-storage` serves as the infrastructure layer that all other RiceCoder crates
//! can depend on for pure storage needs. It has no business logic dependencies and focuses
//! solely on storage and persistence concerns.
//!
//! ## Modules
//!
//! - [`cache`]: Caching infrastructure for configuration and analysis results
//! - [`config`]: Configuration loading and merging
//! - [`session`]: Session persistence and state management
//! - [`global_store`]: Global knowledge base storage
//! - [`project_store`]: Project-local knowledge base storage
//! - [`manager`]: Storage manager and path resolution
//! - [`types`]: Core storage types and enums

pub mod cache;
pub mod cache_implementations;
pub mod config;
pub mod config_cache;
pub mod defaults;
pub mod di;
pub mod error;
pub mod global_store;
pub mod loaders;
pub mod manager;
pub mod project_store;
pub mod session;
pub mod theme;
pub mod types;

// Re-export commonly used types
pub use cache::{CacheEntry, CacheInvalidationStrategy, CacheManager};
pub use cache_implementations::{
    CacheStats, ConfigCache as ConfigCacheImpl, ProjectAnalysisCache, ProviderCache, SpecCache,
};
pub use config::{
    hot_reload::{ConfigConflictResolver, HotReloadManager},
    CliArgs, Config, ConfigLoader, ConfigMerger, DefaultsConfig, DocumentLoader, EnvOverrides,
    ProvidersConfig, StorageModeHandler, TuiAccessibilityConfig, TuiConfig,
};
pub use config_cache::ConfigCache;
pub use defaults::{DefaultsManager, EmbeddedDefault};
pub use error::{IoOperation, StorageError, StorageResult};
pub use global_store::GlobalStore;
pub use manager::{PathResolver, StorageManager};
pub use project_store::ProjectStore;
pub use session::{SessionData, SessionManager, SessionState};
pub use theme::{ThemePreference, ThemeStorage};
pub use types::{
    ConfigFormat, ConfigSubdirectory, DocumentFormat, ResourceType, RuntimeStorageType,
    StorageConfig, StorageDirectory, StorageMode, StorageState,
};

// Re-export loaders
pub use loaders::{
    global_lsp_configs, Agent, AgentLoader, AuthLoader, Command, CommandLoader, LspConfig,
    LspConfigLoader, PromptCategory, PromptLoader, ProviderAuth, ProvidersAuth, Theme,
    ThemeLoader, TipsLoader,
};
