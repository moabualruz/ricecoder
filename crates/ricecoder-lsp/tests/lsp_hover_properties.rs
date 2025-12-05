//! Property-based tests for LSP hover information accuracy
//!
//! **Feature: ricecoder-lsp, Property 5: Hover information accuracy**
//! **Validates: Requirements LSP-4.1, LSP-4.2**

use proptest::prelude::*;
use ricecoder_lsp::types::{Definition, MarkupKind, Position, Range, Symbol, SymbolKind};
use ricecoder_lsp::HoverProvider;

/// Strategy for generating valid positions
fn position_strategy() -> impl Strategy<Value = Position> {
    (0u32..100, 0u32..100).prop_map(|(line, character)| Position::new(line, character))
}

/// Strategy for generating valid ranges
fn range_strategy() -> impl Strategy<Value = Range> {
    (0u32..50, 0u32..50, 50u32..100, 0u32..50).prop_map(
        |(start_line, start_char, end_line, end_char)| {
            Range::new(
                Position::new(start_line, start_char),
                Position::new(end_line, end_char),
            )
        },
    )
}

/// Strategy for generating symbol kinds
fn symbol_kind_strategy() -> impl Strategy<Value = SymbolKind> {
    prop_oneof![
        Just(SymbolKind::Function),
        Just(SymbolKind::Variable),
        Just(SymbolKind::Type),
        Just(SymbolKind::Class),
        Just(SymbolKind::Method),
        Just(SymbolKind::Property),
    ]
}

/// Strategy for generating symbol names
fn symbol_name_strategy() -> impl Strategy<Value = String> {
    "[a-z_][a-z0-9_]{0,20}".prop_map(|s| s.to_string())
}

/// Strategy for generating symbols
fn symbol_strategy() -> impl Strategy<Value = Symbol> {
    (
        symbol_name_strategy(),
        symbol_kind_strategy(),
        range_strategy(),
    )
        .prop_map(|(name, kind, range)| Symbol {
            name,
            kind,
            range,
            definition: None,
            references: vec![],
            documentation: None,
        })
}

/// Strategy for generating symbols with documentation
fn documented_symbol_strategy() -> impl Strategy<Value = Symbol> {
    (
        symbol_name_strategy(),
        symbol_kind_strategy(),
        range_strategy(),
        ".*",
    )
        .prop_map(|(name, kind, range, doc)| Symbol {
            name,
            kind,
            range,
            definition: None,
            references: vec![],
            documentation: Some(doc),
        })
}

/// Strategy for generating symbols with definition locations
fn symbol_with_definition_strategy() -> impl Strategy<Value = Symbol> {
    (
        symbol_name_strategy(),
        symbol_kind_strategy(),
        range_strategy(),
        range_strategy(),
    )
        .prop_map(|(name, kind, range, def_range)| Symbol {
            name,
            kind,
            range,
            definition: Some(Definition {
                uri: "file://test.rs".to_string(),
                range: def_range,
            }),
            references: vec![],
            documentation: None,
        })
}

