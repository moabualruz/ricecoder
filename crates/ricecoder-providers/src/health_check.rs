//! Health check system with caching and timeout support

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, warn};

use crate::error::ProviderError;
use crate::provider::Provider;

/// Health check result with timestamp
#[derive(Clone, Debug)]
pub struct HealthCheckResult {
    /// Whether the provider is healthy
    pub is_healthy: bool,
    /// When the check was performed
    pub checked_at: Instant,
    /// Error if the check failed
    pub error: Option<String>,
}

impl HealthCheckResult {
    /// Check if the result is still valid (not expired)
    pub fn is_valid(&self, ttl: Duration) -> bool {
        self.checked_at.elapsed() < ttl
    }
}

/// Health check cache for providers
pub struct HealthCheckCache {
    /// Cache of health check results by provider ID
    cache: Arc<RwLock<HashMap<String, HealthCheckResult>>>,
    /// Time-to-live for cached results
    ttl: Duration,
    /// Timeout for health check operations
    timeout: Duration,
}

impl HealthCheckCache {
    /// Create a new health check cache
    pub fn new(ttl: Duration, timeout: Duration) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl,
            timeout,
        }
    }
}

impl Default for HealthCheckCache {
    /// Create a new health check cache with default settings
    /// - TTL: 5 minutes
    /// - Timeout: 10 seconds
    fn default() -> Self {
        Self::new(Duration::from_secs(300), Duration::from_secs(10))
    }
}

impl HealthCheckCache {
    /// Check provider health with caching
    pub async fn check_health(&self, provider: &Arc<dyn Provider>) -> Result<bool, ProviderError> {
        let provider_id = provider.id();

        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(result) = cache.get(provider_id) {
                if result.is_valid(self.ttl) {
                    debug!(
                        "Using cached health check for provider: {} (healthy: {})",
                        provider_id, result.is_healthy
                    );
                    return if result.is_healthy {
                        Ok(true)
                    } else {
                        Err(ProviderError::ProviderError(
                            result
                                .error
                                .clone()
                                .unwrap_or_else(|| "Provider unhealthy".to_string()),
                        ))
                    };
                }
            }
        }

        // Perform health check with timeout
        debug!("Performing health check for provider: {}", provider_id);
        let result = match tokio::time::timeout(self.timeout, provider.health_check()).await {
            Ok(Ok(is_healthy)) => HealthCheckResult {
                is_healthy,
                checked_at: Instant::now(),
                error: None,
            },
            Ok(Err(e)) => {
                warn!("Health check failed for provider {}: {}", provider_id, e);
                HealthCheckResult {
                    is_healthy: false,
                    checked_at: Instant::now(),
                    error: Some(e.to_string()),
                }
            }
            Err(_) => {
                warn!("Health check timeout for provider: {}", provider_id);
                HealthCheckResult {
                    is_healthy: false,
                    checked_at: Instant::now(),
                    error: Some("Health check timeout".to_string()),
                }
            }
        };

        // Cache the result
        {
            let mut cache = self.cache.write().await;
            cache.insert(provider_id.to_string(), result.clone());
        }

        if result.is_healthy {
            Ok(true)
        } else {
            Err(ProviderError::ProviderError(
                result
                    .error
                    .unwrap_or_else(|| "Provider unhealthy".to_string()),
            ))
        }
    }

    /// Invalidate cache for a specific provider
    pub async fn invalidate(&self, provider_id: &str) {
        let mut cache = self.cache.write().await;
        cache.remove(provider_id);
        debug!(
            "Invalidated health check cache for provider: {}",
            provider_id
        );
    }

    /// Invalidate all cached results
    pub async fn invalidate_all(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        debug!("Invalidated all health check cache");
    }

    /// Get cached result without performing a new check
    pub async fn get_cached(&self, provider_id: &str) -> Option<HealthCheckResult> {
        let cache = self.cache.read().await;
        cache.get(provider_id).cloned()
    }

    /// Set TTL for cached results
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = ttl;
        self
    }

    /// Set timeout for health check operations
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

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
