//! Property-based tests for semantic index completeness and cross-file relationship tracking
//! **Feature: ricecoder-research, Property 4: Semantic Index Completeness**
//! **Validates: Requirements 1.7, 1.8**
//! **Feature: ricecoder-research, Property 5: Cross-File Relationship Tracking**
//! **Validates: Requirements 1.9**

use proptest::prelude::*;
use ricecoder_research::{SemanticIndex, Symbol, SymbolKind, SymbolReference, ReferenceKind};
use std::path::PathBuf;

// ============================================================================
// Generators for property testing
// ============================================================================

/// Generate a symbol with a given ID and name
fn create_symbol(id: &str, name: &str, kind: SymbolKind, file: &str) -> Symbol {
    Symbol {
        id: id.to_string(),
        name: name.to_string(),
        kind,
        file: PathBuf::from(file),
        line: 1,
        column: 1,
        references: Vec::new(),
    }
}

/// Generate a symbol reference
fn create_reference(symbol_id: &str, file: &str, line: usize, kind: ReferenceKind) -> SymbolReference {
    SymbolReference {
        symbol_id: symbol_id.to_string(),
        file: PathBuf::from(file),
        line,
        kind,
    }
}

/// Strategy to generate valid symbol IDs
fn symbol_id_strategy() -> impl Strategy<Value = String> {
    "sym[0-9]{1,3}".prop_map(|s| s.to_string())
}

/// Strategy to generate valid symbol names
fn symbol_name_strategy() -> impl Strategy<Value = String> {
    "[a-z_][a-z0-9_]{0,20}".prop_map(|s| s.to_string())
}

/// Strategy to generate symbol kinds
fn symbol_kind_strategy() -> impl Strategy<Value = SymbolKind> {
    prop_oneof![
        Just(SymbolKind::Function),
        Just(SymbolKind::Class),
        Just(SymbolKind::Type),
        Just(SymbolKind::Constant),
        Just(SymbolKind::Variable),
        Just(SymbolKind::Module),
        Just(SymbolKind::Trait),
        Just(SymbolKind::Enum),
    ]
}

/// Strategy to generate file paths
fn file_path_strategy() -> impl Strategy<Value = String> {
    "[a-z]{1,10}\\.rs"
}

// ============================================================================
// Property Tests
// ============================================================================

