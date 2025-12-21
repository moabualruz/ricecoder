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
