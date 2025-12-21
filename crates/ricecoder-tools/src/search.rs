//! Web search tool for searching the web
//!
//! Provides functionality to search the web using free APIs or local search engines via MCP.
//! Implements query validation, injection prevention, and pagination support.

use crate::error::ToolError;
use crate::result::ToolResult;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tokio::time::timeout;
use tracing;

/// Maximum timeout for search operations (10 seconds)
const SEARCH_TIMEOUT_SECS: u64 = 10;

/// Default limit for search results
const DEFAULT_LIMIT: usize = 10;

/// Maximum limit for search results
const MAX_LIMIT: usize = 100;

/// Input for web search operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchInput {
    /// Search query string
    pub query: String,
    /// Maximum number of results to return (default: 10, max: 100)
    pub limit: Option<usize>,
    /// Offset for pagination (default: 0)
    pub offset: Option<usize>,
}

impl SearchInput {
    /// Create a new search input
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            limit: None,
            offset: None,
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

    /// Get the effective limit (respects maximum)
    pub fn get_limit(&self) -> usize {
        self.limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT)
    }

    /// Get the effective offset
    pub fn get_offset(&self) -> usize {
        self.offset.unwrap_or(0)
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

/// Web search tool with built-in and MCP support
pub struct SearchTool {
    _http_client: reqwest::Client,
    mcp_available: bool,
}

impl SearchTool {
    /// Create a new search tool
    pub fn new() -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(SEARCH_TIMEOUT_SECS))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            _http_client: http_client,
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
            _http_client: http_client,
            mcp_available,
        }
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

    /// Internal search execution
    async fn execute_search(&self, input: &SearchInput) -> Result<SearchOutput, ToolError> {
        // For now, return mock results that demonstrate the API
        // In production, this would call a real search API like SearXNG
        let mock_results = vec![
            SearchResult::new(
                "Example Result 1",
                "https://example.com/1",
                "This is the first search result snippet",
                1,
            ),
            SearchResult::new(
                "Example Result 2",
                "https://example.com/2",
                "This is the second search result snippet",
                2,
            ),
            SearchResult::new(
                "Example Result 3",
                "https://example.com/3",
                "This is the third search result snippet",
                3,
            ),
        ];

        let limit = input.get_limit();
        let offset = input.get_offset();

        // Apply pagination
        let paginated: Vec<SearchResult> =
            mock_results.into_iter().skip(offset).take(limit).collect();

        Ok(SearchOutput::new(paginated, 3))
    }
}

impl Default for SearchTool {
    fn default() -> Self {
        Self::new()
    }
}