proptest! {
    /// Property 4: Semantic Index Completeness
    /// For any set of symbols added to the index, all symbols should be retrievable
    #[test]
    fn prop_all_added_symbols_are_indexed(
        symbol_ids in prop::collection::vec(symbol_id_strategy(), 1..20),
        symbol_names in prop::collection::vec(symbol_name_strategy(), 1..20),
        kinds in prop::collection::vec(symbol_kind_strategy(), 1..20),
        files in prop::collection::vec(file_path_strategy(), 1..20),
    ) {
        let mut index = SemanticIndex::new();
        let num_symbols = symbol_ids.len().min(symbol_names.len()).min(kinds.len()).min(files.len());

        // Create unique symbol IDs to avoid overwrites
        let unique_ids: Vec<String> = (0..num_symbols)
            .map(|i| format!("sym_{}", i))
            .collect();

        // Add symbols to index
        for i in 0..num_symbols {
            let symbol = create_symbol(
                &unique_ids[i],
                &symbol_names[i],
                kinds[i % kinds.len()],
                &files[i % files.len()],
            );
            index.add_symbol(symbol);
        }

        // Verify all symbols are indexed
        prop_assert_eq!(index.symbol_count(), num_symbols);

        // Verify each symbol can be retrieved by ID
        for i in 0..num_symbols {
            let retrieved = index.get_symbol(&unique_ids[i]);
            prop_assert!(retrieved.is_some());
            prop_assert_eq!(&retrieved.unwrap().id, &unique_ids[i]);
        }
    }

    /// Property: Symbols are retrievable by name
    /// For any symbol added to the index, it should be retrievable by its name
    #[test]
    fn prop_symbols_retrievable_by_name(
        symbol_id in symbol_id_strategy(),
        symbol_name in symbol_name_strategy(),
        kind in symbol_kind_strategy(),
        file in file_path_strategy(),
    ) {
        let mut index = SemanticIndex::new();
        let symbol = create_symbol(&symbol_id, &symbol_name, kind, &file);

        index.add_symbol(symbol.clone());

        let retrieved = index.get_symbols_by_name(&symbol_name);
        prop_assert_eq!(retrieved.len(), 1);
        prop_assert_eq!(&retrieved[0].id, &symbol_id);
    }

    /// Property: Symbols are retrievable by file
    /// For any symbol added to the index, it should be retrievable by its file
    #[test]
    fn prop_symbols_retrievable_by_file(
        symbol_id in symbol_id_strategy(),
        symbol_name in symbol_name_strategy(),
        kind in symbol_kind_strategy(),
        file in file_path_strategy(),
    ) {
        let mut index = SemanticIndex::new();
        let symbol = create_symbol(&symbol_id, &symbol_name, kind, &file);
        let file_path = PathBuf::from(&file);

        index.add_symbol(symbol.clone());

        let retrieved = index.get_symbols_in_file(&file_path);
        prop_assert_eq!(retrieved.len(), 1);
        prop_assert_eq!(&retrieved[0].id, &symbol_id);
    }

    /// Property: Symbols are retrievable by kind
    /// For any symbol of a given kind added to the index, it should be retrievable by kind
    #[test]
    fn prop_symbols_retrievable_by_kind(
        symbol_id in symbol_id_strategy(),
        symbol_name in symbol_name_strategy(),
        kind in symbol_kind_strategy(),
        file in file_path_strategy(),
    ) {
        let mut index = SemanticIndex::new();
        let symbol = create_symbol(&symbol_id, &symbol_name, kind, &file);

        index.add_symbol(symbol.clone());

        let retrieved = index.search_by_kind(kind);
        prop_assert!(retrieved.iter().any(|s| s.id == symbol_id));
    }

    /// Property 5: Cross-File Relationship Tracking
    /// For any reference added to the index, it should be retrievable and track the relationship
    #[test]
    fn prop_all_references_are_tracked(
        symbol_ids in prop::collection::vec(symbol_id_strategy(), 1..10),
        symbol_names in prop::collection::vec(symbol_name_strategy(), 1..10),
        kinds in prop::collection::vec(symbol_kind_strategy(), 1..10),
        files in prop::collection::vec(file_path_strategy(), 1..10),
        ref_files in prop::collection::vec(file_path_strategy(), 1..10),
        ref_lines in prop::collection::vec(1usize..100, 1..10),
    ) {
        let mut index = SemanticIndex::new();
        let num_symbols = symbol_ids.len().min(symbol_names.len()).min(kinds.len()).min(files.len());

        // Add symbols
        for i in 0..num_symbols {
            let symbol = create_symbol(
                &symbol_ids[i],
                &symbol_names[i],
                kinds[i % kinds.len()],
                &files[i % files.len()],
            );
            index.add_symbol(symbol);
        }

        // Add references
        let num_refs = ref_files.len().min(ref_lines.len());
        for i in 0..num_refs {
            let reference = create_reference(
                &symbol_ids[i % num_symbols],
                &ref_files[i],
                ref_lines[i],
                ReferenceKind::Usage,
            );
            index.add_reference(reference);
        }

        // Verify all references are tracked
        prop_assert_eq!(index.reference_count(), num_refs);

        // Verify references can be retrieved
        for i in 0..num_refs {
            let refs = index.get_references_to_symbol(&symbol_ids[i % num_symbols]);
            prop_assert!(!refs.is_empty());
        }
    }

    /// Property: References track cross-file relationships
    /// For any reference, it should track which file references which symbol
    #[test]
    fn prop_references_track_cross_file_relationships(
        symbol_id in symbol_id_strategy(),
        symbol_name in symbol_name_strategy(),
        kind in symbol_kind_strategy(),
        def_file in file_path_strategy(),
        ref_file in file_path_strategy(),
        ref_line in 1usize..100,
    ) {
        let mut index = SemanticIndex::new();
        let symbol = create_symbol(&symbol_id, &symbol_name, kind, &def_file);
        index.add_symbol(symbol);

        let reference = create_reference(&symbol_id, &ref_file, ref_line, ReferenceKind::Usage);
        index.add_reference(reference);

        let refs = index.get_references_to_symbol(&symbol_id);
        prop_assert_eq!(refs.len(), 1);
        prop_assert_eq!(&refs[0].file, &PathBuf::from(&ref_file));
        prop_assert_eq!(refs[0].line, ref_line);
    }

    /// Property: Search results are ordered by relevance
    /// For any search query, results should be ordered by relevance (highest first)
    #[test]
    fn prop_search_results_ordered_by_relevance(
        symbol_id1 in symbol_id_strategy(),
        symbol_id2 in symbol_id_strategy(),
        symbol_id3 in symbol_id_strategy(),
        query in "[a-z]{1,10}",
    ) {
        let mut index = SemanticIndex::new();

        // Add symbols with different name matches
        let symbol1 = create_symbol(&symbol_id1, &query, SymbolKind::Function, "file1.rs");
        let symbol2 = create_symbol(&symbol_id2, &format!("{}_suffix", query), SymbolKind::Function, "file2.rs");
        let symbol3 = create_symbol(&symbol_id3, &format!("prefix_{}", query), SymbolKind::Function, "file3.rs");

        index.add_symbol(symbol1);
        index.add_symbol(symbol2);
        index.add_symbol(symbol3);

        let results = index.search_by_name(&query);

        // Verify results are ordered by relevance
        prop_assert!(results.len() >= 1);
        for i in 0..results.len() - 1 {
            prop_assert!(results[i].relevance >= results[i + 1].relevance);
        }
    }

    /// Property: Index maintains consistency after operations
    /// For any sequence of add/remove operations, the index should remain consistent
    #[test]
    fn prop_index_consistency_after_operations(
        symbol_ids in prop::collection::vec(symbol_id_strategy(), 1..10),
        symbol_names in prop::collection::vec(symbol_name_strategy(), 1..10),
        kinds in prop::collection::vec(symbol_kind_strategy(), 1..10),
        files in prop::collection::vec(file_path_strategy(), 1..10),
    ) {
        let mut index = SemanticIndex::new();
        let num_symbols = symbol_ids.len().min(symbol_names.len()).min(kinds.len()).min(files.len());

        // Add symbols
        for i in 0..num_symbols {
            let symbol = create_symbol(
                &symbol_ids[i],
                &symbol_names[i],
                kinds[i % kinds.len()],
                &files[i % files.len()],
            );
            index.add_symbol(symbol);
        }

        let count_before = index.symbol_count();

        // Clear and verify
        index.clear();
        prop_assert_eq!(index.symbol_count(), 0);

        // Re-add symbols
        for i in 0..num_symbols {
            let symbol = create_symbol(
                &symbol_ids[i],
                &symbol_names[i],
                kinds[i % kinds.len()],
                &files[i % files.len()],
            );
            index.add_symbol(symbol);
        }

        let count_after = index.symbol_count();
        prop_assert_eq!(count_before, count_after);
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[test]
fn test_semantic_index_add_and_retrieve_symbol() {
    let mut index = SemanticIndex::new();
    let symbol = create_symbol("sym1", "my_function", SymbolKind::Function, "main.rs");

    index.add_symbol(symbol.clone());

    assert_eq!(index.symbol_count(), 1);
    assert_eq!(index.get_symbol("sym1"), Some(&symbol));
}

#[test]
fn test_semantic_index_multiple_symbols_same_name() {
    let mut index = SemanticIndex::new();
    let symbol1 = create_symbol("sym1", "my_function", SymbolKind::Function, "file1.rs");
    let symbol2 = create_symbol("sym2", "my_function", SymbolKind::Function, "file2.rs");

    index.add_symbol(symbol1);
    index.add_symbol(symbol2);

    let results = index.get_symbols_by_name("my_function");
    assert_eq!(results.len(), 2);
}

#[test]
fn test_semantic_index_symbols_by_file() {
    let mut index = SemanticIndex::new();
    let symbol1 = create_symbol("sym1", "func1", SymbolKind::Function, "main.rs");
    let symbol2 = create_symbol("sym2", "func2", SymbolKind::Function, "main.rs");
    let symbol3 = create_symbol("sym3", "func3", SymbolKind::Function, "other.rs");

    index.add_symbol(symbol1);
    index.add_symbol(symbol2);
    index.add_symbol(symbol3);

    let main_symbols = index.get_symbols_in_file(&PathBuf::from("main.rs"));
    assert_eq!(main_symbols.len(), 2);

    let other_symbols = index.get_symbols_in_file(&PathBuf::from("other.rs"));
    assert_eq!(other_symbols.len(), 1);
}

#[test]
fn test_semantic_index_add_reference() {
    let mut index = SemanticIndex::new();
    let symbol = create_symbol("sym1", "my_function", SymbolKind::Function, "main.rs");
    index.add_symbol(symbol);

    let reference = create_reference("sym1", "caller.rs", 42, ReferenceKind::Usage);
    index.add_reference(reference);

    assert_eq!(index.reference_count(), 1);
    let refs = index.get_references_to_symbol("sym1");
    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0].line, 42);
}

