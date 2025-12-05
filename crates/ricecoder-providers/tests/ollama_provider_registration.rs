//! Integration tests for Ollama provider registration with ProviderRegistry
//! Tests provider registration, discovery, default provider selection, and configuration loading
//! **Feature: ricecoder-local-models, Integration Tests: Provider Registration**
//! **Validates: Requirements 1.1, 1.2**

use ricecoder_providers::{OllamaProvider, Provider, ProviderRegistry};
use std::sync::Arc;

/// Test: OllamaProvider registration with ProviderRegistry
/// For any OllamaProvider instance, registering with ProviderRegistry SHALL succeed
#[test]
fn test_ollama_provider_registration_success() {
    let mut registry = ProviderRegistry::new();
    let provider = Arc::new(OllamaProvider::new("http://localhost:11434".to_string()).unwrap());

    let result = registry.register(provider);

    assert!(result.is_ok());
    assert!(registry.has_provider("ollama"));
}

/// Test: Provider discovery by ID
/// For any registered OllamaProvider, discovering by ID "ollama" SHALL return the provider
#[test]
fn test_ollama_provider_discovery_by_id() {
    let mut registry = ProviderRegistry::new();
    let provider = Arc::new(OllamaProvider::new("http://localhost:11434".to_string()).unwrap());

    registry.register(provider).unwrap();

    let discovered = registry.get("ollama");
    assert!(discovered.is_ok());

    let discovered_provider = discovered.unwrap();
    assert_eq!(discovered_provider.id(), "ollama");
    assert_eq!(discovered_provider.name(), "Ollama");
}

/// Test: Provider discovery by name
/// For any registered OllamaProvider, discovering by name "Ollama" SHALL return the provider
#[test]
fn test_ollama_provider_discovery_by_name() {
    let mut registry = ProviderRegistry::new();
    let provider = Arc::new(OllamaProvider::new("http://localhost:11434".to_string()).unwrap());

    registry.register(provider).unwrap();

    let discovered = registry.get_by_name("Ollama");
    assert!(discovered.is_ok());

    let discovered_provider = discovered.unwrap();
    assert_eq!(discovered_provider.id(), "ollama");
}

/// Test: Provider not found when not registered
/// For any unregistered provider ID, discovering SHALL return error
#[test]
fn test_ollama_provider_not_found_when_not_registered() {
    let registry = ProviderRegistry::new();

    let result = registry.get("ollama");
    assert!(result.is_err());
}

/// Test: Multiple providers can be registered
/// For any ProviderRegistry, registering multiple providers SHALL succeed
#[test]
fn test_multiple_providers_registration() {
    let mut registry = ProviderRegistry::new();

    let ollama_provider =
        Arc::new(OllamaProvider::new("http://localhost:11434".to_string()).unwrap());

    // Create a mock provider for testing
    let mock_provider = Arc::new(MockProvider {
        id: "mock".to_string(),
        name: "Mock Provider".to_string(),
    });

    registry.register(ollama_provider).unwrap();
    registry.register(mock_provider).unwrap();

    assert_eq!(registry.provider_count(), 2);
    assert!(registry.has_provider("ollama"));
    assert!(registry.has_provider("mock"));
}

/// Test: Provider unregistration
/// For any registered provider, unregistering SHALL succeed and provider SHALL no longer be discoverable
#[test]
fn test_ollama_provider_unregistration() {
    let mut registry = ProviderRegistry::new();
    let provider = Arc::new(OllamaProvider::new("http://localhost:11434".to_string()).unwrap());

    registry.register(provider).unwrap();
    assert!(registry.has_provider("ollama"));

    let result = registry.unregister("ollama");
    assert!(result.is_ok());
    assert!(!registry.has_provider("ollama"));
}

/// Test: Unregistering non-existent provider fails
/// For any non-existent provider ID, unregistering SHALL return error
#[test]
fn test_unregister_non_existent_provider_fails() {
    let mut registry = ProviderRegistry::new();

    let result = registry.unregister("non-existent");
    assert!(result.is_err());
}

