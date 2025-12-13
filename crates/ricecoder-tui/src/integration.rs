//! Widget integration and coordination
//!
//! This module handles wiring all widgets together, managing state synchronization,
//! and coordinating layout between different UI components.

use crate::app::{App, AppMode};
use crate::components::{DialogWidget, ListWidget, MenuWidget, SplitViewWidget, TabWidget};
use crate::diff::DiffWidget;
use crate::layout::Rect;
use crate::prompt::PromptWidget;
use crate::widgets::ChatWidget;
use anyhow::Result;

/// Widget container for managing all active widgets
pub struct WidgetContainer {
    /// Chat widget
    pub chat: ChatWidget,
    /// Diff widget
    pub diff: DiffWidget,
    /// Prompt widget
    pub prompt: PromptWidget,
    /// Menu widget
    pub menu: MenuWidget,
    /// List widget
    pub list: ListWidget,
    /// Dialog widget (optional)
    pub dialog: Option<DialogWidget>,
    /// Split view widget (optional)
    pub split_view: Option<SplitViewWidget>,
    /// Tab widget (optional)
    pub tabs: Option<TabWidget>,
}

impl WidgetContainer {
    /// Create a new widget container
    pub fn new() -> Self {
        Self {
            chat: ChatWidget::new(),
            diff: DiffWidget::new(),
            prompt: PromptWidget::new(),
            menu: MenuWidget::new(),
            list: ListWidget::new(),
            dialog: None,
            split_view: None,
            tabs: None,
        }
    }

    /// Reset all widgets to initial state
    pub fn reset_all(&mut self) {
        self.chat.clear();
        self.diff = DiffWidget::new();
        self.prompt = PromptWidget::new();
        self.menu.clear();
        self.list.clear();
        self.dialog = None;
        self.split_view = None;
        self.tabs = None;
    }

    /// Get the active widget based on app mode
    pub fn get_active_widget_mut(&mut self, mode: AppMode) -> Option<&mut dyn std::any::Any> {
        match mode {
            AppMode::Chat => Some(&mut self.chat as &mut dyn std::any::Any),
            AppMode::Diff => Some(&mut self.diff as &mut dyn std::any::Any),
            AppMode::Command => Some(&mut self.prompt as &mut dyn std::any::Any),
            AppMode::Help => Some(&mut self.menu as &mut dyn std::any::Any),
        }
    }
}

impl Default for WidgetContainer {
    fn default() -> Self {
        Self::new()
    }
}

/// Layout coordinator for managing widget positioning
pub struct LayoutCoordinator {
    /// Terminal width
    pub width: u16,
    /// Terminal height
    pub height: u16,
    /// Minimum width requirement
    pub min_width: u16,
    /// Minimum height requirement
    pub min_height: u16,
}

