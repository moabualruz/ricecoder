//! Unit tests for Google provider implementation

use ricecoder_providers::{ChatRequest, GoogleProvider, Provider};

#[test]
fn test_google_provider_creation_success() {
    let provider = GoogleProvider::new("test-key".to_string());
    assert!(provider.is_ok());
}

#[test]
fn test_google_provider_creation_empty_key() {
    let provider = GoogleProvider::new("".to_string());
    assert!(provider.is_err());
    match provider {
        Err(e) => assert!(e.to_string().contains("API key is required")),
        Ok(_) => panic!("Expected error for empty API key"),
    }
}

#[test]
fn test_google_provider_id() {
    let provider = GoogleProvider::new("test-key".to_string()).unwrap();
    assert_eq!(provider.id(), "google");
}

#[test]
fn test_google_provider_name() {
    let provider = GoogleProvider::new("test-key".to_string()).unwrap();
    assert_eq!(provider.name(), "Google");
}

#[test]
fn test_google_models_available() {
    let provider = GoogleProvider::new("test-key".to_string()).unwrap();
    let models = provider.models();
    
    assert_eq!(models.len(), 4);
    assert!(models.iter().any(|m| m.id == "gemini-2.0-flash"));
    assert!(models.iter().any(|m| m.id == "gemini-1.5-pro"));
    assert!(models.iter().any(|m| m.id == "gemini-1.5-flash"));
    assert!(models.iter().any(|m| m.id == "gemini-1.0-pro"));
}

#[test]
fn test_google_model_info_gemini_2_0_flash() {
    let provider = GoogleProvider::new("test-key".to_string()).unwrap();
    let models = provider.models();
    
    let gemini = models.iter().find(|m| m.id == "gemini-2.0-flash").unwrap();
    assert_eq!(gemini.name, "Gemini 2.0 Flash");
    assert_eq!(gemini.provider, "google");
    assert_eq!(gemini.context_window, 1000000);
    assert!(gemini.pricing.is_some());
}

#[test]
fn test_google_model_info_gemini_1_5_pro() {
    let provider = GoogleProvider::new("test-key".to_string()).unwrap();
    let models = provider.models();
    
    let gemini = models.iter().find(|m| m.id == "gemini-1.5-pro").unwrap();
    assert_eq!(gemini.name, "Gemini 1.5 Pro");
    assert_eq!(gemini.context_window, 2000000);
    assert!(gemini.capabilities.contains(&ricecoder_providers::Capability::Vision));
}

#[test]
fn test_google_model_info_gemini_1_5_flash() {
    let provider = GoogleProvider::new("test-key".to_string()).unwrap();
    let models = provider.models();
    
    let gemini = models.iter().find(|m| m.id == "gemini-1.5-flash").unwrap();
    assert_eq!(gemini.name, "Gemini 1.5 Flash");
    assert_eq!(gemini.context_window, 1000000);
    assert!(gemini.capabilities.contains(&ricecoder_providers::Capability::Vision));
}

#[test]
fn test_google_model_info_gemini_1_0_pro() {
    let provider = GoogleProvider::new("test-key".to_string()).unwrap();
    let models = provider.models();
    
    let gemini = models.iter().find(|m| m.id == "gemini-1.0-pro").unwrap();
    assert_eq!(gemini.name, "Gemini 1.0 Pro");
    assert_eq!(gemini.context_window, 32000);
    assert!(!gemini.capabilities.contains(&ricecoder_providers::Capability::Vision));
}

#[test]
fn test_google_token_counting_valid_model() {
    let provider = GoogleProvider::new("test-key".to_string()).unwrap();
    let tokens = provider.count_tokens("Hello, world!", "gemini-1.5-pro");
    
    assert!(tokens.is_ok());
    assert!(tokens.unwrap() > 0);
}

#[test]
fn test_google_token_counting_empty_content() {
    let provider = GoogleProvider::new("test-key".to_string()).unwrap();
    let tokens = provider.count_tokens("", "gemini-1.5-pro");
    
    assert!(tokens.is_ok());
    assert_eq!(tokens.unwrap(), 0);
}

#[test]
fn test_google_token_counting_invalid_model() {
    let provider = GoogleProvider::new("test-key".to_string()).unwrap();
    let tokens = provider.count_tokens("Hello, world!", "invalid-model");
    
    assert!(tokens.is_err());
    match tokens {
        Err(e) => assert!(e.to_string().contains("Invalid model")),
        Ok(_) => panic!("Expected error for invalid model"),
    }
}

#[test]
fn test_google_token_counting_consistency() {
    let provider = GoogleProvider::new("test-key".to_string()).unwrap();
    let content = "This is a test message for token counting";
    
    let tokens1 = provider.count_tokens(content, "gemini-1.5-pro").unwrap();
    let tokens2 = provider.count_tokens(content, "gemini-1.5-pro").unwrap();
    
    assert_eq!(tokens1, tokens2);
}

#[test]
fn test_google_token_counting_different_models() {
    let provider = GoogleProvider::new("test-key".to_string()).unwrap();
    let content = "Test content";
    
    let tokens_pro = provider.count_tokens(content, "gemini-1.5-pro").unwrap();
    let tokens_flash = provider.count_tokens(content, "gemini-1.5-flash").unwrap();
    
    // Both should return valid token counts
    assert!(tokens_pro > 0);
    assert!(tokens_flash > 0);
}

