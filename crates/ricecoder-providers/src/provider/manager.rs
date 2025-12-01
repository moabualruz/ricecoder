//! Provider manager for orchestrating provider operations

use std::sync::Arc;
use std::time::Duration;

use crate::error::ProviderError;
use crate::health_check::HealthCheckCache;
use crate::models::{ChatRequest, ChatResponse};
use super::{Provider, ProviderRegistry, ChatStream};

/// Central coordinator for provider operations
pub struct ProviderManager {
    registry: ProviderRegistry,
    default_provider_id: String,
    retry_count: usize,
    timeout: Duration,
    health_check_cache: Arc<HealthCheckCache>,
}

impl ProviderManager {
    /// Create a new provider manager
    pub fn new(registry: ProviderRegistry, default_provider_id: String) -> Self {
        Self {
            registry,
            default_provider_id,
            retry_count: 3,
            timeout: Duration::from_secs(30),
            health_check_cache: Arc::new(HealthCheckCache::default()),
        }
    }

    /// Set the number of retries for failed requests
    pub fn with_retry_count(mut self, count: usize) -> Self {
        self.retry_count = count;
        self
    }

    /// Set the request timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the health check cache
    pub fn with_health_check_cache(mut self, cache: Arc<HealthCheckCache>) -> Self {
        self.health_check_cache = cache;
        self
    }

    /// Get the default provider
    pub fn default_provider(&self) -> Result<Arc<dyn Provider>, ProviderError> {
        self.registry.get(&self.default_provider_id)
    }

    /// Get a specific provider
    pub fn get_provider(&self, provider_id: &str) -> Result<Arc<dyn Provider>, ProviderError> {
        self.registry.get(provider_id)
    }

    /// Send a chat request with retry logic
    pub async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, ProviderError> {
        let provider = self.default_provider()?;
        self.chat_with_provider(&provider, request).await
    }

    /// Send a chat request to a specific provider with retry logic
    pub async fn chat_with_provider(
        &self,
        provider: &Arc<dyn Provider>,
        request: ChatRequest,
    ) -> Result<ChatResponse, ProviderError> {
        let mut last_error = None;

        for attempt in 0..=self.retry_count {
            match tokio::time::timeout(self.timeout, provider.chat(request.clone())).await {
                Ok(Ok(response)) => return Ok(response),
                Ok(Err(e)) => {
                    last_error = Some(e);
                    if attempt < self.retry_count {
                        // Exponential backoff
                        let backoff = Duration::from_millis(100 * 2_u64.pow(attempt as u32));
                        tokio::time::sleep(backoff).await;
                    }
                }
                Err(_) => {
                    last_error = Some(ProviderError::ProviderError("Request timeout".to_string()));
                    if attempt < self.retry_count {
                        let backoff = Duration::from_millis(100 * 2_u64.pow(attempt as u32));
                        tokio::time::sleep(backoff).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            ProviderError::ProviderError("Failed after retries".to_string())
        }))
    }

    /// Stream a chat response
    pub async fn chat_stream(&self, request: ChatRequest) -> Result<ChatStream, ProviderError> {
        let provider = self.default_provider()?;
        provider.chat_stream(request).await
    }

    /// Stream a chat response from a specific provider
    pub async fn chat_stream_with_provider(
        &self,
        provider: &Arc<dyn Provider>,
        request: ChatRequest,
    ) -> Result<ChatStream, ProviderError> {
        provider.chat_stream(request).await
    }

    /// Check provider health with caching
    pub async fn health_check(&self, provider_id: &str) -> Result<bool, ProviderError> {
        let provider = self.registry.get(provider_id)?;
        self.health_check_cache.check_health(&provider).await
    }

    /// Check health of all providers with caching
    pub async fn health_check_all(&self) -> Vec<(String, Result<bool, ProviderError>)> {
        let mut results = Vec::new();

        for provider in self.registry.list_all() {
            let id = provider.id().to_string();
            let health = self.health_check_cache.check_health(&provider).await;
            results.push((id, health));
        }

        results
    }

    /// Invalidate health check cache for a provider
    pub async fn invalidate_health_check(&self, provider_id: &str) {
        self.health_check_cache.invalidate(provider_id).await;
    }

    /// Invalidate all health check cache
    pub async fn invalidate_all_health_checks(&self) {
        self.health_check_cache.invalidate_all().await;
    }

    /// Get the health check cache
    pub fn health_check_cache(&self) -> &Arc<HealthCheckCache> {
        &self.health_check_cache
    }

    /// Get the registry
    pub fn registry(&self) -> &ProviderRegistry {
        &self.registry
    }

    /// Get mutable registry
    pub fn registry_mut(&mut self) -> &mut ProviderRegistry {
        &mut self.registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ChatResponse, FinishReason, TokenUsage};

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

        async fn chat(
            &self,
            _request: ChatRequest,
        ) -> Result<ChatResponse, ProviderError> {
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

        async fn chat_stream(
            &self,
            _request: ChatRequest,
        ) -> Result<ChatStream, ProviderError> {
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
