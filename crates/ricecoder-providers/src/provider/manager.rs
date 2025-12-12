//! Provider manager for orchestrating provider operations

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use super::{ChatStream, Provider, ProviderRegistry};
use crate::error::ProviderError;
use crate::health_check::HealthCheckCache;
use crate::models::{Capability, ChatRequest, ChatResponse, ModelInfo};

/// Provider connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Provider is available and healthy
    Connected,
    /// Provider is temporarily unavailable
    Disconnected,
    /// Provider encountered an error
    Error,
    /// Provider is disabled/configured incorrectly
    Disabled,
}

/// Model filtering criteria
#[derive(Debug, Clone)]
pub enum ModelFilterCriteria {
    /// Filter by provider name
    Provider(String),
    /// Filter by required capability
    Capability(Capability),
    /// Only free models
    FreeOnly,
    /// Minimum context window size
    MinContextWindow(usize),
    /// Maximum cost per token
    MaxCostPerToken(f64),
}

/// Model filter for advanced filtering
#[derive(Debug, Clone)]
pub struct ModelFilter {
    criteria: Vec<ModelFilterCriteria>,
}

impl ModelFilter {
    /// Create a new filter
    pub fn new() -> Self {
        Self {
            criteria: Vec::new(),
        }
    }

    /// Add a filter criterion
    pub fn with_criterion(mut self, criterion: ModelFilterCriteria) -> Self {
        self.criteria.push(criterion);
        self
    }

    /// Check if a model matches all filter criteria
    pub fn matches(&self, model: &ModelInfo) -> bool {
        self.criteria.iter().all(|criterion| {
            match criterion {
                ModelFilterCriteria::Provider(ref provider) => model.provider == *provider,
                ModelFilterCriteria::Capability(ref capability) => model.capabilities.contains(capability),
                ModelFilterCriteria::FreeOnly => model.is_free,
                ModelFilterCriteria::MinContextWindow(min_tokens) => model.context_window >= *min_tokens,
                ModelFilterCriteria::MaxCostPerToken(max_cost) => {
                    if let Some(ref pricing) = model.pricing {
                        pricing.input_per_1k_tokens <= *max_cost || pricing.output_per_1k_tokens <= *max_cost
                    } else {
                        true // No pricing info means we can't filter
                    }
                }
            }
        })
    }
}

impl Default for ModelFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// Enhanced provider information with connection state
#[derive(Debug, Clone)]
pub struct ProviderStatus {
    /// Provider ID
    pub id: String,
    /// Provider name
    pub name: String,
    /// Current connection state
    pub state: ConnectionState,
    /// Last health check time
    pub last_checked: Option<std::time::SystemTime>,
    /// Available models
    pub models: Vec<ModelInfo>,
    /// Error message if any
    pub error_message: Option<String>,
}

/// Central coordinator for provider operations
pub struct ProviderManager {
    registry: ProviderRegistry,
    default_provider_id: String,
    retry_count: usize,
    timeout: Duration,
    health_check_cache: Arc<HealthCheckCache>,
    provider_states: HashMap<String, ProviderStatus>,
}

impl ProviderManager {
    /// Create a new provider manager
    pub fn new(registry: ProviderRegistry, default_provider_id: String) -> Self {
        Self {
            registry,
            default_provider_id,
            retry_count: 3,
            timeout: Duration::from_secs(30),
            health_check_cache: Arc::new(HealthCheckCache::default()),
            provider_states: HashMap::new(),
        }
    }

    /// Set the number of retries for failed requests
    pub fn with_retry_count(mut self, count: usize) -> Self {
        self.retry_count = count;
        self
    }

    /// Set the request timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the health check cache
    pub fn with_health_check_cache(mut self, cache: Arc<HealthCheckCache>) -> Self {
        self.health_check_cache = cache;
        self
    }

    /// Auto-detect available providers based on credentials
    pub async fn auto_detect_providers(&mut self) -> Result<Vec<String>, ProviderError> {
        let mut detected_providers = Vec::new();

        // Check for OpenAI API key
        if std::env::var("OPENAI_API_KEY").is_ok() {
            if let Ok(provider) = self.registry.get("openai") {
                if self.test_provider_connection(&provider).await.is_ok() {
                    detected_providers.push("openai".to_string());
                    self.update_provider_state("openai", ConnectionState::Connected, None);
                }
            }
        }

        // Check for Anthropic API key
        if std::env::var("ANTHROPIC_API_KEY").is_ok() {
            if let Ok(provider) = self.registry.get("anthropic") {
                if self.test_provider_connection(&provider).await.is_ok() {
                    detected_providers.push("anthropic".to_string());
                    self.update_provider_state("anthropic", ConnectionState::Connected, None);
                }
            }
        }

        // Check for Ollama (local)
        if let Ok(provider) = self.registry.get("ollama") {
            if self.test_provider_connection(&provider).await.is_ok() {
                detected_providers.push("ollama".to_string());
                self.update_provider_state("ollama", ConnectionState::Connected, None);
            }
        }

        // Check for Google API key
        if std::env::var("GOOGLE_API_KEY").is_ok() {
            if let Ok(provider) = self.registry.get("google") {
                if self.test_provider_connection(&provider).await.is_ok() {
                    detected_providers.push("google".to_string());
                    self.update_provider_state("google", ConnectionState::Connected, None);
                }
            }
        }

        Ok(detected_providers)
    }

