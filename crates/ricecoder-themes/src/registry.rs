//! Theme registry for managing built-in and custom themes
//!
//! Themes are loaded from JSON files in:
//! 1. Bundled themes: `config/themes/*.json` (relative to workspace)
//! 2. User themes: `~/.config/ricecoder/themes/*.json`

use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, RwLock},
};

use anyhow::{anyhow, Result};

use crate::{loader::ThemeLoader, types::Theme};

/// Theme registry for storing and managing themes
#[derive(Clone)]
pub struct ThemeRegistry {
    /// Built-in themes (immutable, loaded from bundled JSON)
    builtin_themes: Arc<HashMap<String, Theme>>,
    /// Custom themes (mutable, loaded from user directory or registered at runtime)
    custom_themes: Arc<RwLock<HashMap<String, Theme>>>,
}

impl ThemeRegistry {
    /// Create a new theme registry with built-in themes loaded from JSON files
    pub fn new() -> Self {
        let builtin = Self::load_builtin_themes();

        Self {
            builtin_themes: Arc::new(builtin),
            custom_themes: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Create a registry with themes loaded from a specific directory
    pub fn from_directory(dir: &Path) -> Result<Self> {
        let themes = ThemeLoader::load_from_directory(dir)?;
        let mut builtin = HashMap::new();
        for theme in themes {
            builtin.insert(theme.name.clone(), theme);
        }
        
        Ok(Self {
            builtin_themes: Arc::new(builtin),
            custom_themes: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// Load built-in themes from bundled JSON files
    fn load_builtin_themes() -> HashMap<String, Theme> {
        let mut themes = HashMap::new();
        
        // Try to find bundled themes directory
        if let Some(bundled_dir) = ThemeLoader::bundled_themes_directory() {
            match ThemeLoader::load_from_directory(&bundled_dir) {
                Ok(loaded) => {
                    tracing::info!("Loaded {} bundled themes from {}", loaded.len(), bundled_dir.display());
                    for theme in loaded {
                        themes.insert(theme.name.clone(), theme);
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to load bundled themes: {}", e);
                }
            }
        } else {
            tracing::debug!("No bundled themes directory found, using fallback theme");
        }
        
        // Ensure we always have at least the default theme
        if themes.is_empty() {
            tracing::info!("No themes loaded from files, using fallback default theme");
            let default = Theme::fallback();
            themes.insert(default.name.clone(), default);
        }
        
        themes
    }
    
    /// Load user themes from ~/.config/ricecoder/themes/
    pub fn load_user_themes(&self) -> Result<usize> {
        let user_dir = ThemeLoader::user_themes_directory()?;
        if !user_dir.exists() {
            return Ok(0);
        }
        
        let loaded = ThemeLoader::load_from_directory(&user_dir)?;
        let count = loaded.len();
        
        let mut custom = self.custom_themes.write()
            .map_err(|e| anyhow!("Failed to lock custom themes: {}", e))?;
        
        for theme in loaded {
            custom.insert(theme.name.clone(), theme);
        }
        
        tracing::info!("Loaded {} user themes from {}", count, user_dir.display());
        Ok(count)
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
        // Registry should have at least 1 theme (fallback if no JSON files found)
        assert!(registry.builtin_count() >= 1);
        assert_eq!(registry.custom_count().unwrap(), 0);
    }

    #[test]
    fn test_get_builtin_theme() {
        let registry = ThemeRegistry::new();
        // At minimum, should have fallback theme
        let builtin = registry.list_builtin();
        assert!(!builtin.is_empty());
        
        // Get first available theme
        let first = &builtin[0];
        assert!(registry.get(first).is_some());
    }

    #[test]
    fn test_get_nonexistent_theme() {
        let registry = ThemeRegistry::new();
        assert!(registry.get("nonexistent-theme-xyz").is_none());
    }

    #[test]
    fn test_register_custom_theme() {
        let registry = ThemeRegistry::new();
        let mut custom = Theme::fallback();
        custom.name = "my-custom".to_string();

        registry.register(custom).unwrap();
        assert_eq!(registry.custom_count().unwrap(), 1);
        assert!(registry.get("my-custom").is_some());
    }

    #[test]
    fn test_unregister_custom_theme() {
        let registry = ThemeRegistry::new();
        let mut custom = Theme::fallback();
        custom.name = "my-custom".to_string();

        registry.register(custom).unwrap();
        assert_eq!(registry.custom_count().unwrap(), 1);

        registry.unregister("my-custom").unwrap();
        assert_eq!(registry.custom_count().unwrap(), 0);
    }

    #[test]
    fn test_list_all_themes() {
        let registry = ThemeRegistry::new();
        let initial_count = registry.builtin_count();
        
        let mut custom = Theme::fallback();
        custom.name = "my-custom".to_string();
        registry.register(custom).unwrap();

        let all = registry.list_all().unwrap();
        assert_eq!(all.len(), initial_count + 1);
    }

    #[test]
    fn test_list_builtin_themes() {
        let registry = ThemeRegistry::new();
        let builtin = registry.list_builtin();
        assert!(!builtin.is_empty());
    }

    #[test]
    fn test_list_custom_themes() {
        let registry = ThemeRegistry::new();
        let mut custom = Theme::fallback();
        custom.name = "my-custom".to_string();
        registry.register(custom).unwrap();

        let custom_list = registry.list_custom().unwrap();
        assert_eq!(custom_list.len(), 1);
    }

    #[test]
    fn test_exists() {
        let registry = ThemeRegistry::new();
        let builtin = registry.list_builtin();
        if !builtin.is_empty() {
            assert!(registry.exists(&builtin[0]));
        }
        assert!(!registry.exists("nonexistent-xyz"));
    }

    #[test]
    fn test_is_builtin() {
        let registry = ThemeRegistry::new();
        let builtin = registry.list_builtin();
        if !builtin.is_empty() {
            assert!(registry.is_builtin(&builtin[0]));
        }
        assert!(!registry.is_builtin("nonexistent-xyz"));
    }

    #[test]
    fn test_is_custom() {
        let registry = ThemeRegistry::new();
        let mut custom = Theme::fallback();
        custom.name = "my-custom".to_string();
        registry.register(custom).unwrap();

        assert!(registry.is_custom("my-custom").unwrap());
        
        let builtin = registry.list_builtin();
        if !builtin.is_empty() {
            assert!(!registry.is_custom(&builtin[0]).unwrap());
        }
    }

    #[test]
    fn test_reset_to_default() {
        let registry = ThemeRegistry::new();
        let builtin = registry.list_builtin();
        
        if !builtin.is_empty() {
            let theme_name = &builtin[0];
            let mut modified = registry.get(theme_name).unwrap();
            modified.name = theme_name.clone();
            registry.register(modified).unwrap();

            registry.reset_to_default(theme_name).unwrap();
            let theme = registry.get(theme_name).unwrap();
            assert_eq!(theme.name, *theme_name);
        }
    }

    #[test]
    fn test_clear_custom() {
        let registry = ThemeRegistry::new();
        let mut custom = Theme::fallback();
        custom.name = "my-custom".to_string();
        registry.register(custom).unwrap();

        assert_eq!(registry.custom_count().unwrap(), 1);
        registry.clear_custom().unwrap();
        assert_eq!(registry.custom_count().unwrap(), 0);
    }
}
