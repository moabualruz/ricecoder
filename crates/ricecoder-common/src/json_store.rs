//! JSON persistence utilities
//!
//! Common patterns for loading and saving JSON configuration files,
//! replacing duplicate implementations across 4+ crates.

use serde::{de::DeserializeOwned, Serialize};
use std::path::Path;
use thiserror::Error;

/// JSON store errors
#[derive(Debug, Error)]
pub enum JsonStoreError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Serialize(#[from] serde_json::Error),

    #[error("File not found: {path}")]
    NotFound { path: String },
}

/// Result type for JSON store operations
pub type JsonStoreResult<T> = Result<T, JsonStoreError>;

/// Load JSON from a file path
pub fn load_json<T, P>(path: P) -> JsonStoreResult<T>
where
    T: DeserializeOwned,
    P: AsRef<Path>,
{
    let path = path.as_ref();
    if !path.exists() {
        return Err(JsonStoreError::NotFound {
            path: path.display().to_string(),
        });
    }
    let content = std::fs::read_to_string(path)?;
    let value = serde_json::from_str(&content)?;
    Ok(value)
}

/// Load JSON from file, returning default if file doesn't exist
pub fn load_json_or_default<T, P>(path: P) -> JsonStoreResult<T>
where
    T: DeserializeOwned + Default,
    P: AsRef<Path>,
{
    match load_json(path) {
        Ok(value) => Ok(value),
        Err(JsonStoreError::NotFound { .. }) => Ok(T::default()),
        Err(e) => Err(e),
    }
}

/// Save value as JSON to a file path
pub fn save_json<T, P>(path: P, value: &T) -> JsonStoreResult<()>
where
    T: Serialize,
    P: AsRef<Path>,
{
    let path = path.as_ref();

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let content = serde_json::to_string_pretty(value)?;
    std::fs::write(path, content)?;
    Ok(())
}

/// Save value as JSON atomically (write to temp, then rename)
pub fn save_json_atomic<T, P>(path: P, value: &T) -> JsonStoreResult<()>
where
    T: Serialize,
    P: AsRef<Path>,
{
    let path = path.as_ref();

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let content = serde_json::to_string_pretty(value)?;

    // Write to temp file first
    let temp_path = path.with_extension("tmp");
    std::fs::write(&temp_path, &content)?;

    // Rename atomically
    std::fs::rename(&temp_path, path)?;

    Ok(())
}

/// Trait for types that can be persisted as JSON
pub trait JsonPersistent: Serialize + DeserializeOwned + Default {
    /// Default file path for this type
    fn default_path() -> &'static str;

    /// Load from default path
    fn load_default() -> JsonStoreResult<Self> {
        load_json_or_default(Self::default_path())
    }

    /// Save to default path
    fn save_default(&self) -> JsonStoreResult<()> {
        save_json(Self::default_path(), self)
    }

    /// Load from custom path
    fn load_from<P: AsRef<Path>>(path: P) -> JsonStoreResult<Self> {
        load_json_or_default(path)
    }

    /// Save to custom path
    fn save_to<P: AsRef<Path>>(&self, path: P) -> JsonStoreResult<()> {
        save_json(path, self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::fs;
    use tempfile::tempdir;

    #[derive(Debug, Serialize, Deserialize, Default, PartialEq)]
    struct TestConfig {
        name: String,
        count: i32,
    }

    #[test]
    fn test_load_save_json() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.json");

        let config = TestConfig {
            name: "test".to_string(),
            count: 42,
        };

        save_json(&path, &config).unwrap();
        let loaded: TestConfig = load_json(&path).unwrap();

        assert_eq!(config, loaded);
    }

    #[test]
    fn test_load_or_default() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nonexistent.json");

        let config: TestConfig = load_json_or_default(&path).unwrap();
        assert_eq!(config, TestConfig::default());
    }

    #[test]
    fn test_atomic_save() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("atomic.json");

        let config = TestConfig {
            name: "atomic".to_string(),
            count: 100,
        };

        save_json_atomic(&path, &config).unwrap();

        // Verify temp file is cleaned up
        assert!(!path.with_extension("tmp").exists());

        let loaded: TestConfig = load_json(&path).unwrap();
        assert_eq!(config, loaded);
    }
}