impl LayoutCoordinator {
    /// Create a new layout coordinator
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            min_width: 80,
            min_height: 24,
        }
    }

    /// Check if terminal size is valid
    pub fn is_valid(&self) -> bool {
        self.width >= self.min_width && self.height >= self.min_height
    }

    /// Get layout for chat mode
    pub fn layout_chat(&self) -> Result<ChatLayout> {
        if !self.is_valid() {
            return Err(anyhow::anyhow!(
                "Terminal too small: {}x{}",
                self.width,
                self.height
            ));
        }

        let prompt_height = 3;
        let chat_height = self.height.saturating_sub(prompt_height);

        Ok(ChatLayout {
            chat_area: Rect {
                x: 0,
                y: 0,
                width: self.width,
                height: chat_height,
            },
            prompt_area: Rect {
                x: 0,
                y: chat_height,
                width: self.width,
                height: prompt_height,
            },
        })
    }

    /// Get layout for diff mode
    pub fn layout_diff(&self) -> Result<DiffLayout> {
        if !self.is_valid() {
            return Err(anyhow::anyhow!(
                "Terminal too small: {}x{}",
                self.width,
                self.height
            ));
        }

        let prompt_height = 3;
        let diff_height = self.height.saturating_sub(prompt_height);

        Ok(DiffLayout {
            diff_area: Rect {
                x: 0,
                y: 0,
                width: self.width,
                height: diff_height,
            },
            prompt_area: Rect {
                x: 0,
                y: diff_height,
                width: self.width,
                height: prompt_height,
            },
        })
    }

    /// Get layout for command mode
    pub fn layout_command(&self) -> Result<CommandLayout> {
        if !self.is_valid() {
            return Err(anyhow::anyhow!(
                "Terminal too small: {}x{}",
                self.width,
                self.height
            ));
        }

        let prompt_height = 3;
        let menu_height = self.height.saturating_sub(prompt_height);

        Ok(CommandLayout {
            menu_area: Rect {
                x: 0,
                y: 0,
                width: self.width,
                height: menu_height,
            },
            prompt_area: Rect {
                x: 0,
                y: menu_height,
                width: self.width,
                height: prompt_height,
            },
        })
    }

    /// Get layout for help mode
    pub fn layout_help(&self) -> Result<HelpLayout> {
        if !self.is_valid() {
            return Err(anyhow::anyhow!(
                "Terminal too small: {}x{}",
                self.width,
                self.height
            ));
        }

        let prompt_height = 3;
        let help_height = self.height.saturating_sub(prompt_height);

        Ok(HelpLayout {
            help_area: Rect {
                x: 0,
                y: 0,
                width: self.width,
                height: help_height,
            },
            prompt_area: Rect {
                x: 0,
                y: help_height,
                width: self.width,
                height: prompt_height,
            },
        })
    }

    /// Update terminal size
    pub fn update_size(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
    }
}

impl Default for LayoutCoordinator {
    fn default() -> Self {
        Self::new(80, 24)
    }
}

/// Chat mode layout
#[derive(Debug, Clone)]
pub struct ChatLayout {
    /// Chat display area
    pub chat_area: Rect,
    /// Prompt input area
    pub prompt_area: Rect,
}

/// Diff mode layout
#[derive(Debug, Clone)]
pub struct DiffLayout {
    /// Diff display area
    pub diff_area: Rect,
    /// Prompt input area
    pub prompt_area: Rect,
}

/// Command mode layout
#[derive(Debug, Clone)]
pub struct CommandLayout {
    /// Menu display area
    pub menu_area: Rect,
    /// Prompt input area
    pub prompt_area: Rect,
}

/// Help mode layout
#[derive(Debug, Clone)]
pub struct HelpLayout {
    /// Help display area
    pub help_area: Rect,
    /// Prompt input area
    pub prompt_area: Rect,
}

/// State synchronizer for keeping widgets in sync
pub struct StateSynchronizer;

impl StateSynchronizer {
    /// Synchronize chat state with prompt
    pub fn sync_chat_to_prompt(chat: &ChatWidget, _prompt: &mut PromptWidget) {
        // Update prompt context based on chat state
        if !chat.messages.is_empty() {
            // Could update prompt indicators based on chat state
            tracing::debug!("Syncing chat state to prompt");
        }
    }

    /// Synchronize prompt state with chat
    pub fn sync_prompt_to_chat(prompt: &PromptWidget, _chat: &mut ChatWidget) {
        // Update chat based on prompt input
        if !prompt.input.is_empty() {
            tracing::debug!("Syncing prompt input to chat: {}", prompt.input);
        }
    }

    /// Synchronize diff state with prompt
    pub fn sync_diff_to_prompt(diff: &DiffWidget, _prompt: &mut PromptWidget) {
        // Update prompt context based on diff state
        let approved = diff.approved_hunks().len();
        let total = diff.hunks.len();
        tracing::debug!("Diff state: {}/{} hunks approved", approved, total);
    }

    /// Synchronize app state across all widgets
    pub fn sync_app_state(app: &App, widgets: &mut WidgetContainer) {
        // Update all widgets based on app state
        match app.mode {
            AppMode::Chat => {
                Self::sync_chat_to_prompt(&widgets.chat, &mut widgets.prompt);
            }
            AppMode::Diff => {
                Self::sync_diff_to_prompt(&widgets.diff, &mut widgets.prompt);
            }
            AppMode::Command => {
                // Command mode specific sync
                tracing::debug!("Syncing command mode state");
            }
            AppMode::Help => {
                // Help mode specific sync
                tracing::debug!("Syncing help mode state");
            }
        }
    }
}

