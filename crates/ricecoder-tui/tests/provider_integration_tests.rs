//! Integration tests for provider integration

use ricecoder_tui::ProviderIntegration;

#[test]
fn test_provider_integration_creation() {
    let integration = ProviderIntegration::new();
    assert!(integration.provider().is_none());
    assert!(integration.model().is_none());
}

#[test]
fn test_provider_integration_with_provider() {
    let integration =
        ProviderIntegration::with_provider(Some("openai".to_string()), Some("gpt-4".to_string()));
    assert_eq!(integration.provider(), Some("openai"));
    assert_eq!(integration.model(), Some("gpt-4"));
}

#[test]
fn test_set_provider() {
    let mut integration = ProviderIntegration::new();
    integration.set_provider("anthropic".to_string());
    assert_eq!(integration.provider(), Some("anthropic"));
}

#[test]
fn test_set_model() {
    let mut integration = ProviderIntegration::new();
    integration.set_model("gpt-4".to_string());
    assert_eq!(integration.model(), Some("gpt-4"));
}

#[test]
fn test_provider_display_name_openai() {
    let integration =
        ProviderIntegration::with_provider(Some("openai".to_string()), Some("gpt-4".to_string()));
    assert_eq!(integration.provider_display_name(), "OpenAI");
}

#[test]
fn test_provider_display_name_anthropic() {
    let integration = ProviderIntegration::with_provider(
        Some("anthropic".to_string()),
        Some("claude-3-opus".to_string()),
    );
    assert_eq!(integration.provider_display_name(), "Anthropic");
}

#[test]
fn test_provider_display_name_ollama() {
    let integration =
        ProviderIntegration::with_provider(Some("ollama".to_string()), Some("llama2".to_string()));
    assert_eq!(integration.provider_display_name(), "Ollama");
}

#[test]
fn test_model_display_name() {
    let integration =
        ProviderIntegration::with_provider(Some("openai".to_string()), Some("gpt-4".to_string()));
    assert_eq!(integration.model_display_name(), "gpt-4");
}

#[test]
fn test_info_string_with_provider_and_model() {
    let integration =
        ProviderIntegration::with_provider(Some("openai".to_string()), Some("gpt-4".to_string()));
    assert_eq!(integration.info_string(), "OpenAI (gpt-4)");
}

#[test]
fn test_info_string_with_provider_only() {
    let integration = ProviderIntegration::with_provider(Some("openai".to_string()), None);
    assert_eq!(integration.info_string(), "OpenAI");
}

#[test]
fn test_info_string_no_provider() {
    let integration = ProviderIntegration::new();
    assert_eq!(integration.info_string(), "No Provider");
}

#[test]
fn test_available_providers() {
    let providers = ProviderIntegration::available_providers();
    assert!(providers.contains(&"openai"));
    assert!(providers.contains(&"anthropic"));
    assert!(providers.contains(&"ollama"));
    assert!(providers.contains(&"google"));
    assert!(providers.contains(&"zen"));
}

#[test]
fn test_available_models_for_openai() {
    let models = ProviderIntegration::available_models_for_provider("openai");
    assert!(models.contains(&"gpt-4"));
    assert!(models.contains(&"gpt-4-turbo"));
    assert!(models.contains(&"gpt-3.5-turbo"));
}

#[test]
fn test_available_models_for_anthropic() {
    let models = ProviderIntegration::available_models_for_provider("anthropic");
    assert!(models.contains(&"claude-3-opus"));
    assert!(models.contains(&"claude-3-sonnet"));
    assert!(models.contains(&"claude-3-haiku"));
}

#[test]
fn test_available_models_for_ollama() {
    let models = ProviderIntegration::available_models_for_provider("ollama");
    assert!(models.contains(&"llama2"));
    assert!(models.contains(&"mistral"));
    assert!(models.contains(&"neural-chat"));
}

#[test]
fn test_validate_valid_provider() {
    let integration =
        ProviderIntegration::with_provider(Some("openai".to_string()), Some("gpt-4".to_string()));
    assert!(integration.validate().is_ok());
}

#[test]
fn test_validate_invalid_provider() {
    let integration =
        ProviderIntegration::with_provider(Some("invalid".to_string()), Some("gpt-4".to_string()));
    assert!(integration.validate().is_err());
}

#[test]
fn test_validate_invalid_model() {
    let integration = ProviderIntegration::with_provider(
        Some("openai".to_string()),
        Some("invalid-model".to_string()),
    );
    assert!(integration.validate().is_err());
}

#[test]
fn test_validate_no_provider() {
    let integration = ProviderIntegration::new();
    assert!(integration.validate().is_ok());
}

#[test]
fn test_has_provider() {
    let mut integration = ProviderIntegration::new();
    assert!(!integration.has_provider());

    integration.set_provider("openai".to_string());
    assert!(integration.has_provider());
}

#[test]
fn test_has_model() {
    let mut integration = ProviderIntegration::new();
    assert!(!integration.has_model());

    integration.set_model("gpt-4".to_string());
    assert!(integration.has_model());
}

#[test]
fn test_provider_integration_default() {
    let integration = ProviderIntegration::default();
    assert!(integration.provider().is_none());
    assert!(integration.model().is_none());
}

#[test]
fn test_multiple_provider_switches() {
    let mut integration = ProviderIntegration::new();

    integration.set_provider("openai".to_string());
    assert_eq!(integration.provider(), Some("openai"));

    integration.set_provider("anthropic".to_string());
    assert_eq!(integration.provider(), Some("anthropic"));

    integration.set_provider("ollama".to_string());
    assert_eq!(integration.provider(), Some("ollama"));
}

#[test]
fn test_provider_and_model_combination() {
    let mut integration = ProviderIntegration::new();

    integration.set_provider("openai".to_string());
    integration.set_model("gpt-4".to_string());

    assert_eq!(integration.info_string(), "OpenAI (gpt-4)");
    assert!(integration.validate().is_ok());

    integration.set_model("gpt-3.5-turbo".to_string());
    assert_eq!(integration.info_string(), "OpenAI (gpt-3.5-turbo)");
    assert!(integration.validate().is_ok());
}
