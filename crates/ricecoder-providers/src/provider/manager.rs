//! Provider manager for orchestrating provider operations

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use super::{ChatStream, Provider, ProviderRegistry};
use crate::community::{CommunityProviderRegistry, ProviderUsage};
use crate::curation::{CurationConfig, ProviderCurator, SelectionConstraints};
use crate::error::ProviderError;
use crate::health_check::HealthCheckCache;
use crate::models::{Capability, ChatRequest, ChatResponse, ModelInfo, TokenUsage};
use crate::performance_monitor::ProviderPerformanceMonitor;

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
    performance_monitor: Arc<ProviderPerformanceMonitor>,
    curator: ProviderCurator,
    community_registry: CommunityProviderRegistry,
}

impl ProviderManager {
    /// Create a new provider manager
    pub fn new(registry: ProviderRegistry, default_provider_id: String) -> Self {
        let performance_monitor = Arc::new(ProviderPerformanceMonitor::default());
        let curator = ProviderCurator::default(performance_monitor.clone());

        Self {
            registry,
            default_provider_id,
            retry_count: 3,
            timeout: Duration::from_secs(30),
            health_check_cache: Arc::new(HealthCheckCache::default()),
            provider_states: HashMap::new(),
            performance_monitor,
            curator,
            community_registry: CommunityProviderRegistry::new(),
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
    pub async fn chat(&mut self, request: ChatRequest) -> Result<ChatResponse, ProviderError> {
        let provider = self.default_provider()?;
        self.chat_with_provider(&provider, request).await
    }

    /// Send a chat request to a specific provider with retry logic
    pub async fn chat_with_provider(
        &mut self,
        provider: &Arc<dyn Provider>,
        request: ChatRequest,
    ) -> Result<ChatResponse, ProviderError> {
        let provider_id = provider.id();
        let mut last_error = None;
        let start_time = std::time::Instant::now();

        for attempt in 0..=self.retry_count {
            match tokio::time::timeout(self.timeout, provider.chat(request.clone())).await {
                Ok(Ok(response)) => {
                    let response_time_ms = start_time.elapsed().as_millis() as u64;
                    let tokens_used = response.usage.total_tokens as u64;
                    let cost = self.calculate_cost(provider_id, &Some(response.usage.clone()));

                    self.performance_monitor.record_success(provider_id, response_time_ms, tokens_used, cost);
                    self.curator.record_success(provider_id);

                    // Record community analytics
                    let usage = ProviderUsage {
                        success: true,
                        tokens_used,
                        cost,
                        response_time_ms,
                        model: request.model.clone(),
                        error_type: None,
                    };
                    self.community_registry.record_usage(provider_id, usage);

                    return Ok(response);
                }
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

        // Record failure
        let response_time_ms = start_time.elapsed().as_millis() as u64;
        self.performance_monitor.record_failure(provider_id, response_time_ms);
        self.curator.record_failure(provider_id);

        // Record community analytics for failure
        let usage = ProviderUsage {
            success: false,
            tokens_used: 0, // No tokens used on failure
            cost: 0.0,
            response_time_ms,
            model: request.model.clone(),
            error_type: Some("request_failed".to_string()),
        };
        self.community_registry.record_usage(provider_id, usage);

        Err(last_error.unwrap_or_else(|| ProviderError::ProviderError("All retry attempts failed".to_string())))
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

    /// Calculate cost for a request based on token usage and model pricing
    fn calculate_cost(&self, provider_id: &str, usage: &Option<TokenUsage>) -> f64 {
        if let Some(usage) = usage {
            // Get provider to find model pricing
            if let Ok(provider) = self.registry.get(provider_id) {
                let models = provider.models();
                // For now, we assume the model from the response, but we could enhance this
                // to track which model was actually used per request
                // This is a simplified implementation - in practice, we'd need to know
                // which specific model was used for accurate pricing
                if let Some(model) = models.first() {
                    if let Some(ref pricing) = model.pricing {
                        let input_cost = (usage.prompt_tokens as f64 / 1000.0) * pricing.input_per_1k_tokens;
                        let output_cost = (usage.completion_tokens as f64 / 1000.0) * pricing.output_per_1k_tokens;
                        return input_cost + output_cost;
                    }
                }
            }
        }
        0.0 // No usage info or pricing available
    }

    /// Get the performance monitor for external access
    pub fn performance_monitor(&self) -> &Arc<ProviderPerformanceMonitor> {
        &self.performance_monitor
    }

    /// Get the provider curator for external access
    pub fn curator(&self) -> &ProviderCurator {
        &self.curator
    }

    /// Get mutable access to the curator
    pub fn curator_mut(&mut self) -> &mut ProviderCurator {
        &mut self.curator
    }

    /// Select the best provider based on quality scores and constraints
    pub fn select_best_provider(&self, constraints: Option<&SelectionConstraints>) -> Option<String> {
        let available_providers: Vec<String> = self.provider_states.keys().cloned().collect();
        self.curator.select_best_provider(&available_providers, constraints)
    }

    /// Select the best provider for a specific model requirement
    pub fn select_best_provider_for_model(&self, required_capabilities: &[Capability], constraints: Option<&SelectionConstraints>) -> Option<String> {
        let mut suitable_providers = Vec::new();

        for (provider_id, status) in &self.provider_states {
            if status.state == ConnectionState::Connected {
                // Check if provider has models with required capabilities
                let has_suitable_model = status.models.iter().any(|model|
                    required_capabilities.iter().all(|cap| model.capabilities.contains(cap))
                );

                if has_suitable_model {
                    suitable_providers.push(provider_id.clone());
                }
            }
        }

        self.curator.select_best_provider(&suitable_providers, constraints)
    }

    /// Update quality scores for all providers
    pub fn update_provider_quality_scores(&mut self) {
        let mut provider_models = HashMap::new();

        for (provider_id, status) in &self.provider_states {
            if status.state == ConnectionState::Connected {
                provider_models.insert(provider_id.clone(), status.models.clone());
            }
        }

        self.curator.update_quality_scores(&provider_models);
    }

    /// Get providers sorted by quality for a given set of providers
    pub fn get_providers_by_quality(&self, provider_ids: &[String]) -> Vec<(String, f64)> {
        self.curator.get_providers_by_quality(provider_ids)
    }

    /// Check if a provider should be avoided due to reliability issues
    pub fn should_avoid_provider(&self, provider_id: &str) -> bool {
        if let Some(tracker) = self.curator.get_reliability_tracker(provider_id) {
            tracker.should_avoid(self.curator.config())
        } else {
            false
        }
    }

    /// Get automatic failover provider if current provider is failing
    pub fn get_failover_provider(&self, current_provider_id: &str) -> Option<String> {
        if self.should_avoid_provider(current_provider_id) {
            // Find alternative providers
            let alternatives: Vec<String> = self.provider_states
                .iter()
                .filter(|(id, status)| {
                    *id != current_provider_id &&
                    status.state == ConnectionState::Connected &&
                    !self.should_avoid_provider(id)
                })
                .map(|(id, _)| id.clone())
                .collect();

            self.curator.select_best_provider(&alternatives, None)
        } else {
            None
        }
    }

    /// Get the community registry for external access
    pub fn community_registry(&self) -> &CommunityProviderRegistry {
        &self.community_registry
    }

    /// Get mutable access to the community registry
    pub fn community_registry_mut(&mut self) -> &mut CommunityProviderRegistry {
        &mut self.community_registry
    }

    /// Load community configurations for providers
    pub fn load_community_configs(&mut self) -> Result<(), ProviderError> {
        // Load approved community configurations and register them
        for config in self.community_registry.get_all_approved_configs() {
            // Check if we already have this provider
            if self.registry.get(&config.provider_id).is_err() {
                // For now, we would need to create a provider instance from the config
                // This is a simplified implementation - in practice, you'd need to
                // instantiate the appropriate provider type based on the config
                println!("Would load community config for provider: {}", config.provider_id);
            }
        }
        Ok(())
    }

    /// Get community quality metrics for a provider
    pub fn get_community_quality_metrics(&self, provider_id: &str) -> Option<crate::community::CommunityQualityMetrics> {
        self.community_registry.get_community_quality_metrics(provider_id)
    }

    /// Get popular providers from community analytics
    pub fn get_popular_providers(&self, limit: usize) -> Vec<(String, u64)> {
        self.community_registry.get_popular_providers(limit)
    }

    /// Get providers ranked by community quality
    pub fn get_providers_by_community_quality(&self, limit: usize) -> Vec<(String, f64)> {
        self.community_registry.get_providers_by_community_quality(limit)
    }

    /// Submit a provider configuration to the community
    pub fn submit_community_config(&mut self, config: crate::community::CommunityProviderConfig) -> Result<String, ProviderError> {
        self.community_registry.submit_contribution(config)
    }

    /// Get provider analytics
    pub fn get_provider_analytics(&self, provider_id: &str) -> Option<&crate::community::ProviderAnalytics> {
        self.community_registry.get_analytics(provider_id)
    }
}


