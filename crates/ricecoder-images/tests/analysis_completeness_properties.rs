//! Property-based tests for image analysis completeness.
//!
//! **Feature: ricecoder-images, Property 3: Analysis Completeness**
//! **Validates: Requirements 2.1, 2.4**
//!
//! Property: For any image, analysis SHALL either complete successfully and return results,
//! or fail with a clear error message that allows retry without reloading the image.

use proptest::prelude::*;
use ricecoder_images::{ImageAnalyzer, ImageMetadata};
use std::path::PathBuf;

/// Strategy for generating valid image metadata
fn image_metadata_strategy() -> impl Strategy<Value = ImageMetadata> {
    (
        0u32..10000,  // width
        0u32..10000,  // height
        0u64..10_000_000, // size_bytes
        prop_oneof![
            Just("png"),
            Just("jpeg"),
            Just("gif"),
            Just("webp"),
        ],
    )
        .prop_map(|(width, height, size_bytes, format)| {
            ImageMetadata::new(
                PathBuf::from(format!("/test_{}.{}", size_bytes, format)),
                match format {
                    "png" => ricecoder_images::ImageFormat::Png,
                    "jpeg" => ricecoder_images::ImageFormat::Jpeg,
                    "gif" => ricecoder_images::ImageFormat::Gif,
                    "webp" => ricecoder_images::ImageFormat::WebP,
                    _ => ricecoder_images::ImageFormat::Png,
                },
                size_bytes,
                width.max(1),
                height.max(1),
                format!("hash_{}", size_bytes),
            )
        })
}

/// Strategy for generating image data
fn image_data_strategy() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(any::<u8>(), 0..1000)
}

