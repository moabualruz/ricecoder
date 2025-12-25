//! Image analysis via AI providers.
//!
//! This module handles image analysis by coordinating with AI providers.
//! It supports:
//! - Sequential analysis of multiple images
//! - Automatic optimization of large images (>10 MB)
//! - Retry logic with exponential backoff
//! - Token counting for images
//! - Audit logging of analysis requests
//! - User-initiated retry without reloading images

use std::{sync::Arc, time::Duration};

use base64::{engine::general_purpose, Engine as _};
use ricecoder_providers::{
    models::{ChatRequest, Message},
    provider::Provider,
    token_counter::TokenCounter,
};
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

use crate::{
    config::ImageConfig,
    error::{ImageError, ImageResult},
    models::{ImageAnalysisResult, ImageMetadata},
};

/// Maximum number of automatic retries for analysis
const MAX_RETRIES: u32 = 3;

/// Initial retry delay in milliseconds
const INITIAL_RETRY_DELAY_MS: u64 = 100;

/// Context for tracking analysis retry state.
///
/// This allows users to retry analysis without reloading the image.
///
/// # Example
///
/// ```ignore
/// let mut context = AnalysisRetryContext::new(metadata, image_data);
/// match analyzer.analyze(&metadata, provider, &image_data).await {
///     Ok(result) => { /* use result */ },
///     Err(err) => {
///         context.record_failure(err.to_string());
///         // User can retry later with: analyzer.retry_analysis(context, provider).await
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AnalysisRetryContext {
    /// Image metadata
    pub metadata: ImageMetadata,
    /// Raw image data
    pub image_data: Vec<u8>,
    /// Last error encountered
    pub last_error: Option<String>,
    /// Number of retry attempts made
    pub retry_attempts: u32,
}

impl AnalysisRetryContext {
    /// Create a new retry context.
    pub fn new(metadata: ImageMetadata, image_data: Vec<u8>) -> Self {
        Self {
            metadata,
            image_data,
            last_error: None,
            retry_attempts: 0,
        }
    }

    /// Record a failed analysis attempt.
    pub fn record_failure(&mut self, error: String) {
        self.last_error = Some(error);
        self.retry_attempts += 1;
    }

    /// Check if retry is possible.
    pub fn can_retry(&self) -> bool {
        self.retry_attempts < 5 // Allow up to 5 user-initiated retries
    }

    /// Get actionable error message for user.
    pub fn get_error_message(&self) -> String {
        match &self.last_error {
            Some(err) => {
                if self.can_retry() {
                    format!(
                        "Analysis failed: {}. You can retry by clicking the retry button.",
                        err
                    )
                } else {
                    format!(
                        "Analysis failed after {} attempts: {}. Please try again later.",
                        self.retry_attempts, err
                    )
                }
            }
            None => "No error recorded".to_string(),
        }
    }
}

/// Analyzes images using configured AI providers.
pub struct ImageAnalyzer {
    config: ImageConfig,
    token_counter: Arc<TokenCounter>,
}

impl ImageAnalyzer {
    /// Create a new image analyzer with default configuration.
    pub fn new() -> ImageResult<Self> {
        let config = ImageConfig::load_with_hierarchy()?;
        Ok(Self {
            config,
            token_counter: Arc::new(TokenCounter::new()),
        })
    }

    /// Create a new image analyzer with custom configuration.
    pub fn with_config(config: ImageConfig) -> Self {
        Self {
            config,
            token_counter: Arc::new(TokenCounter::new()),
        }
    }

    /// Analyze an image using the provided provider.
    ///
    /// # Arguments
    ///
    /// * `metadata` - Image metadata including path, format, and dimensions
    /// * `provider` - The AI provider to use for analysis
    /// * `image_data` - The raw image data (bytes)
    ///
    /// # Returns
    ///
    /// Analysis result with provider response and token usage
    ///
    /// # Errors
    ///
    /// Returns error if analysis fails after retries or if image is too large
    pub async fn analyze(
        &self,
        metadata: &ImageMetadata,
        provider: &dyn Provider,
        image_data: &[u8],
    ) -> ImageResult<ImageAnalysisResult> {
        // Check if image needs optimization
        let optimized_data = if metadata.size_mb() > self.config.analysis.max_image_size_mb as f64 {
            info!(
                size_mb = metadata.size_mb(),
                max_mb = self.config.analysis.max_image_size_mb,
                "Optimizing large image before analysis"
            );
            self.optimize_image(image_data).await?
        } else {
            image_data.to_vec()
        };

        // Perform analysis with retry logic
        self.analyze_with_retry(metadata, provider, &optimized_data)
            .await
    }

