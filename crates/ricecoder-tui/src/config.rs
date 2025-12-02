//! Configuration for the TUI
//!
//! This module provides configuration management for the RiceCoder TUI. Configuration can be
//! loaded from files or environment variables, and includes settings for:
//! - Theme selection
//! - Animation preferences
//! - Mouse support
//! - Terminal dimensions
//! - Accessibility options
//!
//! # Configuration Hierarchy
//!
//! Configuration is loaded in the following priority order (highest to lowest):
//! 1. Runtime overrides (CLI flags, environment variables)
//! 2. Project-level config (`.ricecoder/config.yaml`)
//! 3. User-level config (`~/.ricecoder/config.yaml`)
//! 4. Built-in defaults
//!
//! # Examples
//!
//! Loading default configuration:
//!
//! ```ignore
//! use ricecoder_tui::TuiConfig;
//!
//! let config = TuiConfig::load()?;
//! println!("Theme: {}", config.theme);
//! println!("Animations: {}", config.animations);
//! ```
//!
//! Creating custom configuration:
//!
//! ```ignore
//! use ricecoder_tui::TuiConfig;
//!
//! let config = TuiConfig {
//!     theme: "dracula".to_string(),
//!     animations: false,
//!     mouse: true,
//!     ..Default::default()
//! };
//! config.save()?;
//! ```
//!
//! # Configuration File Format
//!
//! Configuration files use YAML format:
//!
//! ```yaml
//! theme: dracula
//! animations: true
//! mouse: true
//! accessibility:
//!   screen_reader_enabled: false
//!   high_contrast_mode: false
//!   disable_animations: false
//! ```

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::accessibility::AccessibilityConfig;

/// TUI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub accessibility: AccessibilityConfig,
    /// AI provider to use
    #[serde(default)]
    pub provider: Option<String>,
    /// Model to use
    #[serde(default)]
    pub model: Option<String>,
    /// Enable vim keybindings
    #[serde(default)]
    pub vim_mode: bool,
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            animations: true,
            mouse: true,
            width: None,
            height: None,
            accessibility: AccessibilityConfig::default(),
            provider: None,
            model: None,
            vim_mode: false,
        }
    }
}

impl TuiConfig {
    /// Load configuration from file
    pub fn load() -> Result<Self> {
        // TODO: Load from config file if it exists
        Ok(Self::default())
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        // TODO: Save to config file
        Ok(())
    }

    /// Get the config file path
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
        Ok(config_dir.join("ricecoder").join("tui.yaml"))
    }
}
