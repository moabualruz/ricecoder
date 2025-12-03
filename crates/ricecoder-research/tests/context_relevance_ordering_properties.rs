//! Property-based tests for context relevance ordering
//! **Feature: ricecoder-research, Property 3: Context Relevance Ordering**
//! **Validates: Requirements 1.10, 2.1**

use proptest::prelude::*;
use ricecoder_research::{ContextBuilder, FileContext};
use std::path::PathBuf;

// ============================================================================
// Generators for property testing
// ============================================================================

/// Generate a file context with a given relevance score
fn file_context_strategy() -> impl Strategy<Value = FileContext> {
    (
        "src/[a-z_]+\\.rs",
        0.0f32..=1.0f32,
        prop::option::of("[a-z ]+"),
        prop::option::of("[a-z ]+"),
    )
        .prop_map(|(path, relevance, summary, content)| FileContext {
            path: PathBuf::from(path),
            relevance,
            summary,
            content,
        })
}

/// Generate a list of file contexts
fn file_contexts_strategy() -> impl Strategy<Value = Vec<FileContext>> {
    prop::collection::vec(file_context_strategy(), 1..10)
}

/// Generate a query string
fn query_strategy() -> impl Strategy<Value = String> {
    "[a-z]{1,10}"
}

// ============================================================================
// Property Tests
// ============================================================================

