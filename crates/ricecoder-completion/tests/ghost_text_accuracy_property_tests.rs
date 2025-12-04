/// Property-based tests for ghost text accuracy
///
/// **Feature: ricecoder-completion, Property 2: Ghost text accuracy**
/// **Validates: Requirements Completion-2.1, Completion-2.2**
///
/// Property: For any completion, ghost text accurately represents what will be inserted
/// - Generate random completions and verify ghost text matches insertion
/// - Run 100+ iterations with various completion types

use proptest::prelude::*;
use ricecoder_completion::{
    BasicGhostTextGenerator, CompletionItem, CompletionItemKind, GhostTextGenerator, Position,
};

/// Strategy for generating valid completion items
fn completion_item_strategy() -> impl Strategy<Value = CompletionItem> {
    (
        "[a-zA-Z_][a-zA-Z0-9_]*",
        prop_oneof![
            Just(CompletionItemKind::Text),
            Just(CompletionItemKind::Function),
            Just(CompletionItemKind::Variable),
            Just(CompletionItemKind::Keyword),
            Just(CompletionItemKind::Snippet),
            Just(CompletionItemKind::Method),
            Just(CompletionItemKind::Class),
        ],
        ".*",
    )
        .prop_map(|(label, kind, insert_text)| {
            CompletionItem::new(label, kind, insert_text)
        })
}

/// Strategy for generating valid positions
fn position_strategy() -> impl Strategy<Value = Position> {
    (0u32..100, 0u32..100).prop_map(|(line, character)| Position::new(line, character))
}

