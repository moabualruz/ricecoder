//! End-to-end integration tests for complete image workflow.
//!
//! Tests the complete image workflow from drag-and-drop through analysis to display:
//! - Complete image workflow (drag-and-drop → analysis → display → prompt)
//! - Multiple images handling
//! - Error recovery
//!
//! **Requirements: 1.1, 2.1, 5.1**

use std::{io::Write, path::PathBuf};

use ricecoder_images::{
    ImageAnalyzer, ImageCache, ImageConfig, ImageDisplay, ImageFormat, ImageHandler, ImageMetadata,
    MessageImageMetadata, MessageImages, SessionImageManager,
};
use tempfile::NamedTempFile;

/// Test complete workflow: drag-and-drop → validation
#[test]
fn test_workflow_drag_drop_validation() {
    let config = ImageConfig::default();
    let handler = ImageHandler::new(config);

    // Create a temporary file
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(b"test data").unwrap();
    temp_file.flush().unwrap();

    let temp_path = temp_file.path().to_string_lossy().to_string();

    // Simulate drag-and-drop
    let (images, errors) = handler.process_drag_drop_event(&temp_path);

    // Should have errors because file is not a valid image
    assert_eq!(errors.len(), 1);
    assert_eq!(images.len(), 0);
}

/// Test complete workflow: multiple files
#[test]
fn test_workflow_multiple_files() {
    let config = ImageConfig::default();
    let handler = ImageHandler::new(config);

    // Create multiple temporary files
    let temp_file1 = NamedTempFile::new().unwrap();
    let temp_file2 = NamedTempFile::new().unwrap();
    let temp_file3 = NamedTempFile::new().unwrap();

    let path1 = temp_file1.path().to_string_lossy().to_string();
    let path2 = temp_file2.path().to_string_lossy().to_string();
    let path3 = temp_file3.path().to_string_lossy().to_string();

    // Simulate drag-and-drop with multiple files
    let event_data = format!("{}\n{}\n{}", path1, path2, path3);
    let (images, errors) = handler.process_drag_drop_event(&event_data);

    // All should fail because they're not valid images
    assert_eq!(errors.len(), 3);
    assert_eq!(images.len(), 0);
}

/// Test complete workflow: display rendering
#[test]
fn test_workflow_display_rendering() {
    let display = ImageDisplay::new();

    let metadata = ImageMetadata::new(
        PathBuf::from("/tmp/test.png"),
        ImageFormat::Png,
        1024 * 1024,
        800,
        600,
        "abc123".to_string(),
    );

    // Render the image
    let rendered = display.render_image(&metadata).unwrap();

    // Should contain metadata
    assert!(rendered.contains("PNG"));
    assert!(rendered.contains("800x600"));
}

/// Test complete workflow: add to prompt context
#[test]
fn test_workflow_add_to_prompt_context() {
    let mut message_images = MessageImages::new();

    let image_meta = MessageImageMetadata::new(
        "hash1".to_string(),
        ImageMetadata::new(
            PathBuf::from("/tmp/test.png"),
            ImageFormat::Png,
            1024,
            800,
            600,
            "hash1".to_string(),
        ),
        None,
        false,
    );

    message_images.add_image(image_meta).unwrap();

    assert_eq!(message_images.image_count(), 1);
    assert!(message_images.has_images());
}

/// Test complete workflow: cache integration
#[test]
fn test_workflow_cache_integration() {
    use chrono::Utc;

    let cache = ImageCache::new().unwrap();

    let hash = "test_hash";
    let analysis = ricecoder_images::ImageAnalysisResult {
        image_hash: hash.to_string(),
        analysis: "Test analysis".to_string(),
        provider: "test".to_string(),
        timestamp: Utc::now(),
        tokens_used: 100,
    };

    // Insert into cache
    cache.set(hash, &analysis).unwrap();

    // Retrieve from cache
    let cached = cache.get(hash).unwrap();
    assert!(cached.is_some());

    let cached_analysis = cached.unwrap();
    assert_eq!(cached_analysis.analysis, "Test analysis");
}

/// Test complete workflow: session storage
#[test]
fn test_workflow_session_storage() {
    let mut session = SessionImageManager::new("session_1".to_string());

    let metadata = ImageMetadata::new(
        PathBuf::from("/tmp/test.png"),
        ImageFormat::Png,
        1024,
        800,
        600,
        "hash1".to_string(),
    );

    session.add_image(metadata, None).unwrap();

    let retrieved = session.get_image("hash1").unwrap();
    assert!(retrieved.is_some());
}

/// Test complete workflow: error recovery
#[test]
fn test_workflow_error_recovery() {
    let config = ImageConfig::default();
    let handler = ImageHandler::new(config);

    // Try with nonexistent file
    let (images, errors) = handler.process_drag_drop_event("/nonexistent/image.png");

    assert_eq!(images.len(), 0);
    assert_eq!(errors.len(), 1);

    // Error should be recoverable
    let error_msg = errors[0].to_string();
    assert!(!error_msg.is_empty());
}

/// Test complete workflow: multiple images with cache
#[test]
fn test_workflow_multiple_images_with_cache() {
    use chrono::Utc;

    let cache = ImageCache::new().unwrap();

    // Add multiple images to cache
    for i in 1..=3 {
        let hash = format!("hash_{}", i);
        let analysis = ricecoder_images::ImageAnalysisResult {
            image_hash: hash.clone(),
            analysis: format!("Analysis {}", i),
            provider: "test".to_string(),
            timestamp: Utc::now(),
            tokens_used: 100 * i as u32,
        };

        cache.set(&hash, &analysis).unwrap();
    }

    // Verify at least one is cached
    let cached = cache.get("hash_1").unwrap();
    assert!(cached.is_some());
}

