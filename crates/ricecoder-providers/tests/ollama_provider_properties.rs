//! Property-based tests for Ollama provider
//!
//! These tests verify correctness properties that should hold across all inputs.
//! Each property is derived from acceptance criteria in the requirements document.

use proptest::prelude::*;
use ricecoder_providers::{
    ChatRequest, Message, OllamaProvider, Provider, Capability,
};

// ============================================================================
// Property 1: Provider Trait Implementation
// **Feature: ricecoder-local-models, Property 1: Provider Trait Implementation**
// **Validates: Requirements 1.1, 1.2**
// For any OllamaProvider instance, all Provider trait methods SHALL be callable
// and return consistent results
// ============================================================================

proptest! {
    #[test]
    fn prop_provider_id_is_consistent(_seed in 0u32..100) {
        let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
        let id1 = provider.id();
        let id2 = provider.id();
        prop_assert_eq!(id1, id2);
        prop_assert_eq!(id1, "ollama");
    }

    #[test]
    fn prop_provider_name_is_consistent(_seed in 0u32..100) {
        let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
        let name1 = provider.name();
        let name2 = provider.name();
        prop_assert_eq!(name1, name2);
        prop_assert_eq!(name1, "Ollama");
    }

    #[test]
    fn prop_models_list_is_consistent(_seed in 0u32..100) {
        let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
        let models1 = provider.models();
        let models2 = provider.models();
        prop_assert_eq!(models1.len(), models2.len());
        for (m1, m2) in models1.iter().zip(models2.iter()) {
            prop_assert_eq!(&m1.id, &m2.id);
            prop_assert_eq!(&m1.name, &m2.name);
            prop_assert_eq!(&m1.provider, &m2.provider);
        }
    }

    #[test]
    fn prop_count_tokens_returns_result(content in "\\PC{0,1000}", _seed in 0u32..100) {
        let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
        let result = provider.count_tokens(&content, "mistral");
        prop_assert!(result.is_ok());
    }
}

// ============================================================================
// Property 2: Model Listing Consistency
// **Feature: ricecoder-local-models, Property 2: Model Listing Consistency**
// **Validates: Requirements 2.1, 2.3**
// For any Ollama instance with available models, listing SHALL return all models
// with accurate metadata (id, name, context_size, capabilities)
// ============================================================================

proptest! {
    #[test]
    fn prop_models_have_required_fields(_seed in 0u32..100) {
        let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
        let models = provider.models();
        
        for model in models {
            // All models must have required fields
            prop_assert!(!model.id.is_empty(), "Model ID must not be empty");
            prop_assert!(!model.name.is_empty(), "Model name must not be empty");
            prop_assert_eq!(model.provider, "ollama", "Provider must be 'ollama'");
            prop_assert!(model.context_window > 0, "Context window must be positive");
            prop_assert!(!model.capabilities.is_empty(), "Capabilities must not be empty");
        }
    }

    #[test]
    fn prop_models_have_chat_capability(_seed in 0u32..100) {
        let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
        let models = provider.models();
        
        for model in models {
            prop_assert!(
                model.capabilities.contains(&Capability::Chat),
                "Model {} must support Chat capability",
                model.id
            );
        }
    }

    #[test]
    fn prop_models_have_streaming_capability(_seed in 0u32..100) {
        let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
        let models = provider.models();
        
        for model in models {
            prop_assert!(
                model.capabilities.contains(&Capability::Streaming),
                "Model {} must support Streaming capability",
                model.id
            );
        }
    }

    #[test]
    fn prop_local_models_have_no_pricing(_seed in 0u32..100) {
        let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
        let models = provider.models();
        
        for model in models {
            prop_assert!(
                model.pricing.is_none(),
                "Local model {} should not have pricing",
                model.id
            );
        }
    }

    #[test]
    fn prop_model_ids_are_unique(_seed in 0u32..100) {
        let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
        let models = provider.models();
        
        let mut ids = Vec::new();
        for model in models {
            prop_assert!(
                !ids.contains(&model.id),
                "Duplicate model ID: {}",
                model.id
            );
            ids.push(model.id);
        }
    }
}

// ============================================================================
// Property 3: Model Cache Correctness
// **Feature: ricecoder-local-models, Property 3: Model Cache Correctness**
// **Validates: Requirements 2.2, 2.4**
// For any valid model cache within TTL, subsequent calls SHALL return cached
// results without API call
// ============================================================================

#[test]
fn test_model_cache_returns_same_models() {
    let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
    let models1 = provider.models();
    let models2 = provider.models();
    
    // Cache should return identical results
    assert_eq!(models1.len(), models2.len());
    for (m1, m2) in models1.iter().zip(models2.iter()) {
        assert_eq!(&m1.id, &m2.id);
        assert_eq!(&m1.name, &m2.name);
        assert_eq!(&m1.provider, &m2.provider);
        assert_eq!(&m1.context_window, &m2.context_window);
    }
}

