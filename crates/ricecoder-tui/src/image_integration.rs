//! Image integration for ricecoder-tui
//!
//! This module provides integration between ricecoder-tui and ricecoder-images,
//! handling drag-and-drop events and image display in the terminal UI.
//!
//! # Requirements
//!
//! - Req 1.1: Detect drag-and-drop events and pass to ricecoder-images handler
//! - Req 5.1: Display images in terminal using ricecoder-images ImageDisplay
//! - Req 1.4: Add images to prompt context

use std::path::PathBuf;

/// Image integration manager for ricecoder-tui
///
/// Handles:
/// - Drag-and-drop event detection and forwarding to ricecoder-images
/// - Image display coordination with ricecoder-images ImageDisplay
/// - Image context management for prompts
///
/// # Requirements
///
/// - Req 1.1: Create interface for receiving drag-and-drop events from ricecoder-tui
/// - Req 1.1: Implement file path extraction from events
/// - Req 1.1: Handle multiple files in single drag-and-drop
pub struct ImageIntegration {
    /// Whether image integration is enabled
    pub enabled: bool,
    /// Maximum number of images per prompt
    pub max_images_per_prompt: usize,
    /// Current images in the prompt context
    pub current_images: Vec<PathBuf>,
}

impl ImageIntegration {
    /// Create a new image integration manager
    pub fn new() -> Self {
        Self {
            enabled: true,
            max_images_per_prompt: 10,
            current_images: Vec::new(),
        }
    }

    /// Handle a drag-and-drop event
    ///
    /// # Arguments
    ///
    /// * `paths` - File paths from the drag-and-drop event
    ///
    /// # Returns
    ///
    /// Vector of successfully added image paths and any errors
    ///
    /// # Requirements
    ///
    /// - Req 1.1: Handle multiple files in single drag-and-drop
    /// - Req 1.1: Implement file existence and permission checks
    pub fn handle_drag_drop_event(&mut self, paths: Vec<PathBuf>) -> (Vec<PathBuf>, Vec<String>) {
        let mut added = Vec::new();
        let mut errors = Vec::new();

        for path in paths {
            // Check if we've reached the maximum number of images
            if self.current_images.len() >= self.max_images_per_prompt {
                errors.push(format!(
                    "Maximum number of images ({}) reached",
                    self.max_images_per_prompt
                ));
                break;
            }

            // Check if image is already in the context
            if self.current_images.contains(&path) {
                errors.push(format!("Image already in context: {}", path.display()));
                continue;
            }

            // Check if file exists and is readable
            if !path.exists() {
                errors.push(format!("File does not exist: {}", path.display()));
                continue;
            }

            if !path.is_file() {
                errors.push(format!("Path is not a file: {}", path.display()));
                continue;
            }

            // Add to current images
            self.current_images.push(path.clone());
            added.push(path);
        }

        (added, errors)
    }

    /// Remove an image from the prompt context
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the image to remove
    ///
    /// # Returns
    ///
    /// True if image was removed, false if not found
    pub fn remove_image(&mut self, path: &PathBuf) -> bool {
        if let Some(pos) = self.current_images.iter().position(|p| p == path) {
            self.current_images.remove(pos);
            true
        } else {
            false
        }
    }

    /// Clear all images from the prompt context
    pub fn clear_images(&mut self) {
        self.current_images.clear();
    }

    /// Get the current images in the prompt context
    pub fn get_images(&self) -> &[PathBuf] {
        &self.current_images
    }

    /// Check if there are any images in the prompt context
    pub fn has_images(&self) -> bool {
        !self.current_images.is_empty()
    }

    /// Get the number of images in the prompt context
    pub fn image_count(&self) -> usize {
        self.current_images.len()
    }

    /// Enable image integration
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable image integration
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Set the maximum number of images per prompt
    pub fn set_max_images(&mut self, max: usize) {
        self.max_images_per_prompt = max;
    }
}

impl Default for ImageIntegration {
    fn default() -> Self {
        Self::new()
    }
}
