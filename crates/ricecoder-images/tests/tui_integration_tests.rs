//! Integration tests for TUI drag-and-drop and image display.
//!
//! Tests the integration between ricecoder-images and ricecoder-tui:
//! - Drag-and-drop event handling
//! - Image display in terminal
//! - Image removal from prompt
//!
//! **Requirements: 1.1, 5.1**

use ricecoder_images::{ImageConfig, ImageDisplay, ImageHandler};
use std::io::Write;
use std::path::PathBuf;
use tempfile::NamedTempFile;

/// Test drag-and-drop event handling with single file
#[test]
fn test_drag_drop_single_file() {
    let config = ImageConfig::default();
    let handler = ImageHandler::new(config);

    // Create a temporary file
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(b"test").unwrap();
    temp_file.flush().unwrap();

    let temp_path = temp_file.path().to_string_lossy().to_string();

    // Simulate drag-and-drop event with single file
    let (images, errors) = handler.process_drag_drop_event(&temp_path);

    // Should have one error (invalid image format) but no successful images
    // because the file doesn't contain valid image data
    assert_eq!(errors.len(), 1);
    assert_eq!(images.len(), 0);
}

/// Test drag-and-drop event handling with multiple files
#[test]
fn test_drag_drop_multiple_files() {
    let config = ImageConfig::default();
    let handler = ImageHandler::new(config);

    // Create multiple temporary files
    let temp_file1 = NamedTempFile::new().unwrap();
    let temp_file2 = NamedTempFile::new().unwrap();
    let temp_file3 = NamedTempFile::new().unwrap();

    let path1 = temp_file1.path().to_string_lossy().to_string();
    let path2 = temp_file2.path().to_string_lossy().to_string();
    let path3 = temp_file3.path().to_string_lossy().to_string();

    // Simulate drag-and-drop event with multiple files (newline-separated)
    let event_data = format!("{}\n{}\n{}", path1, path2, path3);
    let (images, errors) = handler.process_drag_drop_event(&event_data);

    // All files should fail because they don't contain valid image data
    assert_eq!(errors.len(), 3);
    assert_eq!(images.len(), 0);
}

/// Test drag-and-drop event handling with space-separated paths
#[test]
fn test_drag_drop_space_separated() {
    let config = ImageConfig::default();
    let handler = ImageHandler::new(config);

    // Create temporary files
    let temp_file1 = NamedTempFile::new().unwrap();
    let temp_file2 = NamedTempFile::new().unwrap();

    let path1 = temp_file1.path().to_string_lossy().to_string();
    let path2 = temp_file2.path().to_string_lossy().to_string();

    // Simulate drag-and-drop event with space-separated paths
    let event_data = format!("{} {}", path1, path2);
    let (images, errors) = handler.process_drag_drop_event(&event_data);

    // Both files should fail because they don't contain valid image data
    assert_eq!(errors.len(), 2);
    assert_eq!(images.len(), 0);
}

/// Test drag-and-drop event handling with nonexistent files
#[test]
fn test_drag_drop_nonexistent_files() {
    let config = ImageConfig::default();
    let handler = ImageHandler::new(config);

    // Simulate drag-and-drop event with nonexistent files
    let event_data = "/nonexistent/image1.png\n/nonexistent/image2.jpg";
    let (images, errors) = handler.process_drag_drop_event(event_data);

    // Both files should fail
    assert_eq!(errors.len(), 2);
    assert_eq!(images.len(), 0);

    // Errors should indicate file not found
    for error in errors {
        let error_msg = error.to_string();
        assert!(
            error_msg.contains("does not exist") || error_msg.contains("Invalid"),
            "Error should indicate file not found"
        );
    }
}

/// Test drag-and-drop event handling with empty event data
#[test]
fn test_drag_drop_empty_event() {
    let config = ImageConfig::default();
    let handler = ImageHandler::new(config);

    // Simulate drag-and-drop event with empty data
    let (images, errors) = handler.process_drag_drop_event("");

    // Should have no images or errors
    assert_eq!(images.len(), 0);
    assert_eq!(errors.len(), 0);
}

