//! Anthropic provider implementation
//!
//! Supports Claude 3, 3.5, and 4 family models via the Anthropic API.

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, warn};

use crate::error::ProviderError;
use crate::models::{Capability, ChatRequest, ChatResponse, FinishReason, ModelInfo, TokenUsage};
use crate::provider::Provider;
use crate::token_counter::TokenCounter;

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
        if api_key.is_empty() {
            return Err(ProviderError::ConfigError(
                "Anthropic API key is required".to_string(),
            ));
        }

        Ok(Self {
            api_key,
            client: Arc::new(Client::new()),
            base_url: "https://api.anthropic.com/v1".to_string(),
            token_counter: Arc::new(TokenCounter::new()),
        })
    }

    /// Create a new Anthropic provider with a custom base URL
    pub fn with_base_url(api_key: String, base_url: String) -> Result<Self, ProviderError> {
        if api_key.is_empty() {
            return Err(ProviderError::ConfigError(
                "Anthropic API key is required".to_string(),
            ));
        }

        Ok(Self {
            api_key,
            client: Arc::new(Client::new()),
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

        debug!("Sending chat request to Anthropic for model: {}", request.model);

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
        _request: ChatRequest,
    ) -> Result<crate::provider::ChatStream, ProviderError> {
        // Streaming support will be implemented in a future iteration
        Err(ProviderError::ProviderError(
            "Streaming not yet implemented for Anthropic".to_string(),
        ))
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
                warn!("Anthropic health check failed with status: {}", response.status());
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anthropic_provider_creation() {
        let provider = AnthropicProvider::new("test-key".to_string());
        assert!(provider.is_ok());
    }

    #[test]
    fn test_anthropic_provider_creation_empty_key() {
        let provider = AnthropicProvider::new("".to_string());
        assert!(provider.is_err());
    }

    #[test]
    fn test_anthropic_provider_id() {
        let provider = AnthropicProvider::new("test-key".to_string()).unwrap();
        assert_eq!(provider.id(), "anthropic");
    }

    #[test]
    fn test_anthropic_provider_name() {
        let provider = AnthropicProvider::new("test-key".to_string()).unwrap();
        assert_eq!(provider.name(), "Anthropic");
    }

    #[test]
    fn test_anthropic_models() {
        let provider = AnthropicProvider::new("test-key".to_string()).unwrap();
        let models = provider.models();
        assert_eq!(models.len(), 4);
        assert!(models.iter().any(|m| m.id == "claude-3-opus-20250219"));
        assert!(models.iter().any(|m| m.id == "claude-3-5-sonnet-20241022"));
        assert!(models.iter().any(|m| m.id == "claude-3-5-haiku-20241022"));
        assert!(models.iter().any(|m| m.id == "claude-3-haiku-20240307"));
    }

    #[test]
    fn test_token_counting() {
        let provider = AnthropicProvider::new("test-key".to_string()).unwrap();
        let tokens = provider.count_tokens("Hello, world!", "claude-3-opus-20250219").unwrap();
        assert!(tokens > 0);
    }

    #[test]
    fn test_token_counting_invalid_model() {
        let provider = AnthropicProvider::new("test-key".to_string()).unwrap();
        let result = provider.count_tokens("Hello, world!", "invalid-model");
        assert!(result.is_err());
    }
}
