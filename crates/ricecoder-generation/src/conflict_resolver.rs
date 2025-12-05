//! Conflict resolution strategies for generated files
//!
//! Implements multiple strategies for handling file conflicts:
//! - Skip: Don't write conflicting files
//! - Overwrite: Write and backup original
//! - Merge: Attempt intelligent merge
//! - Prompt: Ask user for each conflict
//!
//! Implements requirements:
//! - Requirement 4.2: Skip strategy - don't write conflicting files
//! - Requirement 4.3: Overwrite strategy - write and backup original
//! - Requirement 4.4: Merge strategy - attempt intelligent merge
//! - Requirement 4.5: Prompt strategy - ask user for each conflict

use crate::conflict_detector::FileConflictInfo;
use crate::error::GenerationError;
use std::fs;
use std::path::Path;

/// Strategy for resolving file conflicts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictStrategy {
    /// Skip the conflicting file (don't write)
    Skip,
    /// Overwrite the existing file (with backup)
    Overwrite,
    /// Attempt to merge changes
    Merge,
    /// Prompt user for each conflict
    Prompt,
}

/// Result of applying a conflict resolution strategy
#[derive(Debug, Clone)]
pub struct ConflictResolutionResult {
    /// Whether the file was written
    pub written: bool,
    /// Path to backup file if created
    pub backup_path: Option<String>,
    /// Resolution action taken
    pub action: String,
}

/// Resolves file conflicts using specified strategy
///
/// Implements requirements:
/// - Requirement 4.2: Skip strategy
/// - Requirement 4.3: Overwrite strategy with backup
/// - Requirement 4.4: Merge strategy
/// - Requirement 4.5: Prompt strategy
pub struct ConflictResolver;

impl ConflictResolver {
    /// Create a new conflict resolver
    pub fn new() -> Self {
        Self
    }

    /// Resolve a single conflict using the specified strategy
    ///
    /// # Arguments
    /// * `conflict` - Conflict information
    /// * `strategy` - Resolution strategy to apply
    /// * `new_content` - New content to write
    ///
    /// # Returns
    /// Resolution result
    ///
    /// # Requirements
    /// - Requirement 4.2: Skip strategy
    /// - Requirement 4.3: Overwrite strategy with backup
    /// - Requirement 4.4: Merge strategy
    pub fn resolve(
        &self,
        conflict: &FileConflictInfo,
        strategy: ConflictStrategy,
        new_content: &str,
    ) -> Result<ConflictResolutionResult, GenerationError> {
        match strategy {
            ConflictStrategy::Skip => self.resolve_skip(conflict),
            ConflictStrategy::Overwrite => self.resolve_overwrite(conflict, new_content),
            ConflictStrategy::Merge => self.resolve_merge(conflict, new_content),
            ConflictStrategy::Prompt => {
                // Prompt strategy is handled by the caller
                Err(GenerationError::ValidationError {
                    file: conflict.path.to_string_lossy().to_string(),
                    line: 0,
                    message: "Prompt strategy requires user interaction".to_string(),
                })
            }
        }
    }

    /// Resolve conflict using skip strategy
    ///
    /// Does not write the file and continues with remaining files.
    ///
    /// # Requirements
    /// - Requirement 4.2: Skip strategy - don't write conflicting files
    fn resolve_skip(&self, conflict: &FileConflictInfo) -> Result<ConflictResolutionResult, GenerationError> {
        Ok(ConflictResolutionResult {
            written: false,
            backup_path: None,
            action: format!("Skipped: {}", conflict.path.display()),
        })
    }

    /// Resolve conflict using overwrite strategy
    ///
    /// Writes the generated file and creates a backup of the original.
    ///
    /// # Arguments
    /// * `conflict` - Conflict information
    /// * `new_content` - New content to write
    ///
    /// # Returns
    /// Resolution result with backup path
    ///
    /// # Requirements
    /// - Requirement 4.3: Overwrite strategy - write and backup original
    fn resolve_overwrite(
        &self,
        conflict: &FileConflictInfo,
        new_content: &str,
    ) -> Result<ConflictResolutionResult, GenerationError> {
        // Create backup of original file
        let backup_path = self.create_backup(&conflict.path)?;

        // Write new content
        fs::write(&conflict.path, new_content)
            .map_err(|e| GenerationError::ValidationError {
                file: conflict.path.to_string_lossy().to_string(),
                line: 0,
                message: format!("Failed to write file: {}", e),
            })?;

        Ok(ConflictResolutionResult {
            written: true,
            backup_path: backup_path.clone(),
            action: format!(
                "Overwritten: {} (backup: {})",
                conflict.path.display(),
                backup_path.unwrap_or_default()
            ),
        })
    }

