//! Local Models Management for RiceCoder
//!
//! This crate provides model management functionality for local models via Ollama.
//! It handles model pulling, removal, updates, and version management.

pub mod di;
pub mod error;
pub mod manager;
pub mod models;

pub use error::LocalModelError;
pub use manager::LocalModelManager;
pub use models::{LocalModel, ModelMetadata, PullProgress};

/// Result type for local model operations
pub type Result<T> = std::result::Result<T, LocalModelError>;
