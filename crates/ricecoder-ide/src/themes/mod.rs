//! IDE Theme System
//!
//! This module provides theme management for the IDE, re-exporting theme types from ricecoder-tui
//! and providing IDE-specific theme manager wrapper functionality.
//!
//! # Architecture
//!
//! The IDE theme system is built on top of the TUI theme system, providing:
//! - Re-exported theme types (Color, Theme, ThemeManager)
//! - IDE-specific theme manager wrapper
//! - Integration with IDE configuration
//! - Theme persistence through ricecoder-storage
//!
//! # Example
//!
//! ```ignore
//! use ricecoder_ide::themes::{IdeThemeManager, Color, Theme};
//!
//! // Create IDE theme manager
//! let theme_manager = IdeThemeManager::new();
//!
//! // Switch to a theme
//! theme_manager.switch_by_name("dracula")?;
//!
//! // Get current theme
//! let current = theme_manager.current()?;
//! println!("Current theme: {}", current.name);
//! ```

pub mod integration;

use anyhow::Result;
use std::path::Path;

// Re-export theme types from ricecoder-themes and ricecoder-tui
pub use ratatui::style::Color;
pub use ricecoder_tui::style::ColorSupport;
pub use ricecoder_themes::types::Theme;
pub use ricecoder_themes::manager::ThemeManager;
pub use ricecoder_themes::loader::ThemeLoader;
pub use ricecoder_themes::registry::ThemeRegistry;
pub use ricecoder_themes::reset::ThemeResetManager;

// Re-export integration types
pub use integration::{IdeThemeConfig, IdeThemeIntegration};

/// IDE-specific theme manager wrapper
///
/// Provides IDE-specific functionality on top of the base ThemeManager,
/// including integration with IDE configuration and storage.
#[derive(Clone, Debug)]
pub struct IdeThemeManager {
    /// Underlying theme manager from ricecoder-tui
    inner: ThemeManager,
}

impl IdeThemeManager {
    /// Create a new IDE theme manager with default theme
    pub fn new() -> Self {
        Self {
            inner: ThemeManager::new(),
        }
    }

    /// Create an IDE theme manager with a specific theme
    pub fn with_theme(theme: Theme) -> Self {
        Self {
            inner: ThemeManager::with_theme(theme),
        }
    }

    /// Create an IDE theme manager with a custom registry
    pub fn with_registry(registry: ThemeRegistry) -> Self {
        Self {
            inner: ThemeManager::with_registry(registry),
        }
    }

    /// Get the current theme
    pub fn current(&self) -> Result<Theme> {
        self.inner.current()
    }

    /// Switch to a theme by name
    pub fn switch_by_name(&self, name: &str) -> Result<()> {
        self.inner.switch_by_name(name)
    }

    /// Switch to a specific theme
    pub fn switch_to(&self, theme: Theme) -> Result<()> {
        self.inner.switch_to(theme)
    }

    /// Get all available theme names
    pub fn available_themes(&self) -> Vec<&'static str> {
        self.inner.available_themes()
    }

    /// Get the current theme name
    pub fn current_name(&self) -> Result<String> {
        self.inner.current_name()
    }

    /// Load a custom theme from a file
    pub fn load_custom_theme(&self, path: &Path) -> Result<()> {
        self.inner.load_custom_theme(path)
    }

    /// Load all custom themes from a directory
    pub fn load_custom_themes_from_directory(&self, dir: &Path) -> Result<Vec<Theme>> {
        self.inner.load_custom_themes_from_directory(dir)
    }

    /// Load all custom themes from a directory and register them
    pub fn load_and_register_custom_themes(&self, dir: &Path) -> Result<Vec<String>> {
        self.inner.load_and_register_custom_themes(dir)
    }

    /// Save current theme as a custom theme
    pub fn save_custom_theme(&self, path: &Path) -> Result<()> {
        self.inner.save_custom_theme(path)
    }

    /// Save a specific theme as a custom theme
    pub fn save_theme_as_custom(&self, theme: &Theme, path: &Path) -> Result<()> {
        self.inner.save_theme_as_custom(theme, path)
    }

    /// Delete a custom theme file and unregister it
    pub fn delete_custom_theme(&self, name: &str, path: &Path) -> Result<()> {
        self.inner.delete_custom_theme(name, path)
    }

    /// Get the default custom themes directory
    pub fn custom_themes_directory() -> Result<std::path::PathBuf> {
        ThemeManager::custom_themes_directory()
    }

    /// Get the theme registry
    pub fn registry(&self) -> &ThemeRegistry {
        self.inner.registry()
    }

    /// List all available themes (built-in and custom)
    pub fn list_all_themes(&self) -> Result<Vec<String>> {
        self.inner.list_all_themes()
    }

    /// List all built-in themes
    pub fn list_builtin_themes(&self) -> Vec<String> {
        self.inner.list_builtin_themes()
    }

    /// List all custom themes
    pub fn list_custom_themes(&self) -> Result<Vec<String>> {
        self.inner.list_custom_themes()
    }

    /// Register a custom theme in the registry
    pub fn register_theme(&self, theme: Theme) -> Result<()> {
        self.inner.register_theme(theme)
    }

    /// Unregister a custom theme from the registry
    pub fn unregister_theme(&self, name: &str) -> Result<()> {
        self.inner.unregister_theme(name)
    }

    /// Check if a theme exists
    pub fn theme_exists(&self, name: &str) -> bool {
        self.inner.theme_exists(name)
    }

    /// Check if a theme is built-in
    pub fn is_builtin_theme(&self, name: &str) -> bool {
        self.inner.is_builtin_theme(name)
    }

    /// Check if a theme is custom
    pub fn is_custom_theme(&self, name: &str) -> Result<bool> {
        self.inner.is_custom_theme(name)
    }

    /// Get the number of built-in themes
    pub fn builtin_theme_count(&self) -> usize {
        self.inner.builtin_theme_count()
    }

    /// Get the number of custom themes
    pub fn custom_theme_count(&self) -> Result<usize> {
        self.inner.custom_theme_count()
    }

    /// Reset all colors in the current theme to their default values
    pub fn reset_colors(&self) -> Result<()> {
        self.inner.reset_colors()
    }

    /// Reset the current theme to its built-in default
    pub fn reset_theme(&self) -> Result<()> {
        self.inner.reset_theme()
    }

    /// Reset a specific color field in the current theme to its default value
    pub fn reset_color(&self, color_name: &str) -> Result<()> {
        self.inner.reset_color(color_name)
    }

    /// Get the default color value for a specific color field in the current theme
    pub fn get_default_color(&self, color_name: &str) -> Result<Color> {
        self.inner.get_default_color(color_name)
    }

    /// Get the theme reset manager
    pub fn reset_manager(&self) -> &ThemeResetManager {
        self.inner.reset_manager()
    }

    /// Register a listener for theme changes
    pub fn on_theme_changed<F>(&self, listener: F) -> Result<()>
    where
        F: Fn(&Theme) + Send + 'static,
    {
        self.inner.on_theme_changed(listener)
    }

    /// Get the underlying theme manager
    pub fn inner(&self) -> &ThemeManager {
        &self.inner
    }
}

