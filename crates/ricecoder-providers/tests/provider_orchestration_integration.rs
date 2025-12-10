//! Integration tests for provider orchestration with mock providers
//! Tests provider registry, manager orchestration, and error handling

use ricecoder_providers::{
    models::{FinishReason, Message},
    provider::{ChatStream, ProviderManager, ProviderRegistry},
    ChatRequest, ChatResponse, ModelInfo, Provider, ProviderError, TokenUsage,
};
use std::sync::Arc;

/// Mock provider for testing
struct TestProvider {
    id: String,
    name: String,
    should_fail: bool,
}

#[async_trait::async_trait]
impl Provider for TestProvider {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn models(&self) -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                id: "model-1".to_string(),
                name: "Model 1".to_string(),
                provider: self.id.clone(),
                context_window: 4096,
                capabilities: vec![],
                pricing: None,
                is_free: true,
            },
            ModelInfo {
                id: "model-2".to_string(),
                name: "Model 2".to_string(),
                provider: self.id.clone(),
                context_window: 8192,
                capabilities: vec![],
                pricing: None,
                is_free: true,
            },
        ]
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, ProviderError> {
        if self.should_fail {
            Err(ProviderError::ProviderError("Test failure".to_string()))
        } else {
            Ok(ChatResponse {
                content: format!("Response from {}", self.id),
                model: request.model,
                usage: TokenUsage {
                    prompt_tokens: 10,
                    completion_tokens: 5,
                    total_tokens: 15,
                },
                finish_reason: FinishReason::Stop,
            })
        }
    }

    async fn chat_stream(&self, _request: ChatRequest) -> Result<ChatStream, ProviderError> {
        Err(ProviderError::NotFound("Not implemented".to_string()))
    }

    fn count_tokens(&self, content: &str, _model: &str) -> Result<usize, ProviderError> {
        Ok((content.len() + 3) / 4)
    }

    async fn health_check(&self) -> Result<bool, ProviderError> {
        Ok(!self.should_fail)
    }
}

/// Test: Provider registry with multiple providers
#[test]
fn test_provider_registry_with_multiple_providers() {
    let mut registry = ProviderRegistry::new();

    let provider1 = Arc::new(TestProvider {
        id: "provider1".to_string(),
        name: "Provider 1".to_string(),
        should_fail: false,
    });
    registry.register(provider1.clone()).unwrap();

    let provider2 = Arc::new(TestProvider {
        id: "provider2".to_string(),
        name: "Provider 2".to_string(),
        should_fail: false,
    });
    registry.register(provider2.clone()).unwrap();

    let provider3 = Arc::new(TestProvider {
        id: "provider3".to_string(),
        name: "Provider 3".to_string(),
        should_fail: false,
    });
    registry.register(provider3.clone()).unwrap();

    // Verify all providers are registered
    assert_eq!(registry.provider_count(), 3);
    assert!(registry.has_provider("provider1"));
    assert!(registry.has_provider("provider2"));
    assert!(registry.has_provider("provider3"));
}

/// Test: Provider registry retrieval by ID
#[test]
fn test_provider_registry_retrieval_by_id() {
    let mut registry = ProviderRegistry::new();

    let provider = Arc::new(TestProvider {
        id: "test-provider".to_string(),
        name: "Test Provider".to_string(),
        should_fail: false,
    });
    registry.register(provider.clone()).unwrap();

    let retrieved = registry.get("test-provider");
    assert!(retrieved.is_ok());
    assert_eq!(retrieved.unwrap().id(), "test-provider");
}

/// Test: Provider registry retrieval by name
#[test]
fn test_provider_registry_retrieval_by_name() {
    let mut registry = ProviderRegistry::new();

    let provider = Arc::new(TestProvider {
        id: "test-provider".to_string(),
        name: "Test Provider".to_string(),
        should_fail: false,
    });
    registry.register(provider.clone()).unwrap();

    let retrieved = registry.get_by_name("Test Provider");
    assert!(retrieved.is_ok());
    assert_eq!(retrieved.unwrap().name(), "Test Provider");
}

/// Test: Provider registry list all providers
#[test]
fn test_provider_registry_list_all() {
    let mut registry = ProviderRegistry::new();

    for i in 1..=5 {
        let provider = Arc::new(TestProvider {
            id: format!("provider{}", i),
            name: format!("Provider {}", i),
            should_fail: false,
        });
        registry.register(provider).unwrap();
    }

    let all = registry.list_all();
    assert_eq!(all.len(), 5);
}

