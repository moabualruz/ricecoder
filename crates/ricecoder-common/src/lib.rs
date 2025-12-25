//! ricecoder-common - Shared utilities for ricecoder crates
//!
//! This crate provides common utilities, traits, and macros used across
//! multiple ricecoder crates to eliminate DRY violations.
//!
//! # Modules
//!
//! - `validation` - Common validation traits and validators
//! - `error` - Error conversion macros and utilities
//! - `collection` - Thread-safe collection access patterns
//! - `cache` - Common cache operation traits
//! - `json_store` - JSON persistence utilities

pub mod cache;
pub mod collection;
pub mod di;
pub mod error;
pub mod json_store;
pub mod logging;
pub mod validation;

// Re-export commonly used items at crate root
pub use cache::CacheOperations;
pub use collection::CollectionAccess;
// impl_error_from! is exported at crate root via #[macro_export]
pub use logging::{LogLevel, LogOptions, Logger, create as create_logger, format_error, init as init_logging};
pub use validation::{Validatable, ValidationError, Validator};