proptest! {
    /// Property 5.1: Hover information is accurate for all symbol types
    ///
    /// For any symbol, hover information should include the symbol name and kind.
    /// **Validates: Requirements LSP-4.1, LSP-4.2**
    #[test]
    fn prop_hover_info_includes_symbol_name_and_kind(
        symbol in symbol_strategy().prop_map(|mut s| {
            // Ensure symbol is on line 0 for simplicity
            s.range = Range::new(Position::new(0, 0), Position::new(0, 10));
            s
        })
    ) {
        let mut provider = HoverProvider::new();
        provider.index_symbols(vec![symbol.clone()]);

        // Create code that matches the symbol range
        let code = "test_code_here";
        let hover = provider.get_hover_info(code, symbol.range.start);

        // Hover should be present for symbol at its start position
        prop_assert!(hover.is_some(), "Hover info should be present for symbol");

        let hover_info = hover.unwrap();
        // Hover content should contain symbol name
        prop_assert!(
            hover_info.contents.value.contains(&symbol.name),
            "Hover content should contain symbol name"
        );
        // Hover content should contain symbol kind
        prop_assert!(
            hover_info.contents.value.contains(&format!("{:?}", symbol.kind)),
            "Hover content should contain symbol kind"
        );
    }

    /// Property 5.2: Type information is correct for variables and functions
    ///
    /// For any variable or function symbol, hover information should include type information.
    /// **Validates: Requirements LSP-4.1**
    #[test]
    fn prop_hover_includes_type_info(
        symbol in prop_oneof![
            symbol_strategy().prop_map(|mut s| {
                s.kind = SymbolKind::Variable;
                s
            }),
            symbol_strategy().prop_map(|mut s| {
                s.kind = SymbolKind::Function;
                s
            }),
        ]
    ) {
        let mut provider = HoverProvider::new();
        provider.index_symbols(vec![symbol.clone()]);

        let type_info = provider.get_type_info(&symbol.name);
        prop_assert!(type_info.is_some(), "Type info should be available");
        prop_assert!(
            type_info.unwrap().contains(&format!("{:?}", symbol.kind)),
            "Type info should contain symbol kind"
        );
    }

    /// Property 5.3: Definition location points to correct file and line
    ///
    /// For any symbol with a definition, hover information should include the definition location.
    /// **Validates: Requirements LSP-4.2**
    #[test]
    fn prop_hover_includes_definition_location(symbol in symbol_with_definition_strategy()) {
        let mut provider = HoverProvider::new();
        provider.index_symbols(vec![symbol.clone()]);

        let location = provider.get_definition_location(&symbol.name);
        prop_assert!(location.is_some(), "Definition location should be available");

        let (uri, line, character) = location.unwrap();
        let def = symbol.definition.unwrap();
        prop_assert_eq!(uri, def.uri, "URI should match definition");
        prop_assert_eq!(line, def.range.start.line, "Line should match definition");
        prop_assert_eq!(character, def.range.start.character, "Character should match definition");
    }

    /// Property 5.4: Documentation is retrieved when available
    ///
    /// For any symbol with documentation, hover information should include the documentation.
    /// **Validates: Requirements LSP-4.2**
    #[test]
    fn prop_hover_includes_documentation(symbol in documented_symbol_strategy()) {
        let mut provider = HoverProvider::new();
        provider.index_symbols(vec![symbol.clone()]);

        let doc = provider.get_documentation(&symbol.name);
        prop_assert!(doc.is_some(), "Documentation should be available");
        prop_assert_eq!(
            doc.unwrap(),
            symbol.documentation.unwrap(),
            "Documentation should match"
        );
    }

    /// Property 5.5: Hover information is consistent across multiple lookups
    ///
    /// For any symbol, looking up hover information multiple times should produce identical results.
    /// **Validates: Requirements LSP-4.1, LSP-4.2**
    #[test]
    fn prop_hover_info_is_consistent(symbol in symbol_strategy()) {
        let mut provider = HoverProvider::new();
        provider.index_symbols(vec![symbol.clone()]);

        let code = "test_code";
        let hover1 = provider.get_hover_info(code, symbol.range.start);
        let hover2 = provider.get_hover_info(code, symbol.range.start);

        prop_assert_eq!(
            hover1.as_ref().map(|h| &h.contents.value),
            hover2.as_ref().map(|h| &h.contents.value),
            "Hover info should be consistent across multiple lookups"
        );
    }

    /// Property 5.6: Hover range is correctly set
    ///
    /// For any symbol, hover information should include the correct range.
    /// **Validates: Requirements LSP-4.1**
    #[test]
    fn prop_hover_range_is_correct(
        symbol in symbol_strategy().prop_map(|mut s| {
            // Ensure symbol is on line 0 for simplicity
            s.range = Range::new(Position::new(0, 0), Position::new(0, 10));
            s
        })
    ) {
        let mut provider = HoverProvider::new();
        provider.index_symbols(vec![symbol.clone()]);

        let code = "test_code_here";
        let hover = provider.get_hover_info(code, symbol.range.start);

        prop_assert!(hover.is_some(), "Hover info should be present");
        let hover_info = hover.unwrap();
        prop_assert!(hover_info.range.is_some(), "Hover range should be set");
        prop_assert_eq!(
            hover_info.range.unwrap(),
            symbol.range,
            "Hover range should match symbol range"
        );
    }

    /// Property 5.7: Hover information uses correct markup kind
    ///
    /// For any symbol, hover information should use markdown markup kind.
    /// **Validates: Requirements LSP-4.1**
    #[test]
    fn prop_hover_uses_markdown_markup(
        symbol in symbol_strategy().prop_map(|mut s| {
            // Ensure symbol is on line 0 for simplicity
            s.range = Range::new(Position::new(0, 0), Position::new(0, 10));
            s
        })
    ) {
        let mut provider = HoverProvider::new();
        provider.index_symbols(vec![symbol.clone()]);

        let code = "test_code_here";
        let hover = provider.get_hover_info(code, symbol.range.start);

        prop_assert!(hover.is_some(), "Hover info should be present");
        let hover_info = hover.unwrap();
        prop_assert_eq!(
            hover_info.contents.kind,
            MarkupKind::Markdown,
            "Hover should use markdown markup"
        );
    }

    /// Property 5.8: Usage count is accurate
    ///
    /// For any symbol, the usage count should match the number of references.
    /// **Validates: Requirements LSP-4.2**
    #[test]
    fn prop_usage_count_is_accurate(
        symbol in symbol_strategy(),
        ref_count in 0usize..10
    ) {
        let mut provider = HoverProvider::new();
        let mut symbol_with_refs = symbol.clone();
        symbol_with_refs.references = (0..ref_count)
            .map(|i| ricecoder_lsp::types::Reference {
                uri: format!("file://test{}.rs", i),
                range: Range::new(
                    Position::new(i as u32, 0),
                    Position::new(i as u32, 10),
                ),
            })
            .collect();

        provider.index_symbols(vec![symbol_with_refs.clone()]);

        let count = provider.get_usage_count(&symbol.name);
        prop_assert_eq!(count, ref_count, "Usage count should match reference count");
    }

    /// Property 5.9: No hover info for non-existent symbols
    ///
    /// For any position without a symbol, hover information should be None.
    /// **Validates: Requirements LSP-4.1**
    #[test]
    fn prop_no_hover_for_empty_position(position in position_strategy()) {
        let provider = HoverProvider::new();
        let code = "test_code";

        let hover = provider.get_hover_info(code, position);
        // Hover should be None when no symbols are indexed
        prop_assert!(hover.is_none(), "Hover should be None for empty provider");
    }

    /// Property 5.10: Hover information is deterministic
    ///
    /// For any symbol and position, hover information should be deterministic.
    /// **Validates: Requirements LSP-4.1, LSP-4.2**
    #[test]
    fn prop_hover_is_deterministic(
        symbols in prop::collection::vec(symbol_strategy(), 1..5),
        position in position_strategy()
    ) {
        let mut provider1 = HoverProvider::new();
        provider1.index_symbols(symbols.clone());

        let mut provider2 = HoverProvider::new();
        provider2.index_symbols(symbols);

        let code = "test_code";
        let hover1 = provider1.get_hover_info(code, position);
        let hover2 = provider2.get_hover_info(code, position);

        prop_assert_eq!(
            hover1.as_ref().map(|h| &h.contents.value),
            hover2.as_ref().map(|h| &h.contents.value),
            "Hover information should be deterministic"
        );
    }
}
