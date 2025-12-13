use ricecoder_providers::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ChatResponse, FinishReason, TokenUsage};
    use crate::provider::ChatStream;
    use std::sync::Arc;

    struct MockProvider {
        id: String,
    }

    #[async_trait::async_trait]
    impl Provider for MockProvider {
        fn id(&self) -> &str {
            &self.id
        }

        fn name(&self) -> &str {
            "Mock"
        }

        fn models(&self) -> Vec<crate::models::ModelInfo> {
            vec![]
        }

        async fn chat(&self, _request: ChatRequest) -> Result<ChatResponse, ProviderError> {
            Ok(ChatResponse {
                content: "test response".to_string(),
                model: "test-model".to_string(),
                usage: TokenUsage {
                    prompt_tokens: 10,
                    completion_tokens: 5,
                    total_tokens: 15,
                },
                finish_reason: FinishReason::Stop,
            })
        }

        async fn chat_stream(&self, _request: ChatRequest) -> Result<ChatStream, ProviderError> {
            Err(ProviderError::NotFound("Not implemented".to_string()))
        }

        fn count_tokens(&self, _content: &str, _model: &str) -> Result<usize, ProviderError> {
            Ok(0)
        }

        async fn health_check(&self) -> Result<bool, ProviderError> {
            Ok(true)
        }
    }

    #[tokio::test]
    async fn test_manager_creation() {
        let mut registry = ProviderRegistry::new();
        let provider = Arc::new(MockProvider {
            id: "test".to_string(),
        });
        registry.register(provider).unwrap();

        let manager = ProviderManager::new(registry, "test".to_string());
        assert!(manager.default_provider().is_ok());
    }

    #[tokio::test]
    async fn test_chat_request() {
        let mut registry = ProviderRegistry::new();
        let provider = Arc::new(MockProvider {
            id: "test".to_string(),
        });
        registry.register(provider).unwrap();

        let manager = ProviderManager::new(registry, "test".to_string());
        let request = ChatRequest {
            model: "test-model".to_string(),
            messages: vec![],
            temperature: None,
            max_tokens: None,
            stream: false,
        };

        let response = manager.chat(request).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_health_check() {
        let mut registry = ProviderRegistry::new();
        let provider = Arc::new(MockProvider {
            id: "test".to_string(),
        });
        registry.register(provider).unwrap();

        let manager = ProviderManager::new(registry, "test".to_string());
        let health = manager.health_check("test").await;
        assert!(health.is_ok());
        assert!(health.unwrap());
    }
}