//! UI widgets for the TUI
//!
//! This module provides the core UI widgets used in the RiceCoder TUI, including:
//! - `ChatWidget`: Displays conversation messages with markdown rendering and streaming support
//! - `Message`: Represents a single message in the chat
//! - `StreamingMessage`: Manages real-time token streaming for AI responses
//! - `MessageAuthor`: Identifies the sender of a message (user or AI)
//!
//! # Examples
//!
//! Creating and displaying a chat message:
//!
//! ```ignore
//! use ricecoder_tui::{Message, MessageAuthor};
//!
//! let message = Message {
//!     content: "Hello, how can I help?".to_string(),
//!     author: MessageAuthor::AI,
//!     streaming: false,
//! };
//! ```
//!
//! Streaming a response token by token:
//!
//! ```ignore
//! use ricecoder_tui::StreamingMessage;
//!
//! let mut streaming = StreamingMessage::new();
//! streaming.append("Hello");
//! streaming.append(" ");
//! streaming.append("world");
//! assert_eq!(streaming.content, "Hello world");
//! ```

use crate::clipboard::{CopyFeedback, CopyOperation};
use std::collections::HashMap;
use chrono::{DateTime, Local};

/// Message in the chat
#[derive(Debug, Clone)]
pub struct Message {
    /// Message content
    pub content: String,
    /// Message author (user or AI)
    pub author: MessageAuthor,
    /// Whether message is being streamed
    pub streaming: bool,
    /// Message timestamp
    pub timestamp: DateTime<Local>,
    /// Optional metadata
    pub metadata: HashMap<String, String>,
    /// Tool calls associated with this message
    pub tool_calls: Vec<ToolCallDisplay>,
}

/// Tool call display info
#[derive(Debug, Clone)]
pub struct ToolCallDisplay {
    /// Tool name
    pub name: String,
    /// Tool parameters (JSON string)
    pub params: String,
    /// Execution status
    pub status: ToolStatus,
    /// Tool output
    pub output: Option<String>,
    /// Execution duration in ms
    pub duration_ms: Option<u64>,
}

/// Tool execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolStatus {
    /// Tool is running
    Running,
    /// Tool completed successfully
    Success,
    /// Tool failed
    Error,
}

/// Streaming message state
#[derive(Debug, Clone)]
pub struct StreamingMessage {

    /// Accumulated content
    pub content: String,
    /// Whether streaming is active
    pub active: bool,
    /// Cursor position for animation
    pub cursor_pos: usize,
    /// Animation frame counter for cursor blinking
    pub animation_frame: u32,
    /// Total tokens received
    pub token_count: usize,
}

impl Default for StreamingMessage {
    fn default() -> Self {
        Self {
            content: String::new(),
            active: true,
            cursor_pos: 0,
            animation_frame: 0,
            token_count: 0,
        }
    }
}

impl StreamingMessage {
    /// Create a new streaming message
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a token to the message
    pub fn append(&mut self, token: &str) {
        self.content.push_str(token);
        self.cursor_pos = self.content.len();
        self.token_count += 1;
    }

    /// Finish streaming
    pub fn finish(&mut self) {
        self.active = false;
    }

    /// Update animation frame for cursor blinking
    pub fn update_animation(&mut self) {
        if self.active {
            self.animation_frame = (self.animation_frame + 1) % 4;
        }
    }

    /// Get the display text with animated cursor
    pub fn display_text(&self) -> String {
        if self.active {
            // Cursor animation: show cursor every other frame
            let cursor = if self.animation_frame < 2 { "_" } else { " " };
            format!("{}{}", self.content, cursor)
        } else {
            self.content.clone()
        }
    }

    /// Get the display text with a specific cursor style
    pub fn display_text_with_cursor(&self, cursor_char: &str) -> String {
        if self.active {
            format!("{}{}", self.content, cursor_char)
        } else {
            self.content.clone()
        }
    }

