//! RiceCoder Theme Management System
//!
//! This crate provides comprehensive theme management functionality for RiceCoder,
//! including theme loading, validation, hot reloading, and runtime theme switching.

pub mod error;
pub mod loader;
pub mod manager;
pub mod registry;
pub mod types;

pub use error::{ThemeError, Result};
pub use loader::ThemeLoader;
pub use manager::ThemeManager;
pub use registry::ThemeRegistry;
pub use types::{Theme, ThemeConfig, ThemeMetadata};