/// Test image display rendering
#[test]
fn test_image_display_rendering() {
    let display = ImageDisplay::new();

    // Create a mock image metadata
    let metadata = ricecoder_images::ImageMetadata::new(
        PathBuf::from("/tmp/test.png"),
        ricecoder_images::ImageFormat::Png,
        1024 * 1024, // 1 MB
        800,
        600,
        "abc123def456".to_string(),
    );

    // Render the image
    let result = display.render_image(&metadata);
    assert!(result.is_ok());

    let rendered = result.unwrap();

    // Check that rendered output contains metadata
    assert!(rendered.contains("PNG"));
    assert!(rendered.contains("800x600"));
    assert!(rendered.contains("1.0 MB"));
}

/// Test image display with multiple images
#[test]
fn test_image_display_multiple_images() {
    let display = ImageDisplay::new();

    // Create multiple mock image metadata
    let metadata1 = ricecoder_images::ImageMetadata::new(
        PathBuf::from("/tmp/test1.png"),
        ricecoder_images::ImageFormat::Png,
        1024 * 1024,
        800,
        600,
        "hash1".to_string(),
    );

    let metadata2 = ricecoder_images::ImageMetadata::new(
        PathBuf::from("/tmp/test2.jpg"),
        ricecoder_images::ImageFormat::Jpeg,
        2048 * 1024,
        1024,
        768,
        "hash2".to_string(),
    );

    // Render both images
    let result1 = display.render_image(&metadata1);
    let result2 = display.render_image(&metadata2);

    assert!(result1.is_ok());
    assert!(result2.is_ok());

    let rendered1 = result1.unwrap();
    let rendered2 = result2.unwrap();

    // Check that both contain their respective metadata
    assert!(rendered1.contains("PNG"));
    assert!(rendered1.contains("800x600"));

    assert!(rendered2.contains("JPG"));
    assert!(rendered2.contains("1024x768"));
}

/// Test image display with ASCII placeholder
#[test]
fn test_image_display_ascii_placeholder() {
    let display = ImageDisplay::new();

    let metadata = ricecoder_images::ImageMetadata::new(
        PathBuf::from("/tmp/test.png"),
        ricecoder_images::ImageFormat::Png,
        1024 * 1024,
        800,
        600,
        "abc123".to_string(),
    );

    let rendered = display.render_image(&metadata).unwrap();

    // Check that ASCII placeholder is included
    assert!(rendered.contains("█") || rendered.contains("*") || rendered.contains("#"));
}

/// Test image removal from prompt context
#[test]
fn test_image_removal_from_prompt() {
    use ricecoder_images::MessageImages;

    let mut message_images = MessageImages::new();

    // Create and add an image
    let image_meta = ricecoder_images::MessageImageMetadata::new(
        "hash1".to_string(),
        ricecoder_images::ImageMetadata::new(
            PathBuf::from("/tmp/test.png"),
            ricecoder_images::ImageFormat::Png,
            1024 * 1024,
            800,
            600,
            "hash1".to_string(),
        ),
        None,
        false,
    );

    message_images.add_image(image_meta).unwrap();
    assert_eq!(message_images.image_count(), 1);

    // Remove the image
    let removed = message_images.remove_image("hash1");
    assert!(removed.is_some());
    assert_eq!(message_images.image_count(), 0);
}

