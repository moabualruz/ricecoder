//! Configuration module for language-agnostic, configuration-driven architecture
//!
//! This module provides configuration loading, validation, and management for
//! language-specific analyzers, diagnostics rules, and code actions.

pub mod loader;
pub mod manager;
pub mod types;

// Re-export commonly used types
pub use loader::ConfigLoader;
pub use manager::ConfigurationManager;
pub use types::{
    CodeActionTemplate, CompletionConfig, ConfigError, ConfigRegistry, ConfigResult,
    DiagnosticRule, LanguageConfig,
};
