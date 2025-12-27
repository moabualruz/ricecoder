//! Tool description loader for RiceCoder
//!
//! Loads tool descriptions from `config/tools/*.txt` files.
//! Each file contains the description for a single tool, identified by filename.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use crate::error::{IoOperation, StorageError, StorageResult};

/// Cached tool descriptions
#[derive(Debug, Clone)]
struct ToolDescriptionCache {
    /// Tool ID -> Description mapping
    descriptions: HashMap<String, String>,
    /// Whether cache has been populated
    loaded: bool,
}

/// Loader for tool description files
pub struct ToolDescriptionLoader {
    config_dir: PathBuf,
    cache: Arc<RwLock<ToolDescriptionCache>>,
}

impl ToolDescriptionLoader {
    /// Create a new tool description loader with the given config directory
    pub fn new(config_dir: PathBuf) -> Self {
        Self {
            config_dir,
            cache: Arc::new(RwLock::new(ToolDescriptionCache {
                descriptions: HashMap::new(),
                loaded: false,
            })),
        }
    }

    /// Create a loader with the default config path (config/tools)
    pub fn with_default_path() -> Self {
        // Try to find config directory relative to current directory or workspace
        let config_dir = Self::find_config_dir().unwrap_or_else(|| PathBuf::from("config/tools"));
        Self::new(config_dir)
    }

    /// Find the config/tools directory by searching up the directory tree
    fn find_config_dir() -> Option<PathBuf> {
        let mut current = std::env::current_dir().ok()?;

        // Check common locations
        let candidates = [
            current.join("config").join("tools"),
            current.join("ricecoder").join("config").join("tools"),
        ];

        for candidate in &candidates {
            if candidate.is_dir() {
                return Some(candidate.clone());
            }
        }

        // Walk up the directory tree
        loop {
            let tools_dir = current.join("config").join("tools");
            if tools_dir.is_dir() {
                return Some(tools_dir);
            }

            if !current.pop() {
                break;
            }
        }

        None
    }

    /// Load all tool descriptions from the config directory
    fn load_all(&self) -> StorageResult<HashMap<String, String>> {
        // Check cache first
        {
            let cache = self.cache.read().map_err(|_| {
                StorageError::Internal("Failed to acquire read lock on tool descriptions cache".to_string())
            })?;
            if cache.loaded {
                return Ok(cache.descriptions.clone());
            }
        }

        // Load from files
        let mut descriptions = HashMap::new();

        if !self.config_dir.exists() {
            tracing::debug!(
                "Tool descriptions directory not found: {:?}",
                self.config_dir
            );
            return Ok(descriptions);
        }

        let entries = fs::read_dir(&self.config_dir).map_err(|e| {
            StorageError::io_error(self.config_dir.clone(), IoOperation::Read, e)
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                StorageError::io_error(self.config_dir.clone(), IoOperation::Read, e)
            })?;

            let path = entry.path();

            // Only process .txt files
            if path.extension().and_then(|s| s.to_str()) != Some("txt") {
                continue;
            }

            // Extract tool ID from filename
            let tool_id = path
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string());

            if let Some(tool_id) = tool_id {
                match fs::read_to_string(&path) {
                    Ok(content) => {
                        tracing::debug!("Loaded description for tool: {}", tool_id);
                        descriptions.insert(tool_id, content.trim().to_string());
                    }
                    Err(e) => {
                        tracing::warn!("Failed to read tool description {:?}: {}", path, e);
                    }
                }
            }
        }

        // Update cache
        {
            let mut cache = self.cache.write().map_err(|_| {
                StorageError::Internal("Failed to acquire write lock on tool descriptions cache".to_string())
            })?;
            cache.descriptions = descriptions.clone();
            cache.loaded = true;
        }

        tracing::info!(
            "Loaded {} tool descriptions from {:?}",
            descriptions.len(),
            self.config_dir
        );

        Ok(descriptions)
    }

    /// Get description for a specific tool
    ///
    /// Returns None if no external description is found (tool should use hardcoded fallback)
    pub fn get_description(&self, tool_id: &str) -> Option<String> {
        // Try cache first
        {
            if let Ok(cache) = self.cache.read() {
                if cache.loaded {
                    return cache.descriptions.get(tool_id).cloned();
                }
            }
        }

        // Load and try again
        if let Ok(descriptions) = self.load_all() {
            return descriptions.get(tool_id).cloned();
        }

        None
    }

    /// Get all tool descriptions
    pub fn get_all(&self) -> StorageResult<HashMap<String, String>> {
        self.load_all()
    }

    /// Check if a tool has an external description
    pub fn has_description(&self, tool_id: &str) -> bool {
        self.get_description(tool_id).is_some()
    }

    /// Reload descriptions from disk (clears cache)
    pub fn reload(&self) -> StorageResult<()> {
        // Clear cache
        {
            let mut cache = self.cache.write().map_err(|_| {
                StorageError::Internal("Failed to acquire write lock on tool descriptions cache".to_string())
            })?;
            cache.descriptions.clear();
            cache.loaded = false;
        }

        // Reload
        self.load_all()?;
        Ok(())
    }

    /// Get the config directory path
    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }
}

