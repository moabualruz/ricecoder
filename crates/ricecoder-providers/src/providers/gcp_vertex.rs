//! Google Cloud Vertex AI provider implementation
//!
//! Supports PaLM and Gemini models via the Google Cloud Vertex AI API.

use std::sync::Arc;

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, warn};

use crate::{
    error::ProviderError,
    models::{Capability, ChatRequest, ChatResponse, FinishReason, ModelInfo, Pricing, TokenUsage},
    provider::Provider,
    token_counter::{TokenCounter, TokenCounterTrait},
};

/// GCP Vertex AI provider implementation
pub struct GcpVertexProvider {
    project_id: String,
    location: String,
    access_token: String,
    client: Arc<Client>,
    token_counter: Arc<TokenCounter>,
}

impl GcpVertexProvider {
    /// Create a new GCP Vertex AI provider instance
    pub fn new(
        project_id: String,
        location: String,
        access_token: String,
    ) -> Result<Self, ProviderError> {
        Self::with_client(
            Arc::new(Client::new()),
            project_id,
            location,
            access_token,
        )
    }

    /// Create a new GCP Vertex AI provider with a custom HTTP client
    pub fn with_client(
        client: Arc<Client>,
        project_id: String,
        location: String,
        access_token: String,
    ) -> Result<Self, ProviderError> {
        if project_id.is_empty() {
            return Err(ProviderError::ConfigError(
                "GCP project ID is required".to_string(),
            ));
        }

        if location.is_empty() {
            return Err(ProviderError::ConfigError(
                "GCP location is required".to_string(),
            ));
        }

        if access_token.is_empty() {
            return Err(ProviderError::ConfigError(
                "GCP access token is required".to_string(),
            ));
        }

        Ok(Self {
            project_id,
            location,
            access_token,
            client,
            token_counter: Arc::new(TokenCounter::new()),
        })
    }

    /// Get the authorization header value
    fn get_auth_header(&self) -> String {
        format!("Bearer {}", self.access_token)
    }

    /// Build the chat completions URL for Gemini
    fn gemini_chat_url(&self, model: &str) -> String {
        format!(
            "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/{}:generateContent",
            self.location, self.project_id, self.location, model
        )
    }

    /// Convert Vertex AI Gemini response to our ChatResponse
    fn convert_gemini_response(
        response: GeminiGenerateResponse,
        model: String,
    ) -> Result<ChatResponse, ProviderError> {
        let content = response
            .candidates
            .first()
            .and_then(|c| c.content.as_ref())
            .and_then(|content| content.parts.first())
            .map(|part| part.text.clone())
            .ok_or_else(|| {
                ProviderError::ProviderError("No content in Gemini response".to_string())
            })?;

        let finish_reason = match response
            .candidates
            .first()
            .and_then(|c| c.finish_reason.as_ref())
        {
            Some(reason) if reason == "STOP" => FinishReason::Stop,
            Some(reason) if reason == "MAX_TOKENS" => FinishReason::Length,
            Some(reason) if reason == "SAFETY" => FinishReason::Error,
            _ => FinishReason::Stop,
        };

        // Estimate token usage (Vertex AI doesn't provide exact counts in the same way)
        let estimated_tokens = content.len() / 4; // Rough estimation

        Ok(ChatResponse {
            content,
            model,
            usage: TokenUsage {
                prompt_tokens: 0, // Not provided by Vertex AI
                completion_tokens: estimated_tokens,
                total_tokens: estimated_tokens,
            },
            finish_reason,
        })
    }
}

#[async_trait]
impl Provider for GcpVertexProvider {
    fn id(&self) -> &str {
        "gcp-vertex"
    }

    fn name(&self) -> &str {
        "Google Cloud Vertex AI"
    }

