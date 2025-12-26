//! TUI module organization
//!
//! This module provides high-level organization for TUI components and widgets.
//! Implements OpenCode-style TUI with proper routing between Home and Session views.
//!
//! Architecture follows OpenCode's pattern:
//! - AppContext: Central state management wired to backend crates
//! - Route-based rendering: Home and Session views
//! - Real-time sync with sessions, providers, MCP status

pub mod app_context;
pub mod border;
pub mod context;
pub mod did_you_know;
pub mod keybind_bridge;
pub mod prompt;
pub mod routes;
pub mod todo_item;

pub use app_context::{
    AgentInfo, AppContext, McpConnectionStatus, McpServerStatus, ModelDisplayInfo,
    ProviderInfo, SessionSummary, SyncStatus,
    AppState as BackendAppState,  // Renamed to avoid conflict with TUI AppState
};
pub use border::{SplitBorder, EMPTY_BORDER};
pub use context::{
    Args, ArgsProvider, LazyProvider, LocalProvider, PromptRef, PromptRefProvider, SdkProvider,
    SimpleProvider,
};
pub use did_you_know::DidYouKnow;
pub use routes::{
    // Home route
    Home, HomeState, HomeTheme, HomeView, McpStatus,
    // Session dialogs
    DialogFork, DialogMessage, DialogSubagent, DialogTimeline, MessageAction, SubagentAction,
    // Session components
    ItemStatus, KeybindHint, SessionFooter, SessionFooterTheme, SessionHeader, SessionHeaderTheme,
    SessionSidebar, SidebarItem, SidebarSection, SidebarTheme,
};
pub use todo_item::{TodoItem, TodoStatus};

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io::{self, Stdout};
use std::time::Duration;

use ricecoder_storage::TuiConfig;
use ricecoder_themes::Theme;

/// Current route in the TUI
#[derive(Debug, Clone, PartialEq)]
pub enum Route {
    /// Home screen - shows logo and prompt
    Home,
    /// Active session with chat
    Session { session_id: String },
}

impl Default for Route {
    fn default() -> Self {
        Route::Home
    }
}

/// Application state for TUI (local UI state)
#[derive(Debug, Clone)]
pub struct TuiState {
    /// Current route
    pub route: Route,
    /// Home screen state
    pub home: HomeState,
    /// Current prompt input
    pub prompt_input: String,
    /// Command palette visible
    pub command_palette_visible: bool,
    /// MCP server status
    pub mcp_status: McpStatus,
    /// Current provider info
    pub current_provider: Option<String>,
    /// Current model info
    pub current_model: Option<String>,
    /// Current agent
    pub current_agent: String,
    /// Session status (idle, running, retry)
    pub session_status: SessionStatus,
    /// Messages in current session
    pub messages: Vec<ChatMessage>,
    /// Current theme
    pub theme: Theme,
    /// Token usage display string (e.g., "12.5K tokens | $0.03")
    pub token_display: String,
}

/// Session status
#[derive(Debug, Clone, PartialEq)]
pub enum SessionStatus {
    Idle,
    Running,
    Retry { attempt: u8, message: String },
}

impl Default for SessionStatus {
    fn default() -> Self {
        SessionStatus::Idle
    }
}

/// Chat message
#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: std::time::Instant,
    /// Whether the message is still streaming
    pub is_streaming: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageRole {
    User,
    Assistant,
}

impl Default for TuiState {
    fn default() -> Self {
        Self {
            route: Route::Home,
            home: HomeState::default(),
            prompt_input: String::new(),
            command_palette_visible: false,
            mcp_status: McpStatus::default(),
            current_provider: None,
            current_model: None,
            current_agent: "build".to_string(),
            session_status: SessionStatus::Idle,
            messages: Vec::new(),
            theme: Theme::default(), // Default dark theme
            token_display: String::new(),
        }
    }
}

/// OpenCode-style ASCII logo for RiceCoder
const LOGO_LEFT: [&str; 4] = [
    "                   ",
    "█▀▀█ ▀█▀ █▀▀█ █▀▀▀",
    "█▄▄▀  █  █    █▀▀▀",
    "▀  ▀ ▀▀▀ ▀▀▀▀ ▀▀▀▀",
];

const LOGO_RIGHT: [&str; 4] = [
    "             ▄     ",
    "█▀▀▀ █▀▀█ █▀▀█ █▀▀█",
    "█    █  █ █  █ █▀▀▀",
    "▀▀▀▀ ▀▀▀▀ ▀▀▀▀ ▀▀▀▀",
];

