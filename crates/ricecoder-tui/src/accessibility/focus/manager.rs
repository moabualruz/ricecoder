//! Focus management for accessibility
//!
//! Manages keyboard focus state and focus history for keyboard navigation.
//! Provides focus tracking, history management for focus restoration,
//! and focus state manipulation.

/// Focus management for accessibility
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FocusManager {
    /// Currently focused element
    pub focused_element: Option<String>,
    /// Focus history for restoration
    pub focus_history: Vec<String>,
}

impl FocusManager {
    /// Create a new focus manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Set focus to an element
    pub fn set_focus(&mut self, element_id: impl Into<String>) {
        let id = element_id.into();
        if let Some(current) = &self.focused_element {
            self.focus_history.push(current.clone());
        }
        self.focused_element = Some(id);
    }

    /// Restore previous focus
    pub fn restore_focus(&mut self) -> Option<String> {
        self.focus_history.pop()
    }

    /// Clear focus
    pub fn clear_focus(&mut self) {
        self.focused_element = None;
    }
}
