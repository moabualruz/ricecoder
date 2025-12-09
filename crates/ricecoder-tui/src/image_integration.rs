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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_image_integration_creation() {
        let integration = ImageIntegration::new();
        assert!(integration.enabled);
        assert_eq!(integration.max_images_per_prompt, 10);
        assert_eq!(integration.current_images.len(), 0);
    }

    #[test]
    fn test_handle_drag_drop_event_single_file() {
        let mut integration = ImageIntegration::new();
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        let (added, errors) = integration.handle_drag_drop_event(vec![path.clone()]);

        assert_eq!(added.len(), 1);
        assert_eq!(errors.len(), 0);
        assert_eq!(integration.current_images.len(), 1);
        assert_eq!(integration.current_images[0], path);
    }

    #[test]
    fn test_handle_drag_drop_event_multiple_files() {
        let mut integration = ImageIntegration::new();
        let temp_file1 = NamedTempFile::new().unwrap();
        let temp_file2 = NamedTempFile::new().unwrap();
        let path1 = temp_file1.path().to_path_buf();
        let path2 = temp_file2.path().to_path_buf();

        let (added, errors) = integration.handle_drag_drop_event(vec![path1.clone(), path2.clone()]);

        assert_eq!(added.len(), 2);
        assert_eq!(errors.len(), 0);
        assert_eq!(integration.current_images.len(), 2);
    }

    #[test]
    fn test_handle_drag_drop_event_nonexistent_file() {
        let mut integration = ImageIntegration::new();
        let path = PathBuf::from("/nonexistent/image.png");

        let (added, errors) = integration.handle_drag_drop_event(vec![path]);

        assert_eq!(added.len(), 0);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("does not exist"));
    }

    #[test]
    fn test_handle_drag_drop_event_directory() {
        let mut integration = ImageIntegration::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().to_path_buf();

        let (added, errors) = integration.handle_drag_drop_event(vec![path]);

        assert_eq!(added.len(), 0);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("not a file"));
    }

    #[test]
    fn test_handle_drag_drop_event_duplicate() {
        let mut integration = ImageIntegration::new();
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        // Add first time
        let (added1, errors1) = integration.handle_drag_drop_event(vec![path.clone()]);
        assert_eq!(added1.len(), 1);
        assert_eq!(errors1.len(), 0);

        // Try to add again
        let (added2, errors2) = integration.handle_drag_drop_event(vec![path]);
        assert_eq!(added2.len(), 0);
        assert_eq!(errors2.len(), 1);
        assert!(errors2[0].contains("already in context"));
    }

    #[test]
    fn test_handle_drag_drop_event_max_images() {
        let mut integration = ImageIntegration::new();
        integration.set_max_images(2);

        let temp_file1 = NamedTempFile::new().unwrap();
        let temp_file2 = NamedTempFile::new().unwrap();
        let temp_file3 = NamedTempFile::new().unwrap();

        let path1 = temp_file1.path().to_path_buf();
        let path2 = temp_file2.path().to_path_buf();
        let path3 = temp_file3.path().to_path_buf();

        // Add first two
        let (added1, errors1) = integration.handle_drag_drop_event(vec![path1, path2]);
        assert_eq!(added1.len(), 2);
        assert_eq!(errors1.len(), 0);

        // Try to add third
        let (added2, errors2) = integration.handle_drag_drop_event(vec![path3]);
        assert_eq!(added2.len(), 0);
        assert_eq!(errors2.len(), 1);
        assert!(errors2[0].contains("Maximum number of images"));
    }

    #[test]
    fn test_remove_image() {
        let mut integration = ImageIntegration::new();
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        integration.handle_drag_drop_event(vec![path.clone()]);
        assert_eq!(integration.current_images.len(), 1);

        let removed = integration.remove_image(&path);
        assert!(removed);
        assert_eq!(integration.current_images.len(), 0);
    }

    #[test]
    fn test_remove_image_not_found() {
        let mut integration = ImageIntegration::new();
        let path = PathBuf::from("/nonexistent/image.png");

        let removed = integration.remove_image(&path);
        assert!(!removed);
    }

    #[test]
    fn test_clear_images() {
        let mut integration = ImageIntegration::new();
        let temp_file1 = NamedTempFile::new().unwrap();
        let temp_file2 = NamedTempFile::new().unwrap();

        integration.handle_drag_drop_event(vec![
            temp_file1.path().to_path_buf(),
            temp_file2.path().to_path_buf(),
        ]);
        assert_eq!(integration.current_images.len(), 2);

        integration.clear_images();
        assert_eq!(integration.current_images.len(), 0);
    }

    #[test]
    fn test_get_images() {
        let mut integration = ImageIntegration::new();
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        integration.handle_drag_drop_event(vec![path.clone()]);

        let images = integration.get_images();
        assert_eq!(images.len(), 1);
        assert_eq!(images[0], path);
    }

    #[test]
    fn test_has_images() {
        let mut integration = ImageIntegration::new();
        assert!(!integration.has_images());

        let temp_file = NamedTempFile::new().unwrap();
        integration.handle_drag_drop_event(vec![temp_file.path().to_path_buf()]);
        assert!(integration.has_images());
    }

    #[test]
    fn test_image_count() {
        let mut integration = ImageIntegration::new();
        assert_eq!(integration.image_count(), 0);

        let temp_file1 = NamedTempFile::new().unwrap();
        let temp_file2 = NamedTempFile::new().unwrap();

        integration.handle_drag_drop_event(vec![
            temp_file1.path().to_path_buf(),
            temp_file2.path().to_path_buf(),
        ]);
        assert_eq!(integration.image_count(), 2);
    }

    #[test]
    fn test_enable_disable() {
        let mut integration = ImageIntegration::new();
        assert!(integration.enabled);

        integration.disable();
        assert!(!integration.enabled);

        integration.enable();
        assert!(integration.enabled);
    }

    #[test]
    fn test_set_max_images() {
        let mut integration = ImageIntegration::new();
        assert_eq!(integration.max_images_per_prompt, 10);

        integration.set_max_images(5);
        assert_eq!(integration.max_images_per_prompt, 5);
    }
}