/// Placeholders for the prompt
const PLACEHOLDERS: [&str; 5] = [
    "Fix a TODO in the codebase",
    "What is the tech stack of this project?",
    "Fix broken tests",
    "Refactor this function to be more readable",
    "Add error handling to this module",
];

/// TUI Application - OpenCode-style interface for RiceCoder
///
/// Provides a beautiful terminal UI with:
/// - Home screen with centered logo and prompt
/// - Session view with chat messages
/// - Command palette for quick actions
/// - MCP server status
pub struct TuiApp {
    /// Terminal instance
    terminal: Terminal<CrosstermBackend<Stdout>>,
    /// Application state (local UI state)
    state: TuiState,
    /// Backend application context (wired to backend crates)
    app_context: std::sync::Arc<AppContext>,
    /// Whether the app should continue running
    running: bool,
    /// Placeholder index for prompt
    placeholder_idx: usize,
}

impl TuiApp {
    /// Create a new TUI application
    pub fn new() -> Result<Self> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        // Initialize random placeholder
        let placeholder_idx = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as usize % PLACEHOLDERS.len())
            .unwrap_or(0);

        // Create backend application context
        let app_context = std::sync::Arc::new(AppContext::new());

        Ok(Self {
            terminal,
            state: TuiState::default(),
            app_context,
            running: true,
            placeholder_idx,
        })
    }

    /// Run the TUI event loop
    pub async fn run(&mut self) -> Result<()> {
        // Initialize backend context (load providers, sessions, MCP, etc.)
        if let Err(e) = self.app_context.initialize().await {
            tracing::warn!("Failed to initialize app context: {}", e);
            // Continue anyway - we'll show defaults
        }

        // Sync initial state from backend
        self.sync_state_from_context().await;

        while self.running {
            // Capture state snapshot for rendering (avoids borrow conflict)
            let state = self.state.clone();
            let placeholder_idx = self.placeholder_idx;
            
            // Draw the UI with captured state
            self.terminal.draw(|frame| {
                render_ui(frame, &state, placeholder_idx);
            })?;

            // Handle events
            if event::poll(Duration::from_millis(50))? {
                match event::read()? {
                    Event::Key(key) if key.kind == KeyEventKind::Press => {
                        self.handle_key(key.code, key.modifiers).await;
                    }
                    Event::Resize(_, _) => {
                        // Terminal will auto-redraw
                    }
                    Event::Mouse(mouse) => {
                        self.handle_mouse(mouse);
                    }
                    _ => {}
                }
            }

            tokio::task::yield_now().await;
        }

        Ok(())
    }

    /// Sync TuiState from AppContext backend state
    async fn sync_state_from_context(&mut self) {
        let backend_state = self.app_context.state.read().await;
        
        // Sync provider/model info
        self.state.current_provider = backend_state.current_provider_id.clone();
        self.state.current_model = backend_state.current_model_id.clone();
        self.state.current_agent = backend_state.current_agent.clone();
        
        // Sync MCP status
        let connected_count = backend_state.connected_mcp_count();
        let has_errors = backend_state.has_mcp_errors();
        self.state.mcp_status = McpStatus {
            connected: connected_count,
            total: backend_state.mcp_servers.len(),
            has_errors,
        };
        
        // Sync directory and version
        self.state.home.directory = backend_state.directory.clone();
        self.state.home.version = backend_state.version.clone();
        
        // Sync token usage display
        self.state.token_display = backend_state.token_display();
        
        // VCS branch is tracked in backend_state.vcs_branch but not exposed in HomeState
        // This can be used for status bar display in the future
    }
}

/// Standalone render function - avoids borrow conflicts with terminal
fn render_ui(frame: &mut Frame, state: &TuiState, placeholder_idx: usize) {
    let area = frame.area();

    match &state.route {
        Route::Home => render_home(frame, area, state, placeholder_idx),
        Route::Session { .. } => render_session(frame, area, state),
    }

    // Render command palette overlay if visible
    if state.command_palette_visible {
        render_command_palette(frame, area, state);
    }
}

// ===== Standalone Render Functions =====
// These are standalone to avoid borrow conflicts with terminal.draw()

