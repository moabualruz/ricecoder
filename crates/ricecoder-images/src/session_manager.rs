//! Session manager for image integration with ricecoder-sessions.
//!
//! This module provides functionality to:
//! - Store and retrieve images from session context
//! - Manage image references in sessions
//! - Support image sharing between sessions
//! - Persist image metadata with sessions

use std::collections::HashMap;

use crate::{
    error::{ImageError, ImageResult},
    models::{ImageAnalysisResult, ImageMetadata},
    session_integration::{MessageImageMetadata, SessionImageContext},
};

/// Manages images within a session
#[derive(Debug, Clone)]
pub struct SessionImageManager {
    /// Session ID
    session_id: String,
    /// Image context for the session
    image_context: SessionImageContext,
    /// Map of image hash to analysis results for quick lookup
    analysis_cache: HashMap<String, ImageAnalysisResult>,
}

impl SessionImageManager {
    /// Create a new session image manager
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            image_context: SessionImageContext::new(),
            analysis_cache: HashMap::new(),
        }
    }

    /// Create a session image manager from persisted context
    pub fn from_context(session_id: String, context: SessionImageContext) -> Self {
        Self {
            session_id,
            image_context: context,
            analysis_cache: HashMap::new(),
        }
    }

    /// Get the session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Add an image to the session context
    ///
    /// # Arguments
    ///
    /// * `metadata` - The image metadata
    /// * `analysis` - Optional analysis result
    ///
    /// # Returns
    ///
    /// The image hash
    pub fn add_image(
        &mut self,
        metadata: ImageMetadata,
        analysis: Option<ImageAnalysisResult>,
    ) -> ImageResult<String> {
        let hash = metadata.hash.clone();

        // Add to context
        self.image_context.add_image(hash.clone(), metadata);

        // Cache analysis if provided
        if let Some(analysis_result) = analysis {
            self.analysis_cache.insert(hash.clone(), analysis_result);
        }

        Ok(hash)
    }

    /// Remove an image from the session context
    ///
    /// Note: The image is removed from current context but remains in history
    pub fn remove_image(&mut self, hash: &str) -> ImageResult<()> {
        self.image_context.remove_image(hash);
        Ok(())
    }

    /// Get an image from the session
    pub fn get_image(&self, hash: &str) -> ImageResult<Option<MessageImageMetadata>> {
        if let Some(metadata) = self.image_context.get_image_metadata(hash) {
            let analysis = self.analysis_cache.get(hash).cloned();
            let was_cached = analysis.is_some();

            Ok(Some(MessageImageMetadata::new(
                hash.to_string(),
                metadata.clone(),
                analysis,
                was_cached,
            )))
        } else {
            Ok(None)
        }
    }

    /// Get all current images in the session
    pub fn get_current_images(&self) -> Vec<MessageImageMetadata> {
        self.image_context
            .get_current_images()
            .into_iter()
            .map(|metadata| {
                let hash = metadata.hash.clone();
                let analysis = self.analysis_cache.get(&hash).cloned();
                let was_cached = analysis.is_some();

                MessageImageMetadata::new(hash, metadata.clone(), analysis, was_cached)
            })
            .collect()
    }

    /// Get all images ever included in the session
    pub fn get_all_images(&self) -> Vec<MessageImageMetadata> {
        self.image_context
            .get_all_images()
            .into_iter()
            .map(|metadata| {
                let hash = metadata.hash.clone();
                let analysis = self.analysis_cache.get(&hash).cloned();
                let was_cached = analysis.is_some();

                MessageImageMetadata::new(hash, metadata.clone(), analysis, was_cached)
            })
            .collect()
    }

    /// Get the number of current images
    pub fn current_image_count(&self) -> usize {
        self.image_context.current_image_count()
    }

    /// Get the total number of images ever included
    pub fn total_image_count(&self) -> usize {
        self.image_context.total_image_count()
    }

    /// Check if an image is in the current context
    pub fn has_image(&self, hash: &str) -> bool {
        self.image_context.has_image(hash)
    }

    /// Clear current images (but keep history)
    pub fn clear_current(&mut self) -> ImageResult<()> {
        self.image_context.clear_current();
        Ok(())
    }

    /// Clear all images including history
    pub fn clear_all(&mut self) -> ImageResult<()> {
        self.image_context.clear_all();
        self.analysis_cache.clear();
        Ok(())
    }

    /// Get the image context for persistence
    pub fn get_context_for_persistence(&self) -> SessionImageContext {
        self.image_context.clone()
    }

    /// Restore image context from persistence
    pub fn restore_from_persistence(&mut self, context: SessionImageContext) {
        self.image_context = context;
    }

    /// Get image hashes in current context
    pub fn get_current_image_hashes(&self) -> Vec<String> {
        self.image_context.current_images.clone()
    }

    /// Get all image hashes ever included
    pub fn get_all_image_hashes(&self) -> Vec<String> {
        self.image_context.all_images.clone()
    }

    /// Update analysis for an image
    pub fn update_analysis(
        &mut self,
        hash: &str,
        analysis: ImageAnalysisResult,
    ) -> ImageResult<()> {
        if !self.image_context.has_image(hash) {
            return Err(ImageError::InvalidFile(format!(
                "Image with hash {} not found in session",
                hash
            )));
        }

        self.analysis_cache.insert(hash.to_string(), analysis);
        Ok(())
    }

    /// Get analysis for an image
    pub fn get_analysis(&self, hash: &str) -> Option<&ImageAnalysisResult> {
        self.analysis_cache.get(hash)
    }

    /// Get all analyses
    pub fn get_all_analyses(&self) -> Vec<&ImageAnalysisResult> {
        self.analysis_cache.values().collect()
    }
}

