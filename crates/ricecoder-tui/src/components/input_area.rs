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
}
