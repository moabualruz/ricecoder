//! RiceCoder Storage and Configuration Module
//!
//! This module provides storage and configuration management for RiceCoder,
//! including global and project-local knowledge bases, configuration loading,
//! and data persistence.

pub mod cache;
pub mod completion;
pub mod config;
pub mod config_cache;
pub mod error;
pub mod first_run;
pub mod global_store;
pub mod industry;
pub mod lsp;
pub mod manager;
pub mod offline;
pub mod project_store;
pub mod relocation;
pub mod types;

// Re-export commonly used types
pub use cache::{CacheEntry, CacheInvalidationStrategy, CacheManager};
pub use completion::{get_builtin_completion_configs, get_completion_config};
pub use config::{
    Config, ConfigLoader, ConfigMerger, DocumentLoader, EnvOverrides, StorageModeHandler,
};
pub use config_cache::ConfigCache;
pub use error::{IoOperation, StorageError, StorageResult};
pub use first_run::FirstRunHandler;
pub use global_store::GlobalStore;
pub use industry::{FileDetectionResult, IndustryFileAdapter, IndustryFileDetector};
pub use lsp::{get_builtin_language_configs, get_language_config};
pub use manager::{PathResolver, StorageManager};
pub use offline::OfflineModeHandler;
pub use project_store::ProjectStore;
pub use relocation::RelocationService;
pub use types::{
    ConfigFormat, DocumentFormat, ResourceType, StorageConfig, StorageMode, StorageState,
};
