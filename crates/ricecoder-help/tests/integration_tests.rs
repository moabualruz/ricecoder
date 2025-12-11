//! Integration tests for ricecoder-help

use ricecoder_help::{HelpContent, HelpDialog, HelpCategory};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[test]
fn test_help_dialog_integration() {
    // Create help content
    let content = HelpContent::new()
        .add_category(
            HelpCategory::new("Test Category")
                .with_description("Test description")
                .add_item("Test Item", "Test content")
        );
    
    // Create dialog
    let mut dialog = HelpDialog::new(content);
    
    // Test initial state
    assert!(!dialog.is_visible());
    
    // Show dialog
    dialog.show();
    assert!(dialog.is_visible());
    
    // Test navigation
    let key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
    let handled = dialog.handle_key(key).unwrap();
    assert!(handled);
    
    // Test search mode
    let key = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL);
    let handled = dialog.handle_key(key).unwrap();
    assert!(handled);
    
    // Test typing in search
    let key = KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE);
    let handled = dialog.handle_key(key).unwrap();
    assert!(handled);
    
    // Test escape to close
    let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    let handled = dialog.handle_key(key).unwrap();
    assert!(handled);
    
    // Test escape again to close dialog
    let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    let handled = dialog.handle_key(key).unwrap();
    assert!(handled);
    assert!(!dialog.is_visible());
}

#[test]
fn test_default_ricecoder_help() {
    let dialog = HelpDialog::default_ricecoder();
    
    // Should have default content
    assert!(!dialog.is_visible());
    
    // Test that we can show it
    let mut dialog = dialog;
    dialog.show();
    assert!(dialog.is_visible());
}

#[test]
fn test_help_content_search() {
    let content = HelpContent::new()
        .add_category(
            HelpCategory::new("Commands")
                .add_item("help", "Show help dialog")
                .add_item("exit", "Exit application")
        )
        .add_category(
            HelpCategory::new("Navigation")
                .add_item("up", "Move up")
                .add_item("down", "Move down")
        );
    
    // Search for "help"
    let results = content.search("help");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].1.title, "help");
    
    // Search for "move"
    let results = content.search("move");
    assert_eq!(results.len(), 2); // "Move up" and "Move down"
    
    // Search for non-existent term
    let results = content.search("nonexistent");
    assert_eq!(results.len(), 0);
}

#[test]
fn test_help_keyboard_shortcuts() {
    let mut dialog = HelpDialog::default_ricecoder();
    dialog.show();
    
    // Test various keyboard shortcuts
    let shortcuts = vec![
        (KeyCode::Up, KeyModifiers::NONE),
        (KeyCode::Down, KeyModifiers::NONE),
        (KeyCode::Left, KeyModifiers::NONE),
        (KeyCode::Right, KeyModifiers::NONE),
        (KeyCode::PageUp, KeyModifiers::NONE),
        (KeyCode::PageDown, KeyModifiers::NONE),
        (KeyCode::Home, KeyModifiers::NONE),
        (KeyCode::End, KeyModifiers::NONE),
    ];
    
    for (code, modifiers) in shortcuts {
        let key = KeyEvent::new(code, modifiers);
        let handled = dialog.handle_key(key).unwrap();
        assert!(handled, "Key {:?} should be handled", code);
    }
}

#[test]
fn test_help_search_functionality() {
    let mut dialog = HelpDialog::default_ricecoder();
    dialog.show();
    
    // Enter search mode
    let key = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL);
    dialog.handle_key(key).unwrap();
    
    // Type search query
    let search_chars = ['h', 'e', 'l', 'p'];
    for c in search_chars {
        let key = KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE);
        dialog.handle_key(key).unwrap();
    }
    
    // Navigate search results
    let key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
    dialog.handle_key(key).unwrap();
    
    let key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
    dialog.handle_key(key).unwrap();
    
    // Exit search mode
    let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    dialog.handle_key(key).unwrap();
}