/// Render home screen (OpenCode style)
fn render_home(frame: &mut Frame, area: Rect, state: &TuiState, placeholder_idx: usize) {
    // Background - use theme color
    let bg = Block::default().style(Style::default().bg(state.theme.background));
    frame.render_widget(bg, area);

    // Layout: flex-grow center area + footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10), // Main content (centered)
            Constraint::Length(2), // Footer
        ])
        .split(area);

    // Render centered logo and prompt
    render_home_content(frame, chunks[0], state, placeholder_idx);

    // Render footer
    render_home_footer(frame, chunks[1], state);
}

/// Render home content (logo + prompt)
fn render_home_content(frame: &mut Frame, area: Rect, state: &TuiState, placeholder_idx: usize) {
    // Calculate vertical center
    let logo_height = 4;
    let prompt_height = 5;
    let gap = 2;
    let total_height = logo_height + gap + prompt_height;

    let start_y = if area.height > total_height {
        area.y + (area.height - total_height) / 2
    } else {
        area.y
    };

    // Render logo
    let logo_area = Rect {
        x: area.x,
        y: start_y,
        width: area.width,
        height: logo_height,
    };
    render_logo(frame, logo_area, &state.theme);

    // Render MCP hint if connected
    if state.mcp_status.has_servers() {
        let hint_y = start_y + logo_height + 1;
        let hint_area = Rect {
            x: area.x,
            y: hint_y,
            width: area.width,
            height: 1,
        };
        render_mcp_hint(frame, hint_area, state);
    }

    // Render prompt
    let prompt_y = start_y + logo_height + gap;
    let prompt_area = Rect {
        x: area.x,
        y: prompt_y,
        width: area.width,
        height: prompt_height,
    };
    render_prompt(frame, prompt_area, state, placeholder_idx);
}

/// Render ASCII logo (OpenCode style)
fn render_logo(frame: &mut Frame, area: Rect, theme: &Theme) {
    let mut lines = Vec::new();

    for i in 0..4 {
        let left = Span::styled(LOGO_LEFT[i], Style::default().fg(theme.text_muted));
        let right = Span::styled(
            LOGO_RIGHT[i],
            Style::default().fg(theme.foreground).add_modifier(Modifier::BOLD),
        );
        lines.push(Line::from(vec![left, Span::raw(" "), right]));
    }

    let logo = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(logo, area);
}

/// Render MCP status hint
fn render_mcp_hint(frame: &mut Frame, area: Rect, state: &TuiState) {
    let (dot, color) = if state.mcp_status.has_errors {
        ("•", state.theme.error)
    } else {
        ("•", state.theme.success)
    };

    let text = Line::from(vec![
        Span::styled(format!("{} ", dot), Style::default().fg(color)),
        Span::styled(
            state.mcp_status.display_text(),
            Style::default().fg(state.theme.foreground),
        ),
    ]);

    let para = Paragraph::new(text).alignment(Alignment::Center);
    frame.render_widget(para, area);
}

/// Render prompt input (OpenCode style)
fn render_prompt(frame: &mut Frame, area: Rect, state: &TuiState, placeholder_idx: usize) {
    // Center the prompt with max width
    let max_width = 75.min(area.width.saturating_sub(4));
    let x_offset = (area.width.saturating_sub(max_width)) / 2;

    let prompt_area = Rect {
        x: area.x + x_offset,
        y: area.y,
        width: max_width,
        height: area.height,
    };

    // Agent color from theme
    let agent_color = state.theme.agent_colors.get(&state.current_agent);

    // Prompt box with left border
    let block = Block::default()
        .borders(Borders::LEFT)
        .border_style(Style::default().fg(agent_color))
        .style(Style::default().bg(state.theme.background_panel));

    let inner = block.inner(prompt_area);
    frame.render_widget(block, prompt_area);

    // Split inner into input area and status line
    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1), // Input
            Constraint::Length(1), // Status
        ])
        .margin(1)
        .split(inner);

    // Input text or placeholder
    let input_text = if state.prompt_input.is_empty() {
        Paragraph::new(format!(
            "Ask anything... \"{}\"",
            PLACEHOLDERS[placeholder_idx]
        ))
        .style(Style::default().fg(state.theme.text_muted))
    } else {
        Paragraph::new(state.prompt_input.as_str())
            .style(Style::default().fg(state.theme.foreground))
    };
    frame.render_widget(input_text, inner_chunks[0]);

    // Status line: Agent | Model | Provider
    let status = Line::from(vec![
        Span::styled(
            format!("{} ", state.current_agent),
            Style::default().fg(agent_color),
        ),
        Span::styled(
            state.current_model.as_deref().unwrap_or("(no model)"),
            Style::default().fg(state.theme.foreground),
        ),
        Span::raw(" "),
        Span::styled(
            state.current_provider.as_deref().unwrap_or(""),
            Style::default().fg(state.theme.text_muted),
        ),
    ]);
    frame.render_widget(Paragraph::new(status), inner_chunks[1]);
}

