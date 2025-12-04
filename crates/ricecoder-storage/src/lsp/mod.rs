//! LSP configuration and language support module
//!
//! This module provides built-in language configurations for the LSP server,
//! including parser plugins, diagnostic rules, and code action transformations.

pub mod languages;

pub use languages::{get_builtin_language_configs, get_language_config};

/// Get all built-in language configurations
pub fn builtin_configs() -> Vec<(&'static str, &'static str)> {
    vec![
        ("rust", include_str!("languages/rust.yaml")),
        ("typescript", include_str!("languages/typescript.yaml")),
        ("python", include_str!("languages/python.yaml")),
    ]
}