/// Global tool description loader singleton
static GLOBAL_LOADER: std::sync::OnceLock<ToolDescriptionLoader> = std::sync::OnceLock::new();

/// Get the global tool description loader
pub fn global_tool_descriptions() -> &'static ToolDescriptionLoader {
    GLOBAL_LOADER.get_or_init(ToolDescriptionLoader::with_default_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_load_tool_descriptions() {
        let temp_dir = TempDir::new().unwrap();
        let tools_dir = temp_dir.path().join("tools");
        fs::create_dir_all(&tools_dir).unwrap();

        // Create test description files
        let mut file = fs::File::create(tools_dir.join("bash.txt")).unwrap();
        writeln!(file, "Execute bash commands in the shell.").unwrap();
        writeln!(file, "Use this for running tests, builds, and git commands.").unwrap();

        let mut file = fs::File::create(tools_dir.join("read.txt")).unwrap();
        writeln!(file, "Read file contents from disk.").unwrap();

        let loader = ToolDescriptionLoader::new(tools_dir);
        let descriptions = loader.get_all().unwrap();

        assert_eq!(descriptions.len(), 2);
        assert!(descriptions.get("bash").unwrap().contains("Execute bash"));
        assert!(descriptions.get("read").unwrap().contains("Read file"));
    }

    #[test]
    fn test_get_single_description() {
        let temp_dir = TempDir::new().unwrap();
        let tools_dir = temp_dir.path().join("tools");
        fs::create_dir_all(&tools_dir).unwrap();

        let mut file = fs::File::create(tools_dir.join("glob.txt")).unwrap();
        writeln!(file, "Find files matching glob patterns.").unwrap();

        let loader = ToolDescriptionLoader::new(tools_dir);

        assert!(loader.get_description("glob").is_some());
        assert!(loader.get_description("nonexistent").is_none());
    }

    #[test]
    fn test_cache_works() {
        let temp_dir = TempDir::new().unwrap();
        let tools_dir = temp_dir.path().join("tools");
        fs::create_dir_all(&tools_dir).unwrap();

        let mut file = fs::File::create(tools_dir.join("test.txt")).unwrap();
        writeln!(file, "Test description").unwrap();

        let loader = ToolDescriptionLoader::new(tools_dir.clone());

        // First load
        let desc1 = loader.get_description("test");
        assert!(desc1.is_some());

        // Modify file (won't be picked up due to cache)
        let mut file = fs::File::create(tools_dir.join("test.txt")).unwrap();
        writeln!(file, "Modified description").unwrap();

        let desc2 = loader.get_description("test");
        assert_eq!(desc1, desc2); // Still cached

        // Reload
        loader.reload().unwrap();
        let desc3 = loader.get_description("test");
        assert!(desc3.unwrap().contains("Modified"));
    }

    #[test]
    fn test_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let tools_dir = temp_dir.path().join("tools");
        fs::create_dir_all(&tools_dir).unwrap();

        let loader = ToolDescriptionLoader::new(tools_dir);
        let descriptions = loader.get_all().unwrap();

        assert!(descriptions.is_empty());
    }

    #[test]
    fn test_nonexistent_directory() {
        let loader = ToolDescriptionLoader::new(PathBuf::from("/nonexistent/path"));
        let descriptions = loader.get_all().unwrap();

        assert!(descriptions.is_empty());
    }
}
