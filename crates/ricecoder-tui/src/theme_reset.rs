//! Theme reset manager for resetting themes to defaults

use crate::style::{Color, Theme};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;

/// Theme reset manager for resetting themes to their default state
pub struct ThemeResetManager {
    /// Default built-in themes for reference
    default_themes: Arc<HashMap<String, Theme>>,
}

impl ThemeResetManager {
    /// Create a new theme reset manager
    pub fn new() -> Self {
        let mut defaults = HashMap::new();

        // Store all built-in themes as defaults
        defaults.insert("dark".to_string(), Theme::default());
        defaults.insert("light".to_string(), Theme::light());
        defaults.insert("monokai".to_string(), Theme::monokai());
        defaults.insert("dracula".to_string(), Theme::dracula());
        defaults.insert("nord".to_string(), Theme::nord());
        defaults.insert("high-contrast".to_string(), Theme::high_contrast());

        Self {
            default_themes: Arc::new(defaults),
        }
    }

    /// Reset all colors in a theme to their default values
    ///
    /// # Arguments
    ///
    /// * `theme` - The theme to reset
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If reset was successful
    /// * `Err` - If the theme is not a built-in theme or reset failed
    pub fn reset_colors(&self, theme: &mut Theme) -> Result<()> {
        let default = self
            .default_themes
            .get(&theme.name)
            .ok_or_else(|| anyhow!("Theme '{}' is not a built-in theme", theme.name))?;

        // Reset all color fields to defaults
        theme.primary = default.primary;
        theme.secondary = default.secondary;
        theme.accent = default.accent;
        theme.background = default.background;
        theme.foreground = default.foreground;
        theme.error = default.error;
        theme.warning = default.warning;
        theme.success = default.success;

        Ok(())
    }

    /// Reset an entire theme to its built-in default
    ///
    /// # Arguments
    ///
    /// * `theme` - The theme to reset
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If reset was successful
    /// * `Err` - If the theme is not a built-in theme or reset failed
    pub fn reset_theme(&self, theme: &mut Theme) -> Result<()> {
        let default = self
            .default_themes
            .get(&theme.name)
            .ok_or_else(|| anyhow!("Theme '{}' is not a built-in theme", theme.name))?;

        // Replace entire theme with default
        *theme = default.clone();

        Ok(())
    }

    /// Get the default color value for a specific color field in a theme
    ///
    /// # Arguments
    ///
    /// * `theme_name` - The name of the theme
    /// * `color_name` - The name of the color field (e.g., "primary", "error", "background")
    ///
    /// # Returns
    ///
    /// * `Ok(Color)` - The default color value
    /// * `Err` - If the theme or color field is not found
    pub fn get_default_color(&self, theme_name: &str, color_name: &str) -> Result<Color> {
        let theme = self
            .default_themes
            .get(theme_name)
            .ok_or_else(|| anyhow!("Theme '{}' is not a built-in theme", theme_name))?;

        match color_name.to_lowercase().as_str() {
            "primary" => Ok(theme.primary),
            "secondary" => Ok(theme.secondary),
            "accent" => Ok(theme.accent),
            "background" => Ok(theme.background),
            "foreground" => Ok(theme.foreground),
            "error" => Ok(theme.error),
            "warning" => Ok(theme.warning),
            "success" => Ok(theme.success),
            _ => Err(anyhow!("Unknown color field: {}", color_name)),
        }
    }

    /// Reset a specific color field in a theme to its default value
    ///
    /// # Arguments
    ///
    /// * `theme` - The theme to modify
    /// * `color_name` - The name of the color field to reset
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If reset was successful
    /// * `Err` - If the theme or color field is not found
    pub fn reset_color(&self, theme: &mut Theme, color_name: &str) -> Result<()> {
        let default_color = self.get_default_color(&theme.name, color_name)?;

        match color_name.to_lowercase().as_str() {
            "primary" => theme.primary = default_color,
            "secondary" => theme.secondary = default_color,
            "accent" => theme.accent = default_color,
            "background" => theme.background = default_color,
            "foreground" => theme.foreground = default_color,
            "error" => theme.error = default_color,
            "warning" => theme.warning = default_color,
            "success" => theme.success = default_color,
            _ => return Err(anyhow!("Unknown color field: {}", color_name)),
        }

        Ok(())
    }

    /// Check if a theme is a built-in theme
    ///
    /// # Arguments
    ///
    /// * `theme_name` - The name of the theme
    ///
    /// # Returns
    ///
    /// * `true` - If the theme is a built-in theme
    /// * `false` - Otherwise
    pub fn is_builtin_theme(&self, theme_name: &str) -> bool {
        self.default_themes.contains_key(theme_name)
    }

