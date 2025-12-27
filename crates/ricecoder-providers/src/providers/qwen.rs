//! Qwen provider implementation
//!
//! Supports Alibaba's Qwen models via DashScope API or local deployment.
//! Includes full streaming support and Qwen3 thinking mode.
//!
//! ## Features
//! - DashScope cloud API (International and China regions)
//! - Local deployment support (vLLM, LM Studio, Ollama)
//! - Qwen3 thinking mode with `<think>` tags
//! - Full streaming support via SSE
//!
//! ## Supported Models
//! - qwen3-235b-a22b (Qwen3 MoE flagship)
//! - qwen3-32b, qwen3-14b, qwen3-8b
//! - qwen2.5-coder-32b-instruct, qwen2.5-coder-14b-instruct, qwen2.5-coder-7b-instruct
//! - qwen-max, qwen-plus, qwen-turbo (DashScope)

use std::sync::Arc;
use std::time::Duration;

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

// DashScope API endpoints
const DASHSCOPE_INTL_URL: &str =
    "https://dashscope-intl.aliyuncs.com/compatible-mode/v1/chat/completions";
const DASHSCOPE_CN_URL: &str =
    "https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions";

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(180);
const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// Qwen3 thinking mode configuration
#[derive(Debug, Clone, Default)]
pub struct ThinkingConfig {
    /// Enable thinking mode (Qwen3 feature)
    pub enabled: bool,
    /// Budget tokens for thinking (optional)
    pub budget_tokens: Option<u32>,
}

/// Qwen provider for Alibaba's Qwen models
pub struct QwenProvider {
    api_key: String,
    base_url: String,
    client: Arc<Client>,
    custom_default_model: Option<String>,
    thinking_config: ThinkingConfig,
    token_counter: Arc<TokenCounter>,
    models: Vec<ModelInfo>,
}

impl QwenProvider {
    /// Create provider for DashScope International (Singapore)
    pub fn dashscope_intl(api_key: String, models: Vec<ModelInfo>) -> Result<Self, ProviderError> {
        Self::with_base_url(api_key, DASHSCOPE_INTL_URL.to_string(), models)
    }

    /// Create provider for DashScope International with default models from registry
    #[allow(dead_code)]
    pub fn dashscope_intl_with_default_models(api_key: String) -> Result<Self, ProviderError> {
        use crate::model_registry::global_registry;
        let models = global_registry().get_provider_models("qwen");
        Self::dashscope_intl(api_key, models)
    }

    /// Create provider for DashScope China (Beijing)
    pub fn dashscope_cn(api_key: String, models: Vec<ModelInfo>) -> Result<Self, ProviderError> {
        Self::with_base_url(api_key, DASHSCOPE_CN_URL.to_string(), models)
    }

    /// Create provider for DashScope China with default models from registry
    #[allow(dead_code)]
    pub fn dashscope_cn_with_default_models(api_key: String) -> Result<Self, ProviderError> {
        use crate::model_registry::global_registry;
        let models = global_registry().get_provider_models("qwen");
        Self::dashscope_cn(api_key, models)
    }

    /// Create provider for local Qwen deployment (vLLM, LM Studio, Ollama)
    pub fn local(base_url: String, models: Vec<ModelInfo>) -> Result<Self, ProviderError> {
        let client = Self::build_client()?;

        Ok(Self {
            api_key: "not-needed".to_string(),
            base_url,
            client,
            custom_default_model: None,
            thinking_config: ThinkingConfig::default(),
            token_counter: Arc::new(TokenCounter::new()),
            models,
        })
    }

    /// Create provider for local Qwen deployment with default models from registry
    #[allow(dead_code)]
    pub fn local_with_default_models(base_url: String) -> Result<Self, ProviderError> {
        use crate::model_registry::global_registry;
        let models = global_registry().get_provider_models("qwen");
        Self::local(base_url, models)
    }

    /// Create with custom base URL and API key
    pub fn with_base_url(api_key: String, base_url: String, models: Vec<ModelInfo>) -> Result<Self, ProviderError> {
        if api_key.is_empty() {
            return Err(ProviderError::ConfigError(
                "Qwen API key is required".to_string(),
            ));
        }

        let client = Self::build_client()?;

        Ok(Self {
            api_key,
            base_url,
            client,
            custom_default_model: None,
            thinking_config: ThinkingConfig::default(),
            token_counter: Arc::new(TokenCounter::new()),
            models,
        })
    }

