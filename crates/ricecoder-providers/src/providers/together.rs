//! Together AI provider implementation
//!
//! Supports various open-source models via the Together AI API.

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error};

use crate::error::ProviderError;
use crate::models::{
    Capability, ChatRequest, ChatResponse, FinishReason, ModelInfo, Pricing, TokenUsage,
};
use crate::provider::Provider;
use crate::token_counter::TokenCounter;
use crate::token_counter::TokenCounterTrait;

/// Together AI provider implementation
pub struct TogetherProvider {
    api_key: String,
    client: Arc<Client>,
    token_counter: Arc<TokenCounter>,
}

impl TogetherProvider {
    /// Create a new Together AI provider instance
    pub fn new(api_key: String) -> Result<Self, ProviderError> {
        if api_key.is_empty() {
            return Err(ProviderError::ConfigError(
                "Together AI API key is required".to_string(),
            ));
        }

        Ok(Self {
            api_key,
            client: Arc::new(Client::new()),
            token_counter: Arc::new(TokenCounter::new()),
        })
    }

    /// Get the authorization header value
    fn get_auth_header(&self) -> String {
        format!("Bearer {}", self.api_key)
    }

    /// Convert Together AI response to our ChatResponse
    fn convert_response(
        response: TogetherChatResponse,
        model: String,
    ) -> Result<ChatResponse, ProviderError> {
        let content = response
            .choices
            .first()
            .and_then(|c| c.text.as_ref())
            .ok_or_else(|| {
                ProviderError::ProviderError("No content in Together AI response".to_string())
            })?
            .clone();

        let finish_reason = match response
            .choices
            .first()
            .and_then(|c| c.finish_reason.as_deref())
        {
            Some("stop") => FinishReason::Stop,
            Some("length") => FinishReason::Length,
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
impl Provider for TogetherProvider {
    fn id(&self) -> &str {
        "together"
    }

    fn name(&self) -> &str {
        "Together AI"
    }

    fn models(&self) -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                id: "meta-llama/Llama-2-70b-chat-hf".to_string(),
                name: "Llama 2 70B Chat".to_string(),
                provider: self.name().to_string(),
                context_window: 4096,
                capabilities: vec![Capability::Chat],
                pricing: Some(Pricing {
                    input_per_1k_tokens: 0.0009,
                    output_per_1k_tokens: 0.0009,
                }),
                is_free: false,
            },
            ModelInfo {
                id: "mistralai/Mistral-7B-Instruct-v0.1".to_string(),
                name: "Mistral 7B Instruct".to_string(),
                provider: self.name().to_string(),
                context_window: 8192,
                capabilities: vec![Capability::Chat],
                pricing: Some(Pricing {
                    input_per_1k_tokens: 0.0002,
                    output_per_1k_tokens: 0.0002,
                }),
                is_free: false,
            },
            ModelInfo {
                id: "codellama/CodeLlama-34b-Instruct-hf".to_string(),
                name: "CodeLlama 34B Instruct".to_string(),
                provider: self.name().to_string(),
                context_window: 16384,
                capabilities: vec![Capability::Chat],
                pricing: Some(Pricing {
                    input_per_1k_tokens: 0.0008,
                    output_per_1k_tokens: 0.0008,
                }),
                is_free: false,
            },
            ModelInfo {
                id: "NousResearch/Nous-Hermes-2-Mixtral-8x7B-DPO".to_string(),
                name: "Nous Hermes 2 Mixtral 8x7B".to_string(),
                provider: self.name().to_string(),
                context_window: 32768,
                capabilities: vec![Capability::Chat],
                pricing: Some(Pricing {
                    input_per_1k_tokens: 0.0006,
                    output_per_1k_tokens: 0.0006,
                }),
                is_free: false,
            },
        ]
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, ProviderError> {
        debug!("Sending chat request to Together AI: {}", request.model);

        let together_request = TogetherChatRequest {
            prompt: request
                .messages
                .iter()
                .map(|m| format!("{}: {}", m.role, m.content))
                .collect::<Vec<_>>()
                .join("\n"),
            model: request.model.clone(),
            max_tokens: request.max_tokens.unwrap_or(512),
            temperature: request.temperature.unwrap_or(0.7),
            stop: None,
        };

        let response = self
            .client
            .post("https://api.together.xyz/inference")
            .header("Authorization", self.get_auth_header())
            .header("Content-Type", "application/json")
            .json(&together_request)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to send request to Together AI: {}", e);
                ProviderError::NetworkError(e.to_string())
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Together AI API error: {} - {}", status, error_text);
            return Err(ProviderError::ProviderError(format!(
                "Together AI API error: {} - {}",
                status, error_text
            )));
        }

        let together_response: TogetherChatResponse = response.json().await.map_err(|e| {
            error!("Failed to parse Together AI response: {}", e);
            ProviderError::ParseError(e.to_string())
        })?;

        Self::convert_response(together_response, request.model)
    }

    async fn chat_stream(
        &self,
        _request: ChatRequest,
    ) -> Result<crate::provider::ChatStream, ProviderError> {
        Err(ProviderError::ProviderError(
            "Streaming not yet implemented for Together AI".to_string(),
        ))
    }

    fn count_tokens(&self, content: &str, model: &str) -> Result<usize, ProviderError> {
        Ok(self.token_counter.count_tokens_openai(content, model))
    }

    async fn health_check(&self) -> Result<bool, ProviderError> {
        Ok(!self.api_key.is_empty())
    }
}

/// Together AI chat request structure
#[derive(Serialize)]
struct TogetherChatRequest {
    prompt: String,
    model: String,
    max_tokens: usize,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
}

/// Together AI chat response structure
#[derive(Deserialize)]
struct TogetherChatResponse {
    choices: Vec<TogetherChoice>,
    usage: TogetherUsage,
}

/// Together AI choice structure
#[derive(Deserialize)]
struct TogetherChoice {
    text: Option<String>,
    finish_reason: Option<String>,
}

/// Together AI usage structure
#[derive(Deserialize)]
struct TogetherUsage {
    prompt_tokens: usize,
    completion_tokens: usize,
    total_tokens: usize,
}
