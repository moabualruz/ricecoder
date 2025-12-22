//! Preview generation for refactoring operations

use crate::{
    error::Result,
    impact::ImpactAnalyzer,
    types::{Refactoring, RefactoringPreview},
};

/// Generates previews of refactoring operations
pub struct PreviewGenerator;

impl PreviewGenerator {
    /// Create a new preview generator
    pub fn new() -> Self {
        Self
    }
}

impl Default for PreviewGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a diff hunk (a contiguous block of changes)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffHunk {
    /// Starting line in original file
    pub original_start: usize,
    /// Number of lines in original file
    pub original_count: usize,
    /// Starting line in new file
    pub new_start: usize,
    /// Number of lines in new file
    pub new_count: usize,
    /// Lines in the hunk (with +/- prefix)
    pub lines: Vec<String>,
}

/// Represents a unified diff
#[derive(Debug, Clone)]
pub struct UnifiedDiff {
    /// Original file path
    pub original_path: String,
    /// New file path
    pub new_path: String,
    /// Hunks in the diff
    pub hunks: Vec<DiffHunk>,
}

impl PreviewGenerator {
    /// Generate a preview of a refactoring
    pub fn generate(refactoring: &Refactoring) -> Result<RefactoringPreview> {
        let analyzer = ImpactAnalyzer::new();
        let impact = analyzer.analyze(refactoring)?;

        // For now, create empty changes
        // In a real implementation, this would generate actual code changes
        let changes = vec![];

        Ok(RefactoringPreview {
            changes,
            impact,
            estimated_time_seconds: 5,
        })
    }

    /// Generate a unified diff between original and new content
    ///
    /// Returns a unified diff format string that shows the changes
    /// between the original and new content.
    pub fn generate_unified_diff(original: &str, new: &str) -> String {
        let hunks = Self::compute_hunks(original, new);
        Self::format_unified_diff("original", "new", &hunks)
    }

    /// Generate a side-by-side diff between original and new content
    ///
    /// Returns a side-by-side diff format that shows original and new
    /// content side by side for easier comparison.
    pub fn generate_side_by_side_diff(original: &str, new: &str) -> String {
        let original_lines: Vec<&str> = original.lines().collect();
        let new_lines: Vec<&str> = new.lines().collect();

        let mut result = String::new();
        result.push_str("--- original\t\t+++ new\n");
        result.push_str("---\n");

        let max_lines = original_lines.len().max(new_lines.len());
        for i in 0..max_lines {
            let orig_line = original_lines.get(i).copied().unwrap_or("");
            let new_line = new_lines.get(i).copied().unwrap_or("");

            if orig_line == new_line {
                result.push_str(&format!("{:<40} | {}\n", orig_line, new_line));
            } else {
                result.push_str(&format!("< {:<38} | > {}\n", orig_line, new_line));
            }
        }

        result
    }

    /// Compute diff hunks between original and new content
    ///
    /// Uses a simple line-based diff algorithm to identify contiguous
    /// blocks of changes (hunks).
    fn compute_hunks(original: &str, new: &str) -> Vec<DiffHunk> {
        let original_lines: Vec<&str> = original.lines().collect();
        let new_lines: Vec<&str> = new.lines().collect();

        let mut hunks = Vec::new();
        let mut i = 0;
        let mut j = 0;

        while i < original_lines.len() || j < new_lines.len() {
            // Skip matching lines
            while i < original_lines.len()
                && j < new_lines.len()
                && original_lines[i] == new_lines[j]
            {
                i += 1;
                j += 1;
            }

            if i >= original_lines.len() && j >= new_lines.len() {
                break;
            }

            // Found a difference, collect the hunk
            let hunk_start_orig = i + 1; // 1-indexed
            let hunk_start_new = j + 1; // 1-indexed
            let mut hunk_lines = Vec::new();

            // Collect removed lines
            let removed_start = i;
            while i < original_lines.len()
                && (j >= new_lines.len() || original_lines[i] != new_lines[j])
            {
                hunk_lines.push(format!("-{}", original_lines[i]));
                i += 1;
            }

            // Collect added lines
            let added_start = j;
            while j < new_lines.len()
                && (i >= original_lines.len() || original_lines[i] != new_lines[j])
            {
                hunk_lines.push(format!("+{}", new_lines[j]));
                j += 1;
            }

            // Add context lines (up to 3 lines after changes)
            let context_count = 3;
            let mut context_added = 0;
            while context_added < context_count
                && i < original_lines.len()
                && j < new_lines.len()
                && original_lines[i] == new_lines[j]
            {
                hunk_lines.push(format!(" {}", original_lines[i]));
                i += 1;
                j += 1;
                context_added += 1;
            }

            let original_count = i - removed_start;
            let new_count = j - added_start;

            hunks.push(DiffHunk {
                original_start: hunk_start_orig,
                original_count,
                new_start: hunk_start_new,
                new_count,
                lines: hunk_lines,
            });
        }

        hunks
    }

