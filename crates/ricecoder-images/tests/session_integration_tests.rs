//! Integration tests for session image storage and retrieval.
//!
//! Tests the integration between ricecoder-images and ricecoder-sessions:
//! - Image storage in sessions
//! - Image retrieval from sessions
//! - Image persistence in session history
//!
//! **Requirements: 1.4, 2.1**

use ricecoder_images::{
    SessionImageManager, MessageImages, MessageImageMetadata,
    ImageMetadata, ImageFormat,
};
use std::path::PathBuf;

/// Test session image manager creation
#[test]
fn test_session_image_manager_creation() {
    let manager = SessionImageManager::new("session_1".to_string());
    assert_eq!(manager.session_id(), "session_1");
}

/// Test adding image to session
#[test]
fn test_add_image_to_session() {
    let mut manager = SessionImageManager::new("session_1".to_string());

    let metadata = ImageMetadata::new(
        PathBuf::from("/tmp/test.png"),
        ImageFormat::Png,
        1024,
        800,
        600,
        "hash1".to_string(),
    );

    let result = manager.add_image(metadata, None);
    assert!(result.is_ok());
}

/// Test retrieving image from session
#[test]
fn test_retrieve_image_from_session() {
    let mut manager = SessionImageManager::new("session_1".to_string());

    let metadata = ImageMetadata::new(
        PathBuf::from("/tmp/test.png"),
        ImageFormat::Png,
        1024,
        800,
        600,
        "hash1".to_string(),
    );

    manager.add_image(metadata, None).unwrap();

    let retrieved = manager.get_image("hash1").unwrap();
    assert!(retrieved.is_some());

    let image = retrieved.unwrap();
    assert_eq!(image.hash, "hash1");
}

/// Test retrieving nonexistent image from session
#[test]
fn test_retrieve_nonexistent_image() {
    let manager = SessionImageManager::new("session_1".to_string());

    let retrieved = manager.get_image("nonexistent").unwrap();
    assert!(retrieved.is_none());
}

/// Test removing image from session
#[test]
fn test_remove_image_from_session() {
    let mut manager = SessionImageManager::new("session_1".to_string());

    let metadata = ImageMetadata::new(
        PathBuf::from("/tmp/test.png"),
        ImageFormat::Png,
        1024,
        800,
        600,
        "hash1".to_string(),
    );

    manager.add_image(metadata, None).unwrap();

    // Remove the image
    let removed = manager.remove_image("hash1");
    assert!(removed.is_ok());
}

/// Test session image manager add multiple images
#[test]
fn test_session_image_manager_add_multiple() {
    let mut manager = SessionImageManager::new("session_1".to_string());

    for i in 1..=3 {
        let metadata = ImageMetadata::new(
            PathBuf::from(format!("/tmp/test{}.png", i)),
            ImageFormat::Png,
            1024,
            800,
            600,
            format!("hash{}", i),
        );

        let result = manager.add_image(metadata, None);
        assert!(result.is_ok());
    }
}

/// Test message images collection
#[test]
fn test_message_images_collection() {
    let mut message_images = MessageImages::new();

    assert_eq!(message_images.image_count(), 0);
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

    assert_eq!(message_images.image_count(), 1);
    assert!(message_images.has_images());
}

/// Test session image manager with different formats
#[test]
fn test_session_images_different_formats() {
    let mut manager = SessionImageManager::new("session_1".to_string());

    let formats = vec![
        ImageFormat::Png,
        ImageFormat::Jpeg,
        ImageFormat::Gif,
        ImageFormat::WebP,
    ];

    for (i, format) in formats.iter().enumerate() {
        let metadata = ImageMetadata::new(
            PathBuf::from(format!("/tmp/test{}.img", i)),
            *format,
            1024,
            800,
            600,
            format!("hash{}", i),
        );

        let result = manager.add_image(metadata, None);
        assert!(result.is_ok());
    }
}

/// Test session image manager with different sizes
#[test]
fn test_session_images_different_sizes() {
    let mut manager = SessionImageManager::new("session_1".to_string());

    let sizes = vec![1024, 1024 * 1024, 5 * 1024 * 1024, 10 * 1024 * 1024];

    for (i, size) in sizes.iter().enumerate() {
        let metadata = ImageMetadata::new(
            PathBuf::from(format!("/tmp/test{}.png", i)),
            ImageFormat::Png,
            *size,
            800,
            600,
            format!("hash{}", i),
        );

        let result = manager.add_image(metadata, None);
        assert!(result.is_ok());
    }
}

/// Test session image manager with different dimensions
#[test]
fn test_session_images_different_dimensions() {
    let mut manager = SessionImageManager::new("session_1".to_string());

    let dimensions = vec![
        (640, 480),
        (800, 600),
        (1024, 768),
        (1920, 1080),
    ];

    for (i, (width, height)) in dimensions.iter().enumerate() {
        let metadata = ImageMetadata::new(
            PathBuf::from(format!("/tmp/test{}.png", i)),
            ImageFormat::Png,
            1024,
            *width,
            *height,
            format!("hash{}", i),
        );

        let result = manager.add_image(metadata, None);
        assert!(result.is_ok());
    }
}

/// Test message image metadata creation
#[test]
fn test_message_image_metadata_creation() {
    let metadata = ImageMetadata::new(
        PathBuf::from("/tmp/test.png"),
        ImageFormat::Png,
        1024,
        800,
        600,
        "hash".to_string(),
    );

    let image_meta = MessageImageMetadata::new(
        "hash".to_string(),
        metadata,
        None,
        false,
    );

    assert_eq!(image_meta.hash, "hash");
    assert!(!image_meta.was_cached);
    assert!(image_meta.analysis.is_none());
}

/// Test message images collection with multiple images
#[test]
fn test_message_images_multiple() {
    let mut message_images = MessageImages::new();

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
}

/// Test message images removal
#[test]
fn test_message_images_removal() {
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

    let removed = message_images.remove_image("hash1");
    assert!(removed.is_some());
    assert_eq!(message_images.image_count(), 0);
}

/// Test session image manager get session id
#[test]
fn test_session_image_manager_get_session_id() {
    let manager = SessionImageManager::new("my_session".to_string());
    assert_eq!(manager.session_id(), "my_session");
}

/// Test message images get all
#[test]
fn test_message_images_get_all() {
    let mut message_images = MessageImages::new();

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

    let all_images = message_images.get_all_images();
    assert_eq!(all_images.len(), 3);
}

/// Test message images get specific
#[test]
fn test_message_images_get_specific() {
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

    let retrieved = message_images.get_image("hash1");
    assert!(retrieved.is_some());
}

/// Test session image manager from context
#[test]
fn test_session_image_manager_from_context() {
    let context = ricecoder_images::SessionImageContext::new();
    let manager = SessionImageManager::from_context("session_1".to_string(), context);
    assert_eq!(manager.session_id(), "session_1");
}
