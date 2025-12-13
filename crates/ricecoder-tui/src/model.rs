//! Core application model for Elm Architecture (TEA) implementation
//!
//! This module defines the immutable application state following the Model-Update-View pattern.
//! All state transitions are pure functions that return new state instances.

use crate::accessibility::{FocusManager, KeyboardNavigationManager, ScreenReaderAnnouncer};
use ricecoder_storage::TuiConfig;
use ricecoder_themes::Theme;
use crate::terminal_state::TerminalCapabilities;
use crate::widgets::ChatWidget;
use ricecoder_help::HelpDialog;

// Stub type for TUI isolation - TokenUsage moved to ricecoder-sessions
#[derive(Debug, Clone, PartialEq)]
pub struct TokenUsage {
    pub input_tokens: usize,
    pub output_tokens: usize,
    pub cached_tokens: usize,
}
use std::collections::HashMap;

/// Unique identifier for operations
pub type OperationId = String;

/// Immutable application state following Elm Architecture
/// Uses structural sharing for efficient updates and reactive state management
#[derive(Clone, Debug, PartialEq)]
pub struct AppModel {
    // UI State
    pub mode: AppMode,
    pub previous_mode: AppMode,
    pub theme: ricecoder_themes::Theme,
    pub terminal_caps: TerminalCapabilities,

    // Domain State
    pub sessions: SessionState,
    pub commands: CommandState,
    pub ui: UiState,

    // Async State
    pub pending_operations: HashMap<OperationId, PendingOperation>,
    pub subscriptions: Vec<Subscription>,
}

/// Application mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppMode {
    /// Chat mode for conversational interaction
    Chat,
    /// Command mode for executing commands
    Command,
    /// Diff mode for reviewing code changes
    Diff,
    /// Help mode
    Help,
}

impl AppMode {
    /// Get the display name for the mode
    pub fn display_name(&self) -> &'static str {
        match self {
            AppMode::Chat => "Chat",
            AppMode::Command => "Command",
            AppMode::Diff => "Diff",
            AppMode::Help => "Help",
        }
    }

    /// Get the keyboard shortcut for the mode
    pub fn shortcut(&self) -> &'static str {
        match self {
            AppMode::Chat => "Ctrl+1",
            AppMode::Command => "Ctrl+2",
            AppMode::Diff => "Ctrl+3",
            AppMode::Help => "Ctrl+4",
        }
    }
}

/// Session state
#[derive(Clone, Debug, PartialEq)]
pub struct SessionState {
    pub active_session_id: Option<String>,
    pub session_count: usize,
    pub total_tokens: TokenUsage,
}

/// Command state
#[derive(Clone, Debug, PartialEq)]
pub struct CommandState {
    pub command_history: Vec<String>,
    pub current_command: String,
    pub command_palette_visible: bool,
}

/// UI state
#[derive(Clone, Debug, PartialEq)]
pub struct UiState {
    pub focus_manager: FocusManager,
    pub keyboard_nav: KeyboardNavigationManager,
    pub screen_reader: ScreenReaderAnnouncer,
    pub chat_widget: ChatWidget,
    pub help_dialog: HelpDialog,
    pub file_picker_visible: bool,
    pub config: TuiConfig,
}

/// Pending asynchronous operation
#[derive(Clone, Debug, PartialEq)]
pub struct PendingOperation {
    pub id: OperationId,
    pub description: String,
    pub start_time: std::time::Instant,
}

/// Subscription for external events
#[derive(Clone, Debug, PartialEq)]
pub enum Subscription {
    FileWatcher,
    NetworkRequest(OperationId),
}

impl AppModel {
    /// Create initial application state
    pub fn init(
        config: TuiConfig,
        theme: Theme,
        terminal_caps: TerminalCapabilities,
    ) -> Self {
        Self {
            mode: AppMode::Chat,
            previous_mode: AppMode::Chat,
            theme,
            terminal_caps,

            sessions: SessionState {
                active_session_id: None,
                session_count: 0,
                total_tokens: TokenUsage::default(),
            },

            commands: CommandState {
                command_history: Vec::new(),
                current_command: String::new(),
                command_palette_visible: false,
            },

            ui: UiState {
                focus_manager: FocusManager::new(),
                keyboard_nav: KeyboardNavigationManager::new(),
                screen_reader: ScreenReaderAnnouncer::new(),
                chat_widget: ChatWidget::new(),
                help_dialog: HelpDialog::default_ricecoder(),
                file_picker_visible: false,
                config,
            },

            pending_operations: HashMap::new(),
            subscriptions: vec![Subscription::FileWatcher],
        }
    }

