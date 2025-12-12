//! Session management and display widgets
//!
//! This module provides widgets for displaying and managing multiple sessions,
//! including session tabs, list views, status indicators, and session switching.

use std::collections::HashMap;

/// Session status indicator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionStatus {
    /// Session is active and running
    Active,
    /// Session is idle
    Idle,
    /// Session has unsaved changes
    Dirty,
    /// Session is loading
    Loading,
    /// Session has an error
    Error,
}

impl SessionStatus {
    /// Get the display symbol for the status
    pub fn symbol(&self) -> &'static str {
        match self {
            SessionStatus::Active => "●",
            SessionStatus::Idle => "○",
            SessionStatus::Dirty => "◆",
            SessionStatus::Loading => "◐",
            SessionStatus::Error => "✕",
        }
    }

    /// Get the display name for the status
    pub fn display_name(&self) -> &'static str {
        match self {
            SessionStatus::Active => "Active",
            SessionStatus::Idle => "Idle",
            SessionStatus::Dirty => "Dirty",
            SessionStatus::Loading => "Loading",
            SessionStatus::Error => "Error",
        }
    }
}

/// Session information
#[derive(Debug, Clone)]
pub struct Session {
    /// Unique session identifier
    pub id: String,
    /// Session name/title
    pub name: String,
    /// Session status
    pub status: SessionStatus,
    /// Last activity timestamp (seconds since epoch)
    pub last_activity: u64,
    /// Number of messages in session
    pub message_count: usize,
    /// Whether session has unsaved changes
    pub has_changes: bool,
    /// Session metadata
    pub metadata: HashMap<String, String>,
}

impl Session {
    /// Create a new session
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            status: SessionStatus::Idle,
            last_activity: 0,
            message_count: 0,
            has_changes: false,
            metadata: HashMap::new(),
        }
    }

    /// Update session status
    pub fn set_status(&mut self, status: SessionStatus) {
        self.status = status;
    }

    /// Mark session as having changes
    pub fn mark_dirty(&mut self) {
        self.has_changes = true;
        self.status = SessionStatus::Dirty;
    }

    /// Mark session as clean
    pub fn mark_clean(&mut self) {
        self.has_changes = false;
        if self.status == SessionStatus::Dirty {
            self.status = SessionStatus::Idle;
        }
    }

    /// Update last activity timestamp
    pub fn update_activity(&mut self) {
        self.last_activity = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
    }

    /// Increment message count
    pub fn add_message(&mut self) {
        self.message_count += 1;
        self.update_activity();
    }
}

/// Session display mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionDisplayMode {
    /// Display sessions as tabs
    Tabs,
    /// Display sessions as a list
    List,
}

/// Session widget for displaying and managing sessions
#[derive(Debug, Clone)]
pub struct SessionWidget {
    /// All sessions
    pub sessions: Vec<Session>,
    /// Currently selected session index
    pub selected_index: usize,
    /// Display mode (tabs or list)
    pub display_mode: SessionDisplayMode,
    /// Whether the session panel is visible
    pub visible: bool,
    /// Scroll offset for list view
    pub scroll_offset: usize,
    /// Maximum visible sessions in list view
    pub max_visible: usize,
}

impl SessionWidget {
    /// Create a new session widget
    pub fn new() -> Self {
        Self {
            sessions: Vec::new(),
            selected_index: 0,
            display_mode: SessionDisplayMode::Tabs,
            visible: true,
            scroll_offset: 0,
            max_visible: 10,
        }
    }

    /// Add a new session
    pub fn add_session(&mut self, session: Session) {
        self.sessions.push(session);
        // New sessions become the current session
        self.selected_index = self.sessions.len() - 1;
    }

    /// Remove a session by index
    pub fn remove_session(&mut self, index: usize) -> Option<Session> {
        if index < self.sessions.len() {
            let removed = self.sessions.remove(index);

            // Adjust selected index if needed
            if self.selected_index >= self.sessions.len() && !self.sessions.is_empty() {
                self.selected_index = self.sessions.len() - 1;
            }

            Some(removed)
        } else {
            None
        }
    }

    /// Get the currently selected session
    pub fn current_session(&self) -> Option<&Session> {
        self.sessions.get(self.selected_index)
    }

    /// Get mutable reference to the currently selected session
    pub fn current_session_mut(&mut self) -> Option<&mut Session> {
        self.sessions.get_mut(self.selected_index)
    }