    fn models(&self) -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                id: "gemini-1.5-pro".to_string(),
                name: "Gemini 1.5 Pro".to_string(),
                provider: self.name().to_string(),
                context_window: 1048576, // 1M tokens
                capabilities: vec![
                    Capability::Chat,
                    Capability::FunctionCalling,
                    Capability::Vision,
                ],
                pricing: Some(Pricing {
                    input_per_1k_tokens: 0.00125,
                    output_per_1k_tokens: 0.005,
                }),
                is_free: false,
            },
            ModelInfo {
                id: "gemini-1.5-flash".to_string(),
                name: "Gemini 1.5 Flash".to_string(),
                provider: self.name().to_string(),
                context_window: 1048576, // 1M tokens
                capabilities: vec![
                    Capability::Chat,
                    Capability::FunctionCalling,
                    Capability::Vision,
                ],
                pricing: Some(Pricing {
                    input_per_1k_tokens: 0.000075,
                    output_per_1k_tokens: 0.0003,
                }),
                is_free: false,
            },
            ModelInfo {
                id: "gemini-1.0-pro".to_string(),
                name: "Gemini 1.0 Pro".to_string(),
                provider: self.name().to_string(),
                context_window: 32768,
                capabilities: vec![Capability::Chat, Capability::FunctionCalling],
                pricing: Some(Pricing {
                    input_per_1k_tokens: 0.0005,
                    output_per_1k_tokens: 0.0015,
                }),
                is_free: false,
            },
            ModelInfo {
                id: "palm-2-text-bison".to_string(),
                name: "PaLM 2 Text Bison".to_string(),
                provider: self.name().to_string(),
                context_window: 8192,
                capabilities: vec![Capability::Chat],
                pricing: Some(Pricing {
                    input_per_1k_tokens: 0.001,
                    output_per_1k_tokens: 0.002,
                }),
                is_free: false,
            },
        ]
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, ProviderError> {
        debug!("Sending chat request to GCP Vertex AI: {}", request.model);

        // For now, implement Gemini support
        if request.model.starts_with("gemini") {
            let gemini_request = GeminiGenerateRequest {
                contents: vec![GeminiContent {
                    role: "user".to_string(),
                    parts: vec![GeminiPart {
                        text: request
                            .messages
                            .iter()
                            .map(|m| m.content.as_str())
                            .collect::<Vec<_>>()
                            .join("\n"),
                    }],
                }],
                generation_config: Some(GeminiGenerationConfig {
                    temperature: request.temperature,
                    max_output_tokens: request.max_tokens,
                    top_p: Some(0.8),
                    top_k: Some(10),
                }),
            };

            let url = self.gemini_chat_url(&request.model);

            let response = self
                .client
                .post(&url)
                .header("Authorization", self.get_auth_header())
                .header("Content-Type", "application/json")
                .json(&gemini_request)
                .send()
                .await
                .map_err(|e| {
                    error!("Failed to send request to GCP Vertex AI: {}", e);
                    ProviderError::NetworkError(e.to_string())
                })?;

            if !response.status().is_success() {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_default();
                error!("GCP Vertex AI API error: {} - {}", status, error_text);
                return Err(ProviderError::ProviderError(format!(
                    "GCP Vertex AI API error: {} - {}",
                    status, error_text
                )));
            }

            let gemini_response: GeminiGenerateResponse = response.json().await.map_err(|e| {
                error!("Failed to parse GCP Vertex AI response: {}", e);
                ProviderError::ParseError(e.to_string())
            })?;

            Self::convert_gemini_response(gemini_response, request.model)
        } else {
            Err(ProviderError::ProviderError(format!(
                "Model {} not supported by GCP Vertex AI provider",
                request.model
            )))
        }
    }

    async fn chat_stream(
        &self,
        _request: ChatRequest,
    ) -> Result<crate::provider::ChatStream, ProviderError> {
        // Streaming implementation would go here
        Err(ProviderError::ProviderError(
            "Streaming not yet implemented for GCP Vertex AI".to_string(),
        ))
    }

    fn count_tokens(&self, content: &str, model: &str) -> Result<usize, ProviderError> {
        Ok(self.token_counter.count_tokens_openai(content, model))
    }

    async fn health_check(&self) -> Result<bool, ProviderError> {
        // Check if we have valid configuration
        Ok(!self.project_id.is_empty()
            && !self.location.is_empty()
            && !self.access_token.is_empty())
    }
}

/// Gemini generate content request
#[derive(Serialize)]
struct GeminiGenerateRequest {
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GeminiGenerationConfig>,
}

/// Gemini content structure
#[derive(Serialize, Deserialize)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

/// Gemini part structure
#[derive(Serialize, Deserialize)]
struct GeminiPart {
    text: String,
}

/// Gemini generation config
#[derive(Serialize)]
struct GeminiGenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_k: Option<usize>,
}

/// Gemini generate response
#[derive(Deserialize)]
struct GeminiGenerateResponse {
    candidates: Vec<GeminiCandidate>,
}

/// Gemini candidate
#[derive(Deserialize)]
struct GeminiCandidate {
    content: Option<GeminiContent>,
    finish_reason: Option<String>,
}
