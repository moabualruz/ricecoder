//! Theme persistence and storage management
//!
//! This module provides storage integration for theme preferences and custom themes.
//! It handles saving and loading theme preferences to/from configuration files,
//! and managing custom theme storage in the `.ricecoder/themes/` directory.

use crate::error::{StorageError, StorageResult};
use crate::manager::PathResolver;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Theme preference configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemePreference {
    /// Current theme name
    pub current_theme: String,
    /// Last updated timestamp
    #[serde(default)]
    pub last_updated: Option<String>,
}

impl Default for ThemePreference {
    fn default() -> Self {
        Self {
            current_theme: "dark".to_string(),
            last_updated: None,
        }
    }
}

/// Theme storage manager for persisting theme preferences and custom themes
pub struct ThemeStorage;

impl ThemeStorage {
    /// Get the themes directory path
    ///
    /// Returns `~/.ricecoder/themes/` or the configured themes directory
    pub fn themes_directory() -> StorageResult<PathBuf> {
        let global_path = PathResolver::resolve_global_path()?;
        let themes_dir = global_path.join("themes");
        Ok(themes_dir)
    }

    /// Get the theme preference file path
    ///
    /// Returns `~/.ricecoder/theme.yaml`
    pub fn preference_file() -> StorageResult<PathBuf> {
        let global_path = PathResolver::resolve_global_path()?;
        let pref_file = global_path.join("theme.yaml");
        Ok(pref_file)
    }

    /// Load theme preference from storage
    ///
    /// Returns the saved theme preference or default if not found
    pub fn load_preference() -> StorageResult<ThemePreference> {
        let pref_file = Self::preference_file()?;

        if !pref_file.exists() {
            return Ok(ThemePreference::default());
        }

        let content = fs::read_to_string(&pref_file).map_err(|e| {
            StorageError::io_error(
                pref_file.clone(),
                crate::error::IoOperation::Read,
                e,
            )
        })?;

        let preference: ThemePreference = serde_yaml::from_str(&content).map_err(|e| {
            StorageError::parse_error(
                pref_file.clone(),
                "yaml",
                e.to_string(),
            )
        })?;

        Ok(preference)
    }