    /// Switch to a session by index
    pub fn switch_to_session(&mut self, index: usize) -> bool {
        if index < self.sessions.len() {
            self.selected_index = index;
            if let Some(session) = self.current_session_mut() {
                session.set_status(SessionStatus::Active);
            }
            true
        } else {
            false
        }
    }

    /// Switch to a session by ID
    pub fn switch_to_session_by_id(&mut self, id: &str) -> bool {
        if let Some(index) = self.sessions.iter().position(|s| s.id == id) {
            self.switch_to_session(index)
        } else {
            false
        }
    }

    /// Get the next session
    pub fn next_session(&mut self) -> bool {
        if self.sessions.is_empty() {
            return false;
        }
        let next_index = (self.selected_index + 1) % self.sessions.len();
        self.switch_to_session(next_index);
        true
    }

    /// Get the previous session
    pub fn previous_session(&mut self) -> bool {
        if self.sessions.is_empty() {
            return false;
        }
        let prev_index = if self.selected_index == 0 {
            self.sessions.len() - 1
        } else {
            self.selected_index - 1
        };
        self.switch_to_session(prev_index);
        true
    }

    /// Rename a session
    pub fn rename_session(&mut self, index: usize, new_name: String) -> bool {
        if let Some(session) = self.sessions.get_mut(index) {
            session.name = new_name;
            true
        } else {
            false
        }
    }

    /// Rename the current session
    pub fn rename_current_session(&mut self, new_name: String) -> bool {
        self.rename_session(self.selected_index, new_name)
    }

    /// Toggle display mode between tabs and list
    pub fn toggle_display_mode(&mut self) {
        self.display_mode = match self.display_mode {
            SessionDisplayMode::Tabs => SessionDisplayMode::List,
            SessionDisplayMode::List => SessionDisplayMode::Tabs,
        };
    }

    /// Set display mode
    pub fn set_display_mode(&mut self, mode: SessionDisplayMode) {
        self.display_mode = mode;
    }

    /// Toggle visibility
    pub fn toggle_visibility(&mut self) {
        self.visible = !self.visible;
    }

    /// Show the session panel
    pub fn show(&mut self) {
        self.visible = true;
    }

    /// Hide the session panel
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Scroll up in list view
    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    /// Scroll down in list view
    pub fn scroll_down(&mut self) {
        let max_scroll = self.sessions.len().saturating_sub(self.max_visible);
        if self.scroll_offset < max_scroll {
            self.scroll_offset += 1;
        }
    }

    /// Get visible sessions for list view
    pub fn visible_sessions(&self) -> Vec<&Session> {
        self.sessions
            .iter()
            .skip(self.scroll_offset)
            .take(self.max_visible)
            .collect()
    }

    /// Get the number of sessions
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Check if there are any sessions
    pub fn has_sessions(&self) -> bool {
        !self.sessions.is_empty()
    }

    /// Clear all sessions
    pub fn clear(&mut self) {
        self.sessions.clear();
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    /// Get session by ID
    pub fn get_session(&self, id: &str) -> Option<&Session> {
        self.sessions.iter().find(|s| s.id == id)
    }

    /// Get mutable session by ID
    pub fn get_session_mut(&mut self, id: &str) -> Option<&mut Session> {
        self.sessions.iter_mut().find(|s| s.id == id)
    }

    /// Get all session IDs
    pub fn session_ids(&self) -> Vec<&str> {
        self.sessions.iter().map(|s| s.id.as_str()).collect()
    }

    /// Get all session names
    pub fn session_names(&self) -> Vec<&str> {
        self.sessions.iter().map(|s| s.name.as_str()).collect()
    }

    /// Find session index by ID
    pub fn find_session_index(&self, id: &str) -> Option<usize> {
        self.sessions.iter().position(|s| s.id == id)
    }

    /// Get the selected session index
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Get the display mode
    pub fn display_mode(&self) -> SessionDisplayMode {
        self.display_mode
    }

    /// Check if visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }
}

impl Default for SessionWidget {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = Session::new("session-1".to_string(), "Session 1".to_string());
        assert_eq!(session.id, "session-1");
        assert_eq!(session.name, "Session 1");
        assert_eq!(session.status, SessionStatus::Idle);
        assert_eq!(session.message_count, 0);
        assert!(!session.has_changes);
    }

    #[test]
    fn test_session_status_changes() {
        let mut session = Session::new("session-1".to_string(), "Session 1".to_string());

        session.set_status(SessionStatus::Active);
        assert_eq!(session.status, SessionStatus::Active);

        session.mark_dirty();
        assert_eq!(session.status, SessionStatus::Dirty);
        assert!(session.has_changes);

        session.mark_clean();
        assert!(!session.has_changes);
    }

