//! Configuration manager for refactoring engine

use crate::error::Result;
use crate::providers::ProviderRegistry;
use crate::types::RefactoringConfig;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use ricecoder_storage::manager::StorageManager;

/// Manages refactoring configurations and providers with storage integration
pub struct ConfigManager {
    configs: Arc<RwLock<HashMap<String, RefactoringConfig>>>,
    provider_registry: Arc<RwLock<Option<ProviderRegistry>>>,
    storage: Option<Arc<dyn StorageManager>>,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new() -> Self {
        Self {
            configs: Arc::new(RwLock::new(HashMap::new())),
            provider_registry: Arc::new(RwLock::new(None)),
            storage: None,
        }
    }

    /// Create a new configuration manager with storage integration
    pub fn with_storage(storage: Arc<dyn StorageManager>) -> Self {
        Self {
            configs: Arc::new(RwLock::new(HashMap::new())),
            provider_registry: Arc::new(RwLock::new(None)),
            storage: Some(storage),
        }
    }

    /// Create a new configuration manager with a provider registry
    pub fn with_provider_registry(provider_registry: ProviderRegistry) -> Self {
        Self {
            configs: Arc::new(RwLock::new(HashMap::new())),
            provider_registry: Arc::new(RwLock::new(Some(provider_registry))),
            storage: None,
        }
    }

    /// Create a new configuration manager with both storage and provider registry
    pub fn with_storage_and_registry(
        storage: Arc<dyn StorageManager>,
        provider_registry: ProviderRegistry,
    ) -> Self {
        Self {
            configs: Arc::new(RwLock::new(HashMap::new())),
            provider_registry: Arc::new(RwLock::new(Some(provider_registry))),
            storage: Some(storage),
        }
    }

    /// Set the storage manager
    pub fn set_storage(&mut self, storage: Arc<dyn StorageManager>) {
        self.storage = Some(storage);
    }

    /// Get the storage manager
    pub fn get_storage(&self) -> Option<&Arc<dyn StorageManager>> {
        self.storage.as_ref()
    }

    /// Set the provider registry
    pub async fn set_provider_registry(&self, registry: ProviderRegistry) -> Result<()> {
        let mut provider_registry = self.provider_registry.write().await;
        *provider_registry = Some(registry);
        Ok(())
    }

    /// Get the provider registry
    pub async fn get_provider_registry(&self) -> Result<Option<ProviderRegistry>> {
        let provider_registry = self.provider_registry.read().await;
        Ok(provider_registry.clone())
    }

    /// Register a configuration for a language
    pub async fn register_config(&self, config: RefactoringConfig) -> Result<()> {
        let mut configs = self.configs.write().await;
        configs.insert(config.language.clone(), config);
        Ok(())
    }

    /// Get configuration for a language
    ///
    /// If storage is configured, loads from storage with hierarchy support.
    /// Otherwise, returns cached configuration.
    pub async fn get_config(&self, language: &str) -> Result<Option<RefactoringConfig>> {
        // If storage is available, load from storage with hierarchy
        if let Some(storage) = &self.storage {
            use crate::config::storage_loader::StorageConfigLoader;
            let loader = StorageConfigLoader::new(storage.clone());
            
            // Check if configuration exists in storage
            if loader.has_language_config(language)? {
                let config = loader.load_language_config(language)?;
                // Cache the loaded configuration
                let mut configs = self.configs.write().await;
                configs.insert(language.to_string(), config.clone());
                return Ok(Some(config));
            }
        }

        // Fall back to cached configuration
        let configs = self.configs.read().await;
        Ok(configs.get(language).cloned())
    }

    /// Load configuration from a file and register it
    pub async fn load_and_register(&self, path: &std::path::Path) -> Result<()> {
        use crate::config::loader::ConfigLoader;

        let config = ConfigLoader::load(path)?;
        ConfigLoader::validate(&config)?;
        self.register_config(config).await?;

        Ok(())
    }

    /// Get all registered languages
    ///
    /// If storage is configured, returns all available languages from storage.
    /// Otherwise, returns cached languages.
    pub async fn get_languages(&self) -> Result<Vec<String>> {
        // If storage is available, get languages from storage
        if let Some(storage) = &self.storage {
            use crate::config::storage_loader::StorageConfigLoader;
            let loader = StorageConfigLoader::new(storage.clone());
            return loader.list_available_languages();
        }

        // Fall back to cached languages
        let configs = self.configs.read().await;
        Ok(configs.keys().cloned().collect())
    }

    /// Check if a language is configured
    ///
    /// If storage is configured, checks storage for configuration.
    /// Otherwise, checks cached configurations.
    pub async fn has_language(&self, language: &str) -> Result<bool> {
        // If storage is available, check storage
        if let Some(storage) = &self.storage {
            use crate::config::storage_loader::StorageConfigLoader;
            let loader = StorageConfigLoader::new(storage.clone());
            return loader.has_language_config(language);
        }

        // Fall back to cached configurations
        let configs = self.configs.read().await;
        Ok(configs.contains_key(language))
    }

    /// Clear all configurations
    pub async fn clear(&self) -> Result<()> {
        let mut configs = self.configs.write().await;
        configs.clear();
        Ok(())
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_and_get_config() -> Result<()> {
        let manager = ConfigManager::new();
        let config = RefactoringConfig {
            language: "rust".to_string(),
            extensions: vec![".rs".to_string()],
            rules: vec![],
            transformations: vec![],
            provider: None,
        };

        manager.register_config(config.clone()).await?;
        let retrieved = manager.get_config("rust").await?;

        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().language, "rust");

        Ok(())
    }

    #[tokio::test]
    async fn test_get_languages() -> Result<()> {
        let manager = ConfigManager::new();

        let rust_config = RefactoringConfig {
            language: "rust".to_string(),
            extensions: vec![".rs".to_string()],
            rules: vec![],
            transformations: vec![],
            provider: None,
        };

        let ts_config = RefactoringConfig {
            language: "typescript".to_string(),
            extensions: vec![".ts".to_string()],
            rules: vec![],
            transformations: vec![],
            provider: None,
        };

        manager.register_config(rust_config).await?;
        manager.register_config(ts_config).await?;

        let languages = manager.get_languages().await?;
        assert_eq!(languages.len(), 2);
        assert!(languages.contains(&"rust".to_string()));
        assert!(languages.contains(&"typescript".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn test_has_language() -> Result<()> {
        let manager = ConfigManager::new();
        let config = RefactoringConfig {
            language: "rust".to_string(),
            extensions: vec![".rs".to_string()],
            rules: vec![],
            transformations: vec![],
            provider: None,
        };

        manager.register_config(config).await?;

        assert!(manager.has_language("rust").await?);
        assert!(!manager.has_language("python").await?);

        Ok(())
    }

    #[tokio::test]
    async fn test_clear() -> Result<()> {
        let manager = ConfigManager::new();
        let config = RefactoringConfig {
            language: "rust".to_string(),
            extensions: vec![".rs".to_string()],
            rules: vec![],
            transformations: vec![],
            provider: None,
        };

        manager.register_config(config).await?;
        assert!(manager.has_language("rust").await?);

        manager.clear().await?;
        assert!(!manager.has_language("rust").await?);

        Ok(())
    }
}
