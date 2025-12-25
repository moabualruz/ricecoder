//! Local model manager for handling model lifecycle operations
//!
//! This module provides the core functionality for managing local AI models
//! through the Ollama API. It implements best practices including:
//! - Connection pooling with TCP keep-alive
//! - Health checks with configurable timeouts
//! - Exponential backoff for transient errors

use std::sync::Arc;
use std::time::Duration;

use reqwest::Client;
use serde::Deserialize;
use tracing::{debug, error, info, warn};

use crate::{
    error::LocalModelError,
    models::{LocalModel, ModelMetadata, PullProgress},
    Result,
};

/// Default timeout for Ollama API requests (30 seconds)
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Default pool idle timeout (90 seconds)
const DEFAULT_POOL_IDLE_TIMEOUT_SECS: u64 = 90;

/// Default TCP keep-alive interval (60 seconds)
const DEFAULT_TCP_KEEPALIVE_SECS: u64 = 60;

/// Health check timeout (5 seconds)
const HEALTH_CHECK_TIMEOUT_SECS: u64 = 5;

/// Ollama API response for model pull
#[derive(Debug, Deserialize)]
struct OllamaPullResponse {
    status: String,
    digest: String,
    total: Option<u64>,
    completed: Option<u64>,
}

/// Ollama API response for model deletion
/// Reserved for future use when model deletion is implemented
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct OllamaDeleteResponse {
    status: String,
}

/// Local model manager for Ollama
///
/// Provides a high-level interface for managing local AI models through the Ollama API.
/// Implements connection pooling, health checks, and model lifecycle management.
pub struct LocalModelManager {
    client: Arc<Client>,
    base_url: String,
    timeout: Duration,
}

impl LocalModelManager {
    /// Create a new local model manager with default timeout
    pub fn new(base_url: String) -> Result<Self> {
        Self::with_timeout(base_url, Duration::from_secs(DEFAULT_TIMEOUT_SECS))
    }

    /// Create a new local model manager with custom timeout
    ///
    /// # Arguments
    /// * `base_url` - The Ollama server URL (e.g., "http://localhost:11434")
    /// * `timeout` - Request timeout duration
    ///
    /// # Errors
    /// Returns `ConfigError` if base_url is empty
    pub fn with_timeout(base_url: String, timeout: Duration) -> Result<Self> {
        if base_url.is_empty() {
            return Err(LocalModelError::ConfigError(
                "Ollama base URL is required".to_string(),
            ));
        }

        // Build client with connection pooling and keep-alive
        let client = Client::builder()
            .timeout(timeout)
            .pool_idle_timeout(Duration::from_secs(DEFAULT_POOL_IDLE_TIMEOUT_SECS))
            .tcp_keepalive(Duration::from_secs(DEFAULT_TCP_KEEPALIVE_SECS))
            .build()
            .map_err(|e| LocalModelError::ConfigError(format!("Failed to build HTTP client: {}", e)))?;

        Ok(Self {
            client: Arc::new(client),
            base_url,
            timeout,
        })
    }

    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Create a new local model manager with default localhost endpoint
    pub fn with_default_endpoint() -> Result<Self> {
        Self::new("http://localhost:11434".to_string())
    }

    /// Get the configured timeout
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Pull a model from Ollama registry
    /// Returns a stream of progress updates
    pub async fn pull_model(&self, model_name: &str) -> Result<Vec<PullProgress>> {
        if model_name.is_empty() {
            return Err(LocalModelError::InvalidModelName(
                "Model name cannot be empty".to_string(),
            ));
        }

        debug!("Pulling model: {}", model_name);

        let url = format!("{}/api/pull", self.base_url);
        let request_body = serde_json::json!({
            "name": model_name,
            "stream": true
        });

        let response = self
            .client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| LocalModelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Failed to pull model {}: {}", model_name, error_text);
            return Err(LocalModelError::PullFailed(format!(
                "HTTP {}: {}",
                status, error_text
            )));
        }

        let body = response.text().await.map_err(|e| {
            error!("Failed to read pull response: {}", e);
            LocalModelError::NetworkError(e.to_string())
        })?;