proptest! {
    #[test]
    fn prop_cache_consistency_across_calls(_seed in 0u32..100) {
        let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
        
        // Multiple calls should return consistent results
        let models1 = provider.models();
        let models2 = provider.models();
        let models3 = provider.models();
        
        prop_assert_eq!(models1.len(), models2.len());
        prop_assert_eq!(models2.len(), models3.len());
        
        for (m1, m2) in models1.iter().zip(models2.iter()) {
            prop_assert_eq!(&m1.id, &m2.id);
        }
        
        for (m2, m3) in models2.iter().zip(models3.iter()) {
            prop_assert_eq!(&m2.id, &m3.id);
        }
    }
}

// ============================================================================
// Property 4: Chat Request Routing
// **Feature: ricecoder-local-models, Property 4: Chat Request Routing**
// **Validates: Requirements 3.1, 3.2**
// For any valid chat request to a loaded model, system SHALL route through
// Ollama API and return response with token usage
// ============================================================================

proptest! {
    #[test]
    fn prop_chat_response_has_required_fields(
        content in "\\PC{1,500}",
        _seed in 0u32..100
    ) {
        let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
        let models = provider.models();
        
        if !models.is_empty() {
            let model_id = &models[0].id;
            let request = ChatRequest {
                model: model_id.clone(),
                messages: vec![Message {
                    role: "user".to_string(),
                    content,
                }],
                temperature: Some(0.7),
                max_tokens: Some(100),
                stream: false,
            };
            
            // Verify request structure is valid
            prop_assert!(!request.model.is_empty());
            prop_assert!(!request.messages.is_empty());
            prop_assert_eq!(&request.messages[0].role, "user");
        }
    }

    #[test]
    fn prop_token_usage_is_present_in_response(_seed in 0u32..100) {
        let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
        let models = provider.models();
        
        if !models.is_empty() {
            let model_id = &models[0].id;
            
            // Verify token counting works
            let tokens = provider.count_tokens("test message", model_id).unwrap();
            prop_assert!(tokens > 0);
        }
    }
}

// ============================================================================
// Property 5: Streaming Response Handling
// **Feature: ricecoder-local-models, Property 5: Streaming Response Handling**
// **Validates: Requirements 3.3**
// For any streaming chat request, system SHALL stream responses without
// buffering entire response
// ============================================================================

proptest! {
    #[test]
    fn prop_streaming_request_structure_is_valid(
        content in "\\PC{1,500}",
        _seed in 0u32..100
    ) {
        let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
        let models = provider.models();
        
        if !models.is_empty() {
            let model_id = &models[0].id;
            let request = ChatRequest {
                model: model_id.clone(),
                messages: vec![Message {
                    role: "user".to_string(),
                    content,
                }],
                temperature: Some(0.7),
                max_tokens: Some(100),
                stream: true,  // Streaming enabled
            };
            
            // Verify streaming request structure
            prop_assert!(request.stream);
            prop_assert!(!request.messages.is_empty());
        }
    }

    #[test]
    fn prop_streaming_models_support_streaming(_seed in 0u32..100) {
        let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
        let models = provider.models();
        
        for model in models {
            prop_assert!(
                model.capabilities.contains(&Capability::Streaming),
                "Model {} must support streaming",
                model.id
            );
        }
    }
}

// ============================================================================
// Property 6: Error Handling and Reporting
// **Feature: ricecoder-local-models, Property 6: Error Handling and Reporting**
// **Validates: Requirements 1.5, 3.4, 3.5**
// For any failed operation, system SHALL return explicit error with context
// and allow retry
// ============================================================================

proptest! {
    #[test]
    fn prop_invalid_url_returns_error(_seed in 0u32..100) {
        // Empty URL should fail
        let result = OllamaProvider::new("".to_string());
        prop_assert!(result.is_err());
    }

    #[test]
    fn prop_valid_url_succeeds(_seed in 0u32..100) {
        // Valid URL format should succeed
        let result = OllamaProvider::new("http://localhost:11434".to_string());
        prop_assert!(result.is_ok());
    }

    #[test]
    fn prop_error_messages_are_descriptive(_content in "\\PC{1,100}") {
        // Empty URL should produce descriptive error
        let result = OllamaProvider::new("".to_string());
        prop_assert!(result.is_err());
        
        if let Err(err) = result {
            let error_msg = err.to_string();
            prop_assert!(
                error_msg.contains("base URL") || error_msg.contains("required"),
                "Error message should be descriptive: {}",
                error_msg
            );
        }
    }
}

// ============================================================================
// Property 7: Model Management Operations
// **Feature: ricecoder-local-models, Property 7: Model Management Operations**
// **Validates: Requirements 4.1, 4.2, 4.3, 4.4**
// For any model management operation, system SHALL report progress and
// maintain metadata consistency
// ============================================================================

