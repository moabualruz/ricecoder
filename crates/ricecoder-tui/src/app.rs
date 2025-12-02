//! Application state and main TUI application

use anyhow::Result;
use crate::config::TuiConfig;
use crate::style::Theme;
use crate::theme::ThemeManager;
use crate::event::{Event, EventLoop};
use crate::render::Renderer;
use crate::integration::WidgetIntegration;
use crate::accessibility::{ScreenReaderAnnouncer, KeyboardNavigationManager, FocusManager, StateChangeEvent};

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

/// Chat state
#[derive(Debug, Clone)]
pub struct ChatState {
    /// Messages in the conversation
    pub messages: Vec<String>,
    /// Current input
    pub input: String,
    /// Whether streaming is active
    pub streaming: bool,
}

impl Default for ChatState {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            input: String::new(),
            streaming: false,
        }
    }
}

/// Main application state
pub struct App {
    /// Current application mode
    pub mode: AppMode,
    /// Previous application mode (for quick switching)
    pub previous_mode: AppMode,
    /// Chat state
    pub chat: ChatState,
    /// Theme manager for runtime theme switching
    pub theme_manager: ThemeManager,
    /// Current theme (cached from theme manager)
    pub theme: Theme,
    /// Application configuration
    pub config: TuiConfig,
    /// Whether the application should exit
    pub should_exit: bool,
    /// Event loop
    pub event_loop: EventLoop,
    /// Renderer
    pub renderer: Renderer,
    /// Mode-specific keybindings enabled
    pub keybindings_enabled: bool,
    /// Widget integration manager
    pub widget_integration: WidgetIntegration,
    /// Screen reader announcer for accessibility
    pub screen_reader: ScreenReaderAnnouncer,
    /// Keyboard navigation manager
    pub keyboard_nav: KeyboardNavigationManager,
    /// Focus manager for accessibility
    pub focus_manager: FocusManager,
    /// Provider integration for AI responses
    pub provider_integration: crate::provider_integration::ProviderIntegration,
}

impl App {
    /// Create a new application instance
    pub fn new() -> Result<Self> {
        let config = TuiConfig::load()?;
        Self::with_config(config)
    }

    /// Create a new application instance with a specific configuration
    pub fn with_config(config: TuiConfig) -> Result<Self> {
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
        };

