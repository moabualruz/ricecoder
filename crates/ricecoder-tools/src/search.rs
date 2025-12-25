//! Web search tool for searching the web
//!
//! Provides functionality to search the web using Exa AI, free APIs, or local search engines via MCP.
//! Implements query validation, injection prevention, and pagination support.

use std::time::Instant;

use regex::Regex;
use serde::{Deserialize, Serialize};
use tokio::time::timeout;
use tracing::{debug, info, warn};

use crate::{error::ToolError, result::ToolResult};

/// Maximum timeout for search operations (25 seconds)
const SEARCH_TIMEOUT_SECS: u64 = 25;

/// Default limit for search results
const DEFAULT_LIMIT: usize = 8;

/// Maximum limit for search results
const MAX_LIMIT: usize = 100;

/// Default context max characters
const DEFAULT_CONTEXT_MAX_CHARS: usize = 10000;

/// Search type for query optimization
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SearchType {
    /// Balanced search (default)
    #[default]
    Auto,
    /// Quick results, less comprehensive
    Fast,
    /// Comprehensive search, slower
    Deep,
}

/// Live crawl mode for content freshness
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LiveCrawlMode {
    /// Use live crawling as backup if cached content unavailable
    #[default]
    Fallback,
    /// Prioritize live crawling for freshest content
    Preferred,
}

/// Search provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchProvider {
    /// Exa AI search (requires API key)
    ExaAI { api_key: String },
    /// DuckDuckGo (free, no API key)
    DuckDuckGo,
    /// SearXNG self-hosted instance
    SearXNG { base_url: String },
    /// MCP server (delegates to configured MCP search server)
    MCP,
}

impl Default for SearchProvider {
    fn default() -> Self {
        SearchProvider::DuckDuckGo
    }
}

/// Input for web search operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchInput {
    /// Search query string
    pub query: String,
    /// Maximum number of results to return (default: 8, max: 100)
    #[serde(default)]
    pub limit: Option<usize>,
    /// Offset for pagination (default: 0)
    #[serde(default)]
    pub offset: Option<usize>,
    /// Search type: auto, fast, or deep
    #[serde(default, rename = "type")]
    pub search_type: SearchType,
    /// Live crawl mode: fallback or preferred
    #[serde(default)]
    pub livecrawl: LiveCrawlMode,
    /// Maximum characters for context string (default: 10000)
    #[serde(default)]
    pub context_max_characters: Option<usize>,
}

impl SearchInput {
    /// Create a new search input
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            limit: None,
            offset: None,
            search_type: SearchType::default(),
            livecrawl: LiveCrawlMode::default(),
            context_max_characters: None,
        }
    }

    /// Set the result limit
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit.min(MAX_LIMIT));
        self
    }

    /// Set the pagination offset
    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Set the search type
    pub fn with_search_type(mut self, search_type: SearchType) -> Self {
        self.search_type = search_type;
        self
    }

    /// Set live crawl mode
    pub fn with_livecrawl(mut self, mode: LiveCrawlMode) -> Self {
        self.livecrawl = mode;
        self
    }

    /// Set context max characters
    pub fn with_context_max_chars(mut self, max_chars: usize) -> Self {
        self.context_max_characters = Some(max_chars);
        self
    }

    /// Get the effective limit (respects maximum)
    pub fn get_limit(&self) -> usize {
        self.limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT)
    }

    /// Get the effective offset
    pub fn get_offset(&self) -> usize {
        self.offset.unwrap_or(0)
    }

    /// Get the effective context max characters
    pub fn get_context_max_chars(&self) -> usize {
        self.context_max_characters.unwrap_or(DEFAULT_CONTEXT_MAX_CHARS)
    }
}

/// Individual search result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchResult {
    /// Result title
    pub title: String,
    /// Result URL
    pub url: String,
    /// Result snippet/description
    pub snippet: String,
    /// Relevance rank (lower is more relevant)
    pub rank: usize,
}

impl SearchResult {
    /// Create a new search result
    pub fn new(
        title: impl Into<String>,
        url: impl Into<String>,
        snippet: impl Into<String>,
        rank: usize,
    ) -> Self {
        Self {
            title: title.into(),
            url: url.into(),
            snippet: snippet.into(),
            rank,
        }
    }
}

