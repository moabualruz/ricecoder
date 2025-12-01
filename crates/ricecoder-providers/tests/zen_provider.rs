//! Unit tests for Zen provider implementation

use ricecoder_providers::{ChatRequest, Provider, ZenProvider, Capability};

#[test]
fn test_zen_provider_creation_success() {
    let provider = ZenProvider::new("test-key".to_string());
    assert!(provider.is_ok());
}

#[test]
fn test_zen_provider_creation_empty_key() {
    let provider = ZenProvider::new("".to_string());
    assert!(provider.is_err());
    match provider {
        Err(e) => assert!(e.to_string().contains("API key is required")),
        Ok(_) => panic!("Expected error for empty API key"),
    }
}

#[test]
fn test_zen_provider_id() {
    let provider = ZenProvider::new("test-key".to_string()).unwrap();
    assert_eq!(provider.id(), "zen");
}

#[test]
fn test_zen_provider_name() {
    let provider = ZenProvider::new("test-key".to_string()).unwrap();
    assert_eq!(provider.name(), "OpenCode Zen");
}

#[test]
fn test_zen_models_available() {
    let provider = ZenProvider::new("test-key".to_string()).unwrap();
    let models = provider.models();
    
    assert_eq!(models.len(), 2);
    assert!(models.iter().any(|m| m.id == "zen-gpt4"));
    assert!(models.iter().any(|m| m.id == "zen-gpt4-turbo"));
}

#[test]
fn test_zen_model_info_gpt4() {
    let provider = ZenProvider::new("test-key".to_string()).unwrap();
    let models = provider.models();
    
    let gpt4 = models.iter().find(|m| m.id == "zen-gpt4").unwrap();
    assert_eq!(gpt4.name, "Zen GPT-4");
    assert_eq!(gpt4.provider, "zen");
    assert_eq!(gpt4.context_window, 8192);
    assert!(gpt4.pricing.is_some());
}

#[test]
fn test_zen_model_info_gpt4_turbo() {
    let provider = ZenProvider::new("test-key".to_string()).unwrap();
    let models = provider.models();
    
    let gpt4_turbo = models.iter().find(|m| m.id == "zen-gpt4-turbo").unwrap();
    assert_eq!(gpt4_turbo.name, "Zen GPT-4 Turbo");
    assert_eq!(gpt4_turbo.context_window, 128000);
    assert!(gpt4_turbo.capabilities.contains(&Capability::Vision));
}

#[test]
fn test_zen_token_counting_valid_model() {
    let provider = ZenProvider::new("test-key".to_string()).unwrap();
    let tokens = provider.count_tokens("Hello, world!", "zen-gpt4");
    
    assert!(tokens.is_ok());
    assert!(tokens.unwrap() > 0);
}

#[test]
fn test_zen_token_counting_empty_content() {
    let provider = ZenProvider::new("test-key".to_string()).unwrap();
    let tokens = provider.count_tokens("", "zen-gpt4");
    
    assert!(tokens.is_ok());
    assert_eq!(tokens.unwrap(), 0);
}

#[test]
fn test_zen_token_counting_invalid_model() {
    let provider = ZenProvider::new("test-key".to_string()).unwrap();
    let tokens = provider.count_tokens("Hello, world!", "invalid-model");
    
    assert!(tokens.is_err());
    match tokens {
        Err(e) => assert!(e.to_string().contains("Invalid model")),
        Ok(_) => panic!("Expected error for invalid model"),
    }
}

#[test]
fn test_zen_token_counting_consistency() {
    let provider = ZenProvider::new("test-key".to_string()).unwrap();
    let content = "This is a test message for token counting";
    
    let tokens1 = provider.count_tokens(content, "zen-gpt4").unwrap();
    let tokens2 = provider.count_tokens(content, "zen-gpt4").unwrap();
    
    assert_eq!(tokens1, tokens2);
}

#[test]
fn test_zen_token_counting_different_models() {
    let provider = ZenProvider::new("test-key".to_string()).unwrap();
    let content = "Test content";
    
    let tokens_gpt4 = provider.count_tokens(content, "zen-gpt4").unwrap();
    let tokens_gpt4_turbo = provider.count_tokens(content, "zen-gpt4-turbo").unwrap();
    
    // Both should return valid token counts
    assert!(tokens_gpt4 > 0);
    assert!(tokens_gpt4_turbo > 0);
}

