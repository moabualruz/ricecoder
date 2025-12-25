//! HTTP client port interfaces
//!
//! This module defines the contracts for HTTP client operations.
//! Implementations in infrastructure crates provide concrete HTTP
//! clients (reqwest, hyper, etc.).

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use crate::errors::*;

// ============================================================================
// HTTP Value Objects
// ============================================================================

/// HTTP method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::Get => write!(f, "GET"),
            HttpMethod::Post => write!(f, "POST"),
            HttpMethod::Put => write!(f, "PUT"),
            HttpMethod::Patch => write!(f, "PATCH"),
            HttpMethod::Delete => write!(f, "DELETE"),
            HttpMethod::Head => write!(f, "HEAD"),
            HttpMethod::Options => write!(f, "OPTIONS"),
        }
    }
}

/// HTTP request configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequest {
    /// Request URL
    pub url: String,
    /// HTTP method
    pub method: HttpMethod,
    /// Request headers
    pub headers: HashMap<String, String>,
    /// Request body (optional)
    pub body: Option<Vec<u8>>,
    /// Request timeout
    pub timeout: Option<Duration>,
    /// Follow redirects
    pub follow_redirects: bool,
}

impl HttpRequest {
    /// Create a new GET request
    pub fn get(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            method: HttpMethod::Get,
            headers: HashMap::new(),
            body: None,
            timeout: None,
            follow_redirects: true,
        }
    }

    /// Create a new POST request
    pub fn post(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            method: HttpMethod::Post,
            headers: HashMap::new(),
            body: None,
            timeout: None,
            follow_redirects: true,
        }
    }

    /// Add a header
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Set JSON body
    pub fn json<T: Serialize>(mut self, body: &T) -> DomainResult<Self> {
        let json = serde_json::to_vec(body).map_err(|e| DomainError::ValidationError {
            field: "body".to_string(),
            reason: format!("Failed to serialize JSON: {}", e),
        })?;
        self.body = Some(json);
        self.headers
            .insert("Content-Type".to_string(), "application/json".to_string());
        Ok(self)
    }

    /// Set raw body
    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }

    /// Set timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

/// HTTP response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    /// Response status code
    pub status: u16,
    /// Response headers
    pub headers: HashMap<String, String>,
    /// Response body
    pub body: Vec<u8>,
    /// Response time in milliseconds
    pub response_time_ms: u64,
}

impl HttpResponse {
    /// Check if response is successful (2xx)
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }

    /// Check if response is client error (4xx)
    pub fn is_client_error(&self) -> bool {
        (400..500).contains(&self.status)
    }

    /// Check if response is server error (5xx)
    pub fn is_server_error(&self) -> bool {
        (500..600).contains(&self.status)
    }

    /// Get body as UTF-8 string
    pub fn text(&self) -> DomainResult<String> {
        String::from_utf8(self.body.clone()).map_err(|e| DomainError::ValidationError {
            field: "body".to_string(),
            reason: format!("Response body is not valid UTF-8: {}", e),
        })
    }

    /// Parse body as JSON
    pub fn json<T: for<'de> Deserialize<'de>>(&self) -> DomainResult<T> {
        serde_json::from_slice(&self.body).map_err(|e| DomainError::ValidationError {
            field: "body".to_string(),
            reason: format!("Failed to parse JSON response: {}", e),
        })
    }
}

// ============================================================================
// HTTP Client Ports (ISP-Compliant)
// ============================================================================

/// HTTP client for making requests (ISP: 5 methods max)
#[async_trait]
pub trait HttpClient: Send + Sync {
    /// Execute an HTTP request
    async fn execute(&self, request: HttpRequest) -> DomainResult<HttpResponse>;

    /// Convenience method for GET requests
    async fn get(&self, url: &str) -> DomainResult<HttpResponse> {
        self.execute(HttpRequest::get(url)).await
    }

    /// Convenience method for POST requests with JSON body
    async fn post_json<T: Serialize + Send + Sync>(
        &self,
        url: &str,
        body: &T,
    ) -> DomainResult<HttpResponse> {
        self.execute(HttpRequest::post(url).json(body)?).await
    }

    /// Check if a URL is reachable (HEAD request)
    async fn is_reachable(&self, url: &str) -> DomainResult<bool> {
        let request = HttpRequest {
            url: url.to_string(),
            method: HttpMethod::Head,
            headers: HashMap::new(),
            body: None,
            timeout: Some(Duration::from_secs(5)),
            follow_redirects: true,
        };
        match self.execute(request).await {
            Ok(response) => Ok(response.is_success()),
            Err(_) => Ok(false),
        }
    }

    /// Get client configuration info
    fn client_info(&self) -> HttpClientInfo;
}

/// HTTP client metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpClientInfo {
    /// Client name/type (e.g., "reqwest", "hyper")
    pub client_type: String,
    /// Default timeout
    pub default_timeout: Duration,
    /// Whether client supports HTTP/2
    pub supports_http2: bool,
    /// Whether client has connection pooling
    pub connection_pooling: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_request_builder() {
        let request = HttpRequest::get("https://example.com")
            .header("Accept", "application/json")
            .timeout(Duration::from_secs(30));

        assert_eq!(request.url, "https://example.com");
        assert_eq!(request.method, HttpMethod::Get);
        assert_eq!(
            request.headers.get("Accept"),
            Some(&"application/json".to_string())
        );
        assert_eq!(request.timeout, Some(Duration::from_secs(30)));
    }

    #[test]
    fn test_http_response_status_checks() {
        let success = HttpResponse {
            status: 200,
            headers: HashMap::new(),
            body: vec![],
            response_time_ms: 100,
        };
        assert!(success.is_success());
        assert!(!success.is_client_error());
        assert!(!success.is_server_error());

        let client_error = HttpResponse {
            status: 404,
            headers: HashMap::new(),
            body: vec![],
            response_time_ms: 50,
        };
        assert!(!client_error.is_success());
        assert!(client_error.is_client_error());

        let server_error = HttpResponse {
            status: 500,
            headers: HashMap::new(),
            body: vec![],
            response_time_ms: 200,
        };
        assert!(!server_error.is_success());
        assert!(server_error.is_server_error());
    }

    #[test]
    fn test_http_method_display() {
        assert_eq!(format!("{}", HttpMethod::Get), "GET");
        assert_eq!(format!("{}", HttpMethod::Post), "POST");
        assert_eq!(format!("{}", HttpMethod::Delete), "DELETE");
    }
}
