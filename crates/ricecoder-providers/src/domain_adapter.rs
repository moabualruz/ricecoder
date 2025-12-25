//! Domain adapter for bridging infrastructure providers to domain traits
//!
//! AI Provider Implementations
//!
//! This module provides adapters that implement domain-defined port traits
//! using the existing infrastructure provider implementations.

use std::sync::Arc;

use async_trait::async_trait;

use ricecoder_domain::{
    ports::{
        AiChatRequest, AiChatResponse, AiProviderChat, AiProviderInfo, ChatMessage, ChatRole,
        FinishReason, HealthCheckResult, ModelCapability, ModelInfo, ProviderHealthStatus,
        TokenUsage,
    },
    DomainError, DomainResult,
};

use crate::{
    circuit_breaker::{CircuitBreaker, CircuitBreakerConfig},
    error::ProviderError,
    models::{self as provider_models, ChatRequest, ChatResponse},
    provider::Provider,
    rate_limiter::ExponentialBackoff,
};

/// Maps ProviderError to DomainError
pub struct ProviderErrorMapper;

impl ProviderErrorMapper {
    /// Map a ProviderError to a DomainError
    pub fn to_domain_error(error: ProviderError) -> DomainError {
        match error {
            ProviderError::NotFound(id) => DomainError::EntityNotFound {
                entity_type: "Provider".to_string(),
                id,
            },
            ProviderError::AuthError => DomainError::InvalidProviderConfig {
                reason: "Authentication failed".to_string(),
            },
            ProviderError::RateLimited(seconds) => DomainError::InvalidProviderConfig {
                reason: format!("Rate limited. Retry after {} seconds", seconds),
            },
            ProviderError::ContextTooLarge(tokens, max) => DomainError::ValidationError {
                field: "context".to_string(),
                reason: format!("Context too large: {} tokens, max {}", tokens, max),
            },
            ProviderError::NetworkError(msg) => DomainError::InvalidProviderConfig {
                reason: format!("Network error: {}", msg),
            },
            ProviderError::ProviderError(msg) => DomainError::InvalidProviderConfig {
                reason: msg,
            },
            ProviderError::ConfigError(msg) => DomainError::InvalidProviderConfig {
                reason: msg,
            },
            ProviderError::InvalidModel(model) => DomainError::ValidationError {
                field: "model".to_string(),
                reason: format!("Invalid model: {}", model),
            },
            ProviderError::ModelNotAvailable(model) => DomainError::InvalidProviderConfig {
                reason: format!("Model not available: {}", model),
            },
            ProviderError::SerializationError(msg) => DomainError::ValidationError {
                field: "serialization".to_string(),
                reason: msg,
            },
            ProviderError::ParseError(msg) => DomainError::ValidationError {
                field: "response".to_string(),
                reason: format!("Parse error: {}", msg),
            },
            ProviderError::Internal(msg) => DomainError::AnalysisFailed { reason: msg },
        }
    }

    /// Map from DomainError to ProviderError (for internal use)
    #[allow(dead_code)]
    pub fn from_domain_error(error: DomainError) -> ProviderError {
        match error {
            DomainError::EntityNotFound { id, .. } => ProviderError::NotFound(id),
            DomainError::InvalidProviderConfig { reason } => ProviderError::ProviderError(reason),
            DomainError::ValidationError { reason, .. } => ProviderError::ConfigError(reason),
            _ => ProviderError::Internal(error.to_string()),
        }
    }
}

/// Convert infrastructure ChatRequest to domain AiChatRequest
#[allow(dead_code)]
fn to_domain_request(request: ChatRequest) -> AiChatRequest {
    AiChatRequest {
        model: request.model,
        messages: request
            .messages
            .into_iter()
            .map(|m| ChatMessage {
                role: match m.role.as_str() {
                    "user" => ChatRole::User,
                    "assistant" => ChatRole::Assistant,
                    "system" => ChatRole::System,
                    _ => ChatRole::User,
                },
                content: m.content,
            })
            .collect(),
        temperature: request.temperature,
        max_tokens: request.max_tokens.map(|t| t as u32),
        stop: None, // Infrastructure ChatRequest doesn't support stop sequences
    }
}

/// Convert domain AiChatRequest to infrastructure ChatRequest
fn to_infra_request(request: AiChatRequest) -> ChatRequest {
    ChatRequest {
        model: request.model,
        messages: request
            .messages
            .into_iter()
            .map(|m| provider_models::Message {
                role: match m.role {
                    ChatRole::User => "user".to_string(),
                    ChatRole::Assistant => "assistant".to_string(),
                    ChatRole::System => "system".to_string(),
                },
                content: m.content,
            })
            .collect(),
        temperature: request.temperature,
        max_tokens: request.max_tokens.map(|t| t as usize),
        stream: false, // Domain requests are non-streaming by default
    }
}

