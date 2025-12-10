//! OpenAI provider implementation
//!
//! Supports GPT-4, GPT-4o, and GPT-3.5-turbo models via the OpenAI API.

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, warn};

use crate::error::ProviderError;
use crate::models::{Capability, ChatRequest, ChatResponse, FinishReason, ModelInfo, TokenUsage};
use crate::provider::Provider;
use crate::token_counter::TokenCounter;

/// OpenAI provider implementation
pub struct OpenAiProvider {
    api_key: String,
    client: Arc<Client>,
    base_url: String,
    token_counter: Arc<TokenCounter>,
}

impl OpenAiProvider {
    /// Create a new OpenAI provider instance
    pub fn new(api_key: String) -> Result<Self, ProviderError> {
        if api_key.is_empty() {
            return Err(ProviderError::ConfigError(
                "OpenAI API key is required".to_string(),
            ));
        }

        Ok(Self {
            api_key,
            client: Arc::new(Client::new()),
            base_url: "https://api.openai.com/v1".to_string(),
            token_counter: Arc::new(TokenCounter::new()),
        })
    }

    /// Create a new OpenAI provider with a custom base URL
    pub fn with_base_url(api_key: String, base_url: String) -> Result<Self, ProviderError> {
        if api_key.is_empty() {
            return Err(ProviderError::ConfigError(
                "OpenAI API key is required".to_string(),
            ));
        }

        Ok(Self {
            api_key,
            client: Arc::new(Client::new()),
            base_url,
            token_counter: Arc::new(TokenCounter::new()),
        })
    }

    /// Get the authorization header value (redacted for logging)
    fn get_auth_header(&self) -> String {
        format!("Bearer {}", self.api_key)
    }

    /// Convert OpenAI API response to our ChatResponse
    fn convert_response(
        response: OpenAiChatResponse,
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
}

#[async_trait]
impl Provider for OpenAiProvider {
    fn id(&self) -> &str {
        "openai"
    }

    fn name(&self) -> &str {
        "OpenAI"
    }

