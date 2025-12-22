//! Local model manager for handling model lifecycle operations

use std::sync::Arc;

use reqwest::Client;
use serde::Deserialize;
use tracing::{debug, error, info, warn};

use crate::{
    error::LocalModelError,
    models::{LocalModel, ModelMetadata, PullProgress},
    Result,
};

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
pub struct LocalModelManager {
    client: Arc<Client>,
    base_url: String,
}

impl LocalModelManager {
    /// Create a new local model manager
    pub fn new(base_url: String) -> Result<Self> {
        if base_url.is_empty() {
            return Err(LocalModelError::ConfigError(
                "Ollama base URL is required".to_string(),
            ));
        }

        Ok(Self {
            client: Arc::new(Client::new()),
            base_url,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_model_manager_creation() {
        let manager = LocalModelManager::new("http://localhost:11434".to_string());
        assert!(manager.is_ok());
    }

    #[test]
    fn test_local_model_manager_empty_url() {
        let manager = LocalModelManager::new("".to_string());
        assert!(manager.is_err());
    }

    #[test]
    fn test_local_model_manager_default_endpoint() {
        let manager = LocalModelManager::with_default_endpoint();
        assert!(manager.is_ok());
    }

    #[test]
    fn test_pull_model_empty_name() {
        let manager = LocalModelManager::new("http://localhost:11434".to_string()).unwrap();
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(manager.pull_model(""));
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_model_empty_name() {
        let manager = LocalModelManager::new("http://localhost:11434".to_string()).unwrap();
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(manager.remove_model(""));
        assert!(result.is_err());
    }

    #[test]
    fn test_update_model_empty_name() {
        let manager = LocalModelManager::new("http://localhost:11434".to_string()).unwrap();
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(manager.update_model(""));
        assert!(result.is_err());
    }

    #[test]
    fn test_get_model_info_empty_name() {
        let manager = LocalModelManager::new("http://localhost:11434".to_string()).unwrap();
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(manager.get_model_info(""));
        assert!(result.is_err());
    }
}
