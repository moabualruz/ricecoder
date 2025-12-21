//! Integration with ricecoder-storage framework
//!
//! This module provides integration between the MCP tool system and the ricecoder-storage
//! framework, enabling tool registry persistence and configuration management.

use crate::error::Result;
use crate::metadata::ToolMetadata;
use crate::registry::ToolRegistry;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Tool registry storage interface
///
/// This trait defines how tool registries are persisted and loaded from storage.
pub trait ToolRegistryStorage: Send + Sync {
    /// Save a tool registry to storage
    ///
    /// # Arguments
    ///
    /// * `registry` - The tool registry to save
    /// * `path` - The path where the registry should be saved
    ///
    /// # Returns
    ///
    /// Result indicating success or failure
    fn save_registry(&self, registry: &ToolRegistry, path: &Path) -> Result<()>;

    /// Load a tool registry from storage
    ///
    /// # Arguments
    ///
    /// * `path` - The path where the registry is stored
    ///
    /// # Returns
    ///
    /// The loaded tool registry
    fn load_registry(&self, path: &Path) -> Result<ToolRegistry>;

    /// Save a single tool to storage
    ///
    /// # Arguments
    ///
    /// * `tool` - The tool to save
    /// * `path` - The path where the tool should be saved
    ///
    /// # Returns
    ///
    /// Result indicating success or failure
    fn save_tool(&self, tool: &ToolMetadata, path: &Path) -> Result<()>;

    /// Load a single tool from storage
    ///
    /// # Arguments
    ///
    /// * `path` - The path where the tool is stored
    ///
    /// # Returns
    ///
    /// The loaded tool metadata
    fn load_tool(&self, path: &Path) -> Result<ToolMetadata>;

    /// List all tools in a directory
    ///
    /// # Arguments
    ///
    /// * `path` - The directory path
    ///
    /// # Returns
    ///
    /// A list of tool metadata for all tools in the directory
    fn list_tools(&self, path: &Path) -> Result<Vec<ToolMetadata>>;
}

/// JSON-based tool registry storage
///
/// This implementation stores tool registries as JSON files.
pub struct JsonToolRegistryStorage;

impl JsonToolRegistryStorage {
    /// Creates a new JSON tool registry storage
    pub fn new() -> Self {
        Self
    }
}

impl Default for JsonToolRegistryStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolRegistryStorage for JsonToolRegistryStorage {
    fn save_registry(&self, registry: &ToolRegistry, path: &Path) -> Result<()> {
        let tools: Vec<ToolMetadata> = registry.list_tools().into_iter().cloned().collect();

        let json = serde_json::to_string_pretty(&tools)
            .map_err(|e| crate::error::Error::SerializationError(e))?;

        std::fs::write(path, json).map_err(|e| crate::error::Error::IoError(e))?;

        Ok(())
    }

    fn load_registry(&self, path: &Path) -> Result<ToolRegistry> {
        let content = std::fs::read_to_string(path).map_err(|e| crate::error::Error::IoError(e))?;

        let tools: Vec<ToolMetadata> = serde_json::from_str(&content)
            .map_err(|e| crate::error::Error::SerializationError(e))?;

        let mut registry = ToolRegistry::new();
        for tool in tools {
            registry.register_tool(tool)?;
        }

        Ok(registry)
    }

    fn save_tool(&self, tool: &ToolMetadata, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(tool)
            .map_err(|e| crate::error::Error::SerializationError(e))?;

        std::fs::write(path, json).map_err(|e| crate::error::Error::IoError(e))?;

        Ok(())
    }

    fn load_tool(&self, path: &Path) -> Result<ToolMetadata> {
        let content = std::fs::read_to_string(path).map_err(|e| crate::error::Error::IoError(e))?;

        let tool: ToolMetadata = serde_json::from_str(&content)
            .map_err(|e| crate::error::Error::SerializationError(e))?;

        Ok(tool)
    }

    fn list_tools(&self, path: &Path) -> Result<Vec<ToolMetadata>> {
        if !path.exists() {
            return Ok(Vec::new());
        }

        let mut tools = Vec::new();

        for entry in std::fs::read_dir(path).map_err(|e| crate::error::Error::IoError(e))? {
            let entry = entry.map_err(|e| crate::error::Error::IoError(e))?;
            let file_path = entry.path();

            if file_path.extension().map_or(false, |ext| ext == "json") {
                match self.load_tool(&file_path) {
                    Ok(tool) => tools.push(tool),
                    Err(e) => {
                        tracing::warn!("Failed to load tool from {:?}: {}", file_path, e);
                    }
                }
            }
        }

        Ok(tools)
    }
}

/// Tool registry persistence manager
///
/// This struct manages the persistence of tool registries using storage backends.
pub struct ToolRegistryPersistence {
    storage: Arc<dyn ToolRegistryStorage>,
    registry_path: PathBuf,
    tools_dir: PathBuf,
}

