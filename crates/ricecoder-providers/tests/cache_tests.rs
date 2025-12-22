use ricecoder_providers::*;

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::models::{FinishReason, Message, TokenUsage};

    fn create_test_request() -> ChatRequest {
        ChatRequest {
            model: "gpt-4".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }],
            temperature: Some(0.7),
            max_tokens: Some(100),
            stream: false,
        }
    }

    fn create_test_response() -> ChatResponse {
        ChatResponse {
            content: "Hi there!".to_string(),
            model: "gpt-4".to_string(),
            usage: TokenUsage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            },
            finish_reason: FinishReason::Stop,
        }
    }

    #[test]
    fn test_cache_set_and_get() -> Result<(), ProviderError> {
        let temp_dir = TempDir::new().unwrap();
        let cache = ProviderCache::new(temp_dir.path(), 3600)?;

        let request = create_test_request();
        let response = create_test_response();

        // Cache response
        cache.set("openai", "gpt-4", &request, &response)?;

        // Retrieve from cache
        let cached = cache.get("openai", "gpt-4", &request)?;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().content, "Hi there!");

        Ok(())
    }

    #[test]
    fn test_cache_miss() -> Result<(), ProviderError> {
        let temp_dir = TempDir::new().unwrap();
        let cache = ProviderCache::new(temp_dir.path(), 3600)?;

        let request = create_test_request();

        // Try to get non-existent entry
        let cached = cache.get("openai", "gpt-4", &request)?;
        assert!(cached.is_none());

        Ok(())
    }

    #[test]
    fn test_cache_invalidate() -> Result<(), ProviderError> {
        let temp_dir = TempDir::new().unwrap();
        let cache = ProviderCache::new(temp_dir.path(), 3600)?;

        let request = create_test_request();
        let response = create_test_response();

        // Cache response
        cache.set("openai", "gpt-4", &request, &response)?;

        // Invalidate
        let invalidated = cache.invalidate("openai", "gpt-4", &request)?;
        assert!(invalidated);

        // Should be gone now
        let cached = cache.get("openai", "gpt-4", &request)?;
        assert!(cached.is_none());

        Ok(())
    }

    #[test]
    fn test_cache_clear() -> Result<(), ProviderError> {
        let temp_dir = TempDir::new().unwrap();
        let cache = ProviderCache::new(temp_dir.path(), 3600)?;

        let request = create_test_request();
        let response = create_test_response();

        // Cache multiple responses
        cache.set("openai", "gpt-4", &request, &response)?;
        cache.set("anthropic", "claude-3", &request, &response)?;

        // Clear all
        cache.clear()?;

        // Both should be gone
        assert!(cache.get("openai", "gpt-4", &request)?.is_none());
        assert!(cache.get("anthropic", "claude-3", &request)?.is_none());

        Ok(())
    }

    #[test]
    fn test_different_requests_different_cache() -> Result<(), ProviderError> {
        let temp_dir = TempDir::new().unwrap();
        let cache = ProviderCache::new(temp_dir.path(), 3600)?;

        let mut request1 = create_test_request();
        let mut request2 = create_test_request();
        request2.messages[0].content = "Different message".to_string();

        let response1 = ChatResponse {
            content: "Response 1".to_string(),
            model: "gpt-4".to_string(),
            usage: TokenUsage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            },
            finish_reason: FinishReason::Stop,
        };

        let response2 = ChatResponse {
            content: "Response 2".to_string(),
            model: "gpt-4".to_string(),
            usage: TokenUsage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            },
            finish_reason: FinishReason::Stop,
        };

        // Cache different responses for different requests
        cache.set("openai", "gpt-4", &request1, &response1)?;
        cache.set("openai", "gpt-4", &request2, &response2)?;

        // Verify they're cached separately
        let cached1 = cache.get("openai", "gpt-4", &request1)?;
        let cached2 = cache.get("openai", "gpt-4", &request2)?;

        assert_eq!(cached1.unwrap().content, "Response 1");
        assert_eq!(cached2.unwrap().content, "Response 2");

        Ok(())
    }
}
