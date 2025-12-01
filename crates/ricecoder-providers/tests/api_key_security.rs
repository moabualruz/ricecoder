//! Property-based tests for API key security
//! **Feature: ricecoder-providers, Property 5: API Key Security**
//! **Validates: Requirements 4.3**

use proptest::prelude::*;
use ricecoder_providers::{redact, contains_sensitive_info, Redacted};

/// Strategy for generating API keys
fn api_key_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // OpenAI keys (sk-*)
        r"sk-[A-Za-z0-9]{20,48}".prop_map(|s| s.to_string()),
        // Anthropic keys (sk-ant-*)
        r"sk-ant-[A-Za-z0-9]{20,48}".prop_map(|s| s.to_string()),
        // Generic API keys
        r"[A-Za-z0-9]{32,64}".prop_map(|s| format!("api_key={}", s)),
    ]
}



/// Property: API keys never appear in redacted output
/// For any API key, after redaction, the original key should not appear in the output
#[test]
fn prop_api_key_never_appears_in_redacted_output() {
    proptest!(|(key in api_key_strategy())| {
        let message = format!("My API key is: {}", key);
        let redacted = redact(&message);
        
        // The original key should not appear in the redacted output
        prop_assert!(!redacted.contains(&key), 
            "Key '{}' found in redacted output: '{}'", key, redacted);
        
        // The redacted output should contain a redaction marker
        prop_assert!(
            redacted.contains("[REDACTED") || redacted.contains("REDACTED]"),
            "No redaction marker found in: '{}'", redacted
        );
    });
}

/// Property: Redaction is consistent
/// For any API key, redacting it multiple times should produce the same result
#[test]
fn prop_redaction_is_consistent() {
    proptest!(|(key in api_key_strategy())| {
        let message = format!("API key: {}", key);
        
        let redacted1 = redact(&message);
        let redacted2 = redact(&message);
        let redacted3 = redact(&message);
        
        // All redactions should be identical
        prop_assert_eq!(&redacted1, &redacted2);
        prop_assert_eq!(&redacted2, &redacted3);
    });
}

/// Property: Sensitive info detection is accurate
/// For any API key, contains_sensitive_info should return true
#[test]
fn prop_sensitive_info_detection_accurate() {
    proptest!(|(key in api_key_strategy())| {
        let message = format!("My key is: {}", key);
        
        // Should detect the sensitive information
        prop_assert!(contains_sensitive_info(&message),
            "Failed to detect sensitive info in: '{}'", message);
    });
}

/// Property: Redacted wrapper prevents key leakage in Debug output
/// For any API key, the Debug output of Redacted should not contain the original key
#[test]
fn prop_redacted_wrapper_prevents_debug_leakage() {
    proptest!(|(key in api_key_strategy())| {
        let redacted = Redacted(format!("API key: {}", key));
        let debug_output = format!("{:?}", redacted);
        
        // The original key should not appear in debug output
        prop_assert!(!debug_output.contains(&key),
            "Key '{}' found in debug output: '{}'", key, debug_output);
        
        // Should contain redaction marker
        prop_assert!(
            debug_output.contains("[REDACTED") || debug_output.contains("REDACTED]"),
            "No redaction marker in debug output: '{}'", debug_output
        );
    });
}

/// Property: Redacted wrapper prevents key leakage in Display output
/// For any API key, the Display output of Redacted should not contain the original key
#[test]
fn prop_redacted_wrapper_prevents_display_leakage() {
    proptest!(|(key in api_key_strategy())| {
        let redacted = Redacted(format!("API key: {}", key));
        let display_output = format!("{}", redacted);
        
        // The original key should not appear in display output
        prop_assert!(!display_output.contains(&key),
            "Key '{}' found in display output: '{}'", key, display_output);
        
        // Should contain redaction marker
        prop_assert!(
            display_output.contains("[REDACTED") || display_output.contains("REDACTED]"),
            "No redaction marker in display output: '{}'", display_output
        );
    });
}

