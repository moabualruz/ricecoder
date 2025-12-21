use ricecoder_providers::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_token_bucket_acquire() {
        let mut limiter = TokenBucketLimiter::new(10.0, 100.0);
        assert!(limiter.try_acquire(50.0));
        let tokens = limiter.current_tokens();
        // Allow for small floating-point variations
        assert!((tokens - 50.0).abs() < 0.1);
    }

    #[test]
    fn test_token_bucket_rate_limited() {
        let mut limiter = TokenBucketLimiter::new(10.0, 100.0);
        // Acquire all tokens
        assert!(limiter.try_acquire(100.0));
        // Try to acquire more (should fail)
        assert!(!limiter.try_acquire(1.0));
    }

    #[test]
    fn test_token_bucket_refill() {
        let mut limiter = TokenBucketLimiter::new(10.0, 100.0);
        // Acquire all tokens
        assert!(limiter.try_acquire(100.0));
        // Wait for refill
        std::thread::sleep(Duration::from_millis(150));
        // Should have some tokens now
        let tokens = limiter.current_tokens();
        assert!(tokens > 0.0);
    }

    #[test]
    fn test_exponential_backoff() {
        let mut backoff =
            ExponentialBackoff::new(Duration::from_millis(100), Duration::from_secs(10), 2.0);

        let delay1 = backoff.next_delay();
        assert!(delay1.as_millis() >= 90 && delay1.as_millis() <= 110);

        let delay2 = backoff.next_delay();
        assert!(delay2.as_millis() >= 180 && delay2.as_millis() <= 220);

        let delay3 = backoff.next_delay();
        assert!(delay3.as_millis() >= 360 && delay3.as_millis() <= 440);
    }

    #[test]
    fn test_exponential_backoff_max_delay() {
        let mut backoff =
            ExponentialBackoff::new(Duration::from_millis(100), Duration::from_secs(1), 2.0);

        // Skip to high attempt number
        for _ in 0..10 {
            backoff.next_delay();
        }

        // Should be capped at max_delay (with small tolerance for timing)
        let delay = backoff.next_delay();
        assert!(delay <= Duration::from_millis(1100));
    }

    #[test]
    fn test_exponential_backoff_reset() {
        let mut backoff =
            ExponentialBackoff::new(Duration::from_millis(100), Duration::from_secs(10), 2.0);

        backoff.next_delay();
        backoff.next_delay();
        assert_eq!(backoff.attempt(), 2);

        backoff.reset();
        assert_eq!(backoff.attempt(), 0);
    }

    #[test]
    fn test_rate_limiter_registry() {
        let registry = RateLimiterRegistry::new();
        registry.register("openai", TokenBucketLimiter::new(10.0, 100.0));

        assert!(registry.try_acquire("openai", 50.0));
        assert!(registry.try_acquire("openai", 50.0));
        assert!(!registry.try_acquire("openai", 1.0));
    }

    #[test]
    fn test_rate_limiter_registry_unknown_provider() {
        let registry = RateLimiterRegistry::new();
        // Unknown provider should be allowed (no limiter registered)
        assert!(registry.try_acquire("unknown", 1000.0));
    }
}
