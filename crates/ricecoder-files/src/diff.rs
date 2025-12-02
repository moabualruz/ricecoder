//! Diff generation and application for file changes

use crate::error::FileError;
use crate::models::{DiffHunk, DiffLine, DiffStats, FileDiff};
use similar::{ChangeTag, TextDiff};
use std::path::PathBuf;

/// Generates and applies diffs between file versions
#[derive(Debug, Clone)]
pub struct DiffEngine;

impl DiffEngine {
    /// Creates a new DiffEngine instance
    pub fn new() -> Self {
        Self
    }

    /// Generates a unified diff between two file versions
    ///
    /// # Arguments
    /// * `old` - The original file content
    /// * `new` - The new file content
    /// * `path` - The path of the file being diffed
    ///
    /// # Returns
    /// A `FileDiff` containing hunks and statistics
    pub fn generate_unified_diff(
        &self,
        old: &str,
        new: &str,
        path: PathBuf,
    ) -> Result<FileDiff, FileError> {
        let hunks = self.extract_hunks(old, new)?;
        let stats = self.compute_stats_from_hunks(&hunks);

        Ok(FileDiff {
            path,
            hunks,
            stats,
        })
    }

    /// Generates a side-by-side diff between two file versions
    ///
    /// # Arguments
    /// * `old` - The original file content
    /// * `new` - The new file content
    /// * `path` - The path of the file being diffed
    ///
    /// # Returns
    /// A `FileDiff` containing hunks and statistics
    pub fn generate_side_by_side_diff(
        &self,
        old: &str,
        new: &str,
        path: PathBuf,
    ) -> Result<FileDiff, FileError> {
        // For side-by-side, we use the same hunk extraction as unified
        // The difference is in presentation, which is handled by the consumer
        self.generate_unified_diff(old, new, path)
    }

    /// Applies a single hunk to source content
    ///
    /// # Arguments
    /// * `source` - The source content to apply the hunk to
    /// * `hunk` - The hunk to apply
    ///
    /// # Returns
    /// The content with the hunk applied
    pub fn apply_hunk(&self, source: &str, hunk: &DiffHunk) -> Result<String, FileError> {
        let lines: Vec<&str> = source.lines().collect();
        let mut result = Vec::new();

        // Copy lines before the hunk
        if hunk.old_start > 0 {
            result.extend_from_slice(&lines[0..hunk.old_start - 1]);
        }

        // Apply the hunk lines
        for line in &hunk.lines {
            match line {
                DiffLine::Context(content) => {
                    result.push(content.as_str());
                }
                DiffLine::Added(content) => {
                    result.push(content.as_str());
                }
                DiffLine::Removed(_) => {
                    // Skip removed lines
                }
            }
        }

        // Copy lines after the hunk
        let end_line = hunk.old_start + hunk.old_count - 1;
        if end_line < lines.len() {
            result.extend_from_slice(&lines[end_line..]);
        }

        Ok(result.join("\n"))
    }

    /// Extracts hunks from a text diff
    fn extract_hunks(&self, old: &str, new: &str) -> Result<Vec<DiffHunk>, FileError> {
        let text_diff = TextDiff::from_lines(old, new);

        let mut hunks = Vec::new();
        let mut current_hunk: Option<DiffHunk> = None;
        let mut old_line_num = 1;
        let mut new_line_num = 1;

        for change in text_diff.iter_all_changes() {
            let line_content = change.value();

            match change.tag() {
                ChangeTag::Delete => {
                    if current_hunk.is_none() {
                        current_hunk = Some(DiffHunk {
                            old_start: old_line_num,
                            old_count: 0,
                            new_start: new_line_num,
                            new_count: 0,
                            lines: Vec::new(),
                        });
                    }

                    if let Some(ref mut hunk) = current_hunk {
                        hunk.old_count += 1;
                        hunk.lines
                            .push(DiffLine::Removed(line_content.trim_end().to_string()));
                    }

                    old_line_num += 1;
                }
                ChangeTag::Insert => {
                    if current_hunk.is_none() {
                        current_hunk = Some(DiffHunk {
                            old_start: old_line_num,
                            old_count: 0,
                            new_start: new_line_num,
                            new_count: 0,
                            lines: Vec::new(),
                        });
                    }

                    if let Some(ref mut hunk) = current_hunk {
                        hunk.new_count += 1;
                        hunk.lines
                            .push(DiffLine::Added(line_content.trim_end().to_string()));
                    }

                    new_line_num += 1;
                }
                ChangeTag::Equal => {
                    if let Some(ref mut hunk) = current_hunk {
                        // If we have a hunk and encounter context, add it to the hunk
                        hunk.old_count += 1;
                        hunk.new_count += 1;
                        hunk.lines
                            .push(DiffLine::Context(line_content.trim_end().to_string()));
                    }

                    old_line_num += 1;
                    new_line_num += 1;
                }
            }
        }

        if let Some(hunk) = current_hunk {
            if !hunk.lines.is_empty() {
                hunks.push(hunk);
            }
        }

        Ok(hunks)
    }

