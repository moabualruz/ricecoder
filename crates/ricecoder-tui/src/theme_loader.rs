//! Custom theme loading from YAML files

use crate::style::{Theme, Color};
use anyhow::{Result, anyhow};
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
    /// Load a theme from a YAML file
    pub fn load_from_file(path: &Path) -> Result<Theme> {
        if !path.exists() {
            return Err(anyhow!("Theme file not found: {}", path.display()));
        }

        if !path.extension().map_or(false, |ext| ext == "yaml" || ext == "yml") {
            return Err(anyhow!("Theme file must be YAML format (.yaml or .yml)"));
        }

        let content = fs::read_to_string(path)?;
        let theme_yaml: ThemeYaml = serde_yaml::from_str(&content)?;
        
        // Validate theme
        Self::validate_theme(&theme_yaml)?;
        
        theme_yaml.to_theme()
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

            if path.is_file() && (path.extension().map_or(false, |ext| ext == "yaml" || ext == "yml")) {
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
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow!("Could not determine config directory"))?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_theme_yaml_to_theme() {
        let theme_yaml = ThemeYaml {
            name: "test".to_string(),
            primary: "#0078ff".to_string(),
            secondary: "#5ac8fa".to_string(),
            accent: "#ff2d55".to_string(),
            background: "#111827".to_string(),
            foreground: "#f3f4f6".to_string(),
            error: "#ef4444".to_string(),
            warning: "#f59e0b".to_string(),
            success: "#22c55e".to_string(),
        };

        let theme = theme_yaml.to_theme().unwrap();
        assert_eq!(theme.name, "test");
        assert_eq!(theme.primary.r, 0);
        assert_eq!(theme.primary.g, 120);
        assert_eq!(theme.primary.b, 255);
    }

    #[test]
    fn test_theme_to_yaml() {
        let theme = Theme::default();
        let yaml = ThemeYaml::from(&theme);
        assert_eq!(yaml.name, theme.name);
        assert_eq!(yaml.primary, theme.primary.to_hex());
    }

    #[test]
    fn test_save_and_load_theme() {
        let temp_dir = TempDir::new().unwrap();
        let theme_path = temp_dir.path().join("test_theme.yaml");

        let theme = Theme::light();
        ThemeLoader::save_to_file(&theme, &theme_path).unwrap();

        let loaded = ThemeLoader::load_from_file(&theme_path).unwrap();
        assert_eq!(loaded.name, theme.name);
        assert_eq!(loaded.primary, theme.primary);
    }

    #[test]
    fn test_load_from_directory() {
        let temp_dir = TempDir::new().unwrap();

        // Save multiple themes
        ThemeLoader::save_to_file(&Theme::default(), &temp_dir.path().join("dark.yaml")).unwrap();
        ThemeLoader::save_to_file(&Theme::light(), &temp_dir.path().join("light.yaml")).unwrap();

        let themes = ThemeLoader::load_from_directory(temp_dir.path()).unwrap();
        assert_eq!(themes.len(), 2);
    }

    #[test]
    fn test_validate_theme_invalid_color() {
        let theme_yaml = ThemeYaml {
            name: "test".to_string(),
            primary: "invalid".to_string(),
            secondary: "#5ac8fa".to_string(),
            accent: "#ff2d55".to_string(),
            background: "#111827".to_string(),
            foreground: "#f3f4f6".to_string(),
            error: "#ef4444".to_string(),
            warning: "#f59e0b".to_string(),
            success: "#22c55e".to_string(),
        };

        assert!(ThemeLoader::validate_theme(&theme_yaml).is_err());
    }

    #[test]
    fn test_validate_theme_empty_name() {
        let theme_yaml = ThemeYaml {
            name: "".to_string(),
            primary: "#0078ff".to_string(),
            secondary: "#5ac8fa".to_string(),
            accent: "#ff2d55".to_string(),
            background: "#111827".to_string(),
            foreground: "#f3f4f6".to_string(),
            error: "#ef4444".to_string(),
            warning: "#f59e0b".to_string(),
            success: "#22c55e".to_string(),
        };

        assert!(ThemeLoader::validate_theme(&theme_yaml).is_err());
    }
}
