//! Theme management for the TUI

use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use ratatui::style::{Color, Color as ColorSupport};
use ricecoder_storage::TuiConfig;

use crate::{
    loader::ThemeLoader,
    registry::ThemeRegistry,
    reset::ThemeResetManager,
    types::{Theme, ThemeError, ThemeManager as ThemeManagerTrait},
};

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
                Ok(content) => match ThemeLoader::load_from_string(&content) {
                    Ok(theme) => {
                        self.register_theme(theme)?;
                        loaded_names.push(theme_name);
                    }
                    Err(e) => {
                        eprintln!("Failed to parse custom theme {}: {}", theme_name, e);
                    }
                },
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
        // let content = serde_yaml::to_string(&theme)?; // TODO: implement theme serialization
        let content = format!("{:?}", theme); // Temporary string representation
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
        // let content = serde_yaml::to_string(theme)?; // TODO: implement theme serialization
        let content = format!("{:?}", theme); // Temporary string representation
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

    /// Adapt the current theme to terminal capabilities
    pub fn adapt_to_terminal(&self, _support: ColorSupport) -> Result<()> {
        let current = self
            .current_theme
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock theme: {}", e))?;

        // current.adapt(support); // TODO: implement adapt method

        // Notify listeners of change
        let listeners = self
            .listeners
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock listeners: {}", e))?;
        for listener in listeners.iter() {
            listener(&current);
        }

        Ok(())
    }

    /// Preview a theme without switching to it permanently
    /// Returns the theme so the UI can render a preview
    pub fn preview_theme(&self, name: &str) -> Result<Theme> {
        self.registry
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Theme not found: {}", name))
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ThemeManagerTrait for ThemeManager {
    fn load_theme(&mut self, name: &str) -> Result<(), ThemeError> {
        if let Some(theme) = self.registry.get(name) {
            let mut current = self
                .current_theme
                .lock()
                .map_err(|e| ThemeError::Parse(format!("Lock error: {}", e)))?;
            *current = theme.clone();
            Ok(())
        } else {
            Err(ThemeError::NotFound(name.to_string()))
        }
    }

    fn get_theme(&self, name: &str) -> Option<Theme> {
        self.registry.get(name)
    }

    fn list_themes(&self) -> Vec<String> {
        self.registry.list_all().unwrap_or_default()
    }
}
