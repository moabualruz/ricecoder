//! Ollama provider implementation
//!
//! Supports local model execution via Ollama.
//! Ollama allows running large language models locally without sending code to external services.

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, warn};

use crate::error::ProviderError;
use crate::models::{Capability, ChatRequest, ChatResponse, FinishReason, ModelInfo, TokenUsage};
use crate::provider::Provider;

/// Ollama provider implementation
pub struct OllamaProvider {
    client: Arc<Client>,
    base_url: String,
    available_models: Vec<ModelInfo>,
}

impl OllamaProvider {
    /// Create a new Ollama provider instance
    pub fn new(base_url: String) -> Result<Self, ProviderError> {
        if base_url.is_empty() {
            return Err(ProviderError::ConfigError(
                "Ollama base URL is required".to_string(),
            ));
        }

        Ok(Self {
            client: Arc::new(Client::new()),
            base_url,
            available_models: vec![],
        })
    }

    /// Create a new Ollama provider with default localhost endpoint
    pub fn with_default_endpoint() -> Result<Self, ProviderError> {
        Self::new("http://localhost:11434".to_string())
    }

    /// Fetch available models from Ollama
    pub async fn fetch_models(&mut self) -> Result<(), ProviderError> {
        debug!("Fetching available models from Ollama");

        let response = self
            .client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await
            .map_err(|e| {
                error!("Failed to fetch models from Ollama: {}", e);
                ProviderError::NetworkError
            })?;

        if !response.status().is_success() {
            return Err(ProviderError::ProviderError(format!(
                "Ollama API error: {}",
                response.status()
            )));
        }

        let tags_response: OllamaTagsResponse = response.json().await.map_err(|e| {
            error!("Failed to parse Ollama tags response: {}", e);
            ProviderError::ProviderError(format!("Failed to parse Ollama response: {}", e))
        })?;

        // Convert Ollama models to our ModelInfo format
        self.available_models = tags_response
            .models
            .unwrap_or_default()
            .into_iter()
            .map(|model| ModelInfo {
                id: model.name.clone(),
                name: model.name.clone(),
                provider: "ollama".to_string(),
                context_window: 4096, // Default context window for local models
                capabilities: vec![Capability::Chat, Capability::Code, Capability::Streaming],
                pricing: None, // Local models have no pricing
            })
            .collect();

        debug!("Fetched {} models from Ollama", self.available_models.len());
        Ok(())
    }

    /// Convert Ollama API response to our ChatResponse
    fn convert_response(
        response: OllamaChatResponse,
        model: String,
    ) -> Result<ChatResponse, ProviderError> {
        Ok(ChatResponse {
            content: response.message.content,
            model,
            usage: TokenUsage {
                prompt_tokens: 0, // Ollama doesn't provide token counts
                completion_tokens: 0,
                total_tokens: 0,
            },
            finish_reason: if response.done {
                FinishReason::Stop
            } else {
                FinishReason::Error
            },
        })
    }
}

#[async_trait]
impl Provider for OllamaProvider {
    fn id(&self) -> &str {
        "ollama"
    }

    fn name(&self) -> &str {
        "Ollama"
    }

