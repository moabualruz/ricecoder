//! Enterprise provider integrations and management

use crate::connections::AuthMethod;
use crate::error::{IndustryError, IndustryResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;

/// Enterprise provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseProviderConfig {
    /// Provider name (e.g., "openai-enterprise", "anthropic-enterprise")
    pub name: String,
    /// Provider type
    pub provider_type: ProviderType,
    /// API base URL
    pub base_url: String,
    /// API version
    pub api_version: String,
    /// Authentication configuration
    pub auth_config: ProviderAuthConfig,
    /// Rate limiting configuration
    pub rate_limit: RateLimitConfig,
    /// Enterprise-specific features
    pub enterprise_features: Vec<EnterpriseFeature>,
    /// Custom configuration
    pub custom_config: HashMap<String, serde_json::Value>,
}

/// Provider types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProviderType {
    /// OpenAI-compatible API
    OpenAI,
    /// Anthropic Claude API
    Anthropic,
    /// Google AI (Gemini, etc.)
    Google,
    /// Azure OpenAI
    AzureOpenAI,
    /// AWS Bedrock
    AWSBedrock,
    /// Custom provider
    Custom(String),
}

/// Provider authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderAuthConfig {
    /// Authentication method
    pub method: AuthMethod,
    /// API key (encrypted)
    pub api_key: Option<String>,
    /// Client ID for OAuth
    pub client_id: Option<String>,
    /// Client secret for OAuth
    pub client_secret: Option<String>,
    /// Tenant ID for enterprise providers
    pub tenant_id: Option<String>,
    /// Organization ID
    pub organization_id: Option<String>,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Requests per minute
    pub requests_per_minute: u32,
    /// Requests per hour
    pub requests_per_hour: u32,
    /// Maximum concurrent requests
    pub max_concurrent: u32,
    /// Burst limit
    pub burst_limit: u32,
}

/// Enterprise-specific features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnterpriseFeature {
    /// Audit logging
    AuditLogging,
    /// Data encryption at rest
    EncryptionAtRest,
    /// SOC 2 compliance
    SOC2Compliance,
    /// HIPAA compliance
    HIPAACompliance,
    /// Custom enterprise feature
    Custom(String),
}

/// Provider health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProviderHealth {
    /// Provider is healthy
    Healthy,
    /// Provider has issues but is operational
    Degraded,
    /// Provider is unavailable
    Unavailable,
    /// Provider is disabled
    Disabled,
}

/// Provider metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMetrics {
    /// Total requests made
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Error rate (0.0 to 1.0)
    pub error_rate: f64,
    /// Current health status
    pub health: ProviderHealth,
    /// Last health check timestamp
    pub last_health_check: i64,
}

/// Enterprise provider interface
#[async_trait]
pub trait EnterpriseProvider: Send + Sync {
    /// Get provider configuration
    fn config(&self) -> &EnterpriseProviderConfig;

    /// Get current health status
    fn health(&self) -> ProviderHealth;

    /// Get provider metrics
    fn metrics(&self) -> ProviderMetrics;

    /// Test provider connectivity
    async fn test_connectivity(&self) -> IndustryResult<bool>;

    /// Make a request to the provider
    async fn make_request(
        &self,
        endpoint: &str,
        method: &str,
        body: Option<serde_json::Value>,
        headers: Option<HashMap<String, String>>,
    ) -> IndustryResult<serde_json::Value>;

    /// Check if provider supports a specific model
    fn supports_model(&self, model: &str) -> bool;

    /// Get available models
    async fn get_available_models(&self) -> IndustryResult<Vec<String>>;

    /// Get rate limit status
    async fn get_rate_limit_status(&self) -> IndustryResult<RateLimitStatus>;
}

/// Rate limit status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitStatus {
    /// Remaining requests in current window
    pub remaining_requests: u32,
    /// Reset time for rate limit window
    pub reset_time: i64,
    /// Current request count in window
    pub current_count: u32,
}

/// Generic enterprise provider implementation
pub struct GenericEnterpriseProvider {
    config: EnterpriseProviderConfig,
    metrics: RwLock<ProviderMetrics>,
    http_client: reqwest::Client,
}

impl GenericEnterpriseProvider {
    /// Create a new generic enterprise provider
    pub fn new(config: EnterpriseProviderConfig) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        let metrics = ProviderMetrics {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            avg_response_time_ms: 0.0,
            error_rate: 0.0,
            health: ProviderHealth::Healthy,
            last_health_check: chrono::Utc::now().timestamp(),
        };

