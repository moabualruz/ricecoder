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
//! 2. Project-level config (`.ricecoder/config.yaml`, `.ricecoder/config.json`, `.ricecoder/config.toml`)
//! 3. User-level config (`~/.ricecoder/tui.yaml`, `~/.ricecoder/tui.json`, `~/.ricecoder/tui.toml`)
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

use std::{fs, path::PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json;
use toml;

/// Focus indicator style
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum FocusIndicatorStyle {
    /// Underline
    Underline,
    /// Border
    #[default]
    Border,
    /// Background color
    Background,
    /// None
    None,
}

/// Animation configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnimationConfig {
    /// Enable fade transitions
    pub fade_enabled: bool,
    /// Transition duration in milliseconds
    pub transition_duration: u32,
    /// Enable slide animations
    pub slide_enabled: bool,
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            fade_enabled: true,
            transition_duration: 200,
            slide_enabled: true,
        }
    }
}

/// Accessibility configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AccessibilityConfig {
    /// Enable screen reader support
    pub screen_reader_enabled: bool,
    /// Enable high contrast mode
    pub high_contrast_enabled: bool,
    /// Disable animations
    pub animations_disabled: bool,
    /// Enable state announcements
    pub announcements_enabled: bool,
    /// Focus indicator style
    pub focus_indicator: FocusIndicatorStyle,
    /// Animation configuration
    #[serde(default)]
    pub animations: AnimationConfig,
    /// Font size multiplier (1.0 = normal, 1.5 = 150%, etc.)
    pub font_size_multiplier: f32,
    /// Enable large click targets
    pub large_click_targets: bool,
    /// Enable auto-advance for forms
    pub auto_advance: bool,
}

impl Default for AccessibilityConfig {
    fn default() -> Self {
        Self {
            screen_reader_enabled: false,
            high_contrast_enabled: false,
            animations_disabled: false,
            announcements_enabled: true,
            focus_indicator: FocusIndicatorStyle::default(),
            animations: AnimationConfig::default(),
            font_size_multiplier: 1.0,
            large_click_targets: false,
            auto_advance: false,
        }
    }
}

impl AccessibilityConfig {
    /// Merge another accessibility config into this one
    pub fn merge(mut self, other: AccessibilityConfig) -> Self {
        // For now, just take the other config. In a real implementation,
        // you'd merge non-default values.
        self.screen_reader_enabled = other.screen_reader_enabled;
        self.high_contrast_enabled = other.high_contrast_enabled;
        self.animations_disabled = other.animations_disabled;
        self.announcements_enabled = other.announcements_enabled;
        self.focus_indicator = other.focus_indicator;
        self.animations = other.animations;
        self.font_size_multiplier = other.font_size_multiplier;
        self.large_click_targets = other.large_click_targets;
        self.auto_advance = other.auto_advance;
        self
    }
}
use std::sync::Arc;

use tokio::sync::RwLock;