    #[test]
    fn test_session_widget_add_session() {
        let mut widget = SessionWidget::new();
        assert_eq!(widget.session_count(), 0);

        let session = Session::new("session-1".to_string(), "Session 1".to_string());
        widget.add_session(session);

        assert_eq!(widget.session_count(), 1);
        assert_eq!(widget.selected_index(), 0);
    }

    #[test]
    fn test_session_widget_switch_session() {
        let mut widget = SessionWidget::new();

        widget.add_session(Session::new(
            "session-1".to_string(),
            "Session 1".to_string(),
        ));
        widget.add_session(Session::new(
            "session-2".to_string(),
            "Session 2".to_string(),
        ));

        // After adding session-2, it becomes the current session
        assert_eq!(widget.selected_index(), 1);

        widget.switch_to_session(0);
        assert_eq!(widget.selected_index(), 0);

        let current = widget.current_session().unwrap();
        assert_eq!(current.id, "session-1");
    }

    #[test]
    fn test_session_widget_next_previous() {
        let mut widget = SessionWidget::new();

        widget.add_session(Session::new(
            "session-1".to_string(),
            "Session 1".to_string(),
        ));
        widget.add_session(Session::new(
            "session-2".to_string(),
            "Session 2".to_string(),
        ));
        widget.add_session(Session::new(
            "session-3".to_string(),
            "Session 3".to_string(),
        ));

        // After adding session-3, it becomes the current session
        assert_eq!(widget.selected_index(), 2);

        widget.next_session();
        assert_eq!(widget.selected_index(), 0); // Wraps around

        widget.next_session();
        assert_eq!(widget.selected_index(), 1);

        widget.next_session();
        assert_eq!(widget.selected_index(), 2);

        widget.previous_session();
        assert_eq!(widget.selected_index(), 1);
    }

    #[test]
    fn test_session_widget_remove_session() {
        let mut widget = SessionWidget::new();

        widget.add_session(Session::new(
            "session-1".to_string(),
            "Session 1".to_string(),
        ));
        widget.add_session(Session::new(
            "session-2".to_string(),
            "Session 2".to_string(),
        ));

        assert_eq!(widget.session_count(), 2);

        let removed = widget.remove_session(0);
        assert!(removed.is_some());
        assert_eq!(widget.session_count(), 1);
        assert_eq!(widget.selected_index(), 0);
    }

    #[test]
    fn test_session_widget_rename() {
        let mut widget = SessionWidget::new();

        widget.add_session(Session::new(
            "session-1".to_string(),
            "Session 1".to_string(),
        ));

        widget.rename_current_session("New Name".to_string());

        let current = widget.current_session().unwrap();
        assert_eq!(current.name, "New Name");
    }

    #[test]
    fn test_session_widget_display_mode() {
        let mut widget = SessionWidget::new();

        assert_eq!(widget.display_mode(), SessionDisplayMode::Tabs);

        widget.toggle_display_mode();
        assert_eq!(widget.display_mode(), SessionDisplayMode::List);

        widget.set_display_mode(SessionDisplayMode::Tabs);
        assert_eq!(widget.display_mode(), SessionDisplayMode::Tabs);
    }

    #[test]
    fn test_session_widget_visibility() {
        let mut widget = SessionWidget::new();

        assert!(widget.is_visible());

        widget.hide();
        assert!(!widget.is_visible());

        widget.show();
        assert!(widget.is_visible());

        widget.toggle_visibility();
        assert!(!widget.is_visible());
    }

    #[test]
    fn test_session_status_symbols() {
        assert_eq!(SessionStatus::Active.symbol(), "●");
        assert_eq!(SessionStatus::Idle.symbol(), "○");
        assert_eq!(SessionStatus::Dirty.symbol(), "◆");
        assert_eq!(SessionStatus::Loading.symbol(), "◐");
        assert_eq!(SessionStatus::Error.symbol(), "✕");
    }

    #[test]
    fn test_session_widget_scroll() {
        let mut widget = SessionWidget::new();
        widget.max_visible = 3;

        for i in 0..10 {
            widget.add_session(Session::new(
                format!("session-{}", i),
                format!("Session {}", i),
            ));
        }

        assert_eq!(widget.scroll_offset, 0);

        widget.scroll_down();
        assert_eq!(widget.scroll_offset, 1);

        widget.scroll_up();
        assert_eq!(widget.scroll_offset, 0);
    }
}
