//! Google Gemini provider implementation
//!
//! Supports Gemini models via the Google AI API.

use std::sync::Arc;

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, warn};

use crate::{
    error::ProviderError,
    models::{Capability, ChatRequest, ChatResponse, FinishReason, ModelInfo, TokenUsage},
    provider::Provider,
    token_counter::TokenCounter,
};

/// Google Gemini provider implementation
pub struct GoogleProvider {
    api_key: String,
    client: Arc<Client>,
    base_url: String,
    token_counter: Arc<TokenCounter>,
}

impl GoogleProvider {
    /// Create a new Google provider instance
    pub fn new(api_key: String) -> Result<Self, ProviderError> {
        Self::with_client(Arc::new(Client::new()), api_key)
    }

    /// Create a new Google provider with a custom base URL
    pub fn with_base_url(api_key: String, base_url: String) -> Result<Self, ProviderError> {
        Self::with_client_and_base_url(Arc::new(Client::new()), api_key, base_url)
    }

    /// Create a new Google provider with a custom HTTP client
    pub fn with_client(client: Arc<Client>, api_key: String) -> Result<Self, ProviderError> {
        if api_key.is_empty() {
            return Err(ProviderError::ConfigError(
                "Google API key is required".to_string(),
            ));
        }

        Ok(Self {
            api_key,
            client,
            base_url: "https://generativelanguage.googleapis.com/v1beta/models".to_string(),
            token_counter: Arc::new(TokenCounter::new()),
        })
    }

    /// Create a new Google provider with a custom HTTP client and base URL
    pub fn with_client_and_base_url(
        client: Arc<Client>,
        api_key: String,
        base_url: String,
    ) -> Result<Self, ProviderError> {
        if api_key.is_empty() {
            return Err(ProviderError::ConfigError(
                "Google API key is required".to_string(),
            ));
        }

        Ok(Self {
            api_key,
            client,
            base_url,
            token_counter: Arc::new(TokenCounter::new()),
        })
    }

    /// Convert Google API response to our ChatResponse
    fn convert_response(
        response: GoogleChatResponse,
        model: String,
    ) -> Result<ChatResponse, ProviderError> {
        let content = response
            .candidates
            .first()
            .and_then(|c| c.content.as_ref())
            .and_then(|c| c.parts.first())
            .map(|p| p.text.clone())
            .ok_or_else(|| ProviderError::ProviderError("No content in response".to_string()))?;

        let finish_reason = response
            .candidates
            .first()
            .and_then(|c| c.finish_reason.as_deref())
            .map(|reason| match reason {
                "STOP" => FinishReason::Stop,
                "MAX_TOKENS" => FinishReason::Length,
                "ERROR" => FinishReason::Error,
                _ => FinishReason::Stop,
            })
            .unwrap_or(FinishReason::Stop);

        // Google API doesn't always return usage info, so we estimate
        let total_tokens = response
            .usage_metadata
            .as_ref()
            .map(|u| u.total_token_count)
            .unwrap_or(0);

        let prompt_tokens = response
            .usage_metadata
            .as_ref()
            .map(|u| u.prompt_token_count)
            .unwrap_or(0);

        let completion_tokens = response
            .usage_metadata
            .as_ref()
            .map(|u| u.candidates_token_count)
            .unwrap_or(0);

        Ok(ChatResponse {
            content,
            model,
            usage: TokenUsage {
                prompt_tokens,
                completion_tokens,
                total_tokens,
            },
            finish_reason,
        })
    }
}

#[async_trait]
impl Provider for GoogleProvider {
    fn id(&self) -> &str {
        "google"
    }

    fn name(&self) -> &str {
        "Google"
    }