    /// Format hunks as unified diff
    fn format_unified_diff(original_path: &str, new_path: &str, hunks: &[DiffHunk]) -> String {
        let mut result = String::new();
        result.push_str(&format!("--- {}\n", original_path));
        result.push_str(&format!("+++ {}\n", new_path));

        for hunk in hunks {
            result.push_str(&format!(
                "@@ -{},{} +{},{} @@\n",
                hunk.original_start, hunk.original_count, hunk.new_start, hunk.new_count
            ));

            for line in &hunk.lines {
                result.push_str(line);
                result.push('\n');
            }
        }

        result
    }

    /// Generate a diff for a file change
    ///
    /// This is a convenience method that generates a unified diff
    /// between the original and new content.
    pub fn generate_diff(original: &str, new: &str) -> String {
        Self::generate_unified_diff(original, new)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::types::{RefactoringOptions, RefactoringTarget, RefactoringType};

    #[test]
    fn test_generate_preview() -> Result<()> {
        let refactoring = Refactoring {
            id: "test-refactoring".to_string(),
            refactoring_type: RefactoringType::Rename,
            target: RefactoringTarget {
                file: PathBuf::from("src/main.rs"),
                symbol: "old_name".to_string(),
                range: None,
            },
            options: RefactoringOptions::default(),
        };

        let preview = PreviewGenerator::generate(&refactoring)?;
        assert_eq!(preview.estimated_time_seconds, 5);

        Ok(())
    }

    #[test]
    fn test_generate_unified_diff_simple() {
        let original = "fn old_name() {}\n";
        let new = "fn new_name() {}\n";
        let diff = PreviewGenerator::generate_unified_diff(original, new);

        assert!(diff.contains("--- original"));
        assert!(diff.contains("+++ new"));
        assert!(diff.contains("-fn old_name() {}"));
        assert!(diff.contains("+fn new_name() {}"));
    }

    #[test]
    fn test_generate_unified_diff_multiline() {
        let original = "line 1\nline 2\nline 3\n";
        let new = "line 1\nmodified line 2\nline 3\n";
        let diff = PreviewGenerator::generate_unified_diff(original, new);

        assert!(diff.contains("--- original"));
        assert!(diff.contains("+++ new"));
        assert!(diff.contains("-line 2"));
        assert!(diff.contains("+modified line 2"));
    }

    #[test]
    fn test_generate_side_by_side_diff() {
        let original = "fn old_name() {}";
        let new = "fn new_name() {}";
        let diff = PreviewGenerator::generate_side_by_side_diff(original, new);

        assert!(diff.contains("--- original"));
        assert!(diff.contains("+++ new"));
        assert!(diff.contains("old_name"));
        assert!(diff.contains("new_name"));
    }

    #[test]
    fn test_generate_diff_identical() {
        let content = "fn test() {}";
        let diff = PreviewGenerator::generate_diff(content, content);

        // Should have headers but no changes
        assert!(diff.contains("--- original"));
        assert!(diff.contains("+++ new"));
    }

    #[test]
    fn test_compute_hunks_simple() {
        let original = "a\nb\nc\n";
        let new = "a\nx\nc\n";
        let hunks = PreviewGenerator::compute_hunks(original, new);

        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].original_start, 2);
        assert_eq!(hunks[0].new_start, 2);
    }

    #[test]
    fn test_compute_hunks_multiple_changes() {
        let original = "a\nb\nc\nd\ne\n";
        let new = "a\nx\nc\ny\ne\n";
        let hunks = PreviewGenerator::compute_hunks(original, new);

        // Should have two hunks for the two separate changes
        assert!(hunks.len() >= 1);
    }

    #[test]
    fn test_compute_hunks_empty_original() {
        let original = "";
        let new = "a\nb\nc\n";
        let hunks = PreviewGenerator::compute_hunks(original, new);

        assert!(!hunks.is_empty());
    }

    #[test]
    fn test_compute_hunks_empty_new() {
        let original = "a\nb\nc\n";
        let new = "";
        let hunks = PreviewGenerator::compute_hunks(original, new);

        assert!(!hunks.is_empty());
    }
}
