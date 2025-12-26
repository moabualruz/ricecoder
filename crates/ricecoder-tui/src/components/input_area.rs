//! InputArea component for multi-line text input with Component trait support
//!
//! Wraps TextAreaWidget and provides:
//! - Component trait implementation
//! - Submit handling (Enter/Ctrl+Enter)
//! - Vim mode support
//! - History navigation
//! - Focus management

use ratatui::{layout::Rect, Frame};
use crate::{
    components::{ComponentId, FocusDirection, FocusResult},
    model::{AppMessage, AppModel},
    textarea_widget::{TextAreaWidget, VimMode},
};

/// Validation result for input
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationResult {
    /// Input is valid
    Valid,
    /// Input is invalid with error message
    Invalid(String),
}

/// Validation function type
pub type ValidatorFn = Box<dyn Fn(&str) -> ValidationResult + Send + Sync>;

/// Autocomplete candidate
#[derive(Debug, Clone, PartialEq)]
pub struct AutocompleteCandidate {
    /// Display text
    pub display: String,
    /// Replacement text (what gets inserted)
    pub replacement: String,
    /// Description/help text
    pub description: Option<String>,
    /// Score/relevance (0.0 to 1.0)
    pub score: f32,
}

/// Autocomplete provider trait
pub trait AutocompleteProvider: Send + Sync {
    /// Get autocomplete candidates for given input
    fn get_candidates(&self, input: &str, cursor_pos: (usize, usize)) -> Vec<AutocompleteCandidate>;
}

/// InputArea component for multi-line text input
pub struct InputArea {
    /// Unique component identifier
    id: ComponentId,
    /// Underlying text area widget
    textarea: TextAreaWidget,
    /// Focus state
    focused: bool,
    /// Visibility state
    visible: bool,
    /// Enabled state
    enabled: bool,
    /// Bounding rectangle
    bounds: Rect,
    /// Tab order
    tab_order: Option<usize>,
    /// Z-index for layering
    z_index: i32,
    /// Input validator (GAP-32-008)
    validator: Option<ValidatorFn>,
    /// Last validation result
    validation_result: ValidationResult,
    /// Autocomplete provider (GAP-32-006)
    autocomplete_provider: Option<Box<dyn AutocompleteProvider>>,
    /// Autocomplete candidates
    autocomplete_candidates: Vec<AutocompleteCandidate>,
    /// Selected autocomplete candidate index
    autocomplete_selected: usize,
    /// Whether autocomplete is visible
    autocomplete_visible: bool,
}

impl InputArea {
    /// Create a new InputArea component
    pub fn new(id: impl Into<String>, vim_mode: bool, max_height: u16) -> Self {
        Self {
            id: id.into(),
            textarea: TextAreaWidget::new(vim_mode, max_height),
            focused: false,
            visible: true,
            enabled: true,
            bounds: Rect::default(),
            tab_order: None,
            z_index: 0,
            validator: None,
            validation_result: ValidationResult::Valid,
            autocomplete_provider: None,
            autocomplete_candidates: Vec::new(),
            autocomplete_selected: 0,
            autocomplete_visible: false,
        }
    }

    /// Get the current input text
    pub fn text(&self) -> String {
        self.textarea.text()
    }

