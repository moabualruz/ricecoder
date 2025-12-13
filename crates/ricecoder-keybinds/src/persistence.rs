//! Persistence layer for saving and loading keybind profiles
//!
//! # Storage Location
//!
//! Keybind profiles are stored in `projects/ricecoder/config/keybinds/` with the following structure:
//!
//! ```text
//! projects/ricecoder/config/keybinds/
//! ├── defaults.json          # Default keybinds (read-only)
//! ├── active_profile.txt     # Name of the currently active profile
//! ├── default.json           # Default profile (auto-created)
//! ├── vim.json               # Example custom profile
//! └── emacs.json             # Example custom profile
//! ```
//!
//! # File Format
//!
//! Each profile is stored as a JSON file with the following structure:
//!
//! ```json
//! {
//!   "name": "default",
//!   "keybinds": [
//!     {
//!       "action_id": "editor.save",
//!       "key": "Ctrl+S",
//!       "category": "editing",
//!       "description": "Save current file",
//!       "is_default": true
//!     }
//!   ]
//! }
//! ```
//!
//! The `active_profile.txt` file contains just the name of the active profile:
//!
//! ```text
//! default
//! ```
//!
//! # Usage
//!
//! To use the default storage location:
//!
//! ```no_run
//! use ricecoder_keybinds::{FileSystemPersistence, KeybindPersistence};
//!
//! let persistence = FileSystemPersistence::with_default_location()?;
//! let profiles = persistence.list_profiles()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::PersistenceError;
use crate::profile::Profile;

/// Trait for persisting keybind profiles
pub trait KeybindPersistence: Send + Sync {
    /// Save a profile to storage
    fn save_profile(&self, profile: &Profile) -> Result<(), PersistenceError>;

    /// Load a profile from storage
    fn load_profile(&self, name: &str) -> Result<Profile, PersistenceError>;

    /// Delete a profile from storage
    fn delete_profile(&self, name: &str) -> Result<(), PersistenceError>;

    /// List all saved profiles
    fn list_profiles(&self) -> Result<Vec<String>, PersistenceError>;
}

/// File system based persistence
pub struct FileSystemPersistence {
    config_dir: PathBuf,
}

impl FileSystemPersistence {
    /// Create a new file system persistence with the given config directory
    pub fn new(config_dir: impl AsRef<Path>) -> Result<Self, PersistenceError> {
        let config_dir = config_dir.as_ref().to_path_buf();

        // Create directory if it doesn't exist
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir).map_err(|e| {
                PersistenceError::IoError(std::io::Error::new(
                    e.kind(),
                    format!("Failed to create config directory: {}", e),
                ))
            })?;
        }

        Ok(FileSystemPersistence { config_dir })
    }

    /// Get the path for a profile file
    fn profile_path(&self, name: &str) -> PathBuf {
        self.config_dir.join(format!("{}.json", name))
    }

    /// Get the active profile file path
    fn active_profile_path(&self) -> PathBuf {
        self.config_dir.join("active_profile.txt")
    }
}

impl KeybindPersistence for FileSystemPersistence {
    fn save_profile(&self, profile: &Profile) -> Result<(), PersistenceError> {
        let path = self.profile_path(&profile.name);

        let json = serde_json::to_string_pretty(profile).map_err(|e| {
            PersistenceError::SerializationError(format!("Failed to serialize profile: {}", e))
        })?;

        fs::write(&path, json).map_err(|e| {
            PersistenceError::IoError(std::io::Error::new(
                e.kind(),
                format!("Failed to write profile file: {}", e),
            ))
        })?;

        Ok(())
    }

    fn load_profile(&self, name: &str) -> Result<Profile, PersistenceError> {
        let path = self.profile_path(name);

        if !path.exists() {
            return Err(PersistenceError::ProfileNotFound(name.to_string()));
        }

        let content = fs::read_to_string(&path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                PersistenceError::ProfileNotFound(name.to_string())
            } else if e.kind() == std::io::ErrorKind::PermissionDenied {
                PersistenceError::PermissionDenied(path.to_string_lossy().to_string())
            } else {
                PersistenceError::IoError(e)
            }
        })?;

        let profile: Profile = serde_json::from_str(&content).map_err(|e| {
            PersistenceError::CorruptedJson(format!("Failed to parse profile: {}", e))
        })?;

        Ok(profile)
    }

    fn delete_profile(&self, name: &str) -> Result<(), PersistenceError> {
        let path = self.profile_path(name);

        if !path.exists() {
            return Err(PersistenceError::ProfileNotFound(name.to_string()));
        }

        fs::remove_file(&path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                PersistenceError::PermissionDenied(path.to_string_lossy().to_string())
            } else {
                PersistenceError::IoError(e)
            }
        })?;

        Ok(())
    }

    fn list_profiles(&self) -> Result<Vec<String>, PersistenceError> {
        let mut profiles = Vec::new();

        if !self.config_dir.exists() {
            return Ok(profiles);
        }

        let entries = fs::read_dir(&self.config_dir).map_err(|e| {
            PersistenceError::IoError(std::io::Error::new(
                e.kind(),
                format!("Failed to read config directory: {}", e),
            ))
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                PersistenceError::IoError(std::io::Error::new(
                    e.kind(),
                    format!("Failed to read directory entry: {}", e),
                ))
            })?;

            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "json") {
                if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                    profiles.push(name.to_string());
                }
            }
        }

        profiles.sort();
        Ok(profiles)
    }
}

impl FileSystemPersistence {
    /// Save the active profile name
    pub fn save_active_profile(&self, name: &str) -> Result<(), PersistenceError> {
        let path = self.active_profile_path();
        fs::write(&path, name).map_err(|e| {
            PersistenceError::IoError(std::io::Error::new(
                e.kind(),
                format!("Failed to write active profile: {}", e),
            ))
        })?;
        Ok(())
    }

    /// Load the active profile name
    pub fn load_active_profile(&self) -> Result<Option<String>, PersistenceError> {
        let path = self.active_profile_path();

        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&path).map_err(|e| {
            PersistenceError::IoError(std::io::Error::new(
                e.kind(),
                format!("Failed to read active profile: {}", e),
            ))
        })?;

        Ok(Some(content.trim().to_string()))
    }

    /// Create a new file system persistence with the default storage location
    /// 
    /// The default storage location is `projects/ricecoder/config/keybinds/`
    /// This function will create the directory if it doesn't exist.
    pub fn with_default_location() -> Result<Self, PersistenceError> {
        // Try multiple possible paths to find the config directory
        let possible_paths = vec![
            PathBuf::from("projects/ricecoder/config/keybinds"),
            PathBuf::from("config/keybinds"),
            PathBuf::from("../../config/keybinds"),
            PathBuf::from("../../../config/keybinds"),
            PathBuf::from("../../../../config/keybinds"),
        ];

        for path in possible_paths {
            if let Ok(persistence) = FileSystemPersistence::new(&path) {
                return Ok(persistence);
            }
        }

        // If none of the paths work, try to create the default path
        let default_path = PathBuf::from("projects/ricecoder/config/keybinds");
        FileSystemPersistence::new(&default_path)
    }

    /// Get the config directory path
    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }
}