/// TUI configuration
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
    /// Load configuration from file with full hierarchy
    pub fn load() -> Result<Self> {
        Self::load_with_hierarchy()
    }

    /// Load configuration with full hierarchy and merging
    pub fn load_with_hierarchy() -> Result<Self> {
        Self::load_with_hierarchy_and_overrides(None)
    }

    /// Load configuration with full hierarchy and runtime overrides
    pub fn load_with_hierarchy_and_overrides(
        runtime_overrides: Option<&TuiConfig>,
    ) -> Result<Self> {
        // Start with defaults
        let mut config = Self::default();

        // Load user-level config (lowest priority except defaults)
        if let Ok(user_config) = Self::load_user_config() {
            config = config.merge(user_config);
        }

        // Load project-level config (higher priority)
        if let Ok(project_config) = Self::load_project_config() {
            config = config.merge(project_config);
        }

        // Apply runtime overrides (highest priority)
        if let Some(overrides) = runtime_overrides {
            config = config.merge(overrides.clone());
        }

        // Load environment variable overrides
        if let Ok(env_config) = Self::load_from_env() {
            config = config.merge(env_config);
        }

        config.validate()?;
        Ok(config)
    }

    /// Load user-level configuration
    pub fn load_user_config() -> Result<Self> {
        // Try user config files in priority order
        let yaml_config = Self::load_yaml();
        if yaml_config.is_ok() {
            return yaml_config;
        }

        let json_config = Self::load_json();
        if json_config.is_ok() {
            return json_config;
        }

        Self::load_toml()
    }

    /// Load project-level configuration
    pub fn load_project_config() -> Result<Self> {
        // Look for project config in current directory and parent directories
        let mut current_dir = std::env::current_dir()?;

        loop {
            let config_paths = [
                current_dir.join(".ricecoder").join("config.yaml"),
                current_dir.join(".ricecoder").join("config.json"),
                current_dir.join(".ricecoder").join("config.toml"),
                current_dir.join(".ricecoder").join("tui.yaml"),
                current_dir.join(".ricecoder").join("tui.json"),
                current_dir.join(".ricecoder").join("tui.toml"),
            ];

            for path in &config_paths {
                if path.exists() {
                    let content = fs::read_to_string(path).map_err(|e| {
                        anyhow::anyhow!("Failed to read project config {}: {}", path.display(), e)
                    })?;

                    let config: TuiConfig = match path.extension().and_then(|s| s.to_str()) {
                        Some("yaml") | Some("yml") => {
                            serde_yaml::from_str(&content).map_err(|e| {
                                anyhow::anyhow!(
                                    "Failed to parse YAML config {}: {}",
                                    path.display(),
                                    e
                                )
                            })?
                        }
                        Some("json") => serde_json::from_str(&content).map_err(|e| {
                            anyhow::anyhow!("Failed to parse JSON config {}: {}", path.display(), e)
                        })?,
                        Some("toml") => toml::from_str(&content).map_err(|e| {
                            anyhow::anyhow!("Failed to parse TOML config {}: {}", path.display(), e)
                        })?,
                        _ => continue,
                    };

                    return Ok(config);
                }
            }

            // Move up one directory
            if !current_dir.pop() {
                break;
            }
        }

        Err(anyhow::anyhow!("No project configuration found"))
    }

    /// Load configuration from environment variables
    pub fn load_from_env() -> Result<Self> {
        let mut config = Self::default();

        // Check for environment variable overrides
        if let Ok(theme) = std::env::var("RICECODER_THEME") {
            config.theme = theme;
        }

        if let Ok(animations) = std::env::var("RICECODER_ANIMATIONS") {
            config.animations = animations.parse().unwrap_or(true);
        }

        if let Ok(mouse) = std::env::var("RICECODER_MOUSE") {
            config.mouse = mouse.parse().unwrap_or(true);
        }

        if let Ok(vim_mode) = std::env::var("RICECODER_VIM_MODE") {
            config.vim_mode = vim_mode.parse().unwrap_or(false);
        }

        Ok(config)
    }

    /// Merge two configurations (self takes precedence over other)
    pub fn merge(mut self, other: Self) -> Self {
        // Only override if values are not default
        if !other.theme.is_empty() && other.theme != Self::default().theme {
            self.theme = other.theme;
        }

        // For boolean values, any explicit setting overrides
        if other.animations != Self::default().animations {
            self.animations = other.animations;
        }

        if other.mouse != Self::default().mouse {
            self.mouse = other.mouse;
        }

        // Merge optional values
        if other.width.is_some() {
            self.width = other.width;
        }

        if other.height.is_some() {
            self.height = other.height;
        }

        // Merge accessibility config
        self.accessibility = self.accessibility.merge(other.accessibility);

        // Merge provider settings
        if other.provider.is_some() {
            self.provider = other.provider;
        }

        if other.model.is_some() {
            self.model = other.model;
        }

        if other.vim_mode != Self::default().vim_mode {
            self.vim_mode = other.vim_mode;
        }

        self
    }

    /// Validate configuration values
    pub fn validate(&self) -> Result<()> {
        // Validate theme name (should be non-empty)
        if self.theme.trim().is_empty() {
            return Err(anyhow::anyhow!("Theme name cannot be empty"));
        }

        // Validate terminal dimensions if specified
        if let Some(width) = self.width {
            if width == 0 {
                return Err(anyhow::anyhow!("Terminal width must be greater than 0"));
            }
        }

        if let Some(height) = self.height {
            if height == 0 {
                return Err(anyhow::anyhow!("Terminal height must be greater than 0"));
            }
        }

        // Validate accessibility config
        if self.accessibility.font_size_multiplier < 1.0
            || self.accessibility.font_size_multiplier > 2.0
        {
            return Err(anyhow::anyhow!(
                "Font size multiplier must be between 1.0 and 2.0"
            ));
        }

        Ok(())
    }
}