        Self {
            config,
            metrics: RwLock::new(metrics),
            http_client,
        }
    }

    /// Update provider metrics
    async fn update_metrics(&self, success: bool, response_time_ms: f64) {
        let mut metrics = self.metrics.write().await;
        metrics.total_requests += 1;

        if success {
            metrics.successful_requests += 1;
        } else {
            metrics.failed_requests += 1;
        }

        // Update average response time (simple moving average)
        metrics.avg_response_time_ms = (metrics.avg_response_time_ms + response_time_ms) / 2.0;

        // Update error rate
        metrics.error_rate = metrics.failed_requests as f64 / metrics.total_requests as f64;

        // Update health based on error rate
        metrics.health = if metrics.error_rate > 0.5 {
            ProviderHealth::Unavailable
        } else if metrics.error_rate > 0.2 {
            ProviderHealth::Degraded
        } else {
            ProviderHealth::Healthy
        };

        metrics.last_health_check = chrono::Utc::now().timestamp();
    }

    /// Build authenticated request
    fn build_request(
        &self,
        endpoint: &str,
        method: &str,
        body: Option<serde_json::Value>,
        headers: Option<HashMap<String, String>>,
    ) -> IndustryResult<reqwest::RequestBuilder> {
        let url = format!("{}{}", self.config.base_url.trim_end_matches('/'), endpoint);

        let mut request = match method.to_uppercase().as_str() {
            "GET" => self.http_client.get(&url),
            "POST" => self.http_client.post(&url),
            "PUT" => self.http_client.put(&url),
            "DELETE" => self.http_client.delete(&url),
            "PATCH" => self.http_client.patch(&url),
            _ => return Err(IndustryError::ProviderError {
                provider: self.config.name.clone(),
                message: format!("Unsupported HTTP method: {}", method),
            }),
        };

        // Add authentication headers
        match &self.config.auth_config.method {
            crate::connections::            AuthMethod::ApiKey { header_name, key } => {
                request = request.header(header_name.as_str(), key);
            }
            crate::connections::AuthMethod::OAuth { .. } => {
                // OAuth tokens should be provided in headers parameter
            }
            crate::connections::AuthMethod::Basic { username, password } => {
                use base64::{Engine as _, engine::general_purpose};
                let credentials = general_purpose::STANDARD.encode(format!("{}:{}", username, password));
                request = request.header("Authorization", format!("Basic {}", credentials));
            }
            crate::connections::            AuthMethod::PersonalAccessToken { header_name, token } => {
                request = request.header(header_name.as_str(), token);
            }
        }

        // Add API version header if specified
        if !self.config.api_version.is_empty() {
            request = request.header("api-version", &self.config.api_version);
        }

        // Add custom headers
        if let Some(custom_headers) = headers {
            for (key, value) in custom_headers {
                request = request.header(&key, value);
            }
        }

        // Add JSON body if provided
        if let Some(body) = body {
            request = request.json(&body);
        }

        Ok(request)
    }
}

#[async_trait]
impl EnterpriseProvider for GenericEnterpriseProvider {
    fn config(&self) -> &EnterpriseProviderConfig {
        &self.config
    }

    fn health(&self) -> ProviderHealth {
        futures::executor::block_on(async {
            self.metrics.read().await.health.clone()
        })
    }

    fn metrics(&self) -> ProviderMetrics {
        futures::executor::block_on(async {
            self.metrics.read().await.clone()
        })
    }

    async fn test_connectivity(&self) -> IndustryResult<bool> {
        // Simple connectivity test - try to access a basic endpoint
        let test_endpoint = match self.config.provider_type {
            ProviderType::OpenAI | ProviderType::AzureOpenAI => "/models",
            ProviderType::Anthropic => "/models",
            ProviderType::Google => "/models",
            ProviderType::AWSBedrock => "/models",
            ProviderType::Custom(_) => "/health",
        };

        let result = self.make_request("GET", test_endpoint, None, None).await;
        Ok(result.is_ok())
    }

    async fn make_request(
        &self,
        endpoint: &str,
        method: &str,
        body: Option<serde_json::Value>,
        headers: Option<HashMap<String, String>>,
    ) -> IndustryResult<serde_json::Value> {
        let request = self.build_request(endpoint, method, body, headers)?;

        let start_time = std::time::Instant::now();
        let response = request.send().await.map_err(|e| IndustryError::NetworkError {
            message: format!("Provider request failed: {}", e),
        })?;

        let response_time = start_time.elapsed().as_millis() as f64;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            self.update_metrics(false, response_time).await;

            return Err(IndustryError::ProviderError {
                provider: self.config.name.clone(),
                message: format!("HTTP {}: {}", status, error_text),
            });
        }

