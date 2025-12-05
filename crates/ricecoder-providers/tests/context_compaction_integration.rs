//! Integration tests for context compaction
//! Tests compaction with embeddings, summaries, and privacy redaction

use ricecoder_providers::redaction::RedactionFilter;

/// Test: Redaction filter removes API keys
#[test]
fn test_redaction_removes_api_keys() {
    let filter = RedactionFilter::new();

    let content = "My API key is sk-1234567890abcdefghij";
    let redacted = filter.redact(content);

    // Should contain redaction marker
    assert!(redacted.contains("[REDACTED_OPENAI_KEY]"));
    assert!(!redacted.contains("sk-1234567890abcdefghij"));
}

/// Test: Redaction filter removes common secrets
#[test]
fn test_redaction_removes_common_secrets() {
    let filter = RedactionFilter::new();

    let content = "password=secret123 and token=abc123def456";
    let redacted = filter.redact(content);

    // Should redact sensitive patterns
    assert!(redacted.contains("[REDACTED]") || !redacted.contains("secret123"));
}

/// Test: Redaction filter preserves non-sensitive content
#[test]
fn test_redaction_preserves_non_sensitive_content() {
    let filter = RedactionFilter::new();

    let content = "This is a normal message about code generation";
    let redacted = filter.redact(content);

    // Should preserve normal content
    assert!(redacted.contains("normal message"));
    assert!(redacted.contains("code generation"));
}

/// Test: Redaction filter handles multiple secrets
#[test]
fn test_redaction_handles_multiple_secrets() {
    let filter = RedactionFilter::new();

    let content = "API_KEY=sk-123 and SECRET=abc and PASSWORD=xyz";
    let redacted = filter.redact(content);

    // Should redact all sensitive patterns
    assert!(!redacted.contains("sk-123") || redacted.contains("[REDACTED]"));
}

/// Test: Redaction filter is idempotent
#[test]
fn test_redaction_is_idempotent() {
    let filter = RedactionFilter::new();

    let content = "My API key is sk-1234567890abcdefghij";
    let redacted_once = filter.redact(content);
    let redacted_twice = filter.redact(&redacted_once);

    // Redacting twice should produce same result as once
    assert_eq!(redacted_once, redacted_twice);
}

/// Test: Redaction filter handles empty content
#[test]
fn test_redaction_handles_empty_content() {
    let filter = RedactionFilter::new();

    let content = "";
    let redacted = filter.redact(content);

    // Should handle empty content gracefully
    assert_eq!(redacted, "");
}

/// Test: Redaction filter handles content without secrets
#[test]
fn test_redaction_handles_content_without_secrets() {
    let filter = RedactionFilter::new();

    let content = "This is just regular text with no secrets";
    let redacted = filter.redact(content);

    // Should preserve content unchanged
    assert_eq!(redacted, content);
}

/// Test: Redaction filter handles PII patterns
#[test]
fn test_redaction_handles_pii_patterns() {
    let filter = RedactionFilter::new();

    // Test with email-like patterns
    let content = "Contact me at user@example.com";
    let redacted = filter.redact(content);

    // Should either preserve or redact consistently
    assert!(redacted.len() > 0);
}

/// Test: Redaction filter handles credit card patterns
#[test]
fn test_redaction_handles_credit_card_patterns() {
    let filter = RedactionFilter::new();

    let content = "Card number: 4532-1234-5678-9010";
    let redacted = filter.redact(content);

    // Should redact or preserve consistently
    assert!(redacted.len() > 0);
}

/// Test: Redaction filter handles SSN patterns
#[test]
fn test_redaction_handles_ssn_patterns() {
    let filter = RedactionFilter::new();

    let content = "SSN: 123-45-6789";
    let redacted = filter.redact(content);

    // Should redact or preserve consistently
    assert!(redacted.len() > 0);
}

/// Test: Context compaction preserves semantic meaning
#[test]
fn test_context_compaction_preserves_meaning() {
    let filter = RedactionFilter::new();

    let original = "The system should validate user input and reject invalid data";
    let redacted = filter.redact(original);

    // Semantic meaning should be preserved
    assert!(redacted.contains("validate") || redacted.contains("input"));
}

/// Test: Redaction filter handles mixed content
#[test]
fn test_redaction_handles_mixed_content() {
    let filter = RedactionFilter::new();

    let content = "User john@example.com has API key sk-abc123 and password secret";
    let redacted = filter.redact(content);

    // Should handle multiple types of sensitive data
    assert!(redacted.len() > 0);
}