proptest! {
    /// Property: Ghost text text field matches completion insert_text
    ///
    /// For any completion item, the ghost text's text field should exactly match
    /// the completion's insert_text field.
    #[test]
    fn prop_ghost_text_matches_insert_text(
        completion in completion_item_strategy(),
        position in position_strategy(),
    ) {
        let generator = BasicGhostTextGenerator::new();
        let ghost_text = generator.generate_ghost_text(&completion, position);

        // Ghost text should match the insert_text exactly
        prop_assert_eq!(ghost_text.text, completion.insert_text);
    }

    /// Property: Ghost text range start matches position
    ///
    /// For any completion and position, the ghost text's range start should
    /// exactly match the provided position.
    #[test]
    fn prop_ghost_text_range_start_matches_position(
        completion in completion_item_strategy(),
        position in position_strategy(),
    ) {
        let generator = BasicGhostTextGenerator::new();
        let ghost_text = generator.generate_ghost_text(&completion, position);

        // Ghost text range start should match the position
        prop_assert_eq!(ghost_text.range.start, position);
    }

    /// Property: Ghost text range end is consistent with text length
    ///
    /// For any single-line completion, the ghost text's range end character
    /// should be the start character plus the text length.
    #[test]
    fn prop_ghost_text_range_end_single_line(
        completion in completion_item_strategy(),
        position in position_strategy(),
    ) {
        // Filter out multi-line completions
        if completion.insert_text.contains('\n') {
            return Ok(());
        }

        let generator = BasicGhostTextGenerator::new();
        let ghost_text = generator.generate_ghost_text(&completion, position);

        // For single-line text, end character should be start + text length
        let expected_end_char = position.character + completion.insert_text.len() as u32;
        prop_assert_eq!(ghost_text.range.end.character, expected_end_char);
        prop_assert_eq!(ghost_text.range.end.line, position.line);
    }

    /// Property: Ghost text range is non-empty for non-empty completions
    ///
    /// For any non-empty completion, the ghost text's range should not be empty
    /// (start and end should be different).
    #[test]
    fn prop_ghost_text_range_non_empty_for_non_empty_completion(
        completion in completion_item_strategy(),
        position in position_strategy(),
    ) {
        // Filter out empty completions
        if completion.insert_text.is_empty() {
            return Ok(());
        }

        let generator = BasicGhostTextGenerator::new();
        let ghost_text = generator.generate_ghost_text(&completion, position);

        // Range should not be empty
        prop_assert_ne!(ghost_text.range.start, ghost_text.range.end);
    }

    /// Property: Ghost text range is empty for empty completions
    ///
    /// For any empty completion, the ghost text's range should be empty
    /// (start and end should be the same).
    #[test]
    fn prop_ghost_text_range_empty_for_empty_completion(
        position in position_strategy(),
    ) {
        let generator = BasicGhostTextGenerator::new();
        let completion = CompletionItem::new(
            "empty".to_string(),
            CompletionItemKind::Text,
            "".to_string(),
        );

        let ghost_text = generator.generate_ghost_text(&completion, position);

        // Range should be empty
        prop_assert_eq!(ghost_text.range.start, ghost_text.range.end);
        prop_assert_eq!(ghost_text.range.start, position);
    }

    /// Property: Multiline ghost text has correct line count
    ///
    /// For any multi-line completion, the ghost text's range end line should
    /// be the start line plus the number of newlines in the text.
    #[test]
    fn prop_multiline_ghost_text_line_count(
        completion in completion_item_strategy(),
        position in position_strategy(),
    ) {
        // Filter to only multi-line completions
        if !completion.insert_text.contains('\n') {
            return Ok(());
        }

        let generator = BasicGhostTextGenerator::new();
        let ghost_text = generator.generate_multiline_ghost_text(&completion, position);

        // Count newlines
        let newline_count = completion.insert_text.matches('\n').count() as u32;
        let expected_end_line = position.line + newline_count;

        prop_assert_eq!(ghost_text.range.end.line, expected_end_line);
    }

    /// Property: Ghost text is deterministic
    ///
    /// For any completion and position, generating ghost text twice should
    /// produce identical results.
    #[test]
    fn prop_ghost_text_is_deterministic(
        completion in completion_item_strategy(),
        position in position_strategy(),
    ) {
        let generator = BasicGhostTextGenerator::new();

        let ghost_text1 = generator.generate_ghost_text(&completion, position);
        let ghost_text2 = generator.generate_ghost_text(&completion, position);

        prop_assert_eq!(ghost_text1, ghost_text2);
    }

    /// Property: Ghost text text is never modified
    ///
    /// For any completion, the ghost text's text field should be an exact copy
    /// of the completion's insert_text, with no modifications or transformations.
    #[test]
    fn prop_ghost_text_text_unmodified(
        completion in completion_item_strategy(),
        position in position_strategy(),
    ) {
        let generator = BasicGhostTextGenerator::new();
        let ghost_text = generator.generate_ghost_text(&completion, position);

        // Text should be identical
        prop_assert_eq!(ghost_text.text.len(), completion.insert_text.len());
        prop_assert_eq!(&ghost_text.text, &completion.insert_text);

        // Verify character-by-character
        for (i, (c1, c2)) in ghost_text
            .text
            .chars()
            .zip(completion.insert_text.chars())
            .enumerate()
        {
            prop_assert_eq!(c1, c2, "Character mismatch at index {}", i);
        }
    }

    /// Property: Ghost text range is consistent across multiple calls
    ///
    /// For any completion and position, the ghost text's range should be
    /// consistent across multiple calls to the generator.
    #[test]
    fn prop_ghost_text_range_consistent(
        completion in completion_item_strategy(),
        position in position_strategy(),
    ) {
        let generator = BasicGhostTextGenerator::new();

        let ghost_text1 = generator.generate_ghost_text(&completion, position);
        let ghost_text2 = generator.generate_ghost_text(&completion, position);
        let ghost_text3 = generator.generate_ghost_text(&completion, position);

        prop_assert_eq!(ghost_text1.range, ghost_text2.range);
        prop_assert_eq!(ghost_text2.range, ghost_text3.range);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ghost_text_accuracy_simple() {
        let generator = BasicGhostTextGenerator::new();
        let completion = CompletionItem::new(
            "test".to_string(),
            CompletionItemKind::Text,
            "hello world".to_string(),
        );
        let position = Position::new(0, 5);

        let ghost_text = generator.generate_ghost_text(&completion, position);

        // Verify accuracy
        assert_eq!(ghost_text.text, "hello world");
        assert_eq!(ghost_text.range.start, position);
        assert_eq!(
            ghost_text.range.end.character,
            position.character + "hello world".len() as u32
        );
    }

    #[test]
    fn test_ghost_text_accuracy_multiline() {
        let generator = BasicGhostTextGenerator::new();
        let completion = CompletionItem::new(
            "block".to_string(),
            CompletionItemKind::Snippet,
            "if true {\n    println!(\"yes\");\n}".to_string(),
        );
        let position = Position::new(5, 10);

        let ghost_text = generator.generate_multiline_ghost_text(&completion, position);

        // Verify accuracy
        assert_eq!(ghost_text.text, "if true {\n    println!(\"yes\");\n}");
        assert_eq!(ghost_text.range.start, position);
        assert_eq!(ghost_text.range.end.line, 7); // 5 + 2 newlines
    }
}
