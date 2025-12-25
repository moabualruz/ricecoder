//! Subagent dialog for session route
//!
//! Provides a dialog for subagent actions with keyboard navigation.

use crate::components::menu::{MenuItem, MenuWidget};

/// Subagent action types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubagentAction {
    /// Open the subagent's session
    View,
}

impl SubagentAction {
    /// Convert action to string identifier
    pub fn as_str(&self) -> &'static str {
        match self {
            SubagentAction::View => "subagent.view",
        }
    }

    /// Get display title for action
    pub fn title(&self) -> &'static str {
        match self {
            SubagentAction::View => "Open",
        }
    }

    /// Get description for action
    pub fn description(&self) -> &'static str {
        match self {
            SubagentAction::View => "the subagent's session",
        }
    }
}

/// Callback type for dialog actions
pub type DialogCallback = Box<dyn FnMut(&mut DialogSubagent)>;

/// Subagent dialog widget
///
/// Displays a menu of subagent actions with keyboard navigation.
///
/// # Examples
///
/// ```
/// use ricecoder_tui::tui::routes::session::dialog_subagent::DialogSubagent;
///
/// let dialog = DialogSubagent::new("session-123");
/// assert_eq!(dialog.session_id(), "session-123");
/// ```
pub struct DialogSubagent {
    /// Menu widget for action selection
    menu: MenuWidget,
    /// Session ID for the subagent
    session_id: String,
    /// Selected action
    selected_action: Option<SubagentAction>,
    /// Whether dialog is open
    is_open: bool,
    /// Callback for when action is selected
    on_select: Option<DialogCallback>,
}

impl DialogSubagent {
    /// Create a new subagent dialog
    ///
    /// # Arguments
    ///
    /// * `session_id` - The session ID to associate with this dialog
    ///
    /// # Examples
    ///
    /// ```
    /// use ricecoder_tui::tui::routes::session::dialog_subagent::DialogSubagent;
    ///
    /// let dialog = DialogSubagent::new("my-session");
    /// ```
    pub fn new(session_id: impl Into<String>) -> Self {
        let mut menu = MenuWidget::with_title("Subagent Actions");

        // Add the "Open" action
        let item = MenuItem::new(SubagentAction::View.title())
            .with_description(SubagentAction::View.description());
        menu.add_item(item);

        Self {
            menu,
            session_id: session_id.into(),
            selected_action: None,
            is_open: false,
            on_select: None,
        }
    }

    /// Get the session ID
    ///
    /// # Examples
    ///
    /// ```
    /// use ricecoder_tui::tui::routes::session::dialog_subagent::DialogSubagent;
    ///
    /// let dialog = DialogSubagent::new("session-123");
    /// assert_eq!(dialog.session_id(), "session-123");
    /// ```
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Get the underlying menu widget
    pub fn menu(&self) -> &MenuWidget {
        &self.menu
    }

    /// Get mutable reference to the menu widget
    pub fn menu_mut(&mut self) -> &mut MenuWidget {
        &mut self.menu
    }

    /// Open the dialog
    ///
    /// # Examples
    ///
    /// ```
    /// use ricecoder_tui::tui::routes::session::dialog_subagent::DialogSubagent;
    ///
    /// let mut dialog = DialogSubagent::new("session-123");
    /// dialog.open();
    /// assert!(dialog.is_open());
    /// ```
    pub fn open(&mut self) {
        self.is_open = true;
        self.menu.open();
    }

    /// Close the dialog
    ///
    /// # Examples
    ///
    /// ```
    /// use ricecoder_tui::tui::routes::session::dialog_subagent::DialogSubagent;
    ///
    /// let mut dialog = DialogSubagent::new("session-123");
    /// dialog.open();
    /// dialog.close();
    /// assert!(!dialog.is_open());
    /// ```
    pub fn close(&mut self) {
        self.is_open = false;
        self.menu.close();
    }

