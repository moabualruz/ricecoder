//! Message dialog for session route
//!
//! Provides a dialog for message actions (Revert, Copy, Fork) with keyboard navigation.

use crate::components::menu::{MenuItem, MenuWidget};

/// Message action types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageAction {
    /// Revert to this message
    Revert,
    /// Copy message text to clipboard
    Copy,
    /// Fork session from this message
    Fork,
}

impl MessageAction {
    /// Convert action to string identifier
    pub fn as_str(&self) -> &'static str {
        match self {
            MessageAction::Revert => "session.revert",
            MessageAction::Copy => "message.copy",
            MessageAction::Fork => "session.fork",
        }
    }

    /// Get display title for action
    pub fn title(&self) -> &'static str {
        match self {
            MessageAction::Revert => "Revert",
            MessageAction::Copy => "Copy",
            MessageAction::Fork => "Fork",
        }
    }

    /// Get description for action
    pub fn description(&self) -> &'static str {
        match self {
            MessageAction::Revert => "undo messages and file changes",
            MessageAction::Copy => "message text to clipboard",
            MessageAction::Fork => "create a new session",
        }
    }

    /// Get all available actions
    pub fn all() -> Vec<Self> {
        vec![
            MessageAction::Revert,
            MessageAction::Copy,
            MessageAction::Fork,
        ]
    }
}

/// Callback type for dialog actions
pub type DialogCallback = Box<dyn FnMut(&mut DialogMessage)>;

/// Message dialog widget
///
/// Displays a menu of message actions with keyboard navigation.
///
/// # Examples
///
/// ```
/// use ricecoder_tui::tui::routes::session::dialog_message::DialogMessage;
///
/// let dialog = DialogMessage::new("msg-123", "session-456");
/// assert_eq!(dialog.message_id(), "msg-123");
/// assert_eq!(dialog.session_id(), "session-456");
/// ```
pub struct DialogMessage {
    /// Menu widget for action selection
    menu: MenuWidget,
    /// Message ID
    message_id: String,
    /// Session ID
    session_id: String,
    /// Selected action
    selected_action: Option<MessageAction>,
    /// Whether dialog is open
    is_open: bool,
    /// Callback for when action is selected
    on_select: Option<DialogCallback>,
}

impl DialogMessage {
    /// Create a new message dialog
    ///
    /// # Arguments
    ///
    /// * `message_id` - The message ID to associate with this dialog
    /// * `session_id` - The session ID to associate with this dialog
    ///
    /// # Examples
    ///
    /// ```
    /// use ricecoder_tui::tui::routes::session::dialog_message::DialogMessage;
    ///
    /// let dialog = DialogMessage::new("msg-123", "session-456");
    /// ```
    pub fn new(message_id: impl Into<String>, session_id: impl Into<String>) -> Self {
        let mut menu = MenuWidget::with_title("Message Actions");

        // Add all actions
        for action in MessageAction::all() {
            let item = MenuItem::new(action.title())
                .with_description(action.description());
            menu.add_item(item);
        }

        Self {
            menu,
            message_id: message_id.into(),
            session_id: session_id.into(),
            selected_action: None,
            is_open: false,
            on_select: None,
        }
    }

    /// Get the message ID
    ///
    /// # Examples
    ///
    /// ```
    /// use ricecoder_tui::tui::routes::session::dialog_message::DialogMessage;
    ///
    /// let dialog = DialogMessage::new("msg-123", "session-456");
    /// assert_eq!(dialog.message_id(), "msg-123");
    /// ```
    pub fn message_id(&self) -> &str {
        &self.message_id
    }