    /// Test provider connection
    async fn test_provider_connection(&self, provider: &Arc<dyn Provider>) -> Result<(), ProviderError> {
        // Simple health check - try to get models
        let _models = provider.models();
        Ok(())
    }

    /// Update provider connection state
    pub fn update_provider_state(&mut self, provider_id: &str, state: ConnectionState, error: Option<String>) {
        let status = self.provider_states.entry(provider_id.to_string()).or_insert_with(|| {
            ProviderStatus {
                id: provider_id.to_string(),
                name: provider_id.to_string(), // Will be updated with actual name
                state: ConnectionState::Disabled,
                last_checked: None,
                models: Vec::new(),
                error_message: None,
            }
        });

        status.state = state;
        status.last_checked = Some(std::time::SystemTime::now());
        status.error_message = error;

        // Update models if connected
        if state == ConnectionState::Connected {
            if let Ok(provider) = self.registry.get(provider_id) {
                status.name = provider.name().to_string();
                status.models = provider.models();
            }
        }
    }

    /// Get provider status
    pub fn get_provider_status(&self, provider_id: &str) -> Option<&ProviderStatus> {
        self.provider_states.get(provider_id)
    }

    /// Get all provider statuses
    pub fn get_all_provider_statuses(&self) -> Vec<&ProviderStatus> {
        self.provider_states.values().collect()
    }

    /// Get available models with metadata filtering
    pub fn get_available_models(&self, filter: Option<ModelFilter>) -> Vec<ModelInfo> {
        let mut models = Vec::new();

        for status in self.provider_states.values() {
            if status.state == ConnectionState::Connected {
                for model in &status.models {
                    if let Some(ref f) = filter {
                        if f.matches(model) {
                            models.push(model.clone());
                        }
                    } else {
                        models.push(model.clone());
                    }
                }
            }
        }

        models
    }

    /// Filter models by criteria
    pub fn filter_models(&self, models: &[ModelInfo], criteria: ModelFilterCriteria) -> Vec<ModelInfo> {
        models.iter().filter(|model| {
            match criteria {
                ModelFilterCriteria::Provider(ref provider) => model.provider == *provider,
                ModelFilterCriteria::Capability(ref capability) => model.capabilities.contains(capability),
                ModelFilterCriteria::FreeOnly => model.is_free,
                ModelFilterCriteria::MinContextWindow(min_tokens) => model.context_window >= min_tokens,
                ModelFilterCriteria::MaxCostPerToken(max_cost) => {
                    if let Some(ref pricing) = model.pricing {
                        pricing.input_per_1k_tokens <= max_cost || pricing.output_per_1k_tokens <= max_cost
                    } else {
                        true // No pricing info means we can't filter
                    }
                }
            }
        }).cloned().collect()
    }

    /// Get the default provider
    pub fn default_provider(&self) -> Result<Arc<dyn Provider>, ProviderError> {
        self.registry.get(&self.default_provider_id)
    }

    /// Get a specific provider
    pub fn get_provider(&self, provider_id: &str) -> Result<Arc<dyn Provider>, ProviderError> {
        self.registry.get(provider_id)
    }

    /// Send a chat request with retry logic
    pub async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, ProviderError> {
        let provider = self.default_provider()?;
        self.chat_with_provider(&provider, request).await
    }

    /// Send a chat request to a specific provider with retry logic
    pub async fn chat_with_provider(
        &self,
        provider: &Arc<dyn Provider>,
        request: ChatRequest,
    ) -> Result<ChatResponse, ProviderError> {
        let mut last_error = None;

        for attempt in 0..=self.retry_count {
            match tokio::time::timeout(self.timeout, provider.chat(request.clone())).await {
                Ok(Ok(response)) => return Ok(response),
                Ok(Err(e)) => {
                    last_error = Some(e);
                    if attempt < self.retry_count {
                        // Exponential backoff
                        let backoff = Duration::from_millis(100 * 2_u64.pow(attempt as u32));
                        tokio::time::sleep(backoff).await;
                    }
                }
                Err(_) => {
                    last_error = Some(ProviderError::ProviderError("Request timeout".to_string()));
                    if attempt < self.retry_count {
                        let backoff = Duration::from_millis(100 * 2_u64.pow(attempt as u32));
                        tokio::time::sleep(backoff).await;
                    }
                }
            }
        }

        Err(last_error
            .unwrap_or_else(|| ProviderError::ProviderError("Failed after retries".to_string())))
    }

