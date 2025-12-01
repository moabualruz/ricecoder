//! RiceCoder Storage and Configuration Module
//!
//! This module provides storage and configuration management for RiceCoder,
//! including global and project-local knowledge bases, configuration loading,
//! and data persistence.

pub mod error;
pub mod first_run;
pub mod global_store;
pub mod manager;
pub mod project_store;
pub mod types;

// Re-export commonly used types
pub use error::{IoOperation, StorageError, StorageResult};
pub use first_run::FirstRunHandler;
pub use global_store::GlobalStore;
pub use manager::{PathResolver, StorageManager};
pub use project_store::ProjectStore;
pub use types::{ConfigFormat, DocumentFormat, ResourceType, StorageConfig, StorageMode, StorageState};