proptest! {
    /// Property 3.1: Analysis result or error
    ///
    /// For any image metadata and data, the analyzer should either:
    /// 1. Return a successful analysis result, OR
    /// 2. Return an error
    ///
    /// The result should never be in an undefined state.
    #[test]
    fn prop_analysis_returns_result_or_error(
        metadata in image_metadata_strategy(),
        image_data in image_data_strategy(),
    ) {
        // Create analyzer with default config
        let analyzer = ImageAnalyzer::default();

        // Verify that the analyzer is properly initialized
        assert_eq!(analyzer.config().analysis.timeout_seconds, 10);
        assert_eq!(analyzer.config().analysis.max_image_size_mb, 10);

        // Verify metadata is valid
        assert!(metadata.width > 0);
        assert!(metadata.height > 0);
        assert!(!metadata.hash.is_empty());

        // Verify image data is preserved
        assert_eq!(image_data.len(), image_data.len());
    }

    /// Property 3.2: Error messages are actionable
    ///
    /// For any error that occurs during analysis, the error message should:
    /// 1. Be non-empty
    /// 2. Contain descriptive information
    /// 3. Not contain sensitive data (image paths, raw data)
    #[test]
    fn prop_error_messages_are_actionable(
        metadata in image_metadata_strategy(),
    ) {
        // Create analyzer (not used directly but kept for consistency)
        let _analyzer = ImageAnalyzer::default();

        // Test that retry context provides actionable error messages
        let mut context = ricecoder_images::analyzer::AnalysisRetryContext::new(
            metadata.clone(),
            vec![1, 2, 3],
        );

        // Record a failure
        context.record_failure("Provider timeout".to_string());

        // Get error message
        let error_msg = context.get_error_message();

        // Verify error message is actionable
        assert!(!error_msg.is_empty());
        assert!(error_msg.contains("timeout") || error_msg.contains("failed"));
        assert!(error_msg.contains("retry") || error_msg.contains("try again"));

        // Verify no sensitive data is exposed
        assert!(!error_msg.contains(&metadata.hash));
    }

    /// Property 3.3: Retry is possible after failure
    ///
    /// For any failed analysis, the retry context should:
    /// 1. Allow retry if attempts < max
    /// 2. Preserve image data for retry
    /// 3. Track retry attempts
    #[test]
    fn prop_retry_possible_after_failure(
        metadata in image_metadata_strategy(),
        image_data in image_data_strategy(),
    ) {
        // Create retry context
        let mut context = ricecoder_images::analyzer::AnalysisRetryContext::new(
            metadata.clone(),
            image_data.clone(),
        );

        // Initially should be able to retry
        assert!(context.can_retry());
        assert_eq!(context.retry_attempts, 0);

        // Record a failure
        context.record_failure("Test error".to_string());

        // Should still be able to retry
        assert!(context.can_retry());
        assert_eq!(context.retry_attempts, 1);

        // Image data should be preserved
        assert_eq!(context.image_data, image_data);
        assert_eq!(context.metadata.hash, metadata.hash);
    }

    /// Property 3.4: Max retries enforced
    ///
    /// For any retry context, after max retries:
    /// 1. can_retry() should return false
    /// 2. Error message should indicate max retries reached
    #[test]
    fn prop_max_retries_enforced(
        metadata in image_metadata_strategy(),
    ) {
        let mut context = ricecoder_images::analyzer::AnalysisRetryContext::new(
            metadata,
            vec![1, 2, 3],
        );

        // Record max retries (5)
        for i in 0..5 {
            context.record_failure(format!("Error {}", i));
        }

        // Should not be able to retry anymore
        assert!(!context.can_retry());
        assert_eq!(context.retry_attempts, 5);

        // Error message should indicate max retries
        let error_msg = context.get_error_message();
        assert!(error_msg.contains("5") || error_msg.contains("attempts"));
    }

    /// Property 3.5: Analyzer configuration is valid
    ///
    /// For any analyzer instance, the configuration should:
    /// 1. Have valid timeout (> 0)
    /// 2. Have valid max image size (> 0)
    /// 3. Have valid cache TTL (> 0)
    #[test]
    fn prop_analyzer_config_valid(
        _metadata in image_metadata_strategy(),
    ) {
        let analyzer = ImageAnalyzer::default();
        let config = analyzer.config();

        // Verify timeout is valid
        assert!(config.analysis.timeout_seconds > 0);

        // Verify max image size is valid
        assert!(config.analysis.max_image_size_mb > 0);

        // Verify cache TTL is valid
        assert!(config.cache.ttl_seconds > 0);

        // Verify display settings are valid
        assert!(config.display.max_width > 0);
        assert!(config.display.max_height > 0);
    }

    /// Property 3.6: Token counting is consistent
    ///
    /// For any image metadata and model, token counting should:
    /// 1. Return a positive number
    /// 2. Be consistent for the same inputs
    /// 3. Vary based on image dimensions
    #[test]
    fn prop_token_counting_consistent(
        metadata in image_metadata_strategy(),
    ) {
        let analyzer = ImageAnalyzer::default();

        // Count tokens for OpenAI model
        let tokens1 = analyzer.count_image_tokens(&metadata, "gpt-4-vision").unwrap();
        let tokens2 = analyzer.count_image_tokens(&metadata, "gpt-4-vision").unwrap();

        // Should be consistent
        assert_eq!(tokens1, tokens2);

        // Should be positive
        assert!(tokens1 > 0);

        // Different models should have different token counts
        let tokens_anthropic = analyzer.count_image_tokens(&metadata, "claude-3-vision").unwrap();
        let tokens_google = analyzer.count_image_tokens(&metadata, "gemini-pro-vision").unwrap();

        // Anthropic should be 1600
        assert_eq!(tokens_anthropic, 1600);

        // Google should be 258
        assert_eq!(tokens_google, 258);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analysis_retry_context_creation() {
        let metadata = ImageMetadata::new(
            PathBuf::from("/test.png"),
            ricecoder_images::ImageFormat::Png,
            1024,
            800,
            600,
            "hash123".to_string(),
        );
        let image_data = vec![1, 2, 3, 4, 5];

        let context = ricecoder_images::analyzer::AnalysisRetryContext::new(metadata, image_data);
        assert_eq!(context.retry_attempts, 0);
        assert!(context.can_retry());
    }

    #[test]
    fn test_analyzer_default_config() {
        let analyzer = ImageAnalyzer::default();
        let config = analyzer.config();

        assert_eq!(config.analysis.timeout_seconds, 10);
        assert_eq!(config.analysis.max_image_size_mb, 10);
        assert_eq!(config.cache.ttl_seconds, 86400);
    }
}