    /// Computes statistics for a diff
    ///
    /// # Arguments
    /// * `diff` - The diff to compute statistics for
    ///
    /// # Returns
    /// Statistics about the diff (additions, deletions, files changed)
    pub fn compute_stats(&self, diff: &FileDiff) -> DiffStats {
        let mut additions = 0;
        let mut deletions = 0;

        for hunk in &diff.hunks {
            for line in &hunk.lines {
                match line {
                    DiffLine::Added(_) => additions += 1,
                    DiffLine::Removed(_) => deletions += 1,
                    DiffLine::Context(_) => {}
                }
            }
        }

        DiffStats {
            additions,
            deletions,
            files_changed: if diff.hunks.is_empty() { 0 } else { 1 },
        }
    }

    /// Internal helper to compute stats from hunks
    fn compute_stats_from_hunks(&self, hunks: &[DiffHunk]) -> DiffStats {
        let mut additions = 0;
        let mut deletions = 0;

        for hunk in hunks {
            for line in &hunk.lines {
                match line {
                    DiffLine::Added(_) => additions += 1,
                    DiffLine::Removed(_) => deletions += 1,
                    DiffLine::Context(_) => {}
                }
            }
        }

        DiffStats {
            additions,
            deletions,
            files_changed: if hunks.is_empty() { 0 } else { 1 },
        }
    }
}

impl Default for DiffEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_unified_diff_simple() {
        let engine = DiffEngine::new();
        let old = "line 1\nline 2\nline 3\n";
        let new = "line 1\nline 2 modified\nline 3\n";

        let diff = engine
            .generate_unified_diff(old, new, PathBuf::from("test.txt"))
            .unwrap();

        assert!(!diff.hunks.is_empty());
        assert_eq!(diff.stats.additions, 1);
        assert_eq!(diff.stats.deletions, 1);
    }

    #[test]
    fn test_generate_unified_diff_no_changes() {
        let engine = DiffEngine::new();
        let content = "line 1\nline 2\nline 3\n";

        let diff = engine
            .generate_unified_diff(content, content, PathBuf::from("test.txt"))
            .unwrap();

        assert_eq!(diff.stats.additions, 0);
        assert_eq!(diff.stats.deletions, 0);
    }

    #[test]
    fn test_apply_hunk_simple() {
        let engine = DiffEngine::new();
        let hunk = DiffHunk {
            old_start: 2,
            old_count: 1,
            new_start: 2,
            new_count: 1,
            lines: vec![DiffLine::Added("new line".to_string())],
        };

        let source = "line 1\nline 2\nline 3\n";
        let result = engine.apply_hunk(source, &hunk).unwrap();

        assert!(result.contains("new line"));
    }

    #[test]
    fn test_compute_stats() {
        let engine = DiffEngine::new();
        let diff = FileDiff {
            path: PathBuf::from("test.txt"),
            hunks: vec![DiffHunk {
                old_start: 1,
                old_count: 2,
                new_start: 1,
                new_count: 3,
                lines: vec![
                    DiffLine::Removed("old".to_string()),
                    DiffLine::Added("new1".to_string()),
                    DiffLine::Added("new2".to_string()),
                ],
            }],
            stats: DiffStats {
                additions: 0,
                deletions: 0,
                files_changed: 0,
            },
        };

        let stats = engine.compute_stats(&diff);
        assert_eq!(stats.additions, 2);
        assert_eq!(stats.deletions, 1);
        assert_eq!(stats.files_changed, 1);
    }
}
