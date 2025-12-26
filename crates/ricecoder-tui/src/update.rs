//! Update function for Elm Architecture (TEA) implementation
//!
//! This module contains the pure update function that handles all state transitions
//! in response to messages. All functions are pure and return new state instances.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ricecoder_themes::Theme;

use crate::model::*;

/// TUI Commands - all actions that can be triggered in the TUI
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    // === Existing commands ===
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
    
    // === File Operations ===
    /// Attach a file to the current session
    AttachFile(std::path::PathBuf),
    /// Remove an attachment by index
    RemoveAttachment(usize),
    /// Browse files using file picker
    BrowseFiles,
    
    // === Session Management ===
    /// Create a new session
    CreateSession,
    /// Delete a session
    DeleteSession(String),
    /// Rename a session
    RenameSession { id: String, name: String },
    /// Share a session
    ShareSession(String),
    /// Unshare a session
    UnshareSession(String),
    /// Compact a session
    CompactSession(String),
    /// Fork session from a message
    ForkSession { from_message: String },
    
    // === Navigation ===
    /// Navigate to home
    NavigateHome,
    /// Navigate to a specific session
    NavigateSession(String),
    /// Navigate back
    NavigateBack,
    /// Navigate forward
    NavigateForward,
    
    // === UI Navigation ===
    /// Toggle command palette
    ToggleCommandPalette,
    /// Toggle help dialog
    ToggleHelp,
    /// Toggle sidebar
    ToggleSidebar,
    /// Toggle thinking display
    ToggleThinking,
    /// Toggle timestamps
    ToggleTimestamps,
    /// Toggle scrollbar
    ToggleScrollbar,
    /// Toggle tool details
    ToggleToolDetails,
    /// Focus prompt input
    FocusPrompt,
    /// Focus history view
    FocusHistory,
    
    // === Messages ===
    /// Scroll up
    ScrollUp,
    /// Scroll down
    ScrollDown,
    /// Scroll page up
    ScrollPageUp,
    /// Scroll page down
    ScrollPageDown,
    /// Scroll to top
    ScrollToTop,
    /// Scroll to bottom
    ScrollToBottom,
    /// Navigate to next message
    NextMessage,
    /// Navigate to previous message
    PrevMessage,
    /// Copy last message
    CopyLastMessage,
    /// Copy session transcript
    CopySessionTranscript,
    /// Undo last message
    UndoMessage,
    /// Redo last message
    RedoMessage,
    
    // === Dialogs ===
    /// Show a dialog
    ShowDialog(DialogType),
    /// Close current dialog
    CloseDialog,
    
    // === Provider/Model ===
    /// List available providers
    ListProviders,
    /// Select a provider
    SelectProvider(String),
    /// Select a model
    SelectModel(String),
    /// Test provider connection
    TestProvider(String),
    
    // === MCP ===
    /// List MCP servers
    ListMcpServers,
    /// Toggle MCP server
    ToggleMcpServer(String),
    /// Refresh MCP servers
    RefreshMcpServers,
    
    // === Agent ===
    /// Select an agent
    SelectAgent(String),
    
    // === Toast/Notifications ===
    /// Show a toast notification
    ShowToast { message: String, variant: ToastVariant },
    /// Clear all toasts
    ClearToasts,
    
    // === History/Stash ===
    /// Navigate history up
    NavigateHistoryUp,
    /// Navigate history down
    NavigateHistoryDown,
    /// Stash current prompt
    StashPrompt,
    /// Pop from stash
    PopStash,
    /// List stash contents
    ListStash,
    
    // === Editor ===
    /// Open external editor
    OpenExternalEditor,
    /// Import from editor
    ImportFromEditor,
    
    // === Child Sessions ===
    /// Navigate to next child session
    NextChildSession,
    /// Navigate to previous child session
    PrevChildSession,
    /// Go to parent session
    GoToParentSession,
    
    // === Misc ===
    /// Interrupt current operation
    Interrupt,
    /// Refresh display
    Refresh,
    /// No operation
    Noop,
}