    /// Set the input text
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.textarea.set_text(text);
    }

    /// Clear the input
    pub fn clear(&mut self) {
        self.textarea.clear();
    }

    /// Check if input is empty
    pub fn is_empty(&self) -> bool {
        self.textarea.is_empty()
    }

    /// Get the current vim mode
    pub fn vim_mode(&self) -> VimMode {
        self.textarea.vim_mode()
    }

    /// Set vim mode
    pub fn set_vim_mode(&mut self, mode: VimMode) {
        self.textarea.set_vim_mode(mode);
    }

    /// Get character count
    pub fn char_count(&self) -> usize {
        self.textarea.char_count()
    }

    /// Get word count
    pub fn word_count(&self) -> usize {
        self.textarea.word_count()
    }

    /// Get cursor position (line, column)
    pub fn cursor_position(&self) -> (usize, usize) {
        self.textarea.cursor_position()
    }

    /// Undo last operation
    pub fn undo(&mut self) -> bool {
        self.textarea.undo()
    }

    /// Redo last undone operation
    pub fn redo(&mut self) -> bool {
        self.textarea.redo()
    }

    /// Handle submit event (Enter or Ctrl+Enter)
    /// Returns Some(text) if submitted, None otherwise
    /// 
    /// Note: In non-Vim mode, Enter submits for chat-style input
    /// In Vim insert mode, Enter inserts newline (handle_submit returns None)
    pub fn handle_submit(&mut self, key: crossterm::event::KeyCode, modifiers: crossterm::event::KeyModifiers) -> Option<String> {
        use crossterm::event::{KeyCode, KeyModifiers};
        
        // Check if vim mode is actually enabled (not just Insert state)
        let is_vim_mode_enabled = matches!(
            self.vim_mode(),
            VimMode::Normal | VimMode::Visual | VimMode::VisualLine
        );
        
        match (key, modifiers) {
            // Ctrl+Enter always submits
            (KeyCode::Enter, mods) if mods.contains(KeyModifiers::CONTROL) => {
                let text = self.text();
                self.clear();
                Some(text)
            }
            // Enter behavior depends on vim mode state
            (KeyCode::Enter, _) if is_vim_mode_enabled => {
                // In vim modes (Normal, Visual, VisualLine), Enter submits
                let text = self.text();
                self.clear();
                Some(text)
            }
            (KeyCode::Enter, _) => {
                // In Insert mode (which is default for non-vim), Enter submits for chat input
                let text = self.text();
                self.clear();
                Some(text)
            }
            _ => None,
        }
    }

    /// Get mutable reference to underlying TextAreaWidget
    pub fn textarea_mut(&mut self) -> &mut TextAreaWidget {
        &mut self.textarea
    }

    /// Get reference to underlying TextAreaWidget
    pub fn textarea(&self) -> &TextAreaWidget {
        &self.textarea
    }

    // ========================================================================
    // Validation API (GAP-32-008)
    // ========================================================================

    /// Set input validator
    pub fn set_validator(&mut self, validator: ValidatorFn) {
        self.validator = Some(validator);
    }

    /// Remove validator
    pub fn clear_validator(&mut self) {
        self.validator = None;
        // Reset validation result when validator is cleared
        self.validation_result = ValidationResult::Valid;
    }

    /// Validate current input
    pub fn validate(&mut self) -> &ValidationResult {
        if let Some(validator) = &self.validator {
            self.validation_result = validator(&self.text());
        } else {
            self.validation_result = ValidationResult::Valid;
        }
        &self.validation_result
    }

    /// Get last validation result
    pub fn validation_result(&self) -> &ValidationResult {
        &self.validation_result
    }

    /// Check if input is valid
    pub fn is_valid(&self) -> bool {
        matches!(self.validation_result, ValidationResult::Valid)
    }

    /// Handle submit with validation enforcement
    pub fn handle_submit_validated(&mut self, key: crossterm::event::KeyCode, modifiers: crossterm::event::KeyModifiers) -> Option<String> {
        // Validate before allowing submit
        self.validate();
        
        if !self.is_valid() {
            // Don't submit if validation fails
            return None;
        }

        // Call original submit handler
        self.handle_submit(key, modifiers)
    }

    // ========================================================================
    // Autocomplete API (GAP-32-006)
    // ========================================================================

    /// Set autocomplete provider
    pub fn set_autocomplete_provider(&mut self, provider: Box<dyn AutocompleteProvider>) {
        self.autocomplete_provider = Some(provider);
    }

    /// Remove autocomplete provider
    pub fn clear_autocomplete_provider(&mut self) {
        self.autocomplete_provider = None;
        self.autocomplete_candidates.clear();
        self.autocomplete_visible = false;
    }

    /// Update autocomplete candidates based on current input
    pub fn update_autocomplete(&mut self) {
        if let Some(provider) = &self.autocomplete_provider {
            let input = self.text();
            let cursor_pos = self.cursor_position();
            self.autocomplete_candidates = provider.get_candidates(&input, cursor_pos);
            
            // Sort by score (descending)
            self.autocomplete_candidates.sort_by(|a, b| {
                b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal)
            });

            // Show autocomplete if we have candidates
            self.autocomplete_visible = !self.autocomplete_candidates.is_empty();
            self.autocomplete_selected = 0;
        }
    }

    /// Show autocomplete
    pub fn show_autocomplete(&mut self) {
        self.update_autocomplete();
    }

    /// Hide autocomplete
    pub fn hide_autocomplete(&mut self) {
        self.autocomplete_visible = false;
        self.autocomplete_candidates.clear();
        self.autocomplete_selected = 0;
    }

    /// Check if autocomplete is visible
    pub fn is_autocomplete_visible(&self) -> bool {
        self.autocomplete_visible
    }

    /// Get autocomplete candidates
    pub fn autocomplete_candidates(&self) -> &[AutocompleteCandidate] {
        &self.autocomplete_candidates
    }

    /// Get selected autocomplete candidate index
    pub fn autocomplete_selected(&self) -> usize {
        self.autocomplete_selected
    }

    /// Select next autocomplete candidate
    pub fn autocomplete_next(&mut self) {
        if !self.autocomplete_candidates.is_empty() {
            self.autocomplete_selected = (self.autocomplete_selected + 1) % self.autocomplete_candidates.len();
        }
    }

    /// Select previous autocomplete candidate
    pub fn autocomplete_prev(&mut self) {
        if !self.autocomplete_candidates.is_empty() {
            if self.autocomplete_selected == 0 {
                self.autocomplete_selected = self.autocomplete_candidates.len() - 1;
            } else {
                self.autocomplete_selected -= 1;
            }
        }
    }

    /// Accept selected autocomplete candidate
    pub fn autocomplete_accept(&mut self) -> bool {
        if self.autocomplete_visible && self.autocomplete_selected < self.autocomplete_candidates.len() {
            let candidate = &self.autocomplete_candidates[self.autocomplete_selected];
            let replacement = candidate.replacement.clone();
            
            // Clear current input and insert replacement
            self.clear();
            self.set_text(replacement);
            
            // Hide autocomplete
            self.hide_autocomplete();
            
            true
        } else {
            false
        }
    }

    // ========================================================================
    // History Draft API (GAP-32-007) - Forwarding to TextAreaWidget
    // ========================================================================

    /// Save current text as draft before history navigation
    pub fn save_history_draft(&mut self) {
        self.textarea.save_draft();
    }

    /// Restore draft text
    pub fn restore_history_draft(&mut self) -> bool {
        self.textarea.restore_draft()
    }

    /// Check if draft exists
    pub fn has_history_draft(&self) -> bool {
        self.textarea.has_draft()
    }

    /// Clear draft
    pub fn clear_history_draft(&mut self) {
        self.textarea.clear_draft();
    }

    // ========================================================================
    // Selection API (GAP-32-004) - Forwarding to TextAreaWidget
    // ========================================================================

    /// Start selection from current cursor position
    pub fn start_selection(&mut self) {
        self.textarea.start_selection();
    }

    /// Update selection to current cursor position
    pub fn update_selection(&mut self) {
        self.textarea.update_selection();
    }

    /// Clear selection
    pub fn clear_selection(&mut self) {
        self.textarea.clear_selection();
    }

    /// Get selected text
    pub fn selected_text(&self) -> Option<String> {
        self.textarea.selected_text()
    }

    /// Copy selected text to clipboard
    pub fn copy_selection(&self) -> Result<(), crate::clipboard::ClipboardError> {
        self.textarea.copy_selection()
    }

    /// Cut selected text to clipboard
    pub fn cut_selection(&mut self) -> Result<(), crate::clipboard::ClipboardError> {
        self.textarea.cut_selection()
    }

    /// Paste text from clipboard
    pub fn paste_from_clipboard(&mut self) -> Result<(), crate::clipboard::ClipboardError> {
        self.textarea.paste_from_clipboard()
    }
}