    /// Check if streaming is complete
    pub fn is_complete(&self) -> bool {
        !self.active
    }

    /// Get the length of accumulated content
    pub fn len(&self) -> usize {
        self.content.len()
    }

    /// Check if content is empty
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }
}

/// Message author
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageAuthor {
    /// User message
    User,
    /// AI message
    Assistant,
}

impl Message {
    /// Create a new user message
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            author: MessageAuthor::User,
            streaming: false,
            timestamp: Local::now(),
            metadata: HashMap::new(),
            tool_calls: Vec::new(),
        }
    }

    /// Create a new assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            author: MessageAuthor::Assistant,
            streaming: false,
            timestamp: Local::now(),
            metadata: HashMap::new(),
            tool_calls: Vec::new(),
        }
    }

    /// Extract all code blocks from the message
    pub fn extract_code_blocks(&self) -> Vec<String> {
        let mut code_blocks = Vec::new();
        let mut in_code_block = false;
        let mut current_block = String::new();

        for line in self.content.lines() {
            if line.starts_with("```") {
                if in_code_block {
                    // End of code block
                    if !current_block.is_empty() {
                        code_blocks.push(current_block.clone());
                        current_block.clear();
                    }
                    in_code_block = false;
                } else {
                    // Start of code block
                    in_code_block = true;
                }
            } else if in_code_block {
                if !current_block.is_empty() {
                    current_block.push('\n');
                }
                current_block.push_str(line);
            }
        }

        code_blocks
    }

    /// Get the first code block from the message
    pub fn get_first_code_block(&self) -> Option<String> {
        self.extract_code_blocks().into_iter().next()
    }

    /// Check if message contains code blocks
    pub fn has_code_blocks(&self) -> bool {
        self.content.contains("```")
    }
}

/// Message action
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageAction {
    /// Copy message content
    Copy,
    /// Copy code block
    CopyCode,
    /// Edit message
    Edit,
    /// Regenerate response
    Regenerate,
    /// Delete message
    Delete,
}

/// Chat widget for displaying conversations
pub struct ChatWidget {
    /// Messages in the chat
    pub messages: Vec<Message>,
    /// Current input
    pub input: String,
    /// Scroll offset
    pub scroll: usize,
    /// Selected message index
    pub selected: Option<usize>,
    /// Available actions for selected message
    pub available_actions: Vec<MessageAction>,
    /// Current streaming message (if any)
    pub streaming_message: Option<StreamingMessage>,
    /// Whether streaming is currently active
    pub is_streaming: bool,
    /// Current copy operation with feedback
    pub copy_operation: Option<CopyOperation>,
    /// Action menu visibility
    pub show_action_menu: bool,
    /// Selected action in menu
    pub selected_action: Option<usize>,
    /// Reactive update subscription (for automatic UI updates)
    pub reactive_subscription: Option<tokio::sync::broadcast::Receiver<(crate::reactive_ui_updates::UpdateType, crate::StateDiff)>>,

    // Component trait fields
    /// Unique component identifier
    id: crate::ComponentId,
    /// Whether the component is focused
    focused: bool,
    /// Whether the component is visible
    visible: bool,
    /// Whether the component is enabled
    enabled: bool,
    /// Component bounds
    bounds: ratatui::layout::Rect,
}

impl ChatWidget {
    /// Create a new chat widget
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            input: String::new(),
            scroll: 0,
            selected: None,
            available_actions: Vec::new(),
            streaming_message: None,
            is_streaming: false,
            copy_operation: None,
            show_action_menu: false,
            selected_action: None,
            reactive_subscription: None,

