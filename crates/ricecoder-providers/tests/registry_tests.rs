use ricecoder_providers::*;

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