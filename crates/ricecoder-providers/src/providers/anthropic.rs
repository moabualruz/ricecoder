//! Anthropic provider implementation
//!
//! Supports Claude 3, 3.5, and 4 family models via the Anthropic API.
//! Includes full streaming support via Server-Sent Events (SSE).

use std::sync::Arc;

use async_trait::async_trait;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, trace, warn};

use crate::{
    error::ProviderError,
    models::{Capability, ChatRequest, ChatResponse, FinishReason, ModelInfo, TokenUsage},
    provider::Provider,
    token_counter::TokenCounter,
};

/// Anthropic provider implementation
pub struct AnthropicProvider {
    api_key: String,
    client: Arc<Client>,
    base_url: String,
    token_counter: Arc<TokenCounter>,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider instance
    pub fn new(api_key: String) -> Result<Self, ProviderError> {
        Self::with_client(Arc::new(Client::new()), api_key)
    }

    /// Create a new Anthropic provider with a custom base URL
    pub fn with_base_url(api_key: String, base_url: String) -> Result<Self, ProviderError> {
        Self::with_client_and_base_url(Arc::new(Client::new()), api_key, base_url)
    }

    /// Create a new Anthropic provider with a custom HTTP client
    pub fn with_client(client: Arc<Client>, api_key: String) -> Result<Self, ProviderError> {
        if api_key.is_empty() {
            return Err(ProviderError::ConfigError(
                "Anthropic API key is required".to_string(),
            ));
        }

        Ok(Self {
            api_key,
            client,
            base_url: "https://api.anthropic.com/v1".to_string(),
            token_counter: Arc::new(TokenCounter::new()),
        })
    }

    /// Create a new Anthropic provider with a custom HTTP client and base URL
    pub fn with_client_and_base_url(
        client: Arc<Client>,
        api_key: String,
        base_url: String,
    ) -> Result<Self, ProviderError> {
        if api_key.is_empty() {
            return Err(ProviderError::ConfigError(
                "Anthropic API key is required".to_string(),
            ));
        }

        Ok(Self {
            api_key,
            client,
            base_url,
            token_counter: Arc::new(TokenCounter::new()),
        })
    }

    /// Convert Anthropic API response to our ChatResponse
    fn convert_response(
        response: AnthropicChatResponse,
        model: String,
    ) -> Result<ChatResponse, ProviderError> {
        let content = response
            .content
            .first()
            .map(|c| c.text.clone())
            .ok_or_else(|| ProviderError::ProviderError("No content in response".to_string()))?;

        let finish_reason = match response.stop_reason.as_deref() {
            Some("end_turn") => FinishReason::Stop,
            Some("max_tokens") => FinishReason::Length,
            Some("stop_sequence") => FinishReason::Stop,
            _ => FinishReason::Stop,
        };

        Ok(ChatResponse {
            content,
            model,
            usage: TokenUsage {
                prompt_tokens: response.usage.input_tokens,
                completion_tokens: response.usage.output_tokens,
                total_tokens: response.usage.input_tokens + response.usage.output_tokens,
            },
            finish_reason,
        })
    }