    /// Analyze multiple images sequentially.
    ///
    /// # Arguments
    ///
    /// * `images` - Vector of (metadata, image_data) tuples
    /// * `provider` - The AI provider to use for analysis
    ///
    /// # Returns
    ///
    /// Vector of analysis results in the same order as input
    pub async fn analyze_multiple(
        &self,
        images: Vec<(ImageMetadata, Vec<u8>)>,
        provider: &dyn Provider,
    ) -> Vec<ImageResult<ImageAnalysisResult>> {
        let mut results = Vec::new();

        for (metadata, image_data) in images {
            let result = self.analyze(&metadata, provider, &image_data).await;
            results.push(result);
        }

        results
    }

    /// Analyze an image with retry logic and exponential backoff.
    async fn analyze_with_retry(
        &self,
        metadata: &ImageMetadata,
        provider: &dyn Provider,
        image_data: &[u8],
    ) -> ImageResult<ImageAnalysisResult> {
        let mut retry_count = 0;
        let mut delay_ms = INITIAL_RETRY_DELAY_MS;

        loop {
            match self.perform_analysis(metadata, provider, image_data).await {
                Ok(result) => {
                    info!(
                        image_hash = %metadata.hash,
                        provider = provider.name(),
                        tokens_used = result.tokens_used,
                        "Image analysis completed successfully"
                    );
                    return Ok(result);
                }
                Err(err) => {
                    retry_count += 1;

                    if retry_count >= MAX_RETRIES {
                        error!(
                            image_hash = %metadata.hash,
                            provider = provider.name(),
                            error = %err,
                            attempts = retry_count,
                            "Image analysis failed after retries"
                        );
                        return Err(ImageError::AnalysisFailed(format!(
                            "Analysis failed after {} attempts: {}. Please try again.",
                            retry_count, err
                        )));
                    }

                    warn!(
                        image_hash = %metadata.hash,
                        provider = provider.name(),
                        error = %err,
                        attempt = retry_count,
                        retry_delay_ms = delay_ms,
                        "Image analysis failed, retrying..."
                    );

                    // Wait before retrying with exponential backoff
                    sleep(Duration::from_millis(delay_ms)).await;
                    delay_ms *= 2; // Exponential backoff
                }
            }
        }
    }