impl Default for IdeThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ide_theme_manager_creation() {
        let manager = IdeThemeManager::new();
        assert_eq!(manager.current().unwrap().name, "dark");
    }

    #[test]
    fn test_ide_theme_manager_with_theme() {
        let theme = Theme::light();
        let manager = IdeThemeManager::with_theme(theme);
        assert_eq!(manager.current().unwrap().name, "light");
    }

    #[test]
    fn test_ide_switch_by_name() {
        let manager = IdeThemeManager::new();
        manager.switch_by_name("dracula").unwrap();
        assert_eq!(manager.current().unwrap().name, "dracula");
    }

    #[test]
    fn test_ide_available_themes() {
        let manager = IdeThemeManager::new();
        let themes = manager.available_themes();
        assert_eq!(themes.len(), 6);
        assert!(themes.contains(&"dark"));
        assert!(themes.contains(&"light"));
        assert!(themes.contains(&"dracula"));
        assert!(themes.contains(&"monokai"));
        assert!(themes.contains(&"nord"));
        assert!(themes.contains(&"high-contrast"));
    }

    #[test]
    fn test_ide_current_name() {
        let manager = IdeThemeManager::new();
        assert_eq!(manager.current_name().unwrap(), "dark");

        manager.switch_by_name("nord").unwrap();
        assert_eq!(manager.current_name().unwrap(), "nord");
    }

    #[test]
    fn test_ide_theme_exists() {
        let manager = IdeThemeManager::new();
        assert!(manager.theme_exists("dark"));
        assert!(manager.theme_exists("light"));
        assert!(!manager.theme_exists("nonexistent"));
    }

    #[test]
    fn test_ide_is_builtin_theme() {
        let manager = IdeThemeManager::new();
        assert!(manager.is_builtin_theme("dark"));
        assert!(manager.is_builtin_theme("light"));
        assert!(!manager.is_builtin_theme("nonexistent"));
    }

    #[test]
    fn test_ide_builtin_theme_count() {
        let manager = IdeThemeManager::new();
        assert_eq!(manager.builtin_theme_count(), 6);
    }

    #[test]
    fn test_ide_reset_colors() {
        let manager = IdeThemeManager::new();
        let original_primary = manager.current().unwrap().primary;

        // Reset colors (should restore to defaults)
        manager.reset_colors().unwrap();

        // Verify reset
        assert_eq!(manager.current().unwrap().primary, original_primary);
    }

    #[test]
    fn test_ide_reset_theme() {
        let manager = IdeThemeManager::new();
        manager.switch_by_name("light").unwrap();

        let original_theme = Theme::light();

        // Reset theme
        manager.reset_theme().unwrap();

        // Verify reset
        let reset = manager.current().unwrap();
        assert_eq!(reset.primary, original_theme.primary);
        assert_eq!(reset.background, original_theme.background);
    }

    #[test]
    fn test_ide_default() {
        let manager = IdeThemeManager::default();
        assert_eq!(manager.current().unwrap().name, "dark");
    }
}
