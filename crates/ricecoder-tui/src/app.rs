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
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

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
    /// Virtual list for chat messages
    pub chat_virtual_list: Option<crate::VirtualList<String>>,
    /// Virtual list for command history
    pub command_virtual_list: Option<crate::VirtualList<String>>,
    /// Lazy loader for chat messages
    pub chat_lazy_loader: Option<crate::LazyLoader<String>>,
    /// Lazy loader for command history
    pub command_lazy_loader: Option<crate::LazyLoader<String>>,
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

        // Create virtual lists (will be initialized with data later)
        let chat_virtual_list = None;
        let command_virtual_list = None;

        // Create lazy loaders
        let chat_lazy_loader = None;
        let command_lazy_loader = None;

        let app = Self {
            reactive_state,
            event_dispatcher,
            optimistic_updater,
            loading_manager,
            virtual_renderer,
            chat_virtual_list,
            command_virtual_list,
            chat_lazy_loader,
            command_lazy_loader,
        };

        // Initialize virtual lists for large datasets
        app.initialize_virtual_lists().await;

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

    /// Initialize virtual lists and lazy loaders for large datasets
    pub async fn initialize_virtual_lists(&mut self) {
        let reactive_state = self.reactive_state.read().await;
        let model = reactive_state.current();

        // Create sample chat messages (would come from actual data source)
        let all_chat_messages = vec![
            "Welcome to RiceCoder!".to_string(),
            "Type your messages here...".to_string(),
            "Use Ctrl+1-4 to switch modes".to_string(),
            "This is a virtual scrolling demo".to_string(),
            "Messages are efficiently rendered".to_string(),
            "Only visible items are processed".to_string(),
            "Lazy loading prevents memory issues".to_string(),
            "Large datasets are handled gracefully".to_string(),
            "Performance is optimized for 60 FPS".to_string(),
            "Virtual DOM diffing reduces re-renders".to_string(),
            "Event batching improves responsiveness".to_string(),
            "Optimistic updates provide instant feedback".to_string(),
        ];

        // Initialize chat lazy loader
        let chat_loaded = Arc::new(RwLock::new(Vec::new()));
        let chat_loaded_clone = Arc::clone(&chat_loaded);

        self.chat_lazy_loader = Some(crate::LazyLoader::new(
            all_chat_messages,
            5, // Load 5 messages at a time
            move |batch| {
                let loaded = chat_loaded_clone.clone();
                tokio::spawn(async move {
                    let mut loaded = loaded.blocking_write();
                    loaded.extend(batch);
                });
            },
        ));

        // Load initial batch of chat messages
        if let Some(loader) = &mut self.chat_lazy_loader {
            loader.load_next_batch().await;
        }

        // Get loaded chat messages for virtual list
        let chat_messages = if let Some(loaded) = chat_loaded.try_read() {
            loaded.clone()
        } else {
            vec![
                "Welcome to RiceCoder!".to_string(),
                "Loading messages...".to_string(),
            ]
        };

        // Initialize chat virtual list
        self.chat_virtual_list = Some(crate::VirtualList::new(
            chat_messages,
            10, // Show 10 messages at a time
            |message: &String, _index: usize| crate::VirtualNode::Paragraph {
                text: message.clone(),
                alignment: Alignment::Left,
            },
        ));

        // Create sample command history (would come from actual data source)
        let all_commands = vec![
            "git status".to_string(),
            "cargo build".to_string(),
            "npm install".to_string(),
            "docker-compose up".to_string(),
            "kubectl get pods".to_string(),
            "terraform plan".to_string(),
            "ansible-playbook deploy.yml".to_string(),
            "pytest tests/".to_string(),
            "gradle build".to_string(),
            "make install".to_string(),
            "composer update".to_string(),
            "bundle install".to_string(),
        ];

        // Initialize command lazy loader
        let command_loaded = Arc::new(RwLock::new(Vec::new()));
        let command_loaded_clone = Arc::clone(&command_loaded);

        self.command_lazy_loader = Some(crate::LazyLoader::new(
            all_commands,
            4, // Load 4 commands at a time
            move |batch| {
                let loaded = command_loaded_clone.clone();
                tokio::spawn(async move {
                    let mut loaded = loaded.blocking_write();
                    loaded.extend(batch);
                });
            },
        ));

        // Load initial batch of commands
        if let Some(loader) = &mut self.command_lazy_loader {
            loader.load_next_batch().await;
        }

        // Get loaded commands for virtual list
        let command_history = if let Some(loaded) = command_loaded.try_read() {
            loaded.clone()
        } else {
            model.commands.command_history.clone()
        };

        // Initialize command virtual list
        self.command_virtual_list = Some(crate::VirtualList::new(
            command_history,
            15, // Show 15 commands at a time
            |command: &String, _index: usize| crate::VirtualNode::Paragraph {
                text: format!("$ {}", command),
                alignment: Alignment::Left,
            },
        ));
    }

    /// Scroll chat messages
    pub fn scroll_chat(&mut self, delta: isize) {
        if let Some(virtual_list) = &mut self.chat_virtual_list {
            virtual_list.scroll_by(delta);
        }
    }

    /// Scroll command history
    pub fn scroll_commands(&mut self, delta: isize) {
        if let Some(virtual_list) = &mut self.command_virtual_list {
            virtual_list.scroll_by(delta);
        }
    }

    /// Check and trigger lazy loading if needed
    pub async fn check_lazy_loading(&mut self) {
        // Check chat messages
        if let Some(virtual_list) = &self.chat_virtual_list {
            let (offset, total) = virtual_list.scroll_position();
            let visible_items = virtual_list.scroll.visible_items;

            if offset + visible_items >= total.saturating_sub(5) && self.can_load_more_chat() {
                self.load_more_chat_messages().await;
            }
        }

        // Check command history
        if let Some(virtual_list) = &self.command_virtual_list {
            let (offset, total) = virtual_list.scroll_position();
            let visible_items = virtual_list.scroll.visible_items;

            if offset + visible_items >= total.saturating_sub(3) && self.can_load_more_commands() {
                self.load_more_commands().await;
            }
        }
    }

    /// Update virtual lists when data changes
    pub async fn update_virtual_lists(&mut self) {
        let reactive_state = self.reactive_state.read().await;
        let model = reactive_state.current();

        // Update command history virtual list
        if let Some(virtual_list) = &mut self.command_virtual_list {
            virtual_list.update_items(model.commands.command_history.clone());
        }
    }

    /// Get virtual scrolling info for UI feedback
    pub fn get_scroll_info(&self) -> (Option<(usize, usize)>, Option<(usize, usize)>) {
        let chat_scroll = self.chat_virtual_list.as_ref()
            .map(|vl| vl.scroll_position());
        let command_scroll = self.command_virtual_list.as_ref()
            .map(|vl| vl.scroll_position());

        (chat_scroll, command_scroll)
    }

    /// Load more chat messages if needed
    pub async fn load_more_chat_messages(&mut self) {
        if let Some(loader) = &mut self.chat_lazy_loader {
            if !loader.is_loading() && !loader.is_fully_loaded() {
                // Start loading
                self.loading_manager.start_loading(
                    "chat_messages".to_string(),
                    "Loading more messages...".to_string(),
                ).await;

                loader.load_next_batch().await;

                // Update virtual list with new data
                if let Some(virtual_list) = &mut self.chat_virtual_list {
                    let loaded_messages = loader.loaded_items().clone();
                    virtual_list.update_items(loaded_messages);
                }

                // Complete loading
                self.loading_manager.complete_loading("chat_messages").await;
            }
        }
    }

    /// Load more command history if needed
    pub async fn load_more_commands(&mut self) {
        if let Some(loader) = &mut self.command_lazy_loader {
            if !loader.is_loading() && !loader.is_fully_loaded() {
                // Start loading
                self.loading_manager.start_loading(
                    "command_history".to_string(),
                    "Loading command history...".to_string(),
                ).await;

                loader.load_next_batch().await;

                // Update virtual list with new data
                if let Some(virtual_list) = &mut self.command_virtual_list {
                    let loaded_commands = loader.loaded_items().clone();
                    virtual_list.update_items(loaded_commands);
                }

                // Complete loading
                self.loading_manager.complete_loading("command_history").await;
            }
        }
    }

    /// Check if more data can be loaded
    pub fn can_load_more_chat(&self) -> bool {
        self.chat_lazy_loader.as_ref()
            .map(|loader| !loader.is_fully_loaded())
            .unwrap_or(false)
    }

    pub fn can_load_more_commands(&self) -> bool {
        self.command_lazy_loader.as_ref()
            .map(|loader| !loader.is_fully_loaded())
            .unwrap_or(false)
    }

    /// Get loading progress
    pub fn get_loading_progress(&self) -> (Option<f32>, Option<f32>) {
        let chat_progress = self.chat_lazy_loader.as_ref().map(|l| l.progress());
        let command_progress = self.command_lazy_loader.as_ref().map(|l| l.progress());

        (chat_progress, command_progress)
    }
}
