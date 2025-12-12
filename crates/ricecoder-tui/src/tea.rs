//! Elm Architecture (TEA) implementation for RiceCoder TUI
//!
//! This module implements the Model-Update-View pattern for predictable,
//! immutable state management with structural sharing and reactive updates.

use crate::accessibility::{FocusManager, KeyboardNavigationManager, ScreenReaderAnnouncer};
use ricecoder_storage::TuiConfig;
use crate::event::Event;
use crate::image_integration::ImageIntegration;
use crate::integration::WidgetIntegration;
use crate::render::Renderer;
use crate::style::Theme;
use crate::terminal_state::TerminalCapabilities;
use crate::theme::ThemeManager;
use crate::widgets::ChatWidget;
use crate::session_integration::SessionIntegration;
use crate::project_bootstrap::ProjectBootstrap;
use crossterm::event::{KeyEvent, MouseEvent};
use ricecoder_files::FileChangeBatch;
use ricecoder_help::HelpDialog;
use ricecoder_sessions::TokenUsage;
use std::collections::HashMap;
use std::path::PathBuf;

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
    pub commands: TeaCommandState,
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
    /// TeaCommand mode for executing commands
    TeaCommand,
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
            AppMode::TeaCommand => "TeaCommand",
            AppMode::Diff => "Diff",
            AppMode::Help => "Help",
        }
    }

    /// Get the keyboard shortcut for the mode
    pub fn shortcut(&self) -> &'static str {
        match self {
            AppMode::Chat => "Ctrl+1",
            AppMode::TeaCommand => "Ctrl+2",
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

/// TeaCommand state
#[derive(Clone, Debug, PartialEq)]
pub struct TeaCommandState {
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

/// Messages that can update the application state
#[derive(Clone)]
pub enum AppMessage {
    // User Input
    KeyPress(KeyEvent),
    MouseEvent(MouseEvent),
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
    FileChanged(FileChangeBatch),
    FilePickerOpened,
    FilePickerClosed,

    // Command Events
    CommandExecuted(String),
    CommandCompleted(TeaCommandResult),

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
pub struct TeaCommandResult {
    pub command: String,
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

/// TeaCommands that produce side effects
#[derive(Clone)]
pub enum TeaCommand {
    /// Execute a shell command
    ExecuteTeaCommand(String),
    /// Load a file
    LoadFile(PathBuf),
    /// Save current session
    SaveSession,
    /// Load a session
    LoadSession(String),
    /// Switch theme
    SwitchTheme(String),
    /// Switch application mode
    SwitchMode(AppMode),
    /// Send a chat message
    SendMessage(String),
    /// Exit the application
    Exit,
}

impl AppModel {
    /// Create initial application state
    pub fn init(
        config: TuiConfig,
        theme_manager: &ThemeManager,
        session_integration: SessionIntegration,
        project_bootstrap: ProjectBootstrap,
        widget_integration: WidgetIntegration,
        image_integration: ImageIntegration,
        renderer: Renderer,
    ) -> Self {
        Self {
            mode: AppMode::Chat,
            previous_mode: AppMode::Chat,
            theme: theme_manager.current_theme().clone(),
            terminal_caps: TerminalCapabilities::detect(),

            sessions: SessionState {
                active_session_id: session_integration.active_session_id(),
                session_count: session_integration.session_count(),
                total_tokens: session_integration.total_tokens(),
            },

            commands: TeaCommandState {
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

    /// Pure update function - returns new state and commands
    pub fn update(self, message: AppMessage) -> (Self, Vec<TeaCommand>) {
        match message {
            AppMessage::KeyPress(key) => self.handle_key_press(key),
            AppMessage::MouseEvent(mouse) => self.handle_mouse_event(mouse),
            AppMessage::Resize { width, height } => self.handle_resize(width, height),
            AppMessage::Scroll { delta } => self.handle_scroll(delta),
            AppMessage::ModeChanged(mode) => self.handle_mode_change(mode),
            AppMessage::ThemeChanged(theme) => self.handle_theme_change(theme),
            AppMessage::FocusChanged(element) => self.handle_focus_change(element),
            AppMessage::CommandPaletteToggled => self.handle_command_palette_toggle(),
            AppMessage::SessionCreated(id) => self.handle_session_created(id),
            AppMessage::SessionActivated(id) => self.handle_session_activated(id),
            AppMessage::SessionClosed(id) => self.handle_session_closed(id),
            AppMessage::TokensUpdated(usage) => self.handle_tokens_updated(usage),
            AppMessage::FileChanged(batch) => self.handle_file_changed(batch),
            AppMessage::FilePickerOpened => self.handle_file_picker_opened(),
            AppMessage::FilePickerClosed => self.handle_file_picker_closed(),
            AppMessage::CommandExecuted(cmd) => self.handle_command_executed(cmd),
            AppMessage::CommandCompleted(result) => self.handle_command_completed(result),
            AppMessage::OperationStarted(op) => self.handle_operation_started(op),
            AppMessage::OperationCompleted(id) => self.handle_operation_completed(id),
            AppMessage::OperationFailed(id, error) => self.handle_operation_failed(id, error),
            AppMessage::Tick => self.handle_tick(),
            AppMessage::ExitRequested => self.handle_exit_requested(),
        }
    }

    // Event handlers - pure functions that return new state and commands

    fn handle_key_press(mut self, key: KeyEvent) -> (Self, Vec<TeaCommand>) {
        // Handle global keybindings first
        if let Some(cmd) = self.handle_global_keybinding(key) {
            return (self, vec![cmd]);
        }

        // Handle mode-specific keybindings
        match self.mode {
            AppMode::Chat => self.handle_chat_key(key),
            AppMode::TeaCommand => self.handle_command_key(key),
            AppMode::Diff => self.handle_diff_key(key),
            AppMode::Help => self.handle_help_key(key),
        }
    }

    fn handle_global_keybinding(&self, key: KeyEvent) -> Option<TeaCommand> {
        // Ctrl+1-4 for mode switching
        if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
            match key.code {
                crossterm::event::KeyCode::Char('1') => Some(TeaCommand::SwitchMode(AppMode::Chat)),
                crossterm::event::KeyCode::Char('2') => Some(TeaCommand::SwitchMode(AppMode::TeaCommand)),
                crossterm::event::KeyCode::Char('3') => Some(TeaCommand::SwitchMode(AppMode::Diff)),
                crossterm::event::KeyCode::Char('4') => Some(TeaCommand::SwitchMode(AppMode::Help)),
                crossterm::event::KeyCode::Char('c') => Some(TeaCommand::Exit),
                _ => None,
            }
        } else {
            None
        }
    }

    fn handle_chat_key(mut self, key: KeyEvent) -> (Self, Vec<TeaCommand>) {
        // Handle chat-specific keybindings
        match key.code {
            crossterm::event::KeyCode::Enter => {
                // Send message
                let message = self.ui.chat_widget.input_content();
                if !message.is_empty() {
                    // Add message to chat
                    // TODO: Implement chat message sending
                    (self, vec![TeaCommand::SendMessage(message)])
                } else {
                    (self, vec![])
                }
            }
            _ => (self, vec![]),
        }
    }

    fn handle_command_key(mut self, key: KeyEvent) -> (Self, Vec<TeaCommand>) {
        // Handle command-specific keybindings
        match key.code {
            crossterm::event::KeyCode::Enter => {
                let command = self.commands.current_command.clone();
                if !command.is_empty() {
                    let mut new_state = self;
                    new_state.commands.command_history.push(command.clone());
                    new_state.commands.current_command.clear();
                    (new_state, vec![TeaCommand::ExecuteTeaCommand(command)])
                } else {
                    (self, vec![])
                }
            }
            crossterm::event::KeyCode::Char(c) => {
                let mut new_state = self;
                new_state.commands.current_command.push(c);
                (new_state, vec![])
            }
            crossterm::event::KeyCode::Backspace => {
                let mut new_state = self;
                new_state.commands.current_command.pop();
                (new_state, vec![])
            }
            _ => (self, vec![]),
        }
    }

    fn handle_diff_key(self, _key: KeyEvent) -> (Self, Vec<TeaCommand>) {
        // Handle diff-specific keybindings
        (self, vec![])
    }

    fn handle_help_key(self, _key: KeyEvent) -> (Self, Vec<TeaCommand>) {
        // Handle help-specific keybindings
        (self, vec![])
    }

    fn handle_mouse_event(self, _mouse: MouseEvent) -> (Self, Vec<TeaCommand>) {
        // Handle mouse events
        (self, vec![])
    }

    fn handle_resize(mut self, width: u16, height: u16) -> (Self, Vec<TeaCommand>) {
        // Update terminal capabilities
        let mut new_state = self;
        new_state.terminal_caps.width = width;
        new_state.terminal_caps.height = height;
        (new_state, vec![])
    }

    fn handle_scroll(self, _delta: isize) -> (Self, Vec<TeaCommand>) {
        // Scroll handling is done at the App level, not in the TEA model
        // The App will handle scrolling the virtual lists directly
        (self, vec![])
    }

    fn handle_mode_change(mut self, mode: AppMode) -> (Self, Vec<TeaCommand>) {
        let mut new_state = self;
        new_state.previous_mode = new_state.mode;
        new_state.mode = mode;
        (new_state, vec![])
    }

    fn handle_theme_change(mut self, theme: Theme) -> (Self, Vec<TeaCommand>) {
        let mut new_state = self;
        new_state.theme = theme;
        (new_state, vec![])
    }

    fn handle_focus_change(self, _element: String) -> (Self, Vec<TeaCommand>) {
        // Handle focus changes
        (self, vec![])
    }

    fn handle_command_palette_toggle(mut self) -> (Self, Vec<TeaCommand>) {
        let mut new_state = self;
        new_state.commands.command_palette_visible = !new_state.commands.command_palette_visible;
        (new_state, vec![])
    }

    fn handle_session_created(mut self, id: String) -> (Self, Vec<TeaCommand>) {
        let mut new_state = self;
        new_state.sessions.session_count += 1;
        new_state.sessions.active_session_id = Some(id);
        (new_state, vec![])
    }

    fn handle_session_activated(mut self, id: String) -> (Self, Vec<TeaCommand>) {
        let mut new_state = self;
        new_state.sessions.active_session_id = Some(id);
        (new_state, vec![])
    }

    fn handle_session_closed(mut self, id: String) -> (Self, Vec<TeaCommand>) {
        let mut new_state = self;
        if new_state.sessions.active_session_id == Some(id) {
            new_state.sessions.active_session_id = None;
        }
        new_state.sessions.session_count = new_state.sessions.session_count.saturating_sub(1);
        (new_state, vec![])
    }

    fn handle_tokens_updated(mut self, usage: TokenUsage) -> (Self, Vec<TeaCommand>) {
        let mut new_state = self;
        new_state.sessions.total_tokens = usage;
        (new_state, vec![])
    }

    fn handle_file_changed(self, _batch: FileChangeBatch) -> (Self, Vec<TeaCommand>) {
        // Handle file changes
        (self, vec![])
    }

    fn handle_file_picker_opened(mut self) -> (Self, Vec<TeaCommand>) {
        let mut new_state = self;
        new_state.ui.file_picker_visible = true;
        (new_state, vec![])
    }

    fn handle_file_picker_closed(mut self) -> (Self, Vec<TeaCommand>) {
        let mut new_state = self;
        new_state.ui.file_picker_visible = false;
        (new_state, vec![])
    }

    fn handle_command_executed(mut self, command: String) -> (Self, Vec<TeaCommand>) {
        let mut new_state = self;
        new_state.commands.command_history.push(command.clone());
        (new_state, vec![TeaCommand::ExecuteTeaCommand(command)])
    }

    fn handle_command_completed(mut self, result: TeaCommandResult) -> (Self, Vec<TeaCommand>) {
        // Handle command completion
        (self, vec![])
    }

    fn handle_operation_started(mut self, op: PendingOperation) -> (Self, Vec<TeaCommand>) {
        let mut new_state = self;
        new_state.pending_operations.insert(op.id.clone(), op);
        (new_state, vec![])
    }

    fn handle_operation_completed(mut self, id: OperationId) -> (Self, Vec<TeaCommand>) {
        let mut new_state = self;
        new_state.pending_operations.remove(&id);
        (new_state, vec![])
    }

    fn handle_operation_failed(mut self, id: OperationId, error: String) -> (Self, Vec<TeaCommand>) {
        let mut new_state = self;
        new_state.pending_operations.remove(&id);
        // TODO: Handle error display
        (new_state, vec![])
    }

    fn handle_tick(self) -> (Self, Vec<TeaCommand>) {
        // Handle periodic updates
        (self, vec![])
    }

    fn handle_exit_requested(self) -> (Self, Vec<TeaCommand>) {
        (self, vec![TeaCommand::Exit])
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

/// Reactive state manager with change tracking
pub struct ReactiveState {
    current: AppModel,
    history: Vec<AppModel>,
    max_history: usize,
}

impl ReactiveState {
    pub fn new(initial_state: AppModel) -> Self {
        Self {
            current: initial_state,
            history: Vec::new(),
            max_history: 50, // Keep last 50 states for undo
        }
    }

    /// Apply a message and return the state diff
    pub fn update(&mut self, message: AppMessage) -> Result<StateDiff, String> {
        if !self.current.can_transition(&message) {
            return Err(format!("Invalid state transition: {:?}", message));
        }

        let previous = self.current.clone();
        let (new_state, _commands) = self.current.update(message);

        // Validate the new state
        new_state.validate()?;

        // Store previous state in history
        self.history.push(previous);
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }

        // Calculate diff
        let diff = new_state.diff(&self.current);

        // Update current state
        self.current = new_state;

        Ok(diff)
    }

    /// Get current state (immutable reference)
    pub fn current(&self) -> &AppModel {
        &self.current
    }

    /// Undo last change
    pub fn undo(&mut self) -> Result<StateDiff, String> {
        if let Some(previous) = self.history.pop() {
            let diff = self.current.diff(&previous);
            self.current = previous;
            Ok(diff)
        } else {
            Err("No more states to undo".to_string())
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.history.is_empty()
    }

    /// Get state at specific history index
    pub fn state_at(&self, index: usize) -> Option<&AppModel> {
        if index == 0 {
            Some(&self.current)
        } else {
            self.history.get(self.history.len().saturating_sub(index))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::ChatWidget;
    use ricecoder_help::HelpDialog;

    fn create_test_model() -> AppModel {
        AppModel {
            mode: AppMode::Chat,
            previous_mode: AppMode::Chat,
            theme: crate::style::Theme::default(),
            terminal_caps: crate::terminal_state::TerminalCapabilities::default(),

            sessions: SessionState {
                active_session_id: Some("test-session".to_string()),
                session_count: 1,
                total_tokens: ricecoder_sessions::TokenUsage::default(),
            },

            commands: TeaCommandState {
                command_history: vec![],
                current_command: "".to_string(),
                command_palette_visible: false,
            },

            ui: UiState {
                focus_manager: crate::accessibility::FocusManager::new(),
                keyboard_nav: crate::accessibility::KeyboardNavigationManager::new(),
                screen_reader: crate::accessibility::ScreenReaderAnnouncer::new(),
                chat_widget: ChatWidget::new(),
                help_dialog: HelpDialog::default_ricecoder(),
                file_picker_visible: false,
                config: TuiConfig::default(),
            },

            pending_operations: std::collections::HashMap::new(),
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
    fn test_mode_change_update() {
        let model = create_test_model();
        let (new_model, commands) = model.update(AppMessage::ModeChanged(AppMode::Command));

        assert_eq!(new_model.mode, AppMode::Command);
        assert_eq!(new_model.previous_mode, AppMode::Chat);
        assert!(commands.is_empty());
    }

    #[test]
    fn test_command_palette_toggle() {
        let model = create_test_model();
        let (new_model, commands) = model.update(AppMessage::CommandPaletteToggled);

        assert!(new_model.commands.command_palette_visible);
        assert!(commands.is_empty());

        let (final_model, _) = new_model.update(AppMessage::CommandPaletteToggled);
        assert!(!final_model.commands.command_palette_visible);
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
    fn test_reactive_state_management() {
        let model = create_test_model();
        let mut reactive = ReactiveState::new(model);

        let diff = reactive.update(AppMessage::ModeChanged(AppMode::Command)).unwrap();
        assert!(diff.has_change(&StateChange::Mode(AppMode::Command)));

        assert!(reactive.can_undo());
        let undo_diff = reactive.undo().unwrap();
        assert_eq!(reactive.current().mode, AppMode::Chat);
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