    /// Get all built-in theme names
    ///
    /// # Returns
    ///
    /// A vector of built-in theme names
    pub fn builtin_theme_names(&self) -> Vec<String> {
        let mut names: Vec<_> = self.default_themes.keys().cloned().collect();
        names.sort();
        names
    }

    /// Get a copy of a built-in theme by name
    ///
    /// # Arguments
    ///
    /// * `theme_name` - The name of the theme
    ///
    /// # Returns
    ///
    /// * `Ok(Theme)` - A copy of the built-in theme
    /// * `Err` - If the theme is not found
    pub fn get_builtin_theme(&self, theme_name: &str) -> Result<Theme> {
        self.default_themes
            .get(theme_name)
            .cloned()
            .ok_or_else(|| anyhow!("Theme '{}' is not a built-in theme", theme_name))
    }
}

impl Default for ThemeResetManager {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ThemeResetManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ThemeResetManager")
            .field("builtin_themes", &self.builtin_theme_names())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reset_manager_creation() {
        let manager = ThemeResetManager::new();
        assert_eq!(manager.builtin_theme_names().len(), 6);
    }

    #[test]
    fn test_reset_colors() {
        let manager = ThemeResetManager::new();
        let mut theme = Theme::default();

        // Modify colors
        theme.primary = Color::new(255, 0, 0);
        theme.error = Color::new(0, 255, 0);

        // Reset colors
        manager.reset_colors(&mut theme).unwrap();

        // Verify colors are reset to defaults
        let default = Theme::default();
        assert_eq!(theme.primary, default.primary);
        assert_eq!(theme.error, default.error);
    }

    #[test]
    fn test_reset_theme() {
        let manager = ThemeResetManager::new();
        let mut theme = Theme::light();

        // Modify theme
        theme.primary = Color::new(255, 0, 0);
        theme.background = Color::new(100, 100, 100);

        // Reset theme
        manager.reset_theme(&mut theme).unwrap();

        // Verify theme is reset to defaults
        let default = Theme::light();
        assert_eq!(theme.primary, default.primary);
        assert_eq!(theme.secondary, default.secondary);
        assert_eq!(theme.accent, default.accent);
        assert_eq!(theme.background, default.background);
        assert_eq!(theme.foreground, default.foreground);
        assert_eq!(theme.error, default.error);
        assert_eq!(theme.warning, default.warning);
        assert_eq!(theme.success, default.success);
    }

    #[test]
    fn test_reset_theme_preserves_name() {
        let manager = ThemeResetManager::new();
        let mut theme = Theme::monokai();
        let original_name = theme.name.clone();

        // Modify theme
        theme.primary = Color::new(255, 0, 0);

        // Reset theme
        manager.reset_theme(&mut theme).unwrap();

        // Verify name is preserved
        assert_eq!(theme.name, original_name);
    }

    #[test]
    fn test_get_default_color() {
        let manager = ThemeResetManager::new();
        let default_primary = manager.get_default_color("dark", "primary").unwrap();
        let theme = Theme::default();
        assert_eq!(default_primary, theme.primary);
    }

    #[test]
    fn test_get_default_color_all_fields() {
        let manager = ThemeResetManager::new();
        let theme = Theme::dracula();

        assert_eq!(
            manager.get_default_color("dracula", "primary").unwrap(),
            theme.primary
        );
        assert_eq!(
            manager.get_default_color("dracula", "secondary").unwrap(),
            theme.secondary
        );
        assert_eq!(
            manager.get_default_color("dracula", "accent").unwrap(),
            theme.accent
        );
        assert_eq!(
            manager.get_default_color("dracula", "background").unwrap(),
            theme.background
        );
        assert_eq!(
            manager.get_default_color("dracula", "foreground").unwrap(),
            theme.foreground
        );
        assert_eq!(
            manager.get_default_color("dracula", "error").unwrap(),
            theme.error
        );
        assert_eq!(
            manager.get_default_color("dracula", "warning").unwrap(),
            theme.warning
        );
        assert_eq!(
            manager.get_default_color("dracula", "success").unwrap(),
            theme.success
        );
    }

    #[test]
    fn test_get_default_color_case_insensitive() {
        let manager = ThemeResetManager::new();
        let color1 = manager.get_default_color("dark", "primary").unwrap();
        let color2 = manager.get_default_color("dark", "PRIMARY").unwrap();
        let color3 = manager.get_default_color("dark", "Primary").unwrap();

        assert_eq!(color1, color2);
        assert_eq!(color2, color3);
    }

