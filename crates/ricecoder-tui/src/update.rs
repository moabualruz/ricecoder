//! Update function for Elm Architecture (TEA) implementation
//!
//! This module contains the pure update function that handles all state transitions
//! in response to messages. All functions are pure and return new state instances.

use crate::model::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ricecoder_themes::Theme;

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
    /// Switch to a different AI provider
    SwitchProvider(String),
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
            AppMessage::SendMessage(msg) => self.handle_send_message(msg),
            AppMessage::FileSelected(path) => self.handle_file_selected(path),
            AppMessage::McpServerAdded(server) => self.handle_mcp_server_added(server),
            AppMessage::McpServerRemoved(server) => self.handle_mcp_server_removed(server),
            AppMessage::McpToolExecuted {
                server,
                tool,
                result,
            } => self.handle_mcp_tool_executed(server, tool, result),
            AppMessage::McpToolExecutionFailed {
                server,
                tool,
                error,
            } => self.handle_mcp_tool_execution_failed(server, tool, error),
            AppMessage::ProviderSwitched(provider_id) => self.handle_provider_switched(provider_id),
            AppMessage::ProviderStatusUpdated {
                provider_id,
                status,
            } => self.handle_provider_status_updated(provider_id, status),
            AppMessage::ProviderMetricsUpdated {
                provider_id,
                metrics,
            } => self.handle_provider_metrics_updated(provider_id, metrics),
            AppMessage::ProviderSelected(provider_id) => self.handle_provider_selected(provider_id),
            AppMessage::ProviderViewModeChanged(mode) => {
                self.handle_provider_view_mode_changed(mode)
            }
            AppMessage::ProviderFilterChanged(filter) => {
                self.handle_provider_filter_changed(filter)
            }
            AppMessage::ComponentMessage {
                component_id,
                message,
            } => self.handle_component_message(component_id, message),
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
            AppMode::Mcp => self.handle_mcp_key(key),
            AppMode::Session => self.handle_session_key(key),
            AppMode::Provider => self.handle_provider_key(key),
            AppMode::Help => self.handle_help_key(key),
        }
    }

    fn handle_global_keybinding(&self, key: KeyEvent) -> Option<Command> {
        // Ctrl+1-6 for mode switching
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('1') => Some(Command::SwitchMode(AppMode::Chat)),
                KeyCode::Char('2') => Some(Command::SwitchMode(AppMode::Command)),
                KeyCode::Char('3') => Some(Command::SwitchMode(AppMode::Diff)),
                KeyCode::Char('4') => Some(Command::SwitchMode(AppMode::Mcp)),
                KeyCode::Char('7') => Some(Command::SwitchMode(AppMode::Session)),
                KeyCode::Char('5') => Some(Command::SwitchMode(AppMode::Provider)),
                KeyCode::Char('6') => Some(Command::SwitchMode(AppMode::Help)),
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

    fn handle_mcp_key(self, _key: KeyEvent) -> (Self, Vec<Command>) {
        // Handle MCP-specific keybindings
        (self, vec![])
    }

    fn handle_session_key(self, key: KeyEvent) -> (Self, Vec<Command>) {
        match key.code {
            KeyCode::Char('n') => {
                // New session
                let mut new_state = self;
                new_state.sessions.editor = SessionEditorState {
                    is_editing: true,
                    session_id: None,
                    name: String::new(),
                    provider: "anthropic".to_string(), // Default provider
                    description: String::new(),
                };
                (new_state, vec![])
            }
            KeyCode::Char('e') => {
                // Edit selected session
                if let Some(selected_session) = self
                    .sessions
                    .browser
                    .sessions
                    .get(self.sessions.browser.selected_index)
                    .cloned()
                {
                    let mut new_state = self;
                    new_state.sessions.editor = SessionEditorState {
                        is_editing: true,
                        session_id: Some(selected_session.id),
                        name: selected_session.name,
                        provider: selected_session.provider,
                        description: String::new(), // Would need to load from session
                    };
                    (new_state, vec![])
                } else {
                    (self, vec![])
                }
            }
            KeyCode::Char('s') => {
                // Share selected session
                if let Some(selected_session) = self
                    .sessions
                    .browser
                    .sessions
                    .get(self.sessions.browser.selected_index)
                    .cloned()
                {
                    let mut new_state = self;
                    new_state.sessions.sharing = SessionSharingState {
                        is_sharing: true,
                        session_id: selected_session.id,
                        share_url: None,
                        expires_in: Some(3600), // 1 hour default
                        permissions: SharePermissions::ReadOnly,
                    };
                    (new_state, vec![])
                } else {
                    (self, vec![])
                }
            }
            KeyCode::Up => {
                // Navigate up in session list
                let mut new_state = self;
                if new_state.sessions.browser.selected_index > 0 {
                    new_state.sessions.browser.selected_index -= 1;
                }
                (new_state, vec![])
            }
            KeyCode::Down => {
                // Navigate down in session list
                let mut new_state = self;
                if new_state.sessions.browser.selected_index
                    < new_state.sessions.browser.sessions.len().saturating_sub(1)
                {
                    new_state.sessions.browser.selected_index += 1;
                }
                (new_state, vec![])
            }
            KeyCode::Enter => {
                // Activate selected session
                if let Some(selected_session) = self
                    .sessions
                    .browser
                    .sessions
                    .get(self.sessions.browser.selected_index)
                    .cloned()
                {
                    (self.with_active_session(Some(selected_session.id)), vec![])
                } else {
                    (self, vec![])
                }
            }
            _ => (self, vec![]),
        }
    }

    fn handle_provider_key(self, key: KeyEvent) -> (Self, Vec<Command>) {
        match key.code {
            KeyCode::Char('l') => (self.with_provider_view_mode(ProviderViewMode::List), vec![]),
            KeyCode::Char('s') => (
                self.with_provider_view_mode(ProviderViewMode::Status),
                vec![],
            ),
            KeyCode::Char('p') => (
                self.with_provider_view_mode(ProviderViewMode::Performance),
                vec![],
            ),
            KeyCode::Char('a') => (
                self.with_provider_view_mode(ProviderViewMode::Analytics),
                vec![],
            ),
            KeyCode::Up => self.select_previous_provider(),
            KeyCode::Down => self.select_next_provider(),
            KeyCode::Enter => self.switch_to_selected_provider(),
            KeyCode::Char('/') => (self, vec![Command::SwitchMode(AppMode::Command)]), // Quick command access
            _ => (self, vec![]),
        }
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
        new_state.terminal_caps.size = (width, height);
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

    fn handle_theme_change(mut self, theme: ricecoder_themes::Theme) -> (Self, Vec<Command>) {
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

    fn handle_send_message(self, _message: String) -> (Self, Vec<Command>) {
        // TODO: Implement send message handling
        (self, vec![])
    }

    fn handle_file_selected(self, _path: String) -> (Self, Vec<Command>) {
        // TODO: Implement file selected handling
        (self, vec![])
    }

    fn handle_mcp_server_added(self, server_name: String) -> (Self, Vec<Command>) {
        // TODO: Implement MCP server added handling
        (self, vec![])
    }

    fn handle_mcp_server_removed(self, server_name: String) -> (Self, Vec<Command>) {
        // TODO: Implement MCP server removed handling
        (self, vec![])
    }

    fn handle_mcp_tool_executed(
        self,
        server: String,
        tool: String,
        result: serde_json::Value,
    ) -> (Self, Vec<Command>) {
        // TODO: Implement MCP tool executed handling
        (self, vec![])
    }

    fn handle_mcp_tool_execution_failed(
        self,
        server: String,
        tool: String,
        error: String,
    ) -> (Self, Vec<Command>) {
        // TODO: Implement MCP tool execution failed handling
        (self, vec![])
    }

    fn handle_provider_switched(mut self, provider_id: String) -> (Self, Vec<Command>) {
        self.providers.current_provider = Some(provider_id.clone());
        (self, vec![Command::SwitchProvider(provider_id)])
    }

    fn handle_provider_status_updated(
        mut self,
        provider_id: String,
        status: ProviderConnectionState,
    ) -> (Self, Vec<Command>) {
        if let Some(provider) = self
            .providers
            .available_providers
            .iter_mut()
            .find(|p| p.id == provider_id)
        {
            provider.state = status;
            provider.last_checked = Some(chrono::Utc::now());
        }
        (self, vec![])
    }

    fn handle_provider_metrics_updated(
        mut self,
        provider_id: String,
        metrics: ProviderMetrics,
    ) -> (Self, Vec<Command>) {
        self.providers.provider_metrics.insert(provider_id, metrics);
        (self, vec![])
    }

    fn handle_provider_selected(mut self, provider_id: String) -> (Self, Vec<Command>) {
        self.providers.selected_provider = Some(provider_id);
        (self, vec![])
    }

    fn handle_provider_view_mode_changed(mut self, mode: ProviderViewMode) -> (Self, Vec<Command>) {
        self.providers.view_mode = mode;
        (self, vec![])
    }

    fn handle_provider_filter_changed(mut self, filter: String) -> (Self, Vec<Command>) {
        self.providers.filter_text = filter;
        (self, vec![])
    }

    fn handle_component_message(
        self,
        _component_id: String,
        _message: String,
    ) -> (Self, Vec<Command>) {
        // TODO: Implement component message handling
        (self, vec![])
    }

    // Provider helper methods
    fn with_provider_view_mode(mut self, mode: ProviderViewMode) -> Self {
        self.providers.view_mode = mode;
        self
    }

    fn select_previous_provider(self) -> (Self, Vec<Command>) {
        let filtered_providers = self.get_filtered_providers();
        if filtered_providers.is_empty() {
            return (self, vec![]);
        }

        let current_index = self
            .providers
            .selected_provider
            .as_ref()
            .and_then(|selected| filtered_providers.iter().position(|p| p.id == *selected))
            .unwrap_or(0);

        let new_index = if current_index == 0 {
            filtered_providers.len() - 1
        } else {
            current_index - 1
        };

        let new_selected = filtered_providers[new_index].id.clone();
        (self.with_selected_provider(new_selected), vec![])
    }

    fn select_next_provider(self) -> (Self, Vec<Command>) {
        let filtered_providers = self.get_filtered_providers();
        if filtered_providers.is_empty() {
            return (self, vec![]);
        }

        let current_index = self
            .providers
            .selected_provider
            .as_ref()
            .and_then(|selected| filtered_providers.iter().position(|p| p.id == *selected))
            .unwrap_or(0);

        let new_index = (current_index + 1) % filtered_providers.len();
        let new_selected = filtered_providers[new_index].id.clone();
        (self.with_selected_provider(new_selected), vec![])
    }

    fn switch_to_selected_provider(self) -> (Self, Vec<Command>) {
        if let Some(provider_id) = self.providers.selected_provider.clone() {
            (self, vec![Command::SwitchProvider(provider_id)])
        } else {
            (self, vec![])
        }
    }

    fn with_selected_provider(mut self, provider_id: String) -> Self {
        self.providers.selected_provider = Some(provider_id);
        self
    }

    fn get_filtered_providers(&self) -> Vec<&ProviderInfo> {
        let filter = &self.providers.filter_text;
        if filter.is_empty() {
            self.providers.available_providers.iter().collect()
        } else {
            self.providers
                .available_providers
                .iter()
                .filter(|p| {
                    p.name.to_lowercase().contains(&filter.to_lowercase())
                        || p.id.to_lowercase().contains(&filter.to_lowercase())
                })
                .collect()
        }
    }
}
