//! High contrast theme management for accessibility
//!
//! Provides high contrast themes for users with visual impairments,
//! including dark, light, and specialized color schemes.

use std::collections::HashMap;

/// High contrast theme manager
#[derive(Debug, Clone)]
pub struct HighContrastThemeManager {
    /// Available high contrast themes
    themes: HashMap<String, HighContrastTheme>,
    /// Current theme name
    current_theme: String,
}

/// High contrast theme definition
#[derive(Debug, Clone)]
pub struct HighContrastTheme {
    pub name: String,
    pub primary: Color,
    pub secondary: Color,
    pub background: Color,
    pub foreground: Color,
    pub accent: Color,
    pub error: Color,
    pub warning: Color,
    pub success: Color,
    pub info: Color,
    pub muted: Color,
    pub border: Color,
}

/// Simple RGB color for accessibility themes
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

impl HighContrastThemeManager {
    pub fn new() -> Self {
        let mut themes = HashMap::new();

        themes.insert(
            "high-contrast-dark".to_string(),
            Self::create_dark_high_contrast_theme(),
        );
        themes.insert(
            "high-contrast-light".to_string(),
            Self::create_light_high_contrast_theme(),
        );
        themes.insert(
            "high-contrast-yellow-blue".to_string(),
            Self::create_yellow_blue_theme(),
        );

        Self {
            themes,
            current_theme: "high-contrast-dark".to_string(),
        }
    }

    /// Get current high contrast theme
    pub fn current_theme(&self) -> Option<&HighContrastTheme> {
        self.themes.get(&self.current_theme)
    }

    /// Set current theme
    pub fn set_theme(&mut self, theme_name: &str) -> bool {
        if self.themes.contains_key(theme_name) {
            self.current_theme = theme_name.to_string();
            true
        } else {
            false
        }
    }

    /// Get available theme names
    pub fn available_themes(&self) -> Vec<String> {
        self.themes.keys().cloned().collect()
    }

    /// Create dark high contrast theme
    fn create_dark_high_contrast_theme() -> HighContrastTheme {
        HighContrastTheme {
            name: "High Contrast Dark".to_string(),
            primary: Color::new(255, 255, 255),
            secondary: Color::new(255, 255, 255),
            background: Color::new(0, 0, 0),
            foreground: Color::new(255, 255, 255),
            accent: Color::new(255, 255, 255),
            error: Color::new(255, 0, 0),
            warning: Color::new(255, 255, 0),
            success: Color::new(0, 255, 0),
            info: Color::new(0, 0, 255),
            muted: Color::new(128, 128, 128),
            border: Color::new(255, 255, 255),
        }
    }

    /// Create light high contrast theme
    fn create_light_high_contrast_theme() -> HighContrastTheme {
        HighContrastTheme {
            name: "High Contrast Light".to_string(),
            primary: Color::new(0, 0, 0),
            secondary: Color::new(0, 0, 0),
            background: Color::new(255, 255, 255),
            foreground: Color::new(0, 0, 0),
            accent: Color::new(0, 0, 0),
            error: Color::new(255, 0, 0),
            warning: Color::new(128, 128, 0),
            success: Color::new(0, 128, 0),
            info: Color::new(0, 0, 128),
            muted: Color::new(64, 64, 64),
            border: Color::new(0, 0, 0),
        }
    }

    /// Create yellow-on-blue high contrast theme
    fn create_yellow_blue_theme() -> HighContrastTheme {
        HighContrastTheme {
            name: "Yellow on Blue".to_string(),
            primary: Color::new(255, 255, 0),
            secondary: Color::new(255, 255, 0),
            background: Color::new(0, 0, 255),
            foreground: Color::new(255, 255, 0),
            accent: Color::new(255, 255, 255),
            error: Color::new(255, 0, 0),
            warning: Color::new(255, 255, 255),
            success: Color::new(0, 255, 0),
            info: Color::new(0, 255, 255),
            muted: Color::new(0, 255, 255),
            border: Color::new(255, 255, 0),
        }
    }
}

impl Default for HighContrastThemeManager {
    fn default() -> Self {
        Self::new()
    }
}
