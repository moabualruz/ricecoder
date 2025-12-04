/// Unit tests for ghost text functionality
///
/// Tests cover:
/// - Ghost text generation from completions
/// - Ghost text acceptance
/// - Ghost text dismissal
/// - Ghost text updates on context change

use ricecoder_completion::{
    BasicGhostTextGenerator, BasicGhostTextKeyHandler, BasicGhostTextStateManager,
    CompletionItem, CompletionItemKind, GhostText, GhostTextGenerator, GhostTextKeyHandler,
    GhostTextState, GhostTextStateManager, PartialAcceptanceMode, Position, Range,
};

#[test]
fn test_ghost_text_generation_from_completion() {
    let generator = BasicGhostTextGenerator::new();
    let completion = CompletionItem::new(
        "println".to_string(),
        CompletionItemKind::Function,
        "println!(\"Hello\")".to_string(),
    );
    let position = Position::new(0, 5);

    let ghost_text = generator.generate_ghost_text(&completion, position);

    assert_eq!(ghost_text.text, "println!(\"Hello\")");
    assert_eq!(ghost_text.range.start, position);
    assert_eq!(
        ghost_text.range.end.character,
        position.character + "println!(\"Hello\")".len() as u32
    );
}

#[test]
fn test_ghost_text_generation_multiline() {
    let generator = BasicGhostTextGenerator::new();
    let completion = CompletionItem::new(
        "fn".to_string(),
        CompletionItemKind::Keyword,
        "fn main() {\n    println!(\"Hello\");\n}".to_string(),
    );
    let position = Position::new(0, 0);

    let ghost_text = generator.generate_multiline_ghost_text(&completion, position);

    assert_eq!(ghost_text.text, "fn main() {\n    println!(\"Hello\");\n}");
    assert_eq!(ghost_text.range.start, position);
    // End position should be on line 2 (0-indexed)
    assert_eq!(ghost_text.range.end.line, 2);
}

#[test]
fn test_ghost_text_acceptance() {
    let mut manager = BasicGhostTextStateManager::new();
    let ghost_text = GhostText::new(
        "test".to_string(),
        Range::new(Position::new(0, 0), Position::new(0, 4)),
    );

    manager.display(ghost_text.clone());
    assert!(manager.is_displayed());

    let accepted = manager.accept();
    assert!(accepted.is_some());
    assert_eq!(accepted.unwrap().text, "test");
    assert_eq!(manager.get_state(), &GhostTextState::Accepted(ghost_text));
}

#[test]
fn test_ghost_text_dismissal() {
    let mut manager = BasicGhostTextStateManager::new();
    let ghost_text = GhostText::new(
        "test".to_string(),
        Range::new(Position::new(0, 0), Position::new(0, 4)),
    );

    manager.display(ghost_text);
    assert!(manager.is_displayed());

    manager.dismiss();
    assert!(!manager.is_displayed());
    assert_eq!(manager.get_state(), &GhostTextState::Dismissed);
}

#[test]
fn test_ghost_text_update_on_context_change() {
    let mut manager = BasicGhostTextStateManager::new();
    let ghost_text1 = GhostText::new(
        "test".to_string(),
        Range::new(Position::new(0, 0), Position::new(0, 4)),
    );
    let ghost_text2 = GhostText::new(
        "updated".to_string(),
        Range::new(Position::new(0, 0), Position::new(0, 7)),
    );

    manager.display(ghost_text1.clone());
    assert_eq!(manager.get_state(), &GhostTextState::Displayed(ghost_text1));

    manager.update(ghost_text2.clone());
    assert_eq!(manager.get_state(), &GhostTextState::Displayed(ghost_text2));
}

#[test]
fn test_ghost_text_partial_acceptance_word() {
    let mut manager = BasicGhostTextStateManager::new();
    let ghost_text = GhostText::new(
        "hello world".to_string(),
        Range::new(Position::new(0, 0), Position::new(0, 11)),
    );

    manager.display(ghost_text);
    let partial = manager.accept_partial(PartialAcceptanceMode::Word);

    assert_eq!(partial, Some("hello".to_string()));
}

#[test]
fn test_ghost_text_partial_acceptance_line() {
    let mut manager = BasicGhostTextStateManager::new();
    let ghost_text = GhostText::new(
        "hello\nworld".to_string(),
        Range::new(Position::new(0, 0), Position::new(1, 5)),
    );

    manager.display(ghost_text);
    let partial = manager.accept_partial(PartialAcceptanceMode::Line);

    assert_eq!(partial, Some("hello".to_string()));
}