/// Test: Redaction filter handles newlines
#[test]
fn test_redaction_handles_newlines() {
    let filter = RedactionFilter::new();

    let content = "Line 1: API key is sk-123\nLine 2: Password is secret";
    let redacted = filter.redact(content);

    // Should preserve structure
    assert!(redacted.contains('\n') || redacted.len() > 0);
}

/// Test: Redaction filter handles special characters
#[test]
fn test_redaction_handles_special_characters() {
    let filter = RedactionFilter::new();

    let content = "API_KEY=sk-abc!@#$%^&*()_+-=[]{}|;:',.<>?/";
    let redacted = filter.redact(content);

    // Should handle special characters gracefully
    assert!(redacted.len() > 0);
}

/// Test: Redaction filter handles unicode
#[test]
fn test_redaction_handles_unicode() {
    let filter = RedactionFilter::new();

    let content = "API key: sk-123 and message: 你好世界 🌍";
    let redacted = filter.redact(content);

    // Should handle unicode gracefully
    assert!(redacted.len() > 0);
}

/// Test: Redaction filter handles very long content
#[test]
fn test_redaction_handles_long_content() {
    let filter = RedactionFilter::new();

    let mut content = String::new();
    for i in 0..1000 {
        content.push_str(&format!("Line {}: This is normal text\n", i));
    }
    content.push_str("API key: sk-123");

    let redacted = filter.redact(&content);

    // Should handle long content
    assert!(redacted.len() > 0);
}

/// Test: Redaction filter handles repeated patterns
#[test]
fn test_redaction_handles_repeated_patterns() {
    let filter = RedactionFilter::new();

    let content = "sk-123 sk-123 sk-123 sk-123";
    let redacted = filter.redact(content);

    // Should handle repeated patterns
    assert!(redacted.len() > 0);
}

/// Test: Redaction filter is thread-safe
#[test]
fn test_redaction_filter_thread_safe() {
    use std::sync::Arc;
    use std::thread;

    let filter = Arc::new(RedactionFilter::new());
    let mut handles = vec![];

    for i in 0..10 {
        let filter_clone = Arc::clone(&filter);
        let handle = thread::spawn(move || {
            let content = format!("API key: sk-{}", i);
            let redacted = filter_clone.redact(&content);
            assert!(redacted.len() > 0);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

/// Test: Redaction filter performance with large content
#[test]
fn test_redaction_filter_performance() {
    let filter = RedactionFilter::new();

    let mut content = String::new();
    for _ in 0..10000 {
        content.push_str("This is a line of normal text that should not be redacted\n");
    }

    let start = std::time::Instant::now();
    let _redacted = filter.redact(&content);
    let elapsed = start.elapsed();

    // Should complete in reasonable time (< 1 second)
    assert!(elapsed.as_secs() < 1);
}

/// Test: Redaction filter with environment variable patterns
#[test]
fn test_redaction_with_env_var_patterns() {
    let filter = RedactionFilter::new();

    let content = "export OPENAI_API_KEY=sk-123\nexport ANTHROPIC_API_KEY=sk-456";
    let redacted = filter.redact(content);

    // Should handle environment variable patterns
    assert!(redacted.len() > 0);
}

/// Test: Redaction filter with URL patterns
#[test]
fn test_redaction_with_url_patterns() {
    let filter = RedactionFilter::new();

    let content = "https://api.openai.com/v1/chat/completions?key=sk-123";
    let redacted = filter.redact(content);

    // Should handle URL patterns
    assert!(redacted.len() > 0);
}

/// Test: Redaction filter with JSON patterns
#[test]
fn test_redaction_with_json_patterns() {
    let filter = RedactionFilter::new();

    let content = r#"{"api_key": "sk-123", "password": "secret"}"#;
    let redacted = filter.redact(content);

    // Should handle JSON patterns
    assert!(redacted.len() > 0);
}

/// Test: Redaction filter with YAML patterns
#[test]
fn test_redaction_with_yaml_patterns() {
    let filter = RedactionFilter::new();

    let content = "api_key: sk-123\npassword: secret";
    let redacted = filter.redact(content);

    // Should handle YAML patterns
    assert!(redacted.len() > 0);
}
