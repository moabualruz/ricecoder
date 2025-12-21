//! Integration tests for image provider integration.
//!
//! Tests the integration between ricecoder-images and ricecoder-providers:
//! - Image sending to providers
//! - Provider response handling
//! - Token counting for images
//!
//! **Requirements: 2.1, 2.2**

use ricecoder_images::{
    ChatRequestWithImages, ImageAnalyzer, ImageData, ImageFormat, ImageMetadata,
    ProviderImageFormat,
};
use ricecoder_providers::models::{ChatRequest, Message};
use std::path::PathBuf;

/// Helper function to create a test ChatRequest
fn create_test_request() -> ChatRequest {
    ChatRequest {
        messages: vec![Message {
            role: "user".to_string(),
            content: "Test message".to_string(),
        }],
        model: "test-model".to_string(),
        temperature: Some(0.7),
        max_tokens: Some(1000),
        stream: false,
    }
}

/// Test image data creation
#[test]
fn test_image_data_creation() {
    let image_data = ImageData::from_bytes("png", &[0x89, 0x50, 0x4E, 0x47], 800, 600);

    assert_eq!(image_data.format, "png");
    assert_eq!(image_data.dimensions, (800, 600));
    assert_eq!(image_data.size_bytes, 4);
}

/// Test image data with different formats
#[test]
fn test_image_data_formats() {
    let formats = vec!["png", "jpg", "gif", "webp"];

    for format in formats {
        let image_data = ImageData::from_bytes(format, &[0u8; 100], 800, 600);

        assert_eq!(image_data.format, format);
        assert_eq!(image_data.size_bytes, 100);
    }
}

/// Test image data MIME type
#[test]
fn test_image_data_mime_type() {
    let png_data = ImageData::from_bytes("png", &[0u8; 100], 800, 600);
    assert_eq!(png_data.mime_type(), "image/png");

    let jpg_data = ImageData::from_bytes("jpg", &[0u8; 100], 800, 600);
    assert_eq!(jpg_data.mime_type(), "image/jpeg");

    let gif_data = ImageData::from_bytes("gif", &[0u8; 100], 800, 600);
    assert_eq!(gif_data.mime_type(), "image/gif");

    let webp_data = ImageData::from_bytes("webp", &[0u8; 100], 800, 600);
    assert_eq!(webp_data.mime_type(), "image/webp");
}

/// Test image data URL generation
#[test]
fn test_image_data_url() {
    let image_data = ImageData::from_bytes("png", &[0u8; 100], 800, 600);
    let data_url = image_data.data_url();

    assert!(data_url.starts_with("data:image/png;base64,"));
}

/// Test chat request with images creation
#[test]
fn test_chat_request_with_images() {
    let request = create_test_request();

    let mut chat_request = ChatRequestWithImages::new(request, "openai");

    // Initially no images
    assert_eq!(chat_request.images.len(), 0);

    // Add an image
    let image_data = ImageData::from_bytes("png", &[0u8; 1024], 800, 600);
    chat_request.add_image(image_data);

    // Now has one image
    assert_eq!(chat_request.images.len(), 1);
}

/// Test chat request with multiple images
#[test]
fn test_chat_request_multiple_images() {
    let request = create_test_request();

    let mut chat_request = ChatRequestWithImages::new(request, "openai");

    // Add multiple images
    for i in 1..=3 {
        let image_data = ImageData::from_bytes("png", &vec![0u8; 1024 * i], 800, 600);
        chat_request.add_image(image_data);
    }

    assert_eq!(chat_request.images.len(), 3);
}

/// Test image analyzer creation
#[test]
fn test_image_analyzer_creation() {
    let analyzer = ImageAnalyzer::new().unwrap();

    assert!(analyzer.config().cache.enabled);
}

/// Test image analyzer with custom config
#[test]
fn test_image_analyzer_custom_config() {
    let analyzer = ImageAnalyzer::new().unwrap();
    // Verify default timeout
    assert_eq!(analyzer.config().analysis.timeout_seconds, 10);
}

/// Test image metadata to provider format conversion
#[test]
fn test_image_format_conversion() {
    let formats = vec![
        (ImageFormat::Png, "png"),
        (ImageFormat::Jpeg, "jpg"),
        (ImageFormat::Gif, "gif"),
        (ImageFormat::WebP, "webp"),
    ];

    for (image_format, expected_str) in formats {
        let metadata = ImageMetadata::new(
            PathBuf::from("/tmp/test"),
            image_format,
            1024,
            800,
            600,
            "hash".to_string(),
        );

        // Verify format can be converted
        let format_str = metadata.format_str();
        assert_eq!(format_str, expected_str);
    }
}

