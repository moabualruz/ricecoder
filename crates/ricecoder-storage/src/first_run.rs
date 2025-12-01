//! First-run initialization and storage confirmation
//!
//! Handles first-time initialization of RiceCoder storage,
//! including user confirmation of storage location.

use crate::error::StorageResult;
use crate::manager::PathResolver;
use std::fs;
use std::path::PathBuf;

/// Marker file name for tracking first-run status
const FIRST_RUN_MARKER: &str = ".ricecoder-initialized";

/// First-run handler for storage initialization
pub struct FirstRunHandler;

impl FirstRunHandler {
    /// Check if this is the first run
    ///
    /// Returns true if the marker file doesn't exist in the global storage path
    pub fn is_first_run(global_path: &PathBuf) -> StorageResult<bool> {
        let marker_path = global_path.join(FIRST_RUN_MARKER);
        Ok(!marker_path.exists())
    }

    /// Mark the first run as complete
    ///
    /// Creates the marker file to indicate initialization is done
    pub fn mark_first_run_complete(global_path: &PathBuf) -> StorageResult<()> {
        let marker_path = global_path.join(FIRST_RUN_MARKER);

        // Ensure parent directory exists
        if let Some(parent) = marker_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| {
                    crate::error::StorageError::directory_creation_failed(
                        parent.to_path_buf(),
                        e,
                    )
                })?;
            }
        }

        // Create the marker file
        fs::write(&marker_path, "").map_err(|e| {
            crate::error::StorageError::io_error(
                marker_path,
                crate::error::IoOperation::Write,
                e,
            )
        })?;

        Ok(())
    }

    /// Get the suggested global storage path
    ///
    /// Returns the path that would be used for global storage
    pub fn get_suggested_path() -> StorageResult<PathBuf> {
        PathResolver::resolve_global_path()
    }

    /// Detect first-time initialization
    ///
    /// Returns true if:
    /// - The global storage directory doesn't exist, OR
    /// - The marker file doesn't exist in the global storage directory
    pub fn detect_first_run() -> StorageResult<bool> {
        let global_path = PathResolver::resolve_global_path()?;

        if !global_path.exists() {
            return Ok(true);
        }

        Self::is_first_run(&global_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_is_first_run_no_marker() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let is_first = FirstRunHandler::is_first_run(&temp_dir.path().to_path_buf())
            .expect("Failed to check first run");
        assert!(is_first, "Should be first run when marker doesn't exist");
    }

    #[test]
    fn test_is_first_run_with_marker() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let marker_path = temp_dir.path().join(FIRST_RUN_MARKER);
        fs::write(&marker_path, "").expect("Failed to create marker");

        let is_first = FirstRunHandler::is_first_run(&temp_dir.path().to_path_buf())
            .expect("Failed to check first run");
        assert!(!is_first, "Should not be first run when marker exists");
    }

    #[test]
    fn test_mark_first_run_complete() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path().to_path_buf();

        // Initially should be first run
        let is_first_before = FirstRunHandler::is_first_run(&path)
            .expect("Failed to check first run");
        assert!(is_first_before);

        // Mark as complete
        FirstRunHandler::mark_first_run_complete(&path)
            .expect("Failed to mark first run complete");

        // Should no longer be first run
        let is_first_after = FirstRunHandler::is_first_run(&path)
            .expect("Failed to check first run");
        assert!(!is_first_after);
    }

    #[test]
    fn test_get_suggested_path() {
        let path = FirstRunHandler::get_suggested_path()
            .expect("Failed to get suggested path");
        assert!(path.to_string_lossy().contains(".ricecoder"));
    }

    #[test]
    fn test_detect_first_run_nonexistent_dir() {
        // This test uses the actual path resolution, so we just verify it returns a result
        let result = FirstRunHandler::detect_first_run();
        assert!(result.is_ok(), "Should successfully detect first run status");
    }
}
