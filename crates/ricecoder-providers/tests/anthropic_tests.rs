use ricecoder_providers::*;

use ricecoder_providers::*;

#[test]
fn test_anthropic_provider_creation() {
    let provider = AnthropicProvider::new("test-key".to_string());
    assert!(provider.is_ok());
}

#[test]
fn test_anthropic_provider_creation_empty_key() {
    let provider = AnthropicProvider::new("".to_string());
    assert!(provider.is_err());
}

#[test]
fn test_anthropic_provider_id() {
    let provider = AnthropicProvider::new("test-key".to_string()).unwrap();
    assert_eq!(provider.id(), "anthropic");
}

#[test]
fn test_anthropic_provider_name() {
    let provider = AnthropicProvider::new("test-key".to_string()).unwrap();
    assert_eq!(provider.name(), "Anthropic");
}

#[test]
fn test_anthropic_models() {
    let provider = AnthropicProvider::new("test-key".to_string()).unwrap();
    let models = provider.models();
    assert_eq!(models.len(), 4);
    assert!(models.iter().any(|m| m.id == "claude-3-opus-20250219"));
    assert!(models.iter().any(|m| m.id == "claude-3-5-sonnet-20241022"));
    assert!(models.iter().any(|m| m.id == "claude-3-5-haiku-20241022"));
    assert!(models.iter().any(|m| m.id == "claude-3-haiku-20240307"));
}

#[test]
fn test_token_counting() {
    let provider = AnthropicProvider::new("test-key".to_string()).unwrap();
    let tokens = provider
        .count_tokens("Hello, world!", "claude-3-opus-20250219")
        .unwrap();
    assert!(tokens > 0);
}

#[test]
fn test_token_counting_invalid_model() {
    let provider = AnthropicProvider::new("test-key".to_string()).unwrap();
    let result = provider.count_tokens("Hello, world!", "invalid-model");
    assert!(result.is_err());
}
