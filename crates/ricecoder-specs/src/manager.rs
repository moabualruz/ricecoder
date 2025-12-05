//! Spec manager for discovering and managing specifications

use crate::error::SpecError;
use crate::models::Spec;
use crate::parsers::{YamlParser, MarkdownParser};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;

/// Central coordinator for spec operations and lifecycle management
pub struct SpecManager {
    /// Cache of loaded specs (path -> spec)
    cache: HashMap<PathBuf, Spec>,
}

impl SpecManager {
    /// Create a new spec manager
    pub fn new() -> Self {
        SpecManager {
            cache: HashMap::new(),
        }
    }

    /// Discover specs in a directory recursively
    ///
    /// Searches for YAML and Markdown spec files in the given directory and all subdirectories.
    /// Supports nested spec discovery (project > feature > task hierarchy).
    ///
    /// # Arguments
    /// * `path` - Root directory to search for specs
    ///
    /// # Returns
    /// A vector of discovered specs, or an error if discovery fails
    pub fn discover_specs(&mut self, path: &Path) -> Result<Vec<Spec>, SpecError> {
        let mut specs = Vec::new();

        if !path.exists() {
            return Ok(specs);
        }

        self.discover_specs_recursive(path, &mut specs)?;
        Ok(specs)
    }

    /// Recursively discover specs in a directory
    fn discover_specs_recursive(&mut self, path: &Path, specs: &mut Vec<Spec>) -> Result<(), SpecError> {
        if !path.is_dir() {
            return Ok(());
        }

        let entries = fs::read_dir(path)
            .map_err(SpecError::IoError)?;

        for entry in entries {
            let entry = entry.map_err(SpecError::IoError)?;
            let path = entry.path();

            if path.is_dir() {
                // Recursively search subdirectories
                self.discover_specs_recursive(&path, specs)?;
            } else if path.is_file() {
                // Check if file is a spec file (YAML or Markdown)
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    if ext_str == "yaml" || ext_str == "yml" || ext_str == "md" {
                        // Try to load the spec
                        if let Ok(spec) = self.load_spec(&path) {
                            specs.push(spec);
                        }
                        // Silently skip files that can't be parsed as specs
                    }
                }
            }
        }

        Ok(())
    }

    /// Load a spec from a file
    ///
    /// Automatically detects the format (YAML or Markdown) based on file extension.
    /// Caches the loaded spec for future access.
    ///
    /// # Arguments
    /// * `path` - Path to the spec file
    ///
    /// # Returns
    /// The loaded spec, or an error if loading fails
    pub fn load_spec(&mut self, path: &Path) -> Result<Spec, SpecError> {
        // Check cache first
        if let Some(spec) = self.cache.get(path) {
            return Ok(spec.clone());
        }

        // Read file content
        let content = fs::read_to_string(path)
            .map_err(SpecError::IoError)?;

        // Determine format and parse
        let spec = if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            match ext_str.as_str() {
                "yaml" | "yml" => YamlParser::parse(&content)?,
                "md" => MarkdownParser::parse(&content)?,
                _ => return Err(SpecError::InvalidFormat(
                    format!("Unsupported file format: {}", ext_str)
                )),
            }
        } else {
            return Err(SpecError::InvalidFormat(
                "File has no extension".to_string()
            ));
        };

        // Cache the spec
        self.cache.insert(path.to_path_buf(), spec.clone());

        Ok(spec)
    }

    /// Save a spec to a file
    ///
    /// Automatically determines the format (YAML or Markdown) based on file extension.
    /// Defaults to YAML if no extension is provided.
    ///
    /// # Arguments
    /// * `spec` - The spec to save
    /// * `path` - Path where the spec should be saved
    ///
    /// # Returns
    /// Ok if successful, or an error if saving fails
    pub fn save_spec(&mut self, spec: &Spec, path: &Path) -> Result<(), SpecError> {
        // Determine format and serialize
        let content = if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            match ext_str.as_str() {
                "yaml" | "yml" => YamlParser::serialize(spec)?,
                "md" => MarkdownParser::serialize(spec)?,
                _ => return Err(SpecError::InvalidFormat(
                    format!("Unsupported file format: {}", ext_str)
                )),
            }
        } else {
            // Default to YAML
            YamlParser::serialize(spec)?
        };

        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)
                    .map_err(SpecError::IoError)?;
            }
        }

        // Write file
        fs::write(path, content)
            .map_err(SpecError::IoError)?;

        // Update cache
        self.cache.insert(path.to_path_buf(), spec.clone());

        Ok(())
    }

    /// Invalidate cache for a specific spec
    ///
    /// # Arguments
    /// * `path` - Path to the spec file to invalidate
    pub fn invalidate_cache(&mut self, path: &Path) {
        self.cache.remove(path);
    }

    /// Clear all cached specs
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get the number of cached specs
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

