//! Provider registry for dynamic provider registration and discovery

use std::collections::HashMap;
use std::sync::Arc;

use super::Provider;
use crate::error::ProviderError;
use crate::models::ModelInfo;

/// Registry for managing available providers
pub struct ProviderRegistry {
    providers: HashMap<String, Arc<dyn Provider>>,
}

impl ProviderRegistry {
    /// Create a new empty provider registry
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// Register a new provider
    pub fn register(&mut self, provider: Arc<dyn Provider>) -> Result<(), ProviderError> {
        let id = provider.id().to_string();
        self.providers.insert(id, provider);
        Ok(())
    }

    /// Unregister a provider by ID
    pub fn unregister(&mut self, provider_id: &str) -> Result<(), ProviderError> {
        self.providers
            .remove(provider_id)
            .ok_or_else(|| ProviderError::NotFound(provider_id.to_string()))?;
        Ok(())
    }

    /// Get a provider by ID
    pub fn get(&self, provider_id: &str) -> Result<Arc<dyn Provider>, ProviderError> {
        self.providers
            .get(provider_id)
            .cloned()
            .ok_or_else(|| ProviderError::NotFound(provider_id.to_string()))
    }

    /// Get a provider by name
    pub fn get_by_name(&self, name: &str) -> Result<Arc<dyn Provider>, ProviderError> {
        self.providers
            .values()
            .find(|p| p.name() == name)
            .cloned()
            .ok_or_else(|| ProviderError::NotFound(name.to_string()))
    }

    /// Get all registered providers
    pub fn list_all(&self) -> Vec<Arc<dyn Provider>> {
        self.providers.values().cloned().collect()
    }

    /// Get all available models across all providers
    pub fn list_all_models(&self) -> Vec<ModelInfo> {
        self.providers
            .values()
            .flat_map(|provider| provider.models())
            .collect()
    }

    /// Get models for a specific provider
    pub fn list_models(&self, provider_id: &str) -> Result<Vec<ModelInfo>, ProviderError> {
        let provider = self.get(provider_id)?;
        Ok(provider.models())
    }

    /// Check if a provider is registered
    pub fn has_provider(&self, provider_id: &str) -> bool {
        self.providers.contains_key(provider_id)
    }

    /// Get the number of registered providers
    pub fn provider_count(&self) -> usize {
        self.providers.len()
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock provider for testing
    struct MockProvider {
        id: String,
        name: String,
    }

    #[async_trait::async_trait]
    impl Provider for MockProvider {
        fn id(&self) -> &str {
            &self.id
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn models(&self) -> Vec<ModelInfo> {
            vec![]
        }

        async fn chat(
            &self,
            _request: crate::models::ChatRequest,
        ) -> Result<crate::models::ChatResponse, ProviderError> {
            Err(ProviderError::NotFound("Not implemented".to_string()))
        }

        async fn chat_stream(
            &self,
            _request: crate::models::ChatRequest,
        ) -> Result<crate::provider::ChatStream, ProviderError> {
            Err(ProviderError::NotFound("Not implemented".to_string()))
        }

        fn count_tokens(&self, _content: &str, _model: &str) -> Result<usize, ProviderError> {
            Ok(0)
        }

        async fn health_check(&self) -> Result<bool, ProviderError> {
            Ok(true)
        }
    }

    #[test]
    fn test_register_provider() {
        let mut registry = ProviderRegistry::new();
        let provider = Arc::new(MockProvider {
            id: "test".to_string(),
            name: "Test Provider".to_string(),
        });

        assert!(registry.register(provider).is_ok());
        assert!(registry.has_provider("test"));
    }

    #[test]
    fn test_get_provider() {
        let mut registry = ProviderRegistry::new();
        let provider = Arc::new(MockProvider {
            id: "test".to_string(),
            name: "Test Provider".to_string(),
        });

        registry.register(provider).unwrap();
        assert!(registry.get("test").is_ok());
        assert!(registry.get("nonexistent").is_err());
    }

    #[test]
    fn test_get_provider_by_name() {
        let mut registry = ProviderRegistry::new();
        let provider = Arc::new(MockProvider {
            id: "test".to_string(),
            name: "Test Provider".to_string(),
        });

        registry.register(provider).unwrap();
        assert!(registry.get_by_name("Test Provider").is_ok());
        assert!(registry.get_by_name("Nonexistent").is_err());
    }

    #[test]
    fn test_unregister_provider() {
        let mut registry = ProviderRegistry::new();
        let provider = Arc::new(MockProvider {
            id: "test".to_string(),
            name: "Test Provider".to_string(),
        });

        registry.register(provider).unwrap();
        assert!(registry.has_provider("test"));
        assert!(registry.unregister("test").is_ok());
        assert!(!registry.has_provider("test"));
    }

    #[test]
    fn test_list_all_providers() {
        let mut registry = ProviderRegistry::new();
        let provider1 = Arc::new(MockProvider {
            id: "test1".to_string(),
            name: "Test Provider 1".to_string(),
        });
        let provider2 = Arc::new(MockProvider {
            id: "test2".to_string(),
            name: "Test Provider 2".to_string(),
        });

        registry.register(provider1).unwrap();
        registry.register(provider2).unwrap();

        assert_eq!(registry.provider_count(), 2);
        assert_eq!(registry.list_all().len(), 2);
    }
}
