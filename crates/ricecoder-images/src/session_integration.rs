//! Session integration for storing and retrieving images in session history and context.
//!
//! This module provides functionality to:
//! - Store images in message history with metadata
//! - Persist image metadata with sessions
//! - Store image references in session context
//! - Support image sharing in sessions

use crate::error::{ImageError, ImageResult};
use crate::models::{ImageAnalysisResult, ImageMetadata};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metadata about an image included in a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageImageMetadata {
    /// SHA256 hash of the image (unique identifier)
    pub hash: String,
    /// Image metadata (path, format, size, dimensions)
    pub metadata: ImageMetadata,
    /// Analysis result if available
    pub analysis: Option<ImageAnalysisResult>,
    /// Whether the image was cached
    pub was_cached: bool,
}

impl MessageImageMetadata {
    /// Create new message image metadata
    pub fn new(
        hash: String,
        metadata: ImageMetadata,
        analysis: Option<ImageAnalysisResult>,
        was_cached: bool,
    ) -> Self {
        Self {
            hash,
            metadata,
            analysis,
            was_cached,
        }
    }
}

/// Images included in a message
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessageImages {
    /// Map of image hash to image metadata
    images: HashMap<String, MessageImageMetadata>,
}

impl MessageImages {
    /// Create a new empty message images collection
    pub fn new() -> Self {
        Self {
            images: HashMap::new(),
        }
    }

    /// Add an image to the message
    ///
    /// # Arguments
    ///
    /// * `image_meta` - The image metadata to add
    ///
    /// # Returns
    ///
    /// Ok if the image was added, Err if an image with the same hash already exists
    pub fn add_image(&mut self, image_meta: MessageImageMetadata) -> ImageResult<()> {
        if self.images.contains_key(&image_meta.hash) {
            return Err(ImageError::InvalidFile(format!(
                "Image with hash {} already exists in message",
                image_meta.hash
            )));
        }

        self.images.insert(image_meta.hash.clone(), image_meta);
        Ok(())
    }

    /// Get an image by hash
    pub fn get_image(&self, hash: &str) -> Option<&MessageImageMetadata> {
        self.images.get(hash)
    }

    /// Get all images in the message
    pub fn get_all_images(&self) -> Vec<&MessageImageMetadata> {
        self.images.values().collect()
    }

    /// Get the number of images in the message
    pub fn image_count(&self) -> usize {
        self.images.len()
    }

    /// Check if the message has any images
    pub fn has_images(&self) -> bool {
        !self.images.is_empty()
    }

    /// Remove an image by hash
    pub fn remove_image(&mut self, hash: &str) -> Option<MessageImageMetadata> {
        self.images.remove(hash)
    }

    /// Get all image hashes
    pub fn get_image_hashes(&self) -> Vec<String> {
        self.images.keys().cloned().collect()
    }

    /// Get all images as a vector
    pub fn to_vec(&self) -> Vec<MessageImageMetadata> {
        self.images.values().cloned().collect()
    }

    /// Clear all images from the message
    pub fn clear(&mut self) {
        self.images.clear();
    }
}

/// Session context for images
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionImageContext {
    /// Images currently in the session context (by hash)
    pub current_images: Vec<String>,
    /// All images ever included in this session (by hash)
    pub all_images: Vec<String>,
    /// Map of image hash to metadata for quick lookup
    pub image_metadata: HashMap<String, ImageMetadata>,
}

impl SessionImageContext {
    /// Create a new session image context
    pub fn new() -> Self {
        Self {
            current_images: Vec::new(),
            all_images: Vec::new(),
            image_metadata: HashMap::new(),
        }
    }

    /// Add an image to the current session context
    ///
    /// # Arguments
    ///
    /// * `hash` - The image hash
    /// * `metadata` - The image metadata
    pub fn add_image(&mut self, hash: String, metadata: ImageMetadata) {
        if !self.current_images.contains(&hash) {
            self.current_images.push(hash.clone());
        }

        if !self.all_images.contains(&hash) {
            self.all_images.push(hash.clone());
        }

        self.image_metadata.insert(hash, metadata);
    }

    /// Remove an image from the current session context
    ///
    /// Note: The image is removed from current context but remains in all_images
    pub fn remove_image(&mut self, hash: &str) {
        self.current_images.retain(|h| h != hash);
    }

    /// Get the current images in the session
    pub fn get_current_images(&self) -> Vec<&ImageMetadata> {
        self.current_images
            .iter()
            .filter_map(|hash| self.image_metadata.get(hash))
            .collect()
    }

    /// Get all images ever included in the session
    pub fn get_all_images(&self) -> Vec<&ImageMetadata> {
        self.all_images
            .iter()
            .filter_map(|hash| self.image_metadata.get(hash))
            .collect()
    }

    /// Get image metadata by hash
    pub fn get_image_metadata(&self, hash: &str) -> Option<&ImageMetadata> {
        self.image_metadata.get(hash)
    }

    /// Check if an image is in the current context
    pub fn has_image(&self, hash: &str) -> bool {
        self.current_images.iter().any(|h| h == hash)
    }

