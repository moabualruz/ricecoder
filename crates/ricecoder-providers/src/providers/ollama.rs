//! Ollama provider implementation
//!
//! Supports local model execution via Ollama.
//! Ollama allows running large language models locally without sending code to external services.

use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};

use async_trait::async_trait;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

use super::ollama_config::OllamaConfig;
use crate::{
    error::ProviderError,
    models::{Capability, ChatRequest, ChatResponse, FinishReason, ModelInfo, TokenUsage},
    provider::Provider,
};

/// Configuration for retry logic
const MAX_RETRIES: u32 = 3;
const INITIAL_BACKOFF_MS: u64 = 100;
const MAX_BACKOFF_MS: u64 = 400;

/// Cache for models with TTL
struct ModelCache {
    models: Option<Vec<ModelInfo>>,
    cached_at: Option<SystemTime>,
    ttl: Duration,
}

impl ModelCache {
    /// Create a new model cache with default TTL (5 minutes)
    fn new() -> Self {
        Self {
            models: None,
            cached_at: None,
            ttl: Duration::from_secs(300), // 5 minutes default
        }
    }

    /// Create a new model cache with custom TTL
    /// Reserved for future use when configurable TTL is needed
    #[allow(dead_code)]
    fn with_ttl(ttl: Duration) -> Self {
        Self {
            models: None,
            cached_at: None,
            ttl,
        }
    }

    /// Check if cache is still valid
    fn is_valid(&self) -> bool {
        if let (Some(cached_at), Some(_)) = (self.cached_at, &self.models) {
            if let Ok(elapsed) = cached_at.elapsed() {
                return elapsed < self.ttl;
            }
        }
        false
    }

    /// Get cached models if valid
    fn get(&self) -> Option<Vec<ModelInfo>> {
        if self.is_valid() {
            self.models.clone()
        } else {
            None
        }
    }

    /// Set cached models
    fn set(&mut self, models: Vec<ModelInfo>) {
        self.models = Some(models);
        self.cached_at = Some(SystemTime::now());
    }

    /// Get cached models even if expired (for fallback)
    fn get_stale(&self) -> Option<Vec<ModelInfo>> {
        self.models.clone()
    }

    /// Clear the cache
    /// Reserved for future use when cache invalidation is needed
    #[allow(dead_code)]
    fn clear(&mut self) {
        self.models = None;
        self.cached_at = None;
    }
}

/// Ollama provider implementation
pub struct OllamaProvider {
    client: Arc<Client>,
    base_url: String,
    available_models: Vec<ModelInfo>,
    model_cache: Arc<tokio::sync::Mutex<ModelCache>>,
}

/// Helper function to determine if an error is transient (retryable)
fn is_transient_error(err: &reqwest::Error) -> bool {
    err.is_timeout() || err.is_connect() || err.status().is_some_and(|s| s.is_server_error())
}

/// Execute a request with exponential backoff retry logic
/// Returns the response if successful, or the last error if all retries fail
async fn execute_with_retry<F, Fut>(mut request_fn: F) -> Result<reqwest::Response, reqwest::Error>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<reqwest::Response, reqwest::Error>>,
{
    let mut attempt = 0;

    loop {
        match request_fn().await {
            Ok(response) => return Ok(response),
            Err(err) => {
                // Check if error is transient and we haven't exceeded max retries
                if is_transient_error(&err) && attempt < MAX_RETRIES {
                    // Calculate exponential backoff: 100ms, 200ms, 400ms
                    let backoff_ms = INITIAL_BACKOFF_MS * 2_u64.pow(attempt);
                    let backoff_ms = backoff_ms.min(MAX_BACKOFF_MS);

                    warn!(
                        "Transient error on attempt {}/{}, retrying after {}ms: {}",
                        attempt + 1,
                        MAX_RETRIES,
                        backoff_ms,
                        err
                    );

                    sleep(Duration::from_millis(backoff_ms)).await;
                    attempt += 1;
                } else {
                    // Permanent error or max retries exceeded
                    if attempt >= MAX_RETRIES {
                        debug!("Max retries ({}) exceeded for request", MAX_RETRIES);
                    }
                    return Err(err);
                }
            }
        }
    }
}