    /// Set custom default model
    pub fn with_default_model(mut self, model: String) -> Self {
        self.custom_default_model = Some(model);
        self
    }

    /// Enable Qwen3 thinking mode
    pub fn with_thinking(mut self, enabled: bool) -> Self {
        self.thinking_config.enabled = enabled;
        self
    }

    /// Set thinking budget tokens
    pub fn with_thinking_budget(mut self, budget_tokens: u32) -> Self {
        self.thinking_config.budget_tokens = Some(budget_tokens);
        self
    }

    fn build_client() -> Result<Arc<Client>, ProviderError> {
        Client::builder()
            .timeout(DEFAULT_TIMEOUT)
            .connect_timeout(DEFAULT_CONNECT_TIMEOUT)
            .build()
            .map(Arc::new)
            .map_err(|e| ProviderError::ConfigError(format!("Failed to create HTTP client: {}", e)))
    }

    /// Get authorization header
    fn get_auth_header(&self) -> String {
        format!("Bearer {}", self.api_key)
    }

    /// Extract thinking content from Qwen3 response
    fn extract_thinking(&self, text: &str) -> (Option<String>, String) {
        if !self.thinking_config.enabled {
            return (None, text.to_string());
        }

        // Look for <think> ... </think> blocks
        if let Some(start) = text.find("<think>") {
            if let Some(end) = text.find("</think>") {
                let thinking = text[start + 7..end].trim().to_string();
                let before = &text[..start];
                let after = &text[end + 8..];
                let remaining = format!("{}{}", before.trim(), after.trim());
                return (Some(thinking), remaining);
            }
        }

        (None, text.to_string())
    }

