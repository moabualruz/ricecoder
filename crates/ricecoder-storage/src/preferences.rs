//! User preferences persistence and management
//!
//! This module provides storage and retrieval of user preferences that persist
//! across sessions, separate from the main configuration.

use crate::error::{StorageError, StorageResult};
use crate::manager::PathResolver;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::RwLock;

/// User preferences structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserPreferences {
    /// UI preferences
    pub ui: UiPreferences,
    /// Editor preferences
    pub editor: EditorPreferences,
    /// Terminal preferences
    pub terminal: TerminalPreferences,
    /// Keybinding preferences
    pub keybindings: KeybindingPreferences,
    /// Theme preferences
    pub themes: ThemePreferences,
    /// Custom preferences (extensible)
    pub custom: HashMap<String, serde_json::Value>,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            ui: UiPreferences::default(),
            editor: EditorPreferences::default(),
            terminal: TerminalPreferences::default(),
            keybindings: KeybindingPreferences::default(),
            themes: ThemePreferences::default(),
            custom: HashMap::new(),
        }
    }
}

/// UI-related preferences
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UiPreferences {
    /// Show line numbers in chat
    pub show_line_numbers: bool,
    /// Word wrap in messages
    pub word_wrap: bool,
    /// Maximum message history to keep
    pub max_history: usize,
    /// Show timestamps on messages
    pub show_timestamps: bool,
    /// Compact mode (reduced spacing)
    pub compact_mode: bool,
    /// Sidebar width
    pub sidebar_width: u16,
    /// Auto-hide sidebar
    pub auto_hide_sidebar: bool,
    /// Custom UI preferences
    pub custom: HashMap<String, serde_json::Value>,
}

impl Default for UiPreferences {
    fn default() -> Self {
        Self {
            show_line_numbers: true,
            word_wrap: true,
            max_history: 1000,
            show_timestamps: false,
            compact_mode: false,
            sidebar_width: 30,
            auto_hide_sidebar: false,
            custom: HashMap::new(),
        }
    }
}

/// Editor-related preferences
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EditorPreferences {
    /// Tab size
    pub tab_size: usize,
    /// Use spaces instead of tabs
    pub insert_spaces: bool,
    /// Auto-save interval in seconds (0 = disabled)
    pub auto_save_interval: u64,
    /// Show minimap
    pub show_minimap: bool,
    /// Font size
    pub font_size: u8,
    /// Word wrap in editor
    pub word_wrap: bool,
    /// Show whitespace characters
    pub show_whitespace: bool,
}

impl Default for EditorPreferences {
    fn default() -> Self {
        Self {
            tab_size: 4,
            insert_spaces: true,
            auto_save_interval: 30,
            show_minimap: true,
            font_size: 12,
            word_wrap: true,
            show_whitespace: false,
        }
    }
}

/// Terminal-related preferences
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TerminalPreferences {
    /// Scrollback buffer size
    pub scrollback_size: usize,
    /// Enable mouse support
    pub mouse_support: bool,
    /// Terminal bell style
    pub bell_style: BellStyle,
    /// Cursor style
    pub cursor_style: CursorStyle,
    /// Enable bracketed paste
    pub bracketed_paste: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BellStyle {
    None,
    Visual,
    Sound,
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CursorStyle {
    Block,
    Underline,
    Bar,
}

impl Default for TerminalPreferences {
    fn default() -> Self {
        Self {
            scrollback_size: 10000,
            mouse_support: true,
            bell_style: BellStyle::Visual,
            cursor_style: CursorStyle::Block,
            bracketed_paste: true,
        }
    }
}

/// Keybinding preferences
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KeybindingPreferences {
    /// Custom keybindings (key -> action)
    pub custom_bindings: HashMap<String, String>,
    /// Enable vim mode
    pub vim_mode: bool,
    /// Leader key timeout in milliseconds
    pub leader_timeout: u64,
    /// Show keybinding hints
    pub show_hints: bool,
}

impl Default for KeybindingPreferences {
    fn default() -> Self {
        Self {
            custom_bindings: HashMap::new(),
            vim_mode: false,
            leader_timeout: 1000,
            show_hints: true,
        }
    }
}

/// Theme preferences
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemePreferences {
    /// Current theme name
    pub current_theme: String,
    /// Theme variant (light/dark/auto)
    pub variant: ThemeVariant,
    /// Custom theme overrides
    pub overrides: HashMap<String, String>,
    /// High contrast mode
    pub high_contrast: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ThemeVariant {
    Light,
    Dark,
    Auto,
}

impl Default for ThemePreferences {
    fn default() -> Self {
        Self {
            current_theme: "default".to_string(),
            variant: ThemeVariant::Auto,
            overrides: HashMap::new(),
            high_contrast: false,
        }
    }
}

/// User preferences manager
pub struct PreferencesManager {
    preferences: RwLock<UserPreferences>,
    preferences_file: PathBuf,
    backup_manager: crate::config::validation::ConfigBackupManager,
}

impl PreferencesManager {
    /// Create a new preferences manager
    pub async fn new() -> StorageResult<Self> {
        let preferences_file = Self::get_preferences_path()?;
        let backup_dir = preferences_file.parent().unwrap().join("backups");

        let preferences = Self::load_preferences(&preferences_file).await
            .unwrap_or_else(|_| {
                tracing::info!("Creating default user preferences");
                UserPreferences::default()
            });

        Ok(Self {
            preferences: RwLock::new(preferences),
            preferences_file,
            backup_manager: crate::config::validation::ConfigBackupManager::new(backup_dir),
        })
    }

    /// Get the preferences file path
    fn get_preferences_path() -> StorageResult<PathBuf> {
        let user_dir = PathResolver::resolve_user_path()?;
        Ok(user_dir.join("preferences.yaml"))
    }

    /// Load preferences from file
    async fn load_preferences(path: &PathBuf) -> StorageResult<UserPreferences> {
        use tokio::fs;

        if !path.exists() {
            return Ok(UserPreferences::default());
        }

        let content = fs::read_to_string(path).await?;
        let preferences: UserPreferences = serde_yaml::from_str(&content)?;

        tracing::info!("Loaded user preferences from {}", path.display());
        Ok(preferences)
    }

    /// Save preferences to file
    async fn save_preferences(&self, preferences: &UserPreferences) -> StorageResult<()> {
        use tokio::fs;

        // Ensure directory exists
        if let Some(parent) = self.preferences_file.parent() {
            fs::create_dir_all(parent).await?;
        }

        let content = serde_yaml::to_string(preferences)?;
        fs::write(&self.preferences_file, content).await?;

        tracing::info!("Saved user preferences to {}", self.preferences_file.display());
        Ok(())
    }
}