/// Manages images across multiple sessions
#[derive(Debug, Clone)]
pub struct MultiSessionImageManager {
    /// Map of session ID to session image manager
    sessions: HashMap<String, SessionImageManager>,
}

impl MultiSessionImageManager {
    /// Create a new multi-session image manager
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    /// Create a new session
    pub fn create_session(&mut self, session_id: String) -> ImageResult<()> {
        if self.sessions.contains_key(&session_id) {
            return Err(ImageError::InvalidFile(format!(
                "Session {} already exists",
                session_id
            )));
        }

        self.sessions
            .insert(session_id.clone(), SessionImageManager::new(session_id));
        Ok(())
    }

    /// Get a session manager
    pub fn get_session(&self, session_id: &str) -> ImageResult<&SessionImageManager> {
        self.sessions
            .get(session_id)
            .ok_or_else(|| ImageError::InvalidFile(format!("Session {} not found", session_id)))
    }

    /// Get a mutable session manager
    pub fn get_session_mut(&mut self, session_id: &str) -> ImageResult<&mut SessionImageManager> {
        self.sessions
            .get_mut(session_id)
            .ok_or_else(|| ImageError::InvalidFile(format!("Session {} not found", session_id)))
    }

    /// Remove a session
    pub fn remove_session(&mut self, session_id: &str) -> ImageResult<()> {
        self.sessions
            .remove(session_id)
            .ok_or_else(|| ImageError::InvalidFile(format!("Session {} not found", session_id)))?;
        Ok(())
    }

    /// Get all session IDs
    pub fn get_session_ids(&self) -> Vec<String> {
        self.sessions.keys().cloned().collect()
    }

    /// Share an image between sessions
    ///
    /// Copies an image from one session to another
    pub fn share_image(
        &mut self,
        from_session: &str,
        to_session: &str,
        image_hash: &str,
    ) -> ImageResult<()> {
        // Get the image from source session
        let image = self
            .get_session(from_session)?
            .get_image(image_hash)?
            .ok_or_else(|| {
                ImageError::InvalidFile(format!(
                    "Image {} not found in session {}",
                    image_hash, from_session
                ))
            })?;

        // Add to target session
        let target_manager = self.get_session_mut(to_session)?;
        target_manager.add_image(image.metadata, image.analysis)?;

        Ok(())
    }

    /// Get the number of sessions
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }
}