impl OllamaProvider {
    /// Create a new Ollama provider instance
    pub fn new(base_url: String) -> Result<Self, ProviderError> {
        if base_url.is_empty() {
            return Err(ProviderError::ConfigError(
                "Ollama base URL is required".to_string(),
            ));
        }

        Ok(Self {
            client: Arc::new(Client::new()),
            base_url,
            available_models: vec![],
            model_cache: Arc::new(tokio::sync::Mutex::new(ModelCache::new())),
        })
    }

    /// Create a new Ollama provider with default localhost endpoint
    pub fn with_default_endpoint() -> Result<Self, ProviderError> {
        Self::new("http://localhost:11434".to_string())
    }

    /// Create a new Ollama provider from configuration files
    /// Loads configuration with proper precedence:
    /// 1. Environment variables (highest priority)
    /// 2. Project config (.ricecoder/config.yaml)
    /// 3. Global config (~/.ricecoder/config.yaml)
    /// 4. Built-in defaults (lowest priority)
    pub fn from_config() -> Result<Self, ProviderError> {
        let config = OllamaConfig::load_with_precedence()?;
        debug!(
            "Creating OllamaProvider from configuration: base_url={}, default_model={}",
            config.base_url, config.default_model
        );
        Self::new(config.base_url)
    }

    /// Get the current configuration
    pub fn config(&self) -> Result<OllamaConfig, ProviderError> {
        OllamaConfig::load_with_precedence()
    }

    /// Detect if Ollama is available at startup
    /// Returns true if Ollama is running and accessible
    pub async fn detect_availability(&self) -> bool {
        debug!("Detecting Ollama availability at {}", self.base_url);

        match self.health_check().await {
            Ok(true) => {
                info!("Ollama is available at {}", self.base_url);
                true
            }
            Ok(false) => {
                warn!("Ollama health check returned false at {}", self.base_url);
                false
            }
            Err(e) => {
                warn!("Ollama is not available at {}: {}", self.base_url, e);
                false
            }
        }
    }

    /// Get models with offline fallback
    /// Returns cached models if available, or default models if offline
    pub async fn get_models_with_fallback(&self) -> Vec<ModelInfo> {
        let cache = self.model_cache.lock().await;

        // Try to get valid cached models
        if let Some(cached_models) = cache.get() {
            debug!("Returning cached models ({} models)", cached_models.len());
            return cached_models;
        }

        // Try to get stale cached models as fallback
        if let Some(stale_models) = cache.get_stale() {
            warn!(
                "Returning stale cached models ({} models) - cache expired",
                stale_models.len()
            );
            return stale_models;
        }

        // No cache available, return default models
        debug!("No cached models available, returning defaults for offline mode");
        vec![
            ModelInfo {
                id: "mistral".to_string(),
                name: "Mistral".to_string(),
                provider: "ollama".to_string(),
                context_window: 8192,
                capabilities: vec![Capability::Chat, Capability::Code, Capability::Streaming],
                pricing: None,
                is_free: true,
            },
            ModelInfo {
                id: "neural-chat".to_string(),
                name: "Neural Chat".to_string(),
                provider: "ollama".to_string(),
                context_window: 4096,
                capabilities: vec![Capability::Chat, Capability::Code, Capability::Streaming],
                pricing: None,
                is_free: true,
            },
            ModelInfo {
                id: "llama2".to_string(),
                name: "Llama 2".to_string(),
                provider: "ollama".to_string(),
                context_window: 4096,
                capabilities: vec![Capability::Chat, Capability::Code, Capability::Streaming],
                pricing: None,
                is_free: true,
            },
        ]
    }