    #[test]
    fn test_get_default_color_invalid_theme() {
        let manager = ThemeResetManager::new();
        assert!(manager.get_default_color("invalid", "primary").is_err());
    }

    #[test]
    fn test_get_default_color_invalid_field() {
        let manager = ThemeResetManager::new();
        assert!(manager.get_default_color("dark", "invalid").is_err());
    }

    #[test]
    fn test_reset_color() {
        let manager = ThemeResetManager::new();
        let mut theme = Theme::nord();
        let default_primary = manager.get_default_color("nord", "primary").unwrap();

        // Modify primary color
        theme.primary = Color::new(255, 0, 0);
        assert_ne!(theme.primary, default_primary);

        // Reset primary color
        manager.reset_color(&mut theme, "primary").unwrap();

        // Verify primary color is reset
        assert_eq!(theme.primary, default_primary);

        // Verify other colors are unchanged
        let default_error = manager.get_default_color("nord", "error").unwrap();
        assert_eq!(theme.error, default_error);
    }

    #[test]
    fn test_reset_color_invalid_field() {
        let manager = ThemeResetManager::new();
        let mut theme = Theme::default();
        assert!(manager.reset_color(&mut theme, "invalid").is_err());
    }

    #[test]
    fn test_is_builtin_theme() {
        let manager = ThemeResetManager::new();
        assert!(manager.is_builtin_theme("dark"));
        assert!(manager.is_builtin_theme("light"));
        assert!(manager.is_builtin_theme("monokai"));
        assert!(manager.is_builtin_theme("dracula"));
        assert!(manager.is_builtin_theme("nord"));
        assert!(manager.is_builtin_theme("high-contrast"));
        assert!(!manager.is_builtin_theme("custom"));
        assert!(!manager.is_builtin_theme("invalid"));
    }

    #[test]
    fn test_builtin_theme_names() {
        let manager = ThemeResetManager::new();
        let names = manager.builtin_theme_names();
        assert_eq!(names.len(), 6);
        assert!(names.contains(&"dark".to_string()));
        assert!(names.contains(&"light".to_string()));
        assert!(names.contains(&"monokai".to_string()));
        assert!(names.contains(&"dracula".to_string()));
        assert!(names.contains(&"nord".to_string()));
        assert!(names.contains(&"high-contrast".to_string()));
    }

    #[test]
    fn test_get_builtin_theme() {
        let manager = ThemeResetManager::new();
        let theme = manager.get_builtin_theme("dark").unwrap();
        assert_eq!(theme.name, "dark");
        let default = Theme::default();
        assert_eq!(theme.primary, default.primary);
        assert_eq!(theme.secondary, default.secondary);
    }

    #[test]
    fn test_get_builtin_theme_invalid() {
        let manager = ThemeResetManager::new();
        assert!(manager.get_builtin_theme("invalid").is_err());
    }

    #[test]
    fn test_reset_colors_invalid_theme() {
        let manager = ThemeResetManager::new();
        let mut theme = Theme::light();
        theme.name = "custom".to_string();

        // Should fail because "custom" is not a built-in theme
        assert!(manager.reset_colors(&mut theme).is_err());
    }

    #[test]
    fn test_reset_theme_invalid_theme() {
        let manager = ThemeResetManager::new();
        let mut theme = Theme::light();
        theme.name = "custom".to_string();

        // Should fail because "custom" is not a built-in theme
        assert!(manager.reset_theme(&mut theme).is_err());
    }

    #[test]
    fn test_reset_all_themes() {
        let manager = ThemeResetManager::new();

        for theme_name in manager.builtin_theme_names() {
            let mut theme = manager.get_builtin_theme(&theme_name).unwrap();

            // Modify all colors
            theme.primary = Color::new(255, 0, 0);
            theme.secondary = Color::new(0, 255, 0);
            theme.accent = Color::new(0, 0, 255);
            theme.background = Color::new(100, 100, 100);
            theme.foreground = Color::new(200, 200, 200);
            theme.error = Color::new(255, 128, 128);
            theme.warning = Color::new(255, 255, 128);
            theme.success = Color::new(128, 255, 128);

            // Reset colors
            manager.reset_colors(&mut theme).unwrap();

            // Verify all colors are reset
            let default = manager.get_builtin_theme(&theme_name).unwrap();
            assert_eq!(theme.primary, default.primary);
            assert_eq!(theme.secondary, default.secondary);
            assert_eq!(theme.accent, default.accent);
            assert_eq!(theme.background, default.background);
            assert_eq!(theme.foreground, default.foreground);
            assert_eq!(theme.error, default.error);
            assert_eq!(theme.warning, default.warning);
            assert_eq!(theme.success, default.success);
        }
    }
}