/// Property: Multiple keys in one message are all redacted
/// For any message with multiple API keys, all keys should be redacted
#[test]
fn prop_multiple_keys_all_redacted() {
    proptest!(|(
        key1 in api_key_strategy(),
        key2 in api_key_strategy(),
        key3 in api_key_strategy()
    )| {
        let message = format!("Keys: {} and {} and {}", key1, key2, key3);
        let redacted = redact(&message);
        
        // None of the original keys should appear
        prop_assert!(!redacted.contains(&key1), "Key1 found in: '{}'", redacted);
        prop_assert!(!redacted.contains(&key2), "Key2 found in: '{}'", redacted);
        prop_assert!(!redacted.contains(&key3), "Key3 found in: '{}'", redacted);
        
        // Should have multiple redaction markers
        let redaction_count = redacted.matches("[REDACTED").count();
        prop_assert!(redaction_count >= 3, 
            "Expected at least 3 redactions, found {}", redaction_count);
    });
}

/// Property: Environment variable patterns are redacted
/// For any environment variable with API key, it should be redacted
#[test]
fn prop_env_var_patterns_redacted() {
    proptest!(|(key in "[A-Za-z0-9]{20,48}")| {
        let env_patterns = vec![
            format!("OPENAI_API_KEY={}", key),
            format!("ANTHROPIC_API_KEY={}", key),
            format!("GOOGLE_API_KEY={}", key),
        ];
        
        for pattern in env_patterns {
            let redacted = redact(&pattern);
            prop_assert!(!redacted.contains(&key),
                "Key found in redacted env var: '{}'", redacted);
            prop_assert!(redacted.contains("[REDACTED]"),
                "No redaction marker in: '{}'", redacted);
        }
    });
}

/// Property: Bearer tokens are redacted
/// For any bearer token, it should be redacted
#[test]
fn prop_bearer_tokens_redacted() {
    proptest!(|(token in "[A-Za-z0-9._-]{20,100}")| {
        let message = format!("Authorization: Bearer {}", token);
        let redacted = redact(&message);
        
        // The original token should not appear
        prop_assert!(!redacted.contains(&token),
            "Token found in: '{}'", redacted);
        
        // Should contain redaction marker
        prop_assert!(redacted.contains("[REDACTED]"),
            "No redaction marker in: '{}'", redacted);
    });
}

/// Property: Normal messages are not affected by redaction
/// For any message without sensitive info, redaction should not change it
#[test]
fn prop_normal_messages_unchanged() {
    proptest!(|(message in r"[a-zA-Z0-9 .,!?]{0,200}")| {
        // Skip if message contains patterns that look like keys
        if contains_sensitive_info(&message) {
            return Ok(());
        }
        
        let redacted = redact(&message);
        
        // Message should be unchanged
        prop_assert_eq!(&message, &redacted,
            "Normal message was modified: '{}' -> '{}'", &message, &redacted);
    });
}

/// Property: Redaction preserves message structure
/// For any message with API key, redaction should preserve the overall structure
#[test]
fn prop_redaction_preserves_structure() {
    proptest!(|(key in api_key_strategy())| {
        let message = format!("Error: Failed to authenticate with key: {}", key);
        let redacted = redact(&message);
        
        // Should still contain the error message structure
        prop_assert!(redacted.contains("Error:"), "Lost error prefix");
        prop_assert!(redacted.contains("Failed to authenticate"), "Lost error message");
        prop_assert!(redacted.contains("key:"), "Lost key label");
        
        // But not the actual key
        prop_assert!(!redacted.contains(&key), "Key not redacted");
    });
}

/// Property: Redaction is idempotent
/// For any message, redacting it multiple times should produce the same result
#[test]
fn prop_redaction_is_idempotent() {
    proptest!(|(key in api_key_strategy())| {
        let message = format!("Key: {}", key);
        
        let redacted1 = redact(&message);
        let redacted2 = redact(&redacted1);
        let redacted3 = redact(&redacted2);
        
        // All subsequent redactions should be identical
        prop_assert_eq!(&redacted1, &redacted2);
        prop_assert_eq!(&redacted2, &redacted3);
    });
}

/// Property: Case-insensitive patterns are redacted
/// For any API key pattern with different cases, it should be redacted
#[test]
fn prop_case_insensitive_redaction() {
    proptest!(|(key in "[A-Za-z0-9]{20,48}")| {
        let patterns = vec![
            format!("api_key={}", key),
            format!("API_KEY={}", key),
            format!("ApiKey={}", key),
            format!("apiKey={}", key),
        ];
        
        for pattern in patterns {
            let redacted = redact(&pattern);
            prop_assert!(!redacted.contains(&key),
                "Key found in: '{}'", redacted);
        }
    });
}