/// Test: Provider registry list all models
#[test]
fn test_provider_registry_list_all_models() {
    let mut registry = ProviderRegistry::new();

    let provider1 = Arc::new(TestProvider {
        id: "provider1".to_string(),
        name: "Provider 1".to_string(),
        should_fail: false,
    });
    registry.register(provider1).unwrap();

    let provider2 = Arc::new(TestProvider {
        id: "provider2".to_string(),
        name: "Provider 2".to_string(),
        should_fail: false,
    });
    registry.register(provider2).unwrap();

    let all_models = registry.list_all_models();
    // Each provider has 2 models, so total should be 4
    assert_eq!(all_models.len(), 4);
}

/// Test: Provider registry list models for specific provider
#[test]
fn test_provider_registry_list_models_for_provider() {
    let mut registry = ProviderRegistry::new();

    let provider = Arc::new(TestProvider {
        id: "test-provider".to_string(),
        name: "Test Provider".to_string(),
        should_fail: false,
    });
    registry.register(provider).unwrap();

    let models = registry.list_models("test-provider");
    assert!(models.is_ok());
    assert_eq!(models.unwrap().len(), 2);
}

/// Test: Provider registry unregister provider
#[test]
fn test_provider_registry_unregister() {
    let mut registry = ProviderRegistry::new();

    let provider = Arc::new(TestProvider {
        id: "test-provider".to_string(),
        name: "Test Provider".to_string(),
        should_fail: false,
    });
    registry.register(provider).unwrap();

    assert!(registry.has_provider("test-provider"));

    registry.unregister("test-provider").unwrap();

    assert!(!registry.has_provider("test-provider"));
}

/// Test: Provider manager orchestration
#[tokio::test]
async fn test_provider_manager_orchestration() {
    let mut registry = ProviderRegistry::new();

    let provider = Arc::new(TestProvider {
        id: "test-provider".to_string(),
        name: "Test Provider".to_string(),
        should_fail: false,
    });
    registry.register(provider).unwrap();

    let manager = ProviderManager::new(registry, "test-provider".to_string());

    let request = ChatRequest {
        model: "test-model".to_string(),
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
    assert_eq!(response.unwrap().content, "Response from test-provider");
}

/// Test: Provider manager with multiple providers
#[tokio::test]
async fn test_provider_manager_with_multiple_providers() {
    let mut registry = ProviderRegistry::new();

    for i in 1..=3 {
        let provider = Arc::new(TestProvider {
            id: format!("provider{}", i),
            name: format!("Provider {}", i),
            should_fail: false,
        });
        registry.register(provider).unwrap();
    }

    let manager = ProviderManager::new(registry, "provider1".to_string());

    // Verify we can get each provider
    assert!(manager.get_provider("provider1").is_ok());
    assert!(manager.get_provider("provider2").is_ok());
    assert!(manager.get_provider("provider3").is_ok());
}

/// Test: Provider manager error handling
#[tokio::test]
async fn test_provider_manager_error_handling() {
    let mut registry = ProviderRegistry::new();

    let provider = Arc::new(TestProvider {
        id: "failing-provider".to_string(),
        name: "Failing Provider".to_string(),
        should_fail: true,
    });
    registry.register(provider).unwrap();

    let manager = ProviderManager::new(registry, "failing-provider".to_string());

    let request = ChatRequest {
        model: "test-model".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: "Hello".to_string(),
        }],
        temperature: None,
        max_tokens: None,
        stream: false,
    };

    let response = manager.chat(request).await;
    assert!(response.is_err());
}

/// Test: Provider manager health check
#[tokio::test]
async fn test_provider_manager_health_check() {
    let mut registry = ProviderRegistry::new();

    let provider = Arc::new(TestProvider {
        id: "test-provider".to_string(),
        name: "Test Provider".to_string(),
        should_fail: false,
    });
    registry.register(provider).unwrap();

    let manager = ProviderManager::new(registry, "test-provider".to_string());

    let health = manager.health_check("test-provider").await;
    assert!(health.is_ok());
    assert!(health.unwrap());
}

/// Test: Provider manager health check all
#[tokio::test]
async fn test_provider_manager_health_check_all() {
    let mut registry = ProviderRegistry::new();

    for i in 1..=3 {
        let provider = Arc::new(TestProvider {
            id: format!("provider{}", i),
            name: format!("Provider {}", i),
            should_fail: false,
        });
        registry.register(provider).unwrap();
    }

    let manager = ProviderManager::new(registry, "provider1".to_string());

    let health_results = manager.health_check_all().await;
    assert_eq!(health_results.len(), 3);

    for (_, health) in health_results {
        assert!(health.is_ok());
        assert!(health.unwrap());
    }
}

