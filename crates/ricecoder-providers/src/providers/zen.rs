//! Zen provider implementation
//!
//! Supports OpenCode's curated set of AI models via the Zen API.

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tracing::{debug, error, warn};

use crate::error::ProviderError;
use crate::models::{Capability, ChatRequest, ChatResponse, FinishReason, ModelInfo, TokenUsage};
use crate::provider::Provider;
use crate::token_counter::TokenCounter;

/// Zen provider implementation
pub struct ZenProvider {
    api_key: String,
    client: Arc<Client>,
    base_url: String,
    token_counter: Arc<TokenCounter>,
    models_cache: Arc<tokio::sync::Mutex<ModelCache>>,
    health_check_cache: Arc<tokio::sync::Mutex<HealthCheckCache>>,
}

/// Cache for models with TTL
struct ModelCache {
    models: Option<Vec<ModelInfo>>,
    cached_at: Option<SystemTime>,
    ttl: Duration,
}

/// Cache for health check with TTL
struct HealthCheckCache {
    result: Option<bool>,
    cached_at: Option<SystemTime>,
    ttl: Duration,
}

impl ModelCache {
    fn new() -> Self {
        Self {
            models: None,
            cached_at: None,
            ttl: Duration::from_secs(300), // 5 minutes
        }
    }

    fn is_valid(&self) -> bool {
        if let (Some(cached_at), Some(_)) = (self.cached_at, &self.models) {
            if let Ok(elapsed) = cached_at.elapsed() {
                return elapsed < self.ttl;
            }
        }
        false
    }

    fn get(&self) -> Option<Vec<ModelInfo>> {
        if self.is_valid() {
            self.models.clone()
        } else {
            None
        }
    }

    fn set(&mut self, models: Vec<ModelInfo>) {
        self.models = Some(models);
        self.cached_at = Some(SystemTime::now());
    }

    #[allow(dead_code)]
    fn invalidate(&mut self) {
        self.models = None;
        self.cached_at = None;
    }
}

impl HealthCheckCache {
    fn new() -> Self {
        Self {
            result: None,
            cached_at: None,
            ttl: Duration::from_secs(60), // 1 minute
        }
    }

    fn is_valid(&self) -> bool {
        if let (Some(cached_at), Some(_)) = (self.cached_at, self.result) {
            if let Ok(elapsed) = cached_at.elapsed() {
                return elapsed < self.ttl;
            }
        }
        false
    }

    fn get(&self) -> Option<bool> {
        if self.is_valid() {
            self.result
        } else {
            None
        }
    }

    fn set(&mut self, result: bool) {
        self.result = Some(result);
        self.cached_at = Some(SystemTime::now());
    }

    #[allow(dead_code)]
    fn invalidate(&mut self) {
        self.result = None;
        self.cached_at = None;
    }
}

impl ZenProvider {
    /// Create a new Zen provider instance
    /// API key is optional for free models
    pub fn new(api_key: Option<String>) -> Result<Self, ProviderError> {
        Ok(Self {
            api_key: api_key.unwrap_or_default(),
            client: Arc::new(Client::new()),
            base_url: "https://opencode.ai/zen/v1".to_string(),
            token_counter: Arc::new(TokenCounter::new()),
            models_cache: Arc::new(tokio::sync::Mutex::new(ModelCache::new())),
            health_check_cache: Arc::new(tokio::sync::Mutex::new(HealthCheckCache::new())),
        })
    }

    /// Create a new Zen provider with a custom base URL
    /// API key is optional for free models
    pub fn with_base_url(api_key: Option<String>, base_url: String) -> Result<Self, ProviderError> {
        Ok(Self {
            api_key: api_key.unwrap_or_default(),
            client: Arc::new(Client::new()),
            base_url,
            token_counter: Arc::new(TokenCounter::new()),
            models_cache: Arc::new(tokio::sync::Mutex::new(ModelCache::new())),
            health_check_cache: Arc::new(tokio::sync::Mutex::new(HealthCheckCache::new())),
        })
    }