/// Convert infrastructure ChatResponse to domain AiChatResponse
fn to_domain_response(response: ChatResponse) -> AiChatResponse {
    AiChatResponse {
        content: response.content,
        model: response.model,
        usage: TokenUsage {
            prompt_tokens: response.usage.prompt_tokens as u32,
            completion_tokens: response.usage.completion_tokens as u32,
            total_tokens: response.usage.total_tokens as u32,
        },
        finish_reason: match response.finish_reason {
            provider_models::FinishReason::Stop => FinishReason::Stop,
            provider_models::FinishReason::Length => FinishReason::Length,
            provider_models::FinishReason::Error => FinishReason::Error,
        },
    }
}

/// Convert infrastructure Capability to domain ModelCapability
fn to_domain_capability(cap: provider_models::Capability) -> ModelCapability {
    match cap {
        provider_models::Capability::Chat => ModelCapability::Chat,
        provider_models::Capability::Code => ModelCapability::Code,
        provider_models::Capability::Vision => ModelCapability::Vision,
        provider_models::Capability::Streaming => ModelCapability::Streaming,
        provider_models::Capability::FunctionCalling => ModelCapability::FunctionCalling,
    }
}

/// Convert infrastructure ModelInfo to domain ModelInfo
fn to_domain_model_info(info: provider_models::ModelInfo) -> ModelInfo {
    ModelInfo {
        id: info.id,
        name: info.name,
        provider: info.provider,
        context_window: info.context_window as u32,
        capabilities: info
            .capabilities
            .into_iter()
            .map(to_domain_capability)
            .collect(),
        is_free: info.is_free,
    }
}

/// Adapter that wraps an infrastructure Provider to implement domain AiProvider trait
///
/// This adapter:
/// - Converts between domain and infrastructure types
/// - Applies circuit breaker pattern for resilience
/// - Implements retry logic with exponential backoff
/// - Maps errors to domain error types
pub struct DomainProviderAdapter {
    /// Underlying infrastructure provider
    inner: Arc<dyn Provider>,
    /// Circuit breaker for this provider
    circuit_breaker: CircuitBreaker,
    /// Maximum retry attempts
    max_retries: u32,
}

impl DomainProviderAdapter {
    /// Create a new adapter wrapping an infrastructure provider
    pub fn new(provider: Arc<dyn Provider>) -> Self {
        let id = provider.id().to_string();
        Self {
            inner: provider,
            circuit_breaker: CircuitBreaker::new(id, CircuitBreakerConfig::default()),
            max_retries: 3,
        }
    }

    /// Create with custom circuit breaker config
    pub fn with_circuit_breaker(provider: Arc<dyn Provider>, config: CircuitBreakerConfig) -> Self {
        let id = provider.id().to_string();
        Self {
            inner: provider,
            circuit_breaker: CircuitBreaker::new(id, config),
            max_retries: 3,
        }
    }

    /// Set maximum retry attempts
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Execute request with retry and circuit breaker
    async fn execute_with_resilience<F, T, Fut>(&self, operation: F) -> DomainResult<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, ProviderError>>,
    {
        // Check circuit breaker
        if !self.circuit_breaker.can_execute() {
            return Err(DomainError::InvalidProviderConfig {
                reason: format!(
                    "Circuit breaker is open - provider {} unavailable",
                    self.inner.id()
                ),
            });
        }

        let mut backoff = ExponentialBackoff::new(
            std::time::Duration::from_millis(100),
            std::time::Duration::from_secs(30),
            2.0,
        );

        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            if attempt > 0 {
                let delay = backoff.next_delay();
                tokio::time::sleep(delay).await;
            }

            match operation().await {
                Ok(result) => {
                    self.circuit_breaker.record_success();
                    return Ok(result);
                }
                Err(e) => {
                    // Check if error is retryable
                    let should_retry = matches!(
                        e,
                        ProviderError::NetworkError(_)
                            | ProviderError::RateLimited(_)
                            | ProviderError::ProviderError(_)
                    );

                    if should_retry && attempt < self.max_retries {
                        tracing::warn!(
                            "Provider {} request failed (attempt {}/{}): {}",
                            self.inner.id(),
                            attempt + 1,
                            self.max_retries + 1,
                            e
                        );
                        last_error = Some(e);
                        continue;
                    }

                    self.circuit_breaker.record_failure();
                    return Err(ProviderErrorMapper::to_domain_error(e));
                }
            }
        }