/// Test image removal with multiple images
#[test]
fn test_image_removal_multiple_images() {
    use ricecoder_images::MessageImages;

    let mut message_images = MessageImages::new();

    // Add multiple images
    for i in 1..=3 {
        let image_meta = ricecoder_images::MessageImageMetadata::new(
            format!("hash{}", i),
            ricecoder_images::ImageMetadata::new(
                PathBuf::from(format!("/tmp/test{}.png", i)),
                ricecoder_images::ImageFormat::Png,
                1024 * 1024,
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

    // Remove one image
    let removed = message_images.remove_image("hash2");
    assert!(removed.is_some());
    assert_eq!(message_images.image_count(), 2);

    // Verify the correct image was removed
    assert!(message_images.get_image("hash1").is_some());
    assert!(message_images.get_image("hash2").is_none());
    assert!(message_images.get_image("hash3").is_some());
}

/// Test image removal with nonexistent hash
#[test]
fn test_image_removal_nonexistent() {
    use ricecoder_images::MessageImages;

    let mut message_images = MessageImages::new();

    // Try to remove an image that doesn't exist
    let removed = message_images.remove_image("nonexistent");
    assert!(removed.is_none());
}

/// Test drag-and-drop event path extraction
#[test]
fn test_drag_drop_path_extraction() {
    // Test single path
    let paths = ImageHandler::extract_paths_from_event("/path/to/image.png");
    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0], PathBuf::from("/path/to/image.png"));

    // Test multiple paths (newline-separated)
    let paths = ImageHandler::extract_paths_from_event("/path/to/image1.png\n/path/to/image2.jpg");
    assert_eq!(paths.len(), 2);

    // Test multiple paths (space-separated)
    let paths = ImageHandler::extract_paths_from_event("/path/to/image1.png /path/to/image2.jpg");
    assert_eq!(paths.len(), 2);

    // Test empty
    let paths = ImageHandler::extract_paths_from_event("");
    assert_eq!(paths.len(), 0);
}

/// Test file accessibility check
#[test]
fn test_file_accessibility_check() {
    // Create a temporary file
    let temp_file = NamedTempFile::new().unwrap();
    let temp_path = temp_file.path();

    // Should be accessible
    let result = ImageHandler::check_file_accessible(temp_path);
    assert!(result.is_ok());

    // Nonexistent file should not be accessible
    let result = ImageHandler::check_file_accessible(std::path::Path::new("/nonexistent/file.png"));
    assert!(result.is_err());
}

/// Test drag-and-drop with mixed valid and invalid files
#[test]
fn test_drag_drop_mixed_files() {
    let config = ImageConfig::default();
    let handler = ImageHandler::new(config);

    // Create one valid file and reference one nonexistent file
    let temp_file = NamedTempFile::new().unwrap();
    let temp_path = temp_file.path().to_string_lossy().to_string();

    let event_data = format!("{}\n/nonexistent/image.png", temp_path);
    let (images, errors) = handler.process_drag_drop_event(&event_data);

    // One file exists but is invalid, one doesn't exist
    assert_eq!(errors.len(), 2);
    assert_eq!(images.len(), 0);
}

/// Test image display configuration
#[test]
fn test_image_display_configuration() {
    let config = ricecoder_images::DisplayConfig {
        max_width: 80,
        max_height: 30,
        placeholder_char: "█".to_string(),
    };

    let display = ImageDisplay::with_config(config);

    let metadata = ricecoder_images::ImageMetadata::new(
        PathBuf::from("/tmp/test.png"),
        ricecoder_images::ImageFormat::Png,
        1024 * 1024,
        800,
        600,
        "abc123".to_string(),
    );

    let rendered = display.render_image(&metadata).unwrap();
    assert!(!rendered.is_empty());
}

/// Test message images collection
#[test]
fn test_message_images_collection() {
    use ricecoder_images::MessageImages;

    let mut message_images = MessageImages::new();

    // Initially empty
    assert_eq!(message_images.image_count(), 0);
    assert!(!message_images.has_images());

    // Add an image
    let image_meta = ricecoder_images::MessageImageMetadata::new(
        "hash1".to_string(),
        ricecoder_images::ImageMetadata::new(
            PathBuf::from("/tmp/test.png"),
            ricecoder_images::ImageFormat::Png,
            1024 * 1024,
            800,
            600,
            "hash1".to_string(),
        ),
        None,
        false,
    );

    message_images.add_image(image_meta).unwrap();

    // Now has images
    assert_eq!(message_images.image_count(), 1);
    assert!(message_images.has_images());

    // Get all images
    let all_images = message_images.get_all_images();
    assert_eq!(all_images.len(), 1);
}

/// Test duplicate image prevention
#[test]
fn test_duplicate_image_prevention() {
    use ricecoder_images::MessageImages;

    let mut message_images = MessageImages::new();

    let image_meta = ricecoder_images::MessageImageMetadata::new(
        "hash1".to_string(),
        ricecoder_images::ImageMetadata::new(
            PathBuf::from("/tmp/test.png"),
            ricecoder_images::ImageFormat::Png,
            1024 * 1024,
            800,
            600,
            "hash1".to_string(),
        ),
        None,
        false,
    );

    // Add the image
    let result1 = message_images.add_image(image_meta.clone());
    assert!(result1.is_ok());

    // Try to add the same image again
    let result2 = message_images.add_image(image_meta);
    assert!(result2.is_err());

    // Should still have only one image
    assert_eq!(message_images.image_count(), 1);
}
