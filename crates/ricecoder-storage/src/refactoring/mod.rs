//! Refactoring configuration and language support module
//!
//! This module provides built-in language configurations for the refactoring engine,
//! including refactoring rules, transformations, and language-specific patterns.

pub mod languages;

pub use languages::{get_builtin_refactoring_configs, get_refactoring_config};

/// Get all built-in refactoring configurations
pub fn builtin_configs() -> Vec<(&'static str, &'static str)> {
    vec![
        ("rust", include_str!("languages/rust.yaml")),
        ("typescript", include_str!("languages/typescript.yaml")),
        ("python", include_str!("languages/python.yaml")),
    ]
}