    fn models(&self) -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                id: "gemini-2.0-flash".to_string(),
                name: "Gemini 2.0 Flash".to_string(),
                provider: "google".to_string(),
                context_window: 1000000,
                capabilities: vec![
                    Capability::Chat,
                    Capability::Code,
                    Capability::Vision,
                    Capability::Streaming,
                ],
                pricing: Some(crate::models::Pricing {
                    input_per_1k_tokens: 0.075,
                    output_per_1k_tokens: 0.3,
                }),
                is_free: false,
            },
            ModelInfo {
                id: "gemini-1.5-pro".to_string(),
                name: "Gemini 1.5 Pro".to_string(),
                provider: "google".to_string(),
                context_window: 2000000,
                capabilities: vec![
                    Capability::Chat,
                    Capability::Code,
                    Capability::Vision,
                    Capability::Streaming,
                ],
                pricing: Some(crate::models::Pricing {
                    input_per_1k_tokens: 1.25,
                    output_per_1k_tokens: 5.0,
                }),
                is_free: false,
            },
            ModelInfo {
                id: "gemini-1.5-flash".to_string(),
                name: "Gemini 1.5 Flash".to_string(),
                provider: "google".to_string(),
                context_window: 1000000,
                capabilities: vec![
                    Capability::Chat,
                    Capability::Code,
                    Capability::Vision,
                    Capability::Streaming,
                ],
                pricing: Some(crate::models::Pricing {
                    input_per_1k_tokens: 0.075,
                    output_per_1k_tokens: 0.3,
                }),
                is_free: false,
            },
            ModelInfo {
                id: "gemini-1.0-pro".to_string(),
                name: "Gemini 1.0 Pro".to_string(),
                provider: "google".to_string(),
                context_window: 32000,
                capabilities: vec![Capability::Chat, Capability::Code, Capability::Streaming],
                pricing: Some(crate::models::Pricing {
                    input_per_1k_tokens: 0.5,
                    output_per_1k_tokens: 1.5,
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

        let google_request = GoogleChatRequest {
            contents: vec![GoogleContent {
                role: "user".to_string(),
                parts: request
                    .messages
                    .iter()
                    .map(|m| GooglePart {
                        text: m.content.clone(),
                    })
                    .collect(),
            }],
            generation_config: Some(GoogleGenerationConfig {
                temperature: request.temperature,
                max_output_tokens: request.max_tokens,
            }),
        };

        debug!(
            "Sending chat request to Google for model: {}",
            request.model
        );

        let url = format!("{}:generateContent?key={}", self.base_url, self.api_key);

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&google_request)
            .send()
            .await
            .map_err(|e| {
                error!("Google API request failed: {}", e);
                ProviderError::from(e)
            })?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("Google API error ({}): {}", status, error_text);

            return match status.as_u16() {
                401 | 403 => Err(ProviderError::AuthError),
                429 => Err(ProviderError::RateLimited(60)),
                _ => Err(ProviderError::ProviderError(format!(
                    "Google API error: {}",
                    status
                ))),
            };
        }

        let google_response: GoogleChatResponse = response.json().await?;
        Self::convert_response(google_response, request.model)
    }

    async fn chat_stream(
        &self,
        _request: ChatRequest,
    ) -> Result<crate::provider::ChatStream, ProviderError> {
        // Streaming support will be implemented in a future iteration
        Err(ProviderError::ProviderError(
            "Streaming not yet implemented for Google".to_string(),
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
        debug!("Performing health check for Google provider");

        // Try to list models as a health check
        let url = format!("{}?key={}", self.base_url, self.api_key);

        let response = self.client.get(&url).send().await.map_err(|e| {
            warn!("Google health check failed: {}", e);
            ProviderError::from(e)
        })?;

        match response.status().as_u16() {
            200 => {
                debug!("Google health check passed");
                Ok(true)
            }
            401 | 403 => {
                error!("Google health check failed: authentication error");
                Err(ProviderError::AuthError)
            }
            _ => {
                warn!(
                    "Google health check failed with status: {}",
                    response.status()
                );
                Ok(false)
            }
        }
    }
}

/// Google API request format
#[derive(Debug, Serialize)]
struct GoogleChatRequest {
    contents: Vec<GoogleContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GoogleGenerationConfig>,
}

/// Google API content format
#[derive(Debug, Serialize, Deserialize)]
struct GoogleContent {
    role: String,
    parts: Vec<GooglePart>,
}

/// Google API part format
#[derive(Debug, Serialize, Deserialize)]
struct GooglePart {
    text: String,
}

/// Google API generation config
#[derive(Debug, Serialize)]
struct GoogleGenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<usize>,
}

/// Google API response format
#[derive(Debug, Deserialize)]
struct GoogleChatResponse {
    candidates: Vec<GoogleCandidate>,
    #[serde(default)]
    usage_metadata: Option<GoogleUsageMetadata>,
}

/// Google API candidate format
#[derive(Debug, Deserialize)]
struct GoogleCandidate {
    content: Option<GoogleContent>,
    finish_reason: Option<String>,
}

/// Google API usage metadata
#[derive(Debug, Deserialize)]
struct GoogleUsageMetadata {
    prompt_token_count: usize,
    candidates_token_count: usize,
    total_token_count: usize,
}