/// Output for web search operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOutput {
    /// Search results
    pub results: Vec<SearchResult>,
    /// Total number of results available (for pagination)
    pub total_count: usize,
}

impl SearchOutput {
    /// Create a new search output
    pub fn new(results: Vec<SearchResult>, total_count: usize) -> Self {
        Self {
            results,
            total_count,
        }
    }
}

/// Web search tool with Exa AI, DuckDuckGo, and MCP support
pub struct SearchTool {
    http_client: reqwest::Client,
    provider: SearchProvider,
    mcp_available: bool,
}

impl SearchTool {
    /// Create a new search tool with default provider (DuckDuckGo)
    pub fn new() -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(SEARCH_TIMEOUT_SECS))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            http_client,
            provider: SearchProvider::default(),
            mcp_available: false,
        }
    }

    /// Create a new search tool with Exa AI provider
    pub fn with_exa_ai(api_key: impl Into<String>) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(SEARCH_TIMEOUT_SECS))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            http_client,
            provider: SearchProvider::ExaAI { api_key: api_key.into() },
            mcp_available: false,
        }
    }

    /// Create a new search tool with custom provider
    pub fn with_provider(provider: SearchProvider) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(SEARCH_TIMEOUT_SECS))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            http_client,
            provider,
            mcp_available: false,
        }
    }

    /// Create a new search tool with MCP support
    pub fn with_mcp(mcp_available: bool) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(SEARCH_TIMEOUT_SECS))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            http_client,
            provider: SearchProvider::MCP,
            mcp_available,
        }
    }

    /// Enable MCP fallback
    pub fn enable_mcp(mut self) -> Self {
        self.mcp_available = true;
        self
    }

    /// Validate search query for injection attacks and format
    pub fn validate_query(query: &str) -> Result<(), ToolError> {
        // Check for empty query
        if query.trim().is_empty() {
            return Err(
                ToolError::new("INVALID_QUERY", "Search query cannot be empty")
                    .with_suggestion("Provide a non-empty search query"),
            );
        }

        // Check query length (reasonable limit)
        if query.len() > 1000 {
            return Err(ToolError::new("INVALID_QUERY", "Search query is too long")
                .with_details("Query exceeds 1000 characters")
                .with_suggestion("Use a shorter search query"));
        }

        // Check for SQL injection patterns
        let sql_patterns = [
            r"(?i)(union|select|insert|update|delete|drop|create|alter|exec|execute)",
            r"(?i)(--|;|/\*|\*/|xp_|sp_)",
            r"'.*=.*'",   // Pattern matching for quoted comparisons like '1'='1'
            r#"".*=.*""#, // Pattern matching for double-quoted comparisons
        ];

        for pattern in &sql_patterns {
            if let Ok(re) = Regex::new(pattern) {
                if re.is_match(query) {
                    return Err(ToolError::new(
                        "INVALID_QUERY",
                        "Query contains suspicious patterns",
                    )
                    .with_suggestion("Use a simple search query without SQL keywords"));
                }
            }
        }

        Ok(())
    }

    /// Execute a web search with MCP fallback to built-in
    pub async fn search(&self, input: SearchInput) -> ToolResult<SearchOutput> {
        let start = Instant::now();

        // Validate query
        if let Err(err) = Self::validate_query(&input.query) {
            return ToolResult::err(err, start.elapsed().as_millis() as u64, "builtin");
        }

        // Try MCP first if available
        if self.mcp_available {
            match self.try_mcp_search(&input).await {
                Ok(output) => {
                    return ToolResult::ok(output, start.elapsed().as_millis() as u64, "mcp");
                }
                Err(err) => {
                    // Log MCP failure and fall back to built-in
                    tracing::warn!("MCP search failed: {}, falling back to built-in", err);
                }
            }
        }

        // Fall back to built-in implementation
        match timeout(
            std::time::Duration::from_secs(SEARCH_TIMEOUT_SECS),
            self.execute_search(&input),
        )
        .await
        {
            Ok(Ok(output)) => ToolResult::ok(output, start.elapsed().as_millis() as u64, "builtin"),
            Ok(Err(err)) => ToolResult::err(err, start.elapsed().as_millis() as u64, "builtin"),
            Err(_) => {
                let err = ToolError::new("TIMEOUT", "Search operation exceeded 10 seconds")
                    .with_suggestion("Try a simpler query or try again later");
                ToolResult::err(err, start.elapsed().as_millis() as u64, "builtin")
            }
        }
    }

    /// Try to execute search via MCP server
    async fn try_mcp_search(&self, _input: &SearchInput) -> Result<SearchOutput, ToolError> {
        // In a real implementation, this would:
        // 1. Query ricecoder-mcp for available search servers
        // 2. Delegate to MCP server (Meilisearch, Typesense, etc.)
        // 3. Handle MCP server failures gracefully
        //
        // For now, return an error to trigger fallback
        Err(ToolError::new(
            "MCP_UNAVAILABLE",
            "MCP search server not available",
        ))
    }

    /// Internal search execution - routes to appropriate provider
    async fn execute_search(&self, input: &SearchInput) -> Result<SearchOutput, ToolError> {
        match &self.provider {
            SearchProvider::ExaAI { api_key } => {
                self.search_exa_ai(input, api_key).await
            }
            SearchProvider::DuckDuckGo => {
                self.search_duckduckgo(input).await
            }
            SearchProvider::SearXNG { base_url } => {
                self.search_searxng(input, base_url).await
            }
            SearchProvider::MCP => {
                // MCP handled separately in search()
                self.search_duckduckgo(input).await
            }
        }
    }

    /// Search using Exa AI API
    async fn search_exa_ai(&self, input: &SearchInput, api_key: &str) -> Result<SearchOutput, ToolError> {
        debug!("Executing Exa AI search for: {}", input.query);

        let search_type = match input.search_type {
            SearchType::Auto => "auto",
            SearchType::Fast => "keyword",
            SearchType::Deep => "neural",
        };

        let use_autoprompt = matches!(input.search_type, SearchType::Auto | SearchType::Deep);

        // Build Exa AI request
        let request_body = serde_json::json!({
            "query": input.query,
            "numResults": input.get_limit(),
            "type": search_type,
            "useAutoprompt": use_autoprompt,
            "contents": {
                "text": {
                    "maxCharacters": input.get_context_max_chars()
                }
            }
        });

        let response = self.http_client
            .post("https://api.exa.ai/search")
            .header("x-api-key", api_key)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| {
                ToolError::new("NETWORK_ERROR", "Failed to connect to Exa AI")
                    .with_details(e.to_string())
                    .with_suggestion("Check network connection and API key")
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(ToolError::new("API_ERROR", format!("Exa AI returned {}", status))
                .with_details(error_text)
                .with_suggestion("Check API key and rate limits"));
        }

        let exa_response: ExaSearchResponse = response.json().await.map_err(|e| {
            ToolError::new("PARSE_ERROR", "Failed to parse Exa AI response")
                .with_details(e.to_string())
        })?;

        let results: Vec<SearchResult> = exa_response
            .results
            .into_iter()
            .enumerate()
            .skip(input.get_offset())
            .take(input.get_limit())
            .map(|(idx, r)| SearchResult::new(
                r.title.unwrap_or_else(|| "Untitled".to_string()),
                r.url,
                r.text.unwrap_or_else(|| r.snippet.unwrap_or_default()),
                idx + 1,
            ))
            .collect();

        let total = results.len();
        info!("Exa AI search returned {} results", total);

        Ok(SearchOutput::new(results, total))
    }

    /// Search using DuckDuckGo Instant Answer API (free)
    async fn search_duckduckgo(&self, input: &SearchInput) -> Result<SearchOutput, ToolError> {
        debug!("Executing DuckDuckGo search for: {}", input.query);

        let url = format!(
            "https://api.duckduckgo.com/?q={}&format=json&no_html=1&skip_disambig=1",
            urlencoding::encode(&input.query)
        );

        let response = self.http_client
            .get(&url)
            .header("User-Agent", "RiceCoder/1.0")
            .send()
            .await
            .map_err(|e| {
                ToolError::new("NETWORK_ERROR", "Failed to connect to DuckDuckGo")
                    .with_details(e.to_string())
            })?;

        if !response.status().is_success() {
            return Err(ToolError::new("API_ERROR", "DuckDuckGo search failed")
                .with_details(format!("Status: {}", response.status())));
        }

        let ddg_response: DuckDuckGoResponse = response.json().await.map_err(|e| {
            ToolError::new("PARSE_ERROR", "Failed to parse DuckDuckGo response")
                .with_details(e.to_string())
        })?;

        let mut results = Vec::new();
        let mut rank = 1;

        // Add abstract if present
        if !ddg_response.abstract_text.is_empty() {
            results.push(SearchResult::new(
                ddg_response.heading.clone(),
                ddg_response.abstract_url.clone(),
                ddg_response.abstract_text.clone(),
                rank,
            ));
            rank += 1;
        }

        // Add related topics
        for topic in ddg_response.related_topics.iter().take(input.get_limit()) {
            if let Some(text) = &topic.text {
                results.push(SearchResult::new(
                    topic.text.clone().unwrap_or_default(),
                    topic.first_url.clone().unwrap_or_default(),
                    text.clone(),
                    rank,
                ));
                rank += 1;
            }
        }

        let total = results.len();
        let paginated: Vec<SearchResult> = results
            .into_iter()
            .skip(input.get_offset())
            .take(input.get_limit())
            .collect();

        info!("DuckDuckGo search returned {} results", paginated.len());

        Ok(SearchOutput::new(paginated, total))
    }

    /// Search using SearXNG instance
    async fn search_searxng(&self, input: &SearchInput, base_url: &str) -> Result<SearchOutput, ToolError> {
        debug!("Executing SearXNG search for: {}", input.query);

        let url = format!(
            "{}/search?q={}&format=json",
            base_url.trim_end_matches('/'),
            urlencoding::encode(&input.query)
        );

        let response = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| {
                ToolError::new("NETWORK_ERROR", "Failed to connect to SearXNG")
                    .with_details(e.to_string())
            })?;

        if !response.status().is_success() {
            return Err(ToolError::new("API_ERROR", "SearXNG search failed")
                .with_details(format!("Status: {}", response.status())));
        }

        let searxng_response: SearXNGResponse = response.json().await.map_err(|e| {
            ToolError::new("PARSE_ERROR", "Failed to parse SearXNG response")
                .with_details(e.to_string())
        })?;

        let results: Vec<SearchResult> = searxng_response
            .results
            .into_iter()
            .enumerate()
            .skip(input.get_offset())
            .take(input.get_limit())
            .map(|(idx, r)| SearchResult::new(
                r.title,
                r.url,
                r.content.unwrap_or_default(),
                idx + 1,
            ))
            .collect();

        let total = results.len();
        info!("SearXNG search returned {} results", total);

        Ok(SearchOutput::new(results, total))
    }
}

