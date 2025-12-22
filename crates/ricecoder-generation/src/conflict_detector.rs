//! Conflict detection for generated files
//!
//! Detects file conflicts before writing and computes diffs between old and new content.
//! Implements requirements:
//! - Requirement 1.5: Detect conflicts when generated files would overwrite existing files
//! - Requirement 4.1: Compute diffs between old and new content

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{error::GenerationError, models::GeneratedFile};

/// Information about a file conflict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileConflictInfo {
    /// Path to the conflicting file
    pub path: PathBuf,
    /// Existing file content
    pub old_content: String,
    /// New generated content
    pub new_content: String,
    /// Diff between old and new content
    pub diff: FileDiff,
}

/// Represents a diff between two file versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDiff {
    /// Lines that were added
    pub added_lines: Vec<DiffLine>,
    /// Lines that were removed
    pub removed_lines: Vec<DiffLine>,
    /// Lines that were modified
    pub modified_lines: Vec<(DiffLine, DiffLine)>,
    /// Total number of changes
    pub total_changes: usize,
}

/// A single line in a diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    /// Line number in the file
    pub line_number: usize,
    /// Content of the line
    pub content: String,
}

/// Detects file conflicts before writing
///
/// Implements requirements:
/// - Requirement 1.5: Detect conflicts when generated files would overwrite existing files
/// - Requirement 4.1: Compute diffs between old and new content
pub struct ConflictDetector;

impl ConflictDetector {
    /// Create a new conflict detector
    pub fn new() -> Self {
        Self
    }

    /// Detect conflicts for a set of generated files
    ///
    /// Checks if any generated files would overwrite existing files and computes diffs.
    ///
    /// # Arguments
    /// * `files` - Generated files to check
    /// * `target_dir` - Target directory where files would be written
    ///
    /// # Returns
    /// List of detected conflicts
    ///
    /// # Requirements
    /// - Requirement 1.5: Detect conflicts when generated files would overwrite existing files
    /// - Requirement 4.1: Compute diffs between old and new content
    pub fn detect(
        &self,
        files: &[GeneratedFile],
        target_dir: &Path,
    ) -> Result<Vec<FileConflictInfo>, GenerationError> {
        let mut conflicts = Vec::new();

        for file in files {
            let file_path = target_dir.join(&file.path);

            // Check if file already exists
            if file_path.exists() {
                // Read existing content
                let old_content = fs::read_to_string(&file_path).map_err(|e| {
                    GenerationError::ValidationError {
                        file: file.path.clone(),
                        line: 0,
                        message: format!("Failed to read existing file: {}", e),
                    }
                })?;

                // Compute diff
                let diff = self.compute_diff(&old_content, &file.content)?;

                conflicts.push(FileConflictInfo {
                    path: file_path,
                    old_content,
                    new_content: file.content.clone(),
                    diff,
                });
            }
        }

        Ok(conflicts)
    }

    /// Detect a single file conflict
    ///
    /// # Arguments
    /// * `file_path` - Path to the file to check
    /// * `new_content` - New content to be written
    ///
    /// # Returns
    /// Conflict info if file exists, None otherwise
    pub fn detect_single(
        &self,
        file_path: &Path,
        new_content: &str,
    ) -> Result<Option<FileConflictInfo>, GenerationError> {
        if !file_path.exists() {
            return Ok(None);
        }

        let old_content =
            fs::read_to_string(file_path).map_err(|e| GenerationError::ValidationError {
                file: file_path.to_string_lossy().to_string(),
                line: 0,
                message: format!("Failed to read existing file: {}", e),
            })?;

        let diff = self.compute_diff(&old_content, new_content)?;

        Ok(Some(FileConflictInfo {
            path: file_path.to_path_buf(),
            old_content,
            new_content: new_content.to_string(),
            diff,
        }))
    }

