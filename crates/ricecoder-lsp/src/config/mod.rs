//! Configuration module for language-agnostic, configuration-driven architecture
//!
//! This module provides configuration loading, validation, and management for
//! language-specific analyzers, diagnostics rules, and code actions.

pub mod types;
pub mod loader;
pub mod manager;

// Re-export commonly used types
pub use types::{
    ConfigError, ConfigResult, LanguageConfig, DiagnosticRule, CodeActionTemplate,
    ConfigRegistry,
};
pub use loader::ConfigLoader;
pub use manager::ConfigurationManager;