/// Test: List all registered providers
/// For any ProviderRegistry with registered providers, listing SHALL return all providers
#[test]
fn test_list_all_registered_providers() {
    let mut registry = ProviderRegistry::new();

    let ollama_provider =
        Arc::new(OllamaProvider::new("http://localhost:11434".to_string()).unwrap());

    let mock_provider = Arc::new(MockProvider {
        id: "mock".to_string(),
        name: "Mock Provider".to_string(),
    });

    registry.register(ollama_provider).unwrap();
    registry.register(mock_provider).unwrap();

    let all_providers = registry.list_all();
    assert_eq!(all_providers.len(), 2);

    let ids: Vec<&str> = all_providers.iter().map(|p| p.id()).collect();
    assert!(ids.contains(&"ollama"));
    assert!(ids.contains(&"mock"));
}

/// Test: List models from registered provider
/// For any registered OllamaProvider, listing models SHALL return available models
#[test]
fn test_list_models_from_registered_provider() {
    let mut registry = ProviderRegistry::new();
    let provider = Arc::new(OllamaProvider::new("http://localhost:11434".to_string()).unwrap());

    registry.register(provider).unwrap();

    let models = registry.list_models("ollama");
    assert!(models.is_ok());

    let model_list = models.unwrap();
    assert!(!model_list.is_empty());
    assert!(model_list.iter().any(|m| m.id == "mistral"));
}

/// Test: List models from non-existent provider fails
/// For any non-existent provider ID, listing models SHALL return error
#[test]
fn test_list_models_from_non_existent_provider_fails() {
    let registry = ProviderRegistry::new();

    let result = registry.list_models("non-existent");
    assert!(result.is_err());
}

/// Test: List all models across all providers
/// For any ProviderRegistry with multiple providers, listing all models SHALL return models from all providers
#[test]
fn test_list_all_models_across_providers() {
    let mut registry = ProviderRegistry::new();

    let ollama_provider =
        Arc::new(OllamaProvider::new("http://localhost:11434".to_string()).unwrap());

    let mock_provider = Arc::new(MockProvider {
        id: "mock".to_string(),
        name: "Mock Provider".to_string(),
    });

    registry.register(ollama_provider).unwrap();
    registry.register(mock_provider).unwrap();

    let all_models = registry.list_all_models();

    // Should have models from Ollama provider
    assert!(!all_models.is_empty());
    assert!(all_models.iter().any(|m| m.provider == "ollama"));
}

/// Test: Provider count
/// For any ProviderRegistry, provider_count() SHALL return correct number of registered providers
#[test]
fn test_provider_count() {
    let mut registry = ProviderRegistry::new();
    assert_eq!(registry.provider_count(), 0);

    let provider1 = Arc::new(OllamaProvider::new("http://localhost:11434".to_string()).unwrap());
    registry.register(provider1).unwrap();
    assert_eq!(registry.provider_count(), 1);

    let provider2 = Arc::new(MockProvider {
        id: "mock".to_string(),
        name: "Mock Provider".to_string(),
    });
    registry.register(provider2).unwrap();
    assert_eq!(registry.provider_count(), 2);
}

/// Test: Provider registration with custom base URL
/// For any OllamaProvider with custom base URL, registration SHALL succeed
#[test]
fn test_ollama_provider_registration_custom_url() {
    let mut registry = ProviderRegistry::new();
    let provider = Arc::new(OllamaProvider::new("http://custom-host:11434".to_string()).unwrap());

    let result = registry.register(provider);
    assert!(result.is_ok());

    let discovered = registry.get("ollama").unwrap();
    assert_eq!(discovered.id(), "ollama");
}

/// Test: Provider registration with default endpoint
/// For any OllamaProvider with default endpoint, registration SHALL succeed
#[test]
fn test_ollama_provider_registration_default_endpoint() {
    let mut registry = ProviderRegistry::new();
    let provider = Arc::new(OllamaProvider::with_default_endpoint().unwrap());

    let result = registry.register(provider);
    assert!(result.is_ok());
    assert!(registry.has_provider("ollama"));
}

