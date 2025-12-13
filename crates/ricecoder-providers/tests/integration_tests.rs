use ricecoder_providers::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_integration_creation() {
        let integration = ProviderIntegration::new();
        assert!(integration.provider().is_none());
        assert!(integration.model().is_none());
    }

    #[test]
    fn test_provider_integration_with_provider() {
        let integration = ProviderIntegration::with_provider(
            Some("openai".to_string()),
            Some("gpt-4".to_string()),
        );
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
    fn test_provider_display_name() {
        let integration = ProviderIntegration::with_provider(
            Some("openai".to_string()),
            Some("gpt-4".to_string()),
        );
        assert_eq!(integration.provider_display_name(), "OpenAI");
    }

    #[test]
    fn test_model_display_name() {
        let integration = ProviderIntegration::with_provider(
            Some("openai".to_string()),
            Some("gpt-4".to_string()),
        );
        assert_eq!(integration.model_display_name(), "gpt-4");
    }

    #[test]
    fn test_info_string() {
        let integration = ProviderIntegration::with_provider(
            Some("openai".to_string()),
            Some("gpt-4".to_string()),
        );
        assert_eq!(integration.info_string(), "OpenAI (gpt-4)");
    }

    #[test]
    fn test_available_providers() {
        let providers = ProviderIntegration::available_providers();
        assert!(providers.contains(&"openai"));
        assert!(providers.contains(&"anthropic"));
        assert!(providers.contains(&"ollama"));
    }

    #[test]
    fn test_available_models_for_provider() {
        let models = ProviderIntegration::available_models_for_provider("openai");
        assert!(models.contains(&"gpt-4"));
        assert!(models.contains(&"gpt-3.5-turbo"));
    }

    #[test]
    fn test_validate_valid_provider() {
        let integration = ProviderIntegration::with_provider(
            Some("openai".to_string()),
            Some("gpt-4".to_string()),
        );
        assert!(integration.validate().is_ok());
    }

    #[test]
    fn test_validate_invalid_provider() {
        let integration = ProviderIntegration::with_provider(
            Some("invalid".to_string()),
            Some("gpt-4".to_string()),
        );
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
    fn test_streaming_enabled_by_default() {
        let integration = ProviderIntegration::new();
        assert!(integration.is_streaming_enabled());
    }

    #[test]
    fn test_set_streaming_enabled() {
        let mut integration = ProviderIntegration::new();
        integration.set_streaming_enabled(false);
        assert!(!integration.is_streaming_enabled());

        integration.set_streaming_enabled(true);
        assert!(integration.is_streaming_enabled());
    }

    #[test]
    fn test_clone_provider_integration() {
        let integration = ProviderIntegration::with_provider(
            Some("openai".to_string()),
            Some("gpt-4".to_string()),
        );
        let cloned = integration.clone();

        assert_eq!(cloned.provider(), integration.provider());
        assert_eq!(cloned.model(), integration.model());
        assert_eq!(
            cloned.is_streaming_enabled(),
            integration.is_streaming_enabled()
        );
    }
}