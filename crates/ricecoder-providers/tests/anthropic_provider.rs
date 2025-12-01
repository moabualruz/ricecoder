//! Unit tests for Anthropic provider implementation

use ricecoder_providers::{AnthropicProvider, Capability, ChatRequest, Provider};

#[test]
fn test_anthropic_provider_creation_success() {
    let provider = AnthropicProvider::new("sk-ant-test-key".to_string());
    assert!(provider.is_ok());
}

#[test]
fn test_anthropic_provider_creation_empty_key() {
    let provider = AnthropicProvider::new("".to_string());
    assert!(provider.is_err());
    match provider {
        Err(e) => assert!(e.to_string().contains("API key is required")),
        Ok(_) => panic!("Expected error for empty API key"),
    }
}

#[test]
fn test_anthropic_provider_id() {
    let provider = AnthropicProvider::new("sk-ant-test-key".to_string()).unwrap();
    assert_eq!(provider.id(), "anthropic");
}

#[test]
fn test_anthropic_provider_name() {
    let provider = AnthropicProvider::new("sk-ant-test-key".to_string()).unwrap();
    assert_eq!(provider.name(), "Anthropic");
}

#[test]
fn test_anthropic_models_available() {
    let provider = AnthropicProvider::new("sk-ant-test-key".to_string()).unwrap();
    let models = provider.models();

    assert_eq!(models.len(), 4);
    assert!(models.iter().any(|m| m.id == "claude-3-opus-20250219"));
    assert!(models.iter().any(|m| m.id == "claude-3-5-sonnet-20241022"));
    assert!(models.iter().any(|m| m.id == "claude-3-5-haiku-20241022"));
    assert!(models.iter().any(|m| m.id == "claude-3-haiku-20240307"));
}

#[test]
fn test_anthropic_model_info_opus() {
    let provider = AnthropicProvider::new("sk-ant-test-key".to_string()).unwrap();
    let models = provider.models();

    let opus = models
        .iter()
        .find(|m| m.id == "claude-3-opus-20250219")
        .unwrap();
    assert_eq!(opus.name, "Claude 3 Opus");
    assert_eq!(opus.provider, "anthropic");
    assert_eq!(opus.context_window, 200000);
    assert!(opus.pricing.is_some());
}

#[test]
fn test_anthropic_model_info_sonnet() {
    let provider = AnthropicProvider::new("sk-ant-test-key".to_string()).unwrap();
    let models = provider.models();

    let sonnet = models
        .iter()
        .find(|m| m.id == "claude-3-5-sonnet-20241022")
        .unwrap();
    assert_eq!(sonnet.name, "Claude 3.5 Sonnet");
    assert_eq!(sonnet.context_window, 200000);
    assert!(sonnet.capabilities.contains(&Capability::Chat));
    assert!(sonnet.capabilities.contains(&Capability::Code));
}

#[test]
fn test_anthropic_model_info_haiku() {
    let provider = AnthropicProvider::new("sk-ant-test-key".to_string()).unwrap();
    let models = provider.models();

    let haiku = models
        .iter()
        .find(|m| m.id == "claude-3-5-haiku-20241022")
        .unwrap();
    assert_eq!(haiku.name, "Claude 3.5 Haiku");
    assert_eq!(haiku.context_window, 200000);
}

#[test]
fn test_anthropic_token_counting_valid_model() {
    let provider = AnthropicProvider::new("sk-ant-test-key".to_string()).unwrap();
    let tokens = provider.count_tokens("Hello, world!", "claude-3-opus-20250219");

    assert!(tokens.is_ok());
    assert!(tokens.unwrap() > 0);
}

#[test]
fn test_anthropic_token_counting_empty_content() {
    let provider = AnthropicProvider::new("sk-ant-test-key".to_string()).unwrap();
    let tokens = provider.count_tokens("", "claude-3-opus-20250219");

    assert!(tokens.is_ok());
    assert_eq!(tokens.unwrap(), 0);
}

#[test]
fn test_anthropic_token_counting_invalid_model() {
    let provider = AnthropicProvider::new("sk-ant-test-key".to_string()).unwrap();
    let tokens = provider.count_tokens("Hello, world!", "invalid-model");

    assert!(tokens.is_err());
    match tokens {
        Err(e) => assert!(e.to_string().contains("Invalid model")),
        Ok(_) => panic!("Expected error for invalid model"),
    }
}

#[test]
fn test_anthropic_token_counting_consistency() {
    let provider = AnthropicProvider::new("sk-ant-test-key".to_string()).unwrap();
    let content = "This is a test message for token counting";

    let tokens1 = provider.count_tokens(content, "claude-3-opus-20250219").unwrap();
    let tokens2 = provider.count_tokens(content, "claude-3-opus-20250219").unwrap();

    assert_eq!(tokens1, tokens2);
}

#[test]
fn test_anthropic_token_counting_different_models() {
    let provider = AnthropicProvider::new("sk-ant-test-key".to_string()).unwrap();
    let content = "Test content";

    let tokens_opus = provider.count_tokens(content, "claude-3-opus-20250219").unwrap();
    let tokens_sonnet = provider.count_tokens(content, "claude-3-5-sonnet-20241022").unwrap();

    // Both should return valid token counts
    assert!(tokens_opus > 0);
    assert!(tokens_sonnet > 0);
}

