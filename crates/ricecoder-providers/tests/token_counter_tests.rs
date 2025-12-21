use ricecoder_providers::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_counter_empty_string() {
        let counter = TokenCounter::new();
        assert_eq!(counter.count_tokens_openai("", "gpt-4"), 0);
    }

    #[test]
    fn test_token_counter_simple_text() {
        let counter = TokenCounter::new();
        let tokens = counter.count_tokens_openai("Hello world", "gpt-4");
        assert!(tokens > 0);
    }

    #[test]
    fn test_token_counter_caching() {
        let counter = TokenCounter::new();
        let content = "This is a test message";
        let tokens1 = counter.count_tokens_openai(content, "gpt-4");
        let tokens2 = counter.count_tokens_openai(content, "gpt-4");
        assert_eq!(tokens1, tokens2);
        assert_eq!(counter.cache_size(), 1);
    }

    #[test]
    fn test_token_counter_different_models() {
        let counter = TokenCounter::new();
        let content = "Test content";
        let _tokens_gpt4 = counter.count_tokens_openai(content, "gpt-4");
        let _tokens_gpt35 = counter.count_tokens_openai(content, "gpt-3.5-turbo");
        // Both should be cached
        assert_eq!(counter.cache_size(), 2);
    }

    #[test]
    fn test_token_counter_special_characters() {
        let counter = TokenCounter::new();
        let simple = counter.count_tokens_openai("hello", "gpt-4");
        let with_special = counter.count_tokens_openai("hello!!!???", "gpt-4");
        // Special characters should increase token count
        assert!(with_special >= simple);
    }

    #[test]
    fn test_token_counter_clear_cache() {
        let counter = TokenCounter::new();
        counter.count_tokens_openai("test", "gpt-4");
        assert_eq!(counter.cache_size(), 1);
        counter.clear_cache();
        assert_eq!(counter.cache_size(), 0);
    }
}
