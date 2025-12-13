//! Core configuration types and data structures

use serde::{Deserialize, Serialize};

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AppConfig {
    /// Editor configuration
    pub editor: EditorConfig,
    /// UI configuration
    pub ui: UiConfig,
    /// Keybinding configuration
    pub keybinds: KeybindConfig,
    /// Theme configuration
    pub theme: ThemeConfig,
}

/// Editor-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EditorConfig {
    /// Tab size for indentation
    pub tab_size: usize,
    /// Whether to insert spaces instead of tabs
    pub insert_spaces: bool,
    /// Whether to wrap long lines
    pub word_wrap: bool,
    /// Whether to show line numbers
    pub line_numbers: bool,
    /// Syntax highlighting enabled
    pub syntax_highlight: bool,
}

/// UI-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UiConfig {
    /// Current theme name
    pub theme: String,
    /// Font size
    pub font_size: u8,
    /// Show status bar
    pub show_status_bar: bool,
    /// Show command palette
    pub show_command_palette: bool,
}

/// Keybinding configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct KeybindConfig {
    /// Custom keybindings
    pub custom: std::collections::HashMap<String, String>,
}

/// Theme configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemeConfig {
    /// Current theme name
    pub current: String,
    /// Theme overrides
    pub overrides: std::collections::HashMap<String, serde_json::Value>,
}



impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            tab_size: 4,
            insert_spaces: true,
            word_wrap: false,
            line_numbers: true,
            syntax_highlight: true,
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            font_size: 12,
            show_status_bar: true,
            show_command_palette: true,
        }
    }
}



impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            current: "dark".to_string(),
            overrides: std::collections::HashMap::new(),
        }
    }
}

/// Configuration manager trait
pub trait ConfigManager {
    /// Load configuration
    fn load_config(&mut self) -> Result<AppConfig, crate::error::ConfigError>;
    /// Save configuration
    fn save_config(&self, config: &AppConfig) -> Result<(), crate::error::ConfigError>;
    /// Validate configuration
    fn validate_config(&self, config: &AppConfig) -> Result<(), crate::error::ConfigError>;
}