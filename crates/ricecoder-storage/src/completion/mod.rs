//! Code completion configuration and language support module
//!
//! This module provides built-in language configurations for the code completion engine,
//! including completion keywords, snippets, and ranking rules.

pub mod languages;

pub use languages::{get_builtin_completion_configs, get_completion_config};

/// Get all built-in completion language configurations
pub fn builtin_configs() -> Vec<(&'static str, &'static str)> {
    vec![
        ("rust", include_str!("languages/rust.yaml")),
        ("typescript", include_str!("languages/typescript.yaml")),
        ("python", include_str!("languages/python.yaml")),
    ]
}
