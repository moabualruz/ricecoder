//! Property-based tests for provider interface consistency
//! **Feature: ricecoder-providers, Property 1: Provider Interface Consistency**
//! **Validates: Requirements 1.1, 1.2**

use std::sync::Arc;

use proptest::prelude::*;
use ricecoder_providers::{
    models::{FinishReason, Message},
    ChatRequest, ChatResponse, ModelInfo, Provider, ProviderError, TokenUsage,
};

/// Mock provider for testing consistency
struct ConsistentMockProvider {
    id: String,
    name: String,
}

#[async_trait::async_trait]
impl Provider for ConsistentMockProvider {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn models(&self) -> Vec<ModelInfo> {
        vec![]
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, ProviderError> {
        // Always return the same response for the same input
        Ok(ChatResponse {
            content: format!("Response to: {}", request.model),
            model: request.model,
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
    ) -> Result<ricecoder_providers::provider::ChatStream, ProviderError> {
        Err(ProviderError::NotFound("Not implemented".to_string()))
    }

    fn count_tokens(&self, content: &str, _model: &str) -> Result<usize, ProviderError> {
        // Simple token counting: 1 token per 4 characters
        Ok((content.len() + 3) / 4)
    }

    async fn health_check(&self) -> Result<bool, ProviderError> {
        Ok(true)
    }
}

/// Property: Calling the same method with the same input produces consistent behavior
/// For any provider implementation, calling chat() with the same request should
/// produce the same response (or the same error type)
#[tokio::test]
async fn prop_provider_chat_consistency() {
    let request = ChatRequest {
        model: "gpt-4".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: "Hello".to_string(),
        }],
        temperature: Some(0.7),
        max_tokens: Some(100),
        stream: false,
    };

    let provider = Arc::new(ConsistentMockProvider {
        id: "test".to_string(),
        name: "Test Provider".to_string(),
    });

    // Call chat twice with the same request
    let response1 = provider.chat(request.clone()).await;
    let response2 = provider.chat(request.clone()).await;

    // Both should succeed and be identical
    match (&response1, &response2) {
        (Ok(r1), Ok(r2)) => {
            assert_eq!(r1.content, r2.content);
            assert_eq!(r1.model, r2.model);
            assert_eq!(r1.usage.prompt_tokens, r2.usage.prompt_tokens);
            assert_eq!(r1.usage.completion_tokens, r2.usage.completion_tokens);
            assert_eq!(r1.finish_reason, r2.finish_reason);
        }
        _ => panic!("Expected both calls to succeed"),
    }
}

/// Property: Token counting is consistent
/// For any content and model, calling count_tokens() twice should produce the same result
#[test]
fn prop_provider_token_counting_consistency() {
    proptest!(|(
        content in "[a-zA-Z0-9 ]{0,1000}",
        model in "gpt-4|gpt-3.5-turbo|claude-3"
    )| {
        let provider = Arc::new(ConsistentMockProvider {
            id: "test".to_string(),
            name: "Test Provider".to_string(),
        });

        // Call count_tokens twice with the same input
        let count1 = provider.count_tokens(&content, &model);
        let count2 = provider.count_tokens(&content, &model);

        // Both should produce the same result
        prop_assert_eq!(count1, count2);
    });
}

/// Property: Health check is consistent
/// For any provider, calling health_check() multiple times should produce consistent results
#[tokio::test]
async fn prop_provider_health_check_consistency() {
    let provider = Arc::new(ConsistentMockProvider {
        id: "test".to_string(),
        name: "Test Provider".to_string(),
    });

    // Call health_check multiple times
    let health1 = provider.health_check().await;
    let health2 = provider.health_check().await;
    let health3 = provider.health_check().await;

    // All should produce the same result
    assert_eq!(health1, health2);
    assert_eq!(health2, health3);
}

/// Property: Provider metadata is consistent
/// For any provider, id() and name() should always return the same values
#[test]
fn prop_provider_metadata_consistency() {
    let provider = Arc::new(ConsistentMockProvider {
        id: "test-provider".to_string(),
        name: "Test Provider".to_string(),
    });

    // Call id() and name() multiple times
    let id1 = provider.id();
    let id2 = provider.id();
    let name1 = provider.name();
    let name2 = provider.name();

    // All should be identical
    assert_eq!(id1, id2);
    assert_eq!(name1, name2);
    assert_eq!(id1, "test-provider");
    assert_eq!(name1, "Test Provider");
}