#[test]
fn test_anthropic_token_counting_longer_content() {
    let provider = AnthropicProvider::new("sk-ant-test-key".to_string()).unwrap();
    let short = "Hello";
    let long = "Hello world, this is a much longer message with more content and words";

    let tokens_short = provider.count_tokens(short, "claude-3-opus-20250219").unwrap();
    let tokens_long = provider.count_tokens(long, "claude-3-opus-20250219").unwrap();

    // Longer content should have more tokens
    assert!(tokens_long > tokens_short);
}

#[test]
fn test_anthropic_token_counting_special_characters() {
    let provider = AnthropicProvider::new("sk-ant-test-key".to_string()).unwrap();
    let simple = "hello";
    let with_special = "hello!!!???";

    let tokens_simple = provider.count_tokens(simple, "claude-3-opus-20250219").unwrap();
    let tokens_special = provider.count_tokens(with_special, "claude-3-opus-20250219").unwrap();

    // Special characters should increase token count
    assert!(tokens_special >= tokens_simple);
}

#[test]
fn test_anthropic_with_base_url() {
    let provider = AnthropicProvider::with_base_url(
        "sk-ant-test-key".to_string(),
        "https://custom.anthropic.com/v1".to_string(),
    );

    assert!(provider.is_ok());
    let provider = provider.unwrap();
    assert_eq!(provider.id(), "anthropic");
}

#[test]
fn test_anthropic_with_base_url_empty_key() {
    let provider = AnthropicProvider::with_base_url(
        "".to_string(),
        "https://custom.anthropic.com/v1".to_string(),
    );

    assert!(provider.is_err());
}

#[test]
fn test_anthropic_chat_request_invalid_model() {
    let provider = AnthropicProvider::new("sk-ant-test-key".to_string()).unwrap();
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
fn test_anthropic_models_have_capabilities() {
    let provider = AnthropicProvider::new("sk-ant-test-key".to_string()).unwrap();
    let models = provider.models();

    for model in models {
        assert!(!model.capabilities.is_empty());
        assert!(model.capabilities.contains(&Capability::Chat));
    }
}

#[test]
fn test_anthropic_models_have_pricing() {
    let provider = AnthropicProvider::new("sk-ant-test-key".to_string()).unwrap();
    let models = provider.models();

    for model in models {
        assert!(model.pricing.is_some());
        let pricing = model.pricing.unwrap();
        assert!(pricing.input_per_1k_tokens > 0.0);
        assert!(pricing.output_per_1k_tokens > 0.0);
    }
}

#[test]
fn test_anthropic_all_models_have_streaming() {
    let provider = AnthropicProvider::new("sk-ant-test-key".to_string()).unwrap();
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
fn test_anthropic_all_models_have_code_capability() {
    let provider = AnthropicProvider::new("sk-ant-test-key".to_string()).unwrap();
    let models = provider.models();

    for model in models {
        assert!(
            model.capabilities.contains(&Capability::Code),
            "Model {} should support code generation",
            model.id
        );
    }
}

#[test]
fn test_anthropic_opus_pricing() {
    let provider = AnthropicProvider::new("sk-ant-test-key".to_string()).unwrap();
    let models = provider.models();

    let opus = models
        .iter()
        .find(|m| m.id == "claude-3-opus-20250219")
        .unwrap();
    let pricing = opus.pricing.as_ref().unwrap();

    // Opus should be more expensive than Haiku
    let haiku = models
        .iter()
        .find(|m| m.id == "claude-3-5-haiku-20241022")
        .unwrap();
    let haiku_pricing = haiku.pricing.as_ref().unwrap();

    assert!(pricing.input_per_1k_tokens > haiku_pricing.input_per_1k_tokens);
}

#[test]
fn test_anthropic_sonnet_pricing() {
    let provider = AnthropicProvider::new("sk-ant-test-key".to_string()).unwrap();
    let models = provider.models();

    let sonnet = models
        .iter()
        .find(|m| m.id == "claude-3-5-sonnet-20241022")
        .unwrap();
    let pricing = sonnet.pricing.as_ref().unwrap();

    // Sonnet should be cheaper than Opus
    let opus = models
        .iter()
        .find(|m| m.id == "claude-3-opus-20250219")
        .unwrap();
    let opus_pricing = opus.pricing.as_ref().unwrap();

    assert!(pricing.input_per_1k_tokens < opus_pricing.input_per_1k_tokens);
}

#[test]
fn test_anthropic_haiku_pricing() {
    let provider = AnthropicProvider::new("sk-ant-test-key".to_string()).unwrap();
    let models = provider.models();

    let haiku = models
        .iter()
        .find(|m| m.id == "claude-3-5-haiku-20241022")
        .unwrap();
    let pricing = haiku.pricing.as_ref().unwrap();

    // Haiku should be the cheapest
    assert!(pricing.input_per_1k_tokens < 0.001);
}

#[test]
fn test_anthropic_all_models_have_context_window() {
    let provider = AnthropicProvider::new("sk-ant-test-key".to_string()).unwrap();
    let models = provider.models();

    for model in models {
        assert!(model.context_window > 0);
        // All Claude models should have at least 200k context window
        assert_eq!(model.context_window, 200000);
    }
}
