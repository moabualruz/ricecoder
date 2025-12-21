use ricecoder_providers::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::redaction::get_redaction_filter;

    #[test]
    fn test_redact_openai_key() {
        let filter = RedactionFilter::new();
        let input = "My API key is sk-1234567890abcdefghij";
        let redacted = filter.redact(input);
        assert!(!redacted.contains("sk-1234567890abcdefghij"));
        assert!(redacted.contains("[REDACTED_OPENAI_KEY]"));
    }

    #[test]
    fn test_redact_anthropic_key() {
        let filter = RedactionFilter::new();
        let input = "My API key is sk-ant-1234567890abcdefghij";
        let redacted = filter.redact(input);
        assert!(!redacted.contains("sk-ant-1234567890abcdefghij"));
        assert!(redacted.contains("[REDACTED_ANTHROPIC_KEY]"));
    }

    #[test]
    fn test_redact_api_key_equals() {
        let filter = RedactionFilter::new();
        let input = "api_key=secret123456789";
        let redacted = filter.redact(input);
        assert!(!redacted.contains("secret123456789"));
        assert!(redacted.contains("[REDACTED]"));
    }

    #[test]
    fn test_redact_bearer_token() {
        let filter = RedactionFilter::new();
        let input = "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";
        let redacted = filter.redact(input);
        assert!(!redacted.contains("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9"));
        // Bearer token should be redacted (either as part of Authorization header or Bearer pattern)
        assert!(redacted.contains("[REDACTED]"));
    }

    #[test]
    fn test_redact_env_var() {
        let filter = RedactionFilter::new();
        let input = "OPENAI_API_KEY=sk-1234567890abcdefghij";
        let redacted = filter.redact(input);
        assert!(!redacted.contains("sk-1234567890abcdefghij"));
        assert!(redacted.contains("[REDACTED]"));
    }

    #[test]
    fn test_contains_sensitive_info_true() {
        let filter = RedactionFilter::new();
        assert!(filter.contains_sensitive_info("My key is sk-1234567890abcdefghij"));
        assert!(filter.contains_sensitive_info("api_key=secret123"));
        assert!(filter.contains_sensitive_info("Bearer token123"));
    }

    #[test]
    fn test_contains_sensitive_info_false() {
        let filter = RedactionFilter::new();
        assert!(!filter.contains_sensitive_info("This is a normal message"));
        assert!(!filter.contains_sensitive_info("No secrets here"));
    }

    #[test]
    fn test_add_custom_pattern() {
        let mut filter = RedactionFilter::new();
        filter
            .add_pattern(r"custom_secret_\d+", "[CUSTOM_REDACTED]")
            .unwrap();

        let input = "Found custom_secret_12345";
        let redacted = filter.redact(input);
        assert!(redacted.contains("[CUSTOM_REDACTED]"));
    }

    #[test]
    fn test_redacted_debug() {
        let secret = "sk-1234567890abcdefghij";
        let redacted = Redacted(secret);
        let debug_str = format!("{:?}", redacted);
        assert!(!debug_str.contains("sk-1234567890abcdefghij"));
        assert!(debug_str.contains("[REDACTED_OPENAI_KEY]"));
    }

    #[test]
    fn test_redacted_display() {
        let secret = "api_key=secret123";
        let redacted = Redacted(secret);
        let display_str = format!("{}", redacted);
        assert!(!display_str.contains("secret123"));
        assert!(display_str.contains("[REDACTED]"));
    }

    #[test]
    fn test_global_redaction_filter() {
        let filter = get_redaction_filter();
        let input = "My key is sk-1234567890abcdefghij";
        let redacted = filter.redact(input);
        assert!(redacted.contains("[REDACTED_OPENAI_KEY]"));
    }

    #[test]
    fn test_global_redact_function() {
        let input = "My key is sk-1234567890abcdefghij";
        let redacted = redact(input);
        assert!(redacted.contains("[REDACTED_OPENAI_KEY]"));
    }

    #[test]
    fn test_multiple_keys_in_string() {
        let filter = RedactionFilter::new();
        let input = "openai: sk-1234567890abcdefghij, anthropic: sk-ant-1234567890abcdefghij";
        let redacted = filter.redact(input);
        assert!(!redacted.contains("sk-1234567890abcdefghij"));
        assert!(!redacted.contains("sk-ant-1234567890abcdefghij"));
        assert!(redacted.contains("[REDACTED_OPENAI_KEY]"));
        assert!(redacted.contains("[REDACTED_ANTHROPIC_KEY]"));
    }

    #[test]
    fn test_case_insensitive_redaction() {
        let filter = RedactionFilter::new();
        let input = "API_KEY=secret123 and ApiKey=secret456";
        let redacted = filter.redact(input);
        assert!(!redacted.contains("secret123"));
        assert!(!redacted.contains("secret456"));
    }
}