/// Test: Provider metadata accessible after registration
/// For any registered OllamaProvider, id() and name() SHALL return correct values
#[test]
fn test_provider_metadata_after_registration() {
    let mut registry = ProviderRegistry::new();
    let provider = Arc::new(OllamaProvider::new("http://localhost:11434".to_string()).unwrap());

    registry.register(provider).unwrap();

    let discovered = registry.get("ollama").unwrap();
    assert_eq!(discovered.id(), "ollama");
    assert_eq!(discovered.name(), "Ollama");
}

/// Test: Provider models accessible after registration
/// For any registered OllamaProvider, models() SHALL return available models
#[test]
fn test_provider_models_after_registration() {
    let mut registry = ProviderRegistry::new();
    let provider = Arc::new(OllamaProvider::new("http://localhost:11434".to_string()).unwrap());

    registry.register(provider).unwrap();

    let discovered = registry.get("ollama").unwrap();
    let models = discovered.models();

    assert!(!models.is_empty());
    assert!(models.iter().any(|m| m.provider == "ollama"));
}

/// Test: Re-registering same provider ID replaces previous
/// For any ProviderRegistry, registering a provider with same ID as existing SHALL replace the previous
#[test]
fn test_re_register_provider_replaces_previous() {
    let mut registry = ProviderRegistry::new();

    let provider1 = Arc::new(OllamaProvider::new("http://localhost:11434".to_string()).unwrap());
    registry.register(provider1).unwrap();

    let provider2 =
        Arc::new(OllamaProvider::new("http://different-host:11434".to_string()).unwrap());
    registry.register(provider2).unwrap();

    // Should still have only one provider
    assert_eq!(registry.provider_count(), 1);
    assert!(registry.has_provider("ollama"));
}

/// Test: Empty registry operations
/// For any empty ProviderRegistry, operations SHALL handle gracefully
#[test]
fn test_empty_registry_operations() {
    let registry = ProviderRegistry::new();

    assert_eq!(registry.provider_count(), 0);
    assert!(!registry.has_provider("ollama"));
    assert!(registry.get("ollama").is_err());
    assert_eq!(registry.list_all().len(), 0);
    assert_eq!(registry.list_all_models().len(), 0);
}

/// Test: Provider discovery consistency
/// For any registered provider, multiple discovery calls SHALL return consistent results
#[test]
fn test_provider_discovery_consistency() {
    let mut registry = ProviderRegistry::new();
    let provider = Arc::new(OllamaProvider::new("http://localhost:11434".to_string()).unwrap());

    registry.register(provider).unwrap();

    let discovered1 = registry.get("ollama").unwrap();
    let discovered2 = registry.get("ollama").unwrap();

    assert_eq!(discovered1.id(), discovered2.id());
    assert_eq!(discovered1.name(), discovered2.name());
}

// ============================================================================
// Mock Provider for Testing
// ============================================================================

struct MockProvider {
    id: String,
    name: String,
}

#[async_trait::async_trait]
impl Provider for MockProvider {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn models(&self) -> Vec<ricecoder_providers::ModelInfo> {
        vec![]
    }

    async fn chat(
        &self,
        _request: ricecoder_providers::ChatRequest,
    ) -> Result<ricecoder_providers::ChatResponse, ricecoder_providers::ProviderError> {
        Err(ricecoder_providers::ProviderError::NotFound(
            "Not implemented".to_string(),
        ))
    }

    async fn chat_stream(
        &self,
        _request: ricecoder_providers::ChatRequest,
    ) -> Result<ricecoder_providers::provider::ChatStream, ricecoder_providers::ProviderError> {
        Err(ricecoder_providers::ProviderError::NotFound(
            "Not implemented".to_string(),
        ))
    }

    fn count_tokens(
        &self,
        _content: &str,
        _model: &str,
    ) -> Result<usize, ricecoder_providers::ProviderError> {
        Ok(0)
    }

    async fn health_check(&self) -> Result<bool, ricecoder_providers::ProviderError> {
        Ok(true)
    }
}
