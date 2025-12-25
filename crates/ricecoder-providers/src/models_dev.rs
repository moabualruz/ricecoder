//! Models.dev API integration for dynamic model discovery
//!
//! This module fetches model metadata from https://models.dev/api.json
//! with caching and fallback support.

use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};
use crate::{error::ProviderError, models::ModelInfo, models::Capability};

/// Models.dev API URL
const MODELS_DEV_API: &str = "https://models.dev/api.json";

/// Cache TTL (60 minutes)
const CACHE_TTL: Duration = Duration::from_secs(60 * 60);

/// Fetch timeout (10 seconds)
const FETCH_TIMEOUT: Duration = Duration::from_secs(10);

/// Models.dev API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsDevResponse {
    pub models: Vec<ModelsDevModel>,
}

/// Model metadata from models.dev
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsDevModel {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub context_window: usize,
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub experimental: bool,
}

/// Cached models.dev response
#[derive(Debug, Clone)]
pub struct ModelsDevCache {
    pub models: Vec<ModelInfo>,
    pub fetched_at: SystemTime,
}

impl ModelsDevCache {
    /// Check if cache is still valid
    pub fn is_valid(&self) -> bool {
        if let Ok(elapsed) = self.fetched_at.elapsed() {
            elapsed < CACHE_TTL
        } else {
            false
        }
    }
}

/// Fetch models from models.dev API
pub async fn fetch_models(http_client: &reqwest::Client) -> Result<Vec<ModelInfo>, ProviderError> {
    let response = http_client
        .get(MODELS_DEV_API)
        .timeout(FETCH_TIMEOUT)
        .header("User-Agent", "ricecoder/0.1")
        .send()
        .await
        .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

    let api_response: ModelsDevResponse = response
        .json()
        .await
        .map_err(|e| ProviderError::SerializationError(e.to_string()))?;

    Ok(api_response.models.into_iter().map(convert_model).collect())
}

/// Convert models.dev model to ModelInfo
fn convert_model(model: ModelsDevModel) -> ModelInfo {
    let capabilities = model
        .capabilities
        .iter()
        .filter_map(|c| match c.as_str() {
            "chat" => Some(Capability::Chat),
            "code" => Some(Capability::Code),
            "vision" => Some(Capability::Vision),
            "function_calling" => Some(Capability::FunctionCalling),
            "streaming" => Some(Capability::Streaming),
            _ => None,
        })
        .collect();

    ModelInfo {
        id: model.id,
        name: model.name,
        provider: model.provider,
        context_window: model.context_window,
        capabilities,
        pricing: None,
        is_free: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_validity() {
        let cache = ModelsDevCache {
            models: vec![],
            fetched_at: SystemTime::now(),
        };
        assert!(cache.is_valid());
    }

    #[test]
    fn test_convert_model() {
        let model = ModelsDevModel {
            id: "test-model".to_string(),
            name: "Test Model".to_string(),
            provider: "test".to_string(),
            context_window: 4096,
            capabilities: vec!["chat".to_string(), "code".to_string()],
            status: "stable".to_string(),
            experimental: false,
        };

        let info = convert_model(model);
        assert_eq!(info.id, "test-model");
        assert_eq!(info.capabilities.len(), 2);
    }
}