#[test]
fn test_semantic_index_multiple_references_to_symbol() {
    let mut index = SemanticIndex::new();
    let symbol = create_symbol("sym1", "my_function", SymbolKind::Function, "main.rs");
    index.add_symbol(symbol);

    let ref1 = create_reference("sym1", "caller1.rs", 10, ReferenceKind::Usage);
    let ref2 = create_reference("sym1", "caller2.rs", 20, ReferenceKind::Usage);
    let ref3 = create_reference("sym1", "caller3.rs", 30, ReferenceKind::Import);

    index.add_reference(ref1);
    index.add_reference(ref2);
    index.add_reference(ref3);

    assert_eq!(index.reference_count(), 3);
    let refs = index.get_references_to_symbol("sym1");
    assert_eq!(refs.len(), 3);
}

#[test]
fn test_semantic_index_search_by_kind() {
    let mut index = SemanticIndex::new();
    let func = create_symbol("sym1", "my_function", SymbolKind::Function, "main.rs");
    let class = create_symbol("sym2", "MyClass", SymbolKind::Class, "main.rs");
    let constant = create_symbol("sym3", "MY_CONST", SymbolKind::Constant, "main.rs");

    index.add_symbol(func);
    index.add_symbol(class);
    index.add_symbol(constant);

    let functions = index.search_by_kind(SymbolKind::Function);
    assert_eq!(functions.len(), 1);

    let classes = index.search_by_kind(SymbolKind::Class);
    assert_eq!(classes.len(), 1);

    let constants = index.search_by_kind(SymbolKind::Constant);
    assert_eq!(constants.len(), 1);
}

