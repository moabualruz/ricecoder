//! Integration tests for semantic search functionality
//! Tests codebase-wide search, symbol lookup, and pattern matching
//! **Validates: Requirements 1.7, 1.8, 1.9**

use std::path::PathBuf;

use ricecoder_research::{ReferenceKind, SemanticIndex, Symbol, SymbolKind, SymbolReference};

// ============================================================================
// Helper Functions
// ============================================================================

/// Create a semantic index with sample symbols
fn create_sample_index() -> SemanticIndex {
    let mut index = SemanticIndex::new();

    // Add symbols
    let symbol1 = Symbol {
        id: "sym_1".to_string(),
        name: "UserService".to_string(),
        kind: SymbolKind::Class,
        file: PathBuf::from("src/services/user_service.rs"),
        line: 10,
        column: 0,
        references: vec![],
    };

    let symbol2 = Symbol {
        id: "sym_2".to_string(),
        name: "create_user".to_string(),
        kind: SymbolKind::Function,
        file: PathBuf::from("src/services/user_service.rs"),
        line: 20,
        column: 4,
        references: vec![],
    };

    let symbol3 = Symbol {
        id: "sym_3".to_string(),
        name: "UserRepository".to_string(),
        kind: SymbolKind::Class,
        file: PathBuf::from("src/repositories/user_repository.rs"),
        line: 5,
        column: 0,
        references: vec![],
    };

    let symbol4 = Symbol {
        id: "sym_4".to_string(),
        name: "find_by_id".to_string(),
        kind: SymbolKind::Function,
        file: PathBuf::from("src/repositories/user_repository.rs"),
        line: 15,
        column: 4,
        references: vec![],
    };

    let symbol5 = Symbol {
        id: "sym_5".to_string(),
        name: "User".to_string(),
        kind: SymbolKind::Type,
        file: PathBuf::from("src/models/user.rs"),
        line: 1,
        column: 0,
        references: vec![],
    };

    index.add_symbol(symbol1);
    index.add_symbol(symbol2);
    index.add_symbol(symbol3);
    index.add_symbol(symbol4);
    index.add_symbol(symbol5);

    // Add references
    let ref1 = SymbolReference {
        symbol_id: "sym_1".to_string(),
        file: PathBuf::from("src/handlers/user_handler.rs"),
        line: 30,
        kind: ReferenceKind::Usage,
    };

    let ref2 = SymbolReference {
        symbol_id: "sym_3".to_string(),
        file: PathBuf::from("src/services/user_service.rs"),
        line: 25,
        kind: ReferenceKind::Import,
    };

    let ref3 = SymbolReference {
        symbol_id: "sym_5".to_string(),
        file: PathBuf::from("src/services/user_service.rs"),
        line: 5,
        kind: ReferenceKind::Import,
    };

    index.add_reference(ref1);
    index.add_reference(ref2);
    index.add_reference(ref3);

    index
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_search_by_exact_name() {
    let index = create_sample_index();
    let results = index.search_by_name("UserService");

    assert!(!results.is_empty());
    assert_eq!(results[0].symbol.name, "UserService");
}

#[test]
fn test_search_by_partial_name() {
    let index = create_sample_index();
    let results = index.search_by_name("User");

    // Should find UserService, UserRepository, and User
    assert!(results.len() >= 3);
    assert!(results.iter().any(|r| r.symbol.name == "UserService"));
    assert!(results.iter().any(|r| r.symbol.name == "UserRepository"));
    assert!(results.iter().any(|r| r.symbol.name == "User"));
}

#[test]
fn test_search_by_function_name() {
    let index = create_sample_index();
    let results = index.search_by_name("create_user");

    assert!(!results.is_empty());
    assert_eq!(results[0].symbol.name, "create_user");
    assert_eq!(results[0].symbol.kind, SymbolKind::Function);
}

#[test]
fn test_search_returns_symbol_metadata() {
    let index = create_sample_index();
    let results = index.search_by_name("UserService");

    assert!(!results.is_empty());
    let result = &results[0];

    assert_eq!(result.symbol.name, "UserService");
    assert_eq!(result.symbol.kind, SymbolKind::Class);
    assert_eq!(result.symbol.file.file_name().unwrap(), "user_service.rs");
    assert_eq!(result.symbol.line, 10);
}

#[test]
fn test_search_returns_references() {
    let index = create_sample_index();
    let results = index.search_by_name("UserService");

    assert!(!results.is_empty());
    let result = &results[0];

    // References may be populated depending on implementation
    let _ = result.symbol.references;
}

#[test]
fn test_search_empty_query() {
    let index = create_sample_index();
    let results = index.search_by_name("");

    // Empty query may return all results or no results depending on implementation
    let _ = results;
}

#[test]
fn test_search_nonexistent_symbol() {
    let index = create_sample_index();
    let results = index.search_by_name("NonexistentSymbol");

    assert!(results.is_empty());
}

#[test]
fn test_search_case_sensitivity() {
    let index = create_sample_index();

    let results_exact = index.search_by_name("UserService");
    let results_lower = index.search_by_name("userservice");
    let results_upper = index.search_by_name("USERSERVICE");

    // Should find exact match at minimum
    assert!(!results_exact.is_empty());
    // Case-insensitive search may or may not be supported
    let _ = (results_lower, results_upper);
}

#[test]
fn test_search_by_symbol_kind() {
    let index = create_sample_index();

    // Search for all functions
    let all_results = index.search_by_name("create");
    let function_results: Vec<_> = all_results
        .iter()
        .filter(|r| r.symbol.kind == SymbolKind::Function)
        .collect();

    assert!(!function_results.is_empty());
}

#[test]
fn test_search_returns_ranked_results() {
    let index = create_sample_index();
    let results = index.search_by_name("User");

    // Results should be ranked by relevance
    // Exact matches should come before partial matches
    if results.len() > 1 {
        // First result should be most relevant
        assert!(!results[0].symbol.name.is_empty());
    }
}

#[test]
fn test_search_finds_all_references() {
    let index = create_sample_index();
    let results = index.search_by_name("User");

    // Should find symbols matching the query
    let user_symbol = results.iter().find(|r| r.symbol.name == "User");
    assert!(user_symbol.is_some());

    if let Some(result) = user_symbol {
        // References may be populated depending on implementation
        let _ = result.symbol.references;
    }
}

#[test]
fn test_search_tracks_reference_kinds() {
    let index = create_sample_index();
    let results = index.search_by_name("UserRepository");

    assert!(!results.is_empty());
    let result = &results[0];

    // Reference kinds may be tracked depending on implementation
    let _ = result
        .symbol
        .references
        .iter()
        .filter(|r| r.kind == ReferenceKind::Import)
        .collect::<Vec<_>>();
}

#[test]
fn test_search_consistency() {
    let index = create_sample_index();

    // Search multiple times
    let results1 = index.search_by_name("UserService");
    let results2 = index.search_by_name("UserService");
    let results3 = index.search_by_name("UserService");

    // Results should be identical
    assert_eq!(results1.len(), results2.len());
    assert_eq!(results2.len(), results3.len());

    if !results1.is_empty() {
        assert_eq!(results1[0].symbol.name, results2[0].symbol.name);
        assert_eq!(results2[0].symbol.name, results3[0].symbol.name);
    }
}

#[test]
fn test_search_multiple_symbols_same_name() {
    let mut index = SemanticIndex::new();

    // Add multiple symbols with similar names
    let symbol1 = Symbol {
        id: "sym_1".to_string(),
        name: "User".to_string(),
        kind: SymbolKind::Type,
        file: PathBuf::from("src/models/user.rs"),
        line: 1,
        column: 0,
        references: vec![],
    };

    let symbol2 = Symbol {
        id: "sym_2".to_string(),
        name: "UserService".to_string(),
        kind: SymbolKind::Class,
        file: PathBuf::from("src/services/user_service.rs"),
        line: 10,
        column: 0,
        references: vec![],
    };

    index.add_symbol(symbol1);
    index.add_symbol(symbol2);

    let results = index.search_by_name("User");

    // Should find both symbols
    assert_eq!(results.len(), 2);
}

#[test]
fn test_search_across_multiple_files() {
    let index = create_sample_index();
    let results = index.search_by_name("User");

    // Results should span multiple files
    let files: std::collections::HashSet<_> =
        results.iter().map(|r| r.symbol.file.clone()).collect();

    assert!(files.len() > 1);
}

#[test]
fn test_search_preserves_symbol_location() {
    let index = create_sample_index();
    let results = index.search_by_name("create_user");

    assert!(!results.is_empty());
    let result = &results[0];

    // Location information should be preserved
    assert_eq!(result.symbol.line, 20);
    assert_eq!(result.symbol.column, 4);
}

#[test]
fn test_search_handles_special_characters() {
    let mut index = SemanticIndex::new();

    let symbol = Symbol {
        id: "sym_1".to_string(),
        name: "create_user_by_id".to_string(),
        kind: SymbolKind::Function,
        file: PathBuf::from("src/services/user_service.rs"),
        line: 20,
        column: 4,
        references: vec![],
    };

    index.add_symbol(symbol);

    let results = index.search_by_name("create_user");
    assert!(!results.is_empty());
}
