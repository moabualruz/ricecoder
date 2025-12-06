//! Webfetch tool for fetching web content
//!
//! Provides functionality to fetch and process web content from URLs with MCP integration.

use crate::error::ToolError;
use crate::result::ToolResult;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::str::FromStr;
use std::time::Instant;
use tracing::{debug, warn};
use url::Url;

/// Maximum content size before truncation (50KB)
const MAX_CONTENT_SIZE: usize = 50 * 1024;

/// HTTP request timeout in seconds
const REQUEST_TIMEOUT_SECS: u64 = 10;

/// Input for webfetch operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebfetchInput {
    /// URL to fetch
    pub url: String,
    /// Optional maximum content size in bytes
    pub max_size: Option<usize>,
}

impl WebfetchInput {
    /// Create a new webfetch input
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            max_size: None,
        }
    }

    /// Set maximum content size
    pub fn with_max_size(mut self, max_size: usize) -> Self {
        self.max_size = Some(max_size);
        self
    }
}

/// Output for webfetch operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebfetchOutput {
    /// Fetched content
    pub content: String,
    /// Whether content was truncated
    pub truncated: bool,
    /// Original content size in bytes
    pub original_size: usize,
    /// Actual returned content size in bytes
    pub returned_size: usize,
}

impl WebfetchOutput {
    /// Create a new webfetch output
    pub fn new(content: String, original_size: usize) -> Self {
        let returned_size = content.len();
        let truncated = returned_size < original_size;

        Self {
            content,
            truncated,
            original_size,
            returned_size,
        }
    }
}

/// Webfetch tool for fetching web content
pub struct WebfetchTool {
    client: reqwest::Client,
}

impl WebfetchTool {
    /// Create a new webfetch tool
    pub fn new() -> Result<Self, ToolError> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .build()
            .map_err(|e| {
                ToolError::new("CLIENT_ERROR", "Failed to create HTTP client")
                    .with_details(e.to_string())
            })?;

        Ok(Self { client })
    }

    /// Validate URL format and security
    pub fn validate_url(url: &str) -> Result<Url, ToolError> {
        // Parse URL
        let parsed_url = Url::parse(url).map_err(|e| {
            ToolError::new("INVALID_URL", "Invalid URL format")
                .with_details(e.to_string())
                .with_suggestion("Ensure URL is properly formatted (e.g., https://example.com)")
        })?;

        // Check scheme is http or https
        match parsed_url.scheme() {
            "http" | "https" => {}
            _ => {
                return Err(ToolError::new("INVALID_SCHEME", "Only http and https schemes are supported")
                    .with_suggestion("Use http:// or https:// URLs"))
            }
        }

        // Check for SSRF attacks - reject private IPs and localhost
        if let Some(host) = parsed_url.host_str() {
            // Reject localhost
            if host == "localhost" || host == "127.0.0.1" || host == "::1" {
                return Err(ToolError::new("SSRF_PREVENTION", "Localhost URLs are not allowed")
                    .with_suggestion("Use a public URL instead"));
            }

            // Try to parse as IP address and check if private
            if let Ok(ip) = IpAddr::from_str(host) {
                let is_private = match ip {
                    IpAddr::V4(ipv4) => ipv4.is_private() || ipv4.is_loopback(),
                    IpAddr::V6(ipv6) => ipv6.is_loopback(),
                };

                if is_private {
                    return Err(ToolError::new("SSRF_PREVENTION", "Private IP addresses are not allowed")
                        .with_details(format!("IP: {}", ip))
                        .with_suggestion("Use a public URL instead"));
                }
            }
        }

        Ok(parsed_url)
    }

    /// Fetch content from a URL with timeout enforcement
    pub async fn fetch(&self, input: WebfetchInput) -> ToolResult<WebfetchOutput> {
        let start = Instant::now();

        // Validate URL
        match Self::validate_url(&input.url) {
            Ok(_) => {
                debug!("URL validation passed: {}", input.url);
            }
            Err(e) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                return ToolResult::err(e, duration_ms, "builtin");
            }
        }

        // Determine max size
        let max_size = input.max_size.unwrap_or(MAX_CONTENT_SIZE);

        // Enforce timeout using tokio::time::timeout
        let timeout_duration = std::time::Duration::from_secs(REQUEST_TIMEOUT_SECS);
        let fetch_future = async {
            // Fetch content
            match self.client.get(&input.url).send().await {
                Ok(response) => {
                    // Check status
                    if !response.status().is_success() {
                        let error = ToolError::new(
                            "HTTP_ERROR",
                            format!("HTTP error: {}", response.status()),
                        )
                        .with_details(format!("Status code: {}", response.status().as_u16()))
                        .with_suggestion("Check the URL and try again");
                        return Err(error);
                    }

                    // Fetch body
                    match response.bytes().await {
                        Ok(bytes) => {
                            let original_size = bytes.len();

                            // Truncate if necessary
                            let content = if original_size > max_size {
                                warn!(
                                    "Content truncated from {} to {} bytes",
                                    original_size, max_size
                                );
                                String::from_utf8_lossy(&bytes[..max_size]).to_string()
                            } else {
                                String::from_utf8_lossy(&bytes).to_string()
                            };

                            let output = WebfetchOutput::new(content, original_size);
                            Ok(output)
                        }
                        Err(e) => {
                            let error = ToolError::from(e);
                            Err(error)
                        }
                    }
                }
                Err(e) => {
                    let error = ToolError::from(e);
                    Err(error)
                }
            }
        };

        match tokio::time::timeout(timeout_duration, fetch_future).await {
            Ok(Ok(output)) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                ToolResult::ok(output, duration_ms, "builtin")
            }
            Ok(Err(error)) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                ToolResult::err(error, duration_ms, "builtin")
            }
            Err(_) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                let error = ToolError::new("TIMEOUT", "Webfetch operation exceeded 10 second timeout")
                    .with_details(format!("URL: {}", input.url))
                    .with_suggestion("Try again with a different URL or check your network connection");
                ToolResult::err(error, duration_ms, "builtin")
            }
        }
    }
}