/// Widget integration manager
pub struct WidgetIntegration {
    /// Widget container
    pub widgets: WidgetContainer,
    /// Layout coordinator
    pub layout: LayoutCoordinator,
}

impl WidgetIntegration {
    /// Create a new widget integration manager
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            widgets: WidgetContainer::new(),
            layout: LayoutCoordinator::new(width, height),
        }
    }

    /// Initialize widgets for the app
    pub fn initialize(&mut self, app: &App) -> Result<()> {
        // Initialize prompt with context
        self.widgets.prompt.context.mode = app.mode;
        self.widgets.prompt.context.project_name = Some("ricecoder".to_string());

        // Initialize chat widget
        self.widgets.chat = ChatWidget::new();

        // Initialize diff widget
        self.widgets.diff = DiffWidget::new();

        // Initialize menu widget
        self.widgets.menu = MenuWidget::new();

        tracing::info!("Widget integration initialized");
        Ok(())
    }

    /// Handle mode switch
    pub fn on_mode_switch(&mut self, old_mode: AppMode, new_mode: AppMode) -> Result<()> {
        tracing::info!("Mode switch: {:?} -> {:?}", old_mode, new_mode);

        // Update prompt context
        self.widgets.prompt.context.mode = new_mode;

        // Mode-specific initialization
        match new_mode {
            AppMode::Chat => {
                // Ensure chat widget is ready
                if self.widgets.chat.messages.is_empty() {
                    tracing::debug!("Chat mode: initializing empty chat");
                }
            }
            AppMode::Diff => {
                // Ensure diff widget is ready
                if self.widgets.diff.hunks.is_empty() {
                    tracing::debug!("Diff mode: no hunks loaded");
                }
            }
            AppMode::Command => {
                // Ensure menu widget is ready
                if self.widgets.menu.is_empty() {
                    tracing::debug!("Command mode: initializing menu");
                }
            }
            AppMode::Help => {
                // Ensure help is ready
                tracing::debug!("Help mode: showing help");
            }
        }

        Ok(())
    }

    /// Handle terminal resize
    pub fn on_resize(&mut self, width: u16, height: u16) -> Result<()> {
        self.layout.update_size(width, height);

        if !self.layout.is_valid() {
            tracing::warn!("Terminal size too small: {}x{}", width, height);
            return Err(anyhow::anyhow!("Terminal too small: {}x{}", width, height));
        }

        tracing::debug!("Terminal resized to {}x{}", width, height);
        Ok(())
    }

    /// Synchronize state across all widgets
    pub fn sync_state(&mut self, app: &App) {
        StateSynchronizer::sync_app_state(app, &mut self.widgets);
    }

    /// Get layout for current mode
    pub fn get_layout(&self, mode: AppMode) -> Result<LayoutInfo> {
        match mode {
            AppMode::Chat => {
                let layout = self.layout.layout_chat()?;
                Ok(LayoutInfo::Chat(layout))
            }
            AppMode::Diff => {
                let layout = self.layout.layout_diff()?;
                Ok(LayoutInfo::Diff(layout))
            }
            AppMode::Command => {
                let layout = self.layout.layout_command()?;
                Ok(LayoutInfo::Command(layout))
            }
            AppMode::Help => {
                let layout = self.layout.layout_help()?;
                Ok(LayoutInfo::Help(layout))
            }
        }
    }
}

impl Default for WidgetIntegration {
    fn default() -> Self {
        Self::new(80, 24)
    }
}

/// Layout information for different modes
#[derive(Debug, Clone)]
pub enum LayoutInfo {
    /// Chat mode layout
    Chat(ChatLayout),
    /// Diff mode layout
    Diff(DiffLayout),
    /// Command mode layout
    Command(CommandLayout),
    /// Help mode layout
    Help(HelpLayout),
}