/// Dialog types that can be shown
#[derive(Debug, Clone, PartialEq)]
pub enum DialogType {
    /// Agent selection dialog
    Agent,
    /// Command palette
    Command,
    /// MCP server management
    Mcp,
    /// Model selection
    Model,
    /// Provider selection
    Provider,
    /// Session list
    SessionList,
    /// Session rename dialog
    SessionRename,
    /// Stash list
    Stash,
    /// Status information
    Status,
    /// Tag management
    Tag,
    /// Theme selection
    ThemeList,
    /// Timeline view
    Timeline,
    /// Fork session dialog
    Fork,
    /// Message options
    Message,
    /// Subagent selection
    Subagent,
    /// Help dialog
    Help,
    /// Confirmation dialog
    Confirm { title: String, message: String },
    /// Prompt dialog
    Prompt { title: String, placeholder: String },
    /// Alert dialog
    Alert { title: String, message: String },
}

/// Toast notification variants
#[derive(Debug, Clone, PartialEq)]
pub enum ToastVariant {
    /// Info notification
    Info,
    /// Success notification
    Success,
    /// Warning notification
    Warning,
    /// Error notification
    Error,
}

impl Command {
    /// Parse a command from a string (for command palette, keybinds)
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            // Basic
            "exit" | "quit" => Some(Self::Exit),
            "save" | "session.save" => Some(Self::SaveSession),
            "refresh" => Some(Self::Refresh),
            "interrupt" | "cancel" => Some(Self::Interrupt),
            
            // Navigation
            "home" | "navigate.home" => Some(Self::NavigateHome),
            "back" | "navigate.back" => Some(Self::NavigateBack),
            "forward" | "navigate.forward" => Some(Self::NavigateForward),
            
            // UI Toggles
            "command_palette" | "toggle.command_palette" => Some(Self::ToggleCommandPalette),
            "help" | "toggle.help" => Some(Self::ToggleHelp),
            "sidebar" | "toggle.sidebar" => Some(Self::ToggleSidebar),
            "thinking" | "toggle.thinking" => Some(Self::ToggleThinking),
            "timestamps" | "toggle.timestamps" => Some(Self::ToggleTimestamps),
            "scrollbar" | "toggle.scrollbar" => Some(Self::ToggleScrollbar),
            "tool_details" | "toggle.tool_details" => Some(Self::ToggleToolDetails),
            
            // Scroll
            "scroll.up" => Some(Self::ScrollUp),
            "scroll.down" => Some(Self::ScrollDown),
            "scroll.page_up" => Some(Self::ScrollPageUp),
            "scroll.page_down" => Some(Self::ScrollPageDown),
            "scroll.top" => Some(Self::ScrollToTop),
            "scroll.bottom" => Some(Self::ScrollToBottom),
            
            // Messages
            "message.next" => Some(Self::NextMessage),
            "message.prev" => Some(Self::PrevMessage),
            "message.copy" => Some(Self::CopyLastMessage),
            "message.undo" => Some(Self::UndoMessage),
            "message.redo" => Some(Self::RedoMessage),
            
            // Session
            "session.create" | "new" => Some(Self::CreateSession),
            "session.copy" => Some(Self::CopySessionTranscript),
            
            // Dialogs
            "dialog.agent" => Some(Self::ShowDialog(DialogType::Agent)),
            "dialog.model" => Some(Self::ShowDialog(DialogType::Model)),
            "dialog.provider" => Some(Self::ShowDialog(DialogType::Provider)),
            "dialog.mcp" => Some(Self::ShowDialog(DialogType::Mcp)),
            "dialog.sessions" => Some(Self::ShowDialog(DialogType::SessionList)),
            "dialog.themes" => Some(Self::ShowDialog(DialogType::ThemeList)),
            "dialog.stash" => Some(Self::ShowDialog(DialogType::Stash)),
            "dialog.status" => Some(Self::ShowDialog(DialogType::Status)),
            "dialog.timeline" => Some(Self::ShowDialog(DialogType::Timeline)),
            "dialog.fork" => Some(Self::ShowDialog(DialogType::Fork)),
            "dialog.message" => Some(Self::ShowDialog(DialogType::Message)),
            "dialog.subagent" => Some(Self::ShowDialog(DialogType::Subagent)),
            "dialog.close" => Some(Self::CloseDialog),
            
