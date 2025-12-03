//! Unit tests for ContextBuilder and related components
//! Tests for context selection, optimization, and provision

use ricecoder_research::{
    ContextBuilder, ContextOptimizer, ContextProvider, RelevanceScorer, FileContext,
};
use std::path::PathBuf;

// ============================================================================
// ContextBuilder Unit Tests
// ============================================================================

#[test]
fn test_context_builder_new() {
    let builder = ContextBuilder::new(8192);
    assert_eq!(builder.max_tokens(), 8192);
}

#[test]
fn test_context_builder_default() {
    let builder = ContextBuilder::default();
    assert_eq!(builder.max_tokens(), 4096);
}

#[test]
fn test_context_builder_set_max_tokens() {
    let mut builder = ContextBuilder::new(4096);
    builder.set_max_tokens(8192);
    assert_eq!(builder.max_tokens(), 8192);
}

#[test]
fn test_select_relevant_files_empty_list() {
    let builder = ContextBuilder::new(4096);
    let result = builder.select_relevant_files("test", vec![]);
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn test_select_relevant_files_single_file() {
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
    assert_eq!(files.len(), 1);
    assert!(files[0].relevance > 0.0);
}

#[test]
fn test_select_relevant_files_multiple_files() {
    let builder = ContextBuilder::new(4096);
    let files = vec![
        FileContext {
            path: PathBuf::from("src/main.rs"),
            relevance: 0.0,
            summary: Some("Main entry point".to_string()),
            content: Some("fn main() {}".to_string()),
        },
        FileContext {
            path: PathBuf::from("src/lib.rs"),
            relevance: 0.0,
            summary: Some("Library module".to_string()),
            content: Some("pub fn helper() {}".to_string()),
        },
        FileContext {
            path: PathBuf::from("src/utils.rs"),
            relevance: 0.0,
            summary: None,
            content: None,
        },
    ];

    let result = builder.select_relevant_files("main", files);
    assert!(result.is_ok());

    let selected = result.unwrap();
    // Should select files with relevance > 0
    assert!(!selected.is_empty());
    // Should be ordered by relevance
    for i in 0..selected.len().saturating_sub(1) {
        assert!(selected[i].relevance >= selected[i + 1].relevance);
    }
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
fn test_build_context_single_file() {
    let builder = ContextBuilder::new(4096);
    let file = FileContext {
        path: PathBuf::from("src/main.rs"),
        relevance: 0.9,
        summary: None,
        content: Some("fn main() {}".to_string()),
    };

    let result = builder.build_context(vec![file]);
    assert!(result.is_ok());

    let context = result.unwrap();
    assert_eq!(context.files.len(), 1);
    assert!(context.total_tokens > 0);
}

#[test]
fn test_build_context_respects_token_budget() {
    let builder = ContextBuilder::new(100); // Very small budget

    let files = vec![
        FileContext {
            path: PathBuf::from("src/file1.rs"),
            relevance: 0.9,
            summary: None,
            content: Some("x".repeat(200)), // ~50 tokens
        },
        FileContext {
            path: PathBuf::from("src/file2.rs"),
            relevance: 0.8,
            summary: None,
            content: Some("y".repeat(200)), // ~50 tokens
        },
        FileContext {
            path: PathBuf::from("src/file3.rs"),
            relevance: 0.7,
            summary: None,
            content: Some("z".repeat(200)), // ~50 tokens
        },
    ];

    let result = builder.build_context(files);
    assert!(result.is_ok());

    let context = result.unwrap();
    assert!(context.total_tokens <= 100);
}

// ============================================================================
// RelevanceScorer Unit Tests
// ============================================================================

#[test]
fn test_relevance_scorer_new() {
    let scorer = RelevanceScorer::new();
    assert_eq!(scorer.weights().path_exact_match, 0.8);
}

#[test]
fn test_relevance_scorer_default() {
    let scorer = RelevanceScorer::default();
    assert_eq!(scorer.weights().path_exact_match, 0.8);
}

#[test]
fn test_score_file_path_match() {
    let scorer = RelevanceScorer::new();
    let file = FileContext {
        path: PathBuf::from("src/utils.rs"),
        relevance: 0.0,
        summary: None,
        content: None,
    };

    let score = scorer.score_file(&file, "utils");
    assert!(score > 0.0);
}

#[test]
fn test_score_file_content_match() {
    let scorer = RelevanceScorer::new();
    let file = FileContext {
        path: PathBuf::from("src/main.rs"),
        relevance: 0.0,
        summary: None,
        content: Some("fn helper_function() {}".to_string()),
    };

    let score = scorer.score_file(&file, "helper");
    assert!(score > 0.0);
}

#[test]
fn test_score_file_summary_match() {
    let scorer = RelevanceScorer::new();
    let file = FileContext {
        path: PathBuf::from("src/main.rs"),
        relevance: 0.0,
        summary: Some("Utility functions for parsing".to_string()),
        content: None,
    };

    let score = scorer.score_file(&file, "parsing");
    assert!(score > 0.0);
}

#[test]
fn test_score_file_no_match() {
    let scorer = RelevanceScorer::new();
    let file = FileContext {
        path: PathBuf::from("src/main.rs"),
        relevance: 0.0,
        summary: None,
        content: Some("fn main() {}".to_string()),
    };

    let score = scorer.score_file(&file, "nonexistent");
    assert_eq!(score, 0.0);
}

#[test]
fn test_score_files() {
    let scorer = RelevanceScorer::new();
    let files = vec![
        FileContext {
            path: PathBuf::from("src/utils.rs"),
            relevance: 0.0,
            summary: None,
            content: None,
        },
        FileContext {
            path: PathBuf::from("src/main.rs"),
            relevance: 0.0,
            summary: None,
            content: None,
        },
    ];

    let scores = scorer.score_files(&files, "utils");
    assert_eq!(scores.len(), 2);
    // First file should have higher score
    assert!(scores[0].1 > scores[1].1);
}

#[test]
fn test_score_symbols() {
    use ricecoder_research::{Symbol, SymbolKind};

    let scorer = RelevanceScorer::new();
    let symbols = vec![
        Symbol {
            id: "1".to_string(),
            name: "helper_function".to_string(),
            kind: SymbolKind::Function,
            file: PathBuf::from("src/main.rs"),
            line: 1,
            column: 1,
            references: vec![],
        },
        Symbol {
            id: "2".to_string(),
            name: "main".to_string(),
            kind: SymbolKind::Function,
            file: PathBuf::from("src/main.rs"),
            line: 10,
            column: 1,
            references: vec![],
        },
    ];

    let scores = scorer.score_symbols(&symbols, "helper");
    assert_eq!(scores.len(), 2);
    // First symbol should have higher score
    assert!(scores[0].1 > scores[1].1);
}

// ============================================================================
// ContextOptimizer Unit Tests
// ============================================================================

#[test]
fn test_context_optimizer_new() {
    let optimizer = ContextOptimizer::new(2048);
    assert_eq!(optimizer.max_tokens_per_file(), 2048);
}

#[test]
fn test_context_optimizer_default() {
    let optimizer = ContextOptimizer::default();
    assert_eq!(optimizer.max_tokens_per_file(), 2048);
}

#[test]
fn test_estimate_tokens() {
    let optimizer = ContextOptimizer::new(2048);
    let content = "x".repeat(400); // 400 chars = ~100 tokens
    let tokens = optimizer.estimate_tokens(&content);
    assert_eq!(tokens, 100);
}

#[test]
fn test_optimize_file_small_content() {
    let optimizer = ContextOptimizer::new(2048);
    let file = FileContext {
        path: PathBuf::from("src/main.rs"),
        relevance: 0.9,
        summary: None,
        content: Some("fn main() {}".to_string()),
    };

    let result = optimizer.optimize_file(&file);
    assert!(result.is_ok());

    let optimized = result.unwrap();
    assert_eq!(optimized.content, file.content);
}

#[test]
fn test_optimize_file_large_content() {
    let optimizer = ContextOptimizer::new(100); // Very small budget

    let large_content = "fn main() {}\n".repeat(100);
    let file = FileContext {
        path: PathBuf::from("src/main.rs"),
        relevance: 0.9,
        summary: None,
        content: Some(large_content),
    };

    let result = optimizer.optimize_file(&file);
    assert!(result.is_ok());

    let optimized = result.unwrap();
    assert!(optimized.content.is_some());
    // Optimized content should be smaller
    let optimized_tokens = optimizer.estimate_tokens(optimized.content.as_ref().unwrap());
    assert!(optimized_tokens <= 100);
}

#[test]
fn test_optimize_files() {
    let optimizer = ContextOptimizer::new(2048);
    let files = vec![
        FileContext {
            path: PathBuf::from("src/main.rs"),
            relevance: 0.9,
            summary: None,
            content: Some("fn main() {}".to_string()),
        },
        FileContext {
            path: PathBuf::from("src/lib.rs"),
            relevance: 0.8,
            summary: None,
            content: Some("pub fn helper() {}".to_string()),
        },
    ];

    let result = optimizer.optimize_files(files);
    assert!(result.is_ok());

    let optimized = result.unwrap();
    assert_eq!(optimized.len(), 2);
}

#[test]
fn test_extract_key_sections() {
    let optimizer = ContextOptimizer::new(2048);
    let content = "fn helper() {\n    println!(\"hello\");\n}\n\nfn main() {\n    helper();\n}\n";

    let sections = optimizer.extract_key_sections(content);
    assert!(!sections.is_empty());
}

#[test]
fn test_set_max_tokens_per_file() {
    let mut optimizer = ContextOptimizer::new(2048);
    optimizer.set_max_tokens_per_file(4096);
    assert_eq!(optimizer.max_tokens_per_file(), 4096);
}

// ============================================================================
// ContextProvider Unit Tests
// ============================================================================

#[test]
fn test_context_provider_new() {
    let provider = ContextProvider::new(4096, 2048);
    assert_eq!(provider.context_builder().max_tokens(), 4096);
    assert_eq!(provider.context_optimizer().max_tokens_per_file(), 2048);
}

#[test]
fn test_context_provider_default() {
    let provider = ContextProvider::default();
    assert_eq!(provider.context_builder().max_tokens(), 4096);
    assert_eq!(provider.context_optimizer().max_tokens_per_file(), 2048);
}

#[test]
fn test_provide_context_for_generation() {
    let provider = ContextProvider::new(4096, 2048);

    let files = vec![
        FileContext {
            path: PathBuf::from("src/main.rs"),
            relevance: 0.0,
            summary: Some("Main entry point".to_string()),
            content: Some("fn main() {}".to_string()),
        },
        FileContext {
            path: PathBuf::from("src/lib.rs"),
            relevance: 0.0,
            summary: Some("Library module".to_string()),
            content: Some("pub fn helper() {}".to_string()),
        },
    ];

    let result = provider.provide_context_for_generation("main", files);
    assert!(result.is_ok());

    let context = result.unwrap();
    assert!(!context.files.is_empty());
}

#[test]
fn test_provide_context_for_review() {
    let provider = ContextProvider::new(4096, 2048);

    let files = vec![
        FileContext {
            path: PathBuf::from("src/main.rs"),
            relevance: 0.0,
            summary: Some("Main entry point".to_string()),
            content: Some("fn main() {}".to_string()),
        },
    ];

    let result = provider.provide_context_for_review("main", files);
    assert!(result.is_ok());

    let context = result.unwrap();
    assert!(!context.files.is_empty());
}

#[test]
fn test_provide_context_for_refactoring() {
    let provider = ContextProvider::new(4096, 2048);

    let files = vec![
        FileContext {
            path: PathBuf::from("src/main.rs"),
            relevance: 0.0,
            summary: Some("Main entry point".to_string()),
            content: Some("fn main() {}".to_string()),
        },
    ];

    let result = provider.provide_context_for_refactoring("main", files);
    assert!(result.is_ok());

    let context = result.unwrap();
    assert!(!context.files.is_empty());
}

#[test]
fn test_provide_context_for_documentation() {
    let provider = ContextProvider::new(4096, 2048);

    let files = vec![
        FileContext {
            path: PathBuf::from("src/main.rs"),
            relevance: 0.0,
            summary: Some("Main entry point".to_string()),
            content: Some("fn main() {}".to_string()),
        },
        FileContext {
            path: PathBuf::from("src/lib.rs"),
            relevance: 0.0,
            summary: None,
            content: Some("pub fn helper() {}".to_string()),
        },
    ];

    let result = provider.provide_context_for_documentation("main", files);
    assert!(result.is_ok());

    let context = result.unwrap();
    assert!(!context.files.is_empty());
}

#[test]
fn test_provide_context_empty_files() {
    let provider = ContextProvider::new(4096, 2048);
    let result = provider.provide_context_for_generation("test", vec![]);
    assert!(result.is_ok());

    let context = result.unwrap();
    assert!(context.files.is_empty());
}

#[test]
fn test_set_max_tokens() {
    let mut provider = ContextProvider::new(4096, 2048);
    provider.set_max_tokens(8192);
    assert_eq!(provider.context_builder().max_tokens(), 8192);
}

#[test]
fn test_context_provider_set_max_tokens_per_file() {
    let mut provider = ContextProvider::new(4096, 2048);
    provider.set_max_tokens_per_file(4096);
    assert_eq!(provider.context_optimizer().max_tokens_per_file(), 4096);
}

#[test]
fn test_context_provider_prioritizes_summaries() {
    let provider = ContextProvider::new(4096, 2048);

    let files = vec![
        FileContext {
            path: PathBuf::from("src/main.rs"),
            relevance: 0.0,
            summary: Some("Main entry point".to_string()),
            content: Some("fn main() {}".to_string()),
        },
        FileContext {
            path: PathBuf::from("src/lib.rs"),
            relevance: 0.0,
            summary: None,
            content: Some("pub fn helper() {}".to_string()),
        },
    ];

    let result = provider.provide_context_for_documentation("main", files);
    assert!(result.is_ok());

    let context = result.unwrap();
    // File with summary should be first
    if context.files.len() >= 2 {
        assert!(context.files[0].summary.is_some());
    }
}