    /// Convert Qwen API response to our ChatResponse
    fn convert_response(
        response: QwenChatResponse,
        model: String,
        thinking_enabled: bool,
    ) -> Result<ChatResponse, ProviderError> {
        let choice = response
            .choices
            .first()
            .ok_or_else(|| ProviderError::ProviderError("No choices in response".to_string()))?;

        let raw_content = choice
            .message
            .content
            .clone()
            .unwrap_or_default();

        // Extract thinking if enabled
        let content = if thinking_enabled {
            if let Some(start) = raw_content.find("<think>") {
                if let Some(end) = raw_content.find("</think>") {
                    let before = &raw_content[..start];
                    let after = &raw_content[end + 8..];
                    format!("{}{}", before.trim(), after.trim())
                } else {
                    raw_content
                }
            } else {
                raw_content
            }
        } else {
            raw_content
        };

        let finish_reason = match choice.finish_reason.as_deref() {
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
                total_tokens: response.usage.prompt_tokens + response.usage.completion_tokens,
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

        for line in body.lines() {
            if let Some(json_str) = line.strip_prefix("data: ") {
                if json_str.trim() == "[DONE]" {
                    trace!("Stream completed with [DONE] marker");
                    continue;
                }

                match serde_json::from_str::<QwenStreamChunk>(json_str) {
                    Ok(chunk) => {
                        if let Some(choice) = chunk.choices.first() {
                            if let Some(delta) = &choice.delta {
                                if let Some(content) = &delta.content {
                                    if !content.is_empty() {
                                        responses.push(Ok(ChatResponse {
                                            content: content.clone(),
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
                        debug!("Failed to parse SSE chunk: {} - data: {}", e, json_str);
                    }
                }
            }
        }

        let chat_stream = futures::stream::iter(responses);
        Ok(chat_stream.boxed())
    }
}

#[async_trait]
impl Provider for QwenProvider {
    fn id(&self) -> &str {
        "qwen"
    }

    fn name(&self) -> &str {
        "Qwen"
    }

    fn models(&self) -> Vec<ModelInfo> {
        self.models.clone()
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, ProviderError> {
        // Build system message with thinking instruction if enabled
        let mut messages: Vec<QwenMessage> = Vec::new();

        if self.thinking_config.enabled {
            let thinking_instruction = if let Some(budget) = self.thinking_config.budget_tokens {
                format!(
                    "You have thinking mode enabled. Use <think></think> tags to show your reasoning. Budget: {} tokens.",
                    budget
                )
            } else {
                "You have thinking mode enabled. Use <think></think> tags to show your reasoning before your final answer.".to_string()
            };
            messages.push(QwenMessage {
                role: "system".to_string(),
                content: Some(thinking_instruction),
            });
        }

        // Convert messages
        for msg in &request.messages {
            messages.push(QwenMessage {
                role: msg.role.clone(),
                content: Some(msg.content.clone()),
            });
        }

        let qwen_request = QwenChatRequest {
            model: request.model.clone(),
            messages,
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            stream: false,
        };

        debug!(
            "Sending chat request to Qwen for model: {}",
            request.model
        );

        let mut request_builder = self
            .client
            .post(&self.base_url)
            .header("Content-Type", "application/json")
            .json(&qwen_request);

        if self.api_key != "not-needed" {
            request_builder = request_builder.header("Authorization", self.get_auth_header());
        }

        let response = request_builder.send().await.map_err(|e| {
            error!("Qwen API request failed: {}", e);
            ProviderError::from(e)
        })?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("Qwen API error ({}): {}", status, error_text);

            return match status.as_u16() {
                401 => Err(ProviderError::AuthError),
                429 => Err(ProviderError::RateLimited(60)),
                _ => Err(ProviderError::ProviderError(format!(
                    "Qwen API error: {}",
                    status
                ))),
            };
        }

        let qwen_response: QwenChatResponse = response.json().await?;
        Self::convert_response(qwen_response, request.model, self.thinking_config.enabled)
    }

    async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<crate::provider::ChatStream, ProviderError> {
        // Build messages with thinking instruction if enabled
        let mut messages: Vec<QwenMessage> = Vec::new();

        if self.thinking_config.enabled {
            let thinking_instruction = if let Some(budget) = self.thinking_config.budget_tokens {
                format!(
                    "You have thinking mode enabled. Use <think></think> tags for reasoning. Budget: {} tokens.",
                    budget
                )
            } else {
                "You have thinking mode enabled. Use <think></think> tags for reasoning.".to_string()
            };
            messages.push(QwenMessage {
                role: "system".to_string(),
                content: Some(thinking_instruction),
            });
        }

        for msg in &request.messages {
            messages.push(QwenMessage {
                role: msg.role.clone(),
                content: Some(msg.content.clone()),
            });
        }

        let qwen_request = QwenChatRequest {
            model: request.model.clone(),
            messages,
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            stream: true,
        };

        debug!(
            "Starting streaming chat request to Qwen for model: {}",
            request.model
        );

        let mut request_builder = self
            .client
            .post(&self.base_url)
            .header("Content-Type", "application/json")
            .json(&qwen_request);

        if self.api_key != "not-needed" {
            request_builder = request_builder.header("Authorization", self.get_auth_header());
        }

        let response = request_builder.send().await.map_err(|e| {
            error!("Qwen streaming request failed: {}", e);
            ProviderError::from(e)
        })?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("Qwen streaming API error ({}): {}", status, error_text);

            return match status.as_u16() {
                401 => Err(ProviderError::AuthError),
                429 => Err(ProviderError::RateLimited(60)),
                _ => Err(ProviderError::ProviderError(format!(
                    "Qwen API error: {}",
                    status
                ))),
            };
        }

        let model = request.model.clone();
        let body = response.text().await.map_err(|e| {
            error!("Failed to read streaming response body: {}", e);
            ProviderError::NetworkError(e.to_string())
        })?;

        Self::parse_sse_response(&body, model)
    }

    fn count_tokens(&self, content: &str, model: &str) -> Result<usize, ProviderError> {
        // Qwen uses similar tokenization to OpenAI
        let tokens = self.token_counter.count_tokens_openai(content, model);
        Ok(tokens)
    }

    async fn health_check(&self) -> Result<bool, ProviderError> {
        debug!("Performing health check for Qwen provider");

        // For local deployments, just check if the endpoint is reachable
        let mut request_builder = self.client.get(&self.base_url.replace("/chat/completions", "/models"));

        if self.api_key != "not-needed" {
            request_builder = request_builder.header("Authorization", self.get_auth_header());
        }

        let response = request_builder.send().await;

        match response {
            Ok(resp) => match resp.status().as_u16() {
                200 => {
                    debug!("Qwen health check passed");
                    Ok(true)
                }
                401 => {
                    error!("Qwen health check failed: authentication error");
                    Err(ProviderError::AuthError)
                }
                _ => {
                    warn!("Qwen health check returned status: {}", resp.status());
                    Ok(false)
                }
            },
            Err(e) => {
                warn!("Qwen health check failed: {}", e);
                Ok(false)
            }
        }
    }
}

// ============================================================================
// Qwen API Types (OpenAI-compatible)
// ============================================================================

#[derive(Debug, Serialize)]
struct QwenChatRequest {
    model: String,
    messages: Vec<QwenMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<usize>,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct QwenMessage {
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct QwenChatResponse {
    choices: Vec<QwenChoice>,
    usage: QwenUsage,
}

#[derive(Debug, Deserialize)]
struct QwenChoice {
    message: QwenMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct QwenUsage {
    prompt_tokens: usize,
    completion_tokens: usize,
}

#[derive(Debug, Deserialize)]
struct QwenStreamChunk {
    choices: Vec<QwenStreamChoice>,
}

#[derive(Debug, Deserialize)]
struct QwenStreamChoice {
    delta: Option<QwenDelta>,
    #[allow(dead_code)]
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct QwenDelta {
    #[serde(default)]
    content: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_models() -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                id: "qwen3-8b".to_string(),
                name: "Qwen3 8B".to_string(),
                provider: "qwen".to_string(),
                context_window: 131072,
                capabilities: vec![Capability::Chat, Capability::Code, Capability::Streaming],
                pricing: None,
                is_free: false,
            },
            ModelInfo {
                id: "qwen2.5-coder-32b-instruct".to_string(),
                name: "Qwen2.5 Coder 32B".to_string(),
                provider: "qwen".to_string(),
                context_window: 131072,
                capabilities: vec![Capability::Chat, Capability::Code, Capability::Streaming],
                pricing: None,
                is_free: false,
            },
            ModelInfo {
                id: "qwen-max".to_string(),
                name: "Qwen Max".to_string(),
                provider: "qwen".to_string(),
                context_window: 32768,
                capabilities: vec![Capability::Chat, Capability::Code, Capability::Streaming],
                pricing: Some(crate::models::Pricing {
                    input_per_1k_tokens: 0.0024,
                    output_per_1k_tokens: 0.0096,
                }),
                is_free: false,
            },
        ]
    }

    #[test]
    fn test_qwen_provider_local() {
        let provider = QwenProvider::local("http://localhost:8000/v1/chat/completions".to_string(), test_models())
            .expect("Should create local provider");
        assert_eq!(provider.id(), "qwen");
        assert_eq!(provider.api_key, "not-needed");
    }

    #[test]
    fn test_thinking_config() {
        let provider = QwenProvider::local("http://localhost:8000/v1/chat/completions".to_string(), test_models())
            .expect("Should create provider")
            .with_thinking(true)
            .with_thinking_budget(5000);

        assert!(provider.thinking_config.enabled);
        assert_eq!(provider.thinking_config.budget_tokens, Some(5000));
    }

    #[test]
    fn test_custom_default_model() {
        let provider = QwenProvider::local("http://localhost:8000/v1/chat/completions".to_string(), test_models())
            .expect("Should create provider")
            .with_default_model("qwen2.5-coder-14b-instruct".to_string());

        assert_eq!(
            provider.custom_default_model,
            Some("qwen2.5-coder-14b-instruct".to_string())
        );
    }

    #[test]
    fn test_extract_thinking() {
        let provider = QwenProvider::local("http://localhost:8000/v1/chat/completions".to_string(), test_models())
            .expect("Should create provider")
            .with_thinking(true);

        let text = "<think>Let me analyze this...</think>Here's the answer.";
        let (thinking, remaining) = provider.extract_thinking(text);

        assert!(thinking.is_some());
        assert!(thinking.unwrap().contains("analyze"));
        assert!(remaining.contains("Here's the answer"));
        assert!(!remaining.contains("<think>"));
    }

    #[test]
    fn test_models_list() {
        let provider = QwenProvider::local("http://localhost:8000/v1/chat/completions".to_string(), test_models())
            .expect("Should create provider");

        let models = provider.models();
        assert!(!models.is_empty());

        let model_ids: Vec<_> = models.iter().map(|m| m.id.as_str()).collect();
        assert!(model_ids.contains(&"qwen3-8b"));
        assert!(model_ids.contains(&"qwen2.5-coder-32b-instruct"));
        assert!(model_ids.contains(&"qwen-max"));
    }
}
