//! Global storage implementation for RiceCoder
//!
//! Manages the global knowledge base stored in ~/Documents/.ricecoder/

use crate::error::{IoOperation, StorageError, StorageResult};
use crate::manager::PathResolver;
use crate::types::ResourceType;
use std::fs;
use std::path::{Path, PathBuf};

/// Global store for managing global knowledge base
pub struct GlobalStore {
    /// Path to the global storage directory
    base_path: PathBuf,
}

impl GlobalStore {
    /// Create a new global store
    pub fn new(base_path: PathBuf) -> Self {
        GlobalStore { base_path }
    }

    /// Create a new global store with default path resolution
    pub fn with_default_path() -> StorageResult<Self> {
        let base_path = PathResolver::resolve_global_path()?;
        Ok(GlobalStore { base_path })
    }

    /// Get the base path
    pub fn base_path(&self) -> &PathBuf {
        &self.base_path
    }

    /// Initialize the global store directory structure
    ///
    /// Creates the base directory and all resource subdirectories:
    /// - templates/
    /// - standards/
    /// - specs/
    /// - steering/
    /// - boilerplates/
    /// - rules/
    /// - cache/
    pub fn initialize(&self) -> StorageResult<()> {
        // Create base directory
        self.create_dir_if_not_exists(&self.base_path)?;

        // Create resource directories
        for resource_type in &[
            ResourceType::Template,
            ResourceType::Standard,
            ResourceType::Spec,
            ResourceType::Steering,
            ResourceType::Boilerplate,
            ResourceType::Rule,
        ] {
            let resource_path = self.resource_path(*resource_type);
            self.create_dir_if_not_exists(&resource_path)?;
        }

        // Create cache directory
        let cache_path = self.base_path.join("cache");
        self.create_dir_if_not_exists(&cache_path)?;

        Ok(())
    }

    /// Get the path for a resource type
    pub fn resource_path(&self, resource_type: ResourceType) -> PathBuf {
        self.base_path.join(resource_type.dir_name())
    }

    /// Store a resource file
    pub fn store_resource(
        &self,
        resource_type: ResourceType,
        name: &str,
        content: &[u8],
    ) -> StorageResult<PathBuf> {
        let resource_dir = self.resource_path(resource_type);
        let file_path = resource_dir.join(name);

        // Ensure directory exists
        self.create_dir_if_not_exists(&resource_dir)?;

        // Write file
        fs::write(&file_path, content)
            .map_err(|e| StorageError::io_error(file_path.clone(), IoOperation::Write, e))?;

        Ok(file_path)
    }

    /// Retrieve a resource file
    pub fn retrieve_resource(
        &self,
        resource_type: ResourceType,
        name: &str,
    ) -> StorageResult<Vec<u8>> {
        let resource_dir = self.resource_path(resource_type);
        let file_path = resource_dir.join(name);

        fs::read(&file_path).map_err(|e| StorageError::io_error(file_path, IoOperation::Read, e))
    }

    /// List all resources of a type
    pub fn list_resources(&self, resource_type: ResourceType) -> StorageResult<Vec<String>> {
        let resource_dir = self.resource_path(resource_type);

        if !resource_dir.exists() {
            return Ok(Vec::new());
        }

        let mut resources = Vec::new();
        let entries = fs::read_dir(&resource_dir)
            .map_err(|e| StorageError::io_error(resource_dir.clone(), IoOperation::Read, e))?;

        for entry in entries {
            let entry = entry
                .map_err(|e| StorageError::io_error(resource_dir.clone(), IoOperation::Read, e))?;

            let path = entry.path();
            if path.is_file() {
                if let Some(file_name) = path.file_name() {
                    if let Some(name_str) = file_name.to_str() {
                        resources.push(name_str.to_string());
                    }
                }
            }
        }

        Ok(resources)
    }

    /// Delete a resource file
    pub fn delete_resource(&self, resource_type: ResourceType, name: &str) -> StorageResult<()> {
        let resource_dir = self.resource_path(resource_type);
        let file_path = resource_dir.join(name);

        if file_path.exists() {
            fs::remove_file(&file_path)
                .map_err(|e| StorageError::io_error(file_path, IoOperation::Delete, e))?;
        }

        Ok(())
    }

    /// Check if a resource exists
    pub fn resource_exists(&self, resource_type: ResourceType, name: &str) -> bool {
        let resource_dir = self.resource_path(resource_type);
        let file_path = resource_dir.join(name);
        file_path.exists()
    }

    /// Create a directory if it doesn't exist
    fn create_dir_if_not_exists(&self, path: &Path) -> StorageResult<()> {
        if !path.exists() {
            fs::create_dir_all(path)
                .map_err(|e| StorageError::directory_creation_failed(path.to_path_buf(), e))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_global_store_initialization() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let store = GlobalStore::new(temp_dir.path().to_path_buf());

        store.initialize().expect("Failed to initialize store");

        // Verify all directories were created
        assert!(store.resource_path(ResourceType::Template).exists());
        assert!(store.resource_path(ResourceType::Standard).exists());
        assert!(store.resource_path(ResourceType::Spec).exists());
        assert!(store.resource_path(ResourceType::Steering).exists());
        assert!(store.resource_path(ResourceType::Boilerplate).exists());
        assert!(store.resource_path(ResourceType::Rule).exists());
        assert!(temp_dir.path().join("cache").exists());
    }

    #[test]
    fn test_store_and_retrieve_resource() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let store = GlobalStore::new(temp_dir.path().to_path_buf());
        store.initialize().expect("Failed to initialize store");

        let content = b"test content";
        let name = "test.txt";

        // Store resource
        let path = store
            .store_resource(ResourceType::Template, name, content)
            .expect("Failed to store resource");

        assert!(path.exists());

        // Retrieve resource
        let retrieved = store
            .retrieve_resource(ResourceType::Template, name)
            .expect("Failed to retrieve resource");

        assert_eq!(retrieved, content);
    }

    #[test]
    fn test_list_resources() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let store = GlobalStore::new(temp_dir.path().to_path_buf());
        store.initialize().expect("Failed to initialize store");

        // Store multiple resources
        store
            .store_resource(ResourceType::Template, "template1.txt", b"content1")
            .expect("Failed to store");
        store
            .store_resource(ResourceType::Template, "template2.txt", b"content2")
            .expect("Failed to store");

        // List resources
        let resources = store
            .list_resources(ResourceType::Template)
            .expect("Failed to list resources");

        assert_eq!(resources.len(), 2);
        assert!(resources.contains(&"template1.txt".to_string()));
        assert!(resources.contains(&"template2.txt".to_string()));
    }

    #[test]
    fn test_delete_resource() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let store = GlobalStore::new(temp_dir.path().to_path_buf());
        store.initialize().expect("Failed to initialize store");

        let name = "test.txt";
        store
            .store_resource(ResourceType::Template, name, b"content")
            .expect("Failed to store");

        assert!(store.resource_exists(ResourceType::Template, name));

        store
            .delete_resource(ResourceType::Template, name)
            .expect("Failed to delete");

        assert!(!store.resource_exists(ResourceType::Template, name));
    }

    #[test]
    fn test_resource_exists() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let store = GlobalStore::new(temp_dir.path().to_path_buf());
        store.initialize().expect("Failed to initialize store");

        let name = "test.txt";
        assert!(!store.resource_exists(ResourceType::Template, name));

        store
            .store_resource(ResourceType::Template, name, b"content")
            .expect("Failed to store");

        assert!(store.resource_exists(ResourceType::Template, name));
    }
}
