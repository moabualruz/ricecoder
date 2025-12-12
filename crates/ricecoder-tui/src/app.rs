//! Application state and main TUI application

use crate::accessibility::{
    FocusManager, KeyboardNavigationManager, ScreenReaderAnnouncer, StateChangeEvent,
};
use crate::config::TuiConfig;
use crate::event::{Event, EventLoop};
use crate::image_integration::ImageIntegration;
use crate::integration::WidgetIntegration;
use crate::render::Renderer;
use crate::style::Theme;
use crate::terminal_state::TerminalCapabilities;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::theme::ThemeManager;
use crate::tea::PendingOperation;
use anyhow::Result;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::prelude::*;

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

    /// Get the next mode in the cycle
    pub fn next(&self) -> AppMode {
        match self {
            AppMode::Chat => AppMode::Command,
            AppMode::Command => AppMode::Diff,
            AppMode::Diff => AppMode::Help,
            AppMode::Help => AppMode::Chat,
        }
    }

    /// Get the previous mode in the cycle
    pub fn previous(&self) -> AppMode {
        match self {
            AppMode::Chat => AppMode::Help,
            AppMode::Command => AppMode::Chat,
            AppMode::Diff => AppMode::Command,
            AppMode::Help => AppMode::Diff,
        }
    }
}

/// Main application state - TEA Architecture Integration
pub struct App {
    /// TEA reactive state manager
    pub reactive_state: std::sync::Arc<tokio::sync::RwLock<crate::ReactiveState>>,
    /// Event dispatcher for async event handling
    pub event_dispatcher: crate::EventDispatcher,
    /// Optimistic update manager
    pub optimistic_updater: crate::OptimisticUpdater,
    /// Loading state manager
    pub loading_manager: crate::LoadingManager,
    /// Virtual DOM renderer for efficient updates
    pub virtual_renderer: crate::VirtualRenderer,
}

impl App {
    /// Create a new application instance with TEA architecture
    pub async fn new() -> Result<Self> {
        // Create initial TEA model
        let initial_model = crate::AppModel::init(
            crate::config::TuiConfig::default(),
            &crate::theme::ThemeManager::new(),
            crate::session_integration::SessionIntegration::new(10),
            crate::project_bootstrap::ProjectBootstrap::new(
                std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
            ),
            crate::integration::WidgetIntegration::new(),
            crate::image_integration::ImageIntegration::new(),
            crate::render::Renderer::new(),
        );

        // Create reactive state manager
        let reactive_state = std::sync::Arc::new(tokio::sync::RwLock::new(
            crate::ReactiveState::new(initial_model)
        ));

        // Create event dispatcher
        let event_dispatcher = crate::EventDispatcher::new();

        // Create optimistic updater
        let optimistic_updater = crate::OptimisticUpdater::new();

        // Create loading manager
        let loading_manager = crate::LoadingManager::new();

        // Create virtual renderer
        let virtual_renderer = crate::VirtualRenderer::new();

        let mut app = Self {
            reactive_state,
            event_dispatcher,
            optimistic_updater,
            loading_manager,
            virtual_renderer,
        };

        // Perform project bootstrap
        if let Err(e) = app.bootstrap_project().await {
            tracing::warn!("Project bootstrap failed: {}", e);
            // Continue anyway - bootstrap failure shouldn't prevent app startup
        } else {
            tracing::info!("Project bootstrap completed successfully");
        }

        Ok(app)
    }

    /// Bootstrap the project using TEA architecture
    async fn bootstrap_project(&self) -> Result<()> {
        // Dispatch bootstrap event
        let event_id = self.event_dispatcher.dispatch_event(
            crate::AppMessage::OperationStarted(PendingOperation {
                id: "project_bootstrap".to_string(),
                description: "Bootstrapping project".to_string(),
                start_time: std::time::Instant::now(),
                timeout: std::time::Duration::from_secs(30),
            }),
            crate::EventPriority::High,
            crate::EventSource::System,
        ).await?;

        // Perform actual bootstrap
        let reactive_state = self.reactive_state.read().await;
        let model = reactive_state.current();

        // TODO: Implement actual project bootstrap logic using TEA model
        // For now, just mark as completed
        drop(reactive_state);

        self.event_dispatcher.dispatch_event(
            crate::AppMessage::OperationCompleted(event_id),
            crate::EventPriority::Normal,
            crate::EventSource::System,
        ).await?;

        Ok(())
    }

    /// Create a new application instance with a specific configuration
    pub async fn with_config(config: TuiConfig) -> Result<Self> {
        // Use default capabilities for backward compatibility
        let capabilities = TerminalCapabilities::detect();
        Self::with_capabilities(config, &capabilities).await
    }