            // Component trait fields
            id: "chat-widget".to_string(),
            focused: false,
            visible: true,
            enabled: true,
            bounds: ratatui::layout::Rect::new(0, 0, 80, 24),
        }
    }

    /// Get the current input content
    pub fn input_content(&self) -> String {
        self.input.clone()
    }

    /// Start streaming a new message
    pub fn start_streaming(&mut self) {
        self.streaming_message = Some(StreamingMessage::new());
        self.is_streaming = true;
    }

    /// Append a token to the streaming message
    pub fn append_token(&mut self, token: &str) {
        if let Some(ref mut msg) = self.streaming_message {
            msg.append(token);
        }
    }

    /// Finish streaming and convert to a regular message
    pub fn finish_streaming(&mut self) -> Option<Message> {
        if let Some(mut msg) = self.streaming_message.take() {
            msg.finish();
            let content = msg.content.clone();
            self.is_streaming = false;

            // Create a regular message from the streaming message
            let message = Message::assistant(content);
            self.messages.push(message.clone());
            return Some(message);
        }
        None
    }

    /// Update streaming animation
    pub fn update_streaming_animation(&mut self) {
        if let Some(ref mut msg) = self.streaming_message {
            msg.update_animation();
        }
    }

    /// Get the current streaming message display text
    pub fn get_streaming_display(&self) -> Option<String> {
        self.streaming_message
            .as_ref()
            .map(|msg| msg.display_text())
    }

    /// Cancel streaming
    pub fn cancel_streaming(&mut self) {
        self.streaming_message = None;
        self.is_streaming = false;
    }

    /// Add a message
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }

    /// Clear all messages
    pub fn clear(&mut self) {
        self.messages.clear();
        self.input.clear();
        self.scroll = 0;
        self.selected = None;
        self.available_actions.clear();
        self.streaming_message = None;
        self.is_streaming = false;
        self.copy_operation = None;
        self.show_action_menu = false;
        self.selected_action = None;
        // Note: reactive_subscription is preserved during clear
    }

    /// Update available actions for selected message
    pub fn update_actions(&mut self) {
        self.available_actions.clear();

        if let Some(idx) = self.selected {
            if let Some(msg) = self.messages.get(idx) {
                self.available_actions.push(MessageAction::Copy);

                if msg.content.contains("```") {
                    self.available_actions.push(MessageAction::CopyCode);
                }

                if msg.author == MessageAuthor::User {
                    self.available_actions.push(MessageAction::Edit);
                } else {
                    self.available_actions.push(MessageAction::Regenerate);
                }

                self.available_actions.push(MessageAction::Delete);
            }
        }
    }

    /// Execute an action on the selected message
    pub fn execute_action(&mut self, action: MessageAction) -> Result<(), String> {
        match action {
            MessageAction::Copy => {
                if let Some(msg) = self.selected_message() {
                    let content = msg.content.clone();
                    let mut op = CopyOperation::new(content);
                    match op.execute() {
                        Ok(()) => {
                            tracing::info!("Copied message to clipboard");
                            self.copy_operation = Some(op);
                            Ok(())
                        }
                        Err(e) => {
                            tracing::error!("Failed to copy message: {}", e);
                            Err(format!("Failed to copy: {}", e))
                        }
                    }
                } else {
                    Err("No message selected".to_string())
                }
            }
            MessageAction::CopyCode => {
                if let Some(msg) = self.selected_message() {
                    if let Some(code) = msg.get_first_code_block() {
                        let mut op = CopyOperation::new(code);
                        match op.execute() {
                            Ok(()) => {
                                tracing::info!("Copied code block to clipboard");
                                self.copy_operation = Some(op);
                                Ok(())
                            }
                            Err(e) => {
                                tracing::error!("Failed to copy code: {}", e);
                                Err(format!("Failed to copy code: {}", e))
                            }
                        }
                    } else {
                        Err("No code block found in message".to_string())
                    }
                } else {
                    Err("No message selected".to_string())
                }
            }
            MessageAction::Edit => {
                if let Some(idx) = self.selected {
                    if let Some(msg) = self.messages.get_mut(idx) {
                        if msg.author == MessageAuthor::User {
                            self.input = msg.content.clone();
                            tracing::info!("Editing message");
                            self.show_action_menu = false;
                            return Ok(());
                        }
                    }
                }
                Err("Cannot edit non-user messages".to_string())
            }
            MessageAction::Regenerate => {
                if let Some(msg) = self.selected_message() {
                    if msg.author == MessageAuthor::Assistant {
                        tracing::info!("Regenerating response");
                        self.show_action_menu = false;
                        return Ok(());
                    }
                }
                Err("Can only regenerate assistant messages".to_string())
            }
            MessageAction::Delete => {
                if let Some(idx) = self.selected {
                    self.messages.remove(idx);
                    self.selected = None;
                    self.available_actions.clear();
                    self.show_action_menu = false;
                    tracing::info!("Deleted message");
                    return Ok(());
                }
                Err("No message selected".to_string())
            }
        }
    }

    /// Get visible messages based on scroll
    pub fn visible_messages(&self, height: usize) -> Vec<&Message> {
        self.messages
            .iter()
            .skip(self.scroll)
            .take(height)
            .collect()
    }

    /// Scroll up
    pub fn scroll_up(&mut self) {
        if self.scroll > 0 {
            self.scroll -= 1;
        }
    }

    /// Scroll down
    pub fn scroll_down(&mut self, height: usize) {
        let max_scroll = self.messages.len().saturating_sub(height);
        if self.scroll < max_scroll {
            self.scroll += 1;
        }
    }

    /// Select next message
    pub fn select_next(&mut self) {
        match self.selected {
            None => self.selected = Some(0),
            Some(idx) if idx < self.messages.len() - 1 => self.selected = Some(idx + 1),
            _ => {}
        }
    }

    /// Select previous message
    pub fn select_prev(&mut self) {
        match self.selected {
            None => {}
            Some(0) => self.selected = None,
            Some(idx) => self.selected = Some(idx - 1),
        }
    }

    /// Get selected message
    pub fn selected_message(&self) -> Option<&Message> {
        self.selected.and_then(|idx| self.messages.get(idx))
    }

    /// Toggle action menu visibility
    pub fn toggle_action_menu(&mut self) {
        if self.selected.is_some() && !self.available_actions.is_empty() {
            self.show_action_menu = !self.show_action_menu;
            if self.show_action_menu {
                self.selected_action = Some(0);
            } else {
                self.selected_action = None;
            }
        }
    }

    /// Close action menu
    pub fn close_action_menu(&mut self) {
        self.show_action_menu = false;
        self.selected_action = None;
    }

    /// Navigate action menu up
    pub fn action_menu_up(&mut self) {
        if let Some(idx) = self.selected_action {
            if idx > 0 {
                self.selected_action = Some(idx - 1);
            }
        }
    }

    /// Navigate action menu down
    pub fn action_menu_down(&mut self) {
        if let Some(idx) = self.selected_action {
            if idx < self.available_actions.len() - 1 {
                self.selected_action = Some(idx + 1);
            }
        }
    }

    /// Execute action by keyboard shortcut
    pub fn execute_action_by_shortcut(&mut self, key: char) -> Result<(), String> {
        let action = match key {
            'c' | 'C' => MessageAction::Copy,
            'o' | 'O' => MessageAction::CopyCode,
            'e' | 'E' => MessageAction::Edit,
            'r' | 'R' => MessageAction::Regenerate,
            'd' | 'D' => MessageAction::Delete,
            _ => return Err(format!("Unknown shortcut: {}", key)),
        };

        if self.available_actions.contains(&action) {
            self.execute_action(action)
        } else {
            Err(format!("Action not available: {:?}", action))
        }
    }

    /// Get current action menu item
    pub fn get_selected_action(&self) -> Option<MessageAction> {
        self.selected_action
            .and_then(|idx| self.available_actions.get(idx))
            .copied()
    }

    /// Execute selected action from menu
    pub fn execute_selected_action(&mut self) -> Result<(), String> {
        if let Some(action) = self.get_selected_action() {
            self.execute_action(action)
        } else {
            Err("No action selected".to_string())
        }
    }

    /// Update copy operation feedback
    pub fn update_copy_feedback(&mut self) {
        if let Some(ref mut op) = self.copy_operation {
            op.update_feedback();
            if !op.is_feedback_visible() {
                self.copy_operation = None;
            }
        }
    }

    /// Get current copy feedback if visible
    pub fn get_copy_feedback(&self) -> Option<CopyFeedback> {
        self.copy_operation
            .as_ref()
            .and_then(|op| op.get_feedback())
    }

    /// Check if copy feedback is visible
    pub fn is_copy_feedback_visible(&self) -> bool {
        self.copy_operation
            .as_ref()
            .map(|op| op.is_feedback_visible())
            .unwrap_or(false)
    }

    /// Subscribe to reactive UI updates for automatic re-rendering
    pub fn subscribe_to_reactive_updates(&mut self, receiver: tokio::sync::broadcast::Receiver<(crate::reactive_ui_updates::UpdateType, crate::StateDiff)>) {
        self.reactive_subscription = Some(receiver);
    }

    /// Process any pending reactive updates
    pub async fn process_reactive_updates(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(ref mut rx) = self.reactive_subscription {
            // Collect all pending updates first to avoid borrow issues
            let mut updates = Vec::new();
            while let Ok(update) = rx.try_recv() {
                updates.push(update);
            }

            // Process each update
            for (update_type, diff) in updates {
                self.handle_reactive_update(update_type, diff).await?;
            }
        }
        Ok(())
    }

    /// Handle a reactive update
    async fn handle_reactive_update(&mut self, update_type: crate::reactive_ui_updates::UpdateType, diff: crate::StateDiff) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // For chat widget, react to message-related state changes
        for change in &diff.changes {
            match change {
                crate::model::StateChange::MessagesUpdated => {
                    // Messages have changed, widget will re-render automatically
                    tracing::debug!("Chat widget reacting to messages update");
                }
                crate::model::StateChange::StreamingStarted => {
                    self.start_streaming();
                }
                crate::model::StateChange::StreamingToken(token) => {
                    self.append_token(token);
                }
                crate::model::StateChange::StreamingFinished => {
                    self.finish_streaming();
                }
                _ => {
                    // Other changes don't affect chat widget directly
                }
            }
        }
        Ok(())
    }
}