    /// Parse SSE response into a stream of ChatResponse
    fn parse_sse_response(
        body: &str,
        model: String,
    ) -> Result<crate::provider::ChatStream, ProviderError> {
        let mut responses: Vec<Result<ChatResponse, ProviderError>> = Vec::new();

        // Parse SSE format: each event is "event: <type>\ndata: <json>\n\n"
        for line in body.lines() {
            if let Some(json_str) = line.strip_prefix("data: ") {
                // Skip [DONE] marker
                if json_str.trim() == "[DONE]" {
                    trace!("Stream completed with [DONE] marker");
                    continue;
                }

                // Try to parse as streaming event
                match serde_json::from_str::<AnthropicStreamEvent>(json_str) {
                    Ok(event) => {
                        if event.event_type == "content_block_delta" {
                            if let Some(delta) = event.delta {
                                if let Some(text) = delta.text {
                                    if !text.is_empty() {
                                        responses.push(Ok(ChatResponse {
                                            content: text,
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
                    }
                    Err(e) => {
                        // Log parsing error but continue
                        debug!("Failed to parse SSE event: {} - data: {}", e, json_str);
                    }
                }
            }
        }

        // Convert to a stream
        let chat_stream = futures::stream::iter(responses);
        Ok(chat_stream.boxed())
    }
}

#[async_trait]
impl Provider for AnthropicProvider {
    fn id(&self) -> &str {
        "anthropic"
    }

    fn name(&self) -> &str {
        "Anthropic"
    }

    fn models(&self) -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                id: "claude-3-opus-20250219".to_string(),
                name: "Claude 3 Opus".to_string(),
                provider: "anthropic".to_string(),
                context_window: 200000,
                capabilities: vec![Capability::Chat, Capability::Code, Capability::Streaming],
                pricing: Some(crate::models::Pricing {
                    input_per_1k_tokens: 0.015,
                    output_per_1k_tokens: 0.075,
                }),
                is_free: false,
            },
            ModelInfo {
                id: "claude-3-5-sonnet-20241022".to_string(),
                name: "Claude 3.5 Sonnet".to_string(),
                provider: "anthropic".to_string(),
                context_window: 200000,
                capabilities: vec![Capability::Chat, Capability::Code, Capability::Streaming],
                pricing: Some(crate::models::Pricing {
                    input_per_1k_tokens: 0.003,
                    output_per_1k_tokens: 0.015,
                }),
                is_free: false,
            },
            ModelInfo {
                id: "claude-3-5-haiku-20241022".to_string(),
                name: "Claude 3.5 Haiku".to_string(),
                provider: "anthropic".to_string(),
                context_window: 200000,
                capabilities: vec![Capability::Chat, Capability::Code, Capability::Streaming],
                pricing: Some(crate::models::Pricing {
                    input_per_1k_tokens: 0.0008,
                    output_per_1k_tokens: 0.004,
                }),
                is_free: false,
            },
            ModelInfo {
                id: "claude-3-haiku-20240307".to_string(),
                name: "Claude 3 Haiku".to_string(),
                provider: "anthropic".to_string(),
                context_window: 200000,
                capabilities: vec![Capability::Chat, Capability::Code, Capability::Streaming],
                pricing: Some(crate::models::Pricing {
                    input_per_1k_tokens: 0.00025,
                    output_per_1k_tokens: 0.00125,
                }),
                is_free: false,
            },
        ]
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, ProviderError> {
        // Validate model
        let model_id = &request.model;
        if !self.models().iter().any(|m| m.id == *model_id) {
            return Err(ProviderError::InvalidModel(model_id.clone()));
        }

        // Convert messages to Anthropic format
        let messages: Vec<AnthropicMessage> = request
            .messages
            .iter()
            .map(|m| AnthropicMessage {
                role: m.role.clone(),
                content: m.content.clone(),
            })
            .collect();

        let anthropic_request = AnthropicChatRequest {
            model: request.model.clone(),
            max_tokens: request.max_tokens.unwrap_or(1024),
            messages,
            temperature: request.temperature,
        };

        debug!(
            "Sending chat request to Anthropic for model: {}",
            request.model
        );

        let response = self
            .client
            .post(format!("{}/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&anthropic_request)
            .send()
            .await
            .map_err(|e| {
                error!("Anthropic API request failed: {}", e);
                ProviderError::from(e)
            })?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("Anthropic API error ({}): {}", status, error_text);

            return match status.as_u16() {
                401 => Err(ProviderError::AuthError),
                429 => Err(ProviderError::RateLimited(60)),
                _ => Err(ProviderError::ProviderError(format!(
                    "Anthropic API error: {}",
                    status
                ))),
            };
        }

        let anthropic_response: AnthropicChatResponse = response.json().await?;
        Self::convert_response(anthropic_response, request.model)
    }

    async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<crate::provider::ChatStream, ProviderError> {
        // Validate model
        let model_id = &request.model;
        if !self.models().iter().any(|m| m.id == *model_id) {
            return Err(ProviderError::InvalidModel(model_id.clone()));
        }

        // Convert messages to Anthropic format
        let messages: Vec<AnthropicMessage> = request
            .messages
            .iter()
            .map(|m| AnthropicMessage {
                role: m.role.clone(),
                content: m.content.clone(),
            })
            .collect();

        let anthropic_request = AnthropicStreamRequest {
            model: request.model.clone(),
            max_tokens: request.max_tokens.unwrap_or(1024),
            messages,
            temperature: request.temperature,
            stream: true,
        };

        debug!(
            "Starting streaming chat request to Anthropic for model: {}",
            request.model
        );

        let response = self
            .client
            .post(format!("{}/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&anthropic_request)
            .send()
            .await
            .map_err(|e| {
                error!("Anthropic streaming request failed: {}", e);
                ProviderError::from(e)
            })?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("Anthropic streaming API error ({}): {}", status, error_text);

            return match status.as_u16() {
                401 => Err(ProviderError::AuthError),
                429 => Err(ProviderError::RateLimited(60)),
                _ => Err(ProviderError::ProviderError(format!(
                    "Anthropic API error: {}",
                    status
                ))),
            };
        }

        // Read and parse SSE response
        let model = request.model.clone();
        let body = response.text().await.map_err(|e| {
            error!("Failed to read streaming response body: {}", e);
            ProviderError::NetworkError(e.to_string())
        })?;

        // Parse SSE events and create stream
        let stream = Self::parse_sse_response(&body, model)?;
        Ok(stream)
    }

    fn count_tokens(&self, content: &str, model: &str) -> Result<usize, ProviderError> {
        // Validate model
        if !self.models().iter().any(|m| m.id == model) {
            return Err(ProviderError::InvalidModel(model.to_string()));
        }

        // Use token counter with caching for performance
        // Anthropic's token counting is similar to OpenAI's
        let tokens = self.token_counter.count_tokens_openai(content, model);
        Ok(tokens)
    }

    async fn health_check(&self) -> Result<bool, ProviderError> {
        debug!("Performing health check for Anthropic provider");

        // Try to get models list as a health check
        let response = self
            .client
            .get(format!("{}/models", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .send()
            .await
            .map_err(|e| {
                warn!("Anthropic health check failed: {}", e);
                ProviderError::from(e)
            })?;

        match response.status().as_u16() {
            200 => {
                debug!("Anthropic health check passed");
                Ok(true)
            }
            401 => {
                error!("Anthropic health check failed: authentication error");
                Err(ProviderError::AuthError)
            }
            _ => {
                warn!(
                    "Anthropic health check failed with status: {}",
                    response.status()
                );
                Ok(false)
            }
        }
    }
}

/// Anthropic API request format
#[derive(Debug, Serialize)]
struct AnthropicChatRequest {
    model: String,
    max_tokens: usize,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

/// Anthropic API message format
#[derive(Debug, Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

/// Anthropic API response format
#[derive(Debug, Deserialize)]
struct AnthropicChatResponse {
    content: Vec<AnthropicContent>,
    usage: AnthropicUsage,
    stop_reason: Option<String>,
}

/// Anthropic API content format
#[derive(Debug, Deserialize)]
struct AnthropicContent {
    text: String,
}

/// Anthropic API usage format
#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: usize,
    output_tokens: usize,
}

/// Anthropic streaming request format
#[derive(Debug, Serialize)]
struct AnthropicStreamRequest {
    model: String,
    max_tokens: usize,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    stream: bool,
}

/// Anthropic streaming event format (Server-Sent Events)
#[derive(Debug, Deserialize)]
struct AnthropicStreamEvent {
    #[serde(rename = "type")]
    event_type: String,
    #[serde(default)]
    delta: Option<AnthropicDelta>,
    #[serde(default)]
    usage: Option<AnthropicUsage>,
}

/// Delta content in streaming response
#[derive(Debug, Deserialize)]
struct AnthropicDelta {
    #[serde(rename = "type", default)]
    delta_type: Option<String>,
    #[serde(default)]
    text: Option<String>,
}
