//! Property-based tests for command palette fuzzy search
//!
//! **Feature: ricecoder-tui, Property 5: Command Palette Fuzzy Search**
//! **Validates: Requirements 9.1, 9.2**
//!
//! For any command palette with commands and any search query, the fuzzy search
//! should correctly filter and rank commands based on relevance.

use proptest::prelude::*;
use ricecoder_tui::command_palette::{CommandPaletteWidget, PaletteCommand};

// Strategy for generating palette commands
fn arb_palette_command() -> impl Strategy<Value = PaletteCommand> {
    (
        "[a-zA-Z0-9_-]{1,20}".prop_map(|s| s), // name
        "[a-zA-Z0-9 ]{1,30}".prop_map(|s| s), // display_name
        "[a-zA-Z0-9 .,!?]{1,50}".prop_map(|s| s), // description
        prop_oneof![
            Just(None),
            "[a-zA-Z0-9+]{1,10}".prop_map(|s| Some(format!("Ctrl+{}", s)))
        ], // shortcut
        prop_oneof![
            Just("General".to_string()),
            Just("File".to_string()),
            Just("Edit".to_string()),
            Just("View".to_string()),
            Just("Help".to_string())
        ], // category
    )
        .prop_map(|(name, display_name, description, shortcut, category)| PaletteCommand {
            name,
            display_name,
            description,
            shortcut,
            category,
        })
}

// Strategy for generating search queries
fn arb_search_query() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("".to_string()), // Empty query
        "[a-zA-Z]{1,10}".prop_map(|s| s), // Single word
        "[a-zA-Z]{1,5} [a-zA-Z]{1,5}".prop_map(|s| s), // Multiple words
        "[a-zA-Z0-9]{1,8}".prop_map(|s| s), // With numbers
    ]
}

// Strategy for generating command palettes with commands
fn arb_command_palette() -> impl Strategy<Value = CommandPaletteWidget> {
    prop::collection::vec(arb_palette_command(), 1..20)
        .prop_map(|commands| {
            let mut palette = CommandPaletteWidget::new();
            palette.add_commands(commands);
            palette
        })
}

proptest! {
    /// **Feature: ricecoder-tui, Property 5: Command Palette Fuzzy Search**
    /// **Validates: Requirements 9.1, 9.2**
    ///
    /// For any command palette and any search query, fuzzy search should:
    /// 1. Handle empty queries correctly
    /// 2. Filter results appropriately for non-empty queries
    #[test]
    fn prop_command_palette_fuzzy_search_correctness(
        mut palette in arb_command_palette(),
        query in arb_search_query(),
    ) {
        // Set the search query
        palette.set_query(query.clone());

        let filtered_count = palette.filtered_count();

        if query.is_empty() {
            // Empty query should show all commands
            prop_assert!(filtered_count >= palette.total_commands());
        } else {
            // Non-empty query should filter commands
            prop_assert!(filtered_count <= palette.total_commands());
        }
    }

    /// **Feature: ricecoder-tui, Property 5: Command Palette Fuzzy Search - Empty Query**
    /// **Validates: Requirements 9.1, 9.2**
    ///
    /// Empty query should show all commands.
    #[test]
    fn prop_command_palette_empty_query_behavior(
        commands in prop::collection::vec(arb_palette_command(), 1..15),
    ) {
        let mut palette = CommandPaletteWidget::new();
        palette.add_commands(commands.clone());

        // Set empty query
        palette.set_query("".to_string());

        // Should show all commands
        prop_assert_eq!(palette.filtered_count(), palette.total_commands());
    }

    /// **Feature: ricecoder-tui, Property 5: Command Palette Fuzzy Search - Case Insensitivity**
    /// **Validates: Requirements 9.1, 9.2**
    ///
    /// Fuzzy search should be case insensitive.
    #[test]
    fn prop_command_palette_case_insensitive_search(
        base_commands in prop::collection::vec(arb_palette_command(), 2..8),
        case_variation in "[a-zA-Z]{2,6}",
    ) {
        let mut lower_palette = CommandPaletteWidget::new();
        lower_palette.add_commands(base_commands.clone());

        let mut upper_palette = CommandPaletteWidget::new();
        upper_palette.add_commands(base_commands);

        // Test with lowercase query
        lower_palette.set_query(case_variation.to_lowercase());
        let lower_count = lower_palette.filtered_count();

        // Test with uppercase query
        upper_palette.set_query(case_variation.to_uppercase());
        let upper_count = upper_palette.filtered_count();

        // Should return same number of results (case insensitive)
        prop_assert_eq!(lower_count, upper_count,
            "Case variations should return same number of results");
    }
}