impl Clone for ChatWidget {
    fn clone(&self) -> Self {
        Self {
            messages: self.messages.clone(),
            input: self.input.clone(),
            scroll: self.scroll,
            selected: self.selected,
            available_actions: self.available_actions.clone(),
            streaming_message: self.streaming_message.clone(),
            is_streaming: self.is_streaming,
            copy_operation: self.copy_operation.clone(),
            show_action_menu: self.show_action_menu,
            selected_action: self.selected_action,
            reactive_subscription: None, // Can't clone Receiver
            id: self.id.clone(),
            focused: self.focused,
            enabled: self.enabled,
            visible: self.visible,
            bounds: self.bounds,
        }
    }
}

impl std::fmt::Debug for ChatWidget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChatWidget")
            .field("messages", &self.messages.len())
            .field("input", &self.input)
            .field("scroll", &self.scroll)
            .field("selected", &self.selected)
            .field("is_streaming", &self.is_streaming)
            .field("show_action_menu", &self.show_action_menu)
            .field("focused", &self.focused)
            .field("enabled", &self.enabled)
            .finish()
    }
}

impl PartialEq for ChatWidget {
    fn eq(&self, other: &Self) -> bool {
        self.input == other.input
            && self.scroll == other.scroll
            && self.selected == other.selected
            && self.available_actions == other.available_actions
            && self.is_streaming == other.is_streaming
            && self.show_action_menu == other.show_action_menu
            && self.selected_action == other.selected_action
            && self.id == other.id
            && self.focused == other.focused
            && self.enabled == other.enabled
            && self.visible == other.visible
            && self.bounds == other.bounds
    }
}