/// Test complete workflow: multiple images with session
#[test]
fn test_workflow_multiple_images_with_session() {
    let mut session = SessionImageManager::new("session_1".to_string());

    // Add multiple images to session
    for i in 1..=3 {
        let metadata = ImageMetadata::new(
            PathBuf::from(format!("/tmp/test{}.png", i)),
            ImageFormat::Png,
            1024,
            800,
            600,
            format!("hash{}", i),
        );

        session.add_image(metadata, None).unwrap();
    }
}

/// Test complete workflow: display multiple images
#[test]
fn test_workflow_display_multiple_images() {
    let display = ImageDisplay::new();

    // Render multiple images
    for i in 1..=3 {
        let metadata = ImageMetadata::new(
            PathBuf::from(format!("/tmp/test{}.png", i)),
            ImageFormat::Png,
            1024,
            800,
            600,
            format!("hash{}", i),
        );

        let rendered = display.render_image(&metadata).unwrap();
        assert!(rendered.contains("PNG"));
    }
}

/// Test complete workflow: image removal
#[test]
fn test_workflow_image_removal() {
    let mut message_images = MessageImages::new();

    // Add multiple images
    for i in 1..=3 {
        let image_meta = MessageImageMetadata::new(
            format!("hash{}", i),
            ImageMetadata::new(
                PathBuf::from(format!("/tmp/test{}.png", i)),
                ImageFormat::Png,
                1024,
                800,
                600,
                format!("hash{}", i),
            ),
            None,
            false,
        );

        message_images.add_image(image_meta).unwrap();
    }

    assert_eq!(message_images.image_count(), 3);

    // Remove one
    message_images.remove_image("hash2");
    assert_eq!(message_images.image_count(), 2);
}

/// Test complete workflow: cache invalidation
#[test]
fn test_workflow_cache_invalidation() {
    use chrono::Utc;

    let cache = ImageCache::new().unwrap();

    let hash = "test_hash";
    let analysis = ricecoder_images::ImageAnalysisResult {
        image_hash: hash.to_string(),
        analysis: "Test".to_string(),
        provider: "test".to_string(),
        timestamp: Utc::now(),
        tokens_used: 100,
    };

    // Insert
    cache.set(hash, &analysis).unwrap();

    // Verify cached
    let cached = cache.get(hash).unwrap();
    assert!(cached.is_some());

    // Invalidate
    cache.invalidate(hash).unwrap();

    // Verify no longer cached
    let cached = cache.get(hash).unwrap();
    assert!(cached.is_none());
}

/// Test complete workflow: handler creation
#[test]
fn test_workflow_handler_creation() {
    let config = ImageConfig::default();
    let handler = ImageHandler::new(config);

    assert!(handler.config().cache.enabled);
}

/// Test complete workflow: analyzer creation
#[test]
fn test_workflow_analyzer_creation() {
    let analyzer = ImageAnalyzer::new().unwrap();

    assert!(analyzer.config().cache.enabled);
}

/// Test complete workflow: display creation
#[test]
fn test_workflow_display_creation() {
    let display = ImageDisplay::new();

    let metadata = ImageMetadata::new(
        PathBuf::from("/tmp/test.png"),
        ImageFormat::Png,
        1024,
        800,
        600,
        "hash".to_string(),
    );

    let rendered = display.render_image(&metadata).unwrap();
    assert!(!rendered.is_empty());
}

/// Test complete workflow: format validation
#[test]
fn test_workflow_format_validation() {
    let config = ImageConfig::default();
    let handler = ImageHandler::new(config);

    // Validate all supported formats
    assert!(handler.validate_format(ImageFormat::Png).is_ok());
    assert!(handler.validate_format(ImageFormat::Jpeg).is_ok());
    assert!(handler.validate_format(ImageFormat::Gif).is_ok());
    assert!(handler.validate_format(ImageFormat::WebP).is_ok());
}

/// Test complete workflow: path extraction
#[test]
fn test_workflow_path_extraction() {
    let paths = ImageHandler::extract_paths_from_event(
        "/path/to/image1.png\n/path/to/image2.jpg\n/path/to/image3.gif",
    );

    assert_eq!(paths.len(), 3);
}

/// Test complete workflow: file accessibility
#[test]
fn test_workflow_file_accessibility() {
    let temp_file = NamedTempFile::new().unwrap();

    let result = ImageHandler::check_file_accessible(temp_file.path());
    assert!(result.is_ok());

    let result = ImageHandler::check_file_accessible(std::path::Path::new("/nonexistent"));
    assert!(result.is_err());
}

/// Test complete workflow: message images collection
#[test]
fn test_workflow_message_images_collection() {
    let mut message_images = MessageImages::new();

    assert!(!message_images.has_images());

    let image_meta = MessageImageMetadata::new(
        "hash1".to_string(),
        ImageMetadata::new(
            PathBuf::from("/tmp/test.png"),
            ImageFormat::Png,
            1024,
            800,
            600,
            "hash1".to_string(),
        ),
        None,
        false,
    );

    message_images.add_image(image_meta).unwrap();

    assert!(message_images.has_images());
    assert_eq!(message_images.image_count(), 1);
}

/// Test complete workflow: session creation
#[test]
fn test_workflow_session_creation() {
    let session = SessionImageManager::new("session_1".to_string());
    assert_eq!(session.session_id(), "session_1");
}