/// Test: Provider manager invalidate health check
#[tokio::test]
async fn test_provider_manager_invalidate_health_check() {
    let mut registry = ProviderRegistry::new();

    let provider = Arc::new(TestProvider {
        id: "test-provider".to_string(),
        name: "Test Provider".to_string(),
        should_fail: false,
    });
    registry.register(provider).unwrap();

    let manager = ProviderManager::new(registry, "test-provider".to_string());

    // Check health
    let health1 = manager.health_check("test-provider").await;
    assert!(health1.is_ok());

    // Invalidate cache
    manager.invalidate_health_check("test-provider").await;

    // Check health again (should be fresh)
    let health2 = manager.health_check("test-provider").await;
    assert!(health2.is_ok());
}

/// Test: Provider manager with custom retry count
#[tokio::test]
async fn test_provider_manager_with_custom_retry_count() {
    let mut registry = ProviderRegistry::new();

    let provider = Arc::new(TestProvider {
        id: "test-provider".to_string(),
        name: "Test Provider".to_string(),
        should_fail: false,
    });
    registry.register(provider).unwrap();

    let manager = ProviderManager::new(registry, "test-provider".to_string()).with_retry_count(5);

    let request = ChatRequest {
        model: "test-model".to_string(),
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

/// Test: Provider manager with custom timeout
#[tokio::test]
async fn test_provider_manager_with_custom_timeout() {
    use std::time::Duration;

    let mut registry = ProviderRegistry::new();

    let provider = Arc::new(TestProvider {
        id: "test-provider".to_string(),
        name: "Test Provider".to_string(),
        should_fail: false,
    });
    registry.register(provider).unwrap();

    let manager = ProviderManager::new(registry, "test-provider".to_string())
        .with_timeout(Duration::from_secs(60));

    let request = ChatRequest {
        model: "test-model".to_string(),
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

/// Test: Provider manager get default provider
#[tokio::test]
async fn test_provider_manager_get_default_provider() {
    let mut registry = ProviderRegistry::new();

    let provider = Arc::new(TestProvider {
        id: "default-provider".to_string(),
        name: "Default Provider".to_string(),
        should_fail: false,
    });
    registry.register(provider).unwrap();

    let manager = ProviderManager::new(registry, "default-provider".to_string());

    let default = manager.default_provider();
    assert!(default.is_ok());
    assert_eq!(default.unwrap().id(), "default-provider");
}

/// Test: Provider manager get specific provider
#[tokio::test]
async fn test_provider_manager_get_specific_provider() {
    let mut registry = ProviderRegistry::new();

    let provider1 = Arc::new(TestProvider {
        id: "provider1".to_string(),
        name: "Provider 1".to_string(),
        should_fail: false,
    });
    registry.register(provider1).unwrap();

    let provider2 = Arc::new(TestProvider {
        id: "provider2".to_string(),
        name: "Provider 2".to_string(),
        should_fail: false,
    });
    registry.register(provider2).unwrap();

    let manager = ProviderManager::new(registry, "provider1".to_string());

    let specific = manager.get_provider("provider2");
    assert!(specific.is_ok());
    assert_eq!(specific.unwrap().id(), "provider2");
}

/// Test: Provider manager registry access
#[test]
fn test_provider_manager_registry_access() {
    let mut registry = ProviderRegistry::new();

    let provider = Arc::new(TestProvider {
        id: "test-provider".to_string(),
        name: "Test Provider".to_string(),
        should_fail: false,
    });
    registry.register(provider).unwrap();

    let manager = ProviderManager::new(registry, "test-provider".to_string());

    let registry_ref = manager.registry();
    assert_eq!(registry_ref.provider_count(), 1);
}

/// Test: Provider manager error on non-existent provider
#[tokio::test]
async fn test_provider_manager_error_on_non_existent_provider() {
    let registry = ProviderRegistry::new();

    let manager = ProviderManager::new(registry, "non-existent".to_string());

    let result = manager.default_provider();
    assert!(result.is_err());
}

/// Test: Provider manager error on non-existent specific provider
#[tokio::test]
async fn test_provider_manager_error_on_non_existent_specific_provider() {
    let mut registry = ProviderRegistry::new();

    let provider = Arc::new(TestProvider {
        id: "existing".to_string(),
        name: "Existing".to_string(),
        should_fail: false,
    });
    registry.register(provider).unwrap();

    let manager = ProviderManager::new(registry, "existing".to_string());

    let result = manager.get_provider("non-existent");
    assert!(result.is_err());
}