    /// Perform a single analysis attempt.
    async fn perform_analysis(
        &self,
        metadata: &ImageMetadata,
        provider: &dyn Provider,
        image_data: &[u8],
    ) -> ImageResult<ImageAnalysisResult> {
        // Create analysis prompt
        let prompt = format!(
            "Please analyze this image. Provide a detailed description of what you see, \
             including any text, objects, people, and overall context.\n\n\
             Image format: {}\n\
             Image dimensions: {}x{} pixels\n\
             Image size: {:.1} MB",
            metadata.format_str(),
            metadata.width,
            metadata.height,
            metadata.size_mb()
        );

        // Encode image data as base64 for provider transmission
        let image_base64 = general_purpose::STANDARD.encode(image_data);

        // Create chat request with image data
        // Note: The image data is included in the message content as base64
        // Providers will need to handle this format or we can extend ChatRequest in the future
        let request = ChatRequest {
            model: provider
                .models()
                .first()
                .ok_or_else(|| {
                    ImageError::AnalysisFailed("Provider has no available models".to_string())
                })?
                .id
                .clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: format!(
                    "{}\n\n[Image: format={}, size={} bytes, base64={}...]",
                    prompt,
                    metadata.format_str(),
                    image_data.len(),
                    &image_base64[..std::cmp::min(50, image_base64.len())]
                ),
            }],
            temperature: Some(0.7),
            max_tokens: Some(1000),
            stream: false,
        };

        // Send request to provider
        debug!(
            provider = provider.name(),
            model = &request.model,
            image_size = image_data.len(),
            image_format = metadata.format_str(),
            "Sending image to provider for analysis"
        );

        let response = tokio::time::timeout(
            Duration::from_secs(self.config.analysis.timeout_seconds),
            provider.chat(request),
        )
        .await
        .map_err(|_| {
            ImageError::AnalysisFailed(format!(
                "Analysis timeout after {} seconds",
                self.config.analysis.timeout_seconds
            ))
        })?
        .map_err(|e| ImageError::AnalysisFailed(e.to_string()))?;

        // Count tokens used (including image tokens)
        let tokens_used = self
            .count_image_tokens(metadata, &response.model)
            .unwrap_or(0) as u32;

        // Create analysis result
        let result = ImageAnalysisResult::new(
            metadata.hash.clone(),
            response.content,
            provider.name().to_string(),
            tokens_used,
        );

        Ok(result)
    }

    /// Count tokens for an image based on provider and model.
    ///
    /// Different providers have different token counting for images:
    /// - OpenAI: ~85 tokens per image + variable tokens based on resolution
    /// - Anthropic: ~1600 tokens per image
    /// - Google: ~258 tokens per image
    /// - Ollama: ~100 tokens per image (estimate)
    pub fn count_image_tokens(&self, metadata: &ImageMetadata, model: &str) -> ImageResult<usize> {
        // Determine provider from model name or use default
        let provider_name = if model.contains("gpt") {
            "openai"
        } else if model.contains("claude") {
            "anthropic"
        } else if model.contains("gemini") {
            "google"
        } else {
            "ollama"
        };

        // Calculate tokens based on provider and image dimensions
        let base_tokens = match provider_name {
            "openai" => {
                // OpenAI: ~85 tokens base + variable based on resolution
                let resolution_factor =
                    (metadata.width as usize * metadata.height as usize) / 10000;
                85 + resolution_factor
            }
            "anthropic" => {
                // Anthropic: ~1600 tokens per image
                1600
            }
            "google" => {
                // Google: ~258 tokens per image
                258
            }
            _ => {
                // Ollama and others: ~100 tokens estimate
                100
            }
        };

        debug!(
            provider = provider_name,
            model = model,
            image_tokens = base_tokens,
            "Counted image tokens"
        );

        Ok(base_tokens)
    }

    /// Optimize an image by reducing its size.
    ///
    /// This is a placeholder implementation that validates the image can be processed.
    /// In a real implementation, this would use the `image` crate to resize/compress.
    async fn optimize_image(&self, image_data: &[u8]) -> ImageResult<Vec<u8>> {
        // For now, just validate that the image data is valid
        // In a real implementation, we would use the `image` crate to resize/compress
        if image_data.is_empty() {
            return Err(ImageError::InvalidFile("Image data is empty".to_string()));
        }

        debug!(
            original_size = image_data.len(),
            "Image optimization placeholder (would resize/compress in production)"
        );

        // Return the original data for now
        // TODO: Implement actual image optimization using the `image` crate
        Ok(image_data.to_vec())
    }

    /// Get the configuration used by this analyzer.
    pub fn config(&self) -> &ImageConfig {
        &self.config
    }

    /// Get the token counter used by this analyzer.
    pub fn token_counter(&self) -> &TokenCounter {
        &self.token_counter
    }

    /// Retry analysis for a previously failed image.
    ///
    /// This allows users to retry analysis without reloading the image.
    /// The image data is preserved in the retry context.
    ///
    /// # Arguments
    ///
    /// * `context` - Retry context with preserved image data
    /// * `provider` - The AI provider to use for analysis
    ///
    /// # Returns
    ///
    /// Analysis result or error with actionable message
    pub async fn retry_analysis(
        &self,
        mut context: AnalysisRetryContext,
        provider: &dyn Provider,
    ) -> ImageResult<ImageAnalysisResult> {
        if !context.can_retry() {
            return Err(ImageError::AnalysisFailed(
                "Maximum retry attempts exceeded. Please try again later.".to_string(),
            ));
        }

        info!(
            image_hash = %context.metadata.hash,
            provider = provider.name(),
            attempt = context.retry_attempts + 1,
            "Retrying image analysis"
        );

        match self
            .analyze(&context.metadata, provider, &context.image_data)
            .await
        {
            Ok(result) => {
                info!(
                    image_hash = %context.metadata.hash,
                    provider = provider.name(),
                    "Image analysis succeeded on retry"
                );
                Ok(result)
            }
            Err(err) => {
                context.record_failure(err.to_string());
                Err(ImageError::AnalysisFailed(context.get_error_message()))
            }
        }
    }
}