    /// Compute a diff between two file contents
    ///
    /// Uses a simple line-based diff algorithm to identify added, removed, and modified lines.
    ///
    /// # Arguments
    /// * `old_content` - Original file content
    /// * `new_content` - New file content
    ///
    /// # Returns
    /// Diff information
    fn compute_diff(
        &self,
        old_content: &str,
        new_content: &str,
    ) -> Result<FileDiff, GenerationError> {
        let old_lines: Vec<&str> = old_content.lines().collect();
        let new_lines: Vec<&str> = new_content.lines().collect();

        let mut added_lines = Vec::new();
        let mut removed_lines = Vec::new();
        let mut modified_lines = Vec::new();

        // Simple line-based diff: compare line by line
        let max_lines = old_lines.len().max(new_lines.len());

        for i in 0..max_lines {
            let old_line = old_lines.get(i).copied();
            let new_line = new_lines.get(i).copied();

            match (old_line, new_line) {
                (Some(old), Some(new)) if old != new => {
                    // Line was modified
                    modified_lines.push((
                        DiffLine {
                            line_number: i + 1,
                            content: old.to_string(),
                        },
                        DiffLine {
                            line_number: i + 1,
                            content: new.to_string(),
                        },
                    ));
                }
                (Some(old), None) => {
                    // Line was removed
                    removed_lines.push(DiffLine {
                        line_number: i + 1,
                        content: old.to_string(),
                    });
                }
                (None, Some(new)) => {
                    // Line was added
                    added_lines.push(DiffLine {
                        line_number: i + 1,
                        content: new.to_string(),
                    });
                }
                (Some(_), Some(_)) => {
                    // Lines are identical, no change
                }
                (None, None) => {
                    // Both are None, shouldn't happen
                }
            }
        }

        let total_changes = added_lines.len() + removed_lines.len() + modified_lines.len();

        Ok(FileDiff {
            added_lines,
            removed_lines,
            modified_lines,
            total_changes,
        })
    }

    /// Check if two files have the same content
    ///
    /// # Arguments
    /// * `old_content` - Original content
    /// * `new_content` - New content
    ///
    /// # Returns
    /// True if contents are identical
    pub fn is_identical(&self, old_content: &str, new_content: &str) -> bool {
        old_content == new_content
    }

    /// Get a human-readable summary of a diff
    ///
    /// # Arguments
    /// * `diff` - Diff to summarize
    ///
    /// # Returns
    /// Summary string
    pub fn summarize_diff(&self, diff: &FileDiff) -> String {
        format!(
            "{} added, {} removed, {} modified",
            diff.added_lines.len(),
            diff.removed_lines.len(),
            diff.modified_lines.len()
        )
    }
}