#[test]
fn test_ghost_text_partial_acceptance_characters() {
    let mut manager = BasicGhostTextStateManager::new();
    let ghost_text = GhostText::new(
        "hello world".to_string(),
        Range::new(Position::new(0, 0), Position::new(0, 11)),
    );

    manager.display(ghost_text);
    let partial = manager.accept_partial(PartialAcceptanceMode::Characters(5));

    assert_eq!(partial, Some("hello".to_string()));
}

#[test]
fn test_ghost_text_key_handler_escape() {
    let state_manager = Box::new(BasicGhostTextStateManager::new());
    let mut handler = BasicGhostTextKeyHandler::new(state_manager);

    // Should not panic
    handler.handle_escape();
}

#[test]
fn test_ghost_text_key_handler_character_input() {
    let state_manager = Box::new(BasicGhostTextStateManager::new());
    let mut handler = BasicGhostTextKeyHandler::new(state_manager);

    // Should not panic
    handler.handle_character_input('a');
}

#[test]
fn test_ghost_text_generation_empty_completion() {
    let generator = BasicGhostTextGenerator::new();
    let completion = CompletionItem::new(
        "empty".to_string(),
        CompletionItemKind::Text,
        "".to_string(),
    );
    let position = Position::new(0, 0);

    let ghost_text = generator.generate_ghost_text(&completion, position);

    assert_eq!(ghost_text.text, "");
    assert_eq!(ghost_text.range.start, position);
    assert_eq!(ghost_text.range.end, position);
}

#[test]
fn test_ghost_text_generation_with_special_characters() {
    let generator = BasicGhostTextGenerator::new();
    let completion = CompletionItem::new(
        "special".to_string(),
        CompletionItemKind::Text,
        "let x = \"hello\\nworld\";".to_string(),
    );
    let position = Position::new(0, 10);

    let ghost_text = generator.generate_ghost_text(&completion, position);

    assert_eq!(ghost_text.text, "let x = \"hello\\nworld\";");
    assert_eq!(ghost_text.range.start, position);
}

#[test]
fn test_ghost_text_state_transitions() {
    let mut manager = BasicGhostTextStateManager::new();

    // Initial state: Dismissed
    assert_eq!(manager.get_state(), &GhostTextState::Dismissed);

    // Display ghost text
    let ghost_text = GhostText::new(
        "test".to_string(),
        Range::new(Position::new(0, 0), Position::new(0, 4)),
    );
    manager.display(ghost_text.clone());
    assert_eq!(manager.get_state(), &GhostTextState::Displayed(ghost_text.clone()));

    // Accept ghost text
    manager.accept();
    assert_eq!(manager.get_state(), &GhostTextState::Accepted(ghost_text));

    // Dismiss after acceptance
    manager.dismiss();
    assert_eq!(manager.get_state(), &GhostTextState::Dismissed);
}

#[test]
fn test_ghost_text_accept_when_dismissed() {
    let mut manager = BasicGhostTextStateManager::new();

    // Try to accept when no ghost text is displayed
    let result = manager.accept();
    assert_eq!(result, None);
}

#[test]
fn test_ghost_text_partial_acceptance_when_dismissed() {
    let mut manager = BasicGhostTextStateManager::new();

    // Try to partially accept when no ghost text is displayed
    let result = manager.accept_partial(PartialAcceptanceMode::Word);
    assert_eq!(result, None);
}

#[test]
fn test_ghost_text_multiline_with_tabs() {
    let generator = BasicGhostTextGenerator::new();
    let completion = CompletionItem::new(
        "if_block".to_string(),
        CompletionItemKind::Snippet,
        "if condition {\n\t// code\n}".to_string(),
    );
    let position = Position::new(0, 0);

    let ghost_text = generator.generate_multiline_ghost_text(&completion, position);

    assert_eq!(ghost_text.text, "if condition {\n\t// code\n}");
    assert_eq!(ghost_text.range.start, position);
    assert_eq!(ghost_text.range.end.line, 2);
}

#[test]
fn test_ghost_text_generation_unicode() {
    let generator = BasicGhostTextGenerator::new();
    let completion = CompletionItem::new(
        "unicode".to_string(),
        CompletionItemKind::Text,
        "println!(\"Hello, 世界\");".to_string(),
    );
    let position = Position::new(0, 0);

    let ghost_text = generator.generate_ghost_text(&completion, position);

    assert_eq!(ghost_text.text, "println!(\"Hello, 世界\");");
    assert_eq!(ghost_text.range.start, position);
}