    fn models(&self) -> Vec<ModelInfo> {
        if self.available_models.is_empty() {
            // Return some common Ollama models as defaults
            vec![
                ModelInfo {
                    id: "mistral".to_string(),
                    name: "Mistral".to_string(),
                    provider: "ollama".to_string(),
                    context_window: 8192,
                    capabilities: vec![Capability::Chat, Capability::Code, Capability::Streaming],
                    pricing: None,
                },
                ModelInfo {
                    id: "neural-chat".to_string(),
                    name: "Neural Chat".to_string(),
                    provider: "ollama".to_string(),
                    context_window: 4096,
                    capabilities: vec![Capability::Chat, Capability::Code, Capability::Streaming],
                    pricing: None,
                },
                ModelInfo {
                    id: "llama2".to_string(),
                    name: "Llama 2".to_string(),
                    provider: "ollama".to_string(),
                    context_window: 4096,
                    capabilities: vec![Capability::Chat, Capability::Code, Capability::Streaming],
                    pricing: None,
                },
            ]
        } else {
            self.available_models.clone()
        }
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, ProviderError> {
        debug!("Sending chat request to Ollama for model: {}", request.model);

        let ollama_request = OllamaChatRequest {
            model: request.model.clone(),
            messages: request
                .messages
                .iter()
                .map(|m| OllamaMessage {
                    role: m.role.clone(),
                    content: m.content.clone(),
                })
                .collect(),
            stream: false,
        };

        let response = self
            .client
            .post(format!("{}/api/chat", self.base_url))
            .json(&ollama_request)
            .send()
            .await
            .map_err(|e| {
                error!("Ollama API request failed: {}", e);
                ProviderError::NetworkError
            })?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("Ollama API error ({}): {}", status, error_text);

            return Err(ProviderError::ProviderError(format!(
                "Ollama API error: {}",
                status
            )));
        }

        let ollama_response: OllamaChatResponse = response.json().await.map_err(|e| {
            error!("Failed to parse Ollama response: {}", e);
            ProviderError::ProviderError(format!("Failed to parse Ollama response: {}", e))
        })?;

        Self::convert_response(ollama_response, request.model)
    }

    async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<crate::provider::ChatStream, ProviderError> {
        debug!("Starting streaming chat request to Ollama for model: {}", request.model);

        let ollama_request = OllamaChatRequest {
            model: request.model.clone(),
            messages: request
                .messages
                .iter()
                .map(|m| OllamaMessage {
                    role: m.role.clone(),
                    content: m.content.clone(),
                })
                .collect(),
            stream: true,
        };

        let response = self
            .client
            .post(format!("{}/api/chat", self.base_url))
            .json(&ollama_request)
            .send()
            .await
            .map_err(|e| {
                error!("Ollama streaming request failed: {}", e);
                ProviderError::NetworkError
            })?;

        let status = response.status();
        if !status.is_success() {
            return Err(ProviderError::ProviderError(format!(
                "Ollama API error: {}",
                status
            )));
        }

        // For now, return an error as streaming implementation requires more complex handling
        Err(ProviderError::ProviderError(
            "Streaming not yet implemented for Ollama".to_string(),
        ))
    }

    fn count_tokens(&self, content: &str, _model: &str) -> Result<usize, ProviderError> {
        // Ollama doesn't provide an exact token counting API
        // Use a reasonable approximation: 1 token ≈ 4 characters
        let token_count = content.len().div_ceil(4);
        Ok(token_count)
    }

    async fn health_check(&self) -> Result<bool, ProviderError> {
        debug!("Performing health check for Ollama provider");

        let response = self
            .client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await
            .map_err(|e| {
                warn!("Ollama health check failed: {}", e);
                ProviderError::NetworkError
            })?;

        match response.status().as_u16() {
            200 => {
                debug!("Ollama health check passed");
                Ok(true)
            }
            _ => {
                warn!("Ollama health check failed with status: {}", response.status());
                Ok(false)
            }
        }
    }
}

/// Ollama API chat request format
#[derive(Debug, Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
}

/// Ollama API message format
#[derive(Debug, Serialize, Deserialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

/// Ollama API chat response format
#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    message: OllamaResponseMessage,
    done: bool,
}

/// Ollama API response message format
#[derive(Debug, Deserialize)]
struct OllamaResponseMessage {
    #[allow(dead_code)]
    role: String,
    content: String,
}

/// Ollama API tags response format
#[derive(Debug, Deserialize)]
struct OllamaTagsResponse {
    models: Option<Vec<OllamaModel>>,
}

/// Ollama model information
#[derive(Debug, Deserialize, Clone)]
struct OllamaModel {
    name: String,
}