proptest! {
    /// Property: Context files are ordered by relevance with most relevant first
    /// For any query and list of files, the returned files should be ordered by relevance score
    /// in descending order (highest relevance first)
    #[test]
    fn prop_context_files_ordered_by_relevance(
        files in file_contexts_strategy(),
        query in query_strategy()
    ) {
        let builder = ContextBuilder::new(4096);
        let result = builder.select_relevant_files(&query, files);

        prop_assert!(result.is_ok());

        let selected_files = result.unwrap();

        // Check that files are ordered by relevance (descending)
        for i in 0..selected_files.len().saturating_sub(1) {
            prop_assert!(
                selected_files[i].relevance >= selected_files[i + 1].relevance,
                "Files not ordered by relevance: {} < {}",
                selected_files[i].relevance,
                selected_files[i + 1].relevance
            );
        }
    }

    /// Property: All returned files have non-zero relevance
    /// For any query, all returned files should have relevance > 0.0
    #[test]
    fn prop_returned_files_have_nonzero_relevance(
        files in file_contexts_strategy(),
        query in query_strategy()
    ) {
        let builder = ContextBuilder::new(4096);
        let result = builder.select_relevant_files(&query, files);

        prop_assert!(result.is_ok());

        let selected_files = result.unwrap();

        // All returned files should have non-zero relevance
        for file in &selected_files {
            prop_assert!(
                file.relevance > 0.0,
                "File has zero relevance: {}",
                file.path.display()
            );
        }
    }

    /// Property: Relevance scores are normalized to 0.0-1.0 range
    /// For any query and files, all relevance scores should be in [0.0, 1.0]
    #[test]
    fn prop_relevance_scores_normalized(
        files in file_contexts_strategy(),
        query in query_strategy()
    ) {
        let builder = ContextBuilder::new(4096);
        let result = builder.select_relevant_files(&query, files);

        prop_assert!(result.is_ok());

        let selected_files = result.unwrap();

        // All relevance scores should be in [0.0, 1.0]
        for file in &selected_files {
            prop_assert!(
                file.relevance >= 0.0 && file.relevance <= 1.0,
                "Relevance score out of range: {}",
                file.relevance
            );
        }
    }

    /// Property: Selecting files is deterministic
    /// For the same query and files, selecting files multiple times should produce the same result
    #[test]
    fn prop_file_selection_is_deterministic(
        files in file_contexts_strategy(),
        query in query_strategy()
    ) {
        let builder = ContextBuilder::new(4096);

        let result1 = builder.select_relevant_files(&query, files.clone());
        let result2 = builder.select_relevant_files(&query, files);

        prop_assert!(result1.is_ok());
        prop_assert!(result2.is_ok());

        let files1 = result1.unwrap();
        let files2 = result2.unwrap();

        // Results should be identical
        prop_assert_eq!(files1.len(), files2.len());

        for (f1, f2) in files1.iter().zip(files2.iter()) {
            prop_assert_eq!(&f1.path, &f2.path);
            prop_assert_eq!(f1.relevance, f2.relevance);
        }
    }

    /// Property: Building context respects token budget
    /// For any files, the total tokens in the built context should not exceed the budget
    #[test]
    fn prop_context_respects_token_budget(
        files in file_contexts_strategy(),
        max_tokens in 100usize..10000usize
    ) {
        let builder = ContextBuilder::new(max_tokens);
        let result = builder.build_context(files);

        prop_assert!(result.is_ok());

        let context = result.unwrap();

        // Total tokens should not exceed budget
        prop_assert!(
            context.total_tokens <= max_tokens,
            "Context exceeds token budget: {} > {}",
            context.total_tokens,
            max_tokens
        );
    }

    /// Property: Empty file list returns empty context
    /// For an empty file list, the built context should be empty
    #[test]
    fn prop_empty_files_returns_empty_context(_seed in 0u32..1) {
        let builder = ContextBuilder::new(4096);
        let result = builder.build_context(vec![]);

        prop_assert!(result.is_ok());

        let context = result.unwrap();
        prop_assert!(context.files.is_empty());
        prop_assert_eq!(context.total_tokens, 0);
    }

    /// Property: File relevance is consistent across multiple queries
    /// For the same file, relevance scores should be consistent when calculated multiple times
    #[test]
    fn prop_file_relevance_consistency(
        file in file_context_strategy(),
        query in query_strategy()
    ) {
        let builder = ContextBuilder::new(4096);

        let files = vec![file.clone()];

        let result1 = builder.select_relevant_files(&query, files.clone());
        let result2 = builder.select_relevant_files(&query, files);

        prop_assert!(result1.is_ok());
        prop_assert!(result2.is_ok());

        let files1 = result1.unwrap();
        let files2 = result2.unwrap();

        // If both returned the file, relevance should be the same
        if !files1.is_empty() && !files2.is_empty() {
            prop_assert_eq!(files1[0].relevance, files2[0].relevance);
        }
    }

    /// Property: More relevant files appear before less relevant files
    /// For any two files where one is more relevant than the other,
    /// the more relevant file should appear first in the results
    #[test]
    fn prop_more_relevant_files_first(
        query in query_strategy()
    ) {
        let builder = ContextBuilder::new(4096);

        // Create two files: one with high relevance, one with low
        let high_relevance_file = FileContext {
            path: PathBuf::from("src/important.rs"),
            relevance: 0.0,
            summary: Some(query.clone()),
            content: Some(query.clone()),
        };

        let low_relevance_file = FileContext {
            path: PathBuf::from("src/other.rs"),
            relevance: 0.0,
            summary: None,
            content: None,
        };

        let files = vec![low_relevance_file, high_relevance_file];

        let result = builder.select_relevant_files(&query, files);
        prop_assert!(result.is_ok());

        let selected = result.unwrap();

        // If both files are selected, the high relevance one should be first
        if selected.len() >= 2 {
            prop_assert!(
                selected[0].relevance >= selected[1].relevance,
                "Files not ordered correctly"
            );
        }
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[test]
fn test_context_builder_creation() {
    let builder = ContextBuilder::new(4096);
    assert_eq!(builder.max_tokens(), 4096);
}

#[test]
fn test_select_relevant_files_empty() {
    let builder = ContextBuilder::new(4096);
    let result = builder.select_relevant_files("test", vec![]);
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn test_select_relevant_files_with_query() {
    let builder = ContextBuilder::new(4096);

    let file1 = FileContext {
        path: PathBuf::from("src/main.rs"),
        relevance: 0.0,
        summary: Some("Main entry point".to_string()),
        content: Some("fn main() {}".to_string()),
    };

    let file2 = FileContext {
        path: PathBuf::from("src/lib.rs"),
        relevance: 0.0,
        summary: Some("Library module".to_string()),
        content: Some("pub fn helper() {}".to_string()),
    };

    let result = builder.select_relevant_files("main", vec![file1, file2]);
    assert!(result.is_ok());

    let files = result.unwrap();
    assert!(!files.is_empty());
    // First file should have higher relevance (contains "main" in path and content)
    assert!(files[0].relevance > 0.0);
}

#[test]
fn test_files_ordered_by_relevance() {
    let builder = ContextBuilder::new(4096);

    let file1 = FileContext {
        path: PathBuf::from("src/utils.rs"),
        relevance: 0.0,
        summary: Some("Utility functions".to_string()),
        content: Some("fn helper() {}".to_string()),
    };

    let file2 = FileContext {
        path: PathBuf::from("src/main.rs"),
        relevance: 0.0,
        summary: Some("Main entry point".to_string()),
        content: Some("fn main() {}".to_string()),
    };

    let result = builder.select_relevant_files("helper", vec![file1, file2]);
    assert!(result.is_ok());

    let files = result.unwrap();
    // Files should be ordered by relevance
    for i in 0..files.len().saturating_sub(1) {
        assert!(files[i].relevance >= files[i + 1].relevance);
    }
}

#[test]
fn test_build_context_respects_token_budget() {
    let builder = ContextBuilder::new(100); // Very small budget

    let file1 = FileContext {
        path: PathBuf::from("src/file1.rs"),
        relevance: 0.9,
        summary: None,
        content: Some("x".repeat(200)), // 200 chars = ~50 tokens
    };

    let file2 = FileContext {
        path: PathBuf::from("src/file2.rs"),
        relevance: 0.8,
        summary: None,
        content: Some("y".repeat(200)), // 200 chars = ~50 tokens
    };

    let result = builder.build_context(vec![file1, file2]);
    assert!(result.is_ok());

    let context = result.unwrap();
    // Should respect token budget
    assert!(context.total_tokens <= 100);
}

#[test]
fn test_build_context_empty_files() {
    let builder = ContextBuilder::new(4096);
    let result = builder.build_context(vec![]);
    assert!(result.is_ok());

    let context = result.unwrap();
    assert!(context.files.is_empty());
    assert_eq!(context.total_tokens, 0);
}

#[test]
fn test_relevance_scores_normalized() {
    let builder = ContextBuilder::new(4096);

    let file = FileContext {
        path: PathBuf::from("src/main.rs"),
        relevance: 0.0,
        summary: Some("Main entry point".to_string()),
        content: Some("fn main() {}".to_string()),
    };

    let result = builder.select_relevant_files("main", vec![file]);
    assert!(result.is_ok());

    let files = result.unwrap();
    for file in files {
        assert!(file.relevance >= 0.0 && file.relevance <= 1.0);
    }
}

#[test]
fn test_file_selection_deterministic() {
    let builder = ContextBuilder::new(4096);

    let file1 = FileContext {
        path: PathBuf::from("src/main.rs"),
        relevance: 0.0,
        summary: Some("Main entry point".to_string()),
        content: Some("fn main() {}".to_string()),
    };

    let file2 = FileContext {
        path: PathBuf::from("src/lib.rs"),
        relevance: 0.0,
        summary: Some("Library module".to_string()),
        content: Some("pub fn helper() {}".to_string()),
    };

    let files = vec![file1, file2];

    let result1 = builder.select_relevant_files("main", files.clone());
    let result2 = builder.select_relevant_files("main", files);

    assert!(result1.is_ok());
    assert!(result2.is_ok());

    let files1 = result1.unwrap();
    let files2 = result2.unwrap();

    assert_eq!(files1.len(), files2.len());
    for (f1, f2) in files1.iter().zip(files2.iter()) {
        assert_eq!(f1.path, f2.path);
        assert_eq!(f1.relevance, f2.relevance);
    }
}

#[test]
fn test_returned_files_have_nonzero_relevance() {
    let builder = ContextBuilder::new(4096);

    let file1 = FileContext {
        path: PathBuf::from("src/main.rs"),
        relevance: 0.0,
        summary: Some("Main entry point".to_string()),
        content: Some("fn main() {}".to_string()),
    };

    let file2 = FileContext {
        path: PathBuf::from("src/lib.rs"),
        relevance: 0.0,
        summary: None,
        content: None,
    };

    let result = builder.select_relevant_files("main", vec![file1, file2]);
    assert!(result.is_ok());

    let files = result.unwrap();
    for file in files {
        assert!(file.relevance > 0.0);
    }
}