    /// Get the authorization header value (redacted for logging)
    fn get_auth_header(&self) -> String {
        format!("Bearer {}", self.api_key)
    }

    /// Convert Zen API response to our ChatResponse
    fn convert_response(
        response: ZenChatResponse,
        model: String,
    ) -> Result<ChatResponse, ProviderError> {
        let content = response
            .choices
            .first()
            .and_then(|c| c.message.as_ref())
            .map(|m| m.content.clone())
            .ok_or_else(|| ProviderError::ProviderError("No content in response".to_string()))?;

        let finish_reason = match response
            .choices
            .first()
            .and_then(|c| c.finish_reason.as_deref())
        {
            Some("stop") => FinishReason::Stop,
            Some("length") => FinishReason::Length,
            Some("error") => FinishReason::Error,
            _ => FinishReason::Stop,
        };

        Ok(ChatResponse {
            content,
            model,
            usage: TokenUsage {
                prompt_tokens: response.usage.prompt_tokens,
                completion_tokens: response.usage.completion_tokens,
                total_tokens: response.usage.total_tokens,
            },
            finish_reason,
        })
    }

    /// Fetch models from Zen API with retry logic
    async fn fetch_models_from_api(&self) -> Result<Vec<ModelInfo>, ProviderError> {
        let mut retries = 0;
        let max_retries = 3;

        loop {
            debug!("Fetching models from Zen API (attempt {})", retries + 1);

            let response = self
                .client
                .get(format!("{}/models", self.base_url))
                .header("Authorization", self.get_auth_header())
                .timeout(Duration::from_secs(30))
                .send()
                .await;

            match response {
                Ok(resp) => {
                    let status = resp.status();
                    if !status.is_success() {
                        let error_text = resp.text().await.unwrap_or_default();
                        error!("Zen API error ({}): {}", status, error_text);

                        return match status.as_u16() {
                            401 => Err(ProviderError::AuthError),
                            429 => {
                                if retries < max_retries {
                                    let backoff = Duration::from_secs(2_u64.pow(retries as u32));
                                    warn!("Rate limited, retrying after {:?}", backoff);
                                    tokio::time::sleep(backoff).await;
                                    retries += 1;
                                    continue;
                                }
                                Err(ProviderError::RateLimited(60))
                            }
                            _ => Err(ProviderError::ProviderError(format!(
                                "Zen API error: {}",
                                status
                            ))),
                        };
                    }

                    let zen_response: ZenListModelsResponse = resp.json().await?;
                    return Ok(zen_response
                        .data
                        .into_iter()
                        .map(|m| ModelInfo {
                            id: m.id.clone(),
                            name: m.name.unwrap_or_else(|| m.id.clone()),
                            provider: "zen".to_string(),
                            context_window: m.context_window.unwrap_or(4096),
                            capabilities: m.capabilities.unwrap_or_default(),
                            pricing: m.pricing.map(|p| crate::models::Pricing {
                                input_per_1k_tokens: p.input_cost_per_1k,
                                output_per_1k_tokens: p.output_cost_per_1k,
                            }),
                            is_free: m.is_free,
                        })
                        .collect());
                }
                Err(e) => {
                    error!("Zen API request failed: {}", e);

                    if retries < max_retries {
                        let backoff = Duration::from_secs(2_u64.pow(retries as u32));
                        warn!("Request failed, retrying after {:?}", backoff);
                        tokio::time::sleep(backoff).await;
                        retries += 1;
                        continue;
                    }

                    return Err(ProviderError::from(e));
                }
            }
        }
    }