/// Render home footer
fn render_home_footer(frame: &mut Frame, area: Rect, state: &TuiState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
        ])
        .margin(1)
        .split(area);

    // Directory (left)
    let dir = Paragraph::new(state.home.directory.as_str())
        .style(Style::default().fg(state.theme.text_muted));
    frame.render_widget(dir, chunks[0]);

    // MCP status (center)
    if state.mcp_status.has_servers() {
        let mcp = Line::from(vec![
            Span::styled(
                "⊙ ",
                Style::default().fg(if state.mcp_status.has_errors {
                    state.theme.error
                } else {
                    state.theme.success
                }),
            ),
            Span::styled(
                format!("{} MCP", state.mcp_status.connected),
                Style::default().fg(state.theme.foreground),
            ),
            Span::styled(" /status", Style::default().fg(state.theme.text_muted)),
        ]);
        frame.render_widget(Paragraph::new(mcp).alignment(Alignment::Center), chunks[1]);
    }

    // Version (right)
    let version = Paragraph::new(format!("v{}", state.home.version))
        .style(Style::default().fg(state.theme.text_muted))
        .alignment(Alignment::Right);
    frame.render_widget(version, chunks[2]);
}

/// Render session view with chat
fn render_session(frame: &mut Frame, area: Rect, state: &TuiState) {
    // Layout: header + messages + prompt + footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // Header
            Constraint::Min(5),     // Messages
            Constraint::Length(5),  // Prompt
            Constraint::Length(1),  // Footer
        ])
        .split(area);

    // Header
    render_session_header(frame, chunks[0], state);

    // Messages area
    render_messages(frame, chunks[1], state);

    // Prompt (reuse placeholder_idx=0 for session)
    render_prompt(frame, chunks[2], state, 0);

    // Footer
    render_session_footer(frame, chunks[3], state);
}

/// Render session header
fn render_session_header(frame: &mut Frame, area: Rect, state: &TuiState) {
    let title = match &state.route {
        Route::Session { session_id } => format!("Session: {}", &session_id[..8.min(session_id.len())]),
        _ => "Session".to_string(),
    };

    // Build header line with title, model, and token usage
    let agent_color = state.theme.agent_colors.get(&state.current_agent);
    let mut header_spans = vec![
        Span::styled(title, Style::default().fg(agent_color).add_modifier(Modifier::BOLD)),
    ];
    
    // Add model info
    if let Some(model) = &state.current_model {
        header_spans.push(Span::styled("  |  ", Style::default().fg(state.theme.text_muted)));
        header_spans.push(Span::styled(model.clone(), Style::default().fg(state.theme.foreground)));
    }
    
    // Add token usage if available
    if !state.token_display.is_empty() {
        header_spans.push(Span::styled("  |  ", Style::default().fg(state.theme.text_muted)));
        header_spans.push(Span::styled(&state.token_display, Style::default().fg(state.theme.success)));
    }

    let header = Paragraph::new(Line::from(header_spans))
        .alignment(Alignment::Left)
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(state.theme.border)));

    frame.render_widget(header, area);
}