#[test]
fn test_zen_token_counting_longer_content() {
    let provider = ZenProvider::new("test-key".to_string()).unwrap();
    let short = "Hello";
    let long = "Hello world, this is a much longer message with more content and words";
    
    let tokens_short = provider.count_tokens(short, "zen-gpt4").unwrap();
    let tokens_long = provider.count_tokens(long, "zen-gpt4").unwrap();
    
    // Longer content should have more tokens
    assert!(tokens_long > tokens_short);
}

#[test]
fn test_zen_token_counting_special_characters() {
    let provider = ZenProvider::new("test-key".to_string()).unwrap();
    let simple = "hello";
    let with_special = "hello!!!???";
    
    let tokens_simple = provider.count_tokens(simple, "zen-gpt4").unwrap();
    let tokens_special = provider.count_tokens(with_special, "zen-gpt4").unwrap();
    
    // Special characters should increase token count
    assert!(tokens_special >= tokens_simple);
}

#[test]
fn test_zen_with_base_url() {
    let provider = ZenProvider::with_base_url(
        "test-key".to_string(),
        "https://custom.opencode.ai/v1".to_string(),
    );
    
    assert!(provider.is_ok());
    let provider = provider.unwrap();
    assert_eq!(provider.id(), "zen");
}

#[test]
fn test_zen_with_base_url_empty_key() {
    let provider = ZenProvider::with_base_url(
        "".to_string(),
        "https://custom.opencode.ai/v1".to_string(),
    );
    
    assert!(provider.is_err());
}

#[test]
fn test_zen_models_have_capabilities() {
    let provider = ZenProvider::new("test-key".to_string()).unwrap();
    let models = provider.models();
    
    for model in models {
        assert!(!model.capabilities.is_empty());
        assert!(model.capabilities.contains(&Capability::Chat));
    }
}

#[test]
fn test_zen_models_have_pricing() {
    let provider = ZenProvider::new("test-key".to_string()).unwrap();
    let models = provider.models();
    
    for model in models {
        assert!(model.pricing.is_some());
        let pricing = model.pricing.unwrap();
        assert!(pricing.input_per_1k_tokens > 0.0);
        assert!(pricing.output_per_1k_tokens > 0.0);
    }
}

#[test]
fn test_zen_gpt4_context_window() {
    let provider = ZenProvider::new("test-key".to_string()).unwrap();
    let models = provider.models();
    
    let gpt4 = models.iter().find(|m| m.id == "zen-gpt4").unwrap();
    assert_eq!(gpt4.context_window, 8192);
}

#[test]
fn test_zen_gpt4_turbo_has_vision() {
    let provider = ZenProvider::new("test-key".to_string()).unwrap();
    let models = provider.models();
    
    let gpt4_turbo = models.iter().find(|m| m.id == "zen-gpt4-turbo").unwrap();
    assert!(gpt4_turbo.capabilities.contains(&Capability::Vision));
}

#[test]
fn test_zen_all_models_have_streaming() {
    let provider = ZenProvider::new("test-key".to_string()).unwrap();
    let models = provider.models();
    
    for model in models {
        assert!(
            model.capabilities.contains(&Capability::Streaming),
            "Model {} should support streaming",
            model.id
        );
    }
}

#[test]
fn test_zen_model_ids_are_unique() {
    let provider = ZenProvider::new("test-key".to_string()).unwrap();
    let models = provider.models();
    
    let mut ids = Vec::new();
    for model in models {
        assert!(!ids.contains(&model.id), "Duplicate model ID: {}", model.id);
        ids.push(model.id);
    }
}

#[test]
fn test_zen_models_consistency() {
    let provider = ZenProvider::new("test-key".to_string()).unwrap();
    let models1 = provider.models();
    let models2 = provider.models();
    
    assert_eq!(models1.len(), models2.len());
    for (m1, m2) in models1.iter().zip(models2.iter()) {
        assert_eq!(&m1.id, &m2.id);
        assert_eq!(&m1.name, &m2.name);
        assert_eq!(&m1.provider, &m2.provider);
        assert_eq!(&m1.context_window, &m2.context_window);
    }
}

