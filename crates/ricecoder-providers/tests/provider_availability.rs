//! Property-based tests for provider availability
//! **Feature: ricecoder-providers, Property 6: Provider Availability**
//! **Validates: Requirements 1.1**

use ricecoder_providers::models::{FinishReason, Message};
use ricecoder_providers::{
    ChatRequest, ChatResponse, HealthCheckCache, ModelInfo, Provider, ProviderError,
    ProviderManager, ProviderRegistry, TokenUsage,
};
use std::sync::Arc;
use std::time::Duration;

/// Mock provider that simulates availability
struct AvailableMockProvider {
    id: String,
    available: bool,
}

#[async_trait::async_trait]
impl Provider for AvailableMockProvider {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        "Mock Provider"
    }

    fn models(&self) -> Vec<ModelInfo> {
        vec![]
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, ProviderError> {
        if self.available {
            Ok(ChatResponse {
                content: "Response".to_string(),
                model: request.model,
                usage: TokenUsage {
                    prompt_tokens: 10,
                    completion_tokens: 5,
                    total_tokens: 15,
                },
                finish_reason: FinishReason::Stop,
            })
        } else {
            Err(ProviderError::ProviderError(
                "Provider unavailable".to_string(),
            ))
        }
    }

    async fn chat_stream(
        &self,
        _request: ChatRequest,
    ) -> Result<ricecoder_providers::provider::ChatStream, ProviderError> {
        Err(ProviderError::NotFound("Not implemented".to_string()))
    }

    fn count_tokens(&self, content: &str, _model: &str) -> Result<usize, ProviderError> {
        Ok((content.len() + 3) / 4)
    }

    async fn health_check(&self) -> Result<bool, ProviderError> {
        if self.available {
            Ok(true)
        } else {
            Err(ProviderError::ProviderError("Provider is down".to_string()))
        }
    }
}

/// Property: System verifies provider availability via health check before use
/// For any configured provider, the system SHALL verify availability via health check before use.
/// This means:
/// 1. Before using a provider, health_check() must be called
/// 2. If health_check() returns Ok(true), the provider is available
/// 3. If health_check() returns Ok(false) or Err, the provider is unavailable
/// 4. The system should not attempt to use an unavailable provider
#[tokio::test]
async fn prop_provider_availability_verified_before_use() {
    // Create a provider manager with a health check cache
    let mut registry = ProviderRegistry::new();
    let provider = Arc::new(AvailableMockProvider {
        id: "test-provider".to_string(),
        available: true,
    });
    registry.register(provider).unwrap();

    let manager = ProviderManager::new(registry, "test-provider".to_string());

    // Verify health check is performed
    let health = manager.health_check("test-provider").await;
    assert!(health.is_ok());
    assert!(health.unwrap());

    // Verify the provider can be used after health check passes
    let request = ChatRequest {
        model: "test".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: "Hello".to_string(),
        }],
        temperature: None,
        max_tokens: None,
        stream: false,
    };

    let response = manager.chat(request).await;
    assert!(response.is_ok());
}

/// Property: Unavailable providers are detected via health check
/// For any provider that is unavailable, health_check() should return an error or Ok(false)
#[tokio::test]
async fn prop_unavailable_provider_detected() {
    let mut registry = ProviderRegistry::new();
    let provider = Arc::new(AvailableMockProvider {
        id: "unavailable-provider".to_string(),
        available: false,
    });
    registry.register(provider).unwrap();

    let manager = ProviderManager::new(registry, "unavailable-provider".to_string());

    // Verify health check detects unavailability
    let health = manager.health_check("unavailable-provider").await;
    assert!(health.is_err());
}

/// Property: Health check results are cached
/// For any provider, calling health_check() multiple times within the TTL should use cached results
#[tokio::test]
async fn prop_health_check_caching() {
    let cache = HealthCheckCache::new(Duration::from_secs(60), Duration::from_secs(10));
    let provider: Arc<dyn Provider> = Arc::new(AvailableMockProvider {
        id: "cached-provider".to_string(),
        available: true,
    });

    // First check
    let result1 = cache.check_health(&provider).await;
    assert!(result1.is_ok());

    // Second check should use cache (no network call)
    let result2 = cache.check_health(&provider).await;
    assert!(result2.is_ok());

    // Verify cache entry exists
    let cached = cache.get_cached("cached-provider").await;
    assert!(cached.is_some());
    assert!(cached.unwrap().is_healthy);
}