/// Render chat messages
fn render_messages(frame: &mut Frame, area: Rect, state: &TuiState) {
    if state.messages.is_empty() {
        let empty = Paragraph::new("Start a conversation by typing a message below...")
            .style(Style::default().fg(state.theme.text_muted))
            .alignment(Alignment::Center);
        frame.render_widget(empty, area);
        return;
    }

    let mut lines: Vec<Line> = Vec::new();
    for msg in &state.messages {
        let (prefix, color) = match msg.role {
            MessageRole::User => ("You", state.theme.agent_colors.build), // User uses build agent color
            MessageRole::Assistant => ("Assistant", state.theme.success),
        };

        lines.push(Line::from(vec![
            Span::styled(format!("{}: ", prefix), Style::default().fg(color).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(Span::styled(&msg.content, Style::default().fg(state.theme.foreground))));
        lines.push(Line::from(""));
    }

    let messages = Paragraph::new(lines)
        .wrap(Wrap { trim: true })
        .scroll((0, 0));
    frame.render_widget(messages, area);
}

/// Render session footer
fn render_session_footer(frame: &mut Frame, area: Rect, state: &TuiState) {
    let status_text = match &state.session_status {
        SessionStatus::Idle => "Ready".to_string(),
        SessionStatus::Running => "Running...".to_string(),
        SessionStatus::Retry { attempt, message } => format!("Retry #{}: {}", attempt, message),
    };

    let footer = Line::from(vec![
        Span::styled("Tab ", Style::default().fg(state.theme.foreground)),
        Span::styled("switch agent  ", Style::default().fg(state.theme.text_muted)),
        Span::styled("Ctrl+K ", Style::default().fg(state.theme.foreground)),
        Span::styled("commands  ", Style::default().fg(state.theme.text_muted)),
        Span::styled(&status_text, Style::default().fg(state.theme.text_muted)),
    ]);

    frame.render_widget(Paragraph::new(footer), area);
}

/// Render command palette overlay
fn render_command_palette(frame: &mut Frame, area: Rect, state: &TuiState) {
    // Center overlay
    let width = 60.min(area.width.saturating_sub(10));
    let height = 15.min(area.height.saturating_sub(6));
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;

    let overlay_area = Rect { x, y, width, height };

    // Clear background
    frame.render_widget(Clear, overlay_area);

    // Command palette
    let commands = vec![
        "/new - Create new session",
        "/sessions - List sessions",
        "/model - Switch model",
        "/agent - Switch agent",
        "/status - View status",
        "/help - Show help",
        "/exit - Exit app",
    ];

    let content: Vec<Line> = commands
        .iter()
        .map(|c| Line::from(Span::styled(*c, Style::default().fg(state.theme.foreground))))
        .collect();

    let palette = Paragraph::new(content)
        .block(
            Block::default()
                .title(" Commands ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(state.theme.border_active))
                .style(Style::default().bg(state.theme.background_element)),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(palette, overlay_area);
}

// ===== TuiApp Implementation (event handling) =====

impl TuiApp {
    /// Handle key events
    async fn handle_key(&mut self, code: KeyCode, modifiers: event::KeyModifiers) {
        let ctrl = modifiers.contains(event::KeyModifiers::CONTROL);

        // Global shortcuts
        match code {
            KeyCode::Char('c') if ctrl => {
                self.running = false;
                return;
            }
            KeyCode::Esc => {
                if self.state.command_palette_visible {
                    self.state.command_palette_visible = false;
                } else if matches!(self.state.route, Route::Session { .. }) {
                    self.state.route = Route::Home;
                } else {
                    self.running = false;
                }
                return;
            }
            KeyCode::Char('k') if ctrl => {
                self.state.command_palette_visible = !self.state.command_palette_visible;
                return;
            }
            KeyCode::Char('p') if ctrl => {
                self.state.command_palette_visible = !self.state.command_palette_visible;
                return;
            }
            _ => {}
        }

        // Command palette handling
        if self.state.command_palette_visible {
            match code {
                KeyCode::Char('q') => self.state.command_palette_visible = false,
                _ => {}
            }
            return;
        }

        // Route-specific handling
        match &self.state.route.clone() {
            Route::Home => self.handle_home_key(code, ctrl).await,
            Route::Session { .. } => self.handle_session_key(code, ctrl).await,
        }
    }

    /// Handle home screen keys
    async fn handle_home_key(&mut self, code: KeyCode, _ctrl: bool) {
        match code {
            KeyCode::Char(c) => {
                self.state.prompt_input.push(c);
            }
            KeyCode::Backspace => {
                self.state.prompt_input.pop();
            }
            KeyCode::Enter => {
                if !self.state.prompt_input.is_empty() {
                    let prompt_content = self.state.prompt_input.clone();
                    self.state.prompt_input.clear();
                    
                    // Create new session via AppContext
                    match self.app_context.create_session().await {
                        Ok(session_id) => {
                            // Add user message to local UI state
                            self.state.messages.push(ChatMessage {
                                role: MessageRole::User,
                                content: prompt_content.clone(),
                                timestamp: std::time::Instant::now(),
                                is_streaming: false,
                            });
                            
                            // Add placeholder assistant message for streaming
                            self.state.messages.push(ChatMessage {
                                role: MessageRole::Assistant,
                                content: String::new(),
                                timestamp: std::time::Instant::now(),
                                is_streaming: true,
                            });
                            
                            // Send message with streaming (updates UI progressively)
                            self.send_with_streaming(prompt_content).await;
                            
                            // Navigate to session
                            self.state.route = Route::Session { session_id };
                        }
                        Err(e) => {
                            tracing::error!("Failed to create session: {}", e);
                        }
                    }
                }
            }
            KeyCode::Tab => {
                // Cycle agent via AppContext
                self.app_context.cycle_agent().await;
                
                // Sync agent from backend
                let backend_state = self.app_context.state.read().await;
                self.state.current_agent = backend_state.current_agent.clone();
            }
            _ => {}
        }
    }

    /// Handle session screen keys
    async fn handle_session_key(&mut self, code: KeyCode, _ctrl: bool) {
        match code {
            KeyCode::Char(c) => {
                self.state.prompt_input.push(c);
            }
            KeyCode::Backspace => {
                self.state.prompt_input.pop();
            }
            KeyCode::Enter => {
                if !self.state.prompt_input.is_empty() {
                    let prompt_content = self.state.prompt_input.clone();
                    self.state.prompt_input.clear();
                    
                    // Add user message to local UI state
                    self.state.messages.push(ChatMessage {
                        role: MessageRole::User,
                        content: prompt_content.clone(),
                        timestamp: std::time::Instant::now(),
                        is_streaming: false,
                    });
                    
                    // Add placeholder assistant message for streaming
                    self.state.messages.push(ChatMessage {
                        role: MessageRole::Assistant,
                        content: String::new(),
                        timestamp: std::time::Instant::now(),
                        is_streaming: true,
                    });
                    
                    // Send message with streaming (updates UI progressively)
                    self.send_with_streaming(prompt_content).await;
                }
            }
            KeyCode::Tab => {
                // Cycle agent via AppContext
                self.app_context.cycle_agent().await;
                
                // Sync agent from backend
                let backend_state = self.app_context.state.read().await;
                self.state.current_agent = backend_state.current_agent.clone();
            }
            _ => {}
        }
    }

    /// Send message with streaming visual effect
    async fn send_with_streaming(&mut self, content: String) {
        // Set session status to running
        self.state.session_status = SessionStatus::Running;
        
        // Get streaming response from AppContext
        match self.app_context.send_message_streaming(content).await {
            Ok((mut word_stream, _usage)) => {
                // Stream words to UI with visual effect
                while let Some(word) = word_stream.next().await {
                    // Update the last message (the streaming assistant message)
                    if let Some(last_msg) = self.state.messages.last_mut() {
                        last_msg.content.push_str(&word);
                    }
                    
                    // Redraw terminal to show progress
                    let state = self.state.clone();
                    let placeholder_idx = self.placeholder_idx;
                    if let Err(e) = self.terminal.draw(|frame| {
                        render_ui(frame, &state, placeholder_idx);
                    }) {
                        tracing::warn!("Failed to redraw during streaming: {}", e);
                    }
                }
                
                // Mark streaming complete
                if let Some(last_msg) = self.state.messages.last_mut() {
                    last_msg.is_streaming = false;
                }
            }
            Err(e) => {
                tracing::warn!("Failed to send message: {}", e);
                // Update the placeholder message with error
                if let Some(last_msg) = self.state.messages.last_mut() {
                    last_msg.content = format!("⚠️ Error: {}", e);
                    last_msg.is_streaming = false;
                }
            }
        }
        
        // Sync token display from backend
        self.sync_state_from_context().await;
        
        // Set session status back to idle
        self.state.session_status = SessionStatus::Idle;
    }

    /// Sync messages from AppContext to local TuiState
    async fn sync_messages_from_context(&mut self) {
        let backend_state = self.app_context.state.read().await;
        
        // Convert backend messages to TUI ChatMessages
        self.state.messages = backend_state.messages.iter().map(|msg| {
            let role = match msg.role {
                ricecoder_sessions::MessageRole::User => MessageRole::User,
                ricecoder_sessions::MessageRole::Assistant => MessageRole::Assistant,
                ricecoder_sessions::MessageRole::System => MessageRole::Assistant, // Map system to assistant for display
            };
            
            // Extract text content from message parts using Message::content()
            let content = msg.content();
            
            ChatMessage {
                role,
                content,
                timestamp: std::time::Instant::now(), // Use current time since we don't have direct conversion
                is_streaming: false,
            }
        }).collect();
    }

    /// Handle mouse events
    fn handle_mouse(&mut self, _mouse: event::MouseEvent) {
        // Mouse handling will be implemented later
    }
}

impl Drop for TuiApp {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        let _ = self.terminal.show_cursor();
    }
}
