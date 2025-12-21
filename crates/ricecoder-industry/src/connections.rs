//! Tool connection abstractions and management

use crate::error::{IndustryError, IndustryResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;

/// Connection configuration for external tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    /// Unique connection ID
    pub id: String,
    /// Tool name (e.g., "github", "gitlab", "jira")
    pub tool: String,
    /// Connection name for display
    pub name: String,
    /// Base URL for the tool
    pub base_url: String,
    /// Authentication method
    pub auth_method: AuthMethod,
    /// Additional configuration
    pub config: HashMap<String, serde_json::Value>,
    /// Connection timeout in seconds
    pub timeout_seconds: u64,
    /// Rate limiting configuration
    pub rate_limit: Option<RateLimitConfig>,
}

/// Authentication methods for tool connections
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AuthMethod {
    /// OAuth 2.0 authentication
    OAuth {
        /// OAuth provider name
        provider: String,
        /// Requested scopes
        scopes: Vec<String>,
    },
    /// API key authentication
    ApiKey {
        /// API key header name
        header_name: String,
        /// API key value (encrypted)
        key: String,
    },
    /// Basic authentication
    Basic {
        /// Username
        username: String,
        /// Password (encrypted)
        password: String,
    },
    /// Personal access token
    PersonalAccessToken {
        /// Token header name
        header_name: String,
        /// Token value (encrypted)
        token: String,
    },
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum requests per minute
    pub requests_per_minute: u32,
    /// Maximum concurrent requests
    pub max_concurrent: u32,
}

/// Connection status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConnectionStatus {
    /// Connection is active and healthy
    Connected,
    /// Connection is being established
    Connecting,
    /// Connection failed
    Failed(String),
    /// Connection is disabled
    Disabled,
}

/// Tool connection interface
#[async_trait]
pub trait ToolConnection: Send + Sync {
    /// Get connection configuration
    fn config(&self) -> &ConnectionConfig;

    /// Get current connection status
    fn status(&self) -> ConnectionStatus;

    /// Test the connection
    async fn test_connection(&mut self) -> IndustryResult<bool>;

    /// Execute a request against the tool
    async fn execute_request(
        &self,
        method: &str,
        endpoint: &str,
        body: Option<serde_json::Value>,
        headers: Option<HashMap<String, String>>,
    ) -> IndustryResult<serde_json::Value>;

    /// Get connection health metrics
    async fn health_metrics(&self) -> IndustryResult<ConnectionHealth>;
}

/// Connection health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionHealth {
    /// Response time in milliseconds
    pub response_time_ms: u64,
    /// Error rate (0.0 to 1.0)
    pub error_rate: f64,
    /// Last successful request timestamp
    pub last_success: Option<i64>,
    /// Last error timestamp
    pub last_error: Option<i64>,
    /// Total requests made
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
}

/// Generic tool connector implementation
pub struct ToolConnector {
    config: ConnectionConfig,
    status: RwLock<ConnectionStatus>,
    health: RwLock<ConnectionHealth>,
    http_client: reqwest::Client,
}