impl Default for MultiSessionImageManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

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

    fn create_test_analysis() -> ImageAnalysisResult {
        ImageAnalysisResult::new(
            "test_hash_123".to_string(),
            "This is a test image".to_string(),
            "openai".to_string(),
            100,
        )
    }

    #[test]
    fn test_session_image_manager_new() {
        let manager = SessionImageManager::new("session1".to_string());
        assert_eq!(manager.session_id(), "session1");
        assert_eq!(manager.current_image_count(), 0);
    }

    #[test]
    fn test_session_image_manager_add_image() {
        let mut manager = SessionImageManager::new("session1".to_string());
        let metadata = create_test_image_metadata();

        let hash = manager.add_image(metadata, None).unwrap();
        assert_eq!(hash, "test_hash_123");
        assert_eq!(manager.current_image_count(), 1);
    }

    #[test]
    fn test_session_image_manager_add_with_analysis() {
        let mut manager = SessionImageManager::new("session1".to_string());
        let metadata = create_test_image_metadata();
        let analysis = create_test_analysis();

        manager.add_image(metadata, Some(analysis)).unwrap();
        assert_eq!(manager.current_image_count(), 1);

        let image = manager.get_image("test_hash_123").unwrap().unwrap();
        assert!(image.analysis.is_some());
    }

    #[test]
    fn test_session_image_manager_remove_image() {
        let mut manager = SessionImageManager::new("session1".to_string());
        let metadata = create_test_image_metadata();

        manager.add_image(metadata, None).unwrap();
        assert_eq!(manager.current_image_count(), 1);

        manager.remove_image("test_hash_123").unwrap();
        assert_eq!(manager.current_image_count(), 0);
        assert_eq!(manager.total_image_count(), 1); // Still in history
    }

    #[test]
    fn test_session_image_manager_get_image() {
        let mut manager = SessionImageManager::new("session1".to_string());
        let metadata = create_test_image_metadata();

        manager.add_image(metadata, None).unwrap();

        let image = manager.get_image("test_hash_123").unwrap();
        assert!(image.is_some());
        assert_eq!(image.unwrap().hash, "test_hash_123");
    }

    #[test]
    fn test_session_image_manager_clear_current() {
        let mut manager = SessionImageManager::new("session1".to_string());
        let metadata = create_test_image_metadata();

        manager.add_image(metadata, None).unwrap();
        manager.clear_current().unwrap();

        assert_eq!(manager.current_image_count(), 0);
        assert_eq!(manager.total_image_count(), 1);
    }

    #[test]
    fn test_session_image_manager_clear_all() {
        let mut manager = SessionImageManager::new("session1".to_string());
        let metadata = create_test_image_metadata();

        manager.add_image(metadata, None).unwrap();
        manager.clear_all().unwrap();

        assert_eq!(manager.current_image_count(), 0);
        assert_eq!(manager.total_image_count(), 0);
    }

    #[test]
    fn test_multi_session_image_manager_create_session() {
        let mut manager = MultiSessionImageManager::new();
        assert!(manager.create_session("session1".to_string()).is_ok());
        assert_eq!(manager.session_count(), 1);
    }

    #[test]
    fn test_multi_session_image_manager_duplicate_session() {
        let mut manager = MultiSessionImageManager::new();
        assert!(manager.create_session("session1".to_string()).is_ok());
        assert!(manager.create_session("session1".to_string()).is_err());
    }

    #[test]
    fn test_multi_session_image_manager_get_session() {
        let mut manager = MultiSessionImageManager::new();
        manager.create_session("session1".to_string()).unwrap();

        let session = manager.get_session("session1");
        assert!(session.is_ok());
    }

    #[test]
    fn test_multi_session_image_manager_remove_session() {
        let mut manager = MultiSessionImageManager::new();
        manager.create_session("session1".to_string()).unwrap();
        assert_eq!(manager.session_count(), 1);

        manager.remove_session("session1").unwrap();
        assert_eq!(manager.session_count(), 0);
    }

    #[test]
    fn test_multi_session_image_manager_share_image() {
        let mut manager = MultiSessionImageManager::new();
        manager.create_session("session1".to_string()).unwrap();
        manager.create_session("session2".to_string()).unwrap();

        let metadata = create_test_image_metadata();
        manager
            .get_session_mut("session1")
            .unwrap()
            .add_image(metadata, None)
            .unwrap();

        manager
            .share_image("session1", "session2", "test_hash_123")
            .unwrap();

        assert_eq!(
            manager
                .get_session("session2")
                .unwrap()
                .current_image_count(),
            1
        );
    }

    #[test]
    fn test_session_image_manager_update_analysis() {
        let mut manager = SessionImageManager::new("session1".to_string());
        let metadata = create_test_image_metadata();

        manager.add_image(metadata, None).unwrap();

        let analysis = create_test_analysis();
        manager.update_analysis("test_hash_123", analysis).unwrap();

        let image = manager.get_image("test_hash_123").unwrap().unwrap();
        assert!(image.analysis.is_some());
    }

    #[test]
    fn test_session_image_manager_persistence() {
        let mut manager1 = SessionImageManager::new("session1".to_string());
        let metadata = create_test_image_metadata();

        manager1.add_image(metadata, None).unwrap();

        let context = manager1.get_context_for_persistence();

        let manager2 = SessionImageManager::from_context("session1".to_string(), context);
        assert_eq!(manager2.current_image_count(), 1);
    }
}
