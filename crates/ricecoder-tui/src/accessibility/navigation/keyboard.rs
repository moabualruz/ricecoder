//! Keyboard navigation manager

use std::collections::HashMap;

use ratatui::style::{Color, Modifier, Style};

use crate::accessibility::{ElementType, TextAlternative};
use crate::model::AppMessage;

/// Keyboard navigation manager
#[derive(Debug, Clone, Default, PartialEq)]
pub struct KeyboardNavigationManager {
    /// Currently focused element ID
    pub focused_element: Option<String>,
    /// Tab order (list of element IDs in tab order)
    pub tab_order: Vec<String>,
    /// Element descriptions
    pub element_descriptions: HashMap<String, TextAlternative>,
}

impl KeyboardNavigationManager {
    /// Create a new keyboard navigation manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an element for keyboard navigation
    pub fn register_element(&mut self, alternative: TextAlternative) {
        self.tab_order.push(alternative.id.clone());
        self.element_descriptions
            .insert(alternative.id.clone(), alternative);
    }

    /// Set focus to an element
    pub fn focus(&mut self, element_id: &str) -> bool {
        if self.element_descriptions.contains_key(element_id) {
            self.focused_element = Some(element_id.to_string());
            true
        } else {
            false
        }
    }

    /// Move focus to the next element
    pub fn focus_next(&mut self) -> Option<&TextAlternative> {
        if self.tab_order.is_empty() {
            return None;
        }

        let next_index = match &self.focused_element {
            None => 0,
            Some(current) => {
                let current_index = self.tab_order.iter().position(|id| id == current)?;
                (current_index + 1) % self.tab_order.len()
            }
        };

        let next_id = self.tab_order[next_index].clone();
        self.focused_element = Some(next_id.clone());
        self.element_descriptions.get(&next_id)
    }

    /// Move focus to the previous element
    pub fn focus_previous(&mut self) -> Option<&TextAlternative> {
        if self.tab_order.is_empty() {
            return None;
        }

        let prev_index = match &self.focused_element {
            None => self.tab_order.len() - 1,
            Some(current) => {
                let current_index = self.tab_order.iter().position(|id| id == current)?;
                if current_index == 0 {
                    self.tab_order.len() - 1
                } else {
                    current_index - 1
                }
            }
        };

        let prev_id = self.tab_order[prev_index].clone();
        self.focused_element = Some(prev_id.clone());
        self.element_descriptions.get(&prev_id)
    }

    /// Get the currently focused element
    pub fn current_focus(&self) -> Option<&TextAlternative> {
        self.focused_element
            .as_ref()
            .and_then(|id| self.element_descriptions.get(id))
    }

    /// Clear all registered elements
    pub fn clear(&mut self) {
        self.focused_element = None;
        self.tab_order.clear();
        self.element_descriptions.clear();
    }
}

/// Enhanced keyboard navigation with WCAG compliance
#[derive(Debug, Clone)]
pub struct EnhancedKeyboardNavigation {
    /// Base keyboard navigation manager
    base_nav: KeyboardNavigationManager,
    /// Focus ring style for visual indicators
    focus_ring_style: Style,
    /// High contrast mode
    high_contrast: bool,
    /// Logical tab order (element IDs in navigation order)
    tab_order: Vec<String>,
    /// Current focus index in tab order
    current_focus_index: Option<usize>,
    /// Focus history for back navigation
    focus_history: Vec<String>,
}

impl EnhancedKeyboardNavigation {
    pub fn new() -> Self {
        Self {
            base_nav: KeyboardNavigationManager::new(),
            focus_ring_style: Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::REVERSED),
            high_contrast: false,
            tab_order: Vec::new(),
            current_focus_index: None,
            focus_history: Vec::new(),
        }
    }

    /// Register an element for keyboard navigation
    pub fn register_element(&mut self, element_id: String, alternative: TextAlternative) {
        self.base_nav.register_element(alternative);
        self.tab_order.push(element_id);
    }

    /// Navigate to next focusable element (Tab)
    pub fn tab_next(&mut self) -> Option<AppMessage> {
        if self.tab_order.is_empty() {
            return None;
        }

        let next_index = match self.current_focus_index {
            None => 0,
            Some(current) => (current + 1) % self.tab_order.len(),
        };

        self.set_focus_by_index(next_index)
    }

    /// Navigate to previous focusable element (Shift+Tab)
    pub fn tab_previous(&mut self) -> Option<AppMessage> {
        if self.tab_order.is_empty() {
            return None;
        }

        let prev_index = match self.current_focus_index {
            None => self.tab_order.len() - 1,
            Some(0) => self.tab_order.len() - 1,
            Some(current) => current - 1,
        };

        self.set_focus_by_index(prev_index)
    }

    /// Set focus by tab order index
    fn set_focus_by_index(&mut self, index: usize) -> Option<AppMessage> {
        if index >= self.tab_order.len() {
            return None;
        }

        let element_id = &self.tab_order[index];

        // Update focus history
        if let Some(current_idx) = self.current_focus_index {
            if let Some(current_id) = self.tab_order.get(current_idx) {
                self.focus_history.push(current_id.clone());
                // Keep only last 10 items
                if self.focus_history.len() > 10 {
                    self.focus_history.remove(0);
                }
            }
        }

        self.current_focus_index = Some(index);
        self.base_nav.focus(element_id);

        Some(AppMessage::FocusChanged(element_id.clone()))
    }

    /// Get current focused element
    pub fn current_focus(&self) -> Option<&TextAlternative> {
        self.base_nav.current_focus()
    }

    /// Get focus ring style
    pub fn focus_ring_style(&self) -> Style {
        if self.high_contrast {
            Style::default().fg(Color::Black).bg(Color::White)
        } else {
            self.focus_ring_style
        }
    }

    /// Set high contrast mode
    pub fn set_high_contrast(&mut self, enabled: bool) {
        self.high_contrast = enabled;
    }

    /// Check if high contrast is enabled
    pub fn is_high_contrast(&self) -> bool {
        self.high_contrast
    }

    /// Get tab order for debugging
    pub fn tab_order(&self) -> &[String] {
        &self.tab_order
    }
}
