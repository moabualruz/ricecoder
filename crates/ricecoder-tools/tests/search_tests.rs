//! Tests for ricecoder-tools search functionality
//!
//! Tests for SearchTool, SearchInput, SearchOutput, SearchResult, and query validation.

use ricecoder_tools::search::{SearchInput, SearchOutput, SearchResult, SearchTool};

#[test]
fn test_search_input_creation() {
    let input = SearchInput::new("rust programming");
    assert_eq!(input.query, "rust programming");
    assert_eq!(input.get_limit(), 10); // DEFAULT_LIMIT
    assert_eq!(input.get_offset(), 0);
}

#[test]
fn test_search_input_with_limit() {
    let input = SearchInput::new("rust").with_limit(50);
    assert_eq!(input.get_limit(), 50);
}

#[test]
fn test_search_input_limit_capped() {
    let input = SearchInput::new("rust").with_limit(200);
    assert_eq!(input.get_limit(), 100); // MAX_LIMIT
}

#[test]
fn test_search_input_with_offset() {
    let input = SearchInput::new("rust").with_offset(20);
    assert_eq!(input.get_offset(), 20);
}

#[test]
fn test_search_result_creation() {
    let result = SearchResult::new("Title", "https://example.com", "Snippet", 1);
    assert_eq!(result.title, "Title");
    assert_eq!(result.url, "https://example.com");
    assert_eq!(result.snippet, "Snippet");
    assert_eq!(result.rank, 1);
}

#[test]
fn test_search_output_creation() {
    let results = vec![SearchResult::new("Title", "https://example.com", "Snippet", 1)];
    let output = SearchOutput::new(results.clone(), 1);
    assert_eq!(output.results, results);
    assert_eq!(output.total_count, 1);
}

#[test]
fn test_validate_query_empty() {
    let result = SearchTool::validate_query("");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code, "INVALID_QUERY");
}

#[test]
fn test_validate_query_whitespace_only() {
    let result = SearchTool::validate_query("   ");
    assert!(result.is_err());
}

#[test]
fn test_validate_query_too_long() {
    let long_query = "a".repeat(1001);
    let result = SearchTool::validate_query(&long_query);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code, "INVALID_QUERY");
}

#[test]
fn test_validate_query_sql_injection() {
    let queries = vec![
        "test' UNION SELECT * FROM users",
        "test; DROP TABLE users",
        "test' OR '1'='1",
    ];

    for query in queries {
        let result = SearchTool::validate_query(query);
        assert!(result.is_err(), "Query should be rejected: {}", query);
    }
}

#[test]
fn test_validate_query_valid() {
    let queries = vec!["rust programming", "how to learn rust", "best practices"];

    for query in queries {
        let result = SearchTool::validate_query(query);
        assert!(result.is_ok(), "Query should be valid: {}", query);
    }
}

#[tokio::test]
async fn test_search_tool_creation() {
    let _tool = SearchTool::new();
    // Tool created successfully
}

#[tokio::test]
async fn test_search_tool_with_mcp() {
    let _tool = SearchTool::with_mcp(true);
    // Tool created successfully with MCP enabled
}

#[tokio::test]
async fn test_search_empty_query() {
    let tool = SearchTool::new();
    let input = SearchInput::new("");
    let result = tool.search(input).await;
    assert!(!result.success);
    assert!(result.error.is_some());
}

#[tokio::test]
async fn test_search_valid_query() {
    let tool = SearchTool::new();
    let input = SearchInput::new("rust programming");
    let result = tool.search(input).await;
    assert!(result.success);
    assert!(result.data.is_some());
    let output = result.data.unwrap();
    assert!(!output.results.is_empty());
}

#[tokio::test]
async fn test_search_pagination() {
    let tool = SearchTool::new();
    let input = SearchInput::new("rust").with_limit(2).with_offset(1);
    let result = tool.search(input).await;
    assert!(result.success);
    let output = result.data.unwrap();
    assert_eq!(output.results.len(), 2);
}

#[tokio::test]
async fn test_search_mcp_fallback() {
    let tool = SearchTool::with_mcp(true);
    let input = SearchInput::new("rust programming");
    let result = tool.search(input).await;
    // Should fall back to built-in when MCP is unavailable
    assert!(result.success);
    assert_eq!(result.metadata.provider, "builtin");
}

#[test]
fn test_search_result_serialization() {
    let result = SearchResult::new("Title", "https://example.com", "Snippet", 1);
    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"title\":\"Title\""));
    assert!(json.contains("\"url\":\"https://example.com\""));
}

#[test]
fn test_search_output_serialization() {
    let results = vec![SearchResult::new("Title", "https://example.com", "Snippet", 1)];
    let output = SearchOutput::new(results, 1);
    let json = serde_json::to_string(&output).unwrap();
    assert!(json.contains("\"results\""));
    assert!(json.contains("\"total_count\":1"));
}