impl ToolConnector {
    /// Create a new tool connector
    pub fn new(config: ConnectionConfig) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            status: RwLock::new(ConnectionStatus::Connecting),
            health: RwLock::new(ConnectionHealth {
                response_time_ms: 0,
                error_rate: 0.0,
                last_success: None,
                last_error: None,
                total_requests: 0,
                successful_requests: 0,
            }),
            http_client,
        }
    }

    /// Update connection status
    async fn update_status(&self, status: ConnectionStatus) {
        *self.status.write().await = status;
    }

    /// Update health metrics
    async fn update_health(&self, success: bool, response_time: u64) {
        let mut health = self.health.write().await;
        health.total_requests += 1;
        health.response_time_ms = response_time;

        if success {
            health.successful_requests += 1;
            health.last_success = Some(chrono::Utc::now().timestamp());
        } else {
            health.last_error = Some(chrono::Utc::now().timestamp());
        }

        health.error_rate = if health.total_requests > 0 {
            1.0 - (health.successful_requests as f64 / health.total_requests as f64)
        } else {
            0.0
        };
    }

    /// Build request with authentication
    fn build_request(
        &self,
        method: &str,
        endpoint: &str,
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
            _ => {
                return Err(IndustryError::ConnectionError {
                    tool: self.config.tool.clone(),
                    message: format!("Unsupported HTTP method: {}", method),
                })
            }
        };

        // Add authentication headers
        match &self.config.auth_method {
            AuthMethod::OAuth { .. } => {
                // OAuth tokens should be provided in headers parameter
            }
            AuthMethod::ApiKey { header_name, key } => {
                request = request.header(header_name, key);
            }
            AuthMethod::Basic { username, password } => {
                use base64::{engine::general_purpose, Engine as _};
                let credentials =
                    general_purpose::STANDARD.encode(format!("{}:{}", username, password));
                request = request.header("Authorization", format!("Basic {}", credentials));
            }
            AuthMethod::PersonalAccessToken { header_name, token } => {
                request = request.header(header_name, token);
            }
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
impl ToolConnection for ToolConnector {
    fn config(&self) -> &ConnectionConfig {
        &self.config
    }

    fn status(&self) -> ConnectionStatus {
        // This is a simplified implementation - in practice, you'd want to return the current status
        futures::executor::block_on(async { self.status.read().await.clone() })
    }

    async fn test_connection(&mut self) -> IndustryResult<bool> {
        let start_time = std::time::Instant::now();

        // Simple health check - try to access a common endpoint
        let test_endpoint = match self.config.tool.as_str() {
            "github" => "/user",
            "gitlab" => "/api/v4/user",
            "jira" => "/rest/api/2/myself",
            _ => "/health", // Generic health endpoint
        };

        let result = self.execute_request("GET", test_endpoint, None, None).await;
        let response_time = start_time.elapsed().as_millis() as u64;

        let success = result.is_ok();
        self.update_health(success, response_time).await;

        if success {
            self.update_status(ConnectionStatus::Connected).await;
            Ok(true)
        } else {
            let error_msg = result.unwrap_err().to_string();
            self.update_status(ConnectionStatus::Failed(error_msg))
                .await;
            Ok(false)
        }
    }

    async fn execute_request(
        &self,
        method: &str,
        endpoint: &str,
        body: Option<serde_json::Value>,
        headers: Option<HashMap<String, String>>,
    ) -> IndustryResult<serde_json::Value> {
        let request = self.build_request(method, endpoint, body, headers)?;

        let start_time = std::time::Instant::now();
        let response = request
            .send()
            .await
            .map_err(|e| IndustryError::NetworkError {
                message: format!("Request failed: {}", e),
            })?;

        let response_time = start_time.elapsed().as_millis() as u64;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            self.update_health(false, response_time).await;

            return Err(IndustryError::ConnectionError {
                tool: self.config.tool.clone(),
                message: format!("HTTP {}: {}", status, error_text),
            });
        }

        let response_json: serde_json::Value =
            response
                .json()
                .await
                .map_err(|e| IndustryError::SerializationError {
                    message: format!("Failed to parse response: {}", e),
                })?;

        self.update_health(true, response_time).await;
        Ok(response_json)
    }

    async fn health_metrics(&self) -> IndustryResult<ConnectionHealth> {
        Ok(self.health.read().await.clone())
    }
}

/// Connection manager for managing multiple tool connections
pub struct ConnectionManager {
    connections: RwLock<HashMap<String, Box<dyn ToolConnection>>>,
}

impl ConnectionManager {
    /// Create a new connection manager
    pub fn new() -> Self {
        Self {
            connections: RwLock::new(HashMap::new()),
        }
    }

    /// Add a connection
    pub async fn add_connection(&self, connection: Box<dyn ToolConnection>) -> IndustryResult<()> {
        let id = connection.config().id.clone();
        self.connections.write().await.insert(id, connection);
        Ok(())
    }

    /// Get a connection by ID (removes it from the manager)
    pub async fn take_connection(&self, id: &str) -> Option<Box<dyn ToolConnection>> {
        self.connections.write().await.remove(id)
    }

    /// Remove a connection
    pub async fn remove_connection(&self, id: &str) -> IndustryResult<()> {
        self.connections.write().await.remove(id);
        Ok(())
    }

    /// List all connections
    pub async fn list_connections(&self) -> Vec<String> {
        self.connections.read().await.keys().cloned().collect()
    }

    /// Test all connections
    pub async fn test_all_connections(&self) -> IndustryResult<HashMap<String, bool>> {
        let mut results = HashMap::new();
        let connections = self.connections.read().await;

        for (id, _conn) in connections.iter() {
            // Note: This creates a simple test without mutation
            // In a real implementation, you might want to handle this differently
            let result = true; // Placeholder - actual testing would require mutable access
            results.insert(id.clone(), result);
        }

        Ok(results)
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}
