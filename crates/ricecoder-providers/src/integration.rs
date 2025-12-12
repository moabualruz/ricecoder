//! Provider integration utilities
//!
//! This module provides utilities for integrating with AI providers,
//! including configuration management and streaming support.

use crate::error::ProviderError;

/// Streaming response handler
pub type StreamHandler = Box<dyn Fn(String) + Send + Sync>;

/// Provider integration utilities
pub struct ProviderIntegration {
    /// Current provider name
    pub current_provider: Option<String>,
    /// Current model name
    pub current_model: Option<String>,
    /// Whether streaming is enabled
    pub streaming_enabled: bool,
    /// Stream handler for processing tokens
    pub stream_handler: Option<StreamHandler>,
}

impl ProviderIntegration {
    /// Create a new provider integration
    pub fn new() -> Self {
        Self {
            current_provider: None,
            current_model: None,
            streaming_enabled: true,
            stream_handler: None,
        }
    }

    /// Create with specific provider and model
    pub fn with_provider(provider: Option<String>, model: Option<String>) -> Self {
        Self {
            current_provider: provider,
            current_model: model,
            streaming_enabled: true,
            stream_handler: None,
        }
    }

    /// Enable or disable streaming
    pub fn set_streaming_enabled(&mut self, enabled: bool) {
        self.streaming_enabled = enabled;
    }

    /// Check if streaming is enabled
    pub fn is_streaming_enabled(&self) -> bool {
        self.streaming_enabled
    }

    /// Set the stream handler for processing tokens
    pub fn set_stream_handler(&mut self, handler: StreamHandler) {
        self.stream_handler = Some(handler);
    }

    /// Handle a streamed token
    pub fn handle_token(&self, token: String) {
        if let Some(ref handler) = self.stream_handler {
            handler(token);
        }
    }

    /// Set the current provider
    pub fn set_provider(&mut self, provider: String) {
        self.current_provider = Some(provider);
    }

    /// Set the current model
    pub fn set_model(&mut self, model: String) {
        self.current_model = Some(model);
    }

    /// Get the current provider
    pub fn provider(&self) -> Option<&str> {
        self.current_provider.as_deref()
    }

    /// Get the current model
    pub fn model(&self) -> Option<&str> {
        self.current_model.as_deref()
    }

    /// Check if a provider is configured
    pub fn has_provider(&self) -> bool {
        self.current_provider.is_some()
    }

    /// Check if a model is configured
    pub fn has_model(&self) -> bool {
        self.current_model.is_some()
    }

    /// Get provider display name
    pub fn provider_display_name(&self) -> String {
        match self.current_provider.as_deref() {
            Some("openai") => "OpenAI".to_string(),
            Some("anthropic") => "Anthropic".to_string(),
            Some("ollama") => "Ollama".to_string(),
            Some("google") => "Google".to_string(),
            Some("zen") => "Zen".to_string(),
            Some(other) => other.to_string(),
            None => "No Provider".to_string(),
        }
    }

    /// Get model display name
    pub fn model_display_name(&self) -> String {
        self.current_model
            .as_deref()
            .unwrap_or("No Model")
            .to_string()
    }

    /// Get full provider info string
    pub fn info_string(&self) -> String {
        match (self.provider(), self.model()) {
            (Some(_), Some(model)) => format!("{} ({})", self.provider_display_name(), model),
            (Some(_), None) => self.provider_display_name(),
            (None, _) => "No Provider".to_string(),
        }
    }

    /// List available providers
    pub fn available_providers() -> Vec<&'static str> {
        vec!["openai", "anthropic", "ollama", "google", "zen"]
    }

    /// List available models for a provider
    pub fn available_models_for_provider(provider: &str) -> Vec<&'static str> {
        match provider {
            "openai" => vec!["gpt-4", "gpt-4-turbo", "gpt-3.5-turbo"],
            "anthropic" => vec!["claude-3-opus", "claude-3-sonnet", "claude-3-haiku"],
            "ollama" => vec!["llama2", "mistral", "neural-chat"],
            "google" => vec!["gemini-pro", "palm-2"],
            "zen" => vec!["zen-default"],
            _ => vec![],
        }
    }

    /// Validate provider and model combination
    pub fn validate(&self) -> Result<(), ProviderError> {
        if let Some(provider) = self.provider() {
            if !Self::available_providers().contains(&provider) {
                return Err(ProviderError::NotFound(provider.to_string()));
            }

            if let Some(model) = self.model() {
                let available = Self::available_models_for_provider(provider);
                if !available.contains(&model) {
                    return Err(ProviderError::InvalidModel(model.to_string()));
                }
            }
        }

        Ok(())
    }
}

impl Default for ProviderIntegration {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ProviderIntegration {
    fn clone(&self) -> Self {
        Self {
            current_provider: self.current_provider.clone(),
            current_model: self.current_model.clone(),
            streaming_enabled: self.streaming_enabled,
            stream_handler: None, // Stream handlers cannot be cloned
        }
    }
}

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