        Ok(app)
    }

    /// Run the application
    pub async fn run(&mut self) -> Result<()> {
        tracing::info!("Starting RiceCoder TUI");

        // Main event loop
        while !self.should_exit {
            // Poll for events
            if let Some(event) = self.event_loop.poll().await? {
                self.handle_event(event)?;
            }

            // Render the UI
            self.renderer.render(self)?;
        }

        tracing::info!("RiceCoder TUI exited successfully");
        Ok(())
    }

    /// Switch to a different mode
    pub fn switch_mode(&mut self, mode: AppMode) {
        if self.mode != mode {
            tracing::info!("Switching mode from {:?} to {:?}", self.mode, mode);
            self.previous_mode = self.mode;
            self.mode = mode;
            
            // Notify widget integration of mode switch
            if let Err(e) = self.widget_integration.on_mode_switch(self.previous_mode, self.mode) {
                tracing::error!("Failed to switch mode: {}", e);
            }
        }
    }

    /// Switch to the next mode in the cycle
    pub fn next_mode(&mut self) {
        let next = self.mode.next();
        self.switch_mode(next);
    }

    /// Switch to the previous mode in the cycle
    pub fn previous_mode_switch(&mut self) {
        let prev = self.mode.previous();
        self.switch_mode(prev);
    }

    /// Toggle between current and previous mode
    pub fn toggle_mode(&mut self) {
        let prev = self.previous_mode;
        self.switch_mode(prev);
    }

    /// Get the current mode display name
    pub fn current_mode_name(&self) -> &'static str {
        self.mode.display_name()
    }

    /// Get the current mode shortcut
    pub fn current_mode_shortcut(&self) -> &'static str {
        self.mode.shortcut()
    }

    /// Switch to a theme by name
    pub fn switch_theme(&mut self, name: &str) -> Result<()> {
        self.theme_manager.switch_by_name(name)?;
        self.theme = self.theme_manager.current()?;
        self.config.theme = name.to_string();
        self.config.save()?;
        tracing::info!("Switched to theme: {}", name);
        Ok(())
    }

    /// Get available themes
    pub fn available_themes(&self) -> Vec<&'static str> {
        self.theme_manager.available_themes()
    }

    /// Get current theme name
    pub fn current_theme_name(&self) -> Result<String> {
        self.theme_manager.current_name()
    }

    /// Synchronize widget state
    pub fn sync_widget_state(&mut self) {
        // Create a temporary copy of app state for synchronization
        let mode = self.mode;
        
        // Sync state with widgets
        self.widget_integration.widgets.prompt.context.mode = mode;
        tracing::debug!("Widget state synchronized for mode: {:?}", mode);
    }

    /// Get the active widgets container
    pub fn widgets(&self) -> &crate::integration::WidgetContainer {
        &self.widget_integration.widgets
    }

    /// Get mutable access to the active widgets container
    pub fn widgets_mut(&mut self) -> &mut crate::integration::WidgetContainer {
        &mut self.widget_integration.widgets
    }

    /// Get the layout coordinator
    pub fn layout(&self) -> &crate::integration::LayoutCoordinator {
        &self.widget_integration.layout
    }

    /// Handle an event
    fn handle_event(&mut self, event: Event) -> Result<()> {
        match event {
            Event::Key(key_event) => {
                tracing::debug!("Key event: {:?}", key_event);
                // Handle key events
                if key_event.code == crate::event::KeyCode::Esc {
                    self.should_exit = true;
                }

                // Tab navigation for keyboard accessibility
                if key_event.code == crate::event::KeyCode::Tab {
                    self.handle_tab_navigation(key_event.modifiers.shift);
                    return Ok(());
                }

                // Mode switching with keyboard shortcuts
                if self.keybindings_enabled {
                    self.handle_mode_switching(key_event);
                }
            }
            Event::Mouse(_mouse_event) => {
                tracing::debug!("Mouse event");
                // Handle mouse events
            }
            Event::Resize { width, height } => {
                tracing::debug!("Resize event: {}x{}", width, height);
                self.config.width = Some(width);
                self.config.height = Some(height);
                
                // Notify widget integration of resize
                if let Err(e) = self.widget_integration.on_resize(width, height) {
                    tracing::error!("Failed to handle resize: {}", e);
                }
            }
            Event::Tick => {
                // Handle tick event for periodic updates
            }
        }
        Ok(())
    }

    /// Handle mode switching keyboard shortcuts
    fn handle_mode_switching(&mut self, key_event: crate::event::KeyEvent) {
        // Ctrl+1: Chat mode
        if key_event.modifiers.ctrl && key_event.code == crate::event::KeyCode::Char('1') {
            self.switch_mode(AppMode::Chat);
            return;
        }

        // Ctrl+2: Command mode
        if key_event.modifiers.ctrl && key_event.code == crate::event::KeyCode::Char('2') {
            self.switch_mode(AppMode::Command);
            return;
        }

        // Ctrl+3: Diff mode
        if key_event.modifiers.ctrl && key_event.code == crate::event::KeyCode::Char('3') {
            self.switch_mode(AppMode::Diff);
            return;
        }

        // Ctrl+4: Help mode
        if key_event.modifiers.ctrl && key_event.code == crate::event::KeyCode::Char('4') {
            self.switch_mode(AppMode::Help);
            return;
        }

        // Ctrl+M: Cycle to next mode
        if key_event.modifiers.ctrl && key_event.code == crate::event::KeyCode::Char('m') {
            self.next_mode();
            return;
        }

        // Ctrl+Shift+M: Cycle to previous mode
        if key_event.modifiers.ctrl && key_event.modifiers.shift && key_event.code == crate::event::KeyCode::Char('m') {
            self.previous_mode_switch();
            return;
        }

        // Tab: Toggle between current and previous mode
        if key_event.code == crate::event::KeyCode::Tab && key_event.modifiers.alt {
            self.toggle_mode();
            return;
        }
    }

    /// Switch mode with accessibility announcement
    pub fn switch_mode_with_announcement(&mut self, mode: AppMode) {
        self.switch_mode(mode);
        
        // Announce mode change if screen reader is enabled
        if self.config.accessibility.screen_reader_enabled {
            self.screen_reader.announce_state_change(
                "Mode",
                &format!("switched to {}", mode.display_name()),
            );
        }
    }

    /// Announce a message to screen readers
    pub fn announce(&mut self, message: impl Into<String>) {
        if self.config.accessibility.announcements_enabled {
            use crate::accessibility::AnnouncementPriority;
            self.screen_reader.announce(message, AnnouncementPriority::Normal);
        }
    }

    /// Announce an error to screen readers
    pub fn announce_error(&mut self, message: impl Into<String>) {
        if self.config.accessibility.announcements_enabled {
            self.screen_reader.announce_error(message);
        }
    }

    /// Announce a success to screen readers
    pub fn announce_success(&mut self, message: impl Into<String>) {
        if self.config.accessibility.announcements_enabled {
            self.screen_reader.announce_success(message);
        }
    }

    /// Enable or disable screen reader support
    pub fn set_screen_reader_enabled(&mut self, enabled: bool) {
        self.config.accessibility.screen_reader_enabled = enabled;
        if enabled {
            self.screen_reader.enable();
        } else {
            self.screen_reader.disable();
        }
    }

    /// Enable or disable high contrast mode
    pub fn set_high_contrast_enabled(&mut self, enabled: bool) {
        self.config.accessibility.high_contrast_enabled = enabled;
        if enabled {
            // Switch to high contrast theme
            if let Err(e) = self.theme_manager.switch_by_name("high-contrast") {
                tracing::warn!("Failed to switch to high contrast theme: {}", e);
            }
        }
    }

    /// Handle Tab key for keyboard navigation
    pub fn handle_tab_navigation(&mut self, shift: bool) {
        if shift {
            // Shift+Tab: Focus previous element
            if let Some(focused) = self.keyboard_nav.focus_previous() {
                if self.config.accessibility.screen_reader_enabled {
                    self.screen_reader.announce(
                        format!("Focused: {}", focused.full_description()),
                        crate::accessibility::AnnouncementPriority::Normal,
                    );
                }
            }
        } else {
            // Tab: Focus next element
            if let Some(focused) = self.keyboard_nav.focus_next() {
                if self.config.accessibility.screen_reader_enabled {
                    self.screen_reader.announce(
                        format!("Focused: {}", focused.full_description()),
                        crate::accessibility::AnnouncementPriority::Normal,
                    );
                }
            }
        }
    }

    /// Get the currently focused element description
    pub fn get_focused_element_description(&self) -> Option<String> {
        self.keyboard_nav
            .current_focus()
            .map(|alt| alt.full_description())
    }

    /// Register an element for keyboard navigation
    pub fn register_keyboard_element(&mut self, alternative: crate::accessibility::TextAlternative) {
        self.keyboard_nav.register_element(alternative);
    }

    /// Clear all keyboard navigation elements
    pub fn clear_keyboard_elements(&mut self) {
        self.keyboard_nav.clear();
    }

    /// Enable or disable animations
    pub fn set_animations_enabled(&mut self, enabled: bool) {
        self.config.accessibility.animations.enabled = enabled;
        self.config.animations = enabled;
        
        if self.config.accessibility.screen_reader_enabled {
            let state = if enabled { "enabled" } else { "disabled" };
            self.screen_reader.announce(
                format!("Animations {}", state),
                crate::accessibility::AnnouncementPriority::Normal,
            );
        }
    }

    /// Enable or disable reduce motion (for accessibility)
    pub fn set_reduce_motion(&mut self, enabled: bool) {
        self.config.accessibility.animations.reduce_motion = enabled;
        
        if self.config.accessibility.screen_reader_enabled {
            let state = if enabled { "enabled" } else { "disabled" };
            self.screen_reader.announce(
                format!("Reduce motion {}", state),
                crate::accessibility::AnnouncementPriority::Normal,
            );
        }
    }

    /// Set animation speed multiplier
    pub fn set_animation_speed(&mut self, speed: f32) {
        let clamped_speed = speed.max(0.1).min(2.0);
        self.config.accessibility.animations.speed = clamped_speed;
        
        if self.config.accessibility.screen_reader_enabled {
            self.screen_reader.announce(
                format!("Animation speed set to {:.1}x", clamped_speed),
                crate::accessibility::AnnouncementPriority::Normal,
            );
        }
    }

    /// Check if animations should be displayed
    pub fn should_animate(&self) -> bool {
        self.config.accessibility.animations.should_animate()
    }

    /// Get animation duration in milliseconds
    pub fn animation_duration_ms(&self, base_ms: u32) -> u32 {
        self.config.accessibility.animations.duration_ms(base_ms)
    }

    /// Announce a state change
    pub fn announce_state_change(&mut self, event: StateChangeEvent) {
        if self.config.accessibility.announcements_enabled {
            self.screen_reader.announce(event.announcement_text(), event.priority);
        }
    }

    /// Set focus to an element and announce it
    pub fn set_focus_with_announcement(&mut self, element_id: impl Into<String>) {
        let id = element_id.into();
        self.focus_manager.set_focus(&id);
        
        if self.config.accessibility.screen_reader_enabled {
            self.screen_reader.announce(
                format!("Focus moved to {}", id),
                crate::accessibility::AnnouncementPriority::Normal,
            );
        }
    }

    /// Restore previous focus
    pub fn restore_focus(&mut self) {
        if let Some(element_id) = self.focus_manager.restore_focus() {
            if self.config.accessibility.screen_reader_enabled {
                self.screen_reader.announce(
                    format!("Focus restored to {}", element_id),
                    crate::accessibility::AnnouncementPriority::Normal,
                );
            }
        }
    }

    /// Announce an operation status
    pub fn announce_operation_status(&mut self, operation: &str, status: &str) {
        if self.config.accessibility.announcements_enabled {
            let priority = if status.contains("error") || status.contains("failed") {
                crate::accessibility::AnnouncementPriority::High
            } else if status.contains("success") || status.contains("complete") {
                crate::accessibility::AnnouncementPriority::Normal
            } else {
                crate::accessibility::AnnouncementPriority::Low
            };

            self.screen_reader.announce(
                format!("{}: {}", operation, status),
                priority,
            );
        }
    }
}