    fn models(&self) -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                id: "gpt-4".to_string(),
                name: "GPT-4".to_string(),
                provider: "openai".to_string(),
                context_window: 8192,
                capabilities: vec![Capability::Chat, Capability::Code, Capability::Streaming],
                pricing: Some(crate::models::Pricing {
                    input_per_1k_tokens: 0.03,
                    output_per_1k_tokens: 0.06,
                }),
                is_free: false,
            },
            ModelInfo {
                id: "gpt-4-turbo".to_string(),
                name: "GPT-4 Turbo".to_string(),
                provider: "openai".to_string(),
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
            ModelInfo {
                id: "gpt-4o".to_string(),
                name: "GPT-4o".to_string(),
                provider: "openai".to_string(),
                context_window: 128000,
                capabilities: vec![
                    Capability::Chat,
                    Capability::Code,
                    Capability::Vision,
                    Capability::Streaming,
                ],
                pricing: Some(crate::models::Pricing {
                    input_per_1k_tokens: 0.005,
                    output_per_1k_tokens: 0.015,
                }),
                is_free: false,
            },
            ModelInfo {
                id: "gpt-3.5-turbo".to_string(),
                name: "GPT-3.5 Turbo".to_string(),
                provider: "openai".to_string(),
                context_window: 4096,
                capabilities: vec![Capability::Chat, Capability::Code, Capability::Streaming],
                pricing: Some(crate::models::Pricing {
                    input_per_1k_tokens: 0.0005,
                    output_per_1k_tokens: 0.0015,
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

        let openai_request = OpenAiChatRequest {
            model: request.model.clone(),
            messages: request
                .messages
                .iter()
                .map(|m| OpenAiMessage {
                    role: m.role.clone(),
                    content: m.content.clone(),
                })
                .collect(),
            temperature: request.temperature,
            max_tokens: request.max_tokens,
        };

        debug!(
            "Sending chat request to OpenAI for model: {}",
            request.model
        );

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", self.get_auth_header())
            .header("Content-Type", "application/json")
            .json(&openai_request)
            .send()
            .await
            .map_err(|e| {
                error!("OpenAI API request failed: {}", e);
                ProviderError::from(e)
            })?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("OpenAI API error ({}): {}", status, error_text);

            return match status.as_u16() {
                401 => Err(ProviderError::AuthError),
                429 => Err(ProviderError::RateLimited(60)),
                _ => Err(ProviderError::ProviderError(format!(
                    "OpenAI API error: {}",
                    status
                ))),
            };
        }

        let openai_response: OpenAiChatResponse = response.json().await?;
        Self::convert_response(openai_response, request.model)
    }

    async fn chat_stream(
        &self,
        _request: ChatRequest,
    ) -> Result<crate::provider::ChatStream, ProviderError> {
        // Streaming support will be implemented in a future iteration
        Err(ProviderError::ProviderError(
            "Streaming not yet implemented for OpenAI".to_string(),
        ))
    }

    fn count_tokens(&self, content: &str, model: &str) -> Result<usize, ProviderError> {
        // Validate model
        if !self.models().iter().any(|m| m.id == model) {
            return Err(ProviderError::InvalidModel(model.to_string()));
        }

        // Use token counter with caching for performance
        let tokens = self.token_counter.count_tokens_openai(content, model);
        Ok(tokens)
    }

    async fn health_check(&self) -> Result<bool, ProviderError> {
        debug!("Performing health check for OpenAI provider");

        // Try to list models as a health check
        let response = self
            .client
            .get(format!("{}/models", self.base_url))
            .header("Authorization", self.get_auth_header())
            .send()
            .await
            .map_err(|e| {
                warn!("OpenAI health check failed: {}", e);
                ProviderError::from(e)
            })?;

        match response.status().as_u16() {
            200 => {
                debug!("OpenAI health check passed");
                Ok(true)
            }
            401 => {
                error!("OpenAI health check failed: authentication error");
                Err(ProviderError::AuthError)
            }
            _ => {
                warn!(
                    "OpenAI health check failed with status: {}",
                    response.status()
                );
                Ok(false)
            }
        }
    }
}

/// OpenAI API request format
#[derive(Debug, Serialize)]
struct OpenAiChatRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<usize>,
}

/// OpenAI API message format
#[derive(Debug, Serialize, Deserialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

/// OpenAI API response format
#[derive(Debug, Deserialize)]
struct OpenAiChatResponse {
    choices: Vec<OpenAiChoice>,
    usage: OpenAiUsage,
}

/// OpenAI API choice format
#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    message: Option<OpenAiMessage>,
    finish_reason: Option<String>,
}

/// OpenAI API usage format
#[derive(Debug, Deserialize)]
struct OpenAiUsage {
    prompt_tokens: usize,
    completion_tokens: usize,
    total_tokens: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_provider_creation() {
        let provider = OpenAiProvider::new("test-key".to_string());
        assert!(provider.is_ok());
    }

    #[test]
    fn test_openai_provider_creation_empty_key() {
        let provider = OpenAiProvider::new("".to_string());
        assert!(provider.is_err());
    }

    #[test]
    fn test_openai_provider_id() {
        let provider = OpenAiProvider::new("test-key".to_string()).unwrap();
        assert_eq!(provider.id(), "openai");
    }

    #[test]
    fn test_openai_provider_name() {
        let provider = OpenAiProvider::new("test-key".to_string()).unwrap();
        assert_eq!(provider.name(), "OpenAI");
    }

    #[test]
    fn test_openai_models() {
        let provider = OpenAiProvider::new("test-key".to_string()).unwrap();
        let models = provider.models();
        assert_eq!(models.len(), 4);
        assert!(models.iter().any(|m| m.id == "gpt-4"));
        assert!(models.iter().any(|m| m.id == "gpt-4-turbo"));
        assert!(models.iter().any(|m| m.id == "gpt-4o"));
        assert!(models.iter().any(|m| m.id == "gpt-3.5-turbo"));
    }

    #[test]
    fn test_token_counting() {
        let provider = OpenAiProvider::new("test-key".to_string()).unwrap();
        let tokens = provider.count_tokens("Hello, world!", "gpt-4").unwrap();
        assert!(tokens > 0);
    }

    #[test]
    fn test_token_counting_invalid_model() {
        let provider = OpenAiProvider::new("test-key".to_string()).unwrap();
        let result = provider.count_tokens("Hello, world!", "invalid-model");
        assert!(result.is_err());
    }
}