            // History
            "history.up" => Some(Self::NavigateHistoryUp),
            "history.down" => Some(Self::NavigateHistoryDown),
            "stash" | "stash.push" => Some(Self::StashPrompt),
            "stash.pop" => Some(Self::PopStash),
            "stash.list" => Some(Self::ListStash),
            
            // Editor
            "editor.open" => Some(Self::OpenExternalEditor),
            "editor.import" => Some(Self::ImportFromEditor),
            
            // MCP
            "mcp.list" => Some(Self::ListMcpServers),
            "mcp.refresh" => Some(Self::RefreshMcpServers),
            
            // Provider
            "provider.list" => Some(Self::ListProviders),
            
            // Files
            "files.browse" => Some(Self::BrowseFiles),
            
            // Child sessions
            "session.child.next" => Some(Self::NextChildSession),
            "session.child.prev" => Some(Self::PrevChildSession),
            "session.parent" => Some(Self::GoToParentSession),
            
            // Focus
            "focus.prompt" => Some(Self::FocusPrompt),
            "focus.history" => Some(Self::FocusHistory),
            
            // Toasts
            "toast.clear" => Some(Self::ClearToasts),
            
            _ => None,
        }
    }
    
    /// Convert command to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Exit => "exit",
            Self::SaveSession => "session.save",
            Self::Refresh => "refresh",
            Self::Interrupt => "interrupt",
            Self::NavigateHome => "navigate.home",
            Self::NavigateBack => "navigate.back",
            Self::NavigateForward => "navigate.forward",
            Self::ToggleCommandPalette => "toggle.command_palette",
            Self::ToggleHelp => "toggle.help",
            Self::ToggleSidebar => "toggle.sidebar",
            Self::ToggleThinking => "toggle.thinking",
            Self::ToggleTimestamps => "toggle.timestamps",
            Self::ToggleScrollbar => "toggle.scrollbar",
            Self::ToggleToolDetails => "toggle.tool_details",
            Self::ScrollUp => "scroll.up",
            Self::ScrollDown => "scroll.down",
            Self::ScrollPageUp => "scroll.page_up",
            Self::ScrollPageDown => "scroll.page_down",
            Self::ScrollToTop => "scroll.top",
            Self::ScrollToBottom => "scroll.bottom",
            Self::NextMessage => "message.next",
            Self::PrevMessage => "message.prev",
            Self::CopyLastMessage => "message.copy",
            Self::UndoMessage => "message.undo",
            Self::RedoMessage => "message.redo",
            Self::CreateSession => "session.create",
            Self::CopySessionTranscript => "session.copy",
            Self::CloseDialog => "dialog.close",
            Self::NavigateHistoryUp => "history.up",
            Self::NavigateHistoryDown => "history.down",
            Self::StashPrompt => "stash.push",
            Self::PopStash => "stash.pop",
            Self::ListStash => "stash.list",
            Self::OpenExternalEditor => "editor.open",
            Self::ImportFromEditor => "editor.import",
            Self::RefreshMcpServers => "mcp.refresh",
            Self::ListMcpServers => "mcp.list",
            Self::ListProviders => "provider.list",
            Self::BrowseFiles => "files.browse",
            Self::NextChildSession => "session.child.next",
            Self::PrevChildSession => "session.child.prev",
            Self::GoToParentSession => "session.parent",
            Self::FocusPrompt => "focus.prompt",
            Self::FocusHistory => "focus.history",
            Self::ClearToasts => "toast.clear",
            Self::ShowDialog(DialogType::Agent) => "dialog.agent",
            Self::ShowDialog(DialogType::Model) => "dialog.model",
            Self::ShowDialog(DialogType::Provider) => "dialog.provider",
            Self::ShowDialog(DialogType::Mcp) => "dialog.mcp",
            Self::ShowDialog(DialogType::SessionList) => "dialog.sessions",
            Self::ShowDialog(DialogType::ThemeList) => "dialog.themes",
            Self::ShowDialog(DialogType::Stash) => "dialog.stash",
            Self::ShowDialog(DialogType::Status) => "dialog.status",
            Self::ShowDialog(DialogType::Timeline) => "dialog.timeline",
            Self::ShowDialog(DialogType::Fork) => "dialog.fork",
            Self::ShowDialog(DialogType::Message) => "dialog.message",
            Self::ShowDialog(DialogType::Subagent) => "dialog.subagent",
            Self::ShowDialog(DialogType::Help) => "dialog.help",
            _ => "unknown",
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_command_from_str_basic() {
        assert_eq!(Command::from_str("exit"), Some(Command::Exit));
        assert_eq!(Command::from_str("quit"), Some(Command::Exit));
        assert_eq!(Command::from_str("save"), Some(Command::SaveSession));
        assert_eq!(Command::from_str("session.save"), Some(Command::SaveSession));
        assert_eq!(Command::from_str("refresh"), Some(Command::Refresh));
        assert_eq!(Command::from_str("interrupt"), Some(Command::Interrupt));
        assert_eq!(Command::from_str("cancel"), Some(Command::Interrupt));
    }
    
    #[test]
    fn test_command_from_str_navigation() {
        assert_eq!(Command::from_str("home"), Some(Command::NavigateHome));
        assert_eq!(Command::from_str("navigate.home"), Some(Command::NavigateHome));
        assert_eq!(Command::from_str("back"), Some(Command::NavigateBack));
        assert_eq!(Command::from_str("navigate.back"), Some(Command::NavigateBack));
        assert_eq!(Command::from_str("forward"), Some(Command::NavigateForward));
        assert_eq!(Command::from_str("navigate.forward"), Some(Command::NavigateForward));
    }
    
    #[test]
    fn test_command_from_str_toggles() {
        assert_eq!(Command::from_str("command_palette"), Some(Command::ToggleCommandPalette));
        assert_eq!(Command::from_str("toggle.command_palette"), Some(Command::ToggleCommandPalette));
        assert_eq!(Command::from_str("help"), Some(Command::ToggleHelp));
        assert_eq!(Command::from_str("toggle.help"), Some(Command::ToggleHelp));
        assert_eq!(Command::from_str("sidebar"), Some(Command::ToggleSidebar));
        assert_eq!(Command::from_str("thinking"), Some(Command::ToggleThinking));
        assert_eq!(Command::from_str("timestamps"), Some(Command::ToggleTimestamps));
        assert_eq!(Command::from_str("scrollbar"), Some(Command::ToggleScrollbar));
        assert_eq!(Command::from_str("tool_details"), Some(Command::ToggleToolDetails));
    }
    
    #[test]
    fn test_command_from_str_scroll() {
        assert_eq!(Command::from_str("scroll.up"), Some(Command::ScrollUp));
        assert_eq!(Command::from_str("scroll.down"), Some(Command::ScrollDown));
        assert_eq!(Command::from_str("scroll.page_up"), Some(Command::ScrollPageUp));
        assert_eq!(Command::from_str("scroll.page_down"), Some(Command::ScrollPageDown));
        assert_eq!(Command::from_str("scroll.top"), Some(Command::ScrollToTop));
        assert_eq!(Command::from_str("scroll.bottom"), Some(Command::ScrollToBottom));
    }
    
    #[test]
    fn test_command_from_str_messages() {
        assert_eq!(Command::from_str("message.next"), Some(Command::NextMessage));
        assert_eq!(Command::from_str("message.prev"), Some(Command::PrevMessage));
        assert_eq!(Command::from_str("message.copy"), Some(Command::CopyLastMessage));
        assert_eq!(Command::from_str("message.undo"), Some(Command::UndoMessage));
        assert_eq!(Command::from_str("message.redo"), Some(Command::RedoMessage));
    }
    
    #[test]
    fn test_command_from_str_session() {
        assert_eq!(Command::from_str("session.create"), Some(Command::CreateSession));
        assert_eq!(Command::from_str("new"), Some(Command::CreateSession));
        assert_eq!(Command::from_str("session.copy"), Some(Command::CopySessionTranscript));
    }
    
    #[test]
    fn test_command_from_str_dialogs() {
        assert_eq!(Command::from_str("dialog.agent"), Some(Command::ShowDialog(DialogType::Agent)));
        assert_eq!(Command::from_str("dialog.model"), Some(Command::ShowDialog(DialogType::Model)));
        assert_eq!(Command::from_str("dialog.provider"), Some(Command::ShowDialog(DialogType::Provider)));
        assert_eq!(Command::from_str("dialog.mcp"), Some(Command::ShowDialog(DialogType::Mcp)));
        assert_eq!(Command::from_str("dialog.sessions"), Some(Command::ShowDialog(DialogType::SessionList)));
        assert_eq!(Command::from_str("dialog.themes"), Some(Command::ShowDialog(DialogType::ThemeList)));
        assert_eq!(Command::from_str("dialog.stash"), Some(Command::ShowDialog(DialogType::Stash)));
        assert_eq!(Command::from_str("dialog.status"), Some(Command::ShowDialog(DialogType::Status)));
        assert_eq!(Command::from_str("dialog.timeline"), Some(Command::ShowDialog(DialogType::Timeline)));
        assert_eq!(Command::from_str("dialog.close"), Some(Command::CloseDialog));
    }
    
    #[test]
    fn test_command_from_str_history() {
        assert_eq!(Command::from_str("history.up"), Some(Command::NavigateHistoryUp));
        assert_eq!(Command::from_str("history.down"), Some(Command::NavigateHistoryDown));
        assert_eq!(Command::from_str("stash"), Some(Command::StashPrompt));
        assert_eq!(Command::from_str("stash.push"), Some(Command::StashPrompt));
        assert_eq!(Command::from_str("stash.pop"), Some(Command::PopStash));
        assert_eq!(Command::from_str("stash.list"), Some(Command::ListStash));
    }
    
    #[test]
    fn test_command_from_str_editor() {
        assert_eq!(Command::from_str("editor.open"), Some(Command::OpenExternalEditor));
        assert_eq!(Command::from_str("editor.import"), Some(Command::ImportFromEditor));
    }
    
    #[test]
    fn test_command_from_str_mcp() {
        assert_eq!(Command::from_str("mcp.list"), Some(Command::ListMcpServers));
        assert_eq!(Command::from_str("mcp.refresh"), Some(Command::RefreshMcpServers));
    }
    
    #[test]
    fn test_command_from_str_child_sessions() {
        assert_eq!(Command::from_str("session.child.next"), Some(Command::NextChildSession));
        assert_eq!(Command::from_str("session.child.prev"), Some(Command::PrevChildSession));
        assert_eq!(Command::from_str("session.parent"), Some(Command::GoToParentSession));
    }
    
    #[test]
    fn test_command_from_str_focus() {
        assert_eq!(Command::from_str("focus.prompt"), Some(Command::FocusPrompt));
        assert_eq!(Command::from_str("focus.history"), Some(Command::FocusHistory));
    }
    
    #[test]
    fn test_command_from_str_invalid() {
        assert_eq!(Command::from_str("invalid"), None);
        assert_eq!(Command::from_str(""), None);
        assert_eq!(Command::from_str("random.command"), None);
    }
    
    #[test]
    fn test_command_as_str() {
        assert_eq!(Command::Exit.as_str(), "exit");
        assert_eq!(Command::SaveSession.as_str(), "session.save");
        assert_eq!(Command::ToggleHelp.as_str(), "toggle.help");
        assert_eq!(Command::ScrollUp.as_str(), "scroll.up");
        assert_eq!(Command::CreateSession.as_str(), "session.create");
        assert_eq!(Command::ShowDialog(DialogType::Agent).as_str(), "dialog.agent");
        assert_eq!(Command::NavigateHistoryUp.as_str(), "history.up");
    }
    
    #[test]
    fn test_command_roundtrip() {
        let commands = vec![
            Command::Exit,
            Command::SaveSession,
            Command::Refresh,
            Command::Interrupt,
            Command::NavigateHome,
            Command::NavigateBack,
            Command::NavigateForward,
            Command::ToggleHelp,
            Command::ToggleCommandPalette,
            Command::ToggleSidebar,
            Command::ToggleThinking,
            Command::ToggleTimestamps,
            Command::ToggleScrollbar,
            Command::ToggleToolDetails,
            Command::ScrollUp,
            Command::ScrollDown,
            Command::ScrollPageUp,
            Command::ScrollPageDown,
            Command::ScrollToTop,
            Command::ScrollToBottom,
            Command::NextMessage,
            Command::PrevMessage,
            Command::CopyLastMessage,
            Command::UndoMessage,
            Command::RedoMessage,
            Command::CreateSession,
            Command::CopySessionTranscript,
            Command::ShowDialog(DialogType::Agent),
            Command::ShowDialog(DialogType::Model),
            Command::ShowDialog(DialogType::Provider),
            Command::ShowDialog(DialogType::Mcp),
            Command::ShowDialog(DialogType::SessionList),
            Command::ShowDialog(DialogType::ThemeList),
            Command::ShowDialog(DialogType::Stash),
            Command::ShowDialog(DialogType::Status),
            Command::ShowDialog(DialogType::Timeline),
            Command::CloseDialog,
            Command::NavigateHistoryUp,
            Command::NavigateHistoryDown,
            Command::StashPrompt,
            Command::PopStash,
            Command::ListStash,
            Command::OpenExternalEditor,
            Command::ImportFromEditor,
            Command::RefreshMcpServers,
            Command::ListMcpServers,
            Command::ListProviders,
            Command::BrowseFiles,
            Command::NextChildSession,
            Command::PrevChildSession,
            Command::GoToParentSession,
            Command::FocusPrompt,
            Command::FocusHistory,
            Command::ClearToasts,
        ];
        
        for cmd in commands {
            let s = cmd.as_str();
            let parsed = Command::from_str(s);
            assert_eq!(parsed, Some(cmd.clone()), "Roundtrip failed for command: {:?}", cmd);
        }
    }
    
    #[test]
    fn test_command_aliases() {
        // Test that aliases map to the same command
        assert_eq!(Command::from_str("exit"), Command::from_str("quit"));
        assert_eq!(Command::from_str("save"), Command::from_str("session.save"));
        assert_eq!(Command::from_str("interrupt"), Command::from_str("cancel"));
        assert_eq!(Command::from_str("home"), Command::from_str("navigate.home"));
        assert_eq!(Command::from_str("back"), Command::from_str("navigate.back"));
        assert_eq!(Command::from_str("help"), Command::from_str("toggle.help"));
        assert_eq!(Command::from_str("stash"), Command::from_str("stash.push"));
        assert_eq!(Command::from_str("new"), Command::from_str("session.create"));
    }
    
    #[test]
    fn test_dialog_type_variants() {
        let dialogs = vec![
            DialogType::Agent,
            DialogType::Command,
            DialogType::Mcp,
            DialogType::Model,
            DialogType::Provider,
            DialogType::SessionList,
            DialogType::SessionRename,
            DialogType::Stash,
            DialogType::Status,
            DialogType::Tag,
            DialogType::ThemeList,
            DialogType::Timeline,
            DialogType::Fork,
            DialogType::Message,
            DialogType::Subagent,
            DialogType::Help,
        ];
        
        // Verify they all implement PartialEq correctly
        for dialog in &dialogs {
            assert_eq!(dialog, dialog);
        }
    }
    
    #[test]
    fn test_toast_variant_variants() {
        let toasts = vec![
            ToastVariant::Info,
            ToastVariant::Success,
            ToastVariant::Warning,
            ToastVariant::Error,
        ];
        
        // Verify they all implement PartialEq correctly
        for toast in &toasts {
            assert_eq!(toast, toast);
        }
    }
}
