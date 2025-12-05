//! Integration tests for multi-provider fallback
//! Tests fallback from primary to secondary provider and error handling

use ricecoder_providers::{
    models::{FinishReason, Message},
    provider::{ChatStream, ProviderManager, ProviderRegistry},
    ChatRequest, ChatResponse, ModelInfo, Provider, ProviderError, TokenUsage,
};
use std::sync::Arc;

/// Mock provider that fails on first call, succeeds on second
struct FailThenSucceedProvider {
    id: String,
    call_count: std::sync::atomic::AtomicUsize,
}

#[async_trait::async_trait]
impl Provider for FailThenSucceedProvider {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        "Fail Then Succeed"
    }

    fn models(&self) -> Vec<ModelInfo> {
        vec![]
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, ProviderError> {
        let count = self
            .call_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if count == 0 {
            Err(ProviderError::ProviderError(
                "Temporary failure".to_string(),
            ))
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
        Ok(true)
    }
}

/// Mock provider that always fails
struct AlwaysFailProvider {
    id: String,
}

#[async_trait::async_trait]
impl Provider for AlwaysFailProvider {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        "Always Fail"
    }

    fn models(&self) -> Vec<ModelInfo> {
        vec![]
    }

    async fn chat(&self, _request: ChatRequest) -> Result<ChatResponse, ProviderError> {
        Err(ProviderError::ProviderError(
            "Provider unavailable".to_string(),
        ))
    }

    async fn chat_stream(&self, _request: ChatRequest) -> Result<ChatStream, ProviderError> {
        Err(ProviderError::NotFound("Not implemented".to_string()))
    }

    fn count_tokens(&self, content: &str, _model: &str) -> Result<usize, ProviderError> {
        Ok((content.len() + 3) / 4)
    }

    async fn health_check(&self) -> Result<bool, ProviderError> {
        Ok(false)
    }
}

/// Mock provider that always succeeds
struct AlwaysSucceedProvider {
    id: String,
}

#[async_trait::async_trait]
impl Provider for AlwaysSucceedProvider {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        "Always Succeed"
    }

    fn models(&self) -> Vec<ModelInfo> {
        vec![]
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, ProviderError> {
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

    async fn chat_stream(&self, _request: ChatRequest) -> Result<ChatStream, ProviderError> {
        Err(ProviderError::NotFound("Not implemented".to_string()))
    }

    fn count_tokens(&self, content: &str, _model: &str) -> Result<usize, ProviderError> {
        Ok((content.len() + 3) / 4)
    }

    async fn health_check(&self) -> Result<bool, ProviderError> {
        Ok(true)
    }
}

/// Test: Fallback from primary to secondary provider on failure
#[tokio::test]
async fn test_fallback_from_primary_to_secondary() {
    let mut registry = ProviderRegistry::new();

    // Register primary provider that fails
    let primary: Arc<dyn Provider> = Arc::new(AlwaysFailProvider {
        id: "primary".to_string(),
    });
    registry.register(primary.clone()).unwrap();

    // Register secondary provider that succeeds
    let secondary: Arc<dyn Provider> = Arc::new(AlwaysSucceedProvider {
        id: "secondary".to_string(),
    });
    registry.register(secondary.clone()).unwrap();

    // Create manager with primary as default
    let manager = ProviderManager::new(registry, "primary".to_string());

    // Try to chat with primary (should fail)
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

    let result = manager.chat(request.clone()).await;
    assert!(result.is_err(), "Primary provider should fail");

    // Now try with secondary provider directly
    let result = manager.chat_with_provider(&secondary, request).await;
    assert!(result.is_ok(), "Secondary provider should succeed");
    assert_eq!(result.unwrap().content, "Response from secondary");
}

/// Test: Fallback chain with multiple providers
#[tokio::test]
async fn test_fallback_chain_multiple_providers() {
    let mut registry = ProviderRegistry::new();

    // Register three providers: fail, fail, succeed
    let provider1: Arc<dyn Provider> = Arc::new(AlwaysFailProvider {
        id: "provider1".to_string(),
    });
    registry.register(provider1.clone()).unwrap();

    let provider2: Arc<dyn Provider> = Arc::new(AlwaysFailProvider {
        id: "provider2".to_string(),
    });
    registry.register(provider2.clone()).unwrap();

    let provider3: Arc<dyn Provider> = Arc::new(AlwaysSucceedProvider {
        id: "provider3".to_string(),
    });
    registry.register(provider3.clone()).unwrap();

    let manager = ProviderManager::new(registry, "provider1".to_string());

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

    // Provider 1 should fail
    let result = manager
        .chat_with_provider(&provider1, request.clone())
        .await;
    assert!(result.is_err());

    // Provider 2 should fail
    let result = manager
        .chat_with_provider(&provider2, request.clone())
        .await;
    assert!(result.is_err());

    // Provider 3 should succeed
    let result = manager
        .chat_with_provider(&provider3, request.clone())
        .await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().content, "Response from provider3");
}