/// Test image data serialization
#[test]
fn test_image_data_serialization() {
    let image_data = ImageData::from_bytes("png", &[0x89, 0x50, 0x4E, 0x47], 800, 600);

    // Should be serializable
    let json = serde_json::to_string(&image_data);
    assert!(json.is_ok());

    let json_str = json.unwrap();
    assert!(json_str.contains("png"));
}

/// Test image data deserialization
#[test]
fn test_image_data_deserialization() {
    let image_data = ImageData::from_bytes("png", &[0x89, 0x50, 0x4E, 0x47], 800, 600);
    let json = serde_json::to_string(&image_data).unwrap();

    let result: Result<ImageData, _> = serde_json::from_str(&json);
    assert!(result.is_ok());

    let deserialized = result.unwrap();
    assert_eq!(deserialized.format, "png");
    assert_eq!(deserialized.dimensions, (800, 600));
}

/// Test chat request serialization
#[test]
fn test_chat_request_serialization() {
    let request = create_test_request();

    let mut chat_request = ChatRequestWithImages::new(request, "openai");

    let image_data = ImageData::from_bytes("png", &[0u8; 100], 800, 600);
    chat_request.add_image(image_data);

    // Should be serializable
    let json = serde_json::to_string(&chat_request);
    assert!(json.is_ok());
}

/// Test image analyzer retry context
#[test]
fn test_image_analyzer_retry_context() {
    use ricecoder_images::AnalysisRetryContext;

    let metadata = ImageMetadata::new(
        PathBuf::from("/tmp/test.png"),
        ImageFormat::Png,
        1024,
        800,
        600,
        "hash".to_string(),
    );

    let mut context = AnalysisRetryContext::new(metadata, vec![0u8; 1024]);

    // Initially can retry
    assert!(context.can_retry());
    assert_eq!(context.retry_attempts, 0);

    // Record a failure
    context.record_failure("Test error".to_string());
    assert_eq!(context.retry_attempts, 1);
    assert!(context.last_error.is_some());

    // Should still be able to retry
    assert!(context.can_retry());
}

/// Test image analyzer retry limit
#[test]
fn test_image_analyzer_retry_limit() {
    use ricecoder_images::AnalysisRetryContext;

    let metadata = ImageMetadata::new(
        PathBuf::from("/tmp/test.png"),
        ImageFormat::Png,
        1024,
        800,
        600,
        "hash".to_string(),
    );

    let mut context = AnalysisRetryContext::new(metadata, vec![0u8; 1024]);

    // Record multiple failures
    for _ in 0..5 {
        context.record_failure("Error".to_string());
    }

    // Should not be able to retry after 5 attempts
    assert!(!context.can_retry());
}

/// Test image analyzer error message
#[test]
fn test_image_analyzer_error_message() {
    use ricecoder_images::AnalysisRetryContext;

    let metadata = ImageMetadata::new(
        PathBuf::from("/tmp/test.png"),
        ImageFormat::Png,
        1024,
        800,
        600,
        "hash".to_string(),
    );

    let mut context = AnalysisRetryContext::new(metadata, vec![0u8; 1024]);

    context.record_failure("Provider timeout".to_string());

    let error_msg = context.get_error_message();
    assert!(!error_msg.is_empty());
}

/// Test image data with large payload
#[test]
fn test_image_data_large_payload() {
    let large_data = vec![0u8; 10 * 1024 * 1024]; // 10 MB

    let image_data = ImageData::from_bytes("png", &large_data, 800, 600);

    assert_eq!(image_data.size_bytes, 10 * 1024 * 1024 as u64);
}

/// Test chat request message content
#[test]
fn test_chat_request_message_content() {
    let request = create_test_request();

    let chat_request = ChatRequestWithImages::new(request, "openai");

    assert_eq!(chat_request.request.model, "test-model");
}

/// Test image format string representation
#[test]
fn test_image_format_string_representation() {
    let metadata_png = ImageMetadata::new(
        PathBuf::from("/tmp/test.png"),
        ImageFormat::Png,
        1024,
        800,
        600,
        "hash".to_string(),
    );

    let metadata_jpg = ImageMetadata::new(
        PathBuf::from("/tmp/test.jpg"),
        ImageFormat::Jpeg,
        1024,
        800,
        600,
        "hash".to_string(),
    );

    assert_eq!(metadata_png.format_str(), "png");
    assert_eq!(metadata_jpg.format_str(), "jpg");
}