        // Parse streaming responses
        let mut progress_updates = Vec::new();
        for line in body.lines() {
            if line.is_empty() {
                continue;
            }

            match serde_json::from_str::<OllamaPullResponse>(line) {
                Ok(resp) => {
                    let progress = PullProgress {
                        model: model_name.to_string(),
                        status: resp.status,
                        digest: resp.digest,
                        total: resp.total.unwrap_or(0),
                        completed: resp.completed.unwrap_or(0),
                    };
                    progress_updates.push(progress);
                }
                Err(e) => {
                    warn!("Failed to parse pull response line: {}", e);
                }
            }
        }

        info!("Successfully pulled model: {}", model_name);
        Ok(progress_updates)
    }

    /// Remove a model from local storage
    pub async fn remove_model(&self, model_name: &str) -> Result<()> {
        if model_name.is_empty() {
            return Err(LocalModelError::InvalidModelName(
                "Model name cannot be empty".to_string(),
            ));
        }

        debug!("Removing model: {}", model_name);

        let url = format!("{}/api/delete", self.base_url);
        let request_body = serde_json::json!({
            "name": model_name
        });

        let response = self
            .client
            .delete(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| LocalModelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Failed to remove model {}: {}", model_name, error_text);
            return Err(LocalModelError::RemovalFailed(format!(
                "HTTP {}: {}",
                status, error_text
            )));
        }

        info!("Successfully removed model: {}", model_name);
        Ok(())
    }

    /// Update a model to the latest version
    pub async fn update_model(&self, model_name: &str) -> Result<Vec<PullProgress>> {
        if model_name.is_empty() {
            return Err(LocalModelError::InvalidModelName(
                "Model name cannot be empty".to_string(),
            ));
        }

        debug!("Updating model: {}", model_name);

        // Update is essentially a pull with the latest tag
        let model_with_tag = if model_name.contains(':') {
            model_name.to_string()
        } else {
            format!("{}:latest", model_name)
        };

        self.pull_model(&model_with_tag).await
    }

    /// Get information about a specific model
    pub async fn get_model_info(&self, model_name: &str) -> Result<LocalModel> {
        if model_name.is_empty() {
            return Err(LocalModelError::InvalidModelName(
                "Model name cannot be empty".to_string(),
            ));
        }

        debug!("Getting model info: {}", model_name);

        let url = format!("{}/api/show", self.base_url);
        let request_body = serde_json::json!({
            "name": model_name
        });

        let response = self
            .client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| LocalModelError::NetworkError(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            if status == 404 {
                return Err(LocalModelError::ModelNotFound(model_name.to_string()));
            }
            let error_text = response.text().await.unwrap_or_default();
            error!(
                "Failed to get model info for {}: {}",
                model_name, error_text
            );
            return Err(LocalModelError::Unknown(format!(
                "HTTP {}: {}",
                status, error_text
            )));
        }

        let model_info: OllamaModelInfo = response.json().await.map_err(|e| {
            error!("Failed to parse model info response: {}", e);
            LocalModelError::NetworkError(e.to_string())
        })?;

        Ok(LocalModel {
            name: model_info.name,
            size: model_info.details.parameter_size.parse().unwrap_or(0),
            digest: model_info.digest,
            modified_at: model_info.modified_at,
            metadata: ModelMetadata {
                format: model_info.details.format,
                family: model_info.details.family,
                parameter_size: model_info.details.parameter_size,
                quantization_level: model_info.details.quantization_level,
            },
        })
    }

    /// List all available models
    pub async fn list_models(&self) -> Result<Vec<LocalModel>> {
        debug!("Listing all models");

        let url = format!("{}/api/tags", self.base_url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| LocalModelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Failed to list models: {}", error_text);
            return Err(LocalModelError::Unknown(format!(
                "HTTP {}: {}",
                status, error_text
            )));
        }

        let tags_response: OllamaTagsResponse = response.json().await.map_err(|e| {
            error!("Failed to parse tags response: {}", e);
            LocalModelError::NetworkError(e.to_string())
        })?;

        let models: Vec<LocalModel> = tags_response
            .models
            .unwrap_or_default()
            .into_iter()
            .map(|m| LocalModel {
                name: m.name,
                size: m.size,
                digest: m.digest,
                modified_at: m.modified_at,
                metadata: ModelMetadata {
                    format: "gguf".to_string(), // Default format
                    family: "unknown".to_string(),
                    parameter_size: "unknown".to_string(),
                    quantization_level: "unknown".to_string(),
                },
            })
            .collect();

        debug!("Listed {} models", models.len());
        Ok(models)
    }

    /// Check if a model exists
    pub async fn model_exists(&self, model_name: &str) -> Result<bool> {
        match self.get_model_info(model_name).await {
            Ok(_) => Ok(true),
            Err(LocalModelError::ModelNotFound(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Check if the Ollama server is reachable and responding
    ///
    /// Performs a lightweight health check by calling the root endpoint.
    /// Uses a shorter timeout (5 seconds) than normal operations.
    /// Returns true if the server responds successfully.
    ///
    /// # Example
    /// ```ignore
    /// let manager = LocalModelManager::with_default_endpoint()?;
    /// if manager.health_check().await? {
    ///     println!("Ollama is running");
    /// }
    /// ```
    pub async fn health_check(&self) -> Result<bool> {
        debug!("Performing health check on Ollama server at {}", self.base_url);

        // Use root endpoint for simple health check
        let url = format!("{}/", self.base_url);

        // Use shorter timeout for health checks
        match self.client
            .get(&url)
            .timeout(Duration::from_secs(HEALTH_CHECK_TIMEOUT_SECS))
            .send()
            .await
        {
            Ok(response) => {
                let healthy = response.status().is_success();
                if healthy {
                    debug!("Ollama server health check passed");
                } else {
                    warn!("Ollama server health check failed: HTTP {}", response.status());
                }
                Ok(healthy)
            }
            Err(e) => {
                warn!("Ollama server health check failed: {}", e);
                // Network errors indicate server is not reachable
                Ok(false)
            }
        }
    }

    /// Check server health with retry and exponential backoff
    ///
    /// Attempts up to 3 retries with exponential backoff (100ms, 200ms, 400ms).
    /// Returns true if any attempt succeeds.
    pub async fn health_check_with_retry(&self) -> Result<bool> {
        const MAX_RETRIES: u32 = 3;
        const INITIAL_BACKOFF_MS: u64 = 100;

        for attempt in 0..MAX_RETRIES {
            match self.health_check().await {
                Ok(true) => return Ok(true),
                Ok(false) | Err(_) if attempt < MAX_RETRIES - 1 => {
                    let backoff_ms = INITIAL_BACKOFF_MS * 2_u64.pow(attempt);
                    debug!("Health check attempt {} failed, retrying in {}ms", attempt + 1, backoff_ms);
                    tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                }
                Ok(false) => return Ok(false),
                Err(e) => return Err(e),
            }
        }
        Ok(false)
    }

    /// Get the HTTP client for advanced usage
    ///
    /// Useful for implementing custom Ollama API calls.
    pub fn client(&self) -> &Arc<Client> {
        &self.client
    }
}

/// Ollama API response for model info
#[derive(Debug, Deserialize)]
struct OllamaModelInfo {
    name: String,
    digest: String,
    modified_at: chrono::DateTime<chrono::Utc>,
    #[allow(dead_code)]
    size: u64,
    details: OllamaModelDetails,
}

/// Ollama model details
#[derive(Debug, Deserialize)]
struct OllamaModelDetails {
    format: String,
    family: String,
    parameter_size: String,
    quantization_level: String,
}

/// Ollama API response for tags
#[derive(Debug, Deserialize)]
struct OllamaTagsResponse {
    models: Option<Vec<OllamaModelTag>>,
}

/// Ollama model tag
#[derive(Debug, Deserialize)]
struct OllamaModelTag {
    name: String,
    digest: String,
    modified_at: chrono::DateTime<chrono::Utc>,
    size: u64,
}

// Tests moved to tests/local_model_manager.rs per project test organization policy