impl Default for SpecManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_spec_manager_creation() {
        let manager = SpecManager::new();
        assert_eq!(manager.cache_size(), 0);
    }

    #[test]
    fn test_spec_manager_default() {
        let manager = SpecManager::default();
        assert_eq!(manager.cache_size(), 0);
    }

    #[test]
    fn test_discover_specs_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = SpecManager::new();
        
        let specs = manager.discover_specs(temp_dir.path()).unwrap();
        assert_eq!(specs.len(), 0);
    }

    #[test]
    fn test_discover_specs_nonexistent_directory() {
        let mut manager = SpecManager::new();
        let nonexistent = Path::new("/nonexistent/path/that/does/not/exist");
        
        let specs = manager.discover_specs(nonexistent).unwrap();
        assert_eq!(specs.len(), 0);
    }

    #[test]
    fn test_save_and_load_yaml_spec() {
        let temp_dir = TempDir::new().unwrap();
        let spec_path = temp_dir.path().join("test.yaml");
        
        let mut manager = SpecManager::new();
        
        // Create a test spec
        let spec = Spec {
            id: "test-spec".to_string(),
            name: "Test Spec".to_string(),
            version: "1.0.0".to_string(),
            requirements: vec![],
            design: None,
            tasks: vec![],
            metadata: crate::models::SpecMetadata {
                author: Some("Test Author".to_string()),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                phase: crate::models::SpecPhase::Requirements,
                status: crate::models::SpecStatus::Draft,
            },
            inheritance: None,
        };
        
        // Save the spec
        manager.save_spec(&spec, &spec_path).unwrap();
        assert!(spec_path.exists());
        
        // Load the spec
        let loaded_spec = manager.load_spec(&spec_path).unwrap();
        assert_eq!(loaded_spec.id, spec.id);
        assert_eq!(loaded_spec.name, spec.name);
        assert_eq!(loaded_spec.version, spec.version);
    }

    #[test]
    fn test_cache_invalidation() {
        let temp_dir = TempDir::new().unwrap();
        let spec_path = temp_dir.path().join("test.yaml");
        
        let mut manager = SpecManager::new();
        
        let spec = Spec {
            id: "test-spec".to_string(),
            name: "Test Spec".to_string(),
            version: "1.0.0".to_string(),
            requirements: vec![],
            design: None,
            tasks: vec![],
            metadata: crate::models::SpecMetadata {
                author: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                phase: crate::models::SpecPhase::Discovery,
                status: crate::models::SpecStatus::Draft,
            },
            inheritance: None,
        };
        
        manager.save_spec(&spec, &spec_path).unwrap();
        assert_eq!(manager.cache_size(), 1);
        
        manager.invalidate_cache(&spec_path);
        assert_eq!(manager.cache_size(), 0);
    }

    #[test]
    fn test_clear_cache() {
        let temp_dir = TempDir::new().unwrap();
        let spec_path1 = temp_dir.path().join("test1.yaml");
        let spec_path2 = temp_dir.path().join("test2.yaml");
        
        let mut manager = SpecManager::new();
        
        let spec = Spec {
            id: "test-spec".to_string(),
            name: "Test Spec".to_string(),
            version: "1.0.0".to_string(),
            requirements: vec![],
            design: None,
            tasks: vec![],
            metadata: crate::models::SpecMetadata {
                author: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                phase: crate::models::SpecPhase::Discovery,
                status: crate::models::SpecStatus::Draft,
            },
            inheritance: None,
        };
        
        manager.save_spec(&spec, &spec_path1).unwrap();
        manager.save_spec(&spec, &spec_path2).unwrap();
        assert_eq!(manager.cache_size(), 2);
        
        manager.clear_cache();
        assert_eq!(manager.cache_size(), 0);
    }

    #[test]
    fn test_load_spec_caching() {
        let temp_dir = TempDir::new().unwrap();
        let spec_path = temp_dir.path().join("test.yaml");
        
        let mut manager = SpecManager::new();
        
        let spec = Spec {
            id: "test-spec".to_string(),
            name: "Test Spec".to_string(),
            version: "1.0.0".to_string(),
            requirements: vec![],
            design: None,
            tasks: vec![],
            metadata: crate::models::SpecMetadata {
                author: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                phase: crate::models::SpecPhase::Discovery,
                status: crate::models::SpecStatus::Draft,
            },
            inheritance: None,
        };
        
        manager.save_spec(&spec, &spec_path).unwrap();
        assert_eq!(manager.cache_size(), 1);
        
        // Load again - should use cache
        let _loaded = manager.load_spec(&spec_path).unwrap();
        assert_eq!(manager.cache_size(), 1);
    }

    #[test]
    fn test_save_creates_parent_directories() {
        let temp_dir = TempDir::new().unwrap();
        let spec_path = temp_dir.path().join("nested/deep/path/test.yaml");
        
        let mut manager = SpecManager::new();
        
        let spec = Spec {
            id: "test-spec".to_string(),
            name: "Test Spec".to_string(),
            version: "1.0.0".to_string(),
            requirements: vec![],
            design: None,
            tasks: vec![],
            metadata: crate::models::SpecMetadata {
                author: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                phase: crate::models::SpecPhase::Discovery,
                status: crate::models::SpecStatus::Draft,
            },
            inheritance: None,
        };
        
        manager.save_spec(&spec, &spec_path).unwrap();
        assert!(spec_path.exists());
    }

    #[test]
    fn test_discover_specs_recursive() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create nested directory structure
        let feature_dir = temp_dir.path().join("feature1");
        fs::create_dir(&feature_dir).unwrap();
        let task_dir = feature_dir.join("task1");
        fs::create_dir(&task_dir).unwrap();
        
        let mut manager = SpecManager::new();
        
        let spec = Spec {
            id: "test-spec".to_string(),
            name: "Test Spec".to_string(),
            version: "1.0.0".to_string(),
            requirements: vec![],
            design: None,
            tasks: vec![],
            metadata: crate::models::SpecMetadata {
                author: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                phase: crate::models::SpecPhase::Discovery,
                status: crate::models::SpecStatus::Draft,
            },
            inheritance: None,
        };
        
        // Save specs at different levels
        manager.save_spec(&spec, &temp_dir.path().join("project.yaml")).unwrap();
        manager.save_spec(&spec, &feature_dir.join("feature.yaml")).unwrap();
        manager.save_spec(&spec, &task_dir.join("task.yaml")).unwrap();
        
        // Discover all specs
        let discovered = manager.discover_specs(temp_dir.path()).unwrap();
        assert_eq!(discovered.len(), 3);
    }

    #[test]
    fn test_unsupported_file_format() {
        let temp_dir = TempDir::new().unwrap();
        let spec_path = temp_dir.path().join("test.txt");
        
        let mut manager = SpecManager::new();
        
        let spec = Spec {
            id: "test-spec".to_string(),
            name: "Test Spec".to_string(),
            version: "1.0.0".to_string(),
            requirements: vec![],
            design: None,
            tasks: vec![],
            metadata: crate::models::SpecMetadata {
                author: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                phase: crate::models::SpecPhase::Discovery,
                status: crate::models::SpecStatus::Draft,
            },
            inheritance: None,
        };
        
        let result = manager.save_spec(&spec, &spec_path);
        assert!(result.is_err());
    }
}
