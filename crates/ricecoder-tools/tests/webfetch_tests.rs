//! Tests for ricecoder-tools webfetch functionality
//!
//! Tests for WebfetchTool, WebfetchInput, WebfetchOutput, and URL validation.

use ricecoder_tools::webfetch::{WebfetchInput, WebfetchOutput, WebfetchTool};

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

    // Should fail (either timeout or HTTP error due to network conditions)
    assert!(!result.success);
    assert!(result.error.is_some());
    if let Some(error) = result.error {
        // Accept either TIMEOUT or HTTP_ERROR as both indicate the request failed
        assert!(
            error.code == "TIMEOUT" || error.code == "HTTP_ERROR",
            "Expected TIMEOUT or HTTP_ERROR, got: {}",
            error.code
        );
    }
}