/// Predefined configuration presets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigPreset {
    /// Developer-friendly settings
    Developer,
    /// Accessibility optimized
    Accessibility,
    /// Minimal, clean interface
    Minimal,
    /// Presentation optimized
    Presentation,
}

impl std::fmt::Display for ConfigPreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigPreset::Developer => write!(f, "Developer"),
            ConfigPreset::Accessibility => write!(f, "Accessibility"),
            ConfigPreset::Minimal => write!(f, "Minimal"),
            ConfigPreset::Presentation => write!(f, "Presentation"),
        }
    }
}

/// Runtime configuration changes that can be applied without restart
#[derive(Debug, Clone, Default)]
pub struct RuntimeConfigChanges {
    /// New theme name to switch to
    pub theme_name: Option<String>,
    /// Enable/disable vim mode
    pub vim_mode: Option<bool>,
    /// Enable/disable screen reader support
    pub screen_reader_enabled: Option<bool>,
    /// Enable/disable high contrast mode
    pub high_contrast_enabled: Option<bool>,
    /// Font size multiplier for accessibility
    pub font_size_multiplier: Option<f32>,
}

impl RuntimeConfigChanges {
    /// Create a new runtime config changes instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Set theme change
    pub fn with_theme(mut self, theme_name: impl Into<String>) -> Self {
        self.theme_name = Some(theme_name.into());
        self
    }

    /// Set vim mode change
    pub fn with_vim_mode(mut self, enabled: bool) -> Self {
        self.vim_mode = Some(enabled);
        self
    }

    /// Set screen reader change
    pub fn with_screen_reader(mut self, enabled: bool) -> Self {
        self.screen_reader_enabled = Some(enabled);
        self
    }

    /// Set high contrast change
    pub fn with_high_contrast(mut self, enabled: bool) -> Self {
        self.high_contrast_enabled = Some(enabled);
        self
    }

    /// Set font size multiplier
    pub fn with_font_size(mut self, multiplier: f32) -> Self {
        self.font_size_multiplier = Some(multiplier);
        self
    }
}

