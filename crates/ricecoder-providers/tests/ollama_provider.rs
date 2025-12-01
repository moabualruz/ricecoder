//! Unit tests for Ollama provider implementation

use ricecoder_providers::{ChatRequest, Message, OllamaProvider, Provider};

#[test]
fn test_ollama_provider_creation_success() {
    let provider = OllamaProvider::new("http://localhost:11434".to_string());
    assert!(provider.is_ok());
}

#[test]
fn test_ollama_provider_creation_empty_url() {
    let provider = OllamaProvider::new("".to_string());
    assert!(provider.is_err());
    match provider {
        Err(e) => assert!(e.to_string().contains("base URL is required")),
        Ok(_) => panic!("Expected error for empty base URL"),
    }
}

#[test]
fn test_ollama_provider_with_default_endpoint() {
    let provider = OllamaProvider::with_default_endpoint();
    assert!(provider.is_ok());
}

#[test]
fn test_ollama_provider_id() {
    let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
    assert_eq!(provider.id(), "ollama");
}

#[test]
fn test_ollama_provider_name() {
    let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
    assert_eq!(provider.name(), "Ollama");
}

#[test]
fn test_ollama_default_models_available() {
    let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
    let models = provider.models();
    
    // Should have default models when none are fetched
    assert!(!models.is_empty());
    assert!(models.iter().any(|m| m.id == "mistral"));
    assert!(models.iter().any(|m| m.id == "neural-chat"));
    assert!(models.iter().any(|m| m.id == "llama2"));
}

#[test]
fn test_ollama_model_info_mistral() {
    let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
    let models = provider.models();
    
    let mistral = models.iter().find(|m| m.id == "mistral").unwrap();
    assert_eq!(mistral.name, "Mistral");
    assert_eq!(mistral.provider, "ollama");
    assert_eq!(mistral.context_window, 8192);
    assert!(mistral.pricing.is_none()); // Local models have no pricing
}

#[test]
fn test_ollama_model_info_llama2() {
    let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
    let models = provider.models();
    
    let llama2 = models.iter().find(|m| m.id == "llama2").unwrap();
    assert_eq!(llama2.name, "Llama 2");
    assert_eq!(llama2.context_window, 4096);
}

#[test]
fn test_ollama_token_counting_valid_model() {
    let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
    let tokens = provider.count_tokens("Hello, world!", "mistral");
    
    assert!(tokens.is_ok());
    assert!(tokens.unwrap() > 0);
}

#[test]
fn test_ollama_token_counting_empty_content() {
    let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
    let tokens = provider.count_tokens("", "mistral");
    
    assert!(tokens.is_ok());
    assert_eq!(tokens.unwrap(), 0);
}

#[test]
fn test_ollama_token_counting_approximation() {
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
fn test_ollama_token_counting_consistency() {
    let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
    let content = "This is a test message for token counting";
    
    let tokens1 = provider.count_tokens(content, "mistral").unwrap();
    let tokens2 = provider.count_tokens(content, "mistral").unwrap();
    
    assert_eq!(tokens1, tokens2);
}

#[test]
fn test_ollama_token_counting_longer_content() {
    let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
    let short = "Hello";
    let long = "Hello world, this is a much longer message with more content and words";
    
    let tokens_short = provider.count_tokens(short, "mistral").unwrap();
    let tokens_long = provider.count_tokens(long, "mistral").unwrap();
    
    // Longer content should have more tokens
    assert!(tokens_long > tokens_short);
}

#[test]
fn test_ollama_token_counting_special_characters() {
    let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
    let simple = "hello";
    let with_special = "hello!!!???";
    
    let tokens_simple = provider.count_tokens(simple, "mistral").unwrap();
    let tokens_special = provider.count_tokens(with_special, "mistral").unwrap();
    
    // Special characters should increase token count
    assert!(tokens_special >= tokens_simple);
}

#[test]
fn test_ollama_models_have_capabilities() {
    let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
    let models = provider.models();
    
    for model in models {
        assert!(!model.capabilities.is_empty());
        assert!(model.capabilities.contains(&ricecoder_providers::Capability::Chat));
        assert!(model.capabilities.contains(&ricecoder_providers::Capability::Code));
    }
}

#[test]
fn test_ollama_models_no_pricing() {
    let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
    let models = provider.models();
    
    for model in models {
        assert!(model.pricing.is_none(), "Local models should not have pricing");
    }
}

#[test]
fn test_ollama_neural_chat_context_window() {
    let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
    let models = provider.models();
    
    let neural_chat = models.iter().find(|m| m.id == "neural-chat").unwrap();
    assert_eq!(neural_chat.context_window, 4096);
}

#[test]
fn test_ollama_all_models_have_streaming() {
    let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
    let models = provider.models();
    
    for model in models {
        assert!(
            model.capabilities.contains(&ricecoder_providers::Capability::Streaming),
            "Model {} should support streaming",
            model.id
        );
    }
}

#[test]
fn test_ollama_chat_request_structure() {
    let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
    let _request = ChatRequest {
        model: "mistral".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: "Hello".to_string(),
        }],
        temperature: Some(0.7),
        max_tokens: Some(100),
        stream: false,
    };
    
    // Verify the model exists
    let models = provider.models();
    assert!(models.iter().any(|m| m.id == "mistral"));
}

#[test]
fn test_ollama_token_counting_large_content() {
    let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
    let large_content = "a".repeat(10000); // 10,000 characters
    
    let tokens = provider.count_tokens(&large_content, "mistral").unwrap();
    
    // Should be approximately 2500 tokens (10000 / 4)
    assert!(tokens >= 2400 && tokens <= 2600);
}

#[test]
fn test_ollama_token_counting_unicode() {
    let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
    let unicode_content = "Hello 世界 🌍";
    
    let tokens = provider.count_tokens(unicode_content, "mistral").unwrap();
    
    // Should handle unicode characters correctly
    assert!(tokens > 0);
}

#[test]
fn test_ollama_provider_multiple_instances() {
    let provider1 = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
    let provider2 = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
    
    assert_eq!(provider1.id(), provider2.id());
    assert_eq!(provider1.name(), provider2.name());
    assert_eq!(provider1.models().len(), provider2.models().len());
}

#[test]
fn test_ollama_token_counting_newlines() {
    let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
    let with_newlines = "line1\nline2\nline3";
    let without_newlines = "line1line2line3";
    
    let tokens_with = provider.count_tokens(with_newlines, "mistral").unwrap();
    let tokens_without = provider.count_tokens(without_newlines, "mistral").unwrap();
    
    // Newlines are characters too, so with_newlines should have more tokens
    assert!(tokens_with > tokens_without);
}
