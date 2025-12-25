//! Webfetch tool for fetching web content
//!
//! Provides functionality to fetch and process web content from URLs with MCP integration.
//! Features:
//! - URL fetching with SSRF protection
//! - HTML to markdown conversion
//! - LRU cache with 15-minute TTL
//! - Configurable output format (text/markdown/html)
//! - 120 second timeout

use std::{
    collections::HashMap,
    net::IpAddr,
    str::FromStr,
    sync::Arc,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use url::Url;

use crate::{error::ToolError, result::ToolResult};

/// Maximum content size before truncation (50KB)
const MAX_CONTENT_SIZE: usize = 50 * 1024;

/// HTTP request timeout in seconds (max 120s)
const REQUEST_TIMEOUT_SECS: u64 = 120;

/// Default timeout if not specified
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Cache TTL in seconds (15 minutes)
const CACHE_TTL_SECS: u64 = 15 * 60;

/// Maximum cache entries
const MAX_CACHE_ENTRIES: usize = 100;

/// Output format for fetched content
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    /// Plain text (strip HTML tags)
    #[default]
    Text,
    /// Convert HTML to markdown
    Markdown,
    /// Return raw HTML
    Html,
}

/// Input for webfetch operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebfetchInput {
    /// URL to fetch
    pub url: String,
    /// Output format (text, markdown, html) - default: text
    #[serde(default)]
    pub format: OutputFormat,
    /// Optional timeout in seconds (max 120s)
    pub timeout: Option<u64>,
    /// Optional maximum content size in bytes
    pub max_size: Option<usize>,
}

impl WebfetchInput {
    /// Create a new webfetch input
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            format: OutputFormat::default(),
            timeout: None,
            max_size: None,
        }
    }

    /// Set output format
    pub fn with_format(mut self, format: OutputFormat) -> Self {
        self.format = format;
        self
    }

    /// Set timeout in seconds (capped at 120s)
    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = Some(timeout.min(REQUEST_TIMEOUT_SECS));
        self
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
    /// Whether result was served from cache
    #[serde(default)]
    pub from_cache: bool,
    /// Output format used
    #[serde(default)]
    pub format: OutputFormat,
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
            from_cache: false,
            format: OutputFormat::default(),
        }
    }

    /// Mark as served from cache
    pub fn from_cache(mut self) -> Self {
        self.from_cache = true;
        self
    }

    /// Set output format
    pub fn with_format(mut self, format: OutputFormat) -> Self {
        self.format = format;
        self
    }
}

/// Cache entry with TTL
#[derive(Debug, Clone)]
struct CacheEntry {
    /// Cached content (raw HTML)
    content: String,
    /// Original size
    original_size: usize,
    /// When this entry was cached
    cached_at: Instant,
}

impl CacheEntry {
    fn new(content: String, original_size: usize) -> Self {
        Self {
            content,
            original_size,
            cached_at: Instant::now(),
        }
    }

    fn is_expired(&self) -> bool {
        self.cached_at.elapsed() > Duration::from_secs(CACHE_TTL_SECS)
    }
}

/// LRU Cache for webfetch results
/// Uses a simple HashMap with TTL-based eviction
#[derive(Debug)]
pub struct WebfetchCache {
    entries: RwLock<HashMap<String, CacheEntry>>,
    max_entries: usize,
}

impl WebfetchCache {
    /// Create a new cache with default settings
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            max_entries: MAX_CACHE_ENTRIES,
        }
    }

    /// Get cached content if available and not expired
    pub async fn get(&self, url: &str) -> Option<(String, usize)> {
        let entries = self.entries.read().await;
        if let Some(entry) = entries.get(url) {
            if !entry.is_expired() {
                debug!("Cache hit for URL: {}", url);
                return Some((entry.content.clone(), entry.original_size));
            }
        }
        None
    }

    /// Store content in cache
    pub async fn set(&self, url: String, content: String, original_size: usize) {
        let mut entries = self.entries.write().await;
        
        // Evict expired entries and enforce max size
        entries.retain(|_, v| !v.is_expired());
        
        // If still at capacity, remove oldest entry
        if entries.len() >= self.max_entries {
            if let Some(oldest_key) = entries
                .iter()
                .min_by_key(|(_, v)| v.cached_at)
                .map(|(k, _)| k.clone())
            {
                entries.remove(&oldest_key);
            }
        }
        
        entries.insert(url, CacheEntry::new(content, original_size));
    }

    /// Clear all cache entries
    pub async fn clear(&self) {
        let mut entries = self.entries.write().await;
        entries.clear();
    }
}

