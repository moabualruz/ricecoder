//! Replicate provider implementation
//!
//! Supports various models via the Replicate API.

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

/// Replicate provider implementation
pub struct ReplicateProvider {
    api_key: String,
    client: Arc<Client>,
    token_counter: Arc<TokenCounter>,
}

impl ReplicateProvider {
    /// Create a new Replicate provider instance
    pub fn new(api_key: String) -> Result<Self, ProviderError> {
        if api_key.is_empty() {
            return Err(ProviderError::ConfigError(
                "Replicate API key is required".to_string(),
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
        format!("Token {}", self.api_key)
    }

    /// Convert Replicate response to our ChatResponse
    fn convert_response(
        response: ReplicateChatResponse,
        model: String,
    ) -> Result<ChatResponse, ProviderError> {
        let content = response
            .output
            .as_ref()
            .and_then(|output| output.first())
            .and_then(|s| Some(s.as_str()))
            .ok_or_else(|| {
                ProviderError::ProviderError("No content in Replicate response".to_string())
            })?
            .to_string();

        // Estimate token usage
        let estimated_tokens = content.len() / 4;

        Ok(ChatResponse {
            content,
            model,
            usage: TokenUsage {
                prompt_tokens: 0, // Not provided
                completion_tokens: estimated_tokens,
                total_tokens: estimated_tokens,
            },
            finish_reason: FinishReason::Stop,
        })
    }
}

#[async_trait]
impl Provider for ReplicateProvider {
    fn id(&self) -> &str {
        "replicate"
    }

    fn name(&self) -> &str {
        "Replicate"
    }

    fn models(&self) -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                id: "meta/llama-2-70b-chat".to_string(),
                name: "Llama 2 70B Chat".to_string(),
                provider: self.name().to_string(),
                context_window: 4096,
                capabilities: vec![Capability::Chat],
                pricing: Some(Pricing {
                    input_per_1k_tokens: 0.0004,
                    output_per_1k_tokens: 0.0004,
                }),
                is_free: false,
            },
            ModelInfo {
                id: "mistralai/mistral-7b-instruct-v0.1".to_string(),
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
                id: "stability-ai/stable-diffusion".to_string(),
                name: "Stable Diffusion".to_string(),
                provider: self.name().to_string(),
                context_window: 77, // Token limit for images
                capabilities: vec![Capability::Vision],
                pricing: Some(Pricing {
                    input_per_1k_tokens: 0.0,
                    output_per_1k_tokens: 0.002,
                }),
                is_free: false,
            },
        ]
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, ProviderError> {
        debug!("Sending chat request to Replicate: {}", request.model);

        let replicate_request = ReplicateChatRequest {
            input: ReplicateInput {
                prompt: request
                    .messages
                    .iter()
                    .map(|m| format!("{}: {}", m.role, m.content))
                    .collect::<Vec<_>>()
                    .join("\n"),
                max_new_tokens: request.max_tokens.unwrap_or(512),
                temperature: request.temperature.unwrap_or(0.7),
            },
        };

        let url = format!(
            "https://api.replicate.com/v1/models/{}/predictions",
            request.model
        );

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.get_auth_header())
            .header("Content-Type", "application/json")
            .json(&replicate_request)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to send request to Replicate: {}", e);
                ProviderError::NetworkError(e.to_string())
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Replicate API error: {} - {}", status, error_text);
            return Err(ProviderError::ProviderError(format!(
                "Replicate API error: {} - {}",
                status, error_text
            )));
        }

        let replicate_response: ReplicateChatResponse = response.json().await.map_err(|e| {
            error!("Failed to parse Replicate response: {}", e);
            ProviderError::ParseError(e.to_string())
        })?;

        Self::convert_response(replicate_response, request.model)
    }

    async fn chat_stream(
        &self,
        _request: ChatRequest,
    ) -> Result<crate::provider::ChatStream, ProviderError> {
        Err(ProviderError::ProviderError(
            "Streaming not yet implemented for Replicate".to_string(),
        ))
    }

    fn count_tokens(&self, content: &str, _model: &str) -> Result<usize, ProviderError> {
        Ok(content.len() / 4)
    }

    async fn health_check(&self) -> Result<bool, ProviderError> {
        Ok(!self.api_key.is_empty())
    }
}

/// Replicate chat request structure
#[derive(Serialize)]
struct ReplicateChatRequest {
    input: ReplicateInput,
}

/// Replicate input structure
#[derive(Serialize)]
struct ReplicateInput {
    prompt: String,
    max_new_tokens: usize,
    temperature: f32,
}

/// Replicate chat response structure
#[derive(Deserialize)]
struct ReplicateChatResponse {
    output: Option<Vec<String>>,
}
