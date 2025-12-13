use ricecoder_providers::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ChatRequest;
    use crate::models::{ChatResponse, FinishReason, TokenUsage};
    use crate::provider::Provider;
    use async_trait::async_trait;

    struct MockHealthyProvider;

    #[async_trait]
    impl Provider for MockHealthyProvider {
        fn id(&self) -> &str {
            "mock-healthy"
        }

        fn name(&self) -> &str {
            "Mock Healthy"
        }

        fn models(&self) -> Vec<crate::models::ModelInfo> {
            vec![]
        }

        async fn chat(&self, _request: ChatRequest) -> Result<ChatResponse, ProviderError> {
            Ok(ChatResponse {
                content: "test".to_string(),
                model: "test".to_string(),
                usage: TokenUsage {
                    prompt_tokens: 0,
                    completion_tokens: 0,
                    total_tokens: 0,
                },
                finish_reason: FinishReason::Stop,
            })
        }

        async fn chat_stream(
            &self,
            _request: ChatRequest,
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

    struct MockUnhealthyProvider;

    #[async_trait]
    impl Provider for MockUnhealthyProvider {
        fn id(&self) -> &str {
            "mock-unhealthy"
        }

        fn name(&self) -> &str {
            "Mock Unhealthy"
        }

        fn models(&self) -> Vec<crate::models::ModelInfo> {
            vec![]
        }

        async fn chat(&self, _request: ChatRequest) -> Result<ChatResponse, ProviderError> {
            Ok(ChatResponse {
                content: "test".to_string(),
                model: "test".to_string(),
                usage: TokenUsage {
                    prompt_tokens: 0,
                    completion_tokens: 0,
                    total_tokens: 0,
                },
                finish_reason: FinishReason::Stop,
            })
        }

        async fn chat_stream(
            &self,
            _request: ChatRequest,
        ) -> Result<crate::provider::ChatStream, ProviderError> {
            Err(ProviderError::NotFound("Not implemented".to_string()))
        }

        fn count_tokens(&self, _content: &str, _model: &str) -> Result<usize, ProviderError> {
            Ok(0)
        }

        async fn health_check(&self) -> Result<bool, ProviderError> {
            Err(ProviderError::ProviderError("Provider is down".to_string()))
        }
    }

    #[tokio::test]
    async fn test_health_check_cache_healthy() {
        let cache = HealthCheckCache::default();
        let provider: Arc<dyn Provider> = Arc::new(MockHealthyProvider);

        let result = cache.check_health(&provider).await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_health_check_cache_unhealthy() {
        let cache = HealthCheckCache::default();
        let provider: Arc<dyn Provider> = Arc::new(MockUnhealthyProvider);

        let result = cache.check_health(&provider).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_health_check_caching() {
        let cache = HealthCheckCache::default();
        let provider: Arc<dyn Provider> = Arc::new(MockHealthyProvider);

        // First check
        let result1 = cache.check_health(&provider).await;
        assert!(result1.is_ok());

        // Second check should use cache
        let result2 = cache.check_health(&provider).await;
        assert!(result2.is_ok());

        // Verify cache entry exists
        let cached = cache.get_cached("mock-healthy").await;
        assert!(cached.is_some());
    }

    #[tokio::test]
    async fn test_health_check_invalidate() {
        let cache = HealthCheckCache::default();
        let provider: Arc<dyn Provider> = Arc::new(MockHealthyProvider);

        // First check
        cache.check_health(&provider).await.ok();

        // Verify cache entry exists
        let cached = cache.get_cached("mock-healthy").await;
        assert!(cached.is_some());

        // Invalidate
        cache.invalidate("mock-healthy").await;

        // Verify cache entry is gone
        let cached = cache.get_cached("mock-healthy").await;
        assert!(cached.is_none());
    }

    #[tokio::test]
    async fn test_health_check_invalidate_all() {
        let cache = HealthCheckCache::default();
        let provider1: Arc<dyn Provider> = Arc::new(MockHealthyProvider);
        let provider2: Arc<dyn Provider> = Arc::new(MockUnhealthyProvider);

        // Perform checks
        cache.check_health(&provider1).await.ok();
        cache.check_health(&provider2).await.ok();

        // Verify cache entries exist
        assert!(cache.get_cached("mock-healthy").await.is_some());
        assert!(cache.get_cached("mock-unhealthy").await.is_some());

        // Invalidate all
        cache.invalidate_all().await;

        // Verify all cache entries are gone
        assert!(cache.get_cached("mock-healthy").await.is_none());
        assert!(cache.get_cached("mock-unhealthy").await.is_none());
    }

    #[tokio::test]
    async fn test_health_check_timeout() {
        let cache = HealthCheckCache::new(Duration::from_secs(300), Duration::from_millis(1));

        struct SlowProvider;

        #[async_trait]
        impl Provider for SlowProvider {
            fn id(&self) -> &str {
                "slow"
            }

            fn name(&self) -> &str {
                "Slow"
            }

            fn models(&self) -> Vec<crate::models::ModelInfo> {
                vec![]
            }

            async fn chat(&self, _request: ChatRequest) -> Result<ChatResponse, ProviderError> {
                Ok(ChatResponse {
                    content: "test".to_string(),
                    model: "test".to_string(),
                    usage: TokenUsage {
                        prompt_tokens: 0,
                        completion_tokens: 0,
                        total_tokens: 0,
                    },
                    finish_reason: FinishReason::Stop,
                })
            }

            async fn chat_stream(
                &self,
                _request: ChatRequest,
            ) -> Result<crate::provider::ChatStream, ProviderError> {
                Err(ProviderError::NotFound("Not implemented".to_string()))
            }

            fn count_tokens(&self, _content: &str, _model: &str) -> Result<usize, ProviderError> {
                Ok(0)
            }

            async fn health_check(&self) -> Result<bool, ProviderError> {
                tokio::time::sleep(Duration::from_secs(10)).await;
                Ok(true)
            }
        }

        let provider: Arc<dyn Provider> = Arc::new(SlowProvider);
        let result = cache.check_health(&provider).await;
        assert!(result.is_err());
    }
}