    /// Fetch available models from Ollama with caching
    /// Returns cached models if available and not expired
    /// Falls back to cached models if Ollama is unavailable
    pub async fn fetch_models(&mut self) -> Result<(), ProviderError> {
        debug!("Fetching available models from Ollama");

        // Check if cache is valid
        let cache = self.model_cache.lock().await;
        if let Some(cached_models) = cache.get() {
            debug!("Using cached models ({} models)", cached_models.len());
            self.available_models = cached_models;
            return Ok(());
        }

        // Cache is invalid, fetch from Ollama
        drop(cache); // Release lock before making network request

        let base_url = self.base_url.clone();
        let client = self.client.clone();

        let response = execute_with_retry(|| {
            let client = client.clone();
            let url = format!("{}/api/tags", base_url);
            async move { client.get(url).send().await }
        })
        .await
        .map_err(|e| {
            error!("Failed to fetch models from Ollama after retries: {}", e);
            ProviderError::NetworkError(e.to_string())
        })?;

        if !response.status().is_success() {
            return Err(ProviderError::ProviderError(format!(
                "Ollama API error: {}",
                response.status()
            )));
        }

        let tags_response: OllamaTagsResponse = response.json().await.map_err(|e| {
            error!("Failed to parse Ollama tags response: {}", e);
            ProviderError::ProviderError(format!("Failed to parse Ollama response: {}", e))
        })?;

        // Convert Ollama models to our ModelInfo format
        self.available_models = tags_response
            .models
            .unwrap_or_default()
            .into_iter()
            .map(|model| ModelInfo {
                id: model.name.clone(),
                name: model.name.clone(),
                provider: "ollama".to_string(),
                context_window: 4096, // Default context window for local models
                capabilities: vec![Capability::Chat, Capability::Code, Capability::Streaming],
                pricing: None, // Local models have no pricing
                is_free: true, // Local models are always free
            })
            .collect();

        // Update cache
        let mut cache = self.model_cache.lock().await;
        cache.set(self.available_models.clone());

        debug!("Fetched {} models from Ollama", self.available_models.len());
        Ok(())
    }

    /// Convert Ollama API response to our ChatResponse
    fn convert_response(
        response: OllamaChatResponse,
        model: String,
    ) -> Result<ChatResponse, ProviderError> {
        Ok(ChatResponse {
            content: response.message.content,
            model,
            usage: TokenUsage {
                prompt_tokens: 0, // Ollama doesn't provide token counts
                completion_tokens: 0,
                total_tokens: 0,
            },
            finish_reason: if response.done {
                FinishReason::Stop
            } else {
                FinishReason::Error
            },
        })
    }
}

#[async_trait]
impl Provider for OllamaProvider {
    fn id(&self) -> &str {
        "ollama"
    }

    fn name(&self) -> &str {
        "Ollama"
    }

    fn models(&self) -> Vec<ModelInfo> {
        if self.available_models.is_empty() {
            // Return some common Ollama models as defaults
            vec![
                ModelInfo {
                    id: "mistral".to_string(),
                    name: "Mistral".to_string(),
                    provider: "ollama".to_string(),
                    context_window: 8192,
                    capabilities: vec![Capability::Chat, Capability::Code, Capability::Streaming],
                    pricing: None,
                    is_free: true,
                },
                ModelInfo {
                    id: "neural-chat".to_string(),
                    name: "Neural Chat".to_string(),
                    provider: "ollama".to_string(),
                    context_window: 4096,
                    capabilities: vec![Capability::Chat, Capability::Code, Capability::Streaming],
                    pricing: None,
                    is_free: true,
                },
                ModelInfo {
                    id: "llama2".to_string(),
                    name: "Llama 2".to_string(),
                    provider: "ollama".to_string(),
                    context_window: 4096,
                    capabilities: vec![Capability::Chat, Capability::Code, Capability::Streaming],
                    pricing: None,
                    is_free: true,
                },
            ]
        } else {
            self.available_models.clone()
        }
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, ProviderError> {
        debug!(
            "Sending chat request to Ollama for model: {}",
            request.model
        );

        let ollama_request = OllamaChatRequest {
            model: request.model.clone(),
            messages: request
                .messages
                .iter()
                .map(|m| OllamaMessage {
                    role: m.role.clone(),
                    content: m.content.clone(),
                })
                .collect(),
            stream: false,
        };

        let base_url = self.base_url.clone();
        let client = self.client.clone();

        let response = execute_with_retry(|| {
            let client = client.clone();
            let url = format!("{}/api/chat", base_url);
            let req = ollama_request.clone();
            async move { client.post(url).json(&req).send().await }
        })
        .await
        .map_err(|e| {
            error!("Ollama API request failed after retries: {}", e);
            ProviderError::NetworkError(e.to_string())
        })?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("Ollama API error ({}): {}", status, error_text);

            return Err(ProviderError::ProviderError(format!(
                "Ollama API error: {}",
                status
            )));
        }

        let ollama_response: OllamaChatResponse = response.json().await.map_err(|e| {
            error!("Failed to parse Ollama response: {}", e);
            ProviderError::ProviderError(format!("Failed to parse Ollama response: {}", e))
        })?;

        Self::convert_response(ollama_response, request.model)
    }