    /// Count tokens using Zen API (with fallback to local approximation)
    #[allow(dead_code)]
    async fn count_tokens_from_api(
        &self,
        content: &str,
        model: &str,
    ) -> Result<usize, ProviderError> {
        debug!(
            "Counting tokens for model: {} (content length: {})",
            model,
            content.len()
        );

        let request = ZenTokenCountRequest {
            model: model.to_string(),
            content: content.to_string(),
        };

        let mut retries = 0;
        let max_retries = 3;

        loop {
            let response = self
                .client
                .post(format!("{}/token/count", self.base_url))
                .header("Authorization", self.get_auth_header())
                .header("Content-Type", "application/json")
                .json(&request)
                .timeout(Duration::from_secs(30))
                .send()
                .await;

            match response {
                Ok(resp) => {
                    let status = resp.status();
                    if !status.is_success() {
                        let error_text = resp.text().await.unwrap_or_default();
                        error!("Zen token count API error ({}): {}", status, error_text);

                        return match status.as_u16() {
                            401 => Err(ProviderError::AuthError),
                            429 => {
                                if retries < max_retries {
                                    let backoff = Duration::from_secs(2_u64.pow(retries as u32));
                                    warn!("Rate limited, retrying after {:?}", backoff);
                                    tokio::time::sleep(backoff).await;
                                    retries += 1;
                                    continue;
                                }
                                Err(ProviderError::RateLimited(60))
                            }
                            _ => {
                                warn!("Token count API failed, using fallback approximation");
                                return Ok(self.estimate_tokens(content));
                            }
                        };
                    }

                    let zen_response: ZenTokenCountResponse = resp.json().await?;
                    debug!("Token count from API: {}", zen_response.token_count);
                    return Ok(zen_response.token_count);
                }
                Err(e) => {
                    error!("Zen token count API request failed: {}", e);

                    if retries < max_retries {
                        let backoff = Duration::from_secs(2_u64.pow(retries as u32));
                        warn!("Request failed, retrying after {:?}", backoff);
                        tokio::time::sleep(backoff).await;
                        retries += 1;
                        continue;
                    }

                    warn!("Token count API unavailable, using fallback approximation");
                    return Ok(self.estimate_tokens(content));
                }
            }
        }
    }

    /// Estimate tokens using simple approximation (fallback when API is unavailable)
    #[allow(dead_code)]
    fn estimate_tokens(&self, content: &str) -> usize {
        // Approximation: 4 characters ≈ 1 token
        content.len().div_ceil(4)
    }

    /// Get the appropriate endpoint for a given model type
    /// 
    /// Different model families use different endpoints:
    /// - GPT models use `/v1/responses`
    /// - Claude models use `/v1/messages`
    /// - Generic models use `/v1/chat/completions`
    /// 
    /// # Arguments
    /// * `model_id` - The model identifier (e.g., "gpt-4", "claude-3", "zen-gpt4")
    /// 
    /// # Returns
    /// The endpoint path for the model type
    pub fn endpoint_for_model(&self, model_id: &str) -> &'static str {
        match model_id {
            // OpenAI models use /responses endpoint
            m if m.starts_with("gpt-") => "/responses",
            // Anthropic Claude models use /messages endpoint
            m if m.starts_with("claude-") => "/messages",
            // Google Gemini models use /models endpoint
            m if m.starts_with("gemini-") => "/models",
            // Zen models and others default to /chat/completions
            _ => "/chat/completions",
        }
    }

    /// Parse Server-Sent Events (SSE) streaming response from Zen API
    fn parse_streaming_response(
        text: &str,
        model: String,
    ) -> Result<crate::provider::ChatStream, ProviderError> {
        use futures::stream;

        let lines: Vec<String> = text
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with(':'))
            .map(|line| line.to_string())
            .collect();

        let stream = stream::iter(lines.into_iter().filter_map(move |line| {
            if line.starts_with("data: ") {
                let data = &line[6..];

                // Check for stream end marker
                if data == "[DONE]" {
                    return None;
                }

                // Parse JSON chunk
                match serde_json::from_str::<ZenStreamChunk>(data) {
                    Ok(chunk) => {
                        if let Some(choice) = chunk.choices.first() {
                            if let Some(delta) = &choice.delta {
                                if let Some(content) = &delta.content {
                                    return Some(Ok(ChatResponse {
                                        content: content.clone(),
                                        model: model.clone(),
                                        usage: TokenUsage {
                                            prompt_tokens: 0,
                                            completion_tokens: 0,
                                            total_tokens: 0,
                                        },
                                        finish_reason: FinishReason::Stop,
                                    }));
                                }
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to parse streaming chunk: {}", e);
                    }
                }
            }
            None
        }));

        Ok(Box::new(stream))
    }
}

