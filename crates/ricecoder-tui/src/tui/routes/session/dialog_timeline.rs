//! Timeline dialog for session route
//!
//! Provides a dialog for navigating session timeline with keyboard navigation.

use crate::components::menu::{MenuItem, MenuWidget};

/// Timeline item representing a message in the timeline
#[derive(Debug, Clone)]
pub struct TimelineItem {
    /// Message ID
    pub message_id: String,
    /// Message preview text
    pub preview: String,
    /// Formatted timestamp
    pub timestamp: String,
}

impl TimelineItem {
    /// Create a new timeline item
    pub fn new(message_id: impl Into<String>, preview: impl Into<String>, timestamp: impl Into<String>) -> Self {
        Self {
            message_id: message_id.into(),
            preview: preview.into(),
            timestamp: timestamp.into(),
        }
    }
}

/// Callback type for dialog actions
pub type DialogCallback = Box<dyn FnMut(&mut DialogTimeline, &str)>;

/// Move callback type for timeline navigation
pub type MoveCallback = Box<dyn FnMut(&str)>;

/// Timeline dialog widget
///
/// Displays a menu of messages in the session timeline with keyboard navigation.
///
/// # Examples
///
/// ```
/// use ricecoder_tui::tui::routes::session::dialog_timeline::{DialogTimeline, TimelineItem};
///
/// let mut dialog = DialogTimeline::new("session-123");
/// dialog.add_item(TimelineItem::new("msg-1", "First message", "2024-01-01 10:00"));
/// assert_eq!(dialog.session_id(), "session-123");
/// ```
pub struct DialogTimeline {
    /// Menu widget for timeline navigation
    menu: MenuWidget,
    /// Session ID
    session_id: String,
    /// Timeline items (message_id -> item)
    items: Vec<TimelineItem>,
    /// Whether dialog is open
    is_open: bool,
    /// Dialog size (normal or large)
    size: DialogSize,
    /// Callback for when item is selected
    on_select: Option<DialogCallback>,
    /// Callback for when moving through items
    on_move: Option<MoveCallback>,
}

/// Dialog size options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogSize {
    /// Normal size
    Normal,
    /// Large size
    Large,
}

impl DialogTimeline {
    /// Create a new timeline dialog
    ///
    /// # Arguments
    ///
    /// * `session_id` - The session ID to associate with this dialog
    ///
    /// # Examples
    ///
    /// ```
    /// use ricecoder_tui::tui::routes::session::dialog_timeline::DialogTimeline;
    ///
    /// let dialog = DialogTimeline::new("my-session");
    /// ```
    pub fn new(session_id: impl Into<String>) -> Self {
        let menu = MenuWidget::with_title("Timeline");

        Self {
            menu,
            session_id: session_id.into(),
            items: Vec::new(),
            is_open: false,
            size: DialogSize::Large,
            on_select: None,
            on_move: None,
        }
    }

    /// Get the session ID
    ///
    /// # Examples
    ///
    /// ```
    /// use ricecoder_tui::tui::routes::session::dialog_timeline::DialogTimeline;
    ///
    /// let dialog = DialogTimeline::new("session-123");
    /// assert_eq!(dialog.session_id(), "session-123");
    /// ```
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Add a timeline item
    ///
    /// # Examples
    ///
    /// ```
    /// use ricecoder_tui::tui::routes::session::dialog_timeline::{DialogTimeline, TimelineItem};
    ///
    /// let mut dialog = DialogTimeline::new("session-123");
    /// dialog.add_item(TimelineItem::new("msg-1", "Hello", "2024-01-01"));
    /// assert_eq!(dialog.item_count(), 1);
    /// ```
    pub fn add_item(&mut self, item: TimelineItem) {
        let menu_item = MenuItem::new(&item.preview)
            .with_description(&item.timestamp);
        self.menu.add_item(menu_item);
        self.items.push(item);
    }

    /// Add multiple timeline items
    pub fn add_items(&mut self, items: Vec<TimelineItem>) {
        for item in items {
            self.add_item(item);
        }
    }

    /// Get the underlying menu widget
    pub fn menu(&self) -> &MenuWidget {
        &self.menu
    }

    /// Get mutable reference to the menu widget
    pub fn menu_mut(&mut self) -> &mut MenuWidget {
        &mut self.menu
    }

    /// Get all timeline items
    pub fn items(&self) -> &[TimelineItem] {
        &self.items
    }

    /// Get item count
    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    /// Open the dialog
    ///
    /// # Examples
    ///
    /// ```
    /// use ricecoder_tui::tui::routes::session::dialog_timeline::DialogTimeline;
    ///
    /// let mut dialog = DialogTimeline::new("session-123");
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
    /// use ricecoder_tui::tui::routes::session::dialog_timeline::DialogTimeline;
    ///
    /// let mut dialog = DialogTimeline::new("session-123");
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
    /// use ricecoder_tui::tui::routes::session::dialog_timeline::DialogTimeline;
    ///
    /// let mut dialog = DialogTimeline::new("session-123");
    /// assert!(!dialog.is_open());
    /// dialog.open();
    /// assert!(dialog.is_open());
    /// ```
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Set dialog size
    pub fn set_size(&mut self, size: DialogSize) {
        self.size = size;
    }