impl Default for ConflictDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_create_conflict_detector() {
        let _detector = ConflictDetector::new();
    }

    #[test]
    fn test_detect_no_conflicts() {
        let temp_dir = TempDir::new().unwrap();
        let detector = ConflictDetector::new();

        let files = vec![GeneratedFile {
            path: "src/main.rs".to_string(),
            content: "fn main() {}".to_string(),
            language: "rust".to_string(),
        }];

        let conflicts = detector.detect(&files, temp_dir.path()).unwrap();
        assert_eq!(conflicts.len(), 0);
    }

    #[test]
    fn test_detect_single_conflict() {
        let temp_dir = TempDir::new().unwrap();
        let detector = ConflictDetector::new();

        // Create existing file
        let file_path = temp_dir.path().join("src").join("main.rs");
        fs::create_dir_all(file_path.parent().unwrap()).unwrap();
        fs::write(&file_path, "// old content").unwrap();

        let files = vec![GeneratedFile {
            path: "src/main.rs".to_string(),
            content: "// new content".to_string(),
            language: "rust".to_string(),
        }];

        let conflicts = detector.detect(&files, temp_dir.path()).unwrap();
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].old_content, "// old content");
        assert_eq!(conflicts[0].new_content, "// new content");
    }

    #[test]
    fn test_detect_multiple_conflicts() {
        let temp_dir = TempDir::new().unwrap();
        let detector = ConflictDetector::new();

        // Create existing files
        let file1_path = temp_dir.path().join("src").join("main.rs");
        fs::create_dir_all(file1_path.parent().unwrap()).unwrap();
        fs::write(&file1_path, "// old main").unwrap();

        let file2_path = temp_dir.path().join("src").join("lib.rs");
        fs::write(&file2_path, "// old lib").unwrap();

        let files = vec![
            GeneratedFile {
                path: "src/main.rs".to_string(),
                content: "// new main".to_string(),
                language: "rust".to_string(),
            },
            GeneratedFile {
                path: "src/lib.rs".to_string(),
                content: "// new lib".to_string(),
                language: "rust".to_string(),
            },
        ];

        let conflicts = detector.detect(&files, temp_dir.path()).unwrap();
        assert_eq!(conflicts.len(), 2);
    }

    #[test]
    fn test_compute_diff_identical() {
        let detector = ConflictDetector::new();
        let old = "line 1\nline 2\nline 3";
        let new = "line 1\nline 2\nline 3";

        let diff = detector.compute_diff(old, new).unwrap();
        assert_eq!(diff.added_lines.len(), 0);
        assert_eq!(diff.removed_lines.len(), 0);
        assert_eq!(diff.modified_lines.len(), 0);
    }

    #[test]
    fn test_compute_diff_added_lines() {
        let detector = ConflictDetector::new();
        let old = "line 1\nline 2";
        let new = "line 1\nline 2\nline 3\nline 4";

        let diff = detector.compute_diff(old, new).unwrap();
        assert_eq!(diff.added_lines.len(), 2);
        assert_eq!(diff.removed_lines.len(), 0);
        assert_eq!(diff.modified_lines.len(), 0);
    }

    #[test]
    fn test_compute_diff_removed_lines() {
        let detector = ConflictDetector::new();
        let old = "line 1\nline 2\nline 3\nline 4";
        let new = "line 1\nline 2";

        let diff = detector.compute_diff(old, new).unwrap();
        assert_eq!(diff.added_lines.len(), 0);
        assert_eq!(diff.removed_lines.len(), 2);
        assert_eq!(diff.modified_lines.len(), 0);
    }

    #[test]
    fn test_compute_diff_modified_lines() {
        let detector = ConflictDetector::new();
        let old = "line 1\nline 2\nline 3";
        let new = "line 1\nmodified line 2\nline 3";

        let diff = detector.compute_diff(old, new).unwrap();
        assert_eq!(diff.added_lines.len(), 0);
        assert_eq!(diff.removed_lines.len(), 0);
        assert_eq!(diff.modified_lines.len(), 1);
    }

    #[test]
    fn test_is_identical_true() {
        let detector = ConflictDetector::new();
        let content = "line 1\nline 2\nline 3";
        assert!(detector.is_identical(content, content));
    }

    #[test]
    fn test_is_identical_false() {
        let detector = ConflictDetector::new();
        let old = "line 1\nline 2";
        let new = "line 1\nline 2\nline 3";
        assert!(!detector.is_identical(old, new));
    }

    #[test]
    fn test_summarize_diff() {
        let detector = ConflictDetector::new();
        let diff = FileDiff {
            added_lines: vec![DiffLine {
                line_number: 1,
                content: "added".to_string(),
            }],
            removed_lines: vec![DiffLine {
                line_number: 2,
                content: "removed".to_string(),
            }],
            modified_lines: vec![(
                DiffLine {
                    line_number: 3,
                    content: "old".to_string(),
                },
                DiffLine {
                    line_number: 3,
                    content: "new".to_string(),
                },
            )],
            total_changes: 3,
        };

        let summary = detector.summarize_diff(&diff);
        assert!(summary.contains("1 added"));
        assert!(summary.contains("1 removed"));
        assert!(summary.contains("1 modified"));
    }

    #[test]
    fn test_detect_single_no_conflict() {
        let temp_dir = TempDir::new().unwrap();
        let detector = ConflictDetector::new();

        let file_path = temp_dir.path().join("nonexistent.rs");
        let result = detector.detect_single(&file_path, "content").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_detect_single_with_conflict() {
        let temp_dir = TempDir::new().unwrap();
        let detector = ConflictDetector::new();

        let file_path = temp_dir.path().join("existing.rs");
        fs::write(&file_path, "old content").unwrap();

        let result = detector.detect_single(&file_path, "new content").unwrap();
        assert!(result.is_some());

        let conflict = result.unwrap();
        assert_eq!(conflict.old_content, "old content");
        assert_eq!(conflict.new_content, "new content");
    }
}