#[test]
fn test_semantic_index_search_by_name_exact() {
    let mut index = SemanticIndex::new();
    let symbol = create_symbol("sym1", "my_function", SymbolKind::Function, "main.rs");
    index.add_symbol(symbol);

    let results = index.search_by_name("my_function");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].relevance, 1.0);
}

#[test]
fn test_semantic_index_search_by_name_prefix() {
    let mut index = SemanticIndex::new();
    let symbol = create_symbol("sym1", "my_function", SymbolKind::Function, "main.rs");
    index.add_symbol(symbol);

    let results = index.search_by_name("my_");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].relevance, 0.8);
}

#[test]
fn test_semantic_index_search_by_name_substring() {
    let mut index = SemanticIndex::new();
    let symbol = create_symbol("sym1", "my_function", SymbolKind::Function, "main.rs");
    index.add_symbol(symbol);

    let results = index.search_by_name("function");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].relevance, 0.5);
}

#[test]
fn test_semantic_index_all_symbols() {
    let mut index = SemanticIndex::new();
    let symbol1 = create_symbol("sym1", "func1", SymbolKind::Function, "main.rs");
    let symbol2 = create_symbol("sym2", "func2", SymbolKind::Function, "main.rs");

    index.add_symbol(symbol1);
    index.add_symbol(symbol2);

    let all = index.all_symbols();
    assert_eq!(all.len(), 2);
}

#[test]
fn test_semantic_index_clear() {
    let mut index = SemanticIndex::new();
    let symbol = create_symbol("sym1", "my_function", SymbolKind::Function, "main.rs");
    index.add_symbol(symbol);

    assert_eq!(index.symbol_count(), 1);

    index.clear();

    assert_eq!(index.symbol_count(), 0);
    assert_eq!(index.reference_count(), 0);
}

#[test]
fn test_semantic_index_cross_file_references() {
    let mut index = SemanticIndex::new();

    // Define symbol in file1.rs
    let symbol = create_symbol("sym1", "my_function", SymbolKind::Function, "file1.rs");
    index.add_symbol(symbol);

    // Reference from file2.rs
    let ref1 = create_reference("sym1", "file2.rs", 10, ReferenceKind::Usage);
    // Reference from file3.rs
    let ref2 = create_reference("sym1", "file3.rs", 20, ReferenceKind::Usage);

    index.add_reference(ref1);
    index.add_reference(ref2);

    let refs = index.get_references_to_symbol("sym1");
    assert_eq!(refs.len(), 2);

    // Verify cross-file relationships
    let files: Vec<_> = refs.iter().map(|r| r.file.clone()).collect();
    assert!(files.contains(&PathBuf::from("file2.rs")));
    assert!(files.contains(&PathBuf::from("file3.rs")));
}
