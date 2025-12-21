//! Unit tests for TextAreaWidget
//!
//! These tests verify the functionality of the TextAreaWidget,
//! including text editing, vim mode, selection, and undo/redo operations.

use ricecoder_tui::textarea_widget::{TextAreaWidget, VimMode};

#[test]
fn test_textarea_creation() {
    let textarea = TextAreaWidget::new(false, 10);
    assert!(textarea.is_empty());
    assert_eq!(textarea.line_count(), 1);
}

#[test]
fn test_textarea_text_operations() {
    let mut textarea = TextAreaWidget::new(false, 10);

    textarea.set_text("Hello, world!");
    assert_eq!(textarea.text(), "Hello, world!");
    assert!(!textarea.is_empty());

    textarea.clear();
    assert!(textarea.is_empty());
}

#[test]
fn test_textarea_line_count() {
    let mut textarea = TextAreaWidget::new(false, 10);

    textarea.set_text("Line 1\nLine 2\nLine 3");
    assert_eq!(textarea.line_count(), 3);
}

#[test]
fn test_textarea_cursor_position() {
    let textarea = TextAreaWidget::new(false, 10);
    let (row, col) = textarea.cursor_position();
    assert_eq!(row, 0);
    assert_eq!(col, 0);
}

#[test]
fn test_textarea_vim_mode_enabled() {
    let textarea = TextAreaWidget::new(true, 10);
    assert_eq!(textarea.vim_mode(), VimMode::Normal);

    let textarea = TextAreaWidget::new(false, 10);
    assert_eq!(textarea.vim_mode(), VimMode::Insert);
}

#[test]
fn test_textarea_selection() {
    let mut textarea = TextAreaWidget::new(false, 10);
    textarea.set_text("Hello World");

    textarea.select_all();
    assert!(textarea.selection().is_some());
    let selected = textarea.selected_text().unwrap();
    assert_eq!(selected, "Hello World");
}

#[test]
fn test_textarea_undo_redo() {
    let mut textarea = TextAreaWidget::new(false, 10);
    textarea.set_text("Hello");

    // Change text again to create undo state
    textarea.set_text("Hello World");
    assert_eq!(textarea.text(), "Hello World");

    // Undo should restore to "Hello"
    assert!(textarea.undo());
    assert_eq!(textarea.text(), "Hello");

    // Redo should restore to "Hello World"
    assert!(textarea.redo());
    assert_eq!(textarea.text(), "Hello World");
}

#[test]
fn test_textarea_char_count() {
    let mut textarea = TextAreaWidget::new(false, 10);
    textarea.set_text("Hello World");
    assert_eq!(textarea.char_count(), 11);
}

#[test]
fn test_textarea_word_count() {
    let mut textarea = TextAreaWidget::new(false, 10);
    textarea.set_text("Hello World");
    assert_eq!(textarea.word_count(), 2);
}

#[test]
fn test_textarea_copy_text() {
    let mut textarea = TextAreaWidget::new(false, 10);
    textarea.set_text("Copy this text");

    let copied = textarea.copy_text();
    assert_eq!(copied, "Copy this text");
}

#[test]
fn test_textarea_paste_text() {
    let mut textarea = TextAreaWidget::new(false, 10);
    textarea.paste_text("Pasted text");

    assert_eq!(textarea.text(), "Pasted text");
}

#[test]
fn test_textarea_required_height() {
    let mut textarea = TextAreaWidget::new(false, 5);

    textarea.set_text("Line 1\nLine 2\nLine 3");
    let height = textarea.required_height();
    assert_eq!(height, 3);

    textarea.set_text("Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6");
    let height = textarea.required_height();
    assert_eq!(height, 5); // Capped at max_height
}

#[test]
fn test_textarea_vim_mode() {
    let textarea = TextAreaWidget::new(true, 10);
    assert_eq!(textarea.vim_mode(), VimMode::Normal);

    let textarea = TextAreaWidget::new(false, 10);
    assert_eq!(textarea.vim_mode(), VimMode::Insert);
}