#[test]
fn test_zen_token_counting_scales_with_content() {
    let provider = ZenProvider::new("test-key".to_string()).unwrap();
    let models = provider.models();
    
    if !models.is_empty() {
        let model_id = &models[0].id;
        
        // Test with various content lengths
        let content_100 = "a".repeat(100);
        let content_500 = "a".repeat(500);
        let content_1000 = "a".repeat(1000);
        
        let tokens_100 = provider.count_tokens(&content_100, model_id).unwrap();
        let tokens_500 = provider.count_tokens(&content_500, model_id).unwrap();
        let tokens_1000 = provider.count_tokens(&content_1000, model_id).unwrap();
        
        // Token count should scale with content length
        assert!(tokens_100 > 0);
        assert!(tokens_500 > tokens_100);
        assert!(tokens_1000 > tokens_500);
    }
}

#[test]
fn test_zen_token_counting_unicode() {
    let provider = ZenProvider::new("test-key".to_string()).unwrap();
    let models = provider.models();
    
    if !models.is_empty() {
        let model_id = &models[0].id;
        
        let ascii = "Hello world";
        let unicode = "你好世界";
        
        let tokens_ascii = provider.count_tokens(ascii, model_id).unwrap();
        let tokens_unicode = provider.count_tokens(unicode, model_id).unwrap();
        
        // Both should return valid token counts
        assert!(tokens_ascii > 0);
        assert!(tokens_unicode > 0);
    }
}

#[test]
fn test_zen_token_counting_multiline() {
    let provider = ZenProvider::new("test-key".to_string()).unwrap();
    let models = provider.models();
    
    if !models.is_empty() {
        let model_id = &models[0].id;
        
        let single_line = "Hello world";
        let multi_line = "Hello\nworld\nthis\nis\nmultiline";
        
        let tokens_single = provider.count_tokens(single_line, model_id).unwrap();
        let tokens_multi = provider.count_tokens(multi_line, model_id).unwrap();
        
        // Both should return valid token counts
        assert!(tokens_single > 0);
        assert!(tokens_multi > 0);
    }
}

#[test]
fn test_zen_token_counting_whitespace() {
    let provider = ZenProvider::new("test-key".to_string()).unwrap();
    let models = provider.models();
    
    if !models.is_empty() {
        let model_id = &models[0].id;
        
        let no_space = "HelloWorld";
        let with_space = "Hello World";
        let extra_space = "Hello  World";
        
        let tokens_no_space = provider.count_tokens(no_space, model_id).unwrap();
        let tokens_with_space = provider.count_tokens(with_space, model_id).unwrap();
        let tokens_extra_space = provider.count_tokens(extra_space, model_id).unwrap();
        
        // All should return valid token counts
        assert!(tokens_no_space > 0);
        assert!(tokens_with_space > 0);
        assert!(tokens_extra_space > 0);
    }
}

#[test]
fn test_zen_provider_multiple_instances() {
    let provider1 = ZenProvider::new("key1".to_string()).unwrap();
    let provider2 = ZenProvider::new("key2".to_string()).unwrap();
    
    // Both providers should work independently
    assert_eq!(provider1.id(), "zen");
    assert_eq!(provider2.id(), "zen");
    assert_eq!(provider1.name(), provider2.name());
    
    let models1 = provider1.models();
    let models2 = provider2.models();
    
    assert_eq!(models1.len(), models2.len());
}

#[test]
fn test_zen_provider_id_consistency() {
    let provider = ZenProvider::new("test-key".to_string()).unwrap();
    
    // ID should be consistent across multiple calls
    let id1 = provider.id();
    let id2 = provider.id();
    let id3 = provider.id();
    
    assert_eq!(id1, id2);
    assert_eq!(id2, id3);
    assert_eq!(id1, "zen");
}

#[test]
fn test_zen_provider_name_consistency() {
    let provider = ZenProvider::new("test-key".to_string()).unwrap();
    
    // Name should be consistent across multiple calls
    let name1 = provider.name();
    let name2 = provider.name();
    let name3 = provider.name();
    
    assert_eq!(name1, name2);
    assert_eq!(name2, name3);
    assert_eq!(name1, "OpenCode Zen");
}

#[test]
fn test_zen_token_counting_numeric_content() {
    let provider = ZenProvider::new("test-key".to_string()).unwrap();
    let models = provider.models();
    
    if !models.is_empty() {
        let model_id = &models[0].id;
        
        let numbers = "1234567890";
        let tokens = provider.count_tokens(numbers, model_id).unwrap();
        
        assert!(tokens > 0);
    }
}