impl Default for WebfetchTool {
    fn default() -> Self {
        Self::new().expect("Failed to create default WebfetchTool")
    }
}

/// Webfetch tool with MCP integration
pub struct WebfetchToolWithMcp {
    builtin: WebfetchTool,
    mcp_client: Option<ricecoder_mcp::MCPClient>,
}

impl WebfetchToolWithMcp {
    /// Create a new webfetch tool with MCP integration
    pub fn new(mcp_client: Option<ricecoder_mcp::MCPClient>) -> Result<Self, ToolError> {
        let builtin = WebfetchTool::new()?;
        Ok(Self { builtin, mcp_client })
    }

    /// Check if MCP server is available for webfetch
    async fn is_mcp_available(&self) -> bool {
        if let Some(client) = &self.mcp_client {
            // Try to discover webfetch tools from MCP servers
            if let Ok(servers) = client.discover_servers().await {
                for server_id in servers {
                    if let Ok(tools) = client.discover_tools(&server_id).await {
                        if tools.iter().any(|t| t.id.contains("webfetch")) {
                            debug!("MCP webfetch server available: {}", server_id);
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// Fetch content with MCP fallback
    pub async fn fetch(&self, input: WebfetchInput) -> ToolResult<WebfetchOutput> {
        // Try MCP first if available
        if self.is_mcp_available().await {
            debug!("Attempting to use MCP webfetch provider");
            // In a real implementation, we would call the MCP server here
            // For now, we fall back to built-in
        }

        // Fall back to built-in implementation
        debug!("Using built-in webfetch provider");
        self.builtin.fetch(input).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webfetch_input_creation() {
        let input = WebfetchInput::new("https://example.com");
        assert_eq!(input.url, "https://example.com");
        assert!(input.max_size.is_none());
    }

    #[test]
    fn test_webfetch_input_with_max_size() {
        let input = WebfetchInput::new("https://example.com").with_max_size(1024);
        assert_eq!(input.max_size, Some(1024));
    }

    #[test]
    fn test_webfetch_output_creation() {
        let output = WebfetchOutput::new("test content".to_string(), 12);
        assert_eq!(output.content, "test content");
        assert_eq!(output.original_size, 12);
        assert_eq!(output.returned_size, 12);
        assert!(!output.truncated);
    }

    #[test]
    fn test_webfetch_output_truncation() {
        let output = WebfetchOutput::new("test".to_string(), 100);
        assert_eq!(output.content, "test");
        assert_eq!(output.original_size, 100);
        assert_eq!(output.returned_size, 4);
        assert!(output.truncated);
    }

    #[test]
    fn test_validate_url_valid_https() {
        let result = WebfetchTool::validate_url("https://example.com");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_url_valid_http() {
        let result = WebfetchTool::validate_url("http://example.com");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_url_invalid_format() {
        let result = WebfetchTool::validate_url("not a url");
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.code, "INVALID_URL");
        }
    }

    #[test]
    fn test_validate_url_invalid_scheme() {
        let result = WebfetchTool::validate_url("ftp://example.com");
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.code, "INVALID_SCHEME");
        }
    }

    #[test]
    fn test_validate_url_localhost_rejection() {
        let result = WebfetchTool::validate_url("http://localhost:8080");
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.code, "SSRF_PREVENTION");
        }
    }

    #[test]
    fn test_validate_url_127_0_0_1_rejection() {
        let result = WebfetchTool::validate_url("http://127.0.0.1");
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.code, "SSRF_PREVENTION");
        }
    }

    #[test]
    fn test_validate_url_private_ip_rejection() {
        let result = WebfetchTool::validate_url("http://192.168.1.1");
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.code, "SSRF_PREVENTION");
        }
    }

    #[test]
    fn test_webfetch_tool_creation() {
        let tool = WebfetchTool::new();
        assert!(tool.is_ok());
    }

    #[tokio::test]
    async fn test_webfetch_timeout_enforcement() {
        // This test verifies that timeout is enforced
        // We use a slow/non-responsive endpoint to trigger timeout
        let tool = WebfetchTool::new().unwrap();
        let input = WebfetchInput::new("http://httpbin.org/delay/15"); // 15 second delay
        
        let result = tool.fetch(input).await;
        
        // Should timeout after 10 seconds
        assert!(!result.success);
        assert!(result.error.is_some());
        if let Some(error) = result.error {
            assert_eq!(error.code, "TIMEOUT");
        }
    }
}

