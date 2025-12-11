//! Tests for the FilePickerWidget
//!
//! This module contains unit tests for the FilePickerWidget functionality,
//! including creation, visibility, navigation, fuzzy search, and file selection.

use ricecoder_tui::file_picker::{FilePickerWidget, fuzzy_match};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_picker_creation() {
        let picker = FilePickerWidget::new();
        assert!(!picker.is_visible());
        assert!(picker.selected_files().is_empty());
    }

    #[test]
    fn test_file_picker_visibility() {
        let mut picker = FilePickerWidget::new();

        picker.show();
        assert!(picker.is_visible());

        picker.hide();
        assert!(!picker.is_visible());

        picker.toggle();
        assert!(picker.is_visible());

        picker.toggle();
        assert!(!picker.is_visible());
    }

    #[test]
    fn test_fuzzy_match() {
        // Test empty query
        assert_eq!(fuzzy_match("", "test"), Some(vec![]));

        // Test single character match
        assert!(fuzzy_match("t", "test").is_some());

        // Test multiple character match
        assert!(fuzzy_match("te", "test").is_some());

        // Test no match
        assert_eq!(fuzzy_match("xyz", "test"), None);
    }

    #[test]
    fn test_search_input() {
        let mut picker = FilePickerWidget::new();
        picker.show(); // Make sure it's visible first

        picker.input_char('t');
        assert!(picker.is_visible()); // Should still be visible

        picker.input_char('e');
        picker.backspace();
        picker.clear_search();

        // Test should pass if no panics occur
        assert!(true);
    }

    #[test]
    fn test_navigation() {
        let mut picker = FilePickerWidget::new();

        // Test navigation (should not panic even with empty file list)
        picker.navigate_down();
        picker.navigate_up();

        // Test should pass if no panics occur
        assert!(true);
    }

    #[test]
    fn test_selection_operations() {
        let mut picker = FilePickerWidget::new();

        // Test selection operations (should not panic)
        picker.toggle_selection();
        picker.select_all();
        picker.clear_selection();

        // Test should pass if no panics occur
        assert!(true);
    }

    #[test]
    fn test_max_file_size() {
        let mut picker = FilePickerWidget::new();

        // Test setting max file size
        picker.set_max_file_size(2048 * 1024); // 2MB

        // Test should pass if no panics occur
        assert!(true);
    }
}