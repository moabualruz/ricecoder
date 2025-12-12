//! Core theme types and data structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A complete theme definition
#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    /// Theme name
    pub name: String,
    /// Primary colors
    pub primary: ratatui::style::Color,
    /// Secondary colors
    pub secondary: ratatui::style::Color,
    /// Background colors
    pub background: ratatui::style::Color,
    /// Text colors
    pub foreground: ratatui::style::Color,
    /// Accent colors
    pub accent: ratatui::style::Color,
    /// Error colors
    pub error: ratatui::style::Color,
    /// Warning colors
    pub warning: ratatui::style::Color,
    /// Success colors
    pub success: ratatui::style::Color,
    /// Syntax highlighting theme
    pub syntax: SyntaxTheme,
}

impl Theme {
    /// Validate the theme data
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Theme name cannot be empty".to_string());
        }
        // Add more validation as needed
        Ok(())
    }

    /// Get a built-in theme by name
    pub fn by_name(name: &str) -> Option<Self> {
        match name {
            "dark" => Some(Self::default()),
            "light" => Some(Self::light()),
            _ => None,
        }
    }

    /// Get all available theme names
    pub fn available_themes() -> Vec<&'static str> {
        vec!["dark", "light"]
    }

    /// Create a default dark theme
    pub fn default() -> Self {
        use ratatui::style::Color;
        Self {
            name: "dark".to_string(),
            primary: Color::Rgb(255, 255, 255),
            secondary: Color::Rgb(204, 204, 204),
            background: Color::Rgb(0, 0, 0),
            foreground: Color::Rgb(255, 255, 255),
            accent: Color::Rgb(255, 255, 0),
            error: Color::Rgb(255, 0, 0),
            warning: Color::Rgb(255, 255, 0),
            success: Color::Rgb(0, 255, 0),
            syntax: SyntaxTheme {
                keyword: Color::Rgb(255, 102, 0),
                string: Color::Rgb(0, 255, 0),
                number: Color::Rgb(255, 255, 0),
                comment: Color::Rgb(136, 136, 136),
                function: Color::Rgb(255, 0, 255),
                variable: Color::Rgb(255, 255, 255),
                r#type: Color::Rgb(0, 255, 255),
                constant: Color::Rgb(255, 102, 0),
            },
        }
    }

    /// Create a light theme
    pub fn light() -> Self {
        use ratatui::style::Color;
        Self {
            name: "light".to_string(),
            primary: Color::Rgb(0, 0, 0),
            secondary: Color::Rgb(51, 51, 51),
            background: Color::Rgb(255, 255, 255),
            foreground: Color::Rgb(0, 0, 0),
            accent: Color::Rgb(0, 0, 255),
            error: Color::Rgb(255, 0, 0),
            warning: Color::Rgb(255, 102, 0),
            success: Color::Rgb(0, 170, 0),
            syntax: SyntaxTheme {
                keyword: Color::Rgb(0, 0, 255),
                string: Color::Rgb(0, 128, 0),
                number: Color::Rgb(255, 102, 0),
                comment: Color::Rgb(119, 119, 119),
                function: Color::Rgb(128, 0, 128),
                variable: Color::Rgb(0, 0, 0),
                r#type: Color::Rgb(0, 128, 128),
                constant: Color::Rgb(0, 0, 255),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_default() {
        let theme = Theme::default();
        assert_eq!(theme.name, "dark");
        // Test that colors are set
        assert!(matches!(theme.primary, ratatui::style::Color::Rgb(255, 255, 255)));
    }

    #[test]
    fn test_theme_light() {
        let theme = Theme::light();
        assert_eq!(theme.name, "light");
        assert!(matches!(theme.primary, ratatui::style::Color::Rgb(0, 0, 0)));
    }

    #[test]
    fn test_theme_by_name() {
        assert!(Theme::by_name("dark").is_some());
        assert!(Theme::by_name("light").is_some());
        assert!(Theme::by_name("invalid").is_none());
    }

    #[test]
    fn test_theme_available_themes() {
        let themes = Theme::available_themes();
        assert!(themes.contains(&"dark"));
        assert!(themes.contains(&"light"));
    }

    #[test]
    fn test_theme_validate() {
        let mut theme = Theme::default();
        assert!(theme.validate().is_ok());

        theme.name = "".to_string();
        assert!(theme.validate().is_err());
    }
}

/// Theme manager trait for loading and managing themes
pub trait ThemeManager {
    /// Load a theme by name
    fn load_theme(&mut self, name: &str) -> Result<(), ThemeError>;
    /// Get a theme by name
    fn get_theme(&self, name: &str) -> Option<&Theme>;
    /// List all available themes
    fn list_themes(&self) -> Vec<String>;
}

/// Theme error type
#[derive(Debug, thiserror::Error)]
pub enum ThemeError {
    #[error("Theme not found: {0}")]
    NotFound(String),
    #[error("Invalid theme format: {0}")]
    InvalidFormat(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(String),
}

/// Theme metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemeMetadata {
    /// Theme name
    pub name: String,
    /// Theme author
    pub author: String,
    /// Theme description
    pub description: String,
    /// Theme version
    pub version: String,
    /// Compatible RiceCoder versions
    pub ricecoder_version: String,
}

/// Color palette for the theme
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemeColors {
    /// Primary colors
    pub primary: ColorDefinition,
    /// Secondary colors
    pub secondary: ColorDefinition,
    /// Background colors
    pub background: ColorDefinition,
    /// Text colors
    pub text: ColorDefinition,
    /// Accent colors
    pub accent: ColorDefinition,
    /// Error colors
    pub error: ColorDefinition,
    /// Warning colors
    pub warning: ColorDefinition,
    /// Success colors
    pub success: ColorDefinition,
    /// Info colors
    pub info: ColorDefinition,
    /// Muted colors
    pub muted: ColorDefinition,
    /// Border colors
    pub border: ColorDefinition,
}

/// Syntax highlighting theme
#[derive(Debug, Clone, PartialEq)]
pub struct SyntaxTheme {
    /// Keywords
    pub keyword: ratatui::style::Color,
    /// Strings
    pub string: ratatui::style::Color,
    /// Numbers
    pub number: ratatui::style::Color,
    /// Comments
    pub comment: ratatui::style::Color,
    /// Functions
    pub function: ratatui::style::Color,
    /// Variables
    pub variable: ratatui::style::Color,
    /// Types
    pub r#type: ratatui::style::Color,
    /// Constants
    pub constant: ratatui::style::Color,
}

/// Color definition with foreground, background, and modifiers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ColorDefinition {
    /// Foreground color (hex or named)
    pub fg: Option<String>,
    /// Background color (hex or named)
    pub bg: Option<String>,
    /// Text modifiers
    pub modifiers: Vec<String>,
}

/// Theme configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemeConfig {
    /// Current theme name
    pub current_theme: String,
    /// Custom theme overrides
    pub overrides: HashMap<String, serde_json::Value>,
    /// Theme settings
    pub settings: ThemeSettings,
}

/// Theme settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemeSettings {
    /// Enable high contrast mode
    pub high_contrast: bool,
    /// Enable accessibility improvements
    pub accessibility: bool,
    /// Animation settings
    pub animations: bool,
}