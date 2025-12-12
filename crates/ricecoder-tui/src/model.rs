//! Core application model for Elm Architecture (TEA) implementation
//!
//! This module defines the immutable application state following the Model-Update-View pattern.
//! All state transitions are pure functions that return new state instances.

use crate::accessibility::{FocusManager, KeyboardNavigationManager, ScreenReaderAnnouncer};
use ricecoder_storage::TuiConfig;
use crate::style::Theme;
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
    pub theme: Theme,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_model() -> AppModel {
        AppModel {
            mode: AppMode::Chat,
            previous_mode: AppMode::Chat,
            theme: Theme::default(),
            terminal_caps: TerminalCapabilities::default(),

            sessions: SessionState {
                active_session_id: Some("test-session".to_string()),
                session_count: 1,
                total_tokens: TokenUsage::default(),
            },

            commands: CommandState {
                command_history: vec![],
                current_command: "".to_string(),
                command_palette_visible: false,
            },

            ui: UiState {
                focus_manager: FocusManager::new(),
                keyboard_nav: KeyboardNavigationManager::new(),
                screen_reader: ScreenReaderAnnouncer::new(),
                chat_widget: ChatWidget::new(),
                help_dialog: HelpDialog::default_ricecoder(),
                file_picker_visible: false,
                config: TuiConfig::default(),
            },

            pending_operations: HashMap::new(),
            subscriptions: vec![],
        }
    }

    #[test]
    fn test_app_model_initialization() {
        let model = create_test_model();
        assert_eq!(model.mode, AppMode::Chat);
        assert!(model.validate().is_ok());
    }

    #[test]
    fn test_state_validation() {
        let mut model = create_test_model();
        // Create an invalid state
        model.sessions.session_count = 0;
        model.sessions.active_session_id = Some("invalid".to_string());

        assert!(model.validate().is_err());
    }

    #[test]
    fn test_structural_sharing() {
        let model = create_test_model();
        let new_model = model.with_mode(AppMode::Command);

        assert_eq!(new_model.mode, AppMode::Command);
        assert_eq!(new_model.previous_mode, AppMode::Chat);
        // Other fields should be shared
        assert_eq!(new_model.theme, model.theme);
    }

    #[test]
    fn test_state_diff_calculation() {
        let model1 = create_test_model();
        let model2 = model1.clone().with_mode(AppMode::Command);

        let diff = model2.diff(&model1);
        assert!(diff.has_change(&StateChange::Mode(AppMode::Command)));
        assert!(diff.mode_changed() == Some(AppMode::Command));
    }

    #[test]
    fn test_atomic_transitions() {
        let model = create_test_model();

        let result = model.transition(|m| m.with_mode(AppMode::Command));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().mode, AppMode::Command);
    }
}