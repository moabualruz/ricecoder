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
use crate::theme::ThemeManager;
use anyhow::Result;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

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
    /// Current prompt context with text and images
    /// Requirements: 1.4 - Add images to prompt context
    pub prompt_context: crate::prompt_context::PromptContext,
}

impl Default for ChatState {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            input: String::new(),
            streaming: false,
            prompt_context: crate::prompt_context::PromptContext::new(),
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
    /// Image integration for drag-and-drop and display
    /// Requirements: 1.1 - Detect drag-and-drop event via crossterm
    pub image_integration: ImageIntegration,
    /// Image widget for displaying images in the terminal
    /// Requirements: 5.1 - Display images in terminal using ricecoder-images ImageDisplay
    pub image_widget: crate::image_widget::ImageWidget,
    /// Help dialog widget
    pub help_dialog: ricecoder_help::HelpDialog,
    /// File picker widget for @ file references
    pub file_picker: crate::file_picker::FilePickerWidget,
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
            image_integration: ImageIntegration::new(),
            image_widget: crate::image_widget::ImageWidget::new(),
            help_dialog: ricecoder_help::HelpDialog::default_ricecoder(),
            file_picker: crate::file_picker::FilePickerWidget::new(),
        };

        Ok(app)
    }

    /// Run the application
    pub async fn run(&mut self) -> Result<()> {
        use crossterm::{
            execute,
            terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
            event::EnableMouseCapture,
        };
        use ratatui::prelude::*;
        use std::io;

        tracing::info!("Starting RiceCoder TUI");

        // Initialize terminal
        let mut stdout = io::stdout();
        enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

        // Create terminal backend
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;

        // Run the main event loop
        let result = self.run_event_loop(&mut terminal).await;

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

    /// Run the main event loop with terminal rendering
    async fn run_event_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
        // Main event loop
        while !self.should_exit {
            // Poll for events
            if let Some(event) = self.event_loop.poll().await? {
                self.handle_event(event)?;
            }

            // Render the UI using ratatui's terminal.draw() closure
            terminal.draw(|f| {
                crate::render::Renderer::render_frame(f, self);
            })?;
        }

        Ok(())
    }

    /// Switch to a different mode
    pub fn switch_mode(&mut self, mode: AppMode) {
        if self.mode != mode {
            tracing::info!("Switching mode from {:?} to {:?}", self.mode, mode);
            self.previous_mode = self.mode;
            self.mode = mode;

            // Notify widget integration of mode switch
            if let Err(e) = self
                .widget_integration
                .on_mode_switch(self.previous_mode, self.mode)
            {
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

    /// Handle input submission
    pub fn handle_input_submission(&mut self, input: &str) -> Result<()> {
        // Check if input is a slash command
        if input.starts_with('/') {
            self.handle_slash_command(input)?;
            return Ok(());
        }

        // Handle regular chat input
        // TODO: Implement chat message sending
        tracing::info!("Chat input: {}", input);

        Ok(())
    }

    /// Handle slash commands
    fn handle_slash_command(&mut self, command: &str) -> Result<()> {
        match command {
            "/help" => {
                self.help_dialog.show();
                tracing::info!("Showing help dialog");
            }
            "/exit" | "/quit" => {
                self.should_exit = true;
                tracing::info!("Exit command received");
            }
            "/new" => {
                // TODO: Create new session
                tracing::info!("New session command");
            }
            "/sessions" => {
                // TODO: List sessions
                tracing::info!("List sessions command");
            }
            "/clear" => {
                // TODO: Clear current session
                tracing::info!("Clear session command");
            }
            _ => {
                tracing::warn!("Unknown slash command: {}", command);
            }
        }
        Ok(())
    }

    /// Handle file picker key events
    fn handle_file_picker_key(&mut self, key_event: crate::event::KeyEvent) -> Result<()> {
        match key_event.code {
            crate::event::KeyCode::Esc => {
                self.file_picker.hide();
            }
            crate::event::KeyCode::Enter => {
                match self.file_picker.confirm_selection() {
                    Ok(selections) => {
                        if !selections.is_empty() {
                            self.handle_file_selections(selections);
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to select files: {}", e);
                    }
                }
            }
            crate::event::KeyCode::Up => {
                self.file_picker.navigate_up();
            }
            crate::event::KeyCode::Down => {
                self.file_picker.navigate_down();
            }
            crate::event::KeyCode::Char(' ') => { // Space
                self.file_picker.toggle_selection();
            }
            crate::event::KeyCode::Char(c) => {
                if key_event.modifiers.ctrl && c == 'a' {
                    self.file_picker.select_all();
                } else {
                    self.file_picker.input_char(c);
                }
            }
            crate::event::KeyCode::Backspace => {
                self.file_picker.backspace();
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle selected files from file picker
    fn handle_file_selections(&mut self, selections: Vec<crate::file_picker::FileSelection>) {
        use crate::file_picker::{FileSelectionStatus, FileInfo};

        let mut included_files = Vec::new();
        let mut warnings = Vec::new();

        for selection in selections {
            match selection.status {
                FileSelectionStatus::Included => {
                    if let Some(content) = selection.content {
                        included_files.push((selection.path, content));
                    }
                }
                FileSelectionStatus::Directory => {
                    warnings.push(format!("Directory '{}' cannot be included", selection.path.display()));
                }
                FileSelectionStatus::BinaryFile => {
                    warnings.push(format!("Binary file '{}' cannot be included", selection.path.display()));
                }
                FileSelectionStatus::TooLarge => {
                    let size_mb = selection.info.size as f64 / (1024.0 * 1024.0);
                    warnings.push(format!("File '{}' is too large ({:.1}MB)", selection.path.display(), size_mb));
                }
                FileSelectionStatus::Error(msg) => {
                    warnings.push(format!("Error reading '{}': {}", selection.path.display(), msg));
                }
            }
        }

        // Include successful files in chat input
        if !included_files.is_empty() {
            let mut file_refs = Vec::new();
            let file_count = included_files.len();

            for (path, content) in included_files {
                let file_ref = format!("@{}:\n```\n{}\n```\n", path.display(), content);
                file_refs.push(file_ref);
            }

            let combined_refs = file_refs.join("\n");
            self.chat.input.push_str(&combined_refs);

            tracing::info!("Included {} files in chat input", file_count);
        }

        // Show warnings for failed inclusions
        for warning in warnings {
            tracing::warn!("{}", warning);
        }
    }

    /// Convert app KeyEvent to crossterm KeyEvent
    fn convert_to_crossterm_key(&self, key_event: crate::event::KeyEvent) -> crossterm::event::KeyEvent {
        use crossterm::event::{KeyCode as CKeyCode, KeyEvent as CKeyEvent, KeyModifiers as CKeyModifiers};

        let code = match key_event.code {
            crate::event::KeyCode::Char(c) => CKeyCode::Char(c),
            crate::event::KeyCode::Enter => CKeyCode::Enter,
            crate::event::KeyCode::Esc => CKeyCode::Esc,
            crate::event::KeyCode::Tab => CKeyCode::Tab,
            crate::event::KeyCode::Backspace => CKeyCode::Backspace,
            crate::event::KeyCode::Delete => CKeyCode::Delete,
            crate::event::KeyCode::Up => CKeyCode::Up,
            crate::event::KeyCode::Down => CKeyCode::Down,
            crate::event::KeyCode::Left => CKeyCode::Left,
            crate::event::KeyCode::Right => CKeyCode::Right,
            crate::event::KeyCode::F(n) => CKeyCode::F(n),
            _ => CKeyCode::Null,
        };

        let modifiers = if key_event.modifiers.ctrl {
            CKeyModifiers::CONTROL
        } else if key_event.modifiers.shift {
            CKeyModifiers::SHIFT
        } else if key_event.modifiers.alt {
            CKeyModifiers::ALT
        } else {
            CKeyModifiers::NONE
        };

        CKeyEvent::new(code, modifiers)
    }

    /// Handle an event
    pub fn handle_event(&mut self, event: Event) -> Result<()> {
        // Handle help dialog events first if it's visible
        if self.help_dialog.is_visible() {
            if let Event::Key(key_event) = &event {
                let crossterm_key = self.convert_to_crossterm_key(*key_event);
                let _ = self.help_dialog.handle_key(crossterm_key)?;
                return Ok(());
            }
        }

        // Handle file picker events if it's visible
        if self.file_picker.is_visible() {
            if let Event::Key(key_event) = &event {
                self.handle_file_picker_key(*key_event)?;
                return Ok(());
            }
        }

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

                // Handle text input in chat mode
                if self.mode == AppMode::Chat {
                    match key_event.code {
                        crate::event::KeyCode::Char('@') => {
                            // Show file picker when @ is typed
                            self.file_picker.show();
                            return Ok(());
                        }
                        crate::event::KeyCode::Char(c) => {
                            self.chat.input.push(c);
                            return Ok(());
                        }
                        crate::event::KeyCode::Backspace => {
                            self.chat.input.pop();
                            return Ok(());
                        }
                        crate::event::KeyCode::Enter => {
                            if !self.chat.input.is_empty() {
                                let input = self.chat.input.clone();
                                self.chat.input.clear();
                                self.handle_input_submission(&input)?;
                            }
                            return Ok(());
                        }
                        _ => {}
                    }
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
            Event::DragDrop { paths } => {
                tracing::debug!("Drag-and-drop event with {} files", paths.len());
                // Handle drag-and-drop events
                // Requirements: 1.1 - Pass drag-and-drop events to ricecoder-images handler
                self.handle_drag_drop_event(paths)?;
            }
        }
        Ok(())
    }

    /// Handle a drag-and-drop event with file paths
    ///
    /// # Arguments
    ///
    /// * `paths` - File paths from the drag-and-drop event
    ///
    /// # Requirements
    ///
    /// - Req 1.1: Pass drag-and-drop events to ricecoder-images handler
    /// - Req 1.1: Handle file path extraction
    /// - Req 1.1: Handle multiple files in single drag-and-drop
    /// - Req 5.1: Add image preview to prompt context
    /// - Req 5.2: Display images in chat interface
    fn handle_drag_drop_event(&mut self, paths: std::vec::Vec<std::path::PathBuf>) -> Result<()> {
        tracing::info!("Processing drag-and-drop event with {} files", paths.len());

        // Pass to image integration
        let (added, errors) = self.image_integration.handle_drag_drop_event(paths);

        // Update image widget with added images
        // Requirements: 5.1 - Add image preview to prompt context
        if !added.is_empty() {
            self.image_widget.add_images(added.clone());
            tracing::info!("Updated image widget with {} images", added.len());

            // Also add to prompt context
            // Requirements: 1.4 - Add images to prompt context
            self.chat.prompt_context.add_images(added.clone());
            tracing::info!("Added {} images to prompt context", added.len());
        }

        // Log results
        if !added.is_empty() {
            tracing::info!("Added {} images to prompt context", added.len());
            for path in &added {
                tracing::debug!("Added image: {}", path.display());
            }
        }

        if !errors.is_empty() {
            tracing::warn!("Encountered {} errors processing drag-and-drop", errors.len());
            for error in &errors {
                tracing::debug!("Error: {}", error);
            }
        }

        Ok(())
    }

    /// Sync prompt context with current state
    ///
    /// # Requirements
    ///
    /// - Req 1.4: Add images to prompt context
    /// - Req 5.1: Include images in message history
    pub fn sync_prompt_context(&mut self) {
        // Sync images from image_integration to prompt_context
        let images = self.image_integration.get_images().to_vec();
        self.chat.prompt_context.clear_images();
        self.chat.prompt_context.add_images(images);

        tracing::debug!(
            "Synced prompt context: {} images",
            self.chat.prompt_context.image_count()
        );
    }

    /// Get the current prompt context
    pub fn get_prompt_context(&self) -> &crate::prompt_context::PromptContext {
        &self.chat.prompt_context
    }

    /// Get mutable access to the current prompt context
    pub fn get_prompt_context_mut(&mut self) -> &mut crate::prompt_context::PromptContext {
        &mut self.chat.prompt_context
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
        if key_event.modifiers.ctrl
            && key_event.modifiers.shift
            && key_event.code == crate::event::KeyCode::Char('m')
        {
            self.previous_mode_switch();
            return;
        }

        // Tab: Toggle between current and previous mode
        if key_event.code == crate::event::KeyCode::Tab && key_event.modifiers.alt {
            self.toggle_mode();
        }
    }

    /// Switch mode with accessibility announcement
    pub fn switch_mode_with_announcement(&mut self, mode: AppMode) {
        self.switch_mode(mode);

        // Announce mode change if screen reader is enabled
        if self.config.accessibility.screen_reader_enabled {
            self.screen_reader
                .announce_state_change("Mode", &format!("switched to {}", mode.display_name()));
        }
    }

    /// Announce a message to screen readers
    pub fn announce(&mut self, message: impl Into<String>) {
        if self.config.accessibility.announcements_enabled {
            use crate::accessibility::AnnouncementPriority;
            self.screen_reader
                .announce(message, AnnouncementPriority::Normal);
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
    pub fn register_keyboard_element(
        &mut self,
        alternative: crate::accessibility::TextAlternative,
    ) {
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
        let clamped_speed = speed.clamp(0.1, 2.0);
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
            self.screen_reader
                .announce(event.announcement_text(), event.priority);
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

            self.screen_reader
                .announce(format!("{}: {}", operation, status), priority);
        }
    }
}