impl ToolRegistryPersistence {
    /// Creates a new tool registry persistence manager
    ///
    /// # Arguments
    ///
    /// * `storage` - The storage backend to use
    /// * `registry_path` - The path where the registry file is stored
    /// * `tools_dir` - The directory where individual tool files are stored
    pub fn new(
        storage: Arc<dyn ToolRegistryStorage>,
        registry_path: PathBuf,
        tools_dir: PathBuf,
    ) -> Self {
        Self {
            storage,
            registry_path,
            tools_dir,
        }
    }

    /// Saves a tool registry to persistent storage
    pub fn save_registry(&self, registry: &ToolRegistry) -> Result<()> {
        // Create parent directories if they don't exist
        if let Some(parent) = self.registry_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| crate::error::Error::IoError(e))?;
        }

        self.storage.save_registry(registry, &self.registry_path)?;
        tracing::info!("Tool registry saved to {:?}", self.registry_path);

        Ok(())
    }

    /// Loads a tool registry from persistent storage
    pub fn load_registry(&self) -> Result<ToolRegistry> {
        if !self.registry_path.exists() {
            tracing::info!(
                "Registry file not found at {:?}, creating new registry",
                self.registry_path
            );
            return Ok(ToolRegistry::new());
        }

        let registry = self.storage.load_registry(&self.registry_path)?;
        tracing::info!("Tool registry loaded from {:?}", self.registry_path);

        Ok(registry)
    }

    /// Saves a single tool to persistent storage
    pub fn save_tool(&self, tool: &ToolMetadata) -> Result<()> {
        // Create tools directory if it doesn't exist
        std::fs::create_dir_all(&self.tools_dir).map_err(|e| crate::error::Error::IoError(e))?;

        let tool_path = self.tools_dir.join(format!("{}.json", tool.id));
        self.storage.save_tool(tool, &tool_path)?;
        tracing::info!("Tool saved to {:?}", tool_path);

        Ok(())
    }

    /// Loads a single tool from persistent storage
    pub fn load_tool(&self, tool_id: &str) -> Result<ToolMetadata> {
        let tool_path = self.tools_dir.join(format!("{}.json", tool_id));
        self.storage.load_tool(&tool_path)
    }

    /// Lists all tools in persistent storage
    pub fn list_tools(&self) -> Result<Vec<ToolMetadata>> {
        self.storage.list_tools(&self.tools_dir)
    }

    /// Exports a registry to a specific format
    pub fn export_registry(
        &self,
        registry: &ToolRegistry,
        path: &Path,
        format: &str,
    ) -> Result<()> {
        match format {
            "json" => {
                let tools: Vec<ToolMetadata> = registry.list_tools().into_iter().cloned().collect();

                let json = serde_json::to_string_pretty(&tools)
                    .map_err(|e| crate::error::Error::SerializationError(e))?;

                std::fs::write(path, json).map_err(|e| crate::error::Error::IoError(e))?;

                Ok(())
            }
            "yaml" => {
                let tools: Vec<ToolMetadata> = registry.list_tools().into_iter().cloned().collect();

                let yaml = serde_yaml::to_string(&tools)
                    .map_err(|e| crate::error::Error::ConfigError(e.to_string()))?;

                std::fs::write(path, yaml).map_err(|e| crate::error::Error::IoError(e))?;

                Ok(())
            }
            _ => Err(crate::error::Error::ConfigError(format!(
                "Unsupported export format: {}",
                format
            ))),
        }
    }

    /// Imports a registry from a specific format
    pub fn import_registry(&self, path: &Path, format: &str) -> Result<ToolRegistry> {
        match format {
            "json" => {
                let content =
                    std::fs::read_to_string(path).map_err(|e| crate::error::Error::IoError(e))?;

                let tools: Vec<ToolMetadata> = serde_json::from_str(&content)
                    .map_err(|e| crate::error::Error::SerializationError(e))?;

                let mut registry = ToolRegistry::new();
                for tool in tools {
                    registry.register_tool(tool)?;
                }

                Ok(registry)
            }
            "yaml" => {
                let content =
                    std::fs::read_to_string(path).map_err(|e| crate::error::Error::IoError(e))?;

                let tools: Vec<ToolMetadata> = serde_yaml::from_str(&content)
                    .map_err(|e| crate::error::Error::ConfigError(e.to_string()))?;

                let mut registry = ToolRegistry::new();
                for tool in tools {
                    registry.register_tool(tool)?;
                }

                Ok(registry)
            }
            _ => Err(crate::error::Error::ConfigError(format!(
                "Unsupported import format: {}",
                format
            ))),
        }
    }
}

