//! Theme loader for RiceCoder
//!
//! Loads theme definitions from `config/themes/*.json` files.
//! Each theme file contains color definitions and semantic color mappings.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{StorageError, StorageResult};

/// Color value that can be a reference to a def or a direct hex color
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ColorValue {
    /// Reference to a color definition (e.g., "darkStep1")
    Reference(String),
}

/// Dark/light color pair for a theme element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColorPair {
    /// Color for dark mode (can be a def reference or hex)
    pub dark: String,
    /// Color for light mode (can be a def reference or hex)
    pub light: String,
}

/// Theme definition loaded from a JSON file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    /// Theme name (derived from filename)
    #[serde(skip)]
    pub name: String,
    /// Optional schema reference
    #[serde(rename = "$schema", default)]
    pub schema: Option<String>,
    /// Color definitions (palette)
    #[serde(default)]
    pub defs: HashMap<String, String>,
    /// Theme color mappings
    #[serde(default)]
    pub theme: HashMap<String, ThemeColorPair>,
}

impl Theme {
    /// Resolve a color reference to its hex value
    pub fn resolve_color(&self, reference: &str, is_dark: bool) -> Option<String> {
        // Check if it's a direct hex color
        if reference.starts_with('#') {
            return Some(reference.to_string());
        }

        // Look up in defs
        self.defs.get(reference).cloned()
    }

    /// Get the resolved color for a theme element
    pub fn get_color(&self, element: &str, is_dark: bool) -> Option<String> {
        let pair = self.theme.get(element)?;
        let reference = if is_dark { &pair.dark } else { &pair.light };
        self.resolve_color(reference, is_dark)
    }
}

/// Loader for theme configuration files
pub struct ThemeLoader {
    config_dir: PathBuf,
}

impl ThemeLoader {
    /// Create a new theme loader with the given config directory
    pub fn new(config_dir: PathBuf) -> Self {
        Self { config_dir }
    }

    /// Create a theme loader using the default config path
    pub fn with_default_path() -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let config_dir = cwd.join("config").join("themes");
        Self::new(config_dir)
    }

    /// List all available theme names
    pub fn list_themes(&self) -> StorageResult<Vec<String>> {
        let mut themes = Vec::new();

        if !self.config_dir.exists() {
            return Ok(themes);
        }

        let entries = fs::read_dir(&self.config_dir).map_err(|e| {
            StorageError::io_error(
                self.config_dir.clone(),
                crate::error::IoOperation::Read,
                e,
            )
        })?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    themes.push(name.to_string());
                }
            }
        }

        themes.sort();
        Ok(themes)
    }

    /// Load all themes from the config directory
    pub fn load_all(&self) -> StorageResult<HashMap<String, Theme>> {
        let mut themes = HashMap::new();

        if !self.config_dir.exists() {
            return Ok(themes);
        }

        let entries = fs::read_dir(&self.config_dir).map_err(|e| {
            StorageError::io_error(
                self.config_dir.clone(),
                crate::error::IoOperation::Read,
                e,
            )
        })?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                if let Ok(theme) = self.load_from_file(&path) {
                    themes.insert(theme.name.clone(), theme);
                }
            }
        }

        Ok(themes)
    }

    /// Load a single theme from a file
    pub fn load_from_file(&self, path: &Path) -> StorageResult<Theme> {
        let content = fs::read_to_string(path).map_err(|e| {
            StorageError::io_error(path.to_path_buf(), crate::error::IoOperation::Read, e)
        })?;

        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let mut theme: Theme = serde_json::from_str(&content).map_err(|e| {
            StorageError::parse_error(path.to_path_buf(), "JSON", e.to_string())
        })?;

        theme.name = name;
        Ok(theme)
    }

    /// Load a specific theme by name
    pub fn load(&self, name: &str) -> StorageResult<Theme> {
        let path = self.config_dir.join(format!("{}.json", name));
        self.load_from_file(&path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_theme() {
        let content = r##"{
            "defs": {
                "primary": "#ff0000",
                "secondary": "#00ff00"
            },
            "theme": {
                "background": {
                    "dark": "primary",
                    "light": "secondary"
                }
            }
        }"##;

        let mut theme: Theme = serde_json::from_str(content).unwrap();
        theme.name = "test".to_string();

        assert_eq!(theme.defs.get("primary"), Some(&"#ff0000".to_string()));
        assert_eq!(theme.get_color("background", true), Some("#ff0000".to_string()));
        assert_eq!(theme.get_color("background", false), Some("#00ff00".to_string()));
    }
}