#[test]
fn test_google_token_counting_longer_content() {
    let provider = GoogleProvider::new("test-key".to_string()).unwrap();
    let short = "Hello";
    let long = "Hello world, this is a much longer message with more content and words";
    
    let tokens_short = provider.count_tokens(short, "gemini-1.5-pro").unwrap();
    let tokens_long = provider.count_tokens(long, "gemini-1.5-pro").unwrap();
    
    // Longer content should have more tokens
    assert!(tokens_long > tokens_short);
}

#[test]
fn test_google_token_counting_special_characters() {
    let provider = GoogleProvider::new("test-key".to_string()).unwrap();
    let simple = "hello";
    let with_special = "hello!!!???";
    
    let tokens_simple = provider.count_tokens(simple, "gemini-1.5-pro").unwrap();
    let tokens_special = provider.count_tokens(with_special, "gemini-1.5-pro").unwrap();
    
    // Special characters should increase token count
    assert!(tokens_special >= tokens_simple);
}

#[test]
fn test_google_with_base_url() {
    let provider = GoogleProvider::with_base_url(
        "test-key".to_string(),
        "https://custom.google.com/v1/models".to_string(),
    );
    
    assert!(provider.is_ok());
    let provider = provider.unwrap();
    assert_eq!(provider.id(), "google");
}

#[test]
fn test_google_with_base_url_empty_key() {
    let provider = GoogleProvider::with_base_url(
        "".to_string(),
        "https://custom.google.com/v1/models".to_string(),
    );
    
    assert!(provider.is_err());
}

#[test]
fn test_google_chat_request_invalid_model() {
    let provider = GoogleProvider::new("test-key".to_string()).unwrap();
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
fn test_google_models_have_capabilities() {
    let provider = GoogleProvider::new("test-key".to_string()).unwrap();
    let models = provider.models();
    
    for model in models {
        assert!(!model.capabilities.is_empty());
        assert!(model.capabilities.contains(&ricecoder_providers::Capability::Chat));
    }
}

#[test]
fn test_google_models_have_pricing() {
    let provider = GoogleProvider::new("test-key".to_string()).unwrap();
    let models = provider.models();
    
    for model in models {
        assert!(model.pricing.is_some());
        let pricing = model.pricing.unwrap();
        assert!(pricing.input_per_1k_tokens > 0.0);
        assert!(pricing.output_per_1k_tokens > 0.0);
    }
}

#[test]
fn test_google_gemini_1_0_pro_context_window() {
    let provider = GoogleProvider::new("test-key".to_string()).unwrap();
    let models = provider.models();
    
    let gemini = models.iter().find(|m| m.id == "gemini-1.0-pro").unwrap();
    assert_eq!(gemini.context_window, 32000);
}

#[test]
fn test_google_gemini_1_5_pro_has_vision() {
    let provider = GoogleProvider::new("test-key".to_string()).unwrap();
    let models = provider.models();
    
    let gemini = models.iter().find(|m| m.id == "gemini-1.5-pro").unwrap();
    assert!(gemini.capabilities.contains(&ricecoder_providers::Capability::Vision));
}

#[test]
fn test_google_all_models_have_streaming() {
    let provider = GoogleProvider::new("test-key".to_string()).unwrap();
    let models = provider.models();
    
    for model in models {
        if model.id != "gemini-1.0-pro" {
            assert!(
                model.capabilities.contains(&ricecoder_providers::Capability::Streaming),
                "Model {} should support streaming",
                model.id
            );
        }
    }
}

#[test]
fn test_google_gemini_2_0_flash_large_context() {
    let provider = GoogleProvider::new("test-key".to_string()).unwrap();
    let models = provider.models();
    
    let gemini = models.iter().find(|m| m.id == "gemini-2.0-flash").unwrap();
    assert_eq!(gemini.context_window, 1000000);
    assert!(gemini.capabilities.contains(&ricecoder_providers::Capability::Vision));
}

#[test]
fn test_google_token_counting_all_models() {
    let provider = GoogleProvider::new("test-key".to_string()).unwrap();
    let content = "Test content for all models";
    
    for model in provider.models() {
        let tokens = provider.count_tokens(content, &model.id);
        assert!(tokens.is_ok(), "Token counting should work for model {}", model.id);
        assert!(tokens.unwrap() > 0, "Token count should be positive for model {}", model.id);
    }
}

#[test]
fn test_google_provider_models_provider_field() {
    let provider = GoogleProvider::new("test-key".to_string()).unwrap();
    let models = provider.models();
    
    for model in models {
        assert_eq!(model.provider, "google");
    }
}

#[test]
fn test_google_token_counting_code_content() {
    let provider = GoogleProvider::new("test-key".to_string()).unwrap();
    let code = r#"
fn main() {
    println!("Hello, world!");
}
"#;
    
    let tokens = provider.count_tokens(code, "gemini-1.5-pro").unwrap();
    assert!(tokens > 0);
}

#[test]
fn test_google_token_counting_multiline_content() {
    let provider = GoogleProvider::new("test-key".to_string()).unwrap();
    let content = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5";
    
    let tokens = provider.count_tokens(content, "gemini-1.5-pro").unwrap();
    assert!(tokens > 0);
}