    /// Resolve conflict using merge strategy
    ///
    /// Attempts to merge changes intelligently. For now, this is a simple merge
    /// that preserves both old and new content with markers.
    ///
    /// # Arguments
    /// * `conflict` - Conflict information
    /// * `new_content` - New content to merge
    ///
    /// # Returns
    /// Resolution result
    ///
    /// # Requirements
    /// - Requirement 4.4: Merge strategy - attempt intelligent merge
    fn resolve_merge(
        &self,
        conflict: &FileConflictInfo,
        new_content: &str,
    ) -> Result<ConflictResolutionResult, GenerationError> {
        // Create backup of original file
        let backup_path = self.create_backup(&conflict.path)?;

        // Perform simple merge: mark conflict regions
        let merged_content = self.merge_contents(&conflict.old_content, new_content)?;

        // Write merged content
        fs::write(&conflict.path, &merged_content)
            .map_err(|e| GenerationError::ValidationError {
                file: conflict.path.to_string_lossy().to_string(),
                line: 0,
                message: format!("Failed to write merged file: {}", e),
            })?;

        Ok(ConflictResolutionResult {
            written: true,
            backup_path: backup_path.clone(),
            action: format!(
                "Merged: {} (backup: {}, conflicts marked)",
                conflict.path.display(),
                backup_path.unwrap_or_default()
            ),
        })
    }

    /// Create a backup of a file
    ///
    /// Creates a backup with .bak extension.
    ///
    /// # Arguments
    /// * `file_path` - Path to file to backup
    ///
    /// # Returns
    /// Path to backup file
    fn create_backup(&self, file_path: &Path) -> Result<Option<String>, GenerationError> {
        let backup_path = format!("{}.bak", file_path.display());
        let backup_path_obj = Path::new(&backup_path);

        // Read original content
        let content = fs::read_to_string(file_path)
            .map_err(|e| GenerationError::ValidationError {
                file: file_path.to_string_lossy().to_string(),
                line: 0,
                message: format!("Failed to read file for backup: {}", e),
            })?;

        // Write backup
        fs::write(backup_path_obj, content)
            .map_err(|e| GenerationError::ValidationError {
                file: file_path.to_string_lossy().to_string(),
                line: 0,
                message: format!("Failed to create backup: {}", e),
            })?;

        Ok(Some(backup_path))
    }

    /// Merge two file contents
    ///
    /// Simple merge that marks conflict regions with markers.
    ///
    /// # Arguments
    /// * `old_content` - Original content
    /// * `new_content` - New content
    ///
    /// # Returns
    /// Merged content
    fn merge_contents(&self, old_content: &str, new_content: &str) -> Result<String, GenerationError> {
        // Simple merge: if contents are identical, return as-is
        if old_content == new_content {
            return Ok(new_content.to_string());
        }

        // Otherwise, create a marked merge with both versions
        let merged = format!(
            "<<<<<<< ORIGINAL\n{}\n=======\n{}\n>>>>>>> GENERATED\n",
            old_content, new_content
        );

        Ok(merged)
    }

    /// Check if a conflict can be auto-merged
    ///
    /// Returns true if the conflict can be automatically resolved without user intervention.
    ///
    /// # Arguments
    /// * `conflict` - Conflict to check
    ///
    /// # Returns
    /// True if auto-mergeable
    pub fn is_auto_mergeable(&self, conflict: &FileConflictInfo) -> bool {
        // A conflict is auto-mergeable if:
        // 1. The files are identical (no actual conflict)
        // 2. The new content is a superset of the old content (only additions)
        
        if conflict.old_content == conflict.new_content {
            return true;
        }

        // Check if new content contains all lines from old content
        let old_lines: Vec<&str> = conflict.old_content.lines().collect();
        let new_content_lines = conflict.new_content.lines().collect::<Vec<_>>();

        old_lines.iter().all(|line| new_content_lines.contains(line))
    }

    /// Get a human-readable description of a strategy
    ///
    /// # Arguments
    /// * `strategy` - Strategy to describe
    ///
    /// # Returns
    /// Description string
    pub fn describe_strategy(&self, strategy: ConflictStrategy) -> &'static str {
        match strategy {
            ConflictStrategy::Skip => "Skip conflicting files (don't write)",
            ConflictStrategy::Overwrite => "Overwrite existing files (with backup)",
            ConflictStrategy::Merge => "Merge changes (mark conflicts)",
            ConflictStrategy::Prompt => "Prompt for each conflict",
        }
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
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_test_conflict(old_content: &str, new_content: &str) -> FileConflictInfo {
        FileConflictInfo {
            path: PathBuf::from("test.rs"),
            old_content: old_content.to_string(),
            new_content: new_content.to_string(),
            diff: crate::conflict_detector::FileDiff {
                added_lines: vec![],
                removed_lines: vec![],
                modified_lines: vec![],
                total_changes: 0,
            },
        }
    }

