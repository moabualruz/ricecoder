//! Core application model for Elm Architecture (TEA) implementation
//!
//! This module defines the immutable application state following the Model-Update-View pattern.
//! All state transitions are pure functions that return new state instances.

use ricecoder_help::HelpDialog;
use ricecoder_storage::TuiConfig;
use ricecoder_themes::Theme;

use crate::{
    accessibility::{FocusManager, KeyboardNavigationManager, ScreenReaderAnnouncer},
    components::Component,
    terminal_state::TerminalCapabilities,
    widgets::ChatWidget,
};

// Stub type for TUI isolation - TokenUsage moved to ricecoder-sessions
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
}

/// Share permissions for session sharing
#[derive(Debug, Clone, PartialEq)]
pub enum SharePermissions {
    ReadOnly,
    ReadWrite,
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
    pub mcp: McpState,
    pub providers: ProviderState,
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
    /// Diff mode for viewing changes
    Diff,
    /// MCP mode for Model Context Protocol management
    Mcp,
    /// Provider mode for AI provider management
    Provider,
    /// Session mode for session management
    Session,
    /// Help mode for displaying help information
    Help,
}

impl AppMode {
    /// Get the display name for the mode
    pub fn display_name(&self) -> &'static str {
        match self {
            AppMode::Chat => "Chat",
            AppMode::Command => "Command",
            AppMode::Diff => "Diff",
            AppMode::Mcp => "MCP",
            AppMode::Provider => "Provider",
            AppMode::Session => "Session",
            AppMode::Help => "Help",
        }
    }

    /// Get the keyboard shortcut for the mode
    pub fn shortcut(&self) -> &'static str {
        match self {
            AppMode::Chat => "Ctrl+1",
            AppMode::Command => "Ctrl+2",
            AppMode::Diff => "Ctrl+3",
            AppMode::Mcp => "Ctrl+4",
            AppMode::Provider => "Ctrl+5",
            AppMode::Session => "Ctrl+7",
            AppMode::Help => "Ctrl+6",
        }
    }

    /// Get the next mode in the cycle
    pub fn next(&self) -> AppMode {
        match self {
            AppMode::Chat => AppMode::Command,
            AppMode::Command => AppMode::Diff,
            AppMode::Diff => AppMode::Mcp,
            AppMode::Mcp => AppMode::Provider,
            AppMode::Provider => AppMode::Session,
            AppMode::Session => AppMode::Help,
            AppMode::Help => AppMode::Chat,
        }
    }

    /// Get the previous mode in the cycle
    pub fn previous(&self) -> AppMode {
        match self {
            AppMode::Chat => AppMode::Help,
            AppMode::Command => AppMode::Chat,
            AppMode::Diff => AppMode::Command,
            AppMode::Mcp => AppMode::Diff,
            AppMode::Provider => AppMode::Mcp,
            AppMode::Session => AppMode::Provider,
            AppMode::Help => AppMode::Provider,
        }
    }
}

/// Session information for display
#[derive(Clone, Debug, PartialEq)]
pub struct SessionInfo {
    pub id: String,
    pub name: String,
    pub status: SessionStatus,
    pub created_at: u64,
    pub last_activity: u64,
    pub provider: String,
    pub token_count: u64,
    pub is_shared: bool,
}

/// Session status
#[derive(Clone, Debug, PartialEq)]
pub enum SessionStatus {
    Active,
    Paused,
    Completed,
    Failed,
}

/// Session browser state
#[derive(Clone, Debug, PartialEq)]
pub struct SessionBrowserState {
    pub sessions: Vec<SessionInfo>,
    pub selected_index: usize,
    pub filter_text: String,
    pub sort_by: SessionSortBy,
    pub view_mode: SessionViewMode,
}

/// Session sort options
#[derive(Clone, Debug, PartialEq)]
pub enum SessionSortBy {
    Name,
    Created,
    LastActivity,
    TokenCount,
}

/// Session view modes
#[derive(Clone, Debug, PartialEq)]
pub enum SessionViewMode {
    List,
    Grid,
    Details,
}

/// Session creation/editing state
#[derive(Clone, Debug, PartialEq)]
pub struct SessionEditorState {
    pub is_editing: bool,
    pub session_id: Option<String>,
    pub name: String,
    pub provider: String,
    pub description: String,
}

/// Session sharing state
#[derive(Clone, Debug, PartialEq)]
pub struct SessionSharingState {
    pub is_sharing: bool,
    pub session_id: String,
    pub share_url: Option<String>,
    pub expires_in: Option<u64>,
    pub permissions: SharePermissions,
}

/// Session state
#[derive(Clone, Debug, PartialEq)]
pub struct SessionState {
    pub active_session_id: Option<String>,
    pub session_count: usize,
    pub total_tokens: TokenUsage,
    pub browser: SessionBrowserState,
    pub editor: SessionEditorState,
    pub sharing: SessionSharingState,
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
    // pub help_dialog: HelpDialog, // Temporarily removed due to trait bounds
    pub file_picker_visible: bool,
    pub config: TuiConfig,
    pub activity_bar: crate::components::activity_bar::ActivityBarState,
    pub git_panel: crate::components::git_panel::GitPanelState,
}

/// MCP state for managing Model Context Protocol servers and tools
#[derive(Clone, Debug, PartialEq)]
pub struct McpState {
    pub servers: Vec<McpServerInfo>,
    pub available_tools: Vec<McpToolInfo>,
    pub selected_server: Option<String>,
    pub execution_history: Vec<McpExecutionRecord>,
}

/// Provider state for managing AI providers
#[derive(Clone, Debug, PartialEq)]
pub struct ProviderState {
    pub available_providers: Vec<ProviderInfo>,
    pub current_provider: Option<String>,
    pub provider_metrics: HashMap<String, ProviderMetrics>,
    pub selected_provider: Option<String>,
    pub view_mode: ProviderViewMode,
    pub filter_text: String,
}