    /// Get the session ID
    ///
    /// # Examples
    ///
    /// ```
    /// use ricecoder_tui::tui::routes::session::dialog_message::DialogMessage;
    ///
    /// let dialog = DialogMessage::new("msg-123", "session-456");
    /// assert_eq!(dialog.session_id(), "session-456");
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
    /// use ricecoder_tui::tui::routes::session::dialog_message::DialogMessage;
    ///
    /// let mut dialog = DialogMessage::new("msg-123", "session-456");
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
    /// use ricecoder_tui::tui::routes::session::dialog_message::DialogMessage;
    ///
    /// let mut dialog = DialogMessage::new("msg-123", "session-456");
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
    /// use ricecoder_tui::tui::routes::session::dialog_message::DialogMessage;
    ///
    /// let mut dialog = DialogMessage::new("msg-123", "session-456");
    /// assert!(!dialog.is_open());
    /// dialog.open();
    /// assert!(dialog.is_open());
    /// ```
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Select an action
    ///
    /// This triggers the on_select callback if one is set.
    ///
    /// # Examples
    ///
    /// ```
    /// use ricecoder_tui::tui::routes::session::dialog_message::{DialogMessage, MessageAction};
    ///
    /// let mut dialog = DialogMessage::new("msg-123", "session-456");
    /// dialog.select(MessageAction::Copy);
    /// assert_eq!(dialog.selected_action(), Some(MessageAction::Copy));
    /// ```
    pub fn select(&mut self, action: MessageAction) {
        self.selected_action = Some(action);

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
    /// use ricecoder_tui::tui::routes::session::dialog_message::{DialogMessage, MessageAction};
    ///
    /// let mut dialog = DialogMessage::new("msg-123", "session-456");
    /// assert_eq!(dialog.selected_action(), None);
    /// dialog.select(MessageAction::Fork);
    /// assert_eq!(dialog.selected_action(), Some(MessageAction::Fork));
    /// ```
    pub fn selected_action(&self) -> Option<MessageAction> {
        self.selected_action
    }

    /// Clear the dialog
    ///
    /// Resets selected action and closes the dialog.
    ///
    /// # Examples
    ///
    /// ```
    /// use ricecoder_tui::tui::routes::session::dialog_message::{DialogMessage, MessageAction};
    ///
    /// let mut dialog = DialogMessage::new("msg-123", "session-456");
    /// dialog.open();
    /// dialog.select(MessageAction::Revert);
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
        F: FnMut(&mut DialogMessage) + 'static,
    {
        self.on_select = Some(Box::new(callback));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_dialog() {
        let dialog = DialogMessage::new("msg-123", "session-456");
        assert_eq!(dialog.message_id(), "msg-123");
        assert_eq!(dialog.session_id(), "session-456");
        assert!(!dialog.is_open());
        assert_eq!(dialog.selected_action(), None);
    }

    #[test]
    fn test_open_close() {
        let mut dialog = DialogMessage::new("msg-123", "session-456");
        assert!(!dialog.is_open());

        dialog.open();
        assert!(dialog.is_open());

        dialog.close();
        assert!(!dialog.is_open());
    }

    #[test]
    fn test_select_action() {
        let mut dialog = DialogMessage::new("msg-123", "session-456");
        assert_eq!(dialog.selected_action(), None);

        dialog.select(MessageAction::Copy);
        assert_eq!(dialog.selected_action(), Some(MessageAction::Copy));

        dialog.select(MessageAction::Fork);
        assert_eq!(dialog.selected_action(), Some(MessageAction::Fork));
    }

    #[test]
    fn test_clear() {
        let mut dialog = DialogMessage::new("msg-123", "session-456");
        dialog.open();
        dialog.select(MessageAction::Revert);

        assert!(dialog.is_open());
        assert_eq!(dialog.selected_action(), Some(MessageAction::Revert));

        dialog.clear();
        assert!(!dialog.is_open());
        assert_eq!(dialog.selected_action(), None);
    }

    #[test]
    fn test_message_action_strings() {
        assert_eq!(MessageAction::Revert.as_str(), "session.revert");
        assert_eq!(MessageAction::Copy.as_str(), "message.copy");
        assert_eq!(MessageAction::Fork.as_str(), "session.fork");

        assert_eq!(MessageAction::Revert.title(), "Revert");
        assert_eq!(MessageAction::Copy.title(), "Copy");
        assert_eq!(MessageAction::Fork.title(), "Fork");

        assert_eq!(MessageAction::Revert.description(), "undo messages and file changes");
        assert_eq!(MessageAction::Copy.description(), "message text to clipboard");
        assert_eq!(MessageAction::Fork.description(), "create a new session");
    }

    #[test]
    fn test_all_actions() {
        let actions = MessageAction::all();
        assert_eq!(actions.len(), 3);
        assert!(actions.contains(&MessageAction::Revert));
        assert!(actions.contains(&MessageAction::Copy));
        assert!(actions.contains(&MessageAction::Fork));
    }

    #[test]
    fn test_menu_integration() {
        let dialog = DialogMessage::new("msg-123", "session-456");
        let menu = dialog.menu();

        assert_eq!(menu.item_count(), 3);
        let item = menu.selected_item().expect("should have item");
        assert_eq!(item.label, "Revert");
    }

    #[test]
    fn test_callback() {
        let mut dialog = DialogMessage::new("msg-123", "session-456");

        dialog.on_select(|d| {
            d.clear();
        });

        dialog.select(MessageAction::Copy);
        // After callback, dialog should be cleared
        assert!(!dialog.is_open());
    }
}
