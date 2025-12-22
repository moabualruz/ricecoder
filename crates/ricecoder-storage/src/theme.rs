//! Theme storage functionality for RiceCoder
//!
//! This module provides storage operations for theme preferences and custom themes.

use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{StorageError, StorageResult};

/// Theme preference data structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemePreference {
    /// Current theme name
    pub current_theme: String,
    /// Last updated timestamp in RFC3339 format
    pub last_updated: Option<String>,
}

/// Theme storage manager for theme preferences and custom themes
pub struct ThemeStorage;

impl ThemeStorage {
    /// Get the theme storage directory
    fn storage_dir() -> StorageResult<PathBuf> {
        let mut dir = dirs::home_dir().ok_or_else(|| {
            StorageError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Home directory not found",
            ))
        })?;
        dir.push(".ricecoder");
        dir.push("themes");
        fs::create_dir_all(&dir).map_err(StorageError::Io)?;
        Ok(dir)
    }

    /// Get the preference file path
    fn preference_path() -> StorageResult<PathBuf> {
        let mut path = Self::storage_dir()?;
        path.push("preference.json");
        Ok(path)
    }

    /// Get the custom themes directory
    fn custom_themes_dir() -> StorageResult<PathBuf> {
        let mut dir = Self::storage_dir()?;
        dir.push("custom");
        fs::create_dir_all(&dir).map_err(StorageError::Io)?;
        Ok(dir)
    }

    /// Load theme preference from storage
    pub fn load_preference() -> StorageResult<ThemePreference> {
        let path = Self::preference_path()?;
        if !path.exists() {
            return Ok(ThemePreference {
                current_theme: "default".to_string(),
                last_updated: None,
            });
        }
        let content = fs::read_to_string(&path).map_err(StorageError::Io)?;
        serde_json::from_str(&content).map_err(|e| {
            StorageError::parse_error(
                path,
                "json",
                format!("Failed to parse theme preference: {}", e),
            )
        })
    }

    /// Save theme preference to storage
    pub fn save_preference(preference: &ThemePreference) -> StorageResult<()> {
        let path = Self::preference_path()?;
        let content = serde_json::to_string_pretty(preference).map_err(|e| {
            StorageError::parse_error(path.clone(), "json", format!("Serialization failed: {}", e))
        })?;
        fs::write(&path, content).map_err(StorageError::Io)?;
        Ok(())
    }
    /// List all custom themes
    pub fn list_custom_themes() -> StorageResult<Vec<String>> {
        let dir = Self::custom_themes_dir()?;
        let mut themes = Vec::new();
        for entry in fs::read_dir(&dir).map_err(StorageError::Io)? {
            let entry = entry.map_err(StorageError::Io)?;
            if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(name) = entry.path().file_stem().and_then(|s| s.to_str()) {
                    themes.push(name.to_string());
                }
            }
        }
        Ok(themes)
    }

    /// Load a custom theme by name
    pub fn load_custom_theme(name: &str) -> StorageResult<String> {
        let mut path = Self::custom_themes_dir()?;
        path.push(format!("{}.json", name));
        fs::read_to_string(&path).map_err(StorageError::Io)
    }

    /// Save a custom theme
    pub fn save_custom_theme(name: &str, content: &str) -> StorageResult<()> {
        let mut path = Self::custom_themes_dir()?;
        path.push(format!("{}.json", name));
        fs::write(&path, content).map_err(StorageError::Io)
    }

    /// Delete a custom theme
    pub fn delete_custom_theme(name: &str) -> StorageResult<()> {
        let mut path = Self::custom_themes_dir()?;
        path.push(format!("{}.json", name));
        if path.exists() {
            fs::remove_file(&path).map_err(StorageError::Io)
        } else {
            Err(StorageError::NotFound(format!(
                "Custom theme '{}' not found",
                name
            )))
        }
    }

    /// Check if a custom theme exists
    pub fn custom_theme_exists(name: &str) -> StorageResult<bool> {
        let mut path = Self::custom_themes_dir()?;
        path.push(format!("{}.json", name));
        Ok(path.exists())
    }
}