// =============================================================================
// API Response Types
// =============================================================================

/// Exa AI search response
#[derive(Debug, Deserialize)]
struct ExaSearchResponse {
    results: Vec<ExaResult>,
}

#[derive(Debug, Deserialize)]
struct ExaResult {
    url: String,
    title: Option<String>,
    text: Option<String>,
    snippet: Option<String>,
}

/// DuckDuckGo Instant Answer response
#[derive(Debug, Deserialize)]
struct DuckDuckGoResponse {
    #[serde(rename = "Heading", default)]
    heading: String,
    #[serde(rename = "AbstractText", default)]
    abstract_text: String,
    #[serde(rename = "AbstractURL", default)]
    abstract_url: String,
    #[serde(rename = "RelatedTopics", default)]
    related_topics: Vec<DdgTopic>,
}

#[derive(Debug, Deserialize)]
struct DdgTopic {
    #[serde(rename = "Text")]
    text: Option<String>,
    #[serde(rename = "FirstURL")]
    first_url: Option<String>,
}

/// SearXNG response
#[derive(Debug, Deserialize)]
struct SearXNGResponse {
    results: Vec<SearXNGResult>,
}

#[derive(Debug, Deserialize)]
struct SearXNGResult {
    title: String,
    url: String,
    content: Option<String>,
}

impl Default for SearchTool {
    fn default() -> Self {
        Self::new()
    }
}