    #[test]
    fn test_create_resolver() {
        let _resolver = ConflictResolver::new();
    }

    #[test]
    fn test_resolve_skip() {
        let resolver = ConflictResolver::new();
        let conflict = create_test_conflict("old", "new");

        let result = resolver.resolve(&conflict, ConflictStrategy::Skip, "new").unwrap();
        assert!(!result.written);
        assert!(result.backup_path.is_none());
    }

    #[test]
    fn test_resolve_overwrite() {
        let temp_dir = TempDir::new().unwrap();
        let resolver = ConflictResolver::new();

        let file_path = temp_dir.path().join("test.rs");
        fs::write(&file_path, "old content").unwrap();

        let mut conflict = create_test_conflict("old content", "new content");
        conflict.path = file_path.clone();

        let result = resolver
            .resolve(&conflict, ConflictStrategy::Overwrite, "new content")
            .unwrap();

        assert!(result.written);
        assert!(result.backup_path.is_some());

        // Verify file was written
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "new content");

        // Verify backup was created
        let backup_path = format!("{}.bak", file_path.display());
        assert!(Path::new(&backup_path).exists());
    }

    #[test]
    fn test_resolve_merge() {
        let temp_dir = TempDir::new().unwrap();
        let resolver = ConflictResolver::new();

        let file_path = temp_dir.path().join("test.rs");
        fs::write(&file_path, "old content").unwrap();

        let mut conflict = create_test_conflict("old content", "new content");
        conflict.path = file_path.clone();

        let result = resolver
            .resolve(&conflict, ConflictStrategy::Merge, "new content")
            .unwrap();

        assert!(result.written);
        assert!(result.backup_path.is_some());

        // Verify merged content contains conflict markers
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("<<<<<<< ORIGINAL"));
        assert!(content.contains("======="));
        assert!(content.contains(">>>>>>> GENERATED"));
    }

    #[test]
    fn test_is_auto_mergeable_identical() {
        let resolver = ConflictResolver::new();
        let conflict = create_test_conflict("content", "content");

        assert!(resolver.is_auto_mergeable(&conflict));
    }

    #[test]
    fn test_is_auto_mergeable_superset() {
        let resolver = ConflictResolver::new();
        let conflict = create_test_conflict("line 1\nline 2", "line 1\nline 2\nline 3");

        assert!(resolver.is_auto_mergeable(&conflict));
    }

    #[test]
    fn test_is_auto_mergeable_not_mergeable() {
        let resolver = ConflictResolver::new();
        let conflict = create_test_conflict("line 1\nline 2", "line 1\nmodified line 2");

        assert!(!resolver.is_auto_mergeable(&conflict));
    }

    #[test]
    fn test_describe_strategy_skip() {
        let resolver = ConflictResolver::new();
        let desc = resolver.describe_strategy(ConflictStrategy::Skip);
        assert!(desc.contains("Skip"));
    }

    #[test]
    fn test_describe_strategy_overwrite() {
        let resolver = ConflictResolver::new();
        let desc = resolver.describe_strategy(ConflictStrategy::Overwrite);
        assert!(desc.contains("Overwrite"));
    }

    #[test]
    fn test_describe_strategy_merge() {
        let resolver = ConflictResolver::new();
        let desc = resolver.describe_strategy(ConflictStrategy::Merge);
        assert!(desc.contains("Merge"));
    }

    #[test]
    fn test_describe_strategy_prompt() {
        let resolver = ConflictResolver::new();
        let desc = resolver.describe_strategy(ConflictStrategy::Prompt);
        assert!(desc.contains("Prompt"));
    }

    #[test]
    fn test_merge_contents_identical() {
        let resolver = ConflictResolver::new();
        let merged = resolver.merge_contents("content", "content").unwrap();
        assert_eq!(merged, "content");
    }

    #[test]
    fn test_merge_contents_different() {
        let resolver = ConflictResolver::new();
        let merged = resolver.merge_contents("old", "new").unwrap();
        assert!(merged.contains("<<<<<<< ORIGINAL"));
        assert!(merged.contains("old"));
        assert!(merged.contains("======="));
        assert!(merged.contains("new"));
        assert!(merged.contains(">>>>>>> GENERATED"));
    }

    #[test]
    fn test_create_backup() {
        let temp_dir = TempDir::new().unwrap();
        let resolver = ConflictResolver::new();

        let file_path = temp_dir.path().join("test.rs");
        fs::write(&file_path, "original content").unwrap();

        let backup_path = resolver.create_backup(&file_path).unwrap();
        assert!(backup_path.is_some());

        let backup_path_str = backup_path.unwrap();
        let backup_file = Path::new(&backup_path_str);
        assert!(backup_file.exists());

        let backup_content = fs::read_to_string(backup_file).unwrap();
        assert_eq!(backup_content, "original content");
    }
}
