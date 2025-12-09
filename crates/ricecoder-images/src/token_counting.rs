//! Token counting for images across different AI providers.
//!
//! This module handles token counting for images, which varies significantly
//! by provider. It integrates with ricecoder-providers TokenCounter for
//! consistent token usage tracking.

use crate::error::{ImageError, ImageResult};
use crate::models::ImageMetadata;
use ricecoder_providers::token_counter::TokenCounter;
use std::sync::Arc;
use tracing::{debug, info};

/// Image token counting for different providers.
///
/// Different AI providers count image tokens differently:
/// - OpenAI: ~85 tokens base + variable based on resolution
/// - Anthropic: ~1600 tokens per image
/// - Google: ~258 tokens per image
/// - Ollama: ~100 tokens per image (estimate)
pub struct ImageTokenCounter {
    token_counter: Arc<TokenCounter>,
}

impl ImageTokenCounter {
    /// Create a new image token counter.
    pub fn new() -> Self {
        Self {
            token_counter: Arc::new(TokenCounter::new()),
        }
    }

    /// Create with an existing token counter.
    pub fn with_counter(token_counter: Arc<TokenCounter>) -> Self {
        Self { token_counter }
    }

    /// Count tokens for a single image based on provider and model.
    ///
    /// # Arguments
    ///
    /// * `metadata` - Image metadata including dimensions
    /// * `provider_name` - Name of the provider (openai, anthropic, google, ollama)
    /// * `model` - Model identifier
    ///
    /// # Returns
    ///
    /// Number of tokens used for the image
    pub fn count_image_tokens(
        &self,
        metadata: &ImageMetadata,
        provider_name: &str,
        model: &str,
    ) -> ImageResult<usize> {
        let tokens = match provider_name.to_lowercase().as_str() {
            "openai" => self.count_openai_tokens(metadata, model),
            "anthropic" => self.count_anthropic_tokens(metadata, model),
            "google" => self.count_google_tokens(metadata, model),
            "ollama" => self.count_ollama_tokens(metadata, model),
            _ => self.count_generic_tokens(metadata, model),
        };

        debug!(
            provider = provider_name,
            model = model,
            image_tokens = tokens,
            image_dimensions = format!("{}x{}", metadata.width, metadata.height),
            "Counted image tokens"
        );

        Ok(tokens)
    }

    /// Count tokens for multiple images.
    ///
    /// # Arguments
    ///
    /// * `images` - Vector of image metadata
    /// * `provider_name` - Name of the provider
    /// * `model` - Model identifier
    ///
    /// # Returns
    ///
    /// Total tokens for all images
    pub fn count_multiple_image_tokens(
        &self,
        images: &[ImageMetadata],
        provider_name: &str,
        model: &str,
    ) -> ImageResult<usize> {
        let mut total_tokens = 0;

        for image in images {
            let tokens = self.count_image_tokens(image, provider_name, model)?;
            total_tokens += tokens;
        }

        info!(
            provider = provider_name,
            model = model,
            image_count = images.len(),
            total_tokens = total_tokens,
            "Counted tokens for multiple images"
        );

        Ok(total_tokens)
    }

    /// Count tokens for OpenAI models.
    ///
    /// OpenAI's vision models use the following token counting:
    /// - Base: ~85 tokens per image
    /// - Resolution factor: additional tokens based on image dimensions
    /// - For high-resolution images: ~170 tokens base + resolution factor
    fn count_openai_tokens(&self, metadata: &ImageMetadata, _model: &str) -> usize {
        // Determine if this is a high-resolution image
        let is_high_res = metadata.width > 768 || metadata.height > 768;

        // Base tokens
        let base_tokens = if is_high_res { 170 } else { 85 };

        // Resolution factor: additional tokens based on image size
        // Roughly 1 token per 512 pixels
        let resolution_factor = ((metadata.width as usize * metadata.height as usize) / 512)
            .min(1000); // Cap at 1000 to avoid excessive token counts

        base_tokens + resolution_factor
    }

