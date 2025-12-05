//! Configuration loading from ricecoder-storage with hierarchy support

use crate::error::Result;
use crate::types::RefactoringConfig;
use ricecoder_storage::manager::StorageManager;
use ricecoder_storage::types::ResourceType;
use std::sync::Arc;

/// Loads refactoring configurations from storage with hierarchy support
pub struct StorageConfigLoader {
    storage: Arc<dyn StorageManager>,
}

impl StorageConfigLoader {
    /// Create a new storage configuration loader
    pub fn new(storage: Arc<dyn StorageManager>) -> Self {
        Self { storage }
    }

    /// Load language configuration with hierarchy support
    ///
    /// Priority (highest to lowest):
    /// 1. Project-level config (./.agent/refactoring/languages/{language}.yaml)
    /// 2. User-level config (~/.ricecoder/refactoring/languages/{language}.yaml)
    /// 3. Built-in config (fallback)
    pub fn load_language_config(&self, language: &str) -> Result<RefactoringConfig> {
        // Try project-level first
        if let Some(project_path) = self
            .storage
            .project_resource_path(ResourceType::RefactoringLanguageConfig)
        {
            let config_path = project_path.join(format!("{}.yaml", language));
            if config_path.exists() {
                tracing::debug!(
                    "Loading refactoring config from project: {}",
                    config_path.display()
                );
                let config = self.load_from_file(&config_path)?;
                config.validate()?;
                return Ok(config);
            }
        }

        // Try user-level
        let global_path = self
            .storage
            .global_resource_path(ResourceType::RefactoringLanguageConfig);
        let config_path = global_path.join(format!("{}.yaml", language));
        if config_path.exists() {
            tracing::debug!(
                "Loading refactoring config from user: {}",
                config_path.display()
            );
            let config = self.load_from_file(&config_path)?;
            config.validate()?;
            return Ok(config);
        }

        // Fall back to built-in (generic refactoring)
        tracing::debug!(
            "No configuration found for language '{}', using generic refactoring",
            language
        );
        Ok(RefactoringConfig::generic_fallback(language))
    }

    /// Load configuration from a file
    fn load_from_file(&self, path: &std::path::Path) -> Result<RefactoringConfig> {
        use crate::config::loader::ConfigLoader;
        ConfigLoader::load(path)
    }

    /// Get all available language configurations
    pub fn list_available_languages(&self) -> Result<Vec<String>> {
        let mut languages = Vec::new();

        // Check project-level
        if let Some(project_path) = self
            .storage
            .project_resource_path(ResourceType::RefactoringLanguageConfig)
        {
            if project_path.exists() {
                if let Ok(entries) = std::fs::read_dir(&project_path) {
                    for entry in entries.flatten() {
                        if let Some(name) = entry.file_name().to_str() {
                            if name.ends_with(".yaml") || name.ends_with(".yml") {
                                let lang = name.trim_end_matches(".yaml").trim_end_matches(".yml");
                                if !languages.contains(&lang.to_string()) {
                                    languages.push(lang.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        // Check user-level
        let global_path = self
            .storage
            .global_resource_path(ResourceType::RefactoringLanguageConfig);
        if global_path.exists() {
            if let Ok(entries) = std::fs::read_dir(&global_path) {
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str() {
                        if name.ends_with(".yaml") || name.ends_with(".yml") {
                            let lang = name.trim_end_matches(".yaml").trim_end_matches(".yml");
                            if !languages.contains(&lang.to_string()) {
                                languages.push(lang.to_string());
                            }
                        }
                    }
                }
            }
        }

        Ok(languages)
    }

    /// Check if a language configuration exists
    pub fn has_language_config(&self, language: &str) -> Result<bool> {
        // Check project-level
        if let Some(project_path) = self
            .storage
            .project_resource_path(ResourceType::RefactoringLanguageConfig)
        {
            let config_path = project_path.join(format!("{}.yaml", language));
            if config_path.exists() {
                return Ok(true);
            }
        }

        // Check user-level
        let global_path = self
            .storage
            .global_resource_path(ResourceType::RefactoringLanguageConfig);
        let config_path = global_path.join(format!("{}.yaml", language));
        Ok(config_path.exists())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_storage::types::StorageMode;
    use std::path::PathBuf;

    /// Mock storage manager for testing
    struct MockStorageManager {
        global_path: PathBuf,
        project_path: Option<PathBuf>,
    }

    impl StorageManager for MockStorageManager {
        fn global_path(&self) -> &PathBuf {
            &self.global_path
        }

        fn project_path(&self) -> Option<&PathBuf> {
            self.project_path.as_ref()
        }

        fn mode(&self) -> StorageMode {
            StorageMode::Merged
        }

        fn global_resource_path(&self, resource_type: ResourceType) -> PathBuf {
            self.global_path.join(resource_type.dir_name())
        }

        fn project_resource_path(&self, resource_type: ResourceType) -> Option<PathBuf> {
            self.project_path
                .as_ref()
                .map(|p| p.join(resource_type.dir_name()))
        }

        fn is_first_run(&self) -> bool {
            false
        }
    }

    #[test]
    fn test_load_language_config_fallback() -> Result<()> {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let storage = Arc::new(MockStorageManager {
            global_path: temp_dir.path().to_path_buf(),
            project_path: None,
        });

        let loader = StorageConfigLoader::new(storage);
        let config = loader.load_language_config("unknown_language")?;

        // Should return generic fallback
        assert_eq!(config.language, "unknown_language");
        assert!(config.rules.is_empty());

        Ok(())
    }

    #[test]
    fn test_list_available_languages() -> Result<()> {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let refactoring_dir = temp_dir.path().join("refactoring/languages");
        std::fs::create_dir_all(&refactoring_dir).expect("Failed to create dir");

        // Create some dummy config files
        std::fs::write(refactoring_dir.join("rust.yaml"), "language: rust").ok();
        std::fs::write(refactoring_dir.join("python.yaml"), "language: python").ok();

        let storage = Arc::new(MockStorageManager {
            global_path: temp_dir.path().to_path_buf(),
            project_path: None,
        });

        let loader = StorageConfigLoader::new(storage);
        let languages = loader.list_available_languages()?;

        assert!(languages.contains(&"rust".to_string()));
        assert!(languages.contains(&"python".to_string()));

        Ok(())
    }

    #[test]
    fn test_has_language_config() -> Result<()> {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let refactoring_dir = temp_dir.path().join("refactoring/languages");
        std::fs::create_dir_all(&refactoring_dir).expect("Failed to create dir");

        std::fs::write(refactoring_dir.join("rust.yaml"), "language: rust").ok();

        let storage = Arc::new(MockStorageManager {
            global_path: temp_dir.path().to_path_buf(),
            project_path: None,
        });

        let loader = StorageConfigLoader::new(storage);

        assert!(loader.has_language_config("rust")?);
        assert!(!loader.has_language_config("unknown")?);

        Ok(())
    }
}
