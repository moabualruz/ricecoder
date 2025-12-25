//! HTTP client implementation

use std::sync::Arc;

use async_trait::async_trait;
use reqwest::{header::HeaderMap, Body, Method, Response};
use tracing::debug;

use crate::{
    config::HttpConfig,
    error::{HttpError, Result},
    middleware::{RetryConfig, RetryMiddleware},
};

/// Mockable HTTP client trait
#[async_trait]
pub trait HttpClientTrait: Send + Sync {
    /// Execute a GET request
    async fn get(&self, url: &str) -> Result<Response>;

    /// Execute a POST request
    async fn post(&self, url: &str, body: Body) -> Result<Response>;

    /// Execute a PUT request
    async fn put(&self, url: &str, body: Body) -> Result<Response>;

    /// Execute a DELETE request
    async fn delete(&self, url: &str) -> Result<Response>;

    /// Execute a custom HTTP request
    async fn request(&self, method: Method, url: &str, body: Option<Body>) -> Result<Response>;
}

/// Production HTTP client
pub struct HttpClient {
    inner: reqwest::Client,
    config: HttpConfig,
    retry: RetryMiddleware,
}

impl HttpClient {
    /// Create a new HTTP client with configuration
    pub fn new(config: HttpConfig) -> Result<Self> {
        let mut builder = reqwest::Client::builder()
            .timeout(config.timeout)
            .connect_timeout(config.connect_timeout)
            .user_agent(&config.user_agent)
            .redirect(if config.max_redirects > 0 {
                reqwest::redirect::Policy::limited(config.max_redirects)
            } else {
                reqwest::redirect::Policy::none()
            });

        // Configure proxy if provided
        if let Some(proxy_url) = &config.proxy {
            let proxy = reqwest::Proxy::all(proxy_url)
                .map_err(|e| HttpError::InvalidProxy(e.to_string()))?;
            builder = builder.proxy(proxy);
        }

        // Configure connection pooling
        if config.pool_enabled {
            builder = builder.pool_idle_timeout(config.pool_idle_timeout);
        } else {
            builder = builder.pool_max_idle_per_host(0);
        }

        let inner = builder
            .build()
            .map_err(|e| HttpError::BuildError(e.to_string()))?;

        let retry_config = RetryConfig {
            max_attempts: config.retry_count,
            initial_delay: config.retry_delay,
            ..Default::default()
        };

        Ok(Self {
            inner,
            config,
            retry: RetryMiddleware::new(retry_config),
        })
    }

    /// Create HTTP client with default configuration
    pub fn with_defaults() -> Result<Self> {
        Self::new(HttpConfig::default())
    }

    /// Get underlying reqwest client (for advanced usage)
    pub fn inner(&self) -> &reqwest::Client {
        &self.inner
    }

    /// Get configuration
    pub fn config(&self) -> &HttpConfig {
        &self.config
    }
}

#[async_trait]
impl HttpClientTrait for HttpClient {
    async fn get(&self, url: &str) -> Result<Response> {
        debug!("HTTP GET: {}", url);
        self.request(Method::GET, url, None).await
    }

    async fn post(&self, url: &str, body: Body) -> Result<Response> {
        debug!("HTTP POST: {}", url);
        self.request(Method::POST, url, Some(body)).await
    }

    async fn put(&self, url: &str, body: Body) -> Result<Response> {
        debug!("HTTP PUT: {}", url);
        self.request(Method::PUT, url, Some(body)).await
    }

    async fn delete(&self, url: &str) -> Result<Response> {
        debug!("HTTP DELETE: {}", url);
        self.request(Method::DELETE, url, None).await
    }

    async fn request(&self, method: Method, url: &str, body: Option<Body>) -> Result<Response> {
        let url = url
            .parse::<url::Url>()
            .map_err(|e| HttpError::InvalidUrl(e.to_string()))?;

        // Note: We cannot retry with a body because reqwest::Body is not Clone
        // For retries with bodies, callers should handle retry logic externally
        let mut request = self.inner.request(method.clone(), url.clone());

        if let Some(body) = body {
            request = request.body(body);
        }

        let response = request.send().await.map_err(HttpError::RequestFailed)?;

        // Check for HTTP error status
        if !response.status().is_success() {
            return Err(HttpError::HttpStatus {
                status: response.status(),
                message: response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string()),
            });
        }

        Ok(response)
    }
}

/// Create a shared HTTP client (Arc-wrapped for cloning)
pub fn shared_client(config: HttpConfig) -> Result<Arc<dyn HttpClientTrait>> {
    Ok(Arc::new(HttpClient::new(config)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_client_creation_with_defaults() {
        let client = HttpClient::with_defaults();
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_creation_with_config() {
        let config = HttpConfig {
            timeout: Duration::from_secs(10),
            retry_count: 2,
            ..Default::default()
        };

        let client = HttpClient::new(config);
        assert!(client.is_ok());

        let client = client.unwrap();
        assert_eq!(client.config().timeout, Duration::from_secs(10));
        assert_eq!(client.config().retry_count, 2);
    }

    #[test]
    fn test_client_with_proxy() {
        let config = HttpConfig::default().with_proxy("http://proxy.example.com:8080");

        let client = HttpClient::new(config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_invalid_proxy() {
        let config = HttpConfig::default().with_proxy("invalid-proxy");

        let result = HttpClient::new(config);
        assert!(matches!(result, Err(HttpError::InvalidProxy(_))));
    }

    #[tokio::test]
    async fn test_get_invalid_url() {
        let client = HttpClient::with_defaults().unwrap();
        let result = client.get("not a url").await;
        assert!(matches!(result, Err(HttpError::InvalidUrl(_))));
    }

    #[test]
    fn test_shared_client_creation() {
        let config = HttpConfig::default();
        let client = shared_client(config);
        assert!(client.is_ok());
    }
}
