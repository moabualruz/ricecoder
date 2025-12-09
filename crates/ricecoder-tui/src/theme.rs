//! Theme management for the TUI

use crate::config::TuiConfig;
use crate::style::{Color, Theme};
use crate::theme_loader::ThemeLoader;
use crate::theme_registry::ThemeRegistry;
use crate::theme_reset::ThemeResetManager;
use anyhow::Result;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Type alias for theme listeners
type ThemeListeners = Arc<Mutex<Vec<Box<dyn Fn(&Theme) + Send>>>>;

/// Theme manager for runtime theme management and switching
#[derive(Clone)]
pub struct ThemeManager {
    /// Current active theme
    current_theme: Arc<Mutex<Theme>>,
    /// Theme change listeners
    listeners: ThemeListeners,
    /// Theme registry for managing all themes
    registry: ThemeRegistry,
    /// Theme reset manager for resetting themes to defaults
    reset_manager: Arc<ThemeResetManager>,
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
            registry: ThemeRegistry::new(),
            reset_manager: Arc::new(ThemeResetManager::new()),
        }
    }

    /// Create a theme manager with a specific theme
    pub fn with_theme(theme: Theme) -> Self {
        Self {
            current_theme: Arc::new(Mutex::new(theme)),
            listeners: Arc::new(Mutex::new(Vec::new())),
            registry: ThemeRegistry::new(),
            reset_manager: Arc::new(ThemeResetManager::new()),
        }
    }

    /// Create a theme manager with a custom registry
    pub fn with_registry(registry: ThemeRegistry) -> Self {
        Self {
            current_theme: Arc::new(Mutex::new(Theme::default())),
            listeners: Arc::new(Mutex::new(Vec::new())),
            registry,
            reset_manager: Arc::new(ThemeResetManager::new()),
        }
    }

    /// Get the current theme
    pub fn current(&self) -> Result<Theme> {
        let theme = self
            .current_theme
            .lock()
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
        let mut current = self
            .current_theme
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock theme: {}", e))?;
        *current = theme.clone();

        // Notify listeners
        let listeners = self
            .listeners
            .lock()
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

    /// Load theme preference from storage
    pub fn load_from_storage(&self) -> Result<()> {
        use ricecoder_storage::ThemeStorage;
        let preference = ThemeStorage::load_preference()?;
        self.switch_by_name(&preference.current_theme)
    }

    /// Save current theme preference to storage
    pub fn save_to_storage(&self) -> Result<()> {
        use ricecoder_storage::ThemeStorage;
        let theme_name = self.current_name()?;
        let preference = ricecoder_storage::ThemePreference {
            current_theme: theme_name,
            last_updated: Some(chrono::Local::now().to_rfc3339()),
        };
        ThemeStorage::save_preference(&preference)?;
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

    /// Load all custom themes from a directory and register them
    pub fn load_and_register_custom_themes(&self, dir: &Path) -> Result<Vec<String>> {
        let themes = self.load_custom_themes_from_directory(dir)?;
        let mut names = Vec::new();
        for theme in themes {
            names.push(theme.name.clone());
            self.register_theme(theme)?;
        }
        Ok(names)
    }

    /// Load all custom themes from storage and register them
    pub fn load_custom_themes_from_storage(&self) -> Result<Vec<String>> {
        use ricecoder_storage::ThemeStorage;
        let theme_names = ThemeStorage::list_custom_themes()?;
        let mut loaded_names = Vec::new();

        for theme_name in theme_names {
            match ThemeStorage::load_custom_theme(&theme_name) {
                Ok(content) => {
                    match ThemeLoader::load_from_string(&content) {
                        Ok(theme) => {
                            self.register_theme(theme)?;
                            loaded_names.push(theme_name);
                        }
                        Err(e) => {
                            eprintln!("Failed to parse custom theme {}: {}", theme_name, e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to load custom theme {}: {}", theme_name, e);
                }
            }
        }

        Ok(loaded_names)
    }

    /// Save current theme as a custom theme to storage
    pub fn save_custom_theme(&self, path: &Path) -> Result<()> {
        let theme = self.current()?;
        ThemeLoader::save_to_file(&theme, path)?;
        // Also register it in the registry
        self.register_theme(theme)?;
        Ok(())
    }

    /// Save current theme as a custom theme to storage by name
    pub fn save_custom_theme_to_storage(&self, theme_name: &str) -> Result<()> {
        use ricecoder_storage::ThemeStorage;
        let theme = self.current()?;
        let content = serde_yaml::to_string(&theme)?;
        ThemeStorage::save_custom_theme(theme_name, &content)?;
        // Also register it in the registry
        self.register_theme(theme)?;
        Ok(())
    }

    /// Save a specific theme as a custom theme
    pub fn save_theme_as_custom(&self, theme: &Theme, path: &Path) -> Result<()> {
        ThemeLoader::save_to_file(theme, path)?;
        // Also register it in the registry
        self.register_theme(theme.clone())?;
        Ok(())
    }

    /// Save a specific theme as a custom theme to storage by name
    pub fn save_theme_as_custom_to_storage(&self, theme: &Theme, theme_name: &str) -> Result<()> {
        use ricecoder_storage::ThemeStorage;
        let content = serde_yaml::to_string(theme)?;
        ThemeStorage::save_custom_theme(theme_name, &content)?;
        // Also register it in the registry
        self.register_theme(theme.clone())?;
        Ok(())
    }

    /// Delete a custom theme file and unregister it
    pub fn delete_custom_theme(&self, name: &str, path: &Path) -> Result<()> {
        // Remove the file
        std::fs::remove_file(path)?;
        // Unregister from registry
        self.unregister_theme(name)?;
        Ok(())
    }

    /// Delete a custom theme from storage and unregister it
    pub fn delete_custom_theme_from_storage(&self, theme_name: &str) -> Result<()> {
        use ricecoder_storage::ThemeStorage;
        ThemeStorage::delete_custom_theme(theme_name)?;
        // Unregister from registry
        self.unregister_theme(theme_name)?;
        Ok(())
    }

    /// Get the default custom themes directory
    pub fn custom_themes_directory() -> Result<std::path::PathBuf> {
        ThemeLoader::themes_directory()
    }

    /// Get the theme registry
    pub fn registry(&self) -> &ThemeRegistry {
        &self.registry
    }

    /// List all available themes (built-in and custom)
    pub fn list_all_themes(&self) -> Result<Vec<String>> {
        self.registry.list_all()
    }

    /// List all built-in themes
    pub fn list_builtin_themes(&self) -> Vec<String> {
        self.registry.list_builtin()
    }

    /// List all custom themes
    pub fn list_custom_themes(&self) -> Result<Vec<String>> {
        self.registry.list_custom()
    }

    /// Register a custom theme in the registry
    pub fn register_theme(&self, theme: Theme) -> Result<()> {
        self.registry.register(theme)
    }

    /// Unregister a custom theme from the registry
    pub fn unregister_theme(&self, name: &str) -> Result<()> {
        self.registry.unregister(name)
    }

    /// Check if a theme exists
    pub fn theme_exists(&self, name: &str) -> bool {
        self.registry.exists(name)
    }

    /// Check if a theme is built-in
    pub fn is_builtin_theme(&self, name: &str) -> bool {
        self.registry.is_builtin(name)
    }

    /// Check if a theme is custom
    pub fn is_custom_theme(&self, name: &str) -> Result<bool> {
        self.registry.is_custom(name)
    }

    /// Get the number of built-in themes
    pub fn builtin_theme_count(&self) -> usize {
        self.registry.builtin_count()
    }

    /// Get the number of custom themes
    pub fn custom_theme_count(&self) -> Result<usize> {
        self.registry.custom_count()
    }

    /// Reset all colors in the current theme to their default values
    pub fn reset_colors(&self) -> Result<()> {
        let mut current = self
            .current_theme
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock theme: {}", e))?;

        self.reset_manager.reset_colors(&mut current)?;

        // Notify listeners of the reset
        let listeners = self
            .listeners
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock listeners: {}", e))?;
        for listener in listeners.iter() {
            listener(&current);
        }

        Ok(())
    }

    /// Reset the current theme to its built-in default
    pub fn reset_theme(&self) -> Result<()> {
        let mut current = self
            .current_theme
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock theme: {}", e))?;

        self.reset_manager.reset_theme(&mut current)?;

        // Notify listeners of the reset
        let listeners = self
            .listeners
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock listeners: {}", e))?;
        for listener in listeners.iter() {
            listener(&current);
        }

        Ok(())
    }

    /// Reset a specific color field in the current theme to its default value
    pub fn reset_color(&self, color_name: &str) -> Result<()> {
        let mut current = self
            .current_theme
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock theme: {}", e))?;

        self.reset_manager.reset_color(&mut current, color_name)?;

        // Notify listeners of the reset
        let listeners = self
            .listeners
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock listeners: {}", e))?;
        for listener in listeners.iter() {
            listener(&current);
        }

        Ok(())
    }

    /// Get the default color value for a specific color field in the current theme
    pub fn get_default_color(&self, color_name: &str) -> Result<Color> {
        let current = self
            .current_theme
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock theme: {}", e))?;

        self.reset_manager
            .get_default_color(&current.name, color_name)
    }

    /// Get the theme reset manager
    pub fn reset_manager(&self) -> &ThemeResetManager {
        &self.reset_manager
    }

    /// Register a listener for theme changes
    pub fn on_theme_changed<F>(&self, listener: F) -> Result<()>
    where
        F: Fn(&Theme) + Send + 'static,
    {
        let mut listeners = self
            .listeners
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock listeners: {}", e))?;
        listeners.push(Box::new(listener));
        Ok(())
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
        manager
            .save_custom_theme(&temp_dir.path().join("dark.yaml"))
            .unwrap();

        manager.switch_by_name("light").unwrap();
        manager
            .save_custom_theme(&temp_dir.path().join("light.yaml"))
            .unwrap();

        let themes = manager
            .load_custom_themes_from_directory(temp_dir.path())
            .unwrap();
        assert_eq!(themes.len(), 2);
    }

    #[test]
    fn test_reset_colors() {
        let manager = ThemeManager::new();
        let original_primary = manager.current().unwrap().primary;

        // Modify current theme
        {
            let mut current = manager.current_theme.lock().unwrap();
            current.primary = crate::style::Color::new(255, 0, 0);
        }

        // Verify modification
        assert_ne!(manager.current().unwrap().primary, original_primary);

        // Reset colors
        manager.reset_colors().unwrap();

        // Verify reset
        assert_eq!(manager.current().unwrap().primary, original_primary);
    }

    #[test]
    fn test_reset_theme() {
        let manager = ThemeManager::new();
        manager.switch_by_name("light").unwrap();

        let original_theme = Theme::light();

        // Modify current theme
        {
            let mut current = manager.current_theme.lock().unwrap();
            current.primary = crate::style::Color::new(255, 0, 0);
            current.background = crate::style::Color::new(100, 100, 100);
        }

        // Verify modification
        let modified = manager.current().unwrap();
        assert_ne!(modified.primary, original_theme.primary);
        assert_ne!(modified.background, original_theme.background);

        // Reset theme
        manager.reset_theme().unwrap();

        // Verify reset
        let reset = manager.current().unwrap();
        assert_eq!(reset.primary, original_theme.primary);
        assert_eq!(reset.background, original_theme.background);
    }

    #[test]
    fn test_reset_color() {
        let manager = ThemeManager::new();
        let original_error = manager.current().unwrap().error;

        // Modify error color
        {
            let mut current = manager.current_theme.lock().unwrap();
            current.error = crate::style::Color::new(255, 0, 0);
        }

        // Verify modification
        assert_ne!(manager.current().unwrap().error, original_error);

        // Reset error color
        manager.reset_color("error").unwrap();

        // Verify reset
        assert_eq!(manager.current().unwrap().error, original_error);
    }

    #[test]
    fn test_get_default_color() {
        let manager = ThemeManager::new();
        let default_primary = manager.get_default_color("primary").unwrap();
        let current_primary = manager.current().unwrap().primary;
        assert_eq!(default_primary, current_primary);
    }

    #[test]
    fn test_reset_notifies_listeners() {
        let manager = ThemeManager::new();
        let listener_called = std::sync::Arc::new(std::sync::Mutex::new(false));
        let listener_called_clone = listener_called.clone();

        manager
            .on_theme_changed(move |_theme| {
                *listener_called_clone.lock().unwrap() = true;
            })
            .unwrap();

        manager.reset_colors().unwrap();

        assert!(*listener_called.lock().unwrap());
    }

    #[test]
    fn test_load_from_storage() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        std::env::set_var("RICECODER_HOME", temp_dir.path());

        // Save a preference
        let pref = ricecoder_storage::ThemePreference {
            current_theme: "light".to_string(),
            last_updated: None,
        };
        ricecoder_storage::ThemeStorage::save_preference(&pref).unwrap();

        // Load it with theme manager
        let manager = ThemeManager::new();
        manager.load_from_storage().unwrap();
        assert_eq!(manager.current().unwrap().name, "light");

        std::env::remove_var("RICECODER_HOME");
    }

    #[test]
    fn test_save_to_storage() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        std::env::set_var("RICECODER_HOME", temp_dir.path());

        let manager = ThemeManager::new();
        manager.switch_by_name("dracula").unwrap();
        manager.save_to_storage().unwrap();

        // Verify it was saved
        let loaded_pref = ricecoder_storage::ThemeStorage::load_preference().unwrap();
        assert_eq!(loaded_pref.current_theme, "dracula");

        std::env::remove_var("RICECODER_HOME");
    }

    #[test]
    fn test_save_custom_theme_to_storage() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        std::env::set_var("RICECODER_HOME", temp_dir.path());

        let manager = ThemeManager::new();
        manager.switch_by_name("monokai").unwrap();
        manager.save_custom_theme_to_storage("my_custom").unwrap();

        // Verify it was saved
        assert!(ricecoder_storage::ThemeStorage::custom_theme_exists("my_custom").unwrap());

        std::env::remove_var("RICECODER_HOME");
    }

    #[test]
    fn test_load_custom_themes_from_storage() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        std::env::set_var("RICECODER_HOME", temp_dir.path());

        // Save some custom themes
        ricecoder_storage::ThemeStorage::save_custom_theme(
            "custom1",
            "name: custom1\nprimary: \"#0078ff\"\nsecondary: \"#5ac8fa\"\naccent: \"#ff2d55\"\nbackground: \"#111827\"\nforeground: \"#f3f4f6\"\nerror: \"#ef4444\"\nwarning: \"#f59e0b\"\nsuccess: \"#22c55e\""
        ).unwrap();

        let manager = ThemeManager::new();
        let loaded = manager.load_custom_themes_from_storage().unwrap();
        assert_eq!(loaded.len(), 1);
        assert!(loaded.contains(&"custom1".to_string()));

        std::env::remove_var("RICECODER_HOME");
    }

    #[test]
    fn test_delete_custom_theme_from_storage() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        std::env::set_var("RICECODER_HOME", temp_dir.path());

        // Save a custom theme
        ricecoder_storage::ThemeStorage::save_custom_theme(
            "to_delete",
            "name: to_delete\nprimary: \"#0078ff\"\nsecondary: \"#5ac8fa\"\naccent: \"#ff2d55\"\nbackground: \"#111827\"\nforeground: \"#f3f4f6\"\nerror: \"#ef4444\"\nwarning: \"#f59e0b\"\nsuccess: \"#22c55e\""
        ).unwrap();

        let manager = ThemeManager::new();
        manager.delete_custom_theme_from_storage("to_delete").unwrap();

        // Verify it was deleted
        assert!(!ricecoder_storage::ThemeStorage::custom_theme_exists("to_delete").unwrap());

        std::env::remove_var("RICECODER_HOME");
    }
}
