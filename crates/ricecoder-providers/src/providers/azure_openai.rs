//! Azure OpenAI provider implementation
//!
//! Supports Azure OpenAI service models via the Azure OpenAI API.

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, warn};

use crate::error::ProviderError;
use crate::token_counter::TokenCounterTrait;
use crate::models::{Capability, ChatRequest, ChatResponse, FinishReason, ModelInfo, Pricing, TokenUsage};
use crate::provider::Provider;
use crate::token_counter::TokenCounter;

/// Azure OpenAI provider implementation
pub struct AzureOpenAiProvider {
    api_key: String,
    client: Arc<Client>,
    base_url: String,
    api_version: String,
    deployment_name: String,
    token_counter: Arc<TokenCounter>,
}

impl AzureOpenAiProvider {
    /// Create a new Azure OpenAI provider instance
    pub fn new(
        api_key: String,
        base_url: String,
        deployment_name: String,
        api_version: String,
    ) -> Result<Self, ProviderError> {
        if api_key.is_empty() {
            return Err(ProviderError::ConfigError(
                "Azure OpenAI API key is required".to_string(),
            ));
        }

        if base_url.is_empty() {
            return Err(ProviderError::ConfigError(
                "Azure OpenAI base URL is required".to_string(),
            ));
        }

        if deployment_name.is_empty() {
            return Err(ProviderError::ConfigError(
                "Azure OpenAI deployment name is required".to_string(),
            ));
        }

        Ok(Self {
            api_key,
            client: Arc::new(Client::new()),
            base_url,
            api_version,
            deployment_name,
            token_counter: Arc::new(TokenCounter::new()),
        })
    }

    /// Get the authorization header value (redacted for logging)
    fn get_auth_header(&self) -> String {
        format!("Bearer {}", self.api_key)
    }

    /// Build the chat completions URL
    fn chat_url(&self) -> String {
        format!(
            "{}/openai/deployments/{}/chat/completions?api-version={}",
            self.base_url.trim_end_matches('/'),
            self.deployment_name,
            self.api_version
        )
    }

    /// Convert Azure OpenAI API response to our ChatResponse
    fn convert_response(
        response: AzureOpenAiChatResponse,
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
            Some("content_filter") => FinishReason::Error,
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
impl Provider for AzureOpenAiProvider {
    fn id(&self) -> &str {
        "azure-openai"
    }

    fn name(&self) -> &str {
        "Azure OpenAI"
    }

    fn models(&self) -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                id: format!("{}-gpt-4", self.deployment_name),
                name: "GPT-4".to_string(),
                provider: self.name().to_string(),
                context_window: 8192,
                capabilities: vec![Capability::Chat, Capability::FunctionCalling],
                pricing: Some(Pricing {
                    input_per_1k_tokens: 0.03,
                    output_per_1k_tokens: 0.06,
                }),
                is_free: false,
            },
            ModelInfo {
                id: format!("{}-gpt-4-turbo", self.deployment_name),
                name: "GPT-4 Turbo".to_string(),
                provider: self.name().to_string(),
                context_window: 128000,
                capabilities: vec![Capability::Chat, Capability::FunctionCalling, Capability::Vision],
                pricing: Some(Pricing {
                    input_per_1k_tokens: 0.01,
                    output_per_1k_tokens: 0.03,
                }),
                is_free: false,
            },
            ModelInfo {
                id: format!("{}-gpt-35-turbo", self.deployment_name),
                name: "GPT-3.5 Turbo".to_string(),
                provider: self.name().to_string(),
                context_window: 16384,
                capabilities: vec![Capability::Chat, Capability::FunctionCalling],
                pricing: Some(Pricing {
                    input_per_1k_tokens: 0.0015,
                    output_per_1k_tokens: 0.002,
                }),
                is_free: false,
            },
        ]
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, ProviderError> {
        debug!("Sending chat request to Azure OpenAI: {}", request.model);

        let openai_request = AzureOpenAiChatRequest {
            messages: request
                .messages
                .iter()
                .map(|m| AzureOpenAiMessage {
                    role: match m.role.as_str() {
                        "user" => "user".to_string(),
                        "assistant" => "assistant".to_string(),
                        "system" => "system".to_string(),
                        _ => "user".to_string(),
                    },
                    content: m.content.clone(),
                })
                .collect(),
            model: request.model.clone(),
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            stream: false,
        };

        let response = self
            .client
            .post(&self.chat_url())
            .header("api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&openai_request)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to send request to Azure OpenAI: {}", e);
                ProviderError::NetworkError(e.to_string())
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Azure OpenAI API error: {} - {}", status, error_text);
            return Err(ProviderError::ProviderError(format!(
                "Azure OpenAI API error: {} - {}",
                status, error_text
            )));
        }

        let openai_response: AzureOpenAiChatResponse = response.json().await.map_err(|e| {
            error!("Failed to parse Azure OpenAI response: {}", e);
            ProviderError::ParseError(e.to_string())
        })?;

        Self::convert_response(openai_response, request.model)
    }

    async fn chat_stream(&self, _request: ChatRequest) -> Result<crate::provider::ChatStream, ProviderError> {
        // Streaming implementation would go here
        // For now, return an error indicating streaming is not implemented
        Err(ProviderError::ProviderError("Streaming not yet implemented for Azure OpenAI".to_string()))
    }

    fn count_tokens(&self, content: &str, model: &str) -> Result<usize, ProviderError> {
        Ok(self.token_counter.count_tokens_openai(content, model))
    }

    async fn health_check(&self) -> Result<bool, ProviderError> {
        // Simple health check - try to get models or make a minimal request
        // For Azure OpenAI, we could try a minimal chat request or check deployment status
        // For now, just return true if we have valid configuration
        Ok(!self.api_key.is_empty() && !self.base_url.is_empty() && !self.deployment_name.is_empty())
    }
}

/// Azure OpenAI chat request structure
#[derive(Serialize)]
struct AzureOpenAiChatRequest {
    messages: Vec<AzureOpenAiMessage>,
    model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    stream: bool,
}

/// Azure OpenAI message structure
#[derive(Serialize, Deserialize)]
struct AzureOpenAiMessage {
    role: String,
    content: String,
}

/// Azure OpenAI chat response structure
#[derive(Deserialize)]
struct AzureOpenAiChatResponse {
    choices: Vec<AzureOpenAiChoice>,
    usage: AzureOpenAiUsage,
}

/// Azure OpenAI choice structure
#[derive(Deserialize)]
struct AzureOpenAiChoice {
    message: Option<AzureOpenAiMessage>,
    finish_reason: Option<String>,
}

/// Azure OpenAI usage structure
#[derive(Deserialize)]
struct AzureOpenAiUsage {
    prompt_tokens: usize,
    completion_tokens: usize,
    total_tokens: usize,
}