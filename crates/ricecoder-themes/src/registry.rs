//! Theme registry for managing built-in and custom themes

use crate::style::Theme;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Theme registry for storing and managing themes
#[derive(Clone)]
pub struct ThemeRegistry {
    /// Built-in themes (immutable)
    builtin_themes: Arc<HashMap<String, Theme>>,
    /// Custom themes (mutable)
    custom_themes: Arc<RwLock<HashMap<String, Theme>>>,
}

impl ThemeRegistry {
    /// Create a new theme registry with built-in themes
    pub fn new() -> Self {
        let mut builtin = HashMap::new();

        // Register all built-in themes dynamically from Theme::available_themes()
        for name in Theme::available_themes() {
            if let Some(theme) = Theme::by_name(name) {
                builtin.insert(name.to_string(), theme);
            }
        }

        Self {
            builtin_themes: Arc::new(builtin),
            custom_themes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get a theme by name (checks built-in first, then custom)
    pub fn get(&self, name: &str) -> Option<Theme> {
        // Check built-in themes first
        if let Some(theme) = self.builtin_themes.get(name) {
            return Some(theme.clone());
        }

        // Check custom themes
        if let Ok(custom) = self.custom_themes.read() {
            if let Some(theme) = custom.get(name) {
                return Some(theme.clone());
            }
        }

        None
    }

    /// Get a built-in theme by name
    pub fn get_builtin(&self, name: &str) -> Option<Theme> {
        self.builtin_themes.get(name).cloned()
    }

    /// Register a custom theme
    pub fn register(&self, theme: Theme) -> Result<()> {
        let mut custom = self
            .custom_themes
            .write()
            .map_err(|e| anyhow!("Failed to lock custom themes: {}", e))?;
        custom.insert(theme.name.clone(), theme);
        Ok(())
    }

    /// Unregister a custom theme
    pub fn unregister(&self, name: &str) -> Result<()> {
        let mut custom = self
            .custom_themes
            .write()
            .map_err(|e| anyhow!("Failed to lock custom themes: {}", e))?;
        custom.remove(name);
        Ok(())
    }

    /// List all available theme names (built-in and custom)
    pub fn list_all(&self) -> Result<Vec<String>> {
        let mut names = Vec::new();

        // Add built-in themes
        names.extend(self.builtin_themes.keys().cloned());

        // Add custom themes
        let custom = self
            .custom_themes
            .read()
            .map_err(|e| anyhow!("Failed to lock custom themes: {}", e))?;
        names.extend(custom.keys().cloned());

        names.sort();
        Ok(names)
    }

    /// List all built-in theme names
    pub fn list_builtin(&self) -> Vec<String> {
        let mut names: Vec<_> = self.builtin_themes.keys().cloned().collect();
        names.sort();
        names
    }

    /// List all custom theme names
    pub fn list_custom(&self) -> Result<Vec<String>> {
        let custom = self
            .custom_themes
            .read()
            .map_err(|e| anyhow!("Failed to lock custom themes: {}", e))?;
        let mut names: Vec<_> = custom.keys().cloned().collect();
        names.sort();
        Ok(names)
    }

    /// Check if a theme exists (built-in or custom)
    pub fn exists(&self, name: &str) -> bool {
        self.builtin_themes.contains_key(name)
            || self
                .custom_themes
                .read()
                .map(|custom| custom.contains_key(name))
                .unwrap_or(false)
    }

    /// Check if a theme is built-in
    pub fn is_builtin(&self, name: &str) -> bool {
        self.builtin_themes.contains_key(name)
    }

    /// Check if a theme is custom
    pub fn is_custom(&self, name: &str) -> Result<bool> {
        let custom = self
            .custom_themes
            .read()
            .map_err(|e| anyhow!("Failed to lock custom themes: {}", e))?;
        Ok(custom.contains_key(name))
    }

    /// Get the number of built-in themes
    pub fn builtin_count(&self) -> usize {
        self.builtin_themes.len()
    }

    /// Get the number of custom themes
    pub fn custom_count(&self) -> Result<usize> {
        let custom = self
            .custom_themes
            .read()
            .map_err(|e| anyhow!("Failed to lock custom themes: {}", e))?;
        Ok(custom.len())
    }

    /// Reset a custom theme to its built-in default
    pub fn reset_to_default(&self, name: &str) -> Result<()> {
        if let Some(builtin) = self.get_builtin(name) {
            self.register(builtin)?;
            Ok(())
        } else {
            Err(anyhow!("Theme not found: {}", name))
        }
    }

    /// Clear all custom themes
    pub fn clear_custom(&self) -> Result<()> {
        let mut custom = self
            .custom_themes
            .write()
            .map_err(|e| anyhow!("Failed to lock custom themes: {}", e))?;
        custom.clear();
        Ok(())
    }
}


impl Default for ThemeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ThemeRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ThemeRegistry")
            .field("builtin_count", &self.builtin_count())
            .field("custom_count", &self.custom_count().unwrap_or(0))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = ThemeRegistry::new();
        assert_eq!(registry.builtin_count(), Theme::available_themes().len());
        assert_eq!(registry.custom_count().unwrap(), 0);
    }

