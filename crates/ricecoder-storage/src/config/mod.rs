//! Configuration loading and management
//!
//! This module provides configuration loading from multiple formats (YAML, TOML, JSON),
//! environment variable overrides, and configuration merging with precedence rules.

pub mod cli;
pub mod documents;
pub mod env;
pub mod hot_reload;
pub mod loader;
pub mod merge;
pub mod modes;
pub mod validation;

// Re-export commonly used types
pub use cli::CliArgs;
pub use documents::{Document, DocumentLoader};
pub use env::EnvOverrides;
pub use hot_reload::{ConfigWatcher, HotReloadManager};
pub use loader::ConfigLoader;
pub use merge::ConfigMerger;
pub use modes::StorageModeHandler;
pub use validation::{ConfigBackupManager, ConfigValidator, ValidationError};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    /// Provider configurations
    pub providers: ProvidersConfig,
    /// Default settings
    pub defaults: DefaultsConfig,
    /// Steering rules
    #[serde(default)]
    pub steering: Vec<SteeringRule>,
    /// TUI-specific configuration
    #[serde(default)]
    pub tui: TuiConfig,
    /// Additional custom settings
    #[serde(default)]
    pub custom: HashMap<String, serde_json::Value>,
}

/// Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProvidersConfig {
    /// API keys for various providers
    #[serde(default)]
    pub api_keys: HashMap<String, String>,
    /// Endpoints for various providers
    #[serde(default)]
    pub endpoints: HashMap<String, String>,
    /// Default provider to use
    pub default_provider: Option<String>,
}

/// Default settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DefaultsConfig {
    /// Default model to use
    pub model: Option<String>,
    /// Default temperature for LLM
    pub temperature: Option<f32>,
    /// Default max tokens for LLM
    pub max_tokens: Option<u32>,
}

/// Steering rule
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SteeringRule {
    /// Rule name
    pub name: String,
    /// Rule content
    pub content: String,
    /// Format of the rule
    pub format: crate::types::DocumentFormat,
}

/// TUI-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TuiConfig {
    /// Theme name
    pub theme: String,
    /// Enable animations
    pub animations: bool,
    /// Enable mouse support
    pub mouse: bool,
    /// Terminal width
    pub width: Option<u16>,
    /// Terminal height
    pub height: Option<u16>,
    /// Accessibility configuration
    #[serde(default)]
    pub accessibility: TuiAccessibilityConfig,
    /// Enable vim keybindings
    #[serde(default)]
    pub vim_mode: bool,
}

/// TUI accessibility configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TuiAccessibilityConfig {
    /// Enable screen reader support
    pub screen_reader_enabled: bool,
    /// Enable high contrast mode
    pub high_contrast_mode: bool,
    /// Disable animations for accessibility
    pub disable_animations: bool,
    /// Focus indicator intensity
    pub focus_indicator_intensity: u8,
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            animations: true,
            mouse: true,
            width: None,
            height: None,
            accessibility: TuiAccessibilityConfig::default(),
            vim_mode: false,
        }
    }
}

impl Default for TuiAccessibilityConfig {
    fn default() -> Self {
        Self {
            screen_reader_enabled: false,
            high_contrast_mode: false,
            disable_animations: false,
            focus_indicator_intensity: 3,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            providers: ProvidersConfig {
                api_keys: HashMap::new(),
                endpoints: HashMap::new(),
                default_provider: None,
            },
            defaults: DefaultsConfig {
                model: None,
                temperature: None,
                max_tokens: None,
            },
            steering: Vec::new(),
            tui: TuiConfig::default(),
            custom: HashMap::new(),
        }
    }
}