    /// Get dialog size
    pub fn size(&self) -> DialogSize {
        self.size
    }

    /// Select the current item
    ///
    /// This triggers the on_select callback if one is set.
    pub fn select_current(&mut self) {
        let selected_idx = self.menu.selected_index();
        if let Some(item) = self.items.get(selected_idx) {
            let message_id = item.message_id.clone();
            
            // Take callback to avoid double borrow
            if let Some(mut callback) = self.on_select.take() {
                callback(self, &message_id);
                self.on_select = Some(callback);
            }
        }
    }

    /// Move to item by index
    ///
    /// This triggers the on_move callback if one is set.
    pub fn move_to(&mut self, index: usize) {
        if index < self.items.len() {
            self.menu.selected = index;
            
            let message_id = &self.items[index].message_id;
            if let Some(callback) = self.on_move.as_mut() {
                callback(message_id);
            }
        }
    }

    /// Clear the dialog
    ///
    /// Removes all items and closes the dialog.
    ///
    /// # Examples
    ///
    /// ```
    /// use ricecoder_tui::tui::routes::session::dialog_timeline::{DialogTimeline, TimelineItem};
    ///
    /// let mut dialog = DialogTimeline::new("session-123");
    /// dialog.add_item(TimelineItem::new("msg-1", "Hello", "2024-01-01"));
    /// dialog.open();
    /// dialog.clear();
    /// assert_eq!(dialog.item_count(), 0);
    /// assert!(!dialog.is_open());
    /// ```
    pub fn clear(&mut self) {
        self.items.clear();
        self.menu.clear();
        self.close();
    }

    /// Set callback for item selection
    ///
    /// The callback receives a mutable reference to the dialog and the message ID.
    pub fn on_select<F>(&mut self, callback: F)
    where
        F: FnMut(&mut DialogTimeline, &str) + 'static,
    {
        self.on_select = Some(Box::new(callback));
    }

    /// Set callback for move events
    ///
    /// The callback receives the message ID.
    pub fn on_move<F>(&mut self, callback: F)
    where
        F: FnMut(&str) + 'static,
    {
        self.on_move = Some(Box::new(callback));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_dialog() {
        let dialog = DialogTimeline::new("test-session");
        assert_eq!(dialog.session_id(), "test-session");
        assert!(!dialog.is_open());
        assert_eq!(dialog.item_count(), 0);
    }

    #[test]
    fn test_add_item() {
        let mut dialog = DialogTimeline::new("test-session");
        
        let item = TimelineItem::new("msg-1", "Test message", "2024-01-01 10:00");
        dialog.add_item(item);
        
        assert_eq!(dialog.item_count(), 1);
        assert_eq!(dialog.items()[0].message_id, "msg-1");
    }

    #[test]
    fn test_add_items() {
        let mut dialog = DialogTimeline::new("test-session");
        
        let items = vec![
            TimelineItem::new("msg-1", "First", "2024-01-01"),
            TimelineItem::new("msg-2", "Second", "2024-01-02"),
        ];
        dialog.add_items(items);
        
        assert_eq!(dialog.item_count(), 2);
    }

    #[test]
    fn test_open_close() {
        let mut dialog = DialogTimeline::new("test-session");
        assert!(!dialog.is_open());

        dialog.open();
        assert!(dialog.is_open());

        dialog.close();
        assert!(!dialog.is_open());
    }

    #[test]
    fn test_clear() {
        let mut dialog = DialogTimeline::new("test-session");
        dialog.add_item(TimelineItem::new("msg-1", "Test", "2024-01-01"));
        dialog.open();

        assert!(dialog.is_open());
        assert_eq!(dialog.item_count(), 1);

        dialog.clear();
        assert!(!dialog.is_open());
        assert_eq!(dialog.item_count(), 0);
    }

    #[test]
    fn test_dialog_size() {
        let mut dialog = DialogTimeline::new("test-session");
        assert_eq!(dialog.size(), DialogSize::Large);

        dialog.set_size(DialogSize::Normal);
        assert_eq!(dialog.size(), DialogSize::Normal);
    }

    #[test]
    fn test_menu_integration() {
        let mut dialog = DialogTimeline::new("test-session");
        dialog.add_item(TimelineItem::new("msg-1", "Hello world", "2024-01-01 10:00"));

        let menu = dialog.menu();
        assert_eq!(menu.item_count(), 1);
        let item = menu.selected_item().expect("should have item");
        assert_eq!(item.label, "Hello world");
        assert_eq!(item.description, Some("2024-01-01 10:00".to_string()));
    }

    #[test]
    fn test_timeline_item() {
        let item = TimelineItem::new("msg-123", "Test message", "2024-01-01 12:00:00");
        assert_eq!(item.message_id, "msg-123");
        assert_eq!(item.preview, "Test message");
        assert_eq!(item.timestamp, "2024-01-01 12:00:00");
    }
}