    /// Save theme preference to storage
    ///
    /// Creates the `.ricecoder/` directory if it doesn't exist
    pub fn save_preference(preference: &ThemePreference) -> StorageResult<()> {
        let pref_file = Self::preference_file()?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = pref_file.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                StorageError::directory_creation_failed(
                    parent.to_path_buf(),
                    e,
                )
            })?;
        }

        let content = serde_yaml::to_string(preference).map_err(|e| {
            StorageError::internal(format!("Failed to serialize theme preference: {}", e))
        })?;

        fs::write(&pref_file, content).map_err(|e| {
            StorageError::io_error(
                pref_file.clone(),
                crate::error::IoOperation::Write,
                e,
            )
        })?;

        Ok(())
    }

    /// Save a custom theme file
    ///
    /// Saves the theme YAML content to `~/.ricecoder/themes/{theme_name}.yaml`
    pub fn save_custom_theme(theme_name: &str, content: &str) -> StorageResult<PathBuf> {
        let themes_dir = Self::themes_directory()?;

        // Create themes directory if it doesn't exist
        fs::create_dir_all(&themes_dir).map_err(|e| {
            StorageError::directory_creation_failed(
                themes_dir.clone(),
                e,
            )
        })?;

        let theme_file = themes_dir.join(format!("{}.yaml", theme_name));

        fs::write(&theme_file, content).map_err(|e| {
            StorageError::io_error(
                theme_file.clone(),
                crate::error::IoOperation::Write,
                e,
            )
        })?;

        Ok(theme_file)
    }

    /// Load a custom theme file
    ///
    /// Loads the theme YAML content from `~/.ricecoder/themes/{theme_name}.yaml`
    pub fn load_custom_theme(theme_name: &str) -> StorageResult<String> {
        let themes_dir = Self::themes_directory()?;
        let theme_file = themes_dir.join(format!("{}.yaml", theme_name));

        if !theme_file.exists() {
            return Err(StorageError::validation_error(
                "theme",
                format!("Theme file not found: {}", theme_file.display()),
            ));
        }

        fs::read_to_string(&theme_file).map_err(|e| {
            StorageError::io_error(
                theme_file.clone(),
                crate::error::IoOperation::Read,
                e,
            )
        })
    }

    /// Delete a custom theme file
    pub fn delete_custom_theme(theme_name: &str) -> StorageResult<()> {
        let themes_dir = Self::themes_directory()?;
        let theme_file = themes_dir.join(format!("{}.yaml", theme_name));

        if !theme_file.exists() {
            return Err(StorageError::validation_error(
                "theme",
                format!("Theme file not found: {}", theme_file.display()),
            ));
        }

        fs::remove_file(&theme_file).map_err(|e| {
            StorageError::io_error(
                theme_file.clone(),
                crate::error::IoOperation::Delete,
                e,
            )
        })?;

        Ok(())
    }

    /// List all custom theme files
    pub fn list_custom_themes() -> StorageResult<Vec<String>> {
        let themes_dir = Self::themes_directory()?;

        if !themes_dir.exists() {
            return Ok(Vec::new());
        }

        let mut themes = Vec::new();

        for entry in fs::read_dir(&themes_dir).map_err(|e| {
            StorageError::io_error(
                themes_dir.clone(),
                crate::error::IoOperation::Read,
                e,
            )
        })? {
            let entry = entry.map_err(|e| {
                StorageError::io_error(
                    themes_dir.clone(),
                    crate::error::IoOperation::Read,
                    e,
                )
            })?;

            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "yaml") {
                if let Some(file_stem) = path.file_stem() {
                    if let Some(theme_name) = file_stem.to_str() {
                        themes.push(theme_name.to_string());
                    }
                }
            }
        }

        Ok(themes)
    }

    /// Check if a custom theme exists
    pub fn custom_theme_exists(theme_name: &str) -> StorageResult<bool> {
        let themes_dir = Self::themes_directory()?;
        let theme_file = themes_dir.join(format!("{}.yaml", theme_name));
        Ok(theme_file.exists())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::sync::Mutex;

    // Mutex to prevent parallel test execution from interfering with environment variables
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_theme_preference_default() {
        let pref = ThemePreference::default();
        assert_eq!(pref.current_theme, "dark");
    }

    #[test]
    fn test_save_and_load_preference() {
        let _lock = TEST_LOCK.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let home_path = temp_dir.path().to_string_lossy().to_string();
        std::env::set_var("RICECODER_HOME", &home_path);

        let pref = ThemePreference {
            current_theme: "light".to_string(),
            last_updated: Some("2025-12-09".to_string()),
        };

        ThemeStorage::save_preference(&pref).unwrap();
        let loaded = ThemeStorage::load_preference().unwrap();

        assert_eq!(loaded.current_theme, "light");
        assert_eq!(loaded.last_updated, Some("2025-12-09".to_string()));

        std::env::remove_var("RICECODER_HOME");
    }

    #[test]
    fn test_load_preference_default_when_missing() {
        let _lock = TEST_LOCK.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let home_path = temp_dir.path().to_string_lossy().to_string();
        std::env::set_var("RICECODER_HOME", &home_path);

        let loaded = ThemeStorage::load_preference().unwrap();
        assert_eq!(loaded.current_theme, "dark");

        std::env::remove_var("RICECODER_HOME");
    }

    #[test]
    fn test_save_and_load_custom_theme() {
        let _lock = TEST_LOCK.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let home_path = temp_dir.path().to_string_lossy().to_string();
        std::env::set_var("RICECODER_HOME", &home_path);

        let theme_content = "name: custom\ncolors:\n  background: '#000000'";
        let saved_path = ThemeStorage::save_custom_theme("custom", theme_content).unwrap();

        // Verify the file was actually created
        assert!(saved_path.exists(), "File not found at: {}", saved_path.display());

        // Try to load it back
        let loaded = ThemeStorage::load_custom_theme("custom").unwrap();
        assert_eq!(loaded, theme_content);

        std::env::remove_var("RICECODER_HOME");
    }

    #[test]
    fn test_delete_custom_theme() {
        let _lock = TEST_LOCK.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let home_path = temp_dir.path().to_string_lossy().to_string();
        std::env::set_var("RICECODER_HOME", &home_path);

        let theme_content = "name: custom\ncolors:\n  background: '#000000'";
        ThemeStorage::save_custom_theme("custom", theme_content).unwrap();

        assert!(ThemeStorage::custom_theme_exists("custom").unwrap());

        ThemeStorage::delete_custom_theme("custom").unwrap();

        assert!(!ThemeStorage::custom_theme_exists("custom").unwrap());

        std::env::remove_var("RICECODER_HOME");
    }

    #[test]
    fn test_list_custom_themes() {
        let _lock = TEST_LOCK.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let home_path = temp_dir.path().to_string_lossy().to_string();
        std::env::set_var("RICECODER_HOME", &home_path);

        ThemeStorage::save_custom_theme("theme1", "name: theme1").unwrap();
        ThemeStorage::save_custom_theme("theme2", "name: theme2").unwrap();

        let themes = ThemeStorage::list_custom_themes().unwrap();
        assert_eq!(themes.len(), 2);
        assert!(themes.contains(&"theme1".to_string()));
        assert!(themes.contains(&"theme2".to_string()));

        std::env::remove_var("RICECODER_HOME");
    }

    #[test]
    fn test_custom_theme_exists() {
        let _lock = TEST_LOCK.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let home_path = temp_dir.path().to_string_lossy().to_string();
        std::env::set_var("RICECODER_HOME", &home_path);

        assert!(!ThemeStorage::custom_theme_exists("nonexistent").unwrap());

        ThemeStorage::save_custom_theme("existing", "name: existing").unwrap();
        assert!(ThemeStorage::custom_theme_exists("existing").unwrap());

        std::env::remove_var("RICECODER_HOME");
    }
}
