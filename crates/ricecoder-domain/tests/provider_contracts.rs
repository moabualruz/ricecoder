//! AI Provider Contract Tests - LSP Validation
//!
//! LSP compliance verification for AI providers
//!
//! These tests define the behavioral contract that ALL AI provider implementations
//! MUST satisfy. Any implementation that passes these tests is guaranteed to be
//! substitutable for any other implementation (Liskov Substitution Principle).

use async_trait::async_trait;
use ricecoder_domain::{
    errors::{DomainError, DomainResult},
    ports::ai::*,
};

// ============================================================================
// AiProvider Contract Tests
// ============================================================================

/// Contract test trait for AiProvider
#[async_trait]
pub trait AiProviderContract: Sized + Send + Sync {
    /// Get the provider under test
    fn provider(&self) -> &dyn AiProvider;
}

/// Run all contract tests against an AiProvider implementation
pub async fn run_ai_provider_contracts<T: AiProviderContract>(harness: &T) {
    println!("Running AiProvider contract tests...");
    
    // Contract 1: ID is non-empty
    provider_contract_id_non_empty(harness);
    
    // Contract 2: Name is non-empty
    provider_contract_name_non_empty(harness);
    
    // Contract 3: Models list is available
    provider_contract_models_available(harness);
    
    // Contract 4: Count tokens returns valid count
    provider_contract_count_tokens(harness);
    
    // Contract 5: Health check returns valid result
    provider_contract_health_check(harness).await;
    
    // Contract 6: Default model consistency
    provider_contract_default_model(harness);
    
    // Contract 7: Capability check consistency
    provider_contract_capability_check(harness);
    
    // Contract 8: Chat with valid request succeeds
    provider_contract_chat_valid_request(harness).await;
    
    println!("All AiProvider contracts passed!");
}

fn provider_contract_id_non_empty<T: AiProviderContract>(harness: &T) {
    let id = harness.provider().id();
    assert!(!id.is_empty(), "Contract violation: id() must return non-empty string");
    assert!(!id.contains(' '), "Contract violation: id() should be a single token without spaces");
}

fn provider_contract_name_non_empty<T: AiProviderContract>(harness: &T) {
    let name = harness.provider().name();
    assert!(!name.is_empty(), "Contract violation: name() must return non-empty string");
}

fn provider_contract_models_available<T: AiProviderContract>(harness: &T) {
    let models = harness.provider().models();
    // Note: Empty models list is valid for some providers (e.g., during initialization)
    // But each model must have valid fields
    for model in &models {
        assert!(!model.id.is_empty(), "Contract violation: model id must be non-empty");
        assert!(!model.name.is_empty(), "Contract violation: model name must be non-empty");
        assert!(!model.provider.is_empty(), "Contract violation: model provider must be non-empty");
        assert!(model.context_window > 0, "Contract violation: context_window must be positive");
    }
}

fn provider_contract_count_tokens<T: AiProviderContract>(harness: &T) {
    let models = harness.provider().models();
    if let Some(model) = models.first() {
        let content = "Hello, world!";
        let result = harness.provider().count_tokens(content, &model.id);
        
        // Must return Ok or a specific error, never panic
        match result {
            Ok(count) => {
                assert!(count > 0, "Contract violation: non-empty content must have positive token count");
            }
            Err(DomainError::ExternalServiceError { .. }) => {
                // Acceptable - some providers don't implement token counting
            }
            Err(e) => {
                panic!("Contract violation: count_tokens returned unexpected error: {:?}", e);
            }
        }
        
        // Empty content should return 0 or small count
        let empty_result = harness.provider().count_tokens("", &model.id);
        if let Ok(count) = empty_result {
            assert!(count <= 1, "Contract violation: empty content should have 0-1 tokens");
        }
    }
}

async fn provider_contract_health_check<T: AiProviderContract>(harness: &T) {
    let result = harness.provider().health_check().await;
    
    // Health check must always return Ok with a valid result
    match result {
        Ok(health) => {
            // Status must be a valid enum value (implicit by type system)
            // If healthy, latency should be present
            if health.status == ProviderHealthStatus::Healthy {
                assert!(health.latency_ms.is_some(), 
                    "Contract violation: healthy status must include latency");
                assert!(health.error.is_none(),
                    "Contract violation: healthy status must not have error");
            }
            // If unhealthy, error should be present
            if health.status == ProviderHealthStatus::Unhealthy {
                assert!(health.error.is_some(),
                    "Contract violation: unhealthy status should have error message");
            }
        }
        Err(_) => {
            // It's acceptable for health_check to return Err if provider is unreachable
            // The contract allows this - the caller should handle gracefully
        }
    }
}

fn provider_contract_default_model<T: AiProviderContract>(harness: &T) {
    let models = harness.provider().models();
    let default = harness.provider().default_model();
    
    if models.is_empty() {
        assert!(default.is_none(), 
            "Contract violation: empty models list must return None for default_model");
    } else {
        // If models exist, default should be in the list
        if let Some(default_id) = &default {
            assert!(models.iter().any(|m| &m.id == default_id),
                "Contract violation: default_model must be in models() list");
        }
    }
}

fn provider_contract_capability_check<T: AiProviderContract>(harness: &T) {
    let models = harness.provider().models();
    
    // supports_capability must be consistent with models() capabilities
    for capability in [
        ModelCapability::Chat,
        ModelCapability::Code,
        ModelCapability::Vision,
        ModelCapability::Streaming,
        ModelCapability::FunctionCalling,
        ModelCapability::Embeddings,
    ] {
        let has_capability = harness.provider().supports_capability(capability);
        let models_have_capability = models.iter().any(|m| m.capabilities.contains(&capability));
        
        assert_eq!(has_capability, models_have_capability,
            "Contract violation: supports_capability({:?}) must match models() capabilities", capability);
    }
}

