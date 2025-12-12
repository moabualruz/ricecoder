//! Custom theme loading from YAML files

use crate::style::{Color, Theme, ColorSupport};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

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
    /// Convert YAML theme to Theme struct
    pub fn to_theme(&self) -> Result<Theme> {
        Ok(Theme {
            name: self.name.clone(),
            primary: Color::from_hex(&self.primary)
                .ok_or_else(|| anyhow!("Invalid primary color: {}", self.primary))?,
            secondary: Color::from_hex(&self.secondary)
                .ok_or_else(|| anyhow!("Invalid secondary color: {}", self.secondary))?,
            accent: Color::from_hex(&self.accent)
                .ok_or_else(|| anyhow!("Invalid accent color: {}", self.accent))?,
            background: Color::from_hex(&self.background)
                .ok_or_else(|| anyhow!("Invalid background color: {}", self.background))?,
            foreground: Color::from_hex(&self.foreground)
                .ok_or_else(|| anyhow!("Invalid foreground color: {}", self.foreground))?,
            error: Color::from_hex(&self.error)
                .ok_or_else(|| anyhow!("Invalid error color: {}", self.error))?,
            warning: Color::from_hex(&self.warning)
                .ok_or_else(|| anyhow!("Invalid warning color: {}", self.warning))?,
            success: Color::from_hex(&self.success)
                .ok_or_else(|| anyhow!("Invalid success color: {}", self.success))?,
        })
    }
}

impl From<&Theme> for ThemeYaml {
    fn from(theme: &Theme) -> Self {
        Self {
            name: theme.name.clone(),
            primary: theme.primary.to_hex(),
            secondary: theme.secondary.to_hex(),
            accent: theme.accent.to_hex(),
            background: theme.background.to_hex(),
            foreground: theme.foreground.to_hex(),
            error: theme.error.to_hex(),
            warning: theme.warning.to_hex(),
            success: theme.success.to_hex(),
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
    pub fn load_from_string_adapted(content: &str, support: ColorSupport) -> Result<Theme> {
        let mut theme = Self::load_from_string(content)?;
        theme.adapt(support);
        Ok(theme)
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
    pub fn load_from_file_adapted(path: &Path, support: ColorSupport) -> Result<Theme> {
        let mut theme = Self::load_from_file(path)?;
        theme.adapt(support);
        Ok(theme)
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
            if Color::from_hex(color).is_none() {
                return Err(anyhow!("Invalid {} color: {}", name, color));
            }
        }

        Ok(())
    }
}

