//! Unit tests for OpenAI provider implementation

use ricecoder_providers::{ChatRequest, OpenAiProvider, Provider};

#[test]
fn test_openai_provider_creation_success() {
    let provider = OpenAiProvider::new("sk-test-key".to_string());
    assert!(provider.is_ok());
}

#[test]
fn test_openai_provider_creation_empty_key() {
    let provider = OpenAiProvider::new("".to_string());
    assert!(provider.is_err());
    match provider {
        Err(e) => assert!(e.to_string().contains("API key is required")),
        Ok(_) => panic!("Expected error for empty API key"),
    }
}

#[test]
fn test_openai_provider_id() {
    let provider = OpenAiProvider::new("sk-test-key".to_string()).unwrap();
    assert_eq!(provider.id(), "openai");
}

#[test]
fn test_openai_provider_name() {
    let provider = OpenAiProvider::new("sk-test-key".to_string()).unwrap();
    assert_eq!(provider.name(), "OpenAI");
}

#[test]
fn test_openai_models_available() {
    let provider = OpenAiProvider::new("sk-test-key".to_string()).unwrap();
    let models = provider.models();
    
    assert_eq!(models.len(), 4);
    assert!(models.iter().any(|m| m.id == "gpt-4"));
    assert!(models.iter().any(|m| m.id == "gpt-4-turbo"));
    assert!(models.iter().any(|m| m.id == "gpt-4o"));
    assert!(models.iter().any(|m| m.id == "gpt-3.5-turbo"));
}

#[test]
fn test_openai_model_info_gpt4() {
    let provider = OpenAiProvider::new("sk-test-key".to_string()).unwrap();
    let models = provider.models();
    
    let gpt4 = models.iter().find(|m| m.id == "gpt-4").unwrap();
    assert_eq!(gpt4.name, "GPT-4");
    assert_eq!(gpt4.provider, "openai");
    assert_eq!(gpt4.context_window, 8192);
    assert!(gpt4.pricing.is_some());
}

#[test]
fn test_openai_model_info_gpt4o() {
    let provider = OpenAiProvider::new("sk-test-key".to_string()).unwrap();
    let models = provider.models();
    
    let gpt4o = models.iter().find(|m| m.id == "gpt-4o").unwrap();
    assert_eq!(gpt4o.name, "GPT-4o");
    assert_eq!(gpt4o.context_window, 128000);
    assert!(gpt4o.capabilities.contains(&ricecoder_providers::Capability::Vision));
}

#[test]
fn test_openai_token_counting_valid_model() {
    let provider = OpenAiProvider::new("sk-test-key".to_string()).unwrap();
    let tokens = provider.count_tokens("Hello, world!", "gpt-4");
    
    assert!(tokens.is_ok());
    assert!(tokens.unwrap() > 0);
}

#[test]
fn test_openai_token_counting_empty_content() {
    let provider = OpenAiProvider::new("sk-test-key".to_string()).unwrap();
    let tokens = provider.count_tokens("", "gpt-4");
    
    assert!(tokens.is_ok());
    assert_eq!(tokens.unwrap(), 0);
}

#[test]
fn test_openai_token_counting_invalid_model() {
    let provider = OpenAiProvider::new("sk-test-key".to_string()).unwrap();
    let tokens = provider.count_tokens("Hello, world!", "invalid-model");
    
    assert!(tokens.is_err());
    match tokens {
        Err(e) => assert!(e.to_string().contains("Invalid model")),
        Ok(_) => panic!("Expected error for invalid model"),
    }
}

#[test]
fn test_openai_token_counting_consistency() {
    let provider = OpenAiProvider::new("sk-test-key".to_string()).unwrap();
    let content = "This is a test message for token counting";
    
    let tokens1 = provider.count_tokens(content, "gpt-4").unwrap();
    let tokens2 = provider.count_tokens(content, "gpt-4").unwrap();
    
    assert_eq!(tokens1, tokens2);
}