/// Tool registry cache for in-memory caching
///
/// This struct provides in-memory caching of tool registries to reduce
/// disk I/O and improve performance.
pub struct ToolRegistryCache {
    cache: tokio::sync::Mutex<HashMap<String, (ToolRegistry, std::time::Instant)>>,
    ttl_secs: u64,
}

impl ToolRegistryCache {
    /// Creates a new tool registry cache
    ///
    /// # Arguments
    ///
    /// * `ttl_secs` - Time-to-live for cache entries in seconds
    pub fn new(ttl_secs: u64) -> Self {
        Self {
            cache: tokio::sync::Mutex::new(HashMap::new()),
            ttl_secs,
        }
    }

    /// Gets a cached registry
    pub async fn get(&self, key: &str) -> Option<ToolRegistry> {
        let cache = self.cache.lock().await;
        if let Some((registry, timestamp)) = cache.get(key) {
            let elapsed = timestamp.elapsed().as_secs();
            if elapsed < self.ttl_secs {
                return Some(registry.clone());
            }
        }
        None
    }

    /// Sets a cached registry
    pub async fn set(&self, key: String, registry: ToolRegistry) {
        let mut cache = self.cache.lock().await;
        cache.insert(key, (registry, std::time::Instant::now()));
    }

    /// Clears the cache
    pub async fn clear(&self) {
        let mut cache = self.cache.lock().await;
        cache.clear();
    }

    /// Removes expired entries from the cache
    pub async fn cleanup_expired(&self) {
        let mut cache = self.cache.lock().await;
        let now = std::time::Instant::now();
        cache.retain(|_, (_, timestamp)| now.duration_since(*timestamp).as_secs() < self.ttl_secs);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::ToolSource;
    use tempfile::TempDir;

    #[test]
    fn test_json_tool_registry_storage_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let registry_path = temp_dir.path().join("registry.json");

        let mut registry = ToolRegistry::new();
        let tool = ToolMetadata::new(
            "test-tool".to_string(),
            "Test Tool".to_string(),
            "A test tool".to_string(),
            "test".to_string(),
            "string".to_string(),
            ToolSource::Custom,
        );
        registry.register_tool(tool).unwrap();

        let storage = JsonToolRegistryStorage::new();
        storage.save_registry(&registry, &registry_path).unwrap();

        assert!(registry_path.exists());

        let loaded_registry = storage.load_registry(&registry_path).unwrap();
        assert_eq!(loaded_registry.tool_count(), 1);
    }

    #[test]
    fn test_tool_registry_persistence_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let registry_path = temp_dir.path().join("registry.json");
        let tools_dir = temp_dir.path().join("tools");

        let storage = Arc::new(JsonToolRegistryStorage::new());
        let persistence = ToolRegistryPersistence::new(storage, registry_path, tools_dir);

        let mut registry = ToolRegistry::new();
        let tool = ToolMetadata::new(
            "test-tool".to_string(),
            "Test Tool".to_string(),
            "A test tool".to_string(),
            "test".to_string(),
            "string".to_string(),
            ToolSource::Custom,
        );
        registry.register_tool(tool).unwrap();

        persistence.save_registry(&registry).unwrap();
        let loaded_registry = persistence.load_registry().unwrap();

        assert_eq!(loaded_registry.tool_count(), 1);
    }

    #[tokio::test]
    async fn test_tool_registry_cache() {
        let cache = ToolRegistryCache::new(60);
        let registry = ToolRegistry::new();

        cache.set("test".to_string(), registry.clone()).await;
        assert!(cache.get("test").await.is_some());

        cache.clear().await;
        assert!(cache.get("test").await.is_none());
    }

    #[tokio::test]
    async fn test_tool_registry_cache_expiration() {
        let cache = ToolRegistryCache::new(0); // 0 second TTL
        let registry = ToolRegistry::new();

        cache.set("test".to_string(), registry).await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        assert!(cache.get("test").await.is_none());
    }

    #[test]
    fn test_tool_registry_persistence_export_json() {
        let temp_dir = TempDir::new().unwrap();
        let registry_path = temp_dir.path().join("registry.json");
        let tools_dir = temp_dir.path().join("tools");
        let export_path = temp_dir.path().join("export.json");

        let storage = Arc::new(JsonToolRegistryStorage::new());
        let persistence = ToolRegistryPersistence::new(storage, registry_path, tools_dir);

        let mut registry = ToolRegistry::new();
        let tool = ToolMetadata::new(
            "test-tool".to_string(),
            "Test Tool".to_string(),
            "A test tool".to_string(),
            "test".to_string(),
            "string".to_string(),
            ToolSource::Custom,
        );
        registry.register_tool(tool).unwrap();

        persistence
            .export_registry(&registry, &export_path, "json")
            .unwrap();
        assert!(export_path.exists());

        let imported_registry = persistence.import_registry(&export_path, "json").unwrap();
        assert_eq!(imported_registry.tool_count(), 1);
    }
}