/// Configuration manager with hot-reload support
///
/// # Hot-Reload Status
///
/// File watching for automatic hot-reload is not yet implemented.
/// Use `reload()` for manual configuration refresh.
///
/// # Future Enhancement
///
/// When file watching is implemented, it will use the `notify` crate
/// to watch config files and automatically trigger `reload()`.
pub struct ConfigManager {
    /// Current configuration
    config: Arc<RwLock<TuiConfig>>,
    /// Callback for configuration changes
    change_callback: Option<Box<dyn Fn(TuiConfig) + Send + Sync>>,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(TuiConfig::default())),
            change_callback: None,
        }
    }

    /// Load configuration from hierarchy and prepare for watching
    ///
    /// Loads configuration from all sources (defaults, user, project, env).
    /// File watching is not yet implemented - use `reload()` for manual refresh.
    pub async fn load_and_watch(&mut self) -> Result<()> {
        // Load initial configuration
        let config = TuiConfig::load_with_hierarchy()?;
        *self.config.write().await = config.clone();

        // Note: File watching not yet implemented
        // When implemented, will call self.start_watching().await?;

        Ok(())
    }

    /// Get current configuration
    pub async fn get_config(&self) -> TuiConfig {
        self.config.read().await.clone()
    }

    /// Set configuration change callback
    pub fn set_change_callback<F>(&mut self, callback: F)
    where
        F: Fn(TuiConfig) + Send + Sync + 'static,
    {
        self.change_callback = Some(Box::new(callback));
    }

    /// Manually reload configuration
    pub async fn reload(&mut self) -> Result<()> {
        let new_config = TuiConfig::load_with_hierarchy()?;

        // Check if configuration actually changed
        let current_config = self.config.read().await;
        if *current_config != new_config {
            drop(current_config); // Release read lock
            *self.config.write().await = new_config.clone();

            // Call change callback if set
            if let Some(callback) = &self.change_callback {
                callback(new_config);
            }

            tracing::info!("Configuration reloaded");
        }

        Ok(())
    }

    /// Start watching configuration files for changes
    ///
    /// # Status: Not Implemented
    ///
    /// This is a placeholder for future file watching functionality.
    /// When implemented, will use the `notify` crate to watch:
    /// - `~/.config/ricecoder/config.toml` (user config)
    /// - `./.ricecoder/config.toml` (project config)
    #[allow(dead_code)]
    async fn start_watching(&mut self) -> Result<()> {
        // Placeholder - file watching not yet implemented
        tracing::debug!("File watching not yet implemented");
        Ok(())
    }

    /// Stop watching configuration files
    ///
    /// # Status: Not Implemented
    ///
    /// Placeholder for stopping file watcher when implemented.
    pub async fn stop_watching(&mut self) -> Result<()> {
        // Placeholder - file watching not yet implemented
        tracing::debug!("File watching not yet implemented");
        Ok(())
    }

    /// Apply runtime configuration changes
    pub async fn apply_runtime_changes(&mut self, changes: RuntimeConfigChanges) -> Result<()> {
        let mut config = self.config.read().await.clone();

        // Apply theme changes
        if let Some(theme_name) = changes.theme_name {
            config.theme = theme_name;
        }

        // Apply vim mode changes
        if let Some(vim_mode) = changes.vim_mode {
            config.vim_mode = vim_mode;
        }

        // Apply accessibility changes
        if let Some(screen_reader) = changes.screen_reader_enabled {
            config.accessibility.screen_reader_enabled = screen_reader;
        }

        if let Some(high_contrast) = changes.high_contrast_enabled {
            config.accessibility.high_contrast_enabled = high_contrast;
        }

        if let Some(font_size) = changes.font_size_multiplier {
            config.accessibility.font_size_multiplier = font_size;
        }

        // Validate the updated config
        config.validate()?;

        // Update the config
        *self.config.write().await = config.clone();

        // Call change callback if set
        if let Some(callback) = &self.change_callback {
            callback(config);
        }

        Ok(())
    }

    /// Get current theme name
    pub async fn current_theme(&self) -> String {
        self.config.read().await.theme.clone()
    }

    /// Get current vim mode setting
    pub async fn vim_mode_enabled(&self) -> bool {
        self.config.read().await.vim_mode
    }

    /// Get current accessibility settings
    pub async fn accessibility_settings(&self) -> AccessibilityConfig {
        self.config.read().await.accessibility.clone()
    }

    /// Apply a configuration preset
    pub async fn apply_preset(&mut self, preset: ConfigPreset) -> Result<()> {
        let changes = match preset {
            ConfigPreset::Developer => RuntimeConfigChanges::new()
                .with_theme("dracula")
                .with_vim_mode(true)
                .with_screen_reader(false)
                .with_high_contrast(false)
                .with_font_size(1.0),
            ConfigPreset::Accessibility => RuntimeConfigChanges::new()
                .with_theme("high-contrast-light")
                .with_vim_mode(false)
                .with_screen_reader(true)
                .with_high_contrast(true)
                .with_font_size(1.5),
            ConfigPreset::Minimal => RuntimeConfigChanges::new()
                .with_theme("dark")
                .with_vim_mode(false)
                .with_screen_reader(false)
                .with_high_contrast(false)
                .with_font_size(1.0),
            ConfigPreset::Presentation => RuntimeConfigChanges::new()
                .with_theme("solarized-light")
                .with_vim_mode(false)
                .with_screen_reader(false)
                .with_high_contrast(false)
                .with_font_size(1.2),
        };

        self.apply_runtime_changes(changes).await
    }

    /// Get available configuration presets
    pub fn available_presets() -> Vec<(ConfigPreset, &'static str)> {
        vec![
            (
                ConfigPreset::Developer,
                "Developer-friendly settings with dark theme and vim mode",
            ),
            (
                ConfigPreset::Accessibility,
                "High contrast and screen reader optimized",
            ),
            (ConfigPreset::Minimal, "Clean, minimal interface"),
            (
                ConfigPreset::Presentation,
                "Light theme optimized for presentations",
            ),
        ]
    }

    /// Get current preset (best match)
    pub async fn current_preset(&self) -> Option<ConfigPreset> {
        let config = self.config.read().await;

        // Try to match against presets
        if config.theme == "dracula"
            && config.vim_mode
            && !config.accessibility.screen_reader_enabled
        {
            Some(ConfigPreset::Developer)
        } else if config.theme == "high-contrast-light"
            && config.accessibility.screen_reader_enabled
            && config.accessibility.high_contrast_enabled
        {
            Some(ConfigPreset::Accessibility)
        } else if config.theme == "dark"
            && !config.vim_mode
            && !config.accessibility.screen_reader_enabled
        {
            Some(ConfigPreset::Minimal)
        } else if config.theme == "solarized-light"
            && !config.vim_mode
            && !config.accessibility.screen_reader_enabled
        {
            Some(ConfigPreset::Presentation)
        } else {
            None
        }
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TuiConfig {
    /// Save configuration to YAML file
    pub fn save_yaml(&self) -> Result<()> {
        self.validate()?; // Validate before saving

        let config_path = Self::yaml_config_path()?;

        // Create parent directories if they don't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                anyhow::anyhow!(
                    "Failed to create config directory {}: {}",
                    parent.display(),
                    e
                )
            })?;
        }

        let yaml_content = serde_yaml::to_string(self)
            .map_err(|e| anyhow::anyhow!("Failed to serialize config to YAML: {}", e))?;

        fs::write(&config_path, yaml_content).map_err(|e| {
            anyhow::anyhow!(
                "Failed to write YAML config file {}: {}",
                config_path.display(),
                e
            )
        })?;

        Ok(())
    }

    /// Save configuration to JSON file
    pub fn save_json(&self) -> Result<()> {
        self.validate()?; // Validate before saving

        let config_path = Self::json_config_path()?;

        // Create parent directories if they don't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                anyhow::anyhow!(
                    "Failed to create config directory {}: {}",
                    parent.display(),
                    e
                )
            })?;
        }

        let json_content = serde_json::to_string_pretty(self)
            .map_err(|e| anyhow::anyhow!("Failed to serialize config to JSON: {}", e))?;

        fs::write(&config_path, json_content).map_err(|e| {
            anyhow::anyhow!(
                "Failed to write JSON config file {}: {}",
                config_path.display(),
                e
            )
        })?;

        Ok(())
    }

    /// Save configuration to TOML file
    pub fn save_toml(&self) -> Result<()> {
        self.validate()?; // Validate before saving

        let config_path = Self::toml_config_path()?;

        // Create parent directories if they don't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                anyhow::anyhow!(
                    "Failed to create config directory {}: {}",
                    parent.display(),
                    e
                )
            })?;
        }

        let toml_content = toml::to_string_pretty(self)
            .map_err(|e| anyhow::anyhow!("Failed to serialize config to TOML: {}", e))?;

        fs::write(&config_path, toml_content).map_err(|e| {
            anyhow::anyhow!(
                "Failed to write TOML config file {}: {}",
                config_path.display(),
                e
            )
        })?;

        Ok(())
    }

    /// Save configuration to file (YAML format by default)
    pub fn save(&self) -> Result<()> {
        self.save_yaml()
    }

    /// Get the YAML config file path
    pub fn yaml_config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
        Ok(config_dir.join("ricecoder").join("tui.yaml"))
    }

    /// Get the JSON config file path
    pub fn json_config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
        Ok(config_dir.join("ricecoder").join("tui.json"))
    }

    /// Get the TOML config file path
    pub fn toml_config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
        Ok(config_dir.join("ricecoder").join("tui.toml"))
    }

    /// Get the config file path (YAML > JSON > TOML precedence)
    pub fn config_path() -> Result<PathBuf> {
        let yaml_path = Self::yaml_config_path()?;
        if yaml_path.exists() {
            return Ok(yaml_path);
        }

        let json_path = Self::json_config_path()?;
        if json_path.exists() {
            return Ok(json_path);
        }

        Self::toml_config_path()
    }

    /// Load configuration from YAML file
    pub fn load_yaml() -> Result<Self> {
        let config_path = Self::yaml_config_path()?;

        if config_path.exists() {
            let content = fs::read_to_string(&config_path).map_err(|e| {
                anyhow::anyhow!(
                    "Failed to read YAML config file {}: {}",
                    config_path.display(),
                    e
                )
            })?;

            let config: TuiConfig = serde_yaml::from_str(&content).map_err(|e| {
                anyhow::anyhow!(
                    "Failed to parse YAML config file {}: {}",
                    config_path.display(),
                    e
                )
            })?;

            config.validate()?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    /// Load configuration from JSON file
    pub fn load_json() -> Result<Self> {
        let config_path = Self::json_config_path()?;

        if config_path.exists() {
            let content = fs::read_to_string(&config_path).map_err(|e| {
                anyhow::anyhow!(
                    "Failed to read JSON config file {}: {}",
                    config_path.display(),
                    e
                )
            })?;

            let config: TuiConfig = serde_json::from_str(&content).map_err(|e| {
                anyhow::anyhow!(
                    "Failed to parse JSON config file {}: {}",
                    config_path.display(),
                    e
                )
            })?;

            config.validate()?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    /// Load configuration from TOML file
    pub fn load_toml() -> Result<Self> {
        let config_path = Self::toml_config_path()?;

        if config_path.exists() {
            let content = fs::read_to_string(&config_path).map_err(|e| {
                anyhow::anyhow!(
                    "Failed to read TOML config file {}: {}",
                    config_path.display(),
                    e
                )
            })?;

            let config: TuiConfig = toml::from_str(&content).map_err(|e| {
                anyhow::anyhow!(
                    "Failed to parse TOML config file {}: {}",
                    config_path.display(),
                    e
                )
            })?;

            config.validate()?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tui_config_default() {
        let config = TuiConfig::default();
        assert_eq!(config.theme, "dark");
        assert!(config.animations);
        assert!(config.mouse);
        assert!(!config.vim_mode);
    }

    #[test]
    fn test_tui_config_validation_valid() {
        let config = TuiConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_tui_config_validation_invalid_font_multiplier() {
        let mut config = TuiConfig::default();
        config.accessibility.font_size_multiplier = 3.0; // Invalid: > 2.0
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_accessibility_config_default() {
        let config = AccessibilityConfig::default();
        assert!(!config.screen_reader_enabled);
        assert!(!config.high_contrast_enabled);
        assert!(!config.animations_disabled);
        assert!(config.announcements_enabled);
        assert_eq!(config.font_size_multiplier, 1.0);
    }

    #[test]
    fn test_config_preset_developer() {
        let preset = ConfigPreset::Developer;
        assert_eq!(format!("{}", preset), "Developer");
    }

    #[test]
    fn test_config_preset_accessibility() {
        let preset = ConfigPreset::Accessibility;
        assert_eq!(format!("{}", preset), "Accessibility");
    }

    #[test]
    fn test_runtime_config_changes() {
        let changes = RuntimeConfigChanges::new()
            .with_theme("dracula")
            .with_vim_mode(true)
            .with_high_contrast(false);

        assert_eq!(changes.theme_name, Some("dracula".to_string()));
        assert_eq!(changes.vim_mode, Some(true));
        assert_eq!(changes.high_contrast_enabled, Some(false));
    }

    #[tokio::test]
    async fn test_config_manager_new() {
        let manager = ConfigManager::new();
        let config = manager.get_config().await;
        assert_eq!(config.theme, "dark"); // Default theme
    }

    #[tokio::test]
    async fn test_config_manager_apply_runtime_changes() {
        let mut manager = ConfigManager::new();

        let changes = RuntimeConfigChanges::new()
            .with_theme("dracula")
            .with_vim_mode(true);

        manager.apply_runtime_changes(changes).await.unwrap();

        let config = manager.get_config().await;
        assert_eq!(config.theme, "dracula");
        assert!(config.vim_mode);
    }

    #[tokio::test]
    async fn test_config_manager_apply_preset() {
        let mut manager = ConfigManager::new();

        manager.apply_preset(ConfigPreset::Developer).await.unwrap();

        let config = manager.get_config().await;
        assert_eq!(config.theme, "dracula");
        assert!(config.vim_mode);
    }

    #[tokio::test]
    async fn test_config_manager_current_preset_developer() {
        let mut manager = ConfigManager::new();
        manager.apply_preset(ConfigPreset::Developer).await.unwrap();

        let preset = manager.current_preset().await;
        assert_eq!(preset, Some(ConfigPreset::Developer));
    }

    #[tokio::test]
    async fn test_config_manager_stop_watching() {
        let mut manager = ConfigManager::new();
        // Should succeed (no-op since watching not implemented)
        assert!(manager.stop_watching().await.is_ok());
    }

    #[test]
    fn test_available_presets() {
        let presets = ConfigManager::available_presets();
        assert_eq!(presets.len(), 4);
        assert!(presets.iter().any(|(p, _)| *p == ConfigPreset::Developer));
        assert!(presets.iter().any(|(p, _)| *p == ConfigPreset::Accessibility));
        assert!(presets.iter().any(|(p, _)| *p == ConfigPreset::Minimal));
        assert!(presets.iter().any(|(p, _)| *p == ConfigPreset::Presentation));
    }
}