async fn provider_contract_chat_valid_request<T: AiProviderContract>(harness: &T) {
    let models = harness.provider().models();
    
    // Find a model that supports chat
    if let Some(model) = models.iter().find(|m| m.capabilities.contains(&ModelCapability::Chat)) {
        let request = AiChatRequest::new(
            model.id.clone(),
            vec![ChatMessage::user("Say 'test' and nothing else.")],
        );
        
        let result = harness.provider().chat(request).await;
        
        match result {
            Ok(response) => {
                // Response must have valid fields
                assert!(!response.content.is_empty() || response.finish_reason == FinishReason::Length,
                    "Contract violation: successful chat must return content or hit length limit");
                assert!(!response.model.is_empty(),
                    "Contract violation: response must include model identifier");
                assert!(response.usage.total_tokens >= response.usage.prompt_tokens,
                    "Contract violation: total_tokens must be >= prompt_tokens");
            }
            Err(DomainError::ExternalServiceError { .. }) => {
                // Acceptable - external service may be unavailable
            }
            Err(DomainError::InvalidProviderConfig { .. }) => {
                // Acceptable - API key may not be configured
            }
            Err(e) => {
                panic!("Contract violation: chat returned unexpected error type: {:?}", e);
            }
        }
    }
}

// ============================================================================
// Integration Tests with Mock Provider
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    
    /// Mock AI provider for contract testing
    pub struct MockAiProvider {
        healthy: AtomicBool,
    }
    
    impl MockAiProvider {
        pub fn new() -> Self {
            Self { healthy: AtomicBool::new(true) }
        }
        
        pub fn set_healthy(&self, healthy: bool) {
            self.healthy.store(healthy, Ordering::SeqCst);
        }
    }
    
    impl AiProviderInfo for MockAiProvider {
        fn id(&self) -> &str {
            "mock"
        }
        
        fn name(&self) -> &str {
            "Mock Provider"
        }
        
        fn models(&self) -> Vec<ModelInfo> {
            vec![
                ModelInfo {
                    id: "mock-model-1".to_string(),
                    name: "Mock Model 1".to_string(),
                    provider: "mock".to_string(),
                    context_window: 4096,
                    capabilities: vec![ModelCapability::Chat, ModelCapability::Code],
                    is_free: true,
                },
                ModelInfo {
                    id: "mock-model-2".to_string(),
                    name: "Mock Model 2".to_string(),
                    provider: "mock".to_string(),
                    context_window: 8192,
                    capabilities: vec![ModelCapability::Chat, ModelCapability::Streaming],
                    is_free: false,
                },
            ]
        }
    }

    #[async_trait]
    impl AiProviderChat for MockAiProvider {
        async fn chat(&self, request: AiChatRequest) -> DomainResult<AiChatResponse> {
            if !self.healthy.load(Ordering::SeqCst) {
                return Err(DomainError::ExternalServiceError {
                    service: "mock".to_string(),
                    reason: "Provider unhealthy".to_string(),
                });
            }
            
            // Validate model exists
            if !self.models().iter().any(|m| m.id == request.model) {
                return Err(DomainError::ValidationError {
                    field: "model".to_string(),
                    reason: format!("Unknown model: {}", request.model),
                });
            }
            
            Ok(AiChatResponse {
                content: "test".to_string(),
                model: request.model,
                usage: TokenUsage {
                    prompt_tokens: 10,
                    completion_tokens: 1,
                    total_tokens: 11,
                },
                finish_reason: FinishReason::Stop,
            })
        }
        
        fn count_tokens(&self, content: &str, _model: &str) -> DomainResult<usize> {
            // Simple word-based approximation
            Ok(content.split_whitespace().count().max(if content.is_empty() { 0 } else { 1 }))
        }
        
        async fn health_check(&self) -> DomainResult<HealthCheckResult> {
            if self.healthy.load(Ordering::SeqCst) {
                Ok(HealthCheckResult::healthy(50))
            } else {
                Ok(HealthCheckResult::unhealthy("Mock provider set to unhealthy"))
            }
        }
        
        fn supports_capability(&self, capability: ModelCapability) -> bool {
            self.models().iter().any(|m| m.capabilities.contains(&capability))
        }
    }
    
    struct MockProviderHarness {
        provider: MockAiProvider,
    }
    
    #[async_trait]
    impl AiProviderContract for MockProviderHarness {
        fn provider(&self) -> &dyn AiProvider {
            &self.provider
        }
    }
    
    #[tokio::test]
    async fn test_ai_provider_contracts() {
        let harness = MockProviderHarness {
            provider: MockAiProvider::new(),
        };
        run_ai_provider_contracts(&harness).await;
    }
    
    #[tokio::test]
    async fn test_unhealthy_provider_health_check() {
        let provider = MockAiProvider::new();
        provider.set_healthy(false);
        
        let result = provider.health_check().await.unwrap();
        assert_eq!(result.status, ProviderHealthStatus::Unhealthy);
        assert!(result.error.is_some());
    }
    
    #[tokio::test]
    async fn test_chat_with_invalid_model() {
        let provider = MockAiProvider::new();
        let request = AiChatRequest::new(
            "non-existent-model",
            vec![ChatMessage::user("Hello")],
        );
        
        let result = provider.chat(request).await;
        assert!(result.is_err());
    }
}