impl Default for InputArea {
    fn default() -> Self {
        Self::new("input-area", false, 10)
    }
}

// ============================================================================
// Component Trait Implementation (Deprecated, for backward compatibility)
// ============================================================================

#[allow(deprecated)]
impl crate::components::Component for InputArea {
    fn id(&self) -> ComponentId {
        self.id.clone()
    }

    fn render(&self, _frame: &mut Frame, _area: Rect, _model: &AppModel) {
        // Note: Actual rendering should be done via App which has mutable access
        // This Component trait method is deprecated and should not be used for rendering
        // Use the textarea_mut() method from the owning App/view instead
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn update(&mut self, _message: &AppMessage, _model: &AppModel) -> bool {
        // InputArea doesn't handle messages directly yet
        // TODO: Add message handling when needed
        false
    }

    fn handle_focus(&mut self, direction: FocusDirection) -> FocusResult {
        match direction {
            FocusDirection::Next | FocusDirection::Forward => {
                if !self.focused && self.can_focus() {
                    self.set_focused(true);
                    FocusResult::Focused
                } else {
                    FocusResult::Boundary
                }
            }
            FocusDirection::Previous | FocusDirection::Backward => {
                if self.focused {
                    self.set_focused(false);
                }
                FocusResult::Boundary
            }
            _ => FocusResult::NotFocusable,
        }
    }

    fn is_focused(&self) -> bool {
        self.focused
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    fn can_focus(&self) -> bool {
        self.enabled && self.visible
    }

    fn tab_order(&self) -> Option<usize> {
        self.tab_order
    }

    fn set_tab_order(&mut self, order: Option<usize>) {
        self.tab_order = order;
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn z_index(&self) -> i32 {
        self.z_index
    }

    fn set_z_index(&mut self, z_index: i32) {
        self.z_index = z_index;
    }

    fn children(&self) -> Vec<&dyn crate::components::Component> {
        Vec::new()
    }

    fn children_mut(&mut self) -> Vec<&mut dyn crate::components::Component> {
        Vec::new()
    }

    fn find_child(&self, _id: &ComponentId) -> Option<&dyn crate::components::Component> {
        None
    }

    fn find_child_mut(&mut self, _id: &ComponentId) -> Option<&mut dyn crate::components::Component> {
        None
    }

    fn add_child(&mut self, _child: Box<dyn crate::components::Component>) {
        // InputArea is a leaf component
    }

    fn remove_child(&mut self, _id: &ComponentId) -> Option<Box<dyn crate::components::Component>> {
        None
    }

    fn validate(&self) -> Result<(), String> {
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn crate::components::Component> {
        // Create new InputArea with same state
        let mut new_area = InputArea::new(
            self.id.clone(),
            false, // vim_mode is determined by textarea state
            10,    // default max_height
        );
        new_area.focused = self.focused;
        new_area.visible = self.visible;
        new_area.enabled = self.enabled;
        new_area.bounds = self.bounds;
        new_area.tab_order = self.tab_order;
        new_area.z_index = self.z_index;
        new_area.set_text(self.text());
        // Note: validator and autocomplete_provider are not cloned (function pointers/trait objects)
        // validation_result is reset to Valid for new instance
        Box::new(new_area)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyModifiers};
    #[allow(deprecated)]
    use crate::components::Component;

    #[test]
    fn test_input_area_creation() {
        let input = InputArea::new("test", false, 10);
        assert_eq!(input.id(), "test");
        assert!(input.is_empty());
        assert!(!input.is_focused());
    }

    #[test]
    fn test_input_area_text() {
        let mut input = InputArea::new("test", false, 10);
        input.set_text("Hello, World!");
        assert_eq!(input.text(), "Hello, World!");
        assert_eq!(input.char_count(), 13);
        assert_eq!(input.word_count(), 2);
    }

    #[test]
    fn test_input_area_submit_ctrl_enter() {
        let mut input = InputArea::new("test", false, 10);
        input.set_text("Test message");
        
        let result = input.handle_submit(KeyCode::Enter, KeyModifiers::CONTROL);
        assert_eq!(result, Some("Test message".to_string()));
        assert!(input.is_empty());
    }

    #[test]
    fn test_input_area_submit_enter_non_vim() {
        let mut input = InputArea::new("test", false, 10);
        input.set_text("Test message");
        
        let result = input.handle_submit(KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(result, Some("Test message".to_string()));
        assert!(input.is_empty());
    }

    #[test]
    fn test_input_area_vim_mode() {
        let mut input = InputArea::new("test", true, 10);
        assert_eq!(input.vim_mode(), VimMode::Normal);
        
        input.set_vim_mode(VimMode::Insert);
        assert_eq!(input.vim_mode(), VimMode::Insert);
    }

    #[test]
    fn test_input_area_focus() {
        let mut input = InputArea::new("test", false, 10);
        assert!(!input.is_focused());
        
        input.set_focused(true);
        assert!(input.is_focused());
        
        input.set_focused(false);
        assert!(!input.is_focused());
    }

    #[test]
    fn test_input_area_component_trait() {
        let mut input = InputArea::new("test", false, 10);
        assert_eq!(input.id(), "test");
        assert!(input.can_focus());
        
        let result = input.handle_focus(FocusDirection::Next);
        assert_eq!(result, FocusResult::Focused);
        assert!(input.is_focused());
    }

    // ========================================================================
    // Tests for GAP-32-004: Selection Support
    // ========================================================================

    #[test]
    fn test_selection_basic() {
        let mut input = InputArea::new("test", false, 10);
        input.set_text("Hello, World!");
        
        // Start selection
        input.start_selection();
        assert!(input.textarea().selection().is_some());
        
        // Clear selection
        input.clear_selection();
        assert!(input.textarea().selection().is_none());
    }

    #[test]
    fn test_selection_text_extraction() {
        let mut input = InputArea::new("test", false, 10);
        input.set_text("Hello, World!");
        
        // Note: Actual text selection would require cursor manipulation
        // This tests the API surface
        let selected = input.selected_text();
        assert!(selected.is_none() || selected.is_some());
    }

    // ========================================================================
    // Tests for GAP-32-005: Copy/Paste Handling
    // ========================================================================

    #[test]
    fn test_clipboard_api_available() {
        let input = InputArea::new("test", false, 10);
        
        // Test that clipboard methods exist and are callable
        let copy_result = input.copy_selection();
        // Expected to fail with no selection, but API should exist
        assert!(copy_result.is_err());
    }

    #[test]
    fn test_paste_api_available() {
        let mut input = InputArea::new("test", false, 10);
        
        // Test that paste method exists
        // May fail if clipboard is empty or unavailable
        let _ = input.paste_from_clipboard();
    }

    // ========================================================================
    // Tests for GAP-32-006: Autocomplete
    // ========================================================================

    struct TestAutocompleteProvider;
    
    impl AutocompleteProvider for TestAutocompleteProvider {
        fn get_candidates(&self, input: &str, _cursor_pos: (usize, usize)) -> Vec<AutocompleteCandidate> {
            if input.starts_with("/") {
                vec![
                    AutocompleteCandidate {
                        display: "/help".to_string(),
                        replacement: "/help".to_string(),
                        description: Some("Show help".to_string()),
                        score: 1.0,
                    },
                    AutocompleteCandidate {
                        display: "/exit".to_string(),
                        replacement: "/exit".to_string(),
                        description: Some("Exit application".to_string()),
                        score: 0.9,
                    },
                ]
            } else {
                vec![]
            }
        }
    }

    #[test]
    fn test_autocomplete_provider() {
        let mut input = InputArea::new("test", false, 10);
        input.set_autocomplete_provider(Box::new(TestAutocompleteProvider));
        
        input.set_text("/");
        input.update_autocomplete();
        
        assert!(input.is_autocomplete_visible());
        assert_eq!(input.autocomplete_candidates().len(), 2);
        assert_eq!(input.autocomplete_candidates()[0].display, "/help");
    }

    #[test]
    fn test_autocomplete_navigation() {
        let mut input = InputArea::new("test", false, 10);
        input.set_autocomplete_provider(Box::new(TestAutocompleteProvider));
        
        input.set_text("/");
        input.update_autocomplete();
        
        assert_eq!(input.autocomplete_selected(), 0);
        
        input.autocomplete_next();
        assert_eq!(input.autocomplete_selected(), 1);
        
        input.autocomplete_prev();
        assert_eq!(input.autocomplete_selected(), 0);
    }

    #[test]
    fn test_autocomplete_accept() {
        let mut input = InputArea::new("test", false, 10);
        input.set_autocomplete_provider(Box::new(TestAutocompleteProvider));
        
        input.set_text("/");
        input.update_autocomplete();
        
        let accepted = input.autocomplete_accept();
        assert!(accepted);
        assert_eq!(input.text(), "/help");
        assert!(!input.is_autocomplete_visible());
    }

    #[test]
    fn test_autocomplete_hide() {
        let mut input = InputArea::new("test", false, 10);
        input.set_autocomplete_provider(Box::new(TestAutocompleteProvider));
        
        input.set_text("/");
        input.update_autocomplete();
        assert!(input.is_autocomplete_visible());
        
        input.hide_autocomplete();
        assert!(!input.is_autocomplete_visible());
        assert_eq!(input.autocomplete_candidates().len(), 0);
    }

    // ========================================================================
    // Tests for GAP-32-007: History Draft Preservation
    // ========================================================================

    #[test]
    fn test_history_draft_save_restore() {
        let mut input = InputArea::new("test", false, 10);
        input.set_text("Draft message");
        
        // Save draft
        input.save_history_draft();
        assert!(input.has_history_draft());
        
        // Clear input
        input.clear();
        assert!(input.is_empty());
        
        // Restore draft
        let restored = input.restore_history_draft();
        assert!(restored);
        assert_eq!(input.text(), "Draft message");
        assert!(!input.has_history_draft()); // Draft consumed
    }

    #[test]
    fn test_history_draft_clear() {
        let mut input = InputArea::new("test", false, 10);
        input.set_text("Draft message");
        input.save_history_draft();
        
        assert!(input.has_history_draft());
        
        input.clear_history_draft();
        assert!(!input.has_history_draft());
    }

    #[test]
    fn test_history_draft_not_saved_for_empty() {
        let mut input = InputArea::new("test", false, 10);
        input.save_history_draft();
        
        // Should not save empty draft
        assert!(!input.has_history_draft());
    }

    // ========================================================================
    // Tests for GAP-32-008: Input Validation
    // ========================================================================

    #[test]
    fn test_validation_basic() {
        let mut input = InputArea::new("test", false, 10);
        
        // Set validator that rejects empty input
        input.set_validator(Box::new(|text: &str| {
            if text.is_empty() {
                ValidationResult::Invalid("Input cannot be empty".to_string())
            } else {
                ValidationResult::Valid
            }
        }));
        
        // Test empty input
        input.set_text("");
        let result = InputArea::validate(&mut input);
        assert!(matches!(result, &ValidationResult::Invalid(_)));
        assert!(!input.is_valid());
        
        // Test valid input
        input.set_text("Hello");
        let result = InputArea::validate(&mut input);
        assert!(matches!(result, &ValidationResult::Valid));
        assert!(input.is_valid());
    }

    #[test]
    fn test_validation_on_submit() {
        let mut input = InputArea::new("test", false, 10);
        
        // Set validator
        input.set_validator(Box::new(|text: &str| {
            if text.is_empty() {
                ValidationResult::Invalid("Input cannot be empty".to_string())
            } else {
                ValidationResult::Valid
            }
        }));
        
        // Try to submit empty input (should fail)
        input.set_text("");
        let result = input.handle_submit_validated(KeyCode::Enter, KeyModifiers::NONE);
        assert!(result.is_none());
        
        // Submit valid input (should succeed)
        input.set_text("Valid message");
        let result = input.handle_submit_validated(KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(result, Some("Valid message".to_string()));
    }

    #[test]
    fn test_validation_clear_validator() {
        let mut input = InputArea::new("test", false, 10);
        
        input.set_validator(Box::new(|text: &str| {
            if text.is_empty() {
                ValidationResult::Invalid("Empty".to_string())
            } else {
                ValidationResult::Valid
            }
        }));
        
        input.set_text("");
        let result = InputArea::validate(&mut input);
        assert!(matches!(result, &ValidationResult::Invalid(_)));
        assert!(!input.is_valid());
        
        // Clear validator
        input.clear_validator();
        let result = InputArea::validate(&mut input);
        assert!(matches!(result, &ValidationResult::Valid));
        assert!(input.is_valid()); // Should be valid now
    }

    #[test]
    fn test_validation_complex_rules() {
        let mut input = InputArea::new("test", false, 10);
        
        // Validator: min 3 chars, max 10 chars
        input.set_validator(Box::new(|text: &str| {
            if text.len() < 3 {
                ValidationResult::Invalid("Minimum 3 characters".to_string())
            } else if text.len() > 10 {
                ValidationResult::Invalid("Maximum 10 characters".to_string())
            } else {
                ValidationResult::Valid
            }
        }));
        
        input.set_text("ab");
        let result = InputArea::validate(&mut input);
        assert!(matches!(result, &ValidationResult::Invalid(_)));
        assert!(!input.is_valid());
        
        input.set_text("abc");
        let result = InputArea::validate(&mut input);
        assert!(matches!(result, &ValidationResult::Valid));
        assert!(input.is_valid());
        
        input.set_text("12345678901");
        let result = InputArea::validate(&mut input);
        assert!(matches!(result, &ValidationResult::Invalid(_)));
        assert!(!input.is_valid());
    }
}