    /// Validate state integrity and constraints
    pub fn validate(&self) -> Result<(), String> {
        // Validate mode consistency
        match self.mode {
            AppMode::Chat => {
                if !self.ui.chat_widget.is_focused() && self.ui.file_picker_visible {
                    return Err("Chat mode should not have file picker visible".to_string());
                }
            }
            AppMode::Command => {
                if self.ui.chat_widget.is_focused() {
                    return Err("Command mode should not have chat focused".to_string());
                }
            }
            _ => {}
        }

        // Validate session state
        if self.sessions.session_count == 0 && self.sessions.active_session_id.is_some() {
            return Err("Cannot have active session when session count is zero".to_string());
        }

        // Validate token usage
        if self.sessions.total_tokens.input_tokens > 1_000_000 {
            return Err("Token usage exceeds reasonable limits".to_string());
        }

        Ok(())
    }

    /// Create a new state with structural sharing
    /// Only clones the parts that actually change
    pub fn with_mode(mut self, mode: AppMode) -> Self {
        self.previous_mode = self.mode;
        self.mode = mode;
        self
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn with_session_id(mut self, session_id: Option<String>) -> Self {
        self.sessions.active_session_id = session_id;
        self
    }

    pub fn with_command(mut self, command: String) -> Self {
        self.commands.current_command = command;
        self
    }

    /// Atomic state transitions with validation
    pub fn transition<F>(self, f: F) -> Result<Self, String>
    where
        F: FnOnce(Self) -> Self,
    {
        let new_state = f(self);
        new_state.validate()?;
        Ok(new_state)
    }

    /// Check if state transition is valid without applying it
    pub fn can_transition(&self, message: &AppMessage) -> bool {
        match message {
            AppMessage::ModeChanged(mode) => {
                // Prevent invalid mode transitions
                !matches!(mode, AppMode::Diff) || self.sessions.active_session_id.is_some()
            }
            AppMessage::SessionActivated(id) => {
                // Session must exist
                self.sessions.session_count > 0
            }
            _ => true,
        }
    }

    /// Get state diff for efficient rendering
    pub fn diff(&self, previous: &Self) -> StateDiff {
        let mut changes = Vec::new();

        if self.mode != previous.mode {
            changes.push(StateChange::Mode(self.mode));
        }

        if self.theme != previous.theme {
            changes.push(StateChange::Theme);
        }

        if self.sessions.active_session_id != previous.sessions.active_session_id {
            changes.push(StateChange::ActiveSession);
        }

        if self.commands.command_palette_visible != previous.commands.command_palette_visible {
            changes.push(StateChange::CommandPalette);
        }

        if self.ui.file_picker_visible != previous.ui.file_picker_visible {
            changes.push(StateChange::FilePicker);
        }

        StateDiff { changes }
    }
}

/// Messages that can update the application state
#[derive(Clone)]
pub enum AppMessage {
    // User Input
    KeyPress(crossterm::event::KeyEvent),
    MouseEvent(crossterm::event::MouseEvent),
    Resize { width: u16, height: u16 },
    Scroll { delta: isize },

    // UI Events
    ModeChanged(AppMode),
    ThemeChanged(Theme),
    FocusChanged(String),
    CommandPaletteToggled,

    // Session Events
    SessionCreated(String),
    SessionActivated(String),
    SessionClosed(String),
    TokensUpdated(TokenUsage),

    // File Events
    FileChanged(ricecoder_files::FileChangeBatch),
    FilePickerOpened,
    FilePickerClosed,

    // Command Events
    CommandExecuted(String),
    CommandCompleted(CommandResult),

    // Async Events
    OperationStarted(PendingOperation),
    OperationCompleted(OperationId),
    OperationFailed(OperationId, String),

    // System Events
    Tick,
    ExitRequested,
}

/// Result of a command execution
#[derive(Clone)]
pub struct CommandResult {
    pub command: String,
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

/// State change enumeration for efficient diffing
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum StateChange {
    Mode(AppMode),
    Theme,
    ActiveSession,
    CommandPalette,
    FilePicker,
    TokenUsage,
    SessionCount,
    MessagesUpdated,
    StreamingStarted,
    StreamingToken(String),
    StreamingFinished,
}

/// State diff for targeted re-renders
#[derive(Clone, Debug)]
pub struct StateDiff {
    pub changes: Vec<StateChange>,
}

/// Message batch for efficient processing
#[derive(Clone, Debug)]
pub struct MessageBatch {
    pub messages: Vec<AppMessage>,
    pub priority: MessagePriority,
    pub timestamp: std::time::Instant,
}

/// Message priority levels
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessagePriority {
    /// Low priority - can be delayed
    Low = 0,
    /// Normal priority - default processing
    Normal = 1,
    /// High priority - process immediately
    High = 2,
    /// Critical priority - process before all others
    Critical = 3,
}

/// Message batch processor for efficient TEA updates
pub struct MessageBatchProcessor {
    batches: std::collections::VecDeque<MessageBatch>,
    max_batch_size: usize,
    batch_timeout: std::time::Duration,
    last_process_time: std::time::Instant,
}

impl MessageBatchProcessor {
    /// Create a new message batch processor
    pub fn new() -> Self {
        Self {
            batches: std::collections::VecDeque::new(),
            max_batch_size: 10, // Process up to 10 messages at once
            batch_timeout: std::time::Duration::from_millis(16), // ~60 FPS
            last_process_time: std::time::Instant::now(),
        }
    }