/// Property: Health check timeout is enforced
/// For any provider that takes too long to respond, health_check() should timeout
#[tokio::test]
async fn prop_health_check_timeout_enforced() {
    let cache = HealthCheckCache::new(Duration::from_secs(60), Duration::from_millis(100));

    struct SlowProvider;

    #[async_trait::async_trait]
    impl Provider for SlowProvider {
        fn id(&self) -> &str {
            "slow"
        }

        fn name(&self) -> &str {
            "Slow"
        }

        fn models(&self) -> Vec<ModelInfo> {
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
        ) -> Result<ricecoder_providers::provider::ChatStream, ProviderError> {
            Err(ProviderError::NotFound("Not implemented".to_string()))
        }

        fn count_tokens(&self, _content: &str, _model: &str) -> Result<usize, ProviderError> {
            Ok(0)
        }

        async fn health_check(&self) -> Result<bool, ProviderError> {
            // Simulate a slow health check
            tokio::time::sleep(Duration::from_secs(5)).await;
            Ok(true)
        }
    }

    let provider: Arc<dyn Provider> = Arc::new(SlowProvider);
    let result = cache.check_health(&provider).await;

    // Should timeout and return an error
    assert!(result.is_err());
}

/// Property: Health check cache can be invalidated
/// For any cached health check result, calling invalidate() should remove it from cache
#[tokio::test]
async fn prop_health_check_cache_invalidation() {
    let cache = HealthCheckCache::default();
    let provider: Arc<dyn Provider> = Arc::new(AvailableMockProvider {
        id: "invalidate-test".to_string(),
        available: true,
    });

    // Perform health check to populate cache
    cache.check_health(&provider).await.ok();

    // Verify cache entry exists
    let cached = cache.get_cached("invalidate-test").await;
    assert!(cached.is_some());

    // Invalidate the cache
    cache.invalidate("invalidate-test").await;

    // Verify cache entry is gone
    let cached = cache.get_cached("invalidate-test").await;
    assert!(cached.is_none());
}

/// Property: Health check cache TTL is respected
/// For any cached health check result, if TTL expires, the cache should be considered invalid
#[tokio::test]
async fn prop_health_check_cache_ttl_respected() {
    let cache = HealthCheckCache::new(Duration::from_millis(100), Duration::from_secs(10));
    let provider: Arc<dyn Provider> = Arc::new(AvailableMockProvider {
        id: "ttl-test".to_string(),
        available: true,
    });

    // Perform health check to populate cache
    cache.check_health(&provider).await.ok();

    // Verify cache entry exists and is valid
    let cached = cache.get_cached("ttl-test").await;
    assert!(cached.is_some());
    assert!(cached.unwrap().is_valid(Duration::from_secs(1)));

    // Wait for TTL to expire
    tokio::time::sleep(Duration::from_millis(150)).await;

    // Verify cache entry is still there but expired
    let cached = cache.get_cached("ttl-test").await;
    assert!(cached.is_some());
    assert!(!cached.unwrap().is_valid(Duration::from_millis(100)));
}

/// Property: Multiple providers can be checked for availability
/// For any set of providers, health_check_all() should check all of them
#[tokio::test]
async fn prop_multiple_providers_availability_check() {
    let mut registry = ProviderRegistry::new();

    let provider1 = Arc::new(AvailableMockProvider {
        id: "provider1".to_string(),
        available: true,
    });
    let provider2 = Arc::new(AvailableMockProvider {
        id: "provider2".to_string(),
        available: false,
    });

    registry.register(provider1).unwrap();
    registry.register(provider2).unwrap();

    let manager = ProviderManager::new(registry, "provider1".to_string());

    // Check all providers
    let results = manager.health_check_all().await;

    // Should have results for both providers
    assert_eq!(results.len(), 2);

    // Find results for each provider
    let provider1_result = results.iter().find(|(id, _)| id == "provider1");
    let provider2_result = results.iter().find(|(id, _)| id == "provider2");

    assert!(provider1_result.is_some());
    assert!(provider2_result.is_some());

    // Provider1 should be healthy
    assert!(provider1_result.unwrap().1.is_ok());
    assert!(provider1_result.unwrap().1.as_ref().unwrap());

    // Provider2 should be unhealthy
    assert!(provider2_result.unwrap().1.is_err());
}