    async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<crate::provider::ChatStream, ProviderError> {
        debug!(
            "Starting streaming chat request to Ollama for model: {}",
            request.model
        );

        let ollama_request = OllamaChatRequest {
            model: request.model.clone(),
            messages: request
                .messages
                .iter()
                .map(|m| OllamaMessage {
                    role: m.role.clone(),
                    content: m.content.clone(),
                })
                .collect(),
            stream: true,
        };

        let base_url = self.base_url.clone();
        let client = self.client.clone();
        let model = request.model.clone();

        let response = execute_with_retry(|| {
            let client = client.clone();
            let url = format!("{}/api/chat", base_url);
            let req = ollama_request.clone();
            async move { client.post(url).json(&req).send().await }
        })
        .await
        .map_err(|e| {
            error!("Ollama streaming request failed after retries: {}", e);
            ProviderError::NetworkError(e.to_string())
        })?;

        let status = response.status();
        if !status.is_success() {
            return Err(ProviderError::ProviderError(format!(
                "Ollama API error: {}",
                status
            )));
        }

        // Read the entire response body and parse it line by line
        // This creates a stream that yields responses as they are parsed
        let body = response.text().await.map_err(|e| {
            error!("Failed to read streaming response body: {}", e);
            ProviderError::NetworkError(e.to_string())
        })?;

        // Parse each line as a JSON object and create a stream
        let responses: Vec<Result<ChatResponse, ProviderError>> = body
            .lines()
            .filter(|line| !line.is_empty())
            .map(
                |line| match serde_json::from_str::<OllamaChatResponse>(line) {
                    Ok(ollama_response) => Ok(ChatResponse {
                        content: ollama_response.message.content,
                        model: model.clone(),
                        usage: TokenUsage {
                            prompt_tokens: 0,
                            completion_tokens: 0,
                            total_tokens: 0,
                        },
                        finish_reason: if ollama_response.done {
                            FinishReason::Stop
                        } else {
                            FinishReason::Error
                        },
                    }),
                    Err(e) => {
                        debug!("Failed to parse streaming response line: {}", e);
                        Err(ProviderError::ParseError(format!(
                            "Failed to parse streaming response: {}",
                            e
                        )))
                    }
                },
            )
            .collect();

        // Convert to a stream
        let chat_stream = futures::stream::iter(responses);
        Ok(chat_stream.boxed())
    }

    fn count_tokens(&self, content: &str, _model: &str) -> Result<usize, ProviderError> {
        // Ollama doesn't provide an exact token counting API
        // Use a reasonable approximation: 1 token ≈ 4 characters
        let token_count = content.len().div_ceil(4);
        Ok(token_count)
    }

    async fn health_check(&self) -> Result<bool, ProviderError> {
        debug!("Performing health check for Ollama provider");

        let base_url = self.base_url.clone();
        let client = self.client.clone();

        let response = execute_with_retry(|| {
            let client = client.clone();
            let url = format!("{}/api/tags", base_url);
            async move { client.get(url).send().await }
        })
        .await
        .map_err(|e| {
            warn!("Ollama health check failed after retries: {}", e);
            ProviderError::NetworkError(e.to_string())
        })?;

        match response.status().as_u16() {
            200 => {
                debug!("Ollama health check passed");
                Ok(true)
            }
            _ => {
                warn!(
                    "Ollama health check failed with status: {}",
                    response.status()
                );
                Ok(false)
            }
        }
    }
}

/// Ollama API chat request format
#[derive(Debug, Serialize, Clone)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
}

/// Ollama API message format
#[derive(Debug, Serialize, Deserialize, Clone)]
struct OllamaMessage {
    role: String,
    content: String,
}

/// Ollama API chat response format
#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    message: OllamaResponseMessage,
    done: bool,
}

/// Ollama API response message format
#[derive(Debug, Deserialize)]
struct OllamaResponseMessage {
    #[allow(dead_code)]
    role: String,
    content: String,
}

/// Ollama API tags response format
#[derive(Debug, Deserialize)]
struct OllamaTagsResponse {
    models: Option<Vec<OllamaModel>>,
}

/// Ollama model information
#[derive(Debug, Deserialize, Clone)]
struct OllamaModel {
    name: String,
}