    /// Count tokens for Anthropic models.
    ///
    /// Anthropic's vision models use:
    /// - ~1600 tokens per image (fixed)
    /// - Additional tokens for image metadata
    fn count_anthropic_tokens(&self, _metadata: &ImageMetadata, _model: &str) -> usize {
        // Base tokens for image
        let base_tokens = 1600;

        // Additional tokens for metadata (format, dimensions)
        let metadata_tokens = 10;

        base_tokens + metadata_tokens
    }

    /// Count tokens for Google models.
    ///
    /// Google's vision models use:
    /// - ~258 tokens per image (fixed)
    /// - Additional tokens based on image complexity
    fn count_google_tokens(&self, metadata: &ImageMetadata, _model: &str) -> usize {
        // Base tokens for image
        let base_tokens = 258;

        // Additional tokens based on resolution
        // Higher resolution images may require more tokens
        let resolution_factor = if metadata.width > 1024 || metadata.height > 1024 {
            50
        } else {
            0
        };

        base_tokens + resolution_factor
    }

    /// Count tokens for Ollama models.
    ///
    /// Ollama's vision models use:
    /// - ~100 tokens per image (estimate)
    /// - Additional tokens based on image size
    fn count_ollama_tokens(&self, metadata: &ImageMetadata, _model: &str) -> usize {
        // Base tokens for image
        let base_tokens = 100;

        // Additional tokens based on image size
        // Roughly 1 token per 10KB
        let size_factor = (metadata.size_bytes / 10240) as usize;

        base_tokens + size_factor
    }

    /// Count tokens for generic/unknown providers.
    ///
    /// Uses a conservative estimate:
    /// - ~100 tokens base per image
    /// - Additional tokens based on resolution
    fn count_generic_tokens(&self, metadata: &ImageMetadata, _model: &str) -> usize {
        // Base tokens for image
        let base_tokens = 100;

        // Resolution factor: 1 token per 1000 pixels
        let resolution_factor = (metadata.width as usize * metadata.height as usize) / 1000;

        base_tokens + resolution_factor
    }

    /// Count tokens for image analysis prompt.
    ///
    /// This counts tokens for the analysis prompt text that accompanies the image.
    pub fn count_prompt_tokens(
        &self,
        prompt: &str,
        model: &str,
    ) -> ImageResult<usize> {
        self.token_counter
            .count(prompt, model)
            .map_err(|e| ImageError::AnalysisFailed(format!("Token counting failed: {}", e)))
    }

    /// Count total tokens for image analysis (image + prompt).
    ///
    /// # Arguments
    ///
    /// * `metadata` - Image metadata
    /// * `prompt` - Analysis prompt text
    /// * `provider_name` - Provider name
    /// * `model` - Model identifier
    ///
    /// # Returns
    ///
    /// Total tokens for image + prompt
    pub fn count_total_tokens(
        &self,
        metadata: &ImageMetadata,
        prompt: &str,
        provider_name: &str,
        model: &str,
    ) -> ImageResult<usize> {
        let image_tokens = self.count_image_tokens(metadata, provider_name, model)?;
        let prompt_tokens = self.count_prompt_tokens(prompt, model)?;

        let total = image_tokens + prompt_tokens;

        debug!(
            provider = provider_name,
            model = model,
            image_tokens = image_tokens,
            prompt_tokens = prompt_tokens,
            total_tokens = total,
            "Counted total tokens for image analysis"
        );

        Ok(total)
    }

    /// Get the underlying token counter.
    pub fn token_counter(&self) -> &TokenCounter {
        &self.token_counter
    }
}

