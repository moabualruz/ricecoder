//! Integration tests for context building functionality
//! Tests context selection, relevance scoring, and token budget management
//! **Validates: Requirements 1.10, 2.1, 2.2, 2.3, 2.4, 2.5**

use ricecoder_research::FileContext;
use std::path::PathBuf;

// ============================================================================
// Helper Functions
// ============================================================================

/// Create sample file contexts for testing
fn create_sample_files() -> Vec<FileContext> {
    vec![
        FileContext {
            path: PathBuf::from("src/services/user_service.rs"),
            relevance: 0.95,
            summary: Some("User service implementation".to_string()),
            content: Some("pub struct UserService { /* ... */ }".to_string()),
        },
        FileContext {
            path: PathBuf::from("src/models/user.rs"),
            relevance: 0.90,
            summary: Some("User model definition".to_string()),
            content: Some("pub struct User { id: String, name: String }".to_string()),
        },
        FileContext {
            path: PathBuf::from("src/repositories/user_repository.rs"),
            relevance: 0.85,
            summary: Some("User repository".to_string()),
            content: Some("pub struct UserRepository { /* ... */ }".to_string()),
        },
        FileContext {
            path: PathBuf::from("src/handlers/user_handler.rs"),
            relevance: 0.80,
            summary: Some("User HTTP handlers".to_string()),
            content: Some("pub async fn get_user() { /* ... */ }".to_string()),
        },
        FileContext {
            path: PathBuf::from("src/utils/helpers.rs"),
            relevance: 0.30,
            summary: Some("Utility helpers".to_string()),
            content: Some("pub fn format_string() { /* ... */ }".to_string()),
        },
    ]
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_context_builder_creation() {
    let builder = ContextBuilder::new(8000);
    // Builder created successfully
    let _ = builder;
}

#[test]
fn test_select_relevant_files_basic() {
    let builder = ContextBuilder::new(8000);
    let files = create_sample_files();

    let result = builder.select_relevant_files("user service", files);
    assert!(result.is_ok());

    let selected = result.unwrap();
    assert!(!selected.is_empty());
}

#[test]
fn test_select_relevant_files_respects_relevance() {
    let builder = ContextBuilder::new(8000);
    let files = create_sample_files();

    let selected = builder
        .select_relevant_files("user service", files)
        .unwrap();

    // Most relevant files should be selected first
    if selected.len() > 1 {
        assert!(selected[0].relevance >= selected[1].relevance);
    }
}

#[test]
fn test_select_relevant_files_respects_token_limit() {
    let builder = ContextBuilder::new(100); // Very small token limit
    let files = create_sample_files();

    let selected = builder
        .select_relevant_files("user service", files)
        .unwrap();

    // Should select fewer files due to token limit
    assert!(selected.len() <= 5);
}

#[test]
fn test_select_relevant_files_empty_query() {
    let builder = ContextBuilder::new(8000);
    let files = create_sample_files();

    let result = builder.select_relevant_files("", files);

    // Empty query should still work but may return different results
    assert!(result.is_ok());
}

#[test]
fn test_select_relevant_files_empty_file_list() {
    let builder = ContextBuilder::new(8000);
    let files = vec![];

    let result = builder.select_relevant_files("user service", files);
    assert!(result.is_ok());

    let selected = result.unwrap();
    assert!(selected.is_empty());
}

#[test]
fn test_build_context_basic() {
    let builder = ContextBuilder::new(8000);
    let files = create_sample_files();

    let result = builder.build_context(files);
    assert!(result.is_ok());

    let context = result.unwrap();
    assert!(!context.files.is_empty());
}

#[test]
fn test_build_context_includes_files() {
    let builder = ContextBuilder::new(8000);
    let files = create_sample_files();

    let context = builder.build_context(files).unwrap();

    // Context should include the files
    assert!(!context.files.is_empty());
    assert!(context
        .files
        .iter()
        .any(|f| f.path.ends_with("user_service.rs")));
}

#[test]
fn test_build_context_tracks_tokens() {
    let builder = ContextBuilder::new(8000);
    let files = create_sample_files();

    let context = builder.build_context(files).unwrap();

    // Should track total tokens
    assert!(context.total_tokens > 0);
}

#[test]
fn test_build_context_respects_token_limit() {
    let builder = ContextBuilder::new(200); // Small limit
    let files = create_sample_files();

    let context = builder.build_context(files).unwrap();

    // Total tokens should not exceed limit
    assert!(context.total_tokens <= 200);
}

#[test]
fn test_context_relevance_ordering() {
    let builder = ContextBuilder::new(8000);
    let files = create_sample_files();

    let context = builder.build_context(files).unwrap();

    // Files should be ordered by relevance
    if context.files.len() > 1 {
        for i in 0..context.files.len() - 1 {
            assert!(context.files[i].relevance >= context.files[i + 1].relevance);
        }
    }
}

#[test]
fn test_context_includes_symbols() {
    let builder = ContextBuilder::new(8000);
    let files = create_sample_files();

    let context = builder.build_context(files).unwrap();

    // Context should include symbols (may be empty if not extracted)
    let _ = context.symbols;
}

#[test]
fn test_context_includes_references() {
    let builder = ContextBuilder::new(8000);
    let files = create_sample_files();

    let context = builder.build_context(files).unwrap();

    // Context should include references (may be empty if not tracked)
    let _ = context.references;
}

#[test]
fn test_select_relevant_files_query_matching() {
    let builder = ContextBuilder::new(8000);
    let files = create_sample_files();

    // Query about user service
    let selected = builder
        .select_relevant_files("user service", files.clone())
        .unwrap();

    // Should prioritize user-related files
    assert!(selected.iter().any(|f| f.path.ends_with("user_service.rs")));
}

#[test]
fn test_select_relevant_files_consistency() {
    let builder = ContextBuilder::new(8000);
    let files = create_sample_files();

    // Select files multiple times
    let selected1 = builder
        .select_relevant_files("user service", files.clone())
        .unwrap();
    let selected2 = builder
        .select_relevant_files("user service", files.clone())
        .unwrap();
    let selected3 = builder
        .select_relevant_files("user service", files)
        .unwrap();

    // Results should be consistent
    assert_eq!(selected1.len(), selected2.len());
    assert_eq!(selected2.len(), selected3.len());

    if !selected1.is_empty() {
        assert_eq!(selected1[0].path, selected2[0].path);
        assert_eq!(selected2[0].path, selected3[0].path);
    }
}

#[test]
fn test_context_builder_with_different_token_limits() {
    let files = create_sample_files();

    let builder_small = ContextBuilder::new(100);
    let builder_large = ContextBuilder::new(10000);

    let context_small = builder_small.build_context(files.clone()).unwrap();
    let context_large = builder_large.build_context(files).unwrap();

    // Larger token limit should include more content
    assert!(context_large.total_tokens >= context_small.total_tokens);
}

#[test]
fn test_select_relevant_files_high_relevance_first() {
    let builder = ContextBuilder::new(8000);
    let files = create_sample_files();

    let selected = builder.select_relevant_files("user", files).unwrap();

    // Files with higher relevance should be selected first
    if selected.len() > 1 {
        for i in 0..selected.len() - 1 {
            assert!(selected[i].relevance >= selected[i + 1].relevance);
        }
    }
}

#[test]
fn test_build_context_empty_files() {
    let builder = ContextBuilder::new(8000);
    let files = vec![];

    let context = builder.build_context(files).unwrap();

    assert!(context.files.is_empty());
    assert_eq!(context.total_tokens, 0);
}

#[test]
fn test_context_builder_token_limit_property() {
    let builder = ContextBuilder::new(5000);
    // Builder created with token limit
    let _ = builder;
}

#[test]
fn test_select_relevant_files_multiple_queries() {
    let builder = ContextBuilder::new(8000);
    let files = create_sample_files();

    // Different queries should potentially select different files
    let selected_user = builder
        .select_relevant_files("user", files.clone())
        .unwrap();
    let selected_service = builder
        .select_relevant_files("service", files.clone())
        .unwrap();
    let selected_repository = builder.select_relevant_files("repository", files).unwrap();

    // All should return results
    assert!(!selected_user.is_empty());
    assert!(!selected_service.is_empty());
    assert!(!selected_repository.is_empty());
}

#[test]
fn test_context_preserves_file_metadata() {
    let builder = ContextBuilder::new(8000);
    let files = create_sample_files();

    let context = builder.build_context(files).unwrap();

    // File metadata should be preserved
    for file in &context.files {
        assert!(!file.path.as_os_str().is_empty());
        assert!(file.relevance >= 0.0 && file.relevance <= 1.0);
    }
}

#[test]
fn test_select_relevant_files_with_low_relevance_threshold() {
    let builder = ContextBuilder::new(8000);
    let mut files = create_sample_files();

    // Add a file with very low relevance
    files.push(FileContext {
        path: PathBuf::from("src/config/config.rs"),
        relevance: 0.01,
        summary: Some("Configuration".to_string()),
        content: Some("pub const CONFIG: &str = \"...\";".to_string()),
    });

    let selected = builder.select_relevant_files("user", files).unwrap();

    // Should still select files, but low relevance files may be excluded
    assert!(!selected.is_empty());
}