    /// Create a new application instance with terminal capabilities
    pub async fn with_capabilities(config: TuiConfig, capabilities: &TerminalCapabilities) -> Result<Self> {
        let theme_manager = ThemeManager::new();

        // Load theme from config
        theme_manager.load_from_config(&config)?;
        let theme = theme_manager.current()?;

        // Create widget integration with default terminal size
        let widget_integration = WidgetIntegration::new(80, 24);

        // Initialize accessibility features
        let screen_reader = ScreenReaderAnnouncer::new(config.accessibility.screen_reader_enabled);
        let keyboard_nav = KeyboardNavigationManager::new();
        let focus_manager = FocusManager::new();

        // Initialize provider integration
        let provider_integration = crate::provider_integration::ProviderIntegration::with_provider(
            config.provider.clone(),
            config.model.clone(),
        );

        let app = Self {
            mode: AppMode::Chat,
            previous_mode: AppMode::Chat,
            chat: ChatState::default(),
            theme_manager,
            theme,
            config,
            should_exit: false,
            event_loop: EventLoop::new(),
            renderer: Renderer::new(),
            keybindings_enabled: true,
            widget_integration,
            screen_reader,
            keyboard_nav,
            focus_manager,
            provider_integration,
            image_integration: ImageIntegration::new(),
            image_widget: crate::image_widget::ImageWidget::new(capabilities),
            help_dialog: ricecoder_help::HelpDialog::default_ricecoder(),
            file_picker: crate::file_picker::FilePickerWidget::new(),
            file_watcher: None,
            file_watcher_receiver: None,
            session_integration: crate::session_integration::SessionIntegration::new(10), // Allow up to 10 sessions
            project_bootstrap: crate::project_bootstrap::ProjectBootstrap::new(std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))),
        };

        // Perform project bootstrap
        if let Err(e) = app.project_bootstrap.bootstrap().await {
            tracing::warn!("Project bootstrap failed: {}", e);
            // Continue anyway - bootstrap failure shouldn't prevent app startup
        } else {
            tracing::info!("Project bootstrap completed successfully");
        }

        Ok(app)
    }

    /// Run the application with TEA architecture
    pub async fn run(&mut self) -> Result<()> {
        use crossterm::{
            execute,
            terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
            event::EnableMouseCapture,
        };
        use ratatui::prelude::*;
        use std::io;

        tracing::info!("Starting RiceCoder TUI with TEA architecture");

        // Initialize terminal
        let mut stdout = io::stdout();
        enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

        // Create terminal backend
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;

        // Start event dispatcher
        let reactive_state_clone = std::sync::Arc::clone(&self.reactive_state);
        tokio::spawn(async move {
            let dispatcher = crate::EventDispatcher::new();
            if let Err(e) = dispatcher.run(reactive_state_clone).await {
                tracing::error!("Event dispatcher error: {}", e);
            }
        });

        // Run the main TEA event loop
        let result = self.run_tea_event_loop(&mut terminal).await;

        // Restore terminal state
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            crossterm::event::DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        tracing::info!("RiceCoder TUI exited successfully");
        result
    }

    /// Run the TEA-based event loop
    async fn run_tea_event_loop(&mut self, terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>) -> Result<()> {
        use crossterm::event::{self as crossterm_event, Event as CrosstermEvent};

        loop {
            // Check if we should exit
            {
                let reactive_state = self.reactive_state.read().await;
                let model = reactive_state.current();
                // TODO: Check exit condition from model
            }

            // Poll for terminal events
            if crossterm::event::poll(std::time::Duration::from_millis(10))? {
                if let CrosstermEvent::Key(key) = crossterm::event::read()? {
                    // Convert crossterm event to TEA message
                    let message = self.convert_key_to_message(key);

                    // Dispatch event through TEA system
                    self.event_dispatcher.dispatch_event(
                        message,
                        crate::EventPriority::Normal,
                        crate::EventSource::UserInput,
                    ).await?;
                }
            }

            // Render the current state
            terminal.draw(|f| {
                self.render_frame(f);
            })?;

            // Small delay to prevent excessive CPU usage
            tokio::time::sleep(std::time::Duration::from_millis(16)).await; // ~60 FPS
        }
    }

    /// Convert crossterm key event to TEA message
    fn convert_key_to_message(&self, key: crossterm::event::KeyEvent) -> crate::AppMessage {
        // Handle global keybindings
        if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
            match key.code {
                crossterm::event::KeyCode::Char('1') => return crate::AppMessage::ModeChanged(crate::AppMode::Chat),
                crossterm::event::KeyCode::Char('2') => return crate::AppMessage::ModeChanged(crate::AppMode::Command),
                crossterm::event::KeyCode::Char('3') => return crate::AppMessage::ModeChanged(crate::AppMode::Diff),
                crossterm::event::KeyCode::Char('4') => return crate::AppMessage::ModeChanged(crate::AppMode::Help),
                crossterm::event::KeyCode::Char('c') => return crate::AppMessage::ExitRequested,
                _ => {}
            }
        }

        // Convert to TEA KeyEvent
        let tea_key = crate::event::KeyEvent {
            code: match key.code {
                crossterm::event::KeyCode::Char(c) => crate::event::KeyCode::Char(c),
                crossterm::event::KeyCode::Enter => crate::event::KeyCode::Enter,
                crossterm::event::KeyCode::Esc => crate::event::KeyCode::Esc,
                crossterm::event::KeyCode::Backspace => crate::event::KeyCode::Backspace,
                crossterm::event::KeyCode::Tab => crate::event::KeyCode::Tab,
                _ => crate::event::KeyCode::Char(' '), // Default fallback
            },
            modifiers: crate::event::KeyModifiers {
                ctrl: key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL),
                alt: key.modifiers.contains(crossterm::event::KeyModifiers::ALT),
                shift: key.modifiers.contains(crossterm::event::KeyModifiers::SHIFT),
            },
        };

        crate::AppMessage::KeyPress(tea_key)
    }

    /// Render the current frame using TEA state
    fn render_frame(&self, f: &mut ratatui::Frame) {
        // Get current model from reactive state
        let reactive_state = futures::executor::block_on(self.reactive_state.read());
        let model = reactive_state.current();

        // Render based on current mode
        match model.mode {
            crate::AppMode::Chat => self.render_chat_mode(f, model),
            crate::AppMode::Command => self.render_command_mode(f, model),
            crate::AppMode::Diff => self.render_diff_mode(f, model),
            crate::AppMode::Help => self.render_help_mode(f, model),
        }

        // Render loading indicators if any
        self.render_loading_indicators(f);
    }

    /// Render chat mode
    fn render_chat_mode(&self, f: &mut ratatui::Frame, model: &crate::AppModel) {
        use ratatui::prelude::*;

        let size = f.size();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),  // Messages area
                Constraint::Length(3), // Input area
                Constraint::Length(1), // Status bar
            ])
            .split(size);

        // Render messages area
        let messages_block = Block::default()
            .borders(Borders::ALL)
            .title("Chat");
        f.render_widget(messages_block, chunks[0]);

        // Render input area
        let input_block = Block::default()
            .borders(Borders::ALL)
            .title("Input");
        f.render_widget(input_block, chunks[1]);

        // Render status bar
        self.render_status_bar(f, chunks[2], model);
    }

    /// Render command mode
    fn render_command_mode(&self, f: &mut ratatui::Frame, model: &crate::AppModel) {
        use ratatui::prelude::*;

        let size = f.size();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),  // Command history
                Constraint::Length(3), // Command input
                Constraint::Length(1), // Status bar
            ])
            .split(size);

        // Render command history
        let history_block = Block::default()
            .borders(Borders::ALL)
            .title("Command History");
        f.render_widget(history_block, chunks[0]);

        // Render command input
        let input_block = Block::default()
            .borders(Borders::ALL)
            .title("Command");
        f.render_widget(input_block, chunks[1]);

        // Render status bar
        self.render_status_bar(f, chunks[2], model);
    }

    /// Render diff mode
    fn render_diff_mode(&self, f: &mut ratatui::Frame, model: &crate::AppModel) {
        use ratatui::prelude::*;

        let size = f.size();
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Diff View - Coming Soon");
        f.render_widget(block, size);

        // Render status bar at bottom
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(size);
        self.render_status_bar(f, chunks[1], model);
    }

    /// Render help mode
    fn render_help_mode(&self, f: &mut ratatui::Frame, model: &crate::AppModel) {
        use ratatui::prelude::*;

        let size = f.size();
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Help - Coming Soon");
        f.render_widget(block, size);

        // Render status bar at bottom
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(size);
        self.render_status_bar(f, chunks[1], model);
    }

    /// Render status bar
    fn render_status_bar(&self, f: &mut ratatui::Frame, area: ratatui::prelude::Rect, model: &crate::AppModel) {
        use ratatui::prelude::*;

        let status_text = format!(
            "Mode: {} | Sessions: {} | Tokens: {}",
            model.mode.display_name(),
            model.sessions.session_count,
            model.sessions.total_tokens.total_tokens()
        );

        let status_bar = Paragraph::new(status_text)
            .style(Style::default().bg(Color::Blue).fg(Color::White))
            .alignment(Alignment::Left);

        f.render_widget(status_bar, area);
    }

    /// Render loading indicators
    fn render_loading_indicators(&self, f: &mut ratatui::Frame) {
        // Get active loadings
        let loadings = futures::executor::block_on(self.loading_manager.get_active_loadings());

        if loadings.is_empty() {
            return;
        }

        // Render loading overlay
        use ratatui::prelude::*;

        let size = f.size();
        let loading_text = if loadings.len() == 1 {
            format!("Loading: {}", loadings[0].description)
        } else {
            format!("Loading {} operations...", loadings.len())
        };

        let loading_widget = Paragraph::new(loading_text)
            .style(Style::default().bg(Color::Yellow).fg(Color::Black))
            .alignment(Alignment::Center);

        // Position at bottom of screen
        let loading_area = Rect {
            x: 0,
            y: size.height.saturating_sub(1),
            width: size.width,
            height: 1,
        };

        f.render_widget(loading_widget, loading_area);
    }
}
