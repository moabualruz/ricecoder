//! RiceCoder Configuration Management System
//!
//! This crate provides comprehensive configuration management functionality for RiceCoder,
//! including loading from multiple sources, validation, hot reloading, and runtime updates.

pub mod error;
pub mod manager;
pub mod tui_config;
pub mod types;

pub use error::{ConfigError, Result};
pub use manager::ConfigManager;
pub use tui_config::TuiConfig;
pub use types::{AppConfig, ConfigManager as ConfigManagerTrait};