    /// Add a message to be batched
    pub fn add_message(&mut self, message: AppMessage, priority: MessagePriority) {
        // Check if we should start a new batch or add to existing
        if let Some(last_batch) = self.batches.back_mut() {
            if last_batch.messages.len() < self.max_batch_size &&
               last_batch.priority == priority {
                // Add to existing batch
                last_batch.messages.push(message);
                return;
            }
        }

        // Create new batch
        let batch = MessageBatch {
            messages: vec![message],
            priority,
            timestamp: std::time::Instant::now(),
        };

        // Insert based on priority (higher priority first)
        let insert_pos = self.batches.iter()
            .position(|b| b.priority < priority)
            .unwrap_or(self.batches.len());

        self.batches.insert(insert_pos, batch);
    }

    /// Process pending message batches
    pub fn process_batches<F>(&mut self, mut processor: F) -> Vec<(AppMessage, AppModel)>
    where
        F: FnMut(&AppMessage, &AppModel) -> AppModel,
    {
        let mut results = Vec::new();
        let now = std::time::Instant::now();

        // Process high and critical priority batches immediately
        while let Some(batch) = self.batches.front() {
            if batch.priority >= MessagePriority::High {
                let batch = self.batches.pop_front().unwrap();
                let mut current_model = None;

                for message in batch.messages {
                    let new_model = if let Some(ref model) = current_model {
                        processor(&message, model)
                    } else {
                        // For the first message, we'd need the current model
                        // This is a simplified implementation
                        continue;
                    };
                    current_model = Some(new_model);
                    results.push((message, current_model.clone().unwrap()));
                }
            } else {
                break;
            }
        }

        // Process normal/low priority batches if timeout exceeded
        if now.duration_since(self.last_process_time) >= self.batch_timeout {
            while let Some(batch) = self.batches.pop_front() {
                let mut current_model = None;

                for message in batch.messages {
                    let new_model = if let Some(ref model) = current_model {
                        processor(&message, model)
                    } else {
                        continue;
                    };
                    current_model = Some(new_model);
                    results.push((message, current_model.clone().unwrap()));
                }

                // Don't process too many batches at once to avoid blocking
                if results.len() >= self.max_batch_size {
                    break;
                }
            }

            self.last_process_time = now;
        }

        results
    }

    /// Check if there are pending batches to process
    pub fn has_pending_batches(&self) -> bool {
        !self.batches.is_empty()
    }

    /// Get the number of pending messages
    pub fn pending_message_count(&self) -> usize {
        self.batches.iter().map(|b| b.messages.len()).sum()
    }

    /// Clear all pending batches
    pub fn clear(&mut self) {
        self.batches.clear();
    }

    /// Get batch statistics
    pub fn stats(&self) -> MessageBatchStats {
        let total_messages = self.pending_message_count();
        let batch_count = self.batches.len();
        let avg_batch_size = if batch_count > 0 {
            total_messages as f64 / batch_count as f64
        } else {
            0.0
        };

        MessageBatchStats {
            total_messages,
            batch_count,
            avg_batch_size,
            priority_distribution: self.priority_distribution(),
        }
    }

    fn priority_distribution(&self) -> std::collections::HashMap<MessagePriority, usize> {
        let mut dist = std::collections::HashMap::new();
        for batch in &self.batches {
            *dist.entry(batch.priority).or_insert(0) += batch.messages.len();
        }
        dist
    }
}

/// Statistics for message batch processing
#[derive(Debug, Clone)]
pub struct MessageBatchStats {
    pub total_messages: usize,
    pub batch_count: usize,
    pub avg_batch_size: f64,
    pub priority_distribution: std::collections::HashMap<MessagePriority, usize>,
}

impl StateDiff {
    /// Check if a specific change occurred
    pub fn has_change(&self, change: &StateChange) -> bool {
        self.changes.contains(change)
    }

    /// Check if mode changed
    pub fn mode_changed(&self) -> Option<AppMode> {
        self.changes.iter().find_map(|c| {
            if let StateChange::Mode(mode) = c {
                Some(*mode)
            } else {
                None
            }
        })
    }

    /// Check if UI needs full re-render
    pub fn needs_full_render(&self) -> bool {
        self.changes.iter().any(|c| {
            matches!(c, StateChange::Mode(_) | StateChange::Theme)
        })
    }
}

impl Default for AppModel {
    fn default() -> Self {
        Self::init(
            TuiConfig::default(),
            Theme::default(),
            TerminalCapabilities::default(),
        )
    }
}