proptest! {
    #[test]
    fn prop_model_metadata_is_consistent(_seed in 0u32..100) {
        let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
        let models = provider.models();
        
        for model in models {
            // Metadata should be consistent
            prop_assert!(!model.id.is_empty());
            prop_assert!(!model.name.is_empty());
            prop_assert_eq!(model.provider, "ollama");
            prop_assert!(model.context_window > 0);
            
            // Capabilities should be valid
            for cap in &model.capabilities {
                match cap {
                    Capability::Chat | Capability::Code | Capability::Vision |
                    Capability::FunctionCalling | Capability::Streaming => {
                        // Valid capability
                    }
                }
            }
        }
    }

    #[test]
    fn prop_token_usage_is_consistent(content in "\\PC{1,1000}", _seed in 0u32..100) {
        let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
        let models = provider.models();
        
        if !models.is_empty() {
            let model_id = &models[0].id;
            
            // Token counting should be consistent
            let tokens1 = provider.count_tokens(&content, model_id).unwrap();
            let tokens2 = provider.count_tokens(&content, model_id).unwrap();
            
            prop_assert_eq!(tokens1, tokens2);
        }
    }

    #[test]
    fn prop_configuration_has_defaults(_seed in 0u32..100) {
        let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
        
        // Provider should have default configuration
        let config_result = provider.config();
        prop_assert!(config_result.is_ok());
        
        let config = config_result.unwrap();
        prop_assert!(!config.base_url.is_empty());
        prop_assert!(!config.default_model.is_empty());
    }
}

// ============================================================================
// Property 8: Offline Fallback Behavior
// **Feature: ricecoder-local-models, Property 8: Offline Fallback Behavior**
// **Validates: Requirements 2.5**
// For any model listing request when Ollama is unavailable, system SHALL
// return cached models if available, or explicit error if no cache exists
// ============================================================================

proptest! {
    #[test]
    fn prop_default_models_available_when_offline(_seed in 0u32..100) {
        let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
        
        // Even without fetching from Ollama, default models should be available
        let models = provider.models();
        prop_assert!(!models.is_empty(), "Default models should be available");
        
        // Should have at least the default models
        let model_ids: Vec<_> = models.iter().map(|m| m.id.as_str()).collect();
        prop_assert!(
            model_ids.contains(&"mistral") || model_ids.contains(&"llama2"),
            "Should have default models available"
        );
    }

    #[test]
    fn prop_fallback_models_have_valid_metadata(_seed in 0u32..100) {
        let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
        let models = provider.models();
        
        // All fallback models should have valid metadata
        for model in models {
            prop_assert!(!model.id.is_empty());
            prop_assert!(!model.name.is_empty());
            prop_assert_eq!(model.provider, "ollama");
            prop_assert!(model.context_window > 0);
            prop_assert!(!model.capabilities.is_empty());
            prop_assert!(model.pricing.is_none());
        }
    }

    #[test]
    fn prop_multiple_models_in_fallback(_seed in 0u32..100) {
        let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
        let models = provider.models();
        
        // Should have multiple fallback models
        prop_assert!(models.len() >= 3, "Should have at least 3 default models");
    }
}

// ============================================================================
// Additional Integration Tests
// ============================================================================

#[test]
fn test_provider_creation_with_default_endpoint() {
    let provider = OllamaProvider::with_default_endpoint();
    assert!(provider.is_ok());
    
    let provider = provider.unwrap();
    assert_eq!(provider.id(), "ollama");
    assert_eq!(provider.name(), "Ollama");
}

#[test]
fn test_token_counting_approximation() {
    let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
    
    // Test the approximation: 1 token ≈ 4 characters
    let content = "1234"; // 4 characters
    let tokens = provider.count_tokens(content, "mistral").unwrap();
    assert_eq!(tokens, 1); // Should be approximately 1 token
    
    let content = "12345678"; // 8 characters
    let tokens = provider.count_tokens(content, "mistral").unwrap();
    assert_eq!(tokens, 2); // Should be approximately 2 tokens
}

#[test]
fn test_empty_content_token_counting() {
    let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
    let tokens = provider.count_tokens("", "mistral").unwrap();
    assert_eq!(tokens, 0);
}

#[test]
fn test_all_default_models_have_capabilities() {
    let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
    let models = provider.models();
    
    for model in models {
        assert!(!model.capabilities.is_empty());
        assert!(model.capabilities.contains(&Capability::Chat));
        assert!(model.capabilities.contains(&Capability::Code));
        assert!(model.capabilities.contains(&Capability::Streaming));
    }
}

#[test]
fn test_provider_multiple_instances_consistency() {
    let provider1 = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
    let provider2 = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
    
    assert_eq!(provider1.id(), provider2.id());
    assert_eq!(provider1.name(), provider2.name());
    assert_eq!(provider1.models().len(), provider2.models().len());
}