#[test]
fn test_openai_token_counting_different_models() {
    let provider = OpenAiProvider::new("sk-test-key".to_string()).unwrap();
    let content = "Test content";
    
    let tokens_gpt4 = provider.count_tokens(content, "gpt-4").unwrap();
    let tokens_gpt35 = provider.count_tokens(content, "gpt-3.5-turbo").unwrap();
    
    // Both should return valid token counts
    assert!(tokens_gpt4 > 0);
    assert!(tokens_gpt35 > 0);
}

#[test]
fn test_openai_token_counting_longer_content() {
    let provider = OpenAiProvider::new("sk-test-key".to_string()).unwrap();
    let short = "Hello";
    let long = "Hello world, this is a much longer message with more content and words";
    
    let tokens_short = provider.count_tokens(short, "gpt-4").unwrap();
    let tokens_long = provider.count_tokens(long, "gpt-4").unwrap();
    
    // Longer content should have more tokens
    assert!(tokens_long > tokens_short);
}

#[test]
fn test_openai_token_counting_special_characters() {
    let provider = OpenAiProvider::new("sk-test-key".to_string()).unwrap();
    let simple = "hello";
    let with_special = "hello!!!???";
    
    let tokens_simple = provider.count_tokens(simple, "gpt-4").unwrap();
    let tokens_special = provider.count_tokens(with_special, "gpt-4").unwrap();
    
    // Special characters should increase token count
    assert!(tokens_special >= tokens_simple);
}

#[test]
fn test_openai_with_base_url() {
    let provider = OpenAiProvider::with_base_url(
        "sk-test-key".to_string(),
        "https://custom.openai.com/v1".to_string(),
    );
    
    assert!(provider.is_ok());
    let provider = provider.unwrap();
    assert_eq!(provider.id(), "openai");
}

#[test]
fn test_openai_with_base_url_empty_key() {
    let provider = OpenAiProvider::with_base_url(
        "".to_string(),
        "https://custom.openai.com/v1".to_string(),
    );
    
    assert!(provider.is_err());
}

#[test]
fn test_openai_chat_request_invalid_model() {
    let provider = OpenAiProvider::new("sk-test-key".to_string()).unwrap();
    let _request = ChatRequest {
        model: "invalid-model".to_string(),
        messages: vec![],
        temperature: None,
        max_tokens: None,
        stream: false,
    };
    
    // This would fail in async context, but we can at least verify the model validation
    let models = provider.models();
    assert!(!models.iter().any(|m| m.id == "invalid-model"));
}

#[test]
fn test_openai_models_have_capabilities() {
    let provider = OpenAiProvider::new("sk-test-key".to_string()).unwrap();
    let models = provider.models();
    
    for model in models {
        assert!(!model.capabilities.is_empty());
        assert!(model.capabilities.contains(&ricecoder_providers::Capability::Chat));
    }
}

#[test]
fn test_openai_models_have_pricing() {
    let provider = OpenAiProvider::new("sk-test-key".to_string()).unwrap();
    let models = provider.models();
    
    for model in models {
        assert!(model.pricing.is_some());
        let pricing = model.pricing.unwrap();
        assert!(pricing.input_per_1k_tokens > 0.0);
        assert!(pricing.output_per_1k_tokens > 0.0);
    }
}

#[test]
fn test_openai_gpt35_turbo_context_window() {
    let provider = OpenAiProvider::new("sk-test-key".to_string()).unwrap();
    let models = provider.models();
    
    let gpt35 = models.iter().find(|m| m.id == "gpt-3.5-turbo").unwrap();
    assert_eq!(gpt35.context_window, 4096);
}

#[test]
fn test_openai_gpt4_turbo_has_vision() {
    let provider = OpenAiProvider::new("sk-test-key".to_string()).unwrap();
    let models = provider.models();
    
    let gpt4_turbo = models.iter().find(|m| m.id == "gpt-4-turbo").unwrap();
    assert!(gpt4_turbo.capabilities.contains(&ricecoder_providers::Capability::Vision));
}

#[test]
fn test_openai_all_models_have_streaming() {
    let provider = OpenAiProvider::new("sk-test-key".to_string()).unwrap();
    let models = provider.models();
    
    for model in models {
        assert!(
            model.capabilities.contains(&ricecoder_providers::Capability::Streaming),
            "Model {} should support streaming",
            model.id
        );
    }
}