        let response_json: serde_json::Value = response.json().await.map_err(|e| {
            IndustryError::SerializationError {
                message: format!("Failed to parse provider response: {}", e),
            }
        })?;

        self.update_metrics(true, response_time).await;
        Ok(response_json)
    }

    fn supports_model(&self, model: &str) -> bool {
        // Basic model support check - in practice, this would query the provider
        match self.config.provider_type {
            ProviderType::OpenAI => model.starts_with("gpt-"),
            ProviderType::Anthropic => model.starts_with("claude-"),
            ProviderType::Google => model.starts_with("gemini-") || model.starts_with("palm-"),
            ProviderType::AzureOpenAI => model.starts_with("gpt-"),
            ProviderType::AWSBedrock => true, // AWS Bedrock supports many models
            ProviderType::Custom(_) => true, // Assume custom providers support all models
        }
    }

    async fn get_available_models(&self) -> IndustryResult<Vec<String>> {
        let response = self.make_request("/models", "GET", None, None).await?;

        // Parse models from response - this is provider-specific
        match self.config.provider_type {
            ProviderType::OpenAI | ProviderType::AzureOpenAI => {
                if let Some(data) = response.get("data") {
                    if let Some(models_array) = data.as_array() {
                        let models: Vec<String> = models_array
                            .iter()
                            .filter_map(|m| m.get("id"))
                            .filter_map(|id| id.as_str())
                            .map(|s| s.to_string())
                            .collect();
                        Ok(models)
                    } else {
                        Ok(vec![])
                    }
                } else {
                    Ok(vec![])
                }
            }
            ProviderType::Anthropic => {
                // Anthropic returns models in a different format
                if let Some(models) = response.as_array() {
                    let model_names: Vec<String> = models
                        .iter()
                        .filter_map(|m| m.get("name"))
                        .filter_map(|name| name.as_str())
                        .map(|s| s.to_string())
                        .collect();
                    Ok(model_names)
                } else {
                    Ok(vec![])
                }
            }
            _ => {
                // For other providers, return a generic response
                Ok(vec!["default-model".to_string()])
            }
        }
    }

    async fn get_rate_limit_status(&self) -> IndustryResult<RateLimitStatus> {
        // This would typically query the provider's rate limit endpoint
        // For now, return a mock status
        Ok(RateLimitStatus {
            remaining_requests: self.config.rate_limit.requests_per_minute,
            reset_time: chrono::Utc::now().timestamp() + 60, // Reset in 1 minute
            current_count: 0,
        })
    }
}

/// Provider manager for managing multiple enterprise providers
pub struct ProviderManager {
    providers: RwLock<HashMap<String, Box<dyn EnterpriseProvider>>>,
}

impl ProviderManager {
    /// Create a new provider manager
    pub fn new() -> Self {
        Self {
            providers: RwLock::new(HashMap::new()),
        }
    }

    /// Add a provider
    pub async fn add_provider(&self, provider: Box<dyn EnterpriseProvider>) -> IndustryResult<()> {
        let name = provider.config().name.clone();
        self.providers.write().await.insert(name, provider);
        Ok(())
    }

    /// Get a provider by name (removes it from the manager)
    pub async fn take_provider(&self, name: &str) -> Option<Box<dyn EnterpriseProvider>> {
        self.providers.write().await.remove(name)
    }

    /// Remove a provider
    pub async fn remove_provider(&self, name: &str) -> IndustryResult<()> {
        self.providers.write().await.remove(name);
        Ok(())
    }

    /// List all providers
    pub async fn list_providers(&self) -> Vec<String> {
        self.providers.read().await.keys().cloned().collect()
    }

    /// Test all providers
    pub async fn test_all_providers(&self) -> IndustryResult<HashMap<String, bool>> {
        let mut results = HashMap::new();
        let providers = self.providers.read().await;

        for (name, provider) in providers.iter() {
            let result = provider.test_connectivity().await.unwrap_or(false);
            results.insert(name.clone(), result);
        }

        Ok(results)
    }

    /// Get provider health overview
    pub async fn get_health_overview(&self) -> HashMap<String, ProviderHealth> {
        let providers = self.providers.read().await;
        let mut overview = HashMap::new();

        for (name, provider) in providers.iter() {
            overview.insert(name.clone(), provider.health());
        }

        overview
    }
}

impl Default for ProviderManager {
    fn default() -> Self {
        Self::new()
    }
}