        // Should not reach here, but handle it gracefully
        self.circuit_breaker.record_failure();
        Err(ProviderErrorMapper::to_domain_error(
            last_error.unwrap_or_else(|| {
                ProviderError::Internal("Retry loop exhausted unexpectedly".to_string())
            }),
        ))
    }
}

impl AiProviderInfo for DomainProviderAdapter {
    fn id(&self) -> &str {
        self.inner.id()
    }

    fn name(&self) -> &str {
        self.inner.name()
    }

    fn models(&self) -> Vec<ModelInfo> {
        self.inner
            .models()
            .into_iter()
            .map(to_domain_model_info)
            .collect()
    }
}

#[async_trait]
impl AiProviderChat for DomainProviderAdapter {
    async fn chat(&self, request: AiChatRequest) -> DomainResult<AiChatResponse> {
        let infra_request = to_infra_request(request);
        let inner = Arc::clone(&self.inner);

        self.execute_with_resilience(|| {
            let req = infra_request.clone();
            let provider = Arc::clone(&inner);
            async move {
                let response = provider.chat(req).await?;
                Ok(to_domain_response(response))
            }
        })
        .await
    }

    fn count_tokens(&self, content: &str, model: &str) -> DomainResult<usize> {
        self.inner
            .count_tokens(content, model)
            .map_err(ProviderErrorMapper::to_domain_error)
    }

    async fn health_check(&self) -> DomainResult<HealthCheckResult> {
        let start = std::time::Instant::now();

        match self.inner.health_check().await {
            Ok(is_healthy) => {
                let latency_ms = start.elapsed().as_millis() as u64;

                if is_healthy {
                    if latency_ms > 1000 {
                        // > 1 second is degraded
                        Ok(HealthCheckResult::degraded(latency_ms))
                    } else {
                        Ok(HealthCheckResult::healthy(latency_ms))
                    }
                } else {
                    Ok(HealthCheckResult::unhealthy("Provider reported unhealthy"))
                }
            }
            Err(e) => Ok(HealthCheckResult::unhealthy(e.to_string())),
        }
    }

