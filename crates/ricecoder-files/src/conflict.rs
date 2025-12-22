//! Conflict detection and resolution for file operations

use std::path::Path;

use tokio::fs;

use crate::{
    error::FileError,
    models::{ConflictInfo, ConflictResolution},
};

/// Detects and resolves file conflicts when target path already exists
#[derive(Debug, Clone)]
pub struct ConflictResolver;

impl ConflictResolver {
    /// Creates a new ConflictResolver instance
    pub fn new() -> Self {
        ConflictResolver
    }

    /// Detects conflicts by checking if target file exists
    ///
    /// # Arguments
    ///
    /// * `path` - Path to check for conflicts
    ///
    /// # Returns
    ///
    /// Ok(Some(ConflictInfo)) if conflict exists, Ok(None) if no conflict,
    /// or an error if the check fails
    pub async fn detect_conflict(
        &self,
        path: &Path,
        new_content: &str,
    ) -> Result<Option<ConflictInfo>, FileError> {
        if !path.exists() {
            return Ok(None);
        }

        let existing_content = fs::read_to_string(path)
            .await
            .map_err(|_e| FileError::ConflictDetected(path.to_path_buf()))?;

        Ok(Some(ConflictInfo {
            path: path.to_path_buf(),
            existing_content,
            new_content: new_content.to_string(),
        }))
    }

    /// Resolves a conflict using the specified strategy
    ///
    /// # Arguments
    ///
    /// * `strategy` - The resolution strategy to use
    /// * `conflict_info` - Information about the conflict
    ///
    /// # Returns
    ///
    /// Ok(()) if resolution succeeds, or an error if it fails
    pub fn resolve(
        &self,
        strategy: ConflictResolution,
        conflict_info: &ConflictInfo,
    ) -> Result<(), FileError> {
        match strategy {
            ConflictResolution::Skip => {
                Err(FileError::ConflictDetected(conflict_info.path.clone()))
            }
            ConflictResolution::Overwrite => {
                // Overwrite is allowed; no error
                Ok(())
            }
            ConflictResolution::Merge => {
                // Merge strategy: combine both versions
                // For now, we'll implement a simple merge that keeps both
                Ok(())
            }
        }
    }

    /// Performs a simple merge of two content versions
    ///
    /// # Arguments
    ///
    /// * `existing` - The existing content
    /// * `new` - The new content
    ///
    /// # Returns
    ///
    /// Merged content
    pub fn merge_content(existing: &str, new: &str) -> String {
        // Simple merge: if content is identical, return it
        if existing == new {
            return new.to_string();
        }

        // Otherwise, combine with markers
        format!(
            "<<<<<<< EXISTING\n{}\n=======\n{}\n>>>>>>> NEW",
            existing, new
        )
    }
}

impl Default for ConflictResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_detect_conflict_no_file() {
        let resolver = ConflictResolver::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("nonexistent.txt");

        let result = resolver.detect_conflict(&path, "new content").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_detect_conflict_file_exists() {
        let resolver = ConflictResolver::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("existing.txt");

        fs::write(&path, "existing content").await.unwrap();

        let result = resolver.detect_conflict(&path, "new content").await;
        assert!(result.is_ok());

        let conflict = result.unwrap();
        assert!(conflict.is_some());

        let conflict_info = conflict.unwrap();
        assert_eq!(conflict_info.path, path);
        assert_eq!(conflict_info.existing_content, "existing content");
        assert_eq!(conflict_info.new_content, "new content");
    }

    #[test]
    fn test_resolve_skip_strategy() {
        let resolver = ConflictResolver::new();
        let conflict_info = ConflictInfo {
            path: "test.txt".into(),
            existing_content: "existing".to_string(),
            new_content: "new".to_string(),
        };

        let result = resolver.resolve(ConflictResolution::Skip, &conflict_info);
        assert!(result.is_err());
        match result {
            Err(FileError::ConflictDetected(_)) => (),
            _ => panic!("Expected ConflictDetected error"),
        }
    }

    #[test]
    fn test_resolve_overwrite_strategy() {
        let resolver = ConflictResolver::new();
        let conflict_info = ConflictInfo {
            path: "test.txt".into(),
            existing_content: "existing".to_string(),
            new_content: "new".to_string(),
        };

        let result = resolver.resolve(ConflictResolution::Overwrite, &conflict_info);
        assert!(result.is_ok());
    }

    #[test]
    fn test_resolve_merge_strategy() {
        let resolver = ConflictResolver::new();
        let conflict_info = ConflictInfo {
            path: "test.txt".into(),
            existing_content: "existing".to_string(),
            new_content: "new".to_string(),
        };

        let result = resolver.resolve(ConflictResolution::Merge, &conflict_info);
        assert!(result.is_ok());
    }

    #[test]
    fn test_merge_content_identical() {
        let content = "same content";
        let merged = ConflictResolver::merge_content(content, content);
        assert_eq!(merged, content);
    }

    #[test]
    fn test_merge_content_different() {
        let existing = "existing content";
        let new = "new content";
        let merged = ConflictResolver::merge_content(existing, new);

        assert!(merged.contains("<<<<<<< EXISTING"));
        assert!(merged.contains("======="));
        assert!(merged.contains(">>>>>>> NEW"));
        assert!(merged.contains(existing));
        assert!(merged.contains(new));
    }
}
