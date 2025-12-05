//! Theme management for the TUI

use crate::style::Theme;
use crate::config::TuiConfig;
use crate::theme_loader::ThemeLoader;
use anyhow::Result;
use std::sync::{Arc, Mutex};
use std::path::Path;

/// Type alias for theme listeners
type ThemeListeners = Arc<Mutex<Vec<Box<dyn Fn(&Theme) + Send>>>>;

/// Theme manager for runtime theme management and switching
#[derive(Clone)]
pub struct ThemeManager {
    /// Current active theme
    current_theme: Arc<Mutex<Theme>>,
    /// Theme change listeners
    listeners: ThemeListeners,
}

impl std::fmt::Debug for ThemeManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ThemeManager")
            .field("current_theme", &self.current_theme)
            .finish()
    }
}

impl ThemeManager {
    /// Create a new theme manager with default theme
    pub fn new() -> Self {
        Self {
            current_theme: Arc::new(Mutex::new(Theme::default())),
            listeners: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create a theme manager with a specific theme
    pub fn with_theme(theme: Theme) -> Self {
        Self {
            current_theme: Arc::new(Mutex::new(theme)),
            listeners: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Get the current theme
    pub fn current(&self) -> Result<Theme> {
        let theme = self.current_theme.lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock theme: {}", e))?
            .clone();
        Ok(theme)
    }

    /// Switch to a theme by name
    pub fn switch_by_name(&self, name: &str) -> Result<()> {
        if let Some(theme) = Theme::by_name(name) {
            self.switch_to(theme)
        } else {
            Err(anyhow::anyhow!("Unknown theme: {}", name))
        }
    }

    /// Switch to a specific theme
    pub fn switch_to(&self, theme: Theme) -> Result<()> {
        let mut current = self.current_theme.lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock theme: {}", e))?;
        *current = theme.clone();

        // Notify listeners
        let listeners = self.listeners.lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock listeners: {}", e))?;
        for listener in listeners.iter() {
            listener(&theme);
        }

        Ok(())
    }

    /// Get all available theme names
    pub fn available_themes(&self) -> Vec<&'static str> {
        Theme::available_themes()
    }

    /// Get the current theme name
    pub fn current_name(&self) -> Result<String> {
        Ok(self.current()?.name)
    }

    /// Load theme from config
    pub fn load_from_config(&self, config: &TuiConfig) -> Result<()> {
        self.switch_by_name(&config.theme)
    }

    /// Save current theme to config
    pub fn save_to_config(&self, config: &mut TuiConfig) -> Result<()> {
        config.theme = self.current_name()?;
        Ok(())
    }

    /// Load a custom theme from a file
    pub fn load_custom_theme(&self, path: &Path) -> Result<()> {
        let theme = ThemeLoader::load_from_file(path)?;
        self.switch_to(theme)
    }

    /// Load all custom themes from a directory
    pub fn load_custom_themes_from_directory(&self, dir: &Path) -> Result<Vec<Theme>> {
        ThemeLoader::load_from_directory(dir)
    }

    /// Save current theme as a custom theme
    pub fn save_custom_theme(&self, path: &Path) -> Result<()> {
        let theme = self.current()?;
        ThemeLoader::save_to_file(&theme, path)
    }

    /// Get the default custom themes directory
    pub fn custom_themes_directory() -> Result<std::path::PathBuf> {
        ThemeLoader::themes_directory()
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_manager_creation() {
        let manager = ThemeManager::new();
        assert_eq!(manager.current().unwrap().name, "dark");
    }

    #[test]
    fn test_theme_manager_with_theme() {
        let theme = Theme::light();
        let manager = ThemeManager::with_theme(theme);
        assert_eq!(manager.current().unwrap().name, "light");
    }

    #[test]
    fn test_switch_by_name() {
        let manager = ThemeManager::new();
        manager.switch_by_name("light").unwrap();
        assert_eq!(manager.current().unwrap().name, "light");

        manager.switch_by_name("monokai").unwrap();
        assert_eq!(manager.current().unwrap().name, "monokai");
    }

    #[test]
    fn test_switch_by_invalid_name() {
        let manager = ThemeManager::new();
        assert!(manager.switch_by_name("invalid").is_err());
    }

    #[test]
    fn test_switch_to() {
        let manager = ThemeManager::new();
        let theme = Theme::dracula();
        manager.switch_to(theme).unwrap();
        assert_eq!(manager.current().unwrap().name, "dracula");
    }

    #[test]
    fn test_available_themes() {
        let manager = ThemeManager::new();
        let themes = manager.available_themes();
        assert_eq!(themes.len(), 6);
    }

    #[test]
    fn test_current_name() {
        let manager = ThemeManager::new();
        assert_eq!(manager.current_name().unwrap(), "dark");

        manager.switch_by_name("nord").unwrap();
        assert_eq!(manager.current_name().unwrap(), "nord");
    }

    #[test]
    fn test_load_from_config() {
        let manager = ThemeManager::new();
        let config = TuiConfig {
            theme: "dracula".to_string(),
            ..Default::default()
        };
        manager.load_from_config(&config).unwrap();
        assert_eq!(manager.current().unwrap().name, "dracula");
    }

    #[test]
    fn test_save_to_config() {
        let manager = ThemeManager::new();
        manager.switch_by_name("monokai").unwrap();

        let mut config = TuiConfig::default();
        manager.save_to_config(&mut config).unwrap();
        assert_eq!(config.theme, "monokai");
    }

    #[test]
    fn test_save_and_load_custom_theme() {
        use tempfile::TempDir;
        
        let temp_dir = TempDir::new().unwrap();
        let theme_path = temp_dir.path().join("custom.yaml");

        let manager = ThemeManager::new();
        manager.switch_by_name("dracula").unwrap();
        manager.save_custom_theme(&theme_path).unwrap();

        let manager2 = ThemeManager::new();
        manager2.load_custom_theme(&theme_path).unwrap();
        assert_eq!(manager2.current().unwrap().name, "dracula");
    }

    #[test]
    fn test_load_custom_themes_from_directory() {
        use tempfile::TempDir;
        
        let temp_dir = TempDir::new().unwrap();

        let manager = ThemeManager::new();
        manager.switch_by_name("dark").unwrap();
        manager.save_custom_theme(&temp_dir.path().join("dark.yaml")).unwrap();

        manager.switch_by_name("light").unwrap();
        manager.save_custom_theme(&temp_dir.path().join("light.yaml")).unwrap();

        let themes = manager.load_custom_themes_from_directory(temp_dir.path()).unwrap();
        assert_eq!(themes.len(), 2);
    }
}