/// Information about an AI provider
#[derive(Clone, Debug, PartialEq)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub state: ProviderConnectionState,
    pub models: Vec<String>,
    pub error_message: Option<String>,
    pub last_checked: Option<chrono::DateTime<chrono::Utc>>,
}

/// Provider connection state
#[derive(Clone, Debug, PartialEq)]
pub enum ProviderConnectionState {
    Connected,
    Disconnected,
    Error,
    Disabled,
}

/// Provider performance metrics
#[derive(Clone, Debug, PartialEq)]
pub struct ProviderMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub avg_response_time_ms: f64,
    pub error_rate: f64,
    pub total_tokens: u64,
    pub total_cost: f64,
    pub requests_per_second: f64,
    pub tokens_per_second: f64,
}

/// Provider view mode for the UI
#[derive(Clone, Debug, PartialEq)]
pub enum ProviderViewMode {
    List,
    Status,
    Performance,
    Analytics,
}

#[derive(Clone, Debug, PartialEq)]
pub struct McpServerInfo {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub enabled: bool,
    pub health_status: String,
    pub last_health_check: Option<u64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct McpToolInfo {
    pub server_name: String,
    pub tool_name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
}

#[derive(Clone, Debug, PartialEq)]
pub struct McpExecutionRecord {
    pub server: String,
    pub tool: String,
    pub parameters: serde_json::Value,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub timestamp: u64,
    pub execution_time_ms: u64,
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
    pub fn init(config: TuiConfig, theme: Theme, terminal_caps: TerminalCapabilities) -> Self {
        Self::init_with_providers(config, theme, terminal_caps, Vec::new(), None)
    }

    pub fn init_with_providers(
        config: TuiConfig,
        theme: Theme,
        terminal_caps: TerminalCapabilities,
        available_providers: Vec<ProviderInfo>,
        current_provider: Option<String>,
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
                browser: SessionBrowserState {
                    sessions: Vec::new(),
                    selected_index: 0,
                    filter_text: String::new(),
                    sort_by: SessionSortBy::LastActivity,
                    view_mode: SessionViewMode::List,
                },
                editor: SessionEditorState {
                    is_editing: false,
                    session_id: None,
                    name: String::new(),
                    provider: String::new(),
                    description: String::new(),
                },
                sharing: SessionSharingState {
                    is_sharing: false,
                    session_id: String::new(),
                    share_url: None,
                    expires_in: None,
                    permissions: SharePermissions::ReadOnly,
                },
            },

            commands: CommandState {
                command_history: Vec::new(),
                current_command: String::new(),
                command_palette_visible: false,
            },

            mcp: McpState {
                servers: Vec::new(),
                available_tools: Vec::new(),
                selected_server: None,
                execution_history: Vec::new(),
            },

            providers: ProviderState {
                available_providers,
                current_provider,
                provider_metrics: HashMap::new(),
                selected_provider: None,
                view_mode: ProviderViewMode::List,
                filter_text: String::new(),
            },

            ui: UiState {
                focus_manager: FocusManager::new(),
                keyboard_nav: KeyboardNavigationManager::new(),
                screen_reader: ScreenReaderAnnouncer::new(false),
                chat_widget: ChatWidget::new(),
                // help_dialog: HelpDialog::default_ricecoder(), // Temporarily removed
                file_picker_visible: false,
                config,
                activity_bar: crate::components::activity_bar::ActivityBarState::default(),
                git_panel: crate::components::git_panel::GitPanelState::default(),
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

    pub fn with_active_session(mut self, session_id: Option<String>) -> Self {
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
#[derive(Clone, Debug)]
pub enum AppMessage {
    // User Input
    KeyPress(crossterm::event::KeyEvent),
    MouseEvent(crossterm::event::MouseEvent),
    Resize {
        width: u16,
        height: u16,
    },
    Scroll {
        delta: isize,
    },

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

    // Chat Events
    SendMessage(String),

    // File Events
    FileSelected(String),

    // MCP Events
    McpServerAdded(String),
    McpServerRemoved(String),
    McpToolExecuted {
        server: String,
        tool: String,
        result: serde_json::Value,
    },
    McpToolExecutionFailed {
        server: String,
        tool: String,
        error: String,
    },

    // Provider Events
    ProviderSwitched(String),
    ProviderStatusUpdated {
        provider_id: String,
        status: ProviderConnectionState,
    },
    ProviderMetricsUpdated {
        provider_id: String,
        metrics: ProviderMetrics,
    },
    ProviderSelected(String),
    ProviderViewModeChanged(ProviderViewMode),
    ProviderFilterChanged(String),

    // Component Events
    ComponentMessage {
        component_id: String,
        message: String,
    },

    // System Events
    Tick,
    ExitRequested,
}

/// Result of a command execution
#[derive(Clone, Debug)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
#[derive(Debug)]
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

    /// Get the current batch timeout
    pub fn batch_timeout(&self) -> std::time::Duration {
        self.batch_timeout
    }

    /// Set the batch timeout
    pub fn set_batch_timeout(&mut self, timeout: std::time::Duration) {
        self.batch_timeout = timeout;
    }

    /// Add a message to be batched
    pub fn add_message(&mut self, message: AppMessage, priority: MessagePriority) {
        // Check if we should start a new batch or add to existing
        if let Some(last_batch) = self.batches.back_mut() {
            if last_batch.messages.len() < self.max_batch_size && last_batch.priority == priority {
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
        let insert_pos = self
            .batches
            .iter()
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
        self.changes
            .iter()
            .any(|c| matches!(c, StateChange::Mode(_) | StateChange::Theme))
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