/// Test: Retry logic with fail-then-succeed provider
#[tokio::test]
async fn test_retry_logic_with_fail_then_succeed() {
    let mut registry = ProviderRegistry::new();

    let provider = Arc::new(FailThenSucceedProvider {
        id: "retry-provider".to_string(),
        call_count: std::sync::atomic::AtomicUsize::new(0),
    });
    registry.register(provider.clone()).unwrap();

    let manager = ProviderManager::new(registry, "retry-provider".to_string()).with_retry_count(3);

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

    // First call should fail, but retry should succeed
    let result = manager.chat(request).await;
    assert!(result.is_ok(), "Should succeed after retry");
    assert_eq!(result.unwrap().content, "Response from retry-provider");
}

/// Test: Error handling in fallback scenarios
#[tokio::test]
async fn test_error_handling_in_fallback() {
    let mut registry = ProviderRegistry::new();

    let provider = Arc::new(AlwaysFailProvider {
        id: "failing-provider".to_string(),
    });
    registry.register(provider.clone()).unwrap();

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

    // Should return error after all retries
    let result = manager.chat(request).await;
    assert!(result.is_err(), "Should fail after all retries");

    match result {
        Err(ProviderError::ProviderError(msg)) => {
            assert!(msg.contains("Provider unavailable") || msg.contains("Failed after retries"));
        }
        _ => panic!("Expected ProviderError"),
    }
}

/// Test: Provider registry with multiple providers
#[tokio::test]
async fn test_provider_registry_with_multiple_providers() {
    let mut registry = ProviderRegistry::new();

    let provider1 = Arc::new(AlwaysSucceedProvider {
        id: "provider1".to_string(),
    });
    registry.register(provider1.clone()).unwrap();

    let provider2 = Arc::new(AlwaysSucceedProvider {
        id: "provider2".to_string(),
    });
    registry.register(provider2.clone()).unwrap();

    let provider3 = Arc::new(AlwaysSucceedProvider {
        id: "provider3".to_string(),
    });
    registry.register(provider3.clone()).unwrap();

    // Verify all providers are registered
    assert_eq!(registry.provider_count(), 3);
    assert!(registry.has_provider("provider1"));
    assert!(registry.has_provider("provider2"));
    assert!(registry.has_provider("provider3"));

    // Verify we can retrieve each provider
    assert!(registry.get("provider1").is_ok());
    assert!(registry.get("provider2").is_ok());
    assert!(registry.get("provider3").is_ok());

    // Verify list_all returns all providers
    let all_providers = registry.list_all();
    assert_eq!(all_providers.len(), 3);
}

/// Test: Fallback when primary provider is not found
#[tokio::test]
async fn test_fallback_when_primary_not_found() {
    let mut registry = ProviderRegistry::new();

    let provider = Arc::new(AlwaysSucceedProvider {
        id: "available-provider".to_string(),
    });
    registry.register(provider).unwrap();

    // Try to create manager with non-existent default provider
    let manager = ProviderManager::new(registry, "non-existent".to_string());

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

    // Should fail because default provider doesn't exist
    let result = manager.chat(request).await;
    assert!(result.is_err());

    match result {
        Err(ProviderError::NotFound(msg)) => {
            assert!(msg.contains("non-existent"));
        }
        _ => panic!("Expected NotFound error"),
    }
}