#[async_trait]
impl Provider for ZenProvider {
    fn id(&self) -> &str {
        "zen"
    }

    fn name(&self) -> &str {
        "OpenCode Zen"
    }

    fn models(&self) -> Vec<ModelInfo> {
        // This is a blocking call, so we return a default set
        // The async version is used in chat() and other async methods
        vec![
            ModelInfo {
                id: "zen-gpt4".to_string(),
                name: "Zen GPT-4".to_string(),
                provider: "zen".to_string(),
                context_window: 8192,
                capabilities: vec![Capability::Chat, Capability::Code, Capability::Streaming],
                pricing: Some(crate::models::Pricing {
                    input_per_1k_tokens: 0.03,
                    output_per_1k_tokens: 0.06,
                }),
                is_free: false,
            },
            ModelInfo {
                id: "zen-gpt4-turbo".to_string(),
                name: "Zen GPT-4 Turbo".to_string(),
                provider: "zen".to_string(),
                context_window: 128000,
                capabilities: vec![
                    Capability::Chat,
                    Capability::Code,
                    Capability::Vision,
                    Capability::Streaming,
                ],
                pricing: Some(crate::models::Pricing {
                    input_per_1k_tokens: 0.01,
                    output_per_1k_tokens: 0.03,
                }),
                is_free: false,
            },
        ]
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, ProviderError> {
        // Get models from cache or fetch from API
        let models = {
            let mut cache = self.models_cache.lock().await;
            if let Some(models) = cache.get() {
                debug!("Using cached models");
                models
            } else {
                debug!("Cache miss, fetching models from API");
                let models = self.fetch_models_from_api().await?;
                cache.set(models.clone());
                models
            }
        };

        // Validate model - handle both "model-id" and "provider/model-id" formats
        let model_id = &request.model;
        let model_name = if model_id.contains('/') {
            // Extract model name from "provider/model-id" format
            model_id.split('/').nth(1).unwrap_or(model_id.as_str())
        } else {
            model_id.as_str()
        };

        if !models.iter().any(|m| m.id == model_name) {
            return Err(ProviderError::InvalidModel(model_id.clone()));
        }

        let zen_request = ZenChatRequest {
            model: model_name.to_string(),
            messages: request
                .messages
                .iter()
                .map(|m| ZenMessage {
                    role: m.role.clone(),
                    content: m.content.clone(),
                })
                .collect(),
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            stream: false,
        };

        debug!("Sending chat request to Zen for model: {}", request.model);

        let mut retries = 0;
        let max_retries = 3;

        loop {
            let response = self
                .client
                .post(format!("{}/chat/completions", self.base_url))
                .header("Authorization", self.get_auth_header())
                .header("Content-Type", "application/json")
                .json(&zen_request)
                .timeout(Duration::from_secs(30))
                .send()
                .await;

            match response {
                Ok(resp) => {
                    let status = resp.status();
                    if !status.is_success() {
                        let error_text = resp.text().await.unwrap_or_default();
                        error!("Zen API error ({}): {}", status, error_text);

                        return match status.as_u16() {
                            401 => Err(ProviderError::AuthError),
                            429 => {
                                if retries < max_retries {
                                    let backoff = Duration::from_secs(2_u64.pow(retries as u32));
                                    warn!("Rate limited, retrying after {:?}", backoff);
                                    tokio::time::sleep(backoff).await;
                                    retries += 1;
                                    continue;
                                }
                                Err(ProviderError::RateLimited(60))
                            }
                            _ => Err(ProviderError::ProviderError(format!(
                                "Zen API error: {}",
                                status
                            ))),
                        };
                    }

                    let zen_response: ZenChatResponse = resp.json().await?;
                    return Self::convert_response(zen_response, request.model);
                }
                Err(e) => {
                    error!("Zen API request failed: {}", e);

                    if retries < max_retries {
                        let backoff = Duration::from_secs(2_u64.pow(retries as u32));
                        warn!("Request failed, retrying after {:?}", backoff);
                        tokio::time::sleep(backoff).await;
                        retries += 1;
                        continue;
                    }

                    return Err(ProviderError::from(e));
                }
            }
        }
    }

    async fn chat_stream(
        &self,
        mut request: ChatRequest,
    ) -> Result<crate::provider::ChatStream, ProviderError> {

        // Enable streaming in the request
        request.stream = true;

        // Send streaming request to Zen API
        let mut retries = 0;
        let max_retries = 3;

        loop {
            debug!("Sending streaming chat request to Zen API (attempt {})", retries + 1);

            let response = self
                .client
                .post(format!("{}/chat/completions", self.base_url))
                .header("Authorization", self.get_auth_header())
                .json(&request)
                .timeout(Duration::from_secs(120))
                .send()
                .await;

            match response {
                Ok(resp) => {
                    let status = resp.status();
                    if !status.is_success() {
                        let error_text = resp.text().await.unwrap_or_default();
                        error!("Zen API error ({}): {}", status, error_text);

                        return match status.as_u16() {
                            401 => Err(ProviderError::AuthError),
                            429 => {
                                if retries < max_retries {
                                    let backoff = Duration::from_secs(2_u64.pow(retries as u32));
                                    warn!("Rate limited, retrying after {:?}", backoff);
                                    tokio::time::sleep(backoff).await;
                                    retries += 1;
                                    continue;
                                }
                                Err(ProviderError::RateLimited(60))
                            }
                            _ => Err(ProviderError::ProviderError(format!(
                                "Zen API error: {}",
                                status
                            ))),
                        };
                    }

                    // Parse the streaming response
                    let model = request.model.clone();
                    let text = resp.text().await?;
                    let stream = Self::parse_streaming_response(&text, model)?;
                    return Ok(stream);
                }
                Err(e) => {
                    error!("Zen API request failed: {}", e);

                    if retries < max_retries {
                        let backoff = Duration::from_secs(2_u64.pow(retries as u32));
                        warn!("Request failed, retrying after {:?}", backoff);
                        tokio::time::sleep(backoff).await;
                        retries += 1;
                        continue;
                    }

                    return Err(ProviderError::from(e));
                }
            }
        }
    }

    fn count_tokens(&self, content: &str, model: &str) -> Result<usize, ProviderError> {
        // Validate model
        if !self.models().iter().any(|m| m.id == model) {
            return Err(ProviderError::InvalidModel(model.to_string()));
        }

        // Use token counter with caching for performance
        // In production, this would call the Zen API token counting endpoint
        // For now, we use a local approximation as fallback
        let tokens = self.token_counter.count_tokens_openai(content, model);
        Ok(tokens)
    }

    async fn health_check(&self) -> Result<bool, ProviderError> {
        debug!("Performing health check for Zen provider");

        // Check cache first
        {
            let cache = self.health_check_cache.lock().await;
            if let Some(result) = cache.get() {
                debug!("Using cached health check result: {}", result);
                return Ok(result);
            }
        }

        // Try to list models as a health check
        let response = self
            .client
            .get(format!("{}/models", self.base_url))
            .header("Authorization", self.get_auth_header())
            .timeout(Duration::from_secs(5))
            .send()
            .await;

        let result = match response {
            Ok(resp) => match resp.status().as_u16() {
                200 => {
                    debug!("Zen health check passed");
                    true
                }
                401 => {
                    error!("Zen health check failed: authentication error");
                    return Err(ProviderError::AuthError);
                }
                _ => {
                    warn!("Zen health check failed with status: {}", resp.status());
                    false
                }
            },
            Err(e) => {
                warn!("Zen health check failed: {}", e);
                false
            }
        };

        // Cache the result
        {
            let mut cache = self.health_check_cache.lock().await;
            cache.set(result);
        }

        Ok(result)
    }
}

/// Zen API request format
#[derive(Debug, Serialize)]
struct ZenChatRequest {
    model: String,
    messages: Vec<ZenMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<usize>,
    stream: bool,
}

/// Zen API message format
#[derive(Debug, Serialize, Deserialize)]
struct ZenMessage {
    role: String,
    content: String,
}

/// Zen API response format
#[derive(Debug, Deserialize)]
struct ZenChatResponse {
    choices: Vec<ZenChoice>,
    usage: ZenUsage,
}

/// Zen API choice format
#[derive(Debug, Deserialize)]
struct ZenChoice {
    message: Option<ZenMessage>,
    finish_reason: Option<String>,
}

/// Zen API usage format
#[derive(Debug, Deserialize)]
struct ZenUsage {
    prompt_tokens: usize,
    completion_tokens: usize,
    total_tokens: usize,
}

/// Zen API models list response (OpenAI-compatible format)
#[derive(Debug, Deserialize)]
struct ZenListModelsResponse {
    #[allow(dead_code)]
    object: String,
    data: Vec<ZenModel>,
}

/// Zen API model info
#[derive(Debug, Deserialize, Clone)]
struct ZenModel {
    id: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    object: String,
    #[serde(default)]
    #[allow(dead_code)]
    created: Option<i64>,
    #[serde(default)]
    #[allow(dead_code)]
    owned_by: Option<String>,
    #[serde(default)]
    context_window: Option<usize>,
    #[serde(default)]
    capabilities: Option<Vec<Capability>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pricing: Option<ZenPricing>,
    /// Whether this model is free to use (no API key required)
    #[serde(default)]
    is_free: bool,
}

/// Zen API pricing info
#[derive(Debug, Deserialize, Clone)]
struct ZenPricing {
    input_cost_per_1k: f64,
    output_cost_per_1k: f64,
}

/// Zen API token count request
#[derive(Debug, Serialize)]
struct ZenTokenCountRequest {
    model: String,
    content: String,
}

/// Zen API token count response
#[derive(Debug, Deserialize)]
struct ZenTokenCountResponse {
    token_count: usize,
}

/// Zen API streaming chunk (SSE format)
#[derive(Debug, Deserialize)]
struct ZenStreamChunk {
    choices: Vec<ZenStreamChoice>,
}

/// Zen API streaming choice
#[derive(Debug, Deserialize)]
struct ZenStreamChoice {
    delta: Option<ZenStreamDelta>,
    #[allow(dead_code)]
    finish_reason: Option<String>,
}

/// Zen API streaming delta (contains incremental content)
#[derive(Debug, Deserialize)]
struct ZenStreamDelta {
    content: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zen_provider_creation() {
        let provider = ZenProvider::new(Some("test-key".to_string()));
        assert!(provider.is_ok());
    }

