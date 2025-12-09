//! Prompt context integration with images
//!
//! This module provides integration between the prompt input and images,
//! managing the context that includes both text and image data.
//!
//! # Requirements
//!
//! - Req 1.4: Add images to prompt context
//! - Req 5.1: Display images in chat interface
//! - Req 5.1: Include images in message history

use std::path::PathBuf;

/// Prompt context containing text and images
///
/// Manages the context for a prompt including:
/// - Text input
/// - Associated images
/// - Metadata about the context
///
/// # Requirements
///
/// - Req 1.4: Add images to prompt context
/// - Req 5.1: Display images in chat interface
/// - Req 5.1: Include images in message history
#[derive(Debug, Clone)]
pub struct PromptContext {
    /// Text content of the prompt
    pub text: String,
    /// Images associated with the prompt
    pub images: Vec<PathBuf>,
    /// Whether the context is complete and ready to send
    pub ready: bool,
    /// Timestamp when context was created
    pub created_at: std::time::SystemTime,
}

impl PromptContext {
    /// Create a new empty prompt context
    pub fn new() -> Self {
        Self {
            text: String::new(),
            images: Vec::new(),
            ready: false,
            created_at: std::time::SystemTime::now(),
        }
    }

    /// Create a new prompt context with text
    pub fn with_text(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            images: Vec::new(),
            ready: false,
            created_at: std::time::SystemTime::now(),
        }
    }

    /// Create a new prompt context with text and images
    pub fn with_text_and_images(text: impl Into<String>, images: Vec<PathBuf>) -> Self {
        Self {
            text: text.into(),
            images,
            ready: false,
            created_at: std::time::SystemTime::now(),
        }
    }

    /// Set the text content
    ///
    /// # Arguments
    ///
    /// * `text` - The text content
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    /// Get the text content
    pub fn get_text(&self) -> &str {
        &self.text
    }

    /// Add an image to the context
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the image
    ///
    /// # Requirements
    ///
    /// - Req 1.4: Add images to prompt context
    pub fn add_image(&mut self, path: PathBuf) {
        if !self.images.contains(&path) {
            self.images.push(path);
        }
    }

    /// Add multiple images to the context
    ///
    /// # Arguments
    ///
    /// * `paths` - Paths to the images
    ///
    /// # Requirements
    ///
    /// - Req 1.4: Add images to prompt context
    pub fn add_images(&mut self, paths: Vec<PathBuf>) {
        for path in paths {
            self.add_image(path);
        }
    }

    /// Remove an image from the context
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the image to remove
    ///
    /// # Returns
    ///
    /// True if image was removed, false if not found
    pub fn remove_image(&mut self, path: &PathBuf) -> bool {
        if let Some(pos) = self.images.iter().position(|p| p == path) {
            self.images.remove(pos);
            true
        } else {
            false
        }
    }

    /// Clear all images from the context
    pub fn clear_images(&mut self) {
        self.images.clear();
    }

    /// Get the images in the context
    pub fn get_images(&self) -> &[PathBuf] {
        &self.images
    }

    /// Get the number of images in the context
    pub fn image_count(&self) -> usize {
        self.images.len()
    }

    /// Check if the context has any images
    pub fn has_images(&self) -> bool {
        !self.images.is_empty()
    }

    /// Check if the context has text
    pub fn has_text(&self) -> bool {
        !self.text.is_empty()
    }

    /// Check if the context is complete (has text or images)
    pub fn is_complete(&self) -> bool {
        self.has_text() || self.has_images()
    }

    /// Mark the context as ready to send
    pub fn mark_ready(&mut self) {
        self.ready = true;
    }

    /// Mark the context as not ready
    pub fn mark_not_ready(&mut self) {
        self.ready = false;
    }

    /// Check if the context is ready to send
    pub fn is_ready(&self) -> bool {
        self.ready && self.is_complete()
    }

    /// Clear the context (text and images)
    pub fn clear(&mut self) {
        self.text.clear();
        self.images.clear();
        self.ready = false;
        self.created_at = std::time::SystemTime::now();
    }

    /// Get a summary of the context
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();

        if self.has_text() {
            parts.push(format!("Text: {} chars", self.text.len()));
        }

        if self.has_images() {
            parts.push(format!("Images: {}", self.image_count()));
        }

        if parts.is_empty() {
            "Empty context".to_string()
        } else {
            parts.join(", ")
        }
    }
}