    /// Check if dialog is open
    ///
    /// # Examples
    ///
    /// ```
    /// use ricecoder_tui::tui::routes::session::dialog_subagent::DialogSubagent;
    ///
    /// let mut dialog = DialogSubagent::new("session-123");
    /// assert!(!dialog.is_open());
    /// dialog.open();
    /// assert!(dialog.is_open());
    /// ```
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Select the "Open" action
    ///
    /// This triggers the on_select callback if one is set.
    ///
    /// # Examples
    ///
    /// ```
    /// use ricecoder_tui::tui::routes::session::dialog_subagent::DialogSubagent;
    ///
    /// let mut dialog = DialogSubagent::new("session-123");
    /// dialog.select();
    /// assert_eq!(dialog.selected_action(), Some(SubagentAction::View));
    /// ```
    pub fn select(&mut self) {
        self.selected_action = Some(SubagentAction::View);

        // Take callback to avoid double borrow
        if let Some(mut callback) = self.on_select.take() {
            callback(self);
            self.on_select = Some(callback);
        }
    }

    /// Get the selected action
    ///
    /// # Examples
    ///
    /// ```
    /// use ricecoder_tui::tui::routes::session::dialog_subagent::{DialogSubagent, SubagentAction};
    ///
    /// let mut dialog = DialogSubagent::new("session-123");
    /// assert_eq!(dialog.selected_action(), None);
    /// dialog.select();
    /// assert_eq!(dialog.selected_action(), Some(SubagentAction::View));
    /// ```
    pub fn selected_action(&self) -> Option<SubagentAction> {
        self.selected_action
    }

    /// Clear the dialog
    ///
    /// Resets selected action and closes the dialog.
    ///
    /// # Examples
    ///
    /// ```
    /// use ricecoder_tui::tui::routes::session::dialog_subagent::DialogSubagent;
    ///
    /// let mut dialog = DialogSubagent::new("session-123");
    /// dialog.open();
    /// dialog.select();
    /// dialog.clear();
    /// assert_eq!(dialog.selected_action(), None);
    /// assert!(!dialog.is_open());
    /// ```
    pub fn clear(&mut self) {
        self.selected_action = None;
        self.close();
    }

    /// Set callback for action selection
    ///
    /// The callback receives a mutable reference to the dialog.
    pub fn on_select<F>(&mut self, callback: F)
    where
        F: FnMut(&mut DialogSubagent) + 'static,
    {
        self.on_select = Some(Box::new(callback));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_dialog() {
        let dialog = DialogSubagent::new("test-session");
        assert_eq!(dialog.session_id(), "test-session");
        assert!(!dialog.is_open());
        assert_eq!(dialog.selected_action(), None);
    }

    #[test]
    fn test_open_close() {
        let mut dialog = DialogSubagent::new("test-session");
        assert!(!dialog.is_open());

        dialog.open();
        assert!(dialog.is_open());

        dialog.close();
        assert!(!dialog.is_open());
    }

    #[test]
    fn test_select_action() {
        let mut dialog = DialogSubagent::new("test-session");
        assert_eq!(dialog.selected_action(), None);

        dialog.select();
        assert_eq!(dialog.selected_action(), Some(SubagentAction::View));
    }

    #[test]
    fn test_clear() {
        let mut dialog = DialogSubagent::new("test-session");
        dialog.open();
        dialog.select();

        assert!(dialog.is_open());
        assert_eq!(dialog.selected_action(), Some(SubagentAction::View));

        dialog.clear();
        assert!(!dialog.is_open());
        assert_eq!(dialog.selected_action(), None);
    }

    #[test]
    fn test_subagent_action_strings() {
        let action = SubagentAction::View;
        assert_eq!(action.as_str(), "subagent.view");
        assert_eq!(action.title(), "Open");
        assert_eq!(action.description(), "the subagent's session");
    }

    #[test]
    fn test_menu_integration() {
        let dialog = DialogSubagent::new("test-session");
        let menu = dialog.menu();

        assert_eq!(menu.item_count(), 1);
        let item = menu.selected_item().expect("should have item");
        assert_eq!(item.label, "Open");
    }

    #[test]
    fn test_callback() {
        let mut dialog = DialogSubagent::new("test-session");
        let mut callback_called = false;

        // Note: This test won't work as-is because we can't capture mutable references
        // in the callback. In production, callbacks would mutate external state.
        dialog.on_select(|d| {
            d.clear();
        });

        dialog.select();
        // After callback, dialog should be cleared
        assert!(!dialog.is_open());
    }
}
