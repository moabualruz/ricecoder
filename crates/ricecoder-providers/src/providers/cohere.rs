//! Cohere provider implementation
//!
//! Supports Command and Base models via the Cohere API.

use std::sync::Arc;

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use crate::{
    error::ProviderError,
    models::{Capability, ChatRequest, ChatResponse, FinishReason, ModelInfo, Pricing, TokenUsage},
    provider::Provider,
    token_counter::TokenCounter,
};

/// Cohere provider implementation
pub struct CohereProvider {
    api_key: String,
    client: Arc<Client>,
    token_counter: Arc<TokenCounter>,
    models: Vec<ModelInfo>,
}

impl CohereProvider {
    /// Create a new Cohere provider instance
    pub fn new(api_key: String, models: Vec<ModelInfo>) -> Result<Self, ProviderError> {
        Self::with_client(Arc::new(Client::new()), api_key, models)
    }

    /// Create a new Cohere provider with default models from registry
    #[allow(dead_code)]
    pub fn with_default_models(api_key: String) -> Result<Self, ProviderError> {
        use crate::model_registry::global_registry;
        let models = global_registry().get_provider_models("cohere");
        Self::new(api_key, models)
    }

    /// Create a new Cohere provider with custom HTTP client
    pub fn with_client(client: Arc<Client>, api_key: String, models: Vec<ModelInfo>) -> Result<Self, ProviderError> {
        if api_key.is_empty() {
            return Err(ProviderError::ConfigError(
                "Cohere API key is required".to_string(),
            ));
        }

        Ok(Self {
            api_key,
            client,
            token_counter: Arc::new(TokenCounter::new()),
            models,
        })
    }

    /// Get the authorization header value
    fn get_auth_header(&self) -> String {
        format!("Bearer {}", self.api_key)
    }

    /// Convert Cohere API response to our ChatResponse
    fn convert_response(
        response: CohereChatResponse,
        model: String,
    ) -> Result<ChatResponse, ProviderError> {
        let content = response
            .generations
            .first()
            .map(|g| g.text.clone())
            .ok_or_else(|| {
                ProviderError::ProviderError("No content in Cohere response".to_string())
            })?;

        let finish_reason = match response
            .generations
            .first()
            .and_then(|g| g.finish_reason.as_deref())
        {
            Some("COMPLETE") => FinishReason::Stop,
            Some("MAX_TOKENS") => FinishReason::Length,
            _ => FinishReason::Stop,
        };

        // Estimate token usage
        let estimated_tokens = content.len() / 4;

        Ok(ChatResponse {
            content,
            model,
            usage: TokenUsage {
                prompt_tokens: response.meta.billed_units.input_tokens,
                completion_tokens: response.meta.billed_units.output_tokens,
                total_tokens: response.meta.billed_units.input_tokens
                    + response.meta.billed_units.output_tokens,
            },
            finish_reason,
        })
    }
}

#[async_trait]
impl Provider for CohereProvider {
    fn id(&self) -> &str {
        "cohere"
    }

    fn name(&self) -> &str {
        "Cohere"
    }

    fn models(&self) -> Vec<ModelInfo> {
        self.models.clone()
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, ProviderError> {
        debug!("Sending chat request to Cohere: {}", request.model);

        let cohere_request = CohereChatRequest {
            prompt: request
                .messages
                .iter()
                .map(|m| format!("{}: {}", m.role, m.content))
                .collect::<Vec<_>>()
                .join("\n"),
            model: Some(request.model.clone()),
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            num_generations: Some(1),
        };

        let response = self
            .client
            .post("https://api.cohere.ai/v1/generate")
            .header("Authorization", self.get_auth_header())
            .header("Content-Type", "application/json")
            .json(&cohere_request)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to send request to Cohere: {}", e);
                ProviderError::NetworkError(e.to_string())
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Cohere API error: {} - {}", status, error_text);
            return Err(ProviderError::ProviderError(format!(
                "Cohere API error: {} - {}",
                status, error_text
            )));
        }

        let cohere_response: CohereChatResponse = response.json().await.map_err(|e| {
            error!("Failed to parse Cohere response: {}", e);
            ProviderError::ParseError(e.to_string())
        })?;

        Self::convert_response(cohere_response, request.model)
    }

    async fn chat_stream(
        &self,
        _request: ChatRequest,
    ) -> Result<crate::provider::ChatStream, ProviderError> {
        Err(ProviderError::ProviderError(
            "Streaming not yet implemented for Cohere".to_string(),
        ))
    }

    fn count_tokens(&self, content: &str, _model: &str) -> Result<usize, ProviderError> {
        // Cohere uses approximate token counting
        Ok(content.len() / 4)
    }

    async fn health_check(&self) -> Result<bool, ProviderError> {
        Ok(!self.api_key.is_empty())
    }
}

/// Cohere chat request structure
#[derive(Serialize)]
struct CohereChatRequest {
    prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_generations: Option<usize>,
}

/// Cohere chat response structure
#[derive(Deserialize)]
struct CohereChatResponse {
    generations: Vec<CohereGeneration>,
    meta: CohereMeta,
}

/// Cohere generation structure
#[derive(Deserialize)]
struct CohereGeneration {
    text: String,
    #[serde(default)]
    finish_reason: Option<String>,
}

/// Cohere meta structure
#[derive(Deserialize)]
struct CohereMeta {
    billed_units: CohereBilledUnits,
}

/// Cohere billed units structure
#[derive(Deserialize)]
struct CohereBilledUnits {
    input_tokens: usize,
    output_tokens: usize,
}