impl Default for PromptContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_context_creation() {
        let context = PromptContext::new();
        assert_eq!(context.text, "");
        assert_eq!(context.images.len(), 0);
        assert!(!context.ready);
    }

    #[test]
    fn test_prompt_context_with_text() {
        let context = PromptContext::with_text("Hello, world!");
        assert_eq!(context.text, "Hello, world!");
        assert_eq!(context.images.len(), 0);
    }

    #[test]
    fn test_prompt_context_with_text_and_images() {
        let images = vec![
            PathBuf::from("/path/to/image1.png"),
            PathBuf::from("/path/to/image2.jpg"),
        ];
        let context = PromptContext::with_text_and_images("Hello", images.clone());

        assert_eq!(context.text, "Hello");
        assert_eq!(context.images.len(), 2);
        assert_eq!(context.images, images);
    }

    #[test]
    fn test_set_text() {
        let mut context = PromptContext::new();
        context.set_text("New text");
        assert_eq!(context.get_text(), "New text");
    }

    #[test]
    fn test_add_image() {
        let mut context = PromptContext::new();
        let path = PathBuf::from("/path/to/image.png");

        context.add_image(path.clone());
        assert_eq!(context.image_count(), 1);
        assert!(context.has_images());
        assert_eq!(context.get_images()[0], path);
    }

    #[test]
    fn test_add_duplicate_image() {
        let mut context = PromptContext::new();
        let path = PathBuf::from("/path/to/image.png");

        context.add_image(path.clone());
        context.add_image(path.clone());

        // Should not add duplicate
        assert_eq!(context.image_count(), 1);
    }

    #[test]
    fn test_add_multiple_images() {
        let mut context = PromptContext::new();
        let paths = vec![
            PathBuf::from("/path/to/image1.png"),
            PathBuf::from("/path/to/image2.jpg"),
        ];

        context.add_images(paths.clone());
        assert_eq!(context.image_count(), 2);
    }

    #[test]
    fn test_remove_image() {
        let mut context = PromptContext::new();
        let path = PathBuf::from("/path/to/image.png");

        context.add_image(path.clone());
        assert_eq!(context.image_count(), 1);

        let removed = context.remove_image(&path);
        assert!(removed);
        assert_eq!(context.image_count(), 0);
    }

    #[test]
    fn test_remove_image_not_found() {
        let mut context = PromptContext::new();
        let path = PathBuf::from("/path/to/image.png");

        let removed = context.remove_image(&path);
        assert!(!removed);
    }

    #[test]
    fn test_clear_images() {
        let mut context = PromptContext::new();
        context.add_images(vec![
            PathBuf::from("/path/to/image1.png"),
            PathBuf::from("/path/to/image2.jpg"),
        ]);

        assert_eq!(context.image_count(), 2);
        context.clear_images();
        assert_eq!(context.image_count(), 0);
    }

    #[test]
    fn test_has_text() {
        let mut context = PromptContext::new();
        assert!(!context.has_text());

        context.set_text("Some text");
        assert!(context.has_text());
    }

    #[test]
    fn test_has_images() {
        let mut context = PromptContext::new();
        assert!(!context.has_images());

        context.add_image(PathBuf::from("/path/to/image.png"));
        assert!(context.has_images());
    }

    #[test]
    fn test_is_complete() {
        let mut context = PromptContext::new();
        assert!(!context.is_complete());

        context.set_text("Some text");
        assert!(context.is_complete());

        context.clear();
        assert!(!context.is_complete());

        context.add_image(PathBuf::from("/path/to/image.png"));
        assert!(context.is_complete());
    }

    #[test]
    fn test_ready_state() {
        let mut context = PromptContext::new();
        assert!(!context.is_ready());

        context.mark_ready();
        assert!(!context.is_ready()); // Still not ready because context is empty

        context.set_text("Some text");
        assert!(context.is_ready());

        context.mark_not_ready();
        assert!(!context.is_ready());
    }

    #[test]
    fn test_clear() {
        let mut context = PromptContext::new();
        context.set_text("Some text");
        context.add_image(PathBuf::from("/path/to/image.png"));
        context.mark_ready();

        assert!(context.has_text());
        assert!(context.has_images());
        assert!(context.ready);

        context.clear();

        assert!(!context.has_text());
        assert!(!context.has_images());
        assert!(!context.ready);
    }

    #[test]
    fn test_summary() {
        let mut context = PromptContext::new();
        assert_eq!(context.summary(), "Empty context");

        context.set_text("Hello, world!");
        assert!(context.summary().contains("Text"));

        context.add_image(PathBuf::from("/path/to/image.png"));
        let summary = context.summary();
        assert!(summary.contains("Text"));
        assert!(summary.contains("Images"));
    }

    #[test]
    fn test_image_count() {
        let mut context = PromptContext::new();
        assert_eq!(context.image_count(), 0);

        context.add_images(vec![
            PathBuf::from("/path/to/image1.png"),
            PathBuf::from("/path/to/image2.jpg"),
            PathBuf::from("/path/to/image3.gif"),
        ]);

        assert_eq!(context.image_count(), 3);
    }

    #[test]
    fn test_get_images() {
        let mut context = PromptContext::new();
        let paths = vec![
            PathBuf::from("/path/to/image1.png"),
            PathBuf::from("/path/to/image2.jpg"),
        ];

        context.add_images(paths.clone());
        assert_eq!(context.get_images(), paths.as_slice());
    }
}