    /// Get the number of current images
    pub fn current_image_count(&self) -> usize {
        self.current_images.len()
    }

    /// Get the total number of images ever included
    pub fn total_image_count(&self) -> usize {
        self.all_images.len()
    }

    /// Clear current images (but keep history)
    pub fn clear_current(&mut self) {
        self.current_images.clear();
    }

    /// Clear all images including history
    pub fn clear_all(&mut self) {
        self.current_images.clear();
        self.all_images.clear();
        self.image_metadata.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_image_metadata() -> ImageMetadata {
        ImageMetadata::new(
            PathBuf::from("/path/to/image.png"),
            crate::formats::ImageFormat::Png,
            1024 * 1024,
            800,
            600,
            "test_hash_123".to_string(),
        )
    }

    fn create_test_message_image_metadata() -> MessageImageMetadata {
        MessageImageMetadata::new(
            "test_hash_123".to_string(),
            create_test_image_metadata(),
            None,
            false,
        )
    }

    #[test]
    fn test_message_images_add() {
        let mut msg_images = MessageImages::new();
        let image_meta = create_test_message_image_metadata();

        assert!(msg_images.add_image(image_meta.clone()).is_ok());
        assert_eq!(msg_images.image_count(), 1);
        assert!(msg_images.has_images());
    }

    #[test]
    fn test_message_images_duplicate() {
        let mut msg_images = MessageImages::new();
        let image_meta = create_test_message_image_metadata();

        assert!(msg_images.add_image(image_meta.clone()).is_ok());
        assert!(msg_images.add_image(image_meta).is_err());
        assert_eq!(msg_images.image_count(), 1);
    }

    #[test]
    fn test_message_images_get() {
        let mut msg_images = MessageImages::new();
        let image_meta = create_test_message_image_metadata();

        msg_images.add_image(image_meta.clone()).unwrap();

        let retrieved = msg_images.get_image("test_hash_123");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().hash, "test_hash_123");
    }

    #[test]
    fn test_message_images_remove() {
        let mut msg_images = MessageImages::new();
        let image_meta = create_test_message_image_metadata();

        msg_images.add_image(image_meta).unwrap();
        assert_eq!(msg_images.image_count(), 1);

        let removed = msg_images.remove_image("test_hash_123");
        assert!(removed.is_some());
        assert_eq!(msg_images.image_count(), 0);
    }

    #[test]
    fn test_message_images_get_all() {
        let mut msg_images = MessageImages::new();

        for i in 0..3 {
            let mut image_meta = create_test_message_image_metadata();
            image_meta.hash = format!("hash_{}", i);
            msg_images.add_image(image_meta).unwrap();
        }

        let all = msg_images.get_all_images();
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn test_session_image_context_add() {
        let mut ctx = SessionImageContext::new();
        let metadata = create_test_image_metadata();

        ctx.add_image("hash1".to_string(), metadata);

        assert_eq!(ctx.current_image_count(), 1);
        assert_eq!(ctx.total_image_count(), 1);
        assert!(ctx.has_image("hash1"));
    }

    #[test]
    fn test_session_image_context_remove() {
        let mut ctx = SessionImageContext::new();
        let metadata = create_test_image_metadata();

        ctx.add_image("hash1".to_string(), metadata);
        assert_eq!(ctx.current_image_count(), 1);

        ctx.remove_image("hash1");
        assert_eq!(ctx.current_image_count(), 0);
        assert_eq!(ctx.total_image_count(), 1); // Still in history
    }

    #[test]
    fn test_session_image_context_history() {
        let mut ctx = SessionImageContext::new();
        let metadata = create_test_image_metadata();

        ctx.add_image("hash1".to_string(), metadata.clone());
        ctx.add_image("hash2".to_string(), metadata.clone());

        ctx.remove_image("hash1");

        assert_eq!(ctx.current_image_count(), 1);
        assert_eq!(ctx.total_image_count(), 2);
        assert!(!ctx.has_image("hash1"));
        assert!(ctx.has_image("hash2"));
    }

    #[test]
    fn test_session_image_context_clear_current() {
        let mut ctx = SessionImageContext::new();
        let metadata = create_test_image_metadata();

        ctx.add_image("hash1".to_string(), metadata);
        ctx.clear_current();

        assert_eq!(ctx.current_image_count(), 0);
        assert_eq!(ctx.total_image_count(), 1);
    }

    #[test]
    fn test_session_image_context_clear_all() {
        let mut ctx = SessionImageContext::new();
        let metadata = create_test_image_metadata();

        ctx.add_image("hash1".to_string(), metadata);
        ctx.clear_all();

        assert_eq!(ctx.current_image_count(), 0);
        assert_eq!(ctx.total_image_count(), 0);
    }

    #[test]
    fn test_session_image_context_get_metadata() {
        let mut ctx = SessionImageContext::new();
        let metadata = create_test_image_metadata();

        ctx.add_image("hash1".to_string(), metadata.clone());

        let retrieved = ctx.get_image_metadata("hash1");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().width, 800);
    }
}
