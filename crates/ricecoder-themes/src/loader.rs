//! Custom theme loading from YAML files

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
use ratatui::style::{Color, Color as ColorSupport};
use serde::{Deserialize, Serialize};

use crate::types::{SyntaxTheme, Theme};

/// YAML theme format for custom themes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeYaml {
    /// Theme name
    pub name: String,
    /// Primary color (hex format)
    pub primary: String,
    /// Secondary color (hex format)
    pub secondary: String,
    /// Accent color (hex format)
    pub accent: String,
    /// Background color (hex format)
    pub background: String,
    /// Foreground color (hex format)
    pub foreground: String,
    /// Error color (hex format)
    pub error: String,
    /// Warning color (hex format)
    pub warning: String,
    /// Success color (hex format)
    pub success: String,
}

impl ThemeYaml {
    /// Parse hex color string to ratatui Color
    fn parse_color(hex: &str) -> Result<Color> {
        if hex.starts_with('#') {
            let hex = &hex[1..];
            if hex.len() == 6 {
                if let Ok(rgb) = u32::from_str_radix(hex, 16) {
                    let r = ((rgb >> 16) & 0xff) as u8;
                    let g = ((rgb >> 8) & 0xff) as u8;
                    let b = (rgb & 0xff) as u8;
                    return Ok(Color::Rgb(r, g, b));
                }
            }
        }
        Err(anyhow!("Invalid hex color: {}", hex))
    }

    /// Convert color to hex string
    fn color_to_hex(color: &Color) -> String {
        match color {
            Color::Rgb(r, g, b) => format!("#{:02x}{:02x}{:02x}", r, g, b),
            _ => "#000000".to_string(), // fallback
        }
    }

    /// Convert YAML theme to Theme struct
    pub fn to_theme(&self) -> Result<Theme> {
        Ok(Theme {
            name: self.name.clone(),
            primary: Self::parse_color(&self.primary)?,
            secondary: Self::parse_color(&self.secondary)?,
            accent: Self::parse_color(&self.accent)?,
            background: Self::parse_color(&self.background)?,
            foreground: Self::parse_color(&self.foreground)?,
            error: Self::parse_color(&self.error)?,
            warning: Self::parse_color(&self.warning)?,
            success: Self::parse_color(&self.success)?,
            syntax: SyntaxTheme {
                keyword: Self::parse_color("#ff6600")?,
                string: Self::parse_color("#00ff00")?,
                number: Self::parse_color("#ffff00")?,
                comment: Self::parse_color("#888888")?,
                function: Self::parse_color("#ff00ff")?,
                variable: Self::parse_color("#ffffff")?,
                r#type: Self::parse_color("#00ffff")?,
                constant: Self::parse_color("#ff6600")?,
            },
        })
    }
}

impl From<&Theme> for ThemeYaml {
    fn from(theme: &Theme) -> Self {
        Self {
            name: theme.name.clone(),
            primary: Self::color_to_hex(&theme.primary),
            secondary: Self::color_to_hex(&theme.secondary),
            accent: Self::color_to_hex(&theme.accent),
            background: Self::color_to_hex(&theme.background),
            foreground: Self::color_to_hex(&theme.foreground),
            error: Self::color_to_hex(&theme.error),
            warning: Self::color_to_hex(&theme.warning),
            success: Self::color_to_hex(&theme.success),
        }
    }
}

/// Custom theme loader
pub struct ThemeLoader;

impl ThemeLoader {
    /// Load a theme from a YAML string
    pub fn load_from_string(content: &str) -> Result<Theme> {
        let theme_yaml: ThemeYaml = serde_yaml::from_str(content)?;

        // Validate theme
        Self::validate_theme(&theme_yaml)?;

        theme_yaml.to_theme()
    }

    /// Load a theme from a YAML string and adapt it to terminal capabilities
    pub fn load_from_string_adapted(content: &str, _support: ColorSupport) -> Result<Theme> {
        // let mut theme = Self::load_from_string(content)?;
        // theme.adapt(support); // TODO: implement adapt method
        Self::load_from_string(content)
    }

    /// Load a theme from a YAML file
    pub fn load_from_file(path: &Path) -> Result<Theme> {
        if !path.exists() {
            return Err(anyhow!("Theme file not found: {}", path.display()));
        }

        if !path
            .extension()
            .is_some_and(|ext| ext == "yaml" || ext == "yml")
        {
            return Err(anyhow!("Theme file must be YAML format (.yaml or .yml)"));
        }

        let content = fs::read_to_string(path)?;
        Self::load_from_string(&content)
    }

    /// Load a theme from a YAML file and adapt it to terminal capabilities
    pub fn load_from_file_adapted(path: &Path, _support: ColorSupport) -> Result<Theme> {
        // let mut theme = Self::load_from_file(path)?;
        // theme.adapt(support); // TODO: implement adapt method
        Self::load_from_file(path)
    }

    /// Save a theme to a YAML file
    pub fn save_to_file(theme: &Theme, path: &Path) -> Result<()> {
        let theme_yaml = ThemeYaml::from(theme);
        let content = serde_yaml::to_string(&theme_yaml)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Load all themes from a directory
    pub fn load_from_directory(dir: &Path) -> Result<Vec<Theme>> {
        if !dir.exists() {
            return Ok(Vec::new());
        }

        if !dir.is_dir() {
            return Err(anyhow!("Path is not a directory: {}", dir.display()));
        }

        let mut themes = Vec::new();

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file()
                && (path
                    .extension()
                    .is_some_and(|ext| ext == "yaml" || ext == "yml"))
            {
                match Self::load_from_file(&path) {
                    Ok(theme) => themes.push(theme),
                    Err(e) => {
                        tracing::warn!("Failed to load theme from {}: {}", path.display(), e);
                    }
                }
            }
        }

        Ok(themes)
    }

    /// Get the default themes directory
    pub fn themes_directory() -> Result<PathBuf> {
        let config_dir =
            dirs::config_dir().ok_or_else(|| anyhow!("Could not determine config directory"))?;
        Ok(config_dir.join("ricecoder").join("themes"))
    }

    /// Validate a theme YAML
    fn validate_theme(theme: &ThemeYaml) -> Result<()> {
        if theme.name.is_empty() {
            return Err(anyhow!("Theme name cannot be empty"));
        }

        // Validate all colors are valid hex
        let colors = vec![
            ("primary", &theme.primary),
            ("secondary", &theme.secondary),
            ("accent", &theme.accent),
            ("background", &theme.background),
            ("foreground", &theme.foreground),
            ("error", &theme.error),
            ("warning", &theme.warning),
            ("success", &theme.success),
        ];

        for (name, color) in colors {
            if ThemeYaml::parse_color(color).is_err() {
                return Err(anyhow!("Invalid {} color: {}", name, color));
            }
        }

        Ok(())
    }
}