    #[test]
    fn test_get_builtin_theme() {
        let registry = ThemeRegistry::new();
        assert!(registry.get("dark").is_some());
        assert!(registry.get("light").is_some());
        assert!(registry.get("monokai").is_some());
        assert!(registry.get("dracula").is_some());
        assert!(registry.get("nord").is_some());
        assert!(registry.get("high-contrast").is_some());
    }

    #[test]
    fn test_get_nonexistent_theme() {
        let registry = ThemeRegistry::new();
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_register_custom_theme() {
        let registry = ThemeRegistry::new();
        let custom_theme = Theme::light();
        let mut custom = custom_theme.clone();
        custom.name = "my-custom".to_string();

        registry.register(custom).unwrap();
        assert_eq!(registry.custom_count().unwrap(), 1);
        assert!(registry.get("my-custom").is_some());
    }

    #[test]
    fn test_unregister_custom_theme() {
        let registry = ThemeRegistry::new();
        let custom_theme = Theme::light();
        let mut custom = custom_theme.clone();
        custom.name = "my-custom".to_string();

        registry.register(custom).unwrap();
        assert_eq!(registry.custom_count().unwrap(), 1);

        registry.unregister("my-custom").unwrap();
        assert_eq!(registry.custom_count().unwrap(), 0);
    }

    #[test]
    fn test_list_all_themes() {
        let registry = ThemeRegistry::new();
        let custom_theme = Theme::light();
        let mut custom = custom_theme.clone();
        custom.name = "my-custom".to_string();
        registry.register(custom).unwrap();

        let all = registry.list_all().unwrap();
        assert_eq!(all.len(), Theme::available_themes().len() + 1);
    }

    #[test]
    fn test_list_builtin_themes() {
        let registry = ThemeRegistry::new();
        let builtin = registry.list_builtin();
        assert_eq!(builtin.len(), Theme::available_themes().len());
    }

    #[test]
    fn test_list_custom_themes() {
        let registry = ThemeRegistry::new();
        let custom_theme = Theme::light();
        let mut custom = custom_theme.clone();
        custom.name = "my-custom".to_string();
        registry.register(custom).unwrap();

        let custom_list = registry.list_custom().unwrap();
        assert_eq!(custom_list.len(), 1);
    }

    #[test]
    fn test_exists() {
        let registry = ThemeRegistry::new();
        assert!(registry.exists("dark"));
        assert!(!registry.exists("nonexistent"));
    }

    #[test]
    fn test_is_builtin() {
        let registry = ThemeRegistry::new();
        assert!(registry.is_builtin("dark"));
        assert!(!registry.is_builtin("nonexistent"));
    }

    #[test]
    fn test_is_custom() {
        let registry = ThemeRegistry::new();
        let custom_theme = Theme::light();
        let mut custom = custom_theme.clone();
        custom.name = "my-custom".to_string();
        registry.register(custom).unwrap();

        assert!(registry.is_custom("my-custom").unwrap());
        assert!(!registry.is_custom("dark").unwrap());
    }

    #[test]
    fn test_reset_to_default() {
        let registry = ThemeRegistry::new();
        let custom_theme = Theme::light();
        let mut custom = custom_theme.clone();
        custom.name = "dark".to_string();
        registry.register(custom).unwrap();

        registry.reset_to_default("dark").unwrap();
        let theme = registry.get("dark").unwrap();
        assert_eq!(theme.name, "dark");
    }

    #[test]
    fn test_clear_custom() {
        let registry = ThemeRegistry::new();
        let custom_theme = Theme::light();
        let mut custom = custom_theme.clone();
        custom.name = "my-custom".to_string();
        registry.register(custom).unwrap();

        assert_eq!(registry.custom_count().unwrap(), 1);
        registry.clear_custom().unwrap();
        assert_eq!(registry.custom_count().unwrap(), 0);
    }
}