    fn supports_capability(&self, capability: ModelCapability) -> bool {
        // Check if any model supports the capability
        self.inner.models().iter().any(|m| {
            m.capabilities.iter().any(|c| match (c, &capability) {
                (provider_models::Capability::Chat, ModelCapability::Chat) => true,
                (provider_models::Capability::Code, ModelCapability::Code) => true,
                (provider_models::Capability::Vision, ModelCapability::Vision) => true,
                (provider_models::Capability::FunctionCalling, ModelCapability::FunctionCalling) => {
                    true
                }
                (provider_models::Capability::Streaming, ModelCapability::Streaming) => true,
                // Embeddings not supported in infrastructure yet
                (_, ModelCapability::Embeddings) => false,
                _ => false,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        Capability, ChatResponse, FinishReason as InfraFinishReason,
        TokenUsage as InfraTokenUsage,
    };
    use std::sync::atomic::{AtomicU32, Ordering};

    /// Mock provider for testing
    struct MockProvider {
        id: String,
        name: String,
        call_count: AtomicU32,
        should_fail: bool,
        fail_count: AtomicU32,
        fail_until: u32,
    }

    impl MockProvider {
        fn new(id: &str, name: &str) -> Self {
            Self {
                id: id.to_string(),
                name: name.to_string(),
                call_count: AtomicU32::new(0),
                should_fail: false,
                fail_count: AtomicU32::new(0),
                fail_until: 0,
            }
        }

        fn failing(id: &str, name: &str) -> Self {
            Self {
                id: id.to_string(),
                name: name.to_string(),
                call_count: AtomicU32::new(0),
                should_fail: true,
                fail_count: AtomicU32::new(0),
                fail_until: u32::MAX,
            }
        }

        fn failing_then_succeeding(id: &str, name: &str, fail_until: u32) -> Self {
            Self {
                id: id.to_string(),
                name: name.to_string(),
                call_count: AtomicU32::new(0),
                should_fail: true,
                fail_count: AtomicU32::new(0),
                fail_until,
            }
        }
    }

    #[async_trait]
    impl Provider for MockProvider {
        fn id(&self) -> &str {
            &self.id
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn models(&self) -> Vec<provider_models::ModelInfo> {
            vec![provider_models::ModelInfo {
                id: "test-model".to_string(),
                name: "Test Model".to_string(),
                provider: self.id.clone(),
                context_window: 4096,
                capabilities: vec![Capability::Chat],
                pricing: None,
                is_free: true,
            }]
        }

        async fn chat(&self, _request: ChatRequest) -> Result<ChatResponse, ProviderError> {
            self.call_count.fetch_add(1, Ordering::SeqCst);

            if self.should_fail {
                let fails = self.fail_count.fetch_add(1, Ordering::SeqCst);
                if fails < self.fail_until {
                    return Err(ProviderError::NetworkError("Connection failed".to_string()));
                }
            }

            Ok(ChatResponse {
                content: "Hello!".to_string(),
                model: "test-model".to_string(),
                usage: InfraTokenUsage {
                    prompt_tokens: 10,
                    completion_tokens: 5,
                    total_tokens: 15,
                },
                finish_reason: InfraFinishReason::Stop,
            })
        }

        async fn chat_stream(
            &self,
            _request: ChatRequest,
        ) -> Result<crate::provider::ChatStream, ProviderError> {
            unimplemented!("Not needed for tests")
        }

        fn count_tokens(&self, content: &str, _model: &str) -> Result<usize, ProviderError> {
            Ok(content.split_whitespace().count())
        }

        async fn health_check(&self) -> Result<bool, ProviderError> {
            Ok(!self.should_fail || self.fail_count.load(Ordering::SeqCst) >= self.fail_until)
        }
    }

    #[tokio::test]
    async fn test_adapter_chat_success() {
        let provider = Arc::new(MockProvider::new("test", "Test Provider"));
        let adapter = DomainProviderAdapter::new(provider);

        let request = AiChatRequest::new("test-model", vec![ChatMessage::user("Hello")]);

        let response = adapter.chat(request).await.unwrap();
        assert_eq!(response.content, "Hello!");
        assert_eq!(response.model, "test-model");
    }

    #[tokio::test]
    async fn test_adapter_retries_on_failure() {
        // Fail first 2 attempts, succeed on 3rd
        let provider = Arc::new(MockProvider::failing_then_succeeding("test", "Test", 2));
        let adapter = DomainProviderAdapter::new(provider.clone()).with_max_retries(3);

        let request = AiChatRequest::new("test-model", vec![ChatMessage::user("Hello")]);

        let response = adapter.chat(request).await.unwrap();
        assert_eq!(response.content, "Hello!");

        // Should have called 3 times (2 failures + 1 success)
        assert_eq!(provider.call_count.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_adapter_circuit_breaker_opens() {
        let provider = Arc::new(MockProvider::failing("test", "Test"));
        let config = CircuitBreakerConfig::default().with_failure_threshold(2);
        let adapter = DomainProviderAdapter::with_circuit_breaker(provider, config)
            .with_max_retries(0); // No retries to speed up test

        let request = AiChatRequest::new("test-model", vec![ChatMessage::user("Hello")]);

        // First failure
        assert!(adapter.chat(request.clone()).await.is_err());

        // Second failure - should open circuit
        assert!(adapter.chat(request.clone()).await.is_err());

        // Third attempt - circuit should be open
        let result = adapter.chat(request).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Circuit breaker is open"));
    }

    #[test]
    fn test_error_mapping() {
        let provider_error = ProviderError::AuthError;
        let domain_error = ProviderErrorMapper::to_domain_error(provider_error);

        match domain_error {
            DomainError::InvalidProviderConfig { reason } => {
                assert!(reason.contains("Authentication"));
            }
            _ => panic!("Expected InvalidProviderConfig"),
        }
    }

    #[test]
    fn test_model_conversion() {
        let infra_model = provider_models::ModelInfo {
            id: "gpt-4".to_string(),
            name: "GPT-4".to_string(),
            provider: "openai".to_string(),
            context_window: 8192,
            capabilities: vec![Capability::Chat, Capability::Code],
            pricing: None,
            is_free: false,
        };

        let domain_model = to_domain_model_info(infra_model);

        assert_eq!(domain_model.id, "gpt-4");
        assert_eq!(domain_model.name, "GPT-4");
        assert_eq!(domain_model.provider, "openai");
        assert_eq!(domain_model.context_window, 8192);
        assert!(!domain_model.is_free);
        assert!(domain_model.capabilities.contains(&ModelCapability::Chat));
        assert!(domain_model.capabilities.contains(&ModelCapability::Code));
    }

    #[tokio::test]
    async fn test_health_check() {
        let provider = Arc::new(MockProvider::new("test", "Test Provider"));
        let adapter = DomainProviderAdapter::new(provider);

        let result = adapter.health_check().await.unwrap();
        assert_eq!(result.status, ProviderHealthStatus::Healthy);
        assert!(result.latency_ms.is_some());
    }

    #[test]
    fn test_count_tokens() {
        let provider = Arc::new(MockProvider::new("test", "Test Provider"));
        let adapter = DomainProviderAdapter::new(provider);

        let count = adapter.count_tokens("hello world test", "test-model").unwrap();
        assert_eq!(count, 3);
    }
}
