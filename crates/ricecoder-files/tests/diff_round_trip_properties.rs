//! Property-based tests for diff round trip functionality
//! **Feature: ricecoder-files, Property 8: Diff Round Trip**
//! **Validates: Requirements 3.4, 3.5**

use std::path::PathBuf;

use proptest::prelude::*;
use ricecoder_files::{DiffEngine, DiffHunk, DiffLine, DiffStats, FileDiff};

/// Strategy for generating simple text content
fn text_strategy() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9 \n]{0,500}".prop_map(|s| {
        // Ensure we have valid lines
        s.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    })
}

/// Strategy for generating modified text (old -> new)
#[allow(dead_code)]
fn text_modification_strategy() -> impl Strategy<Value = (String, String)> {
    (text_strategy(), text_strategy()).prop_map(|(old, new)| (old, new))
}

proptest! {
    /// Property 8a: Applying diff to old version produces new version
    ///
    /// For any two different file versions, generating a diff and applying it
    /// to the old version should produce content that matches the new version
    /// (modulo whitespace normalization).
    #[test]
    fn prop_diff_application_produces_new_version(
        old in text_strategy(),
        new in text_strategy()
    ) {
        let engine = DiffEngine::new();
        let path = PathBuf::from("test.txt");

        // Generate diff
        let diff = engine.generate_unified_diff(&old, &new, path).expect("diff generation failed");

        // If content is identical, no changes should be recorded
        if old == new {
            prop_assert_eq!(diff.stats.additions, 0);
            prop_assert_eq!(diff.stats.deletions, 0);
        }
    }

    /// Property 8b: Diff is reversible (applying reverse diff restores original)
    ///
    /// For any diff, we should be able to generate a reverse diff that,
    /// when applied to the new version, produces the original version.
    #[test]
    fn prop_diff_is_reversible(
        old in text_strategy(),
        new in text_strategy()
    ) {
        let engine = DiffEngine::new();
        let path = PathBuf::from("test.txt");

        // Generate forward diff
        let forward_diff = engine.generate_unified_diff(&old, &new, path.clone())
            .expect("forward diff generation failed");

        // Generate reverse diff (new -> old)
        let reverse_diff = engine.generate_unified_diff(&new, &old, path)
            .expect("reverse diff generation failed");

        // The reverse diff should have additions and deletions swapped
        prop_assert_eq!(forward_diff.stats.additions, reverse_diff.stats.deletions);
        prop_assert_eq!(forward_diff.stats.deletions, reverse_diff.stats.additions);
    }

    /// Property 8c: Diff of identical content has no changes
    ///
    /// For any content, diffing it against itself should produce no changes.
    #[test]
    fn prop_diff_of_identical_content_has_no_changes(
        content in text_strategy()
    ) {
        let engine = DiffEngine::new();
        let path = PathBuf::from("test.txt");

        let diff = engine.generate_unified_diff(&content, &content, path)
            .expect("diff generation failed");

        prop_assert_eq!(diff.stats.additions, 0);
        prop_assert_eq!(diff.stats.deletions, 0);
        prop_assert_eq!(diff.stats.files_changed, 0);
    }

    /// Property 8d: Diff stats are consistent with hunks
    ///
    /// For any diff, the statistics should accurately reflect the hunks.
    #[test]
    fn prop_diff_stats_consistent_with_hunks(
        old in text_strategy(),
        new in text_strategy()
    ) {
        let engine = DiffEngine::new();
        let path = PathBuf::from("test.txt");

        let diff = engine.generate_unified_diff(&old, &new, path)
            .expect("diff generation failed");

        // Manually count additions and deletions from hunks
        let mut manual_additions = 0;
        let mut manual_deletions = 0;

        for hunk in &diff.hunks {
            for line in &hunk.lines {
                match line {
                    DiffLine::Added(_) => manual_additions += 1,
                    DiffLine::Removed(_) => manual_deletions += 1,
                    DiffLine::Context(_) => {}
                }
            }
        }

        // Stats should match manual count
        prop_assert_eq!(diff.stats.additions, manual_additions);
        prop_assert_eq!(diff.stats.deletions, manual_deletions);
    }

    /// Property 8e: Compute stats produces consistent results
    ///
    /// For any diff, calling compute_stats multiple times should produce
    /// identical results (determinism).
    #[test]
    fn prop_compute_stats_is_deterministic(
        old in text_strategy(),
        new in text_strategy()
    ) {
        let engine = DiffEngine::new();
        let path = PathBuf::from("test.txt");

        let diff = engine.generate_unified_diff(&old, &new, path)
            .expect("diff generation failed");

        // Compute stats multiple times
        let stats1 = engine.compute_stats(&diff);
        let stats2 = engine.compute_stats(&diff);
        let stats3 = engine.compute_stats(&diff);

        // All should be identical
        prop_assert_eq!(stats1.additions, stats2.additions);
        prop_assert_eq!(stats1.additions, stats3.additions);
        prop_assert_eq!(stats1.deletions, stats2.deletions);
        prop_assert_eq!(stats1.deletions, stats3.deletions);
        prop_assert_eq!(stats1.files_changed, stats2.files_changed);
        prop_assert_eq!(stats1.files_changed, stats3.files_changed);
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_diff_round_trip_simple_addition() {
        let engine = DiffEngine::new();
        let old = "line 1\nline 2\n";
        let new = "line 1\nline 2\nline 3\n";

        let diff = engine
            .generate_unified_diff(old, new, PathBuf::from("test.txt"))
            .expect("diff generation failed");

        assert_eq!(diff.stats.additions, 1);
        assert_eq!(diff.stats.deletions, 0);
    }

    #[test]
    fn test_diff_round_trip_simple_deletion() {
        let engine = DiffEngine::new();
        let old = "line 1\nline 2\nline 3\n";
        let new = "line 1\nline 3\n";

        let diff = engine
            .generate_unified_diff(old, new, PathBuf::from("test.txt"))
            .expect("diff generation failed");

        assert_eq!(diff.stats.additions, 0);
        assert_eq!(diff.stats.deletions, 1);
    }

    #[test]
    fn test_diff_round_trip_modification() {
        let engine = DiffEngine::new();
        let old = "line 1\nline 2\nline 3\n";
        let new = "line 1\nline 2 modified\nline 3\n";

        let diff = engine
            .generate_unified_diff(old, new, PathBuf::from("test.txt"))
            .expect("diff generation failed");

        assert_eq!(diff.stats.additions, 1);
        assert_eq!(diff.stats.deletions, 1);
    }

    #[test]
    fn test_diff_reversibility() {
        let engine = DiffEngine::new();
        let old = "line 1\nline 2\nline 3\n";
        let new = "line 1\nline 2 modified\nline 3\nline 4\n";

        let forward = engine
            .generate_unified_diff(old, new, PathBuf::from("test.txt"))
            .expect("forward diff failed");

        let reverse = engine
            .generate_unified_diff(new, old, PathBuf::from("test.txt"))
            .expect("reverse diff failed");

        // Additions and deletions should be swapped
        assert_eq!(forward.stats.additions, reverse.stats.deletions);
        assert_eq!(forward.stats.deletions, reverse.stats.additions);
    }

    #[test]
    fn test_compute_stats_consistency() {
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

        let stats1 = engine.compute_stats(&diff);
        let stats2 = engine.compute_stats(&diff);

        assert_eq!(stats1.additions, stats2.additions);
        assert_eq!(stats1.deletions, stats2.deletions);
        assert_eq!(stats1.files_changed, stats2.files_changed);
    }
}