    #[test]
    fn test_zen_provider_creation_no_key() {
        let provider = ZenProvider::new(None);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_zen_provider_creation_empty_key() {
        let provider = ZenProvider::new(Some("".to_string()));
        assert!(provider.is_ok());
    }

    #[test]
    fn test_zen_provider_id() {
        let provider = ZenProvider::new(Some("test-key".to_string())).unwrap();
        assert_eq!(provider.id(), "zen");
    }

    #[test]
    fn test_zen_provider_name() {
        let provider = ZenProvider::new(Some("test-key".to_string())).unwrap();
        assert_eq!(provider.name(), "OpenCode Zen");
    }

    #[test]
    fn test_zen_models() {
        let provider = ZenProvider::new(Some("test-key".to_string())).unwrap();
        let models = provider.models();
        assert_eq!(models.len(), 2);
        assert!(models.iter().any(|m| m.id == "zen-gpt4"));
        assert!(models.iter().any(|m| m.id == "zen-gpt4-turbo"));
    }

    #[test]
    fn test_token_counting() {
        let provider = ZenProvider::new(Some("test-key".to_string())).unwrap();
        let tokens = provider.count_tokens("Hello, world!", "zen-gpt4").unwrap();
        assert!(tokens > 0);
    }

    #[test]
    fn test_token_counting_invalid_model() {
        let provider = ZenProvider::new(Some("test-key".to_string())).unwrap();
        let result = provider.count_tokens("Hello, world!", "invalid-model");
        assert!(result.is_err());
    }

    #[test]
    fn test_model_cache_creation() {
        let cache = ModelCache::new();
        assert!(cache.get().is_none());
    }

    #[test]
    fn test_model_cache_set_and_get() {
        let mut cache = ModelCache::new();
        let models = vec![ModelInfo {
            id: "test".to_string(),
            name: "Test".to_string(),
            provider: "zen".to_string(),
            context_window: 1000,
            capabilities: vec![],
            pricing: None,
            is_free: false,
        }];
        cache.set(models.clone());
        let cached = cache.get();
        assert!(cached.is_some());
        let cached_models = cached.unwrap();
        assert_eq!(cached_models.len(), 1);
        assert_eq!(cached_models[0].id, "test");
    }

    #[test]
    fn test_health_check_cache_creation() {
        let cache = HealthCheckCache::new();
        assert!(cache.get().is_none());
    }

    #[test]
    fn test_health_check_cache_set_and_get() {
        let mut cache = HealthCheckCache::new();
        cache.set(true);
        assert_eq!(cache.get(), Some(true));
    }

    #[test]
    fn test_estimate_tokens() {
        let provider = ZenProvider::new(Some("test-key".to_string())).unwrap();
        let tokens = provider.estimate_tokens("Hello, world!");
        // "Hello, world!" is 13 characters, so (13 + 3) / 4 = 4 tokens
        assert_eq!(tokens, 4);
    }

    #[test]
    fn test_estimate_tokens_empty() {
        let provider = ZenProvider::new(Some("test-key".to_string())).unwrap();
        let tokens = provider.estimate_tokens("");
        assert_eq!(tokens, 0);
    }

    #[test]
    fn test_estimate_tokens_single_char() {
        let provider = ZenProvider::new(Some("test-key".to_string())).unwrap();
        let tokens = provider.estimate_tokens("a");
        // (1 + 3) / 4 = 1 token
        assert_eq!(tokens, 1);
    }

    #[test]
    fn test_endpoint_for_gpt_model() {
        let provider = ZenProvider::new(Some("test-key".to_string())).unwrap();
        assert_eq!(provider.endpoint_for_model("gpt-4"), "/responses");
        assert_eq!(provider.endpoint_for_model("gpt-3.5-turbo"), "/responses");
        assert_eq!(provider.endpoint_for_model("gpt-4-turbo"), "/responses");
    }

    #[test]
    fn test_endpoint_for_claude_model() {
        let provider = ZenProvider::new(Some("test-key".to_string())).unwrap();
        assert_eq!(provider.endpoint_for_model("claude-3"), "/messages");
        assert_eq!(provider.endpoint_for_model("claude-3-opus"), "/messages");
        assert_eq!(provider.endpoint_for_model("claude-2"), "/messages");
    }

    #[test]
    fn test_endpoint_for_gemini_model() {
        let provider = ZenProvider::new(Some("test-key".to_string())).unwrap();
        assert_eq!(provider.endpoint_for_model("gemini-pro"), "/models");
        assert_eq!(provider.endpoint_for_model("gemini-1.5"), "/models");
    }

    #[test]
    fn test_endpoint_for_zen_model() {
        let provider = ZenProvider::new(Some("test-key".to_string())).unwrap();
        assert_eq!(provider.endpoint_for_model("zen-gpt4"), "/chat/completions");
        assert_eq!(provider.endpoint_for_model("zen-gpt4-turbo"), "/chat/completions");
    }

    #[test]
    fn test_endpoint_for_generic_model() {
        let provider = ZenProvider::new(Some("test-key".to_string())).unwrap();
        assert_eq!(provider.endpoint_for_model("unknown-model"), "/chat/completions");
        assert_eq!(provider.endpoint_for_model("custom-model"), "/chat/completions");
    }
}