impl Default for ImageTokenCounter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::ImageFormat;
    use std::path::PathBuf;

    fn create_test_metadata(width: u32, height: u32, size_bytes: u64) -> ImageMetadata {
        ImageMetadata::new(
            PathBuf::from("/test.png"),
            ImageFormat::Png,
            size_bytes,
            width,
            height,
            "hash123".to_string(),
        )
    }

    #[test]
    fn test_image_token_counter_creation() {
        let counter = ImageTokenCounter::new();
        // Verify cache is initialized (size is always >= 0 for usize)
        let _ = counter.token_counter().cache_size();
    }

    #[test]
    fn test_count_openai_tokens_standard() {
        let counter = ImageTokenCounter::new();
        let metadata = create_test_metadata(800, 600, 1024 * 1024);

        let tokens = counter
            .count_image_tokens(&metadata, "openai", "gpt-4-vision")
            .unwrap();

        // Should be at least base tokens (85)
        assert!(tokens >= 85);
    }

    #[test]
    fn test_count_openai_tokens_high_res() {
        let counter = ImageTokenCounter::new();
        let metadata = create_test_metadata(1920, 1080, 2 * 1024 * 1024);

        let tokens = counter
            .count_image_tokens(&metadata, "openai", "gpt-4-vision")
            .unwrap();

        // High-res should have more tokens (base 170 + resolution factor)
        assert!(tokens >= 170);
    }

    #[test]
    fn test_count_anthropic_tokens() {
        let counter = ImageTokenCounter::new();
        let metadata = create_test_metadata(800, 600, 1024 * 1024);

        let tokens = counter
            .count_image_tokens(&metadata, "anthropic", "claude-3-vision")
            .unwrap();

        // Anthropic should be around 1600 + metadata
        assert!(tokens >= 1600);
    }

    #[test]
    fn test_count_google_tokens() {
        let counter = ImageTokenCounter::new();
        let metadata = create_test_metadata(800, 600, 1024 * 1024);

        let tokens = counter
            .count_image_tokens(&metadata, "google", "gemini-pro-vision")
            .unwrap();

        // Google should be around 258
        assert!(tokens >= 258);
    }

    #[test]
    fn test_count_ollama_tokens() {
        let counter = ImageTokenCounter::new();
        let metadata = create_test_metadata(800, 600, 1024 * 1024);

        let tokens = counter
            .count_image_tokens(&metadata, "ollama", "llava")
            .unwrap();

        // Ollama should be around 100 + size factor
        assert!(tokens >= 100);
    }

    #[test]
    fn test_count_generic_tokens() {
        let counter = ImageTokenCounter::new();
        let metadata = create_test_metadata(800, 600, 1024 * 1024);

        let tokens = counter
            .count_image_tokens(&metadata, "unknown", "unknown-model")
            .unwrap();

        // Generic should be at least 100
        assert!(tokens >= 100);
    }

    #[test]
    fn test_count_multiple_image_tokens() {
        let counter = ImageTokenCounter::new();
        let images = vec![
            create_test_metadata(800, 600, 1024 * 1024),
            create_test_metadata(1024, 768, 2 * 1024 * 1024),
        ];

        let total = counter
            .count_multiple_image_tokens(&images, "openai", "gpt-4-vision")
            .unwrap();

        // Should be sum of individual tokens
        assert!(total > 0);
    }

    #[test]
    fn test_count_prompt_tokens() {
        let counter = ImageTokenCounter::new();
        let prompt = "Please analyze this image";

        let tokens = counter.count_prompt_tokens(prompt, "gpt-4").unwrap();

        // Should have some tokens
        assert!(tokens > 0);
    }

    #[test]
    fn test_count_total_tokens() {
        let counter = ImageTokenCounter::new();
        let metadata = create_test_metadata(800, 600, 1024 * 1024);
        let prompt = "Please analyze this image";

        let total = counter
            .count_total_tokens(&metadata, prompt, "openai", "gpt-4-vision")
            .unwrap();

        // Should be image tokens + prompt tokens
        assert!(total > 0);
    }

    #[test]
    fn test_count_tokens_different_providers() {
        let counter = ImageTokenCounter::new();
        let metadata = create_test_metadata(800, 600, 1024 * 1024);

        let openai = counter
            .count_image_tokens(&metadata, "openai", "gpt-4-vision")
            .unwrap();
        let anthropic = counter
            .count_image_tokens(&metadata, "anthropic", "claude-3-vision")
            .unwrap();
        let google = counter
            .count_image_tokens(&metadata, "google", "gemini-pro-vision")
            .unwrap();

        // Different providers should have different token counts
        assert_ne!(openai, anthropic);
        assert_ne!(anthropic, google);
    }

    #[test]
    fn test_count_tokens_empty_prompt() {
        let counter = ImageTokenCounter::new();
        let tokens = counter.count_prompt_tokens("", "gpt-4").unwrap();

        // Empty prompt should have 0 tokens
        assert_eq!(tokens, 0);
    }
}