    /// Stream a chat response
    pub async fn chat_stream(&self, request: ChatRequest) -> Result<ChatStream, ProviderError> {
        let provider = self.default_provider()?;
        provider.chat_stream(request).await
    }

    /// Stream a chat response from a specific provider
    pub async fn chat_stream_with_provider(
        &self,
        provider: &Arc<dyn Provider>,
        request: ChatRequest,
    ) -> Result<ChatStream, ProviderError> {
        provider.chat_stream(request).await
    }

    /// Check provider health with caching
    pub async fn health_check(&self, provider_id: &str) -> Result<bool, ProviderError> {
        let provider = self.registry.get(provider_id)?;
        self.health_check_cache.check_health(&provider).await
    }

    /// Check health of all providers with caching
    pub async fn health_check_all(&self) -> Vec<(String, Result<bool, ProviderError>)> {
        let mut results = Vec::new();

        for provider in self.registry.list_all() {
            let id = provider.id().to_string();
            let health = self.health_check_cache.check_health(&provider).await;
            results.push((id, health));
        }

        results
    }

    /// Invalidate health check cache for a provider
    pub async fn invalidate_health_check(&self, provider_id: &str) {
        self.health_check_cache.invalidate(provider_id).await;
    }

    /// Invalidate all health check cache
    pub async fn invalidate_all_health_checks(&self) {
        self.health_check_cache.invalidate_all().await;
    }

    /// Get the health check cache
    pub fn health_check_cache(&self) -> &Arc<HealthCheckCache> {
        &self.health_check_cache
    }

    /// Get the registry
    pub fn registry(&self) -> &ProviderRegistry {
        &self.registry
    }

    /// Get mutable registry
    pub fn registry_mut(&mut self) -> &mut ProviderRegistry {
        &mut self.registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ChatResponse, FinishReason, TokenUsage};

    struct MockProvider {
        id: String,
    }

    #[async_trait::async_trait]
    impl Provider for MockProvider {
        fn id(&self) -> &str {
            &self.id
        }

        fn name(&self) -> &str {
            "Mock"
        }

        fn models(&self) -> Vec<crate::models::ModelInfo> {
            vec![]
        }

        async fn chat(&self, _request: ChatRequest) -> Result<ChatResponse, ProviderError> {
            Ok(ChatResponse {
                content: "test response".to_string(),
                model: "test-model".to_string(),
                usage: TokenUsage {
                    prompt_tokens: 10,
                    completion_tokens: 5,
                    total_tokens: 15,
                },
                finish_reason: FinishReason::Stop,
            })
        }

        async fn chat_stream(&self, _request: ChatRequest) -> Result<ChatStream, ProviderError> {
            Err(ProviderError::NotFound("Not implemented".to_string()))
        }

        fn count_tokens(&self, _content: &str, _model: &str) -> Result<usize, ProviderError> {
            Ok(0)
        }

        async fn health_check(&self) -> Result<bool, ProviderError> {
            Ok(true)
        }
    }

    #[tokio::test]
    async fn test_manager_creation() {
        let mut registry = ProviderRegistry::new();
        let provider = Arc::new(MockProvider {
            id: "test".to_string(),
        });
        registry.register(provider).unwrap();

        let manager = ProviderManager::new(registry, "test".to_string());
        assert!(manager.default_provider().is_ok());
    }

    #[tokio::test]
    async fn test_chat_request() {
        let mut registry = ProviderRegistry::new();
        let provider = Arc::new(MockProvider {
            id: "test".to_string(),
        });
        registry.register(provider).unwrap();

        let manager = ProviderManager::new(registry, "test".to_string());
        let request = ChatRequest {
            model: "test-model".to_string(),
            messages: vec![],
            temperature: None,
            max_tokens: None,
            stream: false,
        };

        let response = manager.chat(request).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_health_check() {
        let mut registry = ProviderRegistry::new();
        let provider = Arc::new(MockProvider {
            id: "test".to_string(),
        });
        registry.register(provider).unwrap();

        let manager = ProviderManager::new(registry, "test".to_string());
        let health = manager.health_check("test").await;
        assert!(health.is_ok());
        assert!(health.unwrap());
    }
}
