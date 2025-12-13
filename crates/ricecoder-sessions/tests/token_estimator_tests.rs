use ricecoder_sessions::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_estimator_creation() {
        let estimator = TokenEstimator::new();
        assert_eq!(estimator.default_model, "gpt-3.5-turbo");
        assert!(!estimator.pricing.is_empty());
    }

    #[test]
    fn test_token_estimation() {
        let mut estimator = TokenEstimator::new();
        let estimate = estimator.estimate_tokens("Hello world", Some("gpt-3.5-turbo")).unwrap();

        assert_eq!(estimate.model, "gpt-3.5-turbo");
        assert_eq!(estimate.characters, 11);
        assert!(estimate.tokens > 0);
        assert!(estimate.estimated_cost.is_some());
    }

    #[test]
    fn test_token_limit_status() {
        let estimator = TokenEstimator::new();

        assert_eq!(estimator.check_token_limits(1000, "gpt-3.5-turbo"), TokenLimitStatus::Normal);
        assert_eq!(estimator.check_token_limits(3500, "gpt-3.5-turbo"), TokenLimitStatus::Warning);
        assert_eq!(estimator.check_token_limits(3800, "gpt-3.5-turbo"), TokenLimitStatus::Critical);
    }

    #[test]
    fn test_usage_tracker() {
        let pricing = ModelPricing {
            input_per_1k: 0.0015,
            output_per_1k: 0.002,
            max_tokens: 4096,
        };

        let mut tracker = TokenUsageTracker {
            model: "gpt-3.5-turbo".to_string(),
            total_tokens: 0,
            prompt_tokens: 0,
            completion_tokens: 0,
            estimated_cost: 0.0,
            token_limit: pricing.max_tokens,
            pricing,
        };

        tracker.record_prompt(100);
        tracker.record_completion(50);

        assert_eq!(tracker.total_tokens, 150);
        assert_eq!(tracker.prompt_tokens, 100);
        assert_eq!(tracker.completion_tokens, 50);
        assert!(tracker.estimated_cost > 0.0);
    }

    #[test]
    fn test_limit_status_colors() {
        assert_eq!(TokenLimitStatus::Normal.color(), "green");
        assert_eq!(TokenLimitStatus::Warning.color(), "yellow");
        assert_eq!(TokenLimitStatus::Critical.color(), "red");
        assert_eq!(TokenLimitStatus::Unknown.color(), "gray");
    }

    #[test]
    fn test_limit_status_symbols() {
        assert_eq!(TokenLimitStatus::Normal.symbol(), "✓");
        assert_eq!(TokenLimitStatus::Warning.symbol(), "⚠");
        assert_eq!(TokenLimitStatus::Critical.symbol(), "🚨");
        assert_eq!(TokenLimitStatus::Unknown.symbol(), "?");
    }
}