impl Default for WebfetchCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert HTML to plain text (strip tags)
fn html_to_text(html: &str) -> String {
    // Simple HTML tag stripping - removes all <...> tags
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut in_script = false;
    let mut in_style = false;
    
    let html_lower = html.to_lowercase();
    let chars: Vec<char> = html.chars().collect();
    let lower_chars: Vec<char> = html_lower.chars().collect();
    
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        
        // Check for script/style start
        if i + 7 < chars.len() {
            let slice: String = lower_chars[i..i+7].iter().collect();
            if slice == "<script" {
                in_script = true;
            } else if slice == "</scrip" {
                in_script = false;
                // Skip to end of tag
                while i < chars.len() && chars[i] != '>' {
                    i += 1;
                }
                i += 1;
                continue;
            }
        }
        if i + 6 < chars.len() {
            let slice: String = lower_chars[i..i+6].iter().collect();
            if slice == "<style" {
                in_style = true;
            } else if slice == "</styl" {
                in_style = false;
                while i < chars.len() && chars[i] != '>' {
                    i += 1;
                }
                i += 1;
                continue;
            }
        }
        
        if in_script || in_style {
            i += 1;
            continue;
        }
        
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag {
            result.push(c);
        }
        i += 1;
    }
    
    // Decode common HTML entities
    result
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        // Collapse multiple whitespace
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Convert HTML to markdown (simplified conversion)
fn html_to_markdown(html: &str) -> String {
    let mut result = html.to_string();
    
    // Remove script and style tags with content
    let script_re = regex::Regex::new(r"(?is)<script[^>]*>.*?</script>").unwrap();
    let style_re = regex::Regex::new(r"(?is)<style[^>]*>.*?</style>").unwrap();
    result = script_re.replace_all(&result, "").to_string();
    result = style_re.replace_all(&result, "").to_string();
    
    // Convert common HTML elements to markdown
    // Headers
    let h1_re = regex::Regex::new(r"(?is)<h1[^>]*>(.*?)</h1>").unwrap();
    let h2_re = regex::Regex::new(r"(?is)<h2[^>]*>(.*?)</h2>").unwrap();
    let h3_re = regex::Regex::new(r"(?is)<h3[^>]*>(.*?)</h3>").unwrap();
    let h4_re = regex::Regex::new(r"(?is)<h4[^>]*>(.*?)</h4>").unwrap();
    result = h1_re.replace_all(&result, "\n# $1\n").to_string();
    result = h2_re.replace_all(&result, "\n## $1\n").to_string();
    result = h3_re.replace_all(&result, "\n### $1\n").to_string();
    result = h4_re.replace_all(&result, "\n#### $1\n").to_string();
    
    // Paragraphs and line breaks
    let p_re = regex::Regex::new(r"(?is)<p[^>]*>(.*?)</p>").unwrap();
    let br_re = regex::Regex::new(r"(?i)<br\s*/?>").unwrap();
    result = p_re.replace_all(&result, "\n$1\n").to_string();
    result = br_re.replace_all(&result, "\n").to_string();
    
    // Links
    let a_re = regex::Regex::new(r#"(?is)<a[^>]*href=["']([^"']+)["'][^>]*>(.*?)</a>"#).unwrap();
    result = a_re.replace_all(&result, "[$2]($1)").to_string();
    
    // Bold and italic
    let strong_re = regex::Regex::new(r"(?is)<(strong|b)[^>]*>(.*?)</\1>").unwrap();
    let em_re = regex::Regex::new(r"(?is)<(em|i)[^>]*>(.*?)</\1>").unwrap();
    result = strong_re.replace_all(&result, "**$2**").to_string();
    result = em_re.replace_all(&result, "*$2*").to_string();
    
    // Code
    let code_re = regex::Regex::new(r"(?is)<code[^>]*>(.*?)</code>").unwrap();
    let pre_re = regex::Regex::new(r"(?is)<pre[^>]*>(.*?)</pre>").unwrap();
    result = code_re.replace_all(&result, "`$1`").to_string();
    result = pre_re.replace_all(&result, "\n```\n$1\n```\n").to_string();
    
    // Lists
    let li_re = regex::Regex::new(r"(?is)<li[^>]*>(.*?)</li>").unwrap();
    result = li_re.replace_all(&result, "- $1\n").to_string();
    
    // Remove remaining tags
    let tag_re = regex::Regex::new(r"<[^>]+>").unwrap();
    result = tag_re.replace_all(&result, "").to_string();
    
    // Decode HTML entities
    result = result
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'");
    
    // Clean up whitespace
    let multi_newline = regex::Regex::new(r"\n{3,}").unwrap();
    result = multi_newline.replace_all(&result, "\n\n").to_string();
    result.trim().to_string()
}

/// Webfetch tool for fetching web content
pub struct WebfetchTool {
    client: reqwest::Client,
    cache: Arc<WebfetchCache>,
}

impl WebfetchTool {
    /// Create a new webfetch tool
    pub fn new() -> Result<Self, ToolError> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .user_agent("RiceCoder/1.0 (WebFetch Tool)")
            .build()
            .map_err(|e| {
                ToolError::new("CLIENT_ERROR", "Failed to create HTTP client")
                    .with_details(e.to_string())
            })?;

        Ok(Self { 
            client,
            cache: Arc::new(WebfetchCache::new()),
        })
    }

    /// Create with shared cache
    pub fn with_cache(cache: Arc<WebfetchCache>) -> Result<Self, ToolError> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .user_agent("RiceCoder/1.0 (WebFetch Tool)")
            .build()
            .map_err(|e| {
                ToolError::new("CLIENT_ERROR", "Failed to create HTTP client")
                    .with_details(e.to_string())
            })?;

        Ok(Self { client, cache })
    }

    /// Get the cache (for sharing between instances)
    pub fn cache(&self) -> Arc<WebfetchCache> {
        Arc::clone(&self.cache)
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
                return Err(ToolError::new(
                    "INVALID_SCHEME",
                    "Only http and https schemes are supported",
                )
                .with_suggestion("Use http:// or https:// URLs"))
            }
        }

        // Check for SSRF attacks - reject private IPs and localhost
        if let Some(host) = parsed_url.host_str() {
            // Reject localhost
            if host == "localhost" || host == "127.0.0.1" || host == "::1" {
                return Err(
                    ToolError::new("SSRF_PREVENTION", "Localhost URLs are not allowed")
                        .with_suggestion("Use a public URL instead"),
                );
            }

            // Try to parse as IP address and check if private
            if let Ok(ip) = IpAddr::from_str(host) {
                let is_private = match ip {
                    IpAddr::V4(ipv4) => ipv4.is_private() || ipv4.is_loopback(),
                    IpAddr::V6(ipv6) => ipv6.is_loopback(),
                };

                if is_private {
                    return Err(ToolError::new(
                        "SSRF_PREVENTION",
                        "Private IP addresses are not allowed",
                    )
                    .with_details(format!("IP: {}", ip))
                    .with_suggestion("Use a public URL instead"));
                }
            }
        }

        Ok(parsed_url)
    }

    /// Fetch content from a URL with caching and format conversion
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

        // Check cache first (cache stores raw HTML)
        if let Some((cached_content, original_size)) = self.cache.get(&input.url).await {
            info!("Serving from cache: {}", input.url);
            let content = Self::apply_format(&cached_content, input.format);
            let max_size = input.max_size.unwrap_or(MAX_CONTENT_SIZE);
            let (content, truncated) = Self::truncate_content(content, max_size);
            
            let output = WebfetchOutput {
                content,
                truncated,
                original_size,
                returned_size: 0, // Will be set below
                from_cache: true,
                format: input.format,
            };
            let output = WebfetchOutput {
                returned_size: output.content.len(),
                ..output
            };
            
            let duration_ms = start.elapsed().as_millis() as u64;
            return ToolResult::ok(output, duration_ms, "builtin");
        }

        // Determine timeout (user-specified or default, capped at max)
        let timeout_secs = input.timeout
            .unwrap_or(DEFAULT_TIMEOUT_SECS)
            .min(REQUEST_TIMEOUT_SECS);
        let timeout_duration = Duration::from_secs(timeout_secs);
        
        // Determine max size
        let max_size = input.max_size.unwrap_or(MAX_CONTENT_SIZE);

        let url_for_cache = input.url.clone();
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
                            let raw_content = String::from_utf8_lossy(&bytes).to_string();
                            
                            Ok((raw_content, original_size))
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
            Ok(Ok((raw_content, original_size))) => {
                // Store raw content in cache
                self.cache.set(url_for_cache, raw_content.clone(), original_size).await;
                
                // Apply format conversion
                let content = Self::apply_format(&raw_content, input.format);
                let (content, truncated) = Self::truncate_content(content, max_size);
                
                let output = WebfetchOutput {
                    returned_size: content.len(),
                    content,
                    truncated,
                    original_size,
                    from_cache: false,
                    format: input.format,
                };
                
                let duration_ms = start.elapsed().as_millis() as u64;
                ToolResult::ok(output, duration_ms, "builtin")
            }
            Ok(Err(error)) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                ToolResult::err(error, duration_ms, "builtin")
            }
            Err(_) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                let error = ToolError::new(
                    "TIMEOUT",
                    format!("Webfetch operation exceeded {} second timeout", timeout_secs),
                )
                .with_details(format!("URL: {}", input.url))
                .with_suggestion(
                    "Try again with a longer timeout or check your network connection",
                );
                ToolResult::err(error, duration_ms, "builtin")
            }
        }
    }

    /// Apply format conversion to content
    fn apply_format(content: &str, format: OutputFormat) -> String {
        match format {
            OutputFormat::Html => content.to_string(),
            OutputFormat::Text => html_to_text(content),
            OutputFormat::Markdown => html_to_markdown(content),
        }
    }

    /// Truncate content if necessary, returns (content, was_truncated)
    fn truncate_content(content: String, max_size: usize) -> (String, bool) {
        if content.len() > max_size {
            warn!("Content truncated from {} to {} bytes", content.len(), max_size);
            // Truncate at char boundary
            let truncated: String = content.chars().take(max_size).collect();
            (truncated, true)
        } else {
            (content, false)
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
        Ok(Self {
            builtin,
            mcp_client,
        })
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
