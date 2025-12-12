//! Update function for Elm Architecture (TEA) implementation
//!
//! This module contains the pure update function that handles all state transitions
//! in response to messages. All functions are pure and return new state instances.

use crate::model::*;
use crate::style::Theme;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Commands that produce side effects
#[derive(Clone)]
pub enum Command {
    /// Execute a shell command
    ExecuteCommand(String),
    /// Load a file
    LoadFile(std::path::PathBuf),
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
    /// Pure update function - returns new state and commands
    pub fn update(self, message: AppMessage) -> (Self, Vec<Command>) {
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

    fn handle_key_press(mut self, key: KeyEvent) -> (Self, Vec<Command>) {
        // Handle global keybindings first
        if let Some(cmd) = self.handle_global_keybinding(key) {
            return (self, vec![cmd]);
        }

        // Handle mode-specific keybindings
        match self.mode {
            AppMode::Chat => self.handle_chat_key(key),
            AppMode::Command => self.handle_command_key(key),
            AppMode::Diff => self.handle_diff_key(key),
            AppMode::Help => self.handle_help_key(key),
        }
    }

    fn handle_global_keybinding(&self, key: KeyEvent) -> Option<Command> {
        // Ctrl+1-4 for mode switching
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('1') => Some(Command::SwitchMode(AppMode::Chat)),
                KeyCode::Char('2') => Some(Command::SwitchMode(AppMode::Command)),
                KeyCode::Char('3') => Some(Command::SwitchMode(AppMode::Diff)),
                KeyCode::Char('4') => Some(Command::SwitchMode(AppMode::Help)),
                KeyCode::Char('c') => Some(Command::Exit),
                _ => None,
            }
        } else {
            None
        }
    }

    fn handle_chat_key(mut self, key: KeyEvent) -> (Self, Vec<Command>) {
        // Handle chat-specific keybindings
        match key.code {
            KeyCode::Enter => {
                // Send message
                let message = self.ui.chat_widget.input_content();
                if !message.is_empty() {
                    // Add message to chat
                    // TODO: Implement chat message sending
                    (self, vec![Command::SendMessage(message)])
                } else {
                    (self, vec![])
                }
            }
            _ => (self, vec![]),
        }
    }

    fn handle_command_key(mut self, key: KeyEvent) -> (Self, Vec<Command>) {
        // Handle command-specific keybindings
        match key.code {
            KeyCode::Enter => {
                let command = self.commands.current_command.clone();
                if !command.is_empty() {
                    let mut new_state = self;
                    new_state.commands.command_history.push(command.clone());
                    new_state.commands.current_command.clear();
                    (new_state, vec![Command::ExecuteCommand(command)])
                } else {
                    (self, vec![])
                }
            }
            KeyCode::Char(c) => {
                let mut new_state = self;
                new_state.commands.current_command.push(c);
                (new_state, vec![])
            }
            KeyCode::Backspace => {
                let mut new_state = self;
                new_state.commands.current_command.pop();
                (new_state, vec![])
            }
            _ => (self, vec![]),
        }
    }

    fn handle_diff_key(self, _key: KeyEvent) -> (Self, Vec<Command>) {
        // Handle diff-specific keybindings
        (self, vec![])
    }

    fn handle_help_key(self, _key: KeyEvent) -> (Self, Vec<Command>) {
        // Handle help-specific keybindings
        (self, vec![])
    }

    fn handle_mouse_event(self, _mouse: crossterm::event::MouseEvent) -> (Self, Vec<Command>) {
        // Handle mouse events
        (self, vec![])
    }

    fn handle_resize(mut self, width: u16, height: u16) -> (Self, Vec<Command>) {
        // Update terminal capabilities
        let mut new_state = self;
        new_state.terminal_caps.width = width;
        new_state.terminal_caps.height = height;
        (new_state, vec![])
    }

    fn handle_scroll(self, _delta: isize) -> (Self, Vec<Command>) {
        // Scroll handling is done at the App level, not in the TEA model
        // The App will handle scrolling the virtual lists directly
        (self, vec![])
    }

    fn handle_mode_change(mut self, mode: AppMode) -> (Self, Vec<Command>) {
        let mut new_state = self;
        new_state.previous_mode = new_state.mode;
        new_state.mode = mode;
        (new_state, vec![])
    }

    fn handle_theme_change(mut self, theme: Theme) -> (Self, Vec<Command>) {
        let mut new_state = self;
        new_state.theme = theme;
        (new_state, vec![])
    }

    fn handle_focus_change(self, _element: String) -> (Self, Vec<Command>) {
        // Handle focus changes
        (self, vec![])
    }

    fn handle_command_palette_toggle(mut self) -> (Self, Vec<Command>) {
        let mut new_state = self;
        new_state.commands.command_palette_visible = !new_state.commands.command_palette_visible;
        (new_state, vec![])
    }

    fn handle_session_created(mut self, id: String) -> (Self, Vec<Command>) {
        let mut new_state = self;
        new_state.sessions.session_count += 1;
        new_state.sessions.active_session_id = Some(id);
        (new_state, vec![])
    }

    fn handle_session_activated(mut self, id: String) -> (Self, Vec<Command>) {
        let mut new_state = self;
        new_state.sessions.active_session_id = Some(id);
        (new_state, vec![])
    }

    fn handle_session_closed(mut self, id: String) -> (Self, Vec<Command>) {
        let mut new_state = self;
        if new_state.sessions.active_session_id == Some(id) {
            new_state.sessions.active_session_id = None;
        }
        new_state.sessions.session_count = new_state.sessions.session_count.saturating_sub(1);
        (new_state, vec![])
    }

    fn handle_tokens_updated(mut self, usage: TokenUsage) -> (Self, Vec<Command>) {
        let mut new_state = self;
        new_state.sessions.total_tokens = usage;
        (new_state, vec![])
    }

    fn handle_file_changed(self, _batch: ricecoder_files::FileChangeBatch) -> (Self, Vec<Command>) {
        // Handle file changes
        (self, vec![])
    }

    fn handle_file_picker_opened(mut self) -> (Self, Vec<Command>) {
        let mut new_state = self;
        new_state.ui.file_picker_visible = true;
        (new_state, vec![])
    }

    fn handle_file_picker_closed(mut self) -> (Self, Vec<Command>) {
        let mut new_state = self;
        new_state.ui.file_picker_visible = false;
        (new_state, vec![])
    }

    fn handle_command_executed(mut self, command: String) -> (Self, Vec<Command>) {
        let mut new_state = self;
        new_state.commands.command_history.push(command.clone());
        (new_state, vec![Command::ExecuteCommand(command)])
    }

    fn handle_command_completed(mut self, result: CommandResult) -> (Self, Vec<Command>) {
        // Handle command completion
        (self, vec![])
    }

    fn handle_operation_started(mut self, op: PendingOperation) -> (Self, Vec<Command>) {
        let mut new_state = self;
        new_state.pending_operations.insert(op.id.clone(), op);
        (new_state, vec![])
    }

    fn handle_operation_completed(mut self, id: OperationId) -> (Self, Vec<Command>) {
        let mut new_state = self;
        new_state.pending_operations.remove(&id);
        (new_state, vec![])
    }

    fn handle_operation_failed(mut self, id: OperationId, error: String) -> (Self, Vec<Command>) {
        let mut new_state = self;
        new_state.pending_operations.remove(&id);
        // TODO: Handle error display
        (new_state, vec![])
    }

    fn handle_tick(self) -> (Self, Vec<Command>) {
        // Handle periodic updates
        (self, vec![])
    }

    fn handle_exit_requested(self) -> (Self, Vec<Command>) {
        (self, vec![Command::Exit])
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

            pending_operations: std::collections::HashMap::new(),
            subscriptions: vec![],
        }
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
    fn test_global_keybindings() {
        let model = create_test_model();

        // Test Ctrl+1 for Chat mode
        let key = KeyEvent::new(KeyCode::Char('1'), KeyModifiers::CONTROL);
        let (_, commands) = model.clone().update(AppMessage::KeyPress(key));
        assert_eq!(commands.len(), 1);
        assert!(matches!(commands[0], Command::SwitchMode(AppMode::Chat)));

        // Test Ctrl+C for exit
        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        let (_, commands) = model.update(AppMessage::KeyPress(key));
        assert_eq!(commands.len(), 1);
        assert!(matches!(commands[0], Command::Exit));
    }

    #[test]
    fn test_command_input() {
        let model = create_test_model();

        // Type a character
        let key = KeyEvent::new(KeyCode::Char('l'), KeyModifiers::empty());
        let (new_model, commands) = model.update(AppMessage::KeyPress(key));
        assert_eq!(new_model.commands.current_command, "l");
        assert!(commands.is_empty());

        // Press Enter to execute
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());
        let (final_model, commands) = new_model.update(AppMessage::KeyPress(key));
        assert_eq!(final_model.commands.current_command, "");
        assert_eq!(final_model.commands.command_history, vec!["l"]);
        assert_eq!(commands.len(), 1);
        assert!(matches!(commands[0], Command::ExecuteCommand(cmd) if cmd == "l"));
    }

    #[test]
    fn test_session_operations() {
        let model = create_test_model();

        // Create session
        let (model, _) = model.update(AppMessage::SessionCreated("new-session".to_string()));
        assert_eq!(model.sessions.session_count, 2);
        assert_eq!(model.sessions.active_session_id, Some("new-session".to_string()));

        // Close session
        let (model, _) = model.update(AppMessage::SessionClosed("new-session".to_string()));
        assert_eq!(model.sessions.session_count, 1);
        assert_eq!(model.sessions.active_session_id, None);
    }

    #[test]
    fn test_operation_lifecycle() {
        let model = create_test_model();
        let op = PendingOperation {
            id: "test-op".to_string(),
            description: "Test operation".to_string(),
            start_time: std::time::Instant::now(),
        };

        // Start operation
        let (model, _) = model.update(AppMessage::OperationStarted(op.clone()));
        assert!(model.pending_operations.contains_key("test-op"));

        // Complete operation
        let (model, _) = model.update(AppMessage::OperationCompleted("test-op".to_string()));
        assert!(!model.pending_operations.contains_key("test-op"));
    }
}