impl crate::Component for ChatWidget {
    fn id(&self) -> crate::ComponentId {
        self.id.clone()
    }

    fn render(&self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect, _model: &crate::AppModel) {
        // Use existing render logic
        // For now, just render a placeholder - in a real implementation,
        // this would call the existing render methods
        use ratatui::widgets::{Block, Borders, Paragraph};
        use ratatui::text::Line;

        let content = vec![
            Line::from("Chat Widget - Component Architecture"),
            Line::from(format!("Messages: {}", self.messages.len())),
            Line::from(format!("Input: {}", self.input)),
            Line::from(format!("Focused: {}", self.focused)),
            Line::from(format!("Streaming: {}", self.is_streaming)),
        ];

        let block = Block::default()
            .title("Chat")
            .borders(Borders::ALL);

        let paragraph = Paragraph::new(content)
            .block(block);

        frame.render_widget(paragraph, area);
    }

    fn update(&mut self, message: &crate::AppMessage, _model: &crate::AppModel) -> bool {
        // Handle component-specific messages
        match message {
            crate::AppMessage::KeyPress(key) => {
                if self.focused {
                    // Handle keyboard input for chat widget
                    match key.code {
                        crossterm::event::KeyCode::Char(c) => {
                            self.input.push(c);
                            true
                        }
                        crossterm::event::KeyCode::Backspace => {
                            self.input.pop();
                            true
                        }
                        crossterm::event::KeyCode::Enter => {
                            if !self.input.is_empty() {
                                // Add message to chat
                                // TODO: Implement proper message handling
                                self.input.clear();
                            }
                            true
                        }
                        _ => false,
                    }
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn is_focused(&self) -> bool {
        self.focused
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
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

    fn bounds(&self) -> ratatui::layout::Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: ratatui::layout::Rect) {
        self.bounds = bounds;
    }

    fn handle_focus(&mut self, direction: crate::FocusDirection) -> crate::FocusResult {
        use crate::FocusDirection;
        match direction {
            FocusDirection::Forward | FocusDirection::Next => {
                // Chat widget handles focus internally
                crate::FocusResult::Handled
            }
            _ => crate::FocusResult::Handled,
        }
    }

    fn validate(&self) -> Result<(), String> {
        if self.scroll > self.messages.len() {
            return Err("Scroll offset exceeds message count".to_string());
        }
        if let Some(selected) = self.selected {
            if selected >= self.messages.len() {
                return Err("Selected message index out of bounds".to_string());
            }
        }
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn crate::Component> {
        Box::new(self.clone())
    }

    fn children(&self) -> Vec<&dyn crate::Component> {
        Vec::new()
    }

    fn children_mut(&mut self) -> Vec<&mut dyn crate::Component> {
        Vec::new()
    }

    fn find_child(&self, _id: &crate::ComponentId) -> Option<&dyn crate::Component> {
        None
    }

    fn find_child_mut(&mut self, _id: &crate::ComponentId) -> Option<&mut dyn crate::Component> {
        None
    }

    fn add_child(&mut self, _child: Box<dyn crate::Component>) {
        // Chat widget doesn't support children
    }

    fn remove_child(&mut self, _id: &crate::ComponentId) -> Option<Box<dyn crate::Component>> {
        None
    }

    fn z_index(&self) -> i32 {
        0 // Normal z-index for main content
    }

    fn set_z_index(&mut self, _z_index: i32) {
        // z-index is fixed for chat widget
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn tab_order(&self) -> Option<usize> {
        Some(0)
    }

    fn set_tab_order(&mut self, _order: Option<usize>) {
        // Tab order is fixed
    }
}

impl Default for ChatWidget {
    fn default() -> Self {
        Self::new()
    }
}


