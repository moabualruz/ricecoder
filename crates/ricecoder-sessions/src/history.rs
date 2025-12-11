//! History management for session conversations

use crate::models::Message;

/// Manages conversation history for a session
#[derive(Debug, Clone)]
pub struct HistoryManager {
    /// Messages in the history, ordered by timestamp
    messages: Vec<Message>,
    /// Maximum number of messages to keep in history (None = unlimited)
    max_size: Option<usize>,
}

impl HistoryManager {
    /// Create a new history manager
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            max_size: None,
        }
    }

    /// Create a new history manager with a maximum size limit
    pub fn with_max_size(max_size: usize) -> Self {
        Self {
            messages: Vec::new(),
            max_size: Some(max_size),
        }
    }

    /// Add a message to history
    ///
    /// Messages are automatically ordered by timestamp. If the history exceeds
    /// the maximum size, the oldest message is removed.
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);

        // Sort by timestamp to maintain ordering
        self.messages.sort_by_key(|m| m.timestamp);

        // Enforce size limit if set
        if let Some(max) = self.max_size {
            if self.messages.len() > max {
                // Remove oldest messages to stay within limit
                let remove_count = self.messages.len() - max;
                self.messages.drain(0..remove_count);
            }
        }
    }

    /// Get the most recent N messages
    ///
    /// Returns messages in chronological order (oldest first).
    pub fn get_recent_messages(&self, count: usize) -> Vec<Message> {
        if count == 0 {
            return Vec::new();
        }

        let start = if self.messages.len() > count {
            self.messages.len() - count
        } else {
            0
        };

        self.messages[start..].to_vec()
    }

    /// Search history by content
    ///
    /// Returns all messages whose content contains the query string (case-insensitive).
    /// Results are returned in chronological order.
    pub fn search_by_content(&self, query: &str) -> Vec<Message> {
        let query_lower = query.to_lowercase();
        self.messages
            .iter()
            .filter(|m| m.content().to_lowercase().contains(&query_lower))
            .cloned()
            .collect()
    }

    /// Get all messages in the history
    ///
    /// Returns messages in chronological order (oldest first).
    pub fn get_all_messages(&self) -> Vec<Message> {
        self.messages.clone()
    }

    /// Get the number of messages in the history
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// Clear all messages from the history
    pub fn clear(&mut self) {
        self.messages.clear();
    }

    /// Get the maximum size limit (if set)
    pub fn max_size(&self) -> Option<usize> {
        self.max_size
    }
}

impl Default for HistoryManager {
    fn default() -> Self {
        Self::new()
    }
}
