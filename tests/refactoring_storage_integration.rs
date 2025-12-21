//! Integration tests for storage integration with refactoring engine

use ricecoder_refactoring::{ConfigManager, StorageConfigLoader};
use ricecoder_storage::manager::StorageManager;
use ricecoder_storage::types::{ResourceType, StorageMode};
use std::path::PathBuf;
use std::sync::Arc;

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

#[tokio::test]
async fn test_config_manager_with_storage() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let storage = Arc::new(MockStorageManager {
        global_path: temp_dir.path().to_path_buf(),
        project_path: None,
    });

    let manager = ConfigManager::with_storage(storage.clone());

    // Create a test configuration file
    let refactoring_dir = temp_dir.path().join("refactoring/languages");
    std::fs::create_dir_all(&refactoring_dir)?;

    let rust_config = r#"
language: rust
extensions:
  - .rs
rules: []
transformations: []
"#;

    std::fs::write(refactoring_dir.join("rust.yaml"), rust_config)?;

    // Test loading configuration from storage
    let config = manager.get_config("rust").await?;
    assert!(config.is_some());
    assert_eq!(config.unwrap().language, "rust");

    Ok(())
}

#[tokio::test]
async fn test_config_manager_fallback_to_generic() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let storage = Arc::new(MockStorageManager {
        global_path: temp_dir.path().to_path_buf(),
        project_path: None,
    });

    let manager = ConfigManager::with_storage(storage.clone());

    // Try to load configuration for unconfigured language
    let config = manager.get_config("unknown_language").await?;
    assert!(config.is_some());
    assert_eq!(config.unwrap().language, "unknown_language");

    Ok(())
}

#[tokio::test]
async fn test_storage_config_loader_hierarchy() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;

    // Create project-level configuration
    let project_dir = temp_dir.path().join(".agent");
    let project_refactoring_dir = project_dir.join("refactoring/languages");
    std::fs::create_dir_all(&project_refactoring_dir)?;

    let project_config = r#"
language: rust
extensions:
  - .rs
rules:
  - name: "project_rule"
    pattern: "test"
    refactoring_type: "Rename"
    enabled: true
transformations: []
"#;

    std::fs::write(project_refactoring_dir.join("rust.yaml"), project_config)?;

    // Create user-level configuration
    let global_refactoring_dir = temp_dir.path().join("refactoring/languages");
    std::fs::create_dir_all(&global_refactoring_dir)?;

    let global_config = r#"
language: rust
extensions:
  - .rs
rules:
  - name: "global_rule"
    pattern: "test"
    refactoring_type: "Rename"
    enabled: true
transformations: []
"#;

    std::fs::write(global_refactoring_dir.join("rust.yaml"), global_config)?;

    let storage = Arc::new(MockStorageManager {
        global_path: temp_dir.path().to_path_buf(),
        project_path: Some(project_dir),
    });

    let loader = StorageConfigLoader::new(storage);

    // Should load project-level configuration (higher priority)
    let config = loader.load_language_config("rust")?;
    assert_eq!(config.rules.len(), 1);
    assert_eq!(config.rules[0].name, "project_rule");

    Ok(())
}

#[tokio::test]
async fn test_storage_config_loader_list_languages() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let refactoring_dir = temp_dir.path().join("refactoring/languages");
    std::fs::create_dir_all(&refactoring_dir)?;

    // Create multiple language configurations
    std::fs::write(refactoring_dir.join("rust.yaml"), "language: rust")?;
    std::fs::write(refactoring_dir.join("python.yaml"), "language: python")?;
    std::fs::write(
        refactoring_dir.join("typescript.yaml"),
        "language: typescript",
    )?;

    let storage = Arc::new(MockStorageManager {
        global_path: temp_dir.path().to_path_buf(),
        project_path: None,
    });

    let loader = StorageConfigLoader::new(storage);
    let languages = loader.list_available_languages()?;

    assert_eq!(languages.len(), 3);
    assert!(languages.contains(&"rust".to_string()));
    assert!(languages.contains(&"python".to_string()));
    assert!(languages.contains(&"typescript".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_storage_config_loader_has_language() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let refactoring_dir = temp_dir.path().join("refactoring/languages");
    std::fs::create_dir_all(&refactoring_dir)?;

    std::fs::write(refactoring_dir.join("rust.yaml"), "language: rust")?;

    let storage = Arc::new(MockStorageManager {
        global_path: temp_dir.path().to_path_buf(),
        project_path: None,
    });

    let loader = StorageConfigLoader::new(storage);

    assert!(loader.has_language_config("rust")?);
    assert!(!loader.has_language_config("unknown")?);

    Ok(())
}

#[tokio::test]
async fn test_config_manager_get_languages_from_storage() -> Result<(), Box<dyn std::error::Error>>
{
    let temp_dir = tempfile::tempdir()?;
    let refactoring_dir = temp_dir.path().join("refactoring/languages");
    std::fs::create_dir_all(&refactoring_dir)?;

    std::fs::write(refactoring_dir.join("rust.yaml"), "language: rust")?;
    std::fs::write(refactoring_dir.join("python.yaml"), "language: python")?;

    let storage = Arc::new(MockStorageManager {
        global_path: temp_dir.path().to_path_buf(),
        project_path: None,
    });

    let manager = ConfigManager::with_storage(storage);
    let languages = manager.get_languages().await?;

    assert_eq!(languages.len(), 2);
    assert!(languages.contains(&"rust".to_string()));
    assert!(languages.contains(&"python".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_config_manager_has_language_from_storage() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let refactoring_dir = temp_dir.path().join("refactoring/languages");
    std::fs::create_dir_all(&refactoring_dir)?;

    std::fs::write(refactoring_dir.join("rust.yaml"), "language: rust")?;

    let storage = Arc::new(MockStorageManager {
        global_path: temp_dir.path().to_path_buf(),
        project_path: None,
    });

    let manager = ConfigManager::with_storage(storage);

    assert!(manager.has_language("rust").await?);
    assert!(!manager.has_language("unknown").await?);

    Ok(())
}

#[tokio::test]
async fn test_config_manager_caches_loaded_config() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let refactoring_dir = temp_dir.path().join("refactoring/languages");
    std::fs::create_dir_all(&refactoring_dir)?;

    let rust_config = r#"
language: rust
extensions:
  - .rs
rules: []
transformations: []
"#;

    std::fs::write(refactoring_dir.join("rust.yaml"), rust_config)?;

    let storage = Arc::new(MockStorageManager {
        global_path: temp_dir.path().to_path_buf(),
        project_path: None,
    });

    let manager = ConfigManager::with_storage(storage);

    // First load
    let config1 = manager.get_config("rust").await?;
    assert!(config1.is_some());

    // Delete the file
    std::fs::remove_file(refactoring_dir.join("rust.yaml"))?;

    // Second load should still work (from cache)
    let config2 = manager.get_config("rust").await?;
    assert!(config2.is_some());
    assert_eq!(config1.unwrap().language, config2.unwrap().language);

    Ok(())
}

#[test]
fn test_resource_type_refactoring_dir_name() {
    assert_eq!(
        ResourceType::RefactoringLanguageConfig.dir_name(),
        "refactoring/languages"
    );
}