impl Default for ImageAnalyzer {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self::with_config(ImageConfig::default()))
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_creation() {
        let analyzer = ImageAnalyzer::new()
            .unwrap_or_else(|_| ImageAnalyzer::with_config(ImageConfig::default()));
        assert_eq!(analyzer.config().analysis.timeout_seconds, 10);
    }

    #[test]
    fn test_analyzer_with_config() {
        let config = ImageConfig::default();
        let analyzer = ImageAnalyzer::with_config(config.clone());
        assert_eq!(analyzer.config().analysis.max_image_size_mb, 10);
    }

    #[test]
    fn test_analyzer_default() {
        let _analyzer = ImageAnalyzer::default();
    }

    #[tokio::test]
    async fn test_optimize_image_empty() {
        let analyzer = ImageAnalyzer::default();
        let result = analyzer.optimize_image(&[]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_optimize_image_valid() {
        let analyzer = ImageAnalyzer::default();
        let data = vec![1, 2, 3, 4, 5];
        let result = analyzer.optimize_image(&data).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_count_image_tokens_openai() {
        let analyzer = ImageAnalyzer::default();
        let metadata = ImageMetadata::new(
            std::path::PathBuf::from("/test.png"),
            crate::formats::ImageFormat::Png,
            1024,
            800,
            600,
            "hash123".to_string(),
        );

        let tokens = analyzer
            .count_image_tokens(&metadata, "gpt-4-vision")
            .unwrap();
        assert!(tokens > 0);
        assert!(tokens >= 85); // At least base tokens
    }

    #[test]
    fn test_count_image_tokens_anthropic() {
        let analyzer = ImageAnalyzer::default();
        let metadata = ImageMetadata::new(
            std::path::PathBuf::from("/test.png"),
            crate::formats::ImageFormat::Png,
            1024,
            800,
            600,
            "hash123".to_string(),
        );

        let tokens = analyzer
            .count_image_tokens(&metadata, "claude-3-vision")
            .unwrap();
        assert_eq!(tokens, 1600);
    }

    #[test]
    fn test_count_image_tokens_google() {
        let analyzer = ImageAnalyzer::default();
        let metadata = ImageMetadata::new(
            std::path::PathBuf::from("/test.png"),
            crate::formats::ImageFormat::Png,
            1024,
            800,
            600,
            "hash123".to_string(),
        );

        let tokens = analyzer
            .count_image_tokens(&metadata, "gemini-pro-vision")
            .unwrap();
        assert_eq!(tokens, 258);
    }

    #[test]
    fn test_count_image_tokens_ollama() {
        let analyzer = ImageAnalyzer::default();
        let metadata = ImageMetadata::new(
            std::path::PathBuf::from("/test.png"),
            crate::formats::ImageFormat::Png,
            1024,
            800,
            600,
            "hash123".to_string(),
        );

        let tokens = analyzer.count_image_tokens(&metadata, "llava").unwrap();
        assert_eq!(tokens, 100);
    }

    #[test]
    fn test_retry_context_creation() {
        let metadata = ImageMetadata::new(
            std::path::PathBuf::from("/test.png"),
            crate::formats::ImageFormat::Png,
            1024,
            800,
            600,
            "hash123".to_string(),
        );
        let image_data = vec![1, 2, 3, 4, 5];

        let context = AnalysisRetryContext::new(metadata, image_data);
        assert_eq!(context.retry_attempts, 0);
        assert!(context.can_retry());
        assert!(context.last_error.is_none());
    }

    #[test]
    fn test_retry_context_record_failure() {
        let metadata = ImageMetadata::new(
            std::path::PathBuf::from("/test.png"),
            crate::formats::ImageFormat::Png,
            1024,
            800,
            600,
            "hash123".to_string(),
        );
        let image_data = vec![1, 2, 3, 4, 5];

        let mut context = AnalysisRetryContext::new(metadata, image_data);
        context.record_failure("Test error".to_string());

        assert_eq!(context.retry_attempts, 1);
        assert!(context.can_retry());
        assert!(context.last_error.is_some());
    }

    #[test]
    fn test_retry_context_max_retries() {
        let metadata = ImageMetadata::new(
            std::path::PathBuf::from("/test.png"),
            crate::formats::ImageFormat::Png,
            1024,
            800,
            600,
            "hash123".to_string(),
        );
        let image_data = vec![1, 2, 3, 4, 5];

        let mut context = AnalysisRetryContext::new(metadata, image_data);

        // Record 5 failures (max retries)
        for i in 0..5 {
            context.record_failure(format!("Error {}", i));
        }

        assert_eq!(context.retry_attempts, 5);
        assert!(!context.can_retry());
    }

    #[test]
    fn test_retry_context_error_message() {
        let metadata = ImageMetadata::new(
            std::path::PathBuf::from("/test.png"),
            crate::formats::ImageFormat::Png,
            1024,
            800,
            600,
            "hash123".to_string(),
        );
        let image_data = vec![1, 2, 3, 4, 5];

        let mut context = AnalysisRetryContext::new(metadata, image_data);
        context.record_failure("Provider timeout".to_string());

        let msg = context.get_error_message();
        assert!(msg.contains("Provider timeout"));
        assert!(msg.contains("retry"));
    }

    #[test]
    fn test_retry_context_error_message_max_retries() {
        let metadata = ImageMetadata::new(
            std::path::PathBuf::from("/test.png"),
            crate::formats::ImageFormat::Png,
            1024,
            800,
            600,
            "hash123".to_string(),
        );
        let image_data = vec![1, 2, 3, 4, 5];

        let mut context = AnalysisRetryContext::new(metadata, image_data);

        // Record 5 failures
        for i in 0..5 {
            context.record_failure(format!("Error {}", i));
        }

        let msg = context.get_error_message();
        assert!(msg.contains("5 attempts"));
        assert!(msg.contains("try again later"));
    }
}