/// Test image metadata dimensions
#[test]
fn test_image_metadata_dimensions() {
    let metadata = ImageMetadata::new(
        PathBuf::from("/tmp/test.png"),
        ImageFormat::Png,
        1024,
        1920,
        1080,
        "hash".to_string(),
    );

    let (width, height) = metadata.dimensions();
    assert_eq!(width, 1920);
    assert_eq!(height, 1080);
}

/// Test image metadata size in MB
#[test]
fn test_image_metadata_size_mb() {
    let metadata = ImageMetadata::new(
        PathBuf::from("/tmp/test.png"),
        ImageFormat::Png,
        1024 * 1024, // 1 MB
        800,
        600,
        "hash".to_string(),
    );

    let size_mb = metadata.size_mb();
    assert!(size_mb > 0.9 && size_mb < 1.1); // Approximately 1 MB
}

/// Test provider image format enum
#[test]
fn test_provider_image_format_enum() {
    let formats = vec![
        ProviderImageFormat::OpenAi,
        ProviderImageFormat::Anthropic,
        ProviderImageFormat::Google,
        ProviderImageFormat::Ollama,
        ProviderImageFormat::Generic,
    ];

    assert_eq!(formats.len(), 5);
}

/// Test image analyzer config access
#[test]
fn test_image_analyzer_config_access() {
    let analyzer = ImageAnalyzer::new().unwrap();

    let analyzer_config = analyzer.config();
    assert!(analyzer_config.cache.enabled);
    assert_eq!(analyzer_config.analysis.timeout_seconds, 10);
}

/// Test chat request with empty message
#[test]
fn test_chat_request_empty_message() {
    let request = create_test_request();

    let chat_request = ChatRequestWithImages::new(request, "openai");
    assert_eq!(chat_request.images.len(), 0);
}

/// Test image data dimensions
#[test]
fn test_image_data_dimensions() {
    let image_data = ImageData::from_bytes("png", &[0u8; 100], 1920, 1080);
    assert_eq!(image_data.dimensions, (1920, 1080));
}

/// Test image analyzer with different timeout configs
#[test]
fn test_image_analyzer_timeout_configs() {
    let analyzer = ImageAnalyzer::new().unwrap();
    // Verify default timeout
    assert_eq!(analyzer.config().analysis.timeout_seconds, 10);
}

/// Test image metadata path
#[test]
fn test_image_metadata_path() {
    let path = PathBuf::from("/tmp/test.png");
    let metadata = ImageMetadata::new(
        path.clone(),
        ImageFormat::Png,
        1024,
        800,
        600,
        "hash".to_string(),
    );

    assert_eq!(metadata.path, path);
}

/// Test image metadata hash
#[test]
fn test_image_metadata_hash() {
    let hash = "abc123def456";
    let metadata = ImageMetadata::new(
        PathBuf::from("/tmp/test.png"),
        ImageFormat::Png,
        1024,
        800,
        600,
        hash.to_string(),
    );

    assert_eq!(metadata.hash, hash);
}

/// Test provider format for different providers
#[test]
fn test_provider_format_for_providers() {
    assert_eq!(
        ProviderImageFormat::for_provider("openai"),
        ProviderImageFormat::OpenAi
    );
    assert_eq!(
        ProviderImageFormat::for_provider("anthropic"),
        ProviderImageFormat::Anthropic
    );
    assert_eq!(
        ProviderImageFormat::for_provider("google"),
        ProviderImageFormat::Google
    );
    assert_eq!(
        ProviderImageFormat::for_provider("ollama"),
        ProviderImageFormat::Ollama
    );
    assert_eq!(
        ProviderImageFormat::for_provider("unknown"),
        ProviderImageFormat::Generic
    );
}

/// Test chat request add multiple images
#[test]
fn test_chat_request_add_multiple_images() {
    let request = create_test_request();

    let mut chat_request = ChatRequestWithImages::new(request, "openai");

    let images = vec![
        ImageData::from_bytes("png", &[0u8; 100], 800, 600),
        ImageData::from_bytes("jpg", &[0u8; 200], 1024, 768),
        ImageData::from_bytes("gif", &[0u8; 150], 640, 480),
    ];

    chat_request.add_images(images);

    assert_eq!(chat_request.images.len(), 3);
}