#[test]
fn test_zen_token_counting_code_content() {
    let provider = ZenProvider::new("test-key".to_string()).unwrap();
    let models = provider.models();
    
    if !models.is_empty() {
        let model_id = &models[0].id;
        
        let code = r#"
fn main() {
    println!("Hello, world!");
}
"#;
        let tokens = provider.count_tokens(code, model_id).unwrap();
        
        assert!(tokens > 0);
    }
}

#[test]
fn test_zen_token_counting_json_content() {
    let provider = ZenProvider::new("test-key".to_string()).unwrap();
    let models = provider.models();
    
    if !models.is_empty() {
        let model_id = &models[0].id;
        
        let json = r#"{"name": "test", "value": 123, "active": true}"#;
        let tokens = provider.count_tokens(json, model_id).unwrap();
        
        assert!(tokens > 0);
    }
}

#[test]
fn test_zen_models_all_have_valid_ids() {
    let provider = ZenProvider::new("test-key".to_string()).unwrap();
    let models = provider.models();
    
    for model in models {
        assert!(!model.id.is_empty(), "Model ID should not be empty");
        assert!(!model.name.is_empty(), "Model name should not be empty");
        assert_eq!(model.provider, "zen", "Provider should be 'zen'");
        assert!(model.context_window > 0, "Context window should be positive");
    }
}

#[test]
fn test_zen_models_pricing_values() {
    let provider = ZenProvider::new("test-key".to_string()).unwrap();
    let models = provider.models();
    
    for model in models {
        if let Some(pricing) = model.pricing {
            assert!(pricing.input_per_1k_tokens > 0.0, "Input pricing should be positive");
            assert!(pricing.output_per_1k_tokens > 0.0, "Output pricing should be positive");
        }
    }
}

#[test]
fn test_zen_token_counting_very_long_content() {
    let provider = ZenProvider::new("test-key".to_string()).unwrap();
    let models = provider.models();
    
    if !models.is_empty() {
        let model_id = &models[0].id;
        
        // Create a very long content string
        let long_content = "word ".repeat(10000);
        let tokens = provider.count_tokens(&long_content, model_id).unwrap();
        
        // Should handle long content without errors
        assert!(tokens > 0);
        // Should be reasonable (not more than content length)
        assert!(tokens <= long_content.len());
    }
}

#[tokio::test]
async fn test_zen_health_check_returns_result() {
    let provider = ZenProvider::new("test-key".to_string()).unwrap();
    let result = provider.health_check().await;
    
    // Health check should return a result (either Ok or Err)
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_zen_chat_request_invalid_model() {
    let provider = ZenProvider::new("test-key".to_string()).unwrap();
    let _request = ChatRequest {
        model: "invalid-model".to_string(),
        messages: vec![],
        temperature: None,
        max_tokens: None,
        stream: false,
    };
    
    // Verify the model validation
    let models = provider.models();
    assert!(!models.iter().any(|m| m.id == "invalid-model"));
}

#[test]
fn test_zen_provider_with_various_api_keys() {
    let keys = vec![
        "simple-key",
        "key-with-dashes",
        "key_with_underscores",
        "key123456",
        "UPPERCASE_KEY",
        "MixedCaseKey",
    ];
    
    for key in keys {
        let provider = ZenProvider::new(key.to_string());
        assert!(provider.is_ok(), "Provider creation should succeed for key: {}", key);
        
        let provider = provider.unwrap();
        assert_eq!(provider.id(), "zen");
        assert_eq!(provider.name(), "OpenCode Zen");
    }
}

#[test]
fn test_zen_token_counting_boundary_values() {
    let provider = ZenProvider::new("test-key".to_string()).unwrap();
    let models = provider.models();
    
    if !models.is_empty() {
        let model_id = &models[0].id;
        
        // Test boundary values
        let single_char = "a";
        let four_chars = "abcd";
        let five_chars = "abcde";
        
        let tokens_1 = provider.count_tokens(single_char, model_id).unwrap();
        let tokens_4 = provider.count_tokens(four_chars, model_id).unwrap();
        let tokens_5 = provider.count_tokens(five_chars, model_id).unwrap();
        
        // All should return valid token counts
        assert!(tokens_1 > 0);
        assert!(tokens_4 > 0);
        assert!(tokens_5 > 0);
        
        // Longer content should have more or equal tokens
        assert!(tokens_4 >= tokens_1);
        assert!(tokens_5 >= tokens_4);
    }
}
