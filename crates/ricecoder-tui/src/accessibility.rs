//! Accessibility features for the TUI
//!
//! This module provides accessibility support including:
//! - Screen reader support with text alternatives
//! - Full keyboard navigation
//! - High contrast mode
//! - Animation controls
//! - State announcements

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::tea::AppMessage;
use ratatui::style::{Color, Modifier, Style};

/// Animation configuration
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AnimationConfig {
    /// Enable animations
    pub enabled: bool,
    /// Animation speed (0.1 to 2.0, where 1.0 is normal)
    pub speed: f32,
    /// Reduce motion for accessibility
    pub reduce_motion: bool,
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            speed: 1.0,
            reduce_motion: false,
        }
    }
}

impl AnimationConfig {
    /// Get the effective animation duration in milliseconds
    pub fn duration_ms(&self, base_ms: u32) -> u32 {
        if !self.enabled || self.reduce_motion {
            return 0;
        }
        ((base_ms as f32) / self.speed) as u32
    }

    /// Check if animations should be shown
    pub fn should_animate(&self) -> bool {
        self.enabled && !self.reduce_motion
    }
}

/// Accessibility configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityConfig {
    /// Enable screen reader support
    pub screen_reader_enabled: bool,
    /// Enable high contrast mode
    pub high_contrast_enabled: bool,
    /// Disable animations
    pub animations_disabled: bool,
    /// Enable state announcements
    pub announcements_enabled: bool,
    /// Focus indicator style
    pub focus_indicator: FocusIndicatorStyle,
    /// Animation configuration
    #[serde(default)]
    pub animations: AnimationConfig,
    /// Font size multiplier (1.0 = normal, 1.5 = 150%, etc.)
    pub font_size_multiplier: f32,
    /// Enable large click targets
    pub large_click_targets: bool,
    /// Enable auto-advance for forms
    pub auto_advance: bool,
}

impl Default for AccessibilityConfig {
    fn default() -> Self {
        Self {
            screen_reader_enabled: false,
            high_contrast_enabled: false,
            animations_disabled: false,
            announcements_enabled: true,
            focus_indicator: FocusIndicatorStyle::Bracket,
            animations: AnimationConfig::default(),
            font_size_multiplier: 1.0,
            large_click_targets: false,
            auto_advance: false,
        }
    }
}

impl AccessibilityConfig {
    /// Enable accessibility features
    pub fn enable(&mut self) {
        self.screen_reader_enabled = true;
        self.high_contrast_enabled = true;
        self.announcements_enabled = true;
        self.animations.reduce_motion = true;
        self.large_click_targets = true;
    }

    /// Disable accessibility features
    pub fn disable(&mut self) {
        self.screen_reader_enabled = false;
        self.high_contrast_enabled = false;
        self.announcements_enabled = false;
        self.animations.reduce_motion = false;
        self.large_click_targets = false;
    }

    /// Set font size multiplier for accessibility
    pub fn set_font_size_multiplier(&mut self, multiplier: f32) {
        self.font_size_multiplier = multiplier.clamp(1.0, 2.0);
    }

    /// Enable large click targets for motor accessibility
    pub fn enable_large_click_targets(&mut self) {
        self.large_click_targets = true;
    }

    /// Enable auto-advance for forms
    pub fn enable_auto_advance(&mut self) {
        self.auto_advance = true;
    }

    /// Check if large text is enabled (WCAG 2.1 AA requires 1.5x normal)
    pub fn is_large_text(&self) -> bool {
        self.font_size_multiplier >= 1.5
    }

    /// Get effective font size
    pub fn effective_font_size(&self, base_size: u16) -> u16 {
        ((base_size as f32) * self.font_size_multiplier) as u16
    }
}

/// Focus indicator style for keyboard navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FocusIndicatorStyle {
    /// Use brackets: [focused element]
    Bracket,
    /// Use asterisks: *focused element*
    Asterisk,
    /// Use underline: _focused element_
    Underline,
    /// Use arrow: > focused element
    Arrow,
}

impl FocusIndicatorStyle {
    /// Get the prefix for this style
    pub fn prefix(&self) -> &'static str {
        match self {
            FocusIndicatorStyle::Bracket => "[",
            FocusIndicatorStyle::Asterisk => "*",
            FocusIndicatorStyle::Underline => "_",
            FocusIndicatorStyle::Arrow => "> ",
        }
    }

    /// Get the suffix for this style
    pub fn suffix(&self) -> &'static str {
        match self {
            FocusIndicatorStyle::Bracket => "]",
            FocusIndicatorStyle::Asterisk => "*",
            FocusIndicatorStyle::Underline => "_",
            FocusIndicatorStyle::Arrow => "",
        }
    }
}

/// Text alternative for visual elements
#[derive(Debug, Clone)]
pub struct TextAlternative {
    /// Unique identifier for the element
    pub id: String,
    /// Short description (for screen readers)
    pub short_description: String,
    /// Long description (for detailed context)
    pub long_description: Option<String>,
    /// Element type (button, input, list, etc.)
    pub element_type: ElementType,
}

impl TextAlternative {
    /// Create a new text alternative
    pub fn new(
        id: impl Into<String>,
        short_desc: impl Into<String>,
        element_type: ElementType,
    ) -> Self {
        Self {
            id: id.into(),
            short_description: short_desc.into(),
            long_description: None,
            element_type,
        }
    }

    /// Add a long description
    pub fn with_long_description(mut self, desc: impl Into<String>) -> Self {
        self.long_description = Some(desc.into());
        self
    }

    /// Get the full description for screen readers
    pub fn full_description(&self) -> String {
        match &self.long_description {
            Some(long) => format!("{}: {}", self.short_description, long),
            None => self.short_description.clone(),
        }
    }
}

/// Element type for semantic structure
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementType {
    /// Button element
    Button,
    /// Input field
    Input,
    /// List or menu
    List,
    /// List item
    ListItem,
    /// Tab
    Tab,
    /// Tab panel
    TabPanel,
    /// Dialog
    Dialog,
    /// Text content
    Text,
    /// Heading
    Heading,
    /// Code block
    CodeBlock,
    /// Message (chat)
    Message,
    /// Status indicator
    Status,
}

impl ElementType {
    /// Get the semantic role name
    pub fn role(&self) -> &'static str {
        match self {
            ElementType::Button => "button",
            ElementType::Input => "textbox",
            ElementType::List => "list",
            ElementType::ListItem => "listitem",
            ElementType::Tab => "tab",
            ElementType::TabPanel => "tabpanel",
            ElementType::Dialog => "dialog",
            ElementType::Text => "text",
            ElementType::Heading => "heading",
            ElementType::CodeBlock => "code",
            ElementType::Message => "article",
            ElementType::Status => "status",
        }
    }
}

/// Screen reader announcer for state changes
#[derive(Debug, Clone)]
pub struct ScreenReaderAnnouncer {
    /// Whether announcements are enabled
    enabled: bool,
    /// Announcement history (for testing)
    history: Vec<Announcement>,
}

/// An announcement for screen readers
#[derive(Debug, Clone)]
pub struct Announcement {
    /// The announcement text
    pub text: String,
    /// Priority level
    pub priority: AnnouncementPriority,
}

/// Priority level for announcements
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AnnouncementPriority {
    /// Low priority (polite)
    Low,
    /// Normal priority (assertive)
    Normal,
    /// High priority (alert)
    High,
}

impl ScreenReaderAnnouncer {
    /// Create a new announcer
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            history: Vec::new(),
        }
    }

    /// Announce a message
    pub fn announce(&mut self, text: impl Into<String>, priority: AnnouncementPriority) {
        if !self.enabled {
            return;
        }

        let announcement = Announcement {
            text: text.into(),
            priority,
        };

        self.history.push(announcement);
    }

    /// Announce a state change
    pub fn announce_state_change(&mut self, element: &str, state: &str) {
        self.announce(
            format!("{} {}", element, state),
            AnnouncementPriority::Normal,
        );
    }

    /// Announce an error
    pub fn announce_error(&mut self, message: impl Into<String>) {
        self.announce(message, AnnouncementPriority::High);
    }

    /// Announce a success
    pub fn announce_success(&mut self, message: impl Into<String>) {
        self.announce(message, AnnouncementPriority::Normal);
    }

    /// Get the last announcement
    pub fn last_announcement(&self) -> Option<&Announcement> {
        self.history.last()
    }

    /// Get all announcements
    pub fn announcements(&self) -> &[Announcement] {
        &self.history
    }

    /// Clear announcement history
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// Enable announcements
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable announcements
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Check if announcements are enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Generate ARIA-like label for an element
    pub fn generate_aria_label(&self, element_id: &str, base_label: &str, state_info: Option<&str>) -> String {
        let mut label = base_label.to_string();
        if let Some(state) = state_info {
            label.push_str(&format!(", {}", state));
        }
        label
    }

    /// Generate ARIA-like description for an element
    pub fn generate_aria_description(&self, element_id: &str, description: &str, instructions: Option<&str>) -> String {
        let mut desc = description.to_string();
        if let Some(instr) = instructions {
            desc.push_str(&format!(". {}", instr));
        }
        desc
    }

    /// Announce focus changes with ARIA-like information
    pub fn announce_focus_change(&mut self, element_id: &str, element_type: &str, element_label: &str) {
        self.announce(
            format!("Focused {}: {}", element_type, element_label),
            AnnouncementPriority::Normal,
        );
    }

    /// Announce navigation context
    pub fn announce_navigation_context(&mut self, context: &str, position: Option<(usize, usize)>) {
        let message = if let Some((current, total)) = position {
            format!("{}: item {} of {}", context, current + 1, total)
        } else {
            context.to_string()
        };
        self.announce(message, AnnouncementPriority::Low);
    }

    /// Announce completion status
    pub fn announce_completion(&mut self, operation: &str, success: bool, details: Option<&str>) {
        let status = if success { "completed" } else { "failed" };
        let mut message = format!("{} {}", operation, status);
        if let Some(details) = details {
            message.push_str(&format!(": {}", details));
        }
        let priority = if success { AnnouncementPriority::Normal } else { AnnouncementPriority::High };
        self.announce(message, priority);
    }
}

/// State change event for announcements
#[derive(Debug, Clone)]
pub struct StateChangeEvent {
    /// Component or element that changed
    pub component: String,
    /// Previous state
    pub previous_state: String,
    /// New state
    pub new_state: String,
    /// Priority of the announcement
    pub priority: AnnouncementPriority,
}

impl StateChangeEvent {
    /// Create a new state change event
    pub fn new(
        component: impl Into<String>,
        previous: impl Into<String>,
        new: impl Into<String>,
        priority: AnnouncementPriority,
    ) -> Self {
        Self {
            component: component.into(),
            previous_state: previous.into(),
            new_state: new.into(),
            priority,
        }
    }

    /// Get the announcement text
    pub fn announcement_text(&self) -> String {
        format!(
            "{} changed from {} to {}",
            self.component, self.previous_state, self.new_state
        )
    }
}

/// Focus management for accessibility
#[derive(Debug, Clone, Default)]
pub struct FocusManager {
    /// Currently focused element
    pub focused_element: Option<String>,
    /// Focus history for restoration
    pub focus_history: Vec<String>,
}

impl FocusManager {
    /// Create a new focus manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Set focus to an element
    pub fn set_focus(&mut self, element_id: impl Into<String>) {
        let id = element_id.into();
        if let Some(current) = &self.focused_element {
            self.focus_history.push(current.clone());
        }
        self.focused_element = Some(id);
    }

    /// Restore previous focus
    pub fn restore_focus(&mut self) -> Option<String> {
        self.focus_history.pop()
    }

    /// Clear focus
    pub fn clear_focus(&mut self) {
        self.focused_element = None;
    }
}

/// Keyboard navigation manager
#[derive(Debug, Clone, Default)]
pub struct KeyboardNavigationManager {
    /// Currently focused element ID
    pub focused_element: Option<String>,
    /// Tab order (list of element IDs in tab order)
    pub tab_order: Vec<String>,
    /// Element descriptions
    pub element_descriptions: HashMap<String, TextAlternative>,
}

impl KeyboardNavigationManager {
    /// Create a new keyboard navigation manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an element for keyboard navigation
    pub fn register_element(&mut self, alternative: TextAlternative) {
        self.tab_order.push(alternative.id.clone());
        self.element_descriptions
            .insert(alternative.id.clone(), alternative);
    }

    /// Set focus to an element
    pub fn focus(&mut self, element_id: &str) -> bool {
        if self.element_descriptions.contains_key(element_id) {
            self.focused_element = Some(element_id.to_string());
            true
        } else {
            false
        }
    }

    /// Move focus to the next element
    pub fn focus_next(&mut self) -> Option<&TextAlternative> {
        if self.tab_order.is_empty() {
            return None;
        }

        let next_index = match &self.focused_element {
            None => 0,
            Some(current) => {
                let current_index = self.tab_order.iter().position(|id| id == current)?;
                (current_index + 1) % self.tab_order.len()
            }
        };

        let next_id = self.tab_order[next_index].clone();
        self.focused_element = Some(next_id.clone());
        self.element_descriptions.get(&next_id)
    }

    /// Move focus to the previous element
    pub fn focus_previous(&mut self) -> Option<&TextAlternative> {
        if self.tab_order.is_empty() {
            return None;
        }

        let prev_index = match &self.focused_element {
            None => self.tab_order.len() - 1,
            Some(current) => {
                let current_index = self.tab_order.iter().position(|id| id == current)?;
                if current_index == 0 {
                    self.tab_order.len() - 1
                } else {
                    current_index - 1
                }
            }
        };

        let prev_id = self.tab_order[prev_index].clone();
        self.focused_element = Some(prev_id.clone());
        self.element_descriptions.get(&prev_id)
    }

    /// Get the currently focused element
    pub fn current_focus(&self) -> Option<&TextAlternative> {
        self.focused_element
            .as_ref()
            .and_then(|id| self.element_descriptions.get(id))
    }

    /// Clear all registered elements
    pub fn clear(&mut self) {
        self.focused_element = None;
        self.tab_order.clear();
        self.element_descriptions.clear();
    }
}

/// Enhanced keyboard navigation with WCAG compliance
#[derive(Debug, Clone)]
pub struct EnhancedKeyboardNavigation {
    /// Base keyboard navigation manager
    base_nav: KeyboardNavigationManager,
    /// Focus ring style for visual indicators
    focus_ring_style: Style,
    /// High contrast mode
    high_contrast: bool,
    /// Logical tab order (element IDs in navigation order)
    tab_order: Vec<String>,
    /// Current focus index in tab order
    current_focus_index: Option<usize>,
    /// Focus history for back navigation
    focus_history: Vec<String>,
}

impl EnhancedKeyboardNavigation {
    pub fn new() -> Self {
        Self {
            base_nav: KeyboardNavigationManager::new(),
            focus_ring_style: Style::default().fg(Color::White).add_modifier(Modifier::REVERSED),
            high_contrast: false,
            tab_order: Vec::new(),
            current_focus_index: None,
            focus_history: Vec::new(),
        }
    }

    /// Register an element for keyboard navigation
    pub fn register_element(&mut self, element_id: String, alternative: TextAlternative) {
        self.base_nav.register_element(alternative);
        self.tab_order.push(element_id);
    }

    /// Navigate to next focusable element (Tab)
    pub fn tab_next(&mut self) -> Option<AppMessage> {
        if self.tab_order.is_empty() {
            return None;
        }

        let next_index = match self.current_focus_index {
            None => 0,
            Some(current) => (current + 1) % self.tab_order.len(),
        };

        self.set_focus_by_index(next_index)
    }

    /// Navigate to previous focusable element (Shift+Tab)
    pub fn tab_previous(&mut self) -> Option<AppMessage> {
        if self.tab_order.is_empty() {
            return None;
        }

        let prev_index = match self.current_focus_index {
            None => self.tab_order.len() - 1,
            Some(0) => self.tab_order.len() - 1,
            Some(current) => current - 1,
        };

        self.set_focus_by_index(prev_index)
    }

    /// Set focus by tab order index
    fn set_focus_by_index(&mut self, index: usize) -> Option<AppMessage> {
        if index >= self.tab_order.len() {
            return None;
        }

        let element_id = &self.tab_order[index];

        // Update focus history
        if let Some(current_idx) = self.current_focus_index {
            if let Some(current_id) = self.tab_order.get(current_idx) {
                self.focus_history.push(current_id.clone());
                // Keep only last 10 items
                if self.focus_history.len() > 10 {
                    self.focus_history.remove(0);
                }
            }
        }

        self.current_focus_index = Some(index);
        self.base_nav.focus(element_id);

        Some(AppMessage::FocusChanged(element_id.clone()))
    }

    /// Get current focused element
    pub fn current_focus(&self) -> Option<&TextAlternative> {
        self.base_nav.current_focus()
    }

    /// Get focus ring style
    pub fn focus_ring_style(&self) -> Style {
        if self.high_contrast {
            Style::default().fg(Color::Black).bg(Color::White)
        } else {
            self.focus_ring_style
        }
    }

    /// Set high contrast mode
    pub fn set_high_contrast(&mut self, enabled: bool) {
        self.high_contrast = enabled;
    }

    /// Check if high contrast is enabled
    pub fn is_high_contrast(&self) -> bool {
        self.high_contrast
    }

    /// Get tab order for debugging
    pub fn tab_order(&self) -> &[String] {
        &self.tab_order
    }
}

/// High contrast theme manager
#[derive(Debug, Clone)]
pub struct HighContrastThemeManager {
    /// Available high contrast themes
    themes: HashMap<String, crate::style::Theme>,
    /// Current theme name
    current_theme: String,
}

impl HighContrastThemeManager {
    pub fn new() -> Self {
        let mut themes = HashMap::new();

        // Create high contrast themes
        themes.insert("high-contrast-dark".to_string(), Self::create_dark_high_contrast_theme());
        themes.insert("high-contrast-light".to_string(), Self::create_light_high_contrast_theme());
        themes.insert("high-contrast-yellow-blue".to_string(), Self::create_yellow_blue_theme());

        Self {
            themes,
            current_theme: "high-contrast-dark".to_string(),
        }
    }

    /// Get current high contrast theme
    pub fn current_theme(&self) -> &crate::style::Theme {
        self.themes.get(&self.current_theme).unwrap_or_else(|| {
            self.themes.get("high-contrast-dark").unwrap()
        })
    }

    /// Set current theme
    pub fn set_theme(&mut self, theme_name: String) -> bool {
        if self.themes.contains_key(&theme_name) {
            self.current_theme = theme_name;
            true
        } else {
            false
        }
    }

    /// Get available theme names
    pub fn available_themes(&self) -> Vec<String> {
        self.themes.keys().cloned().collect()
    }

    /// Create dark high contrast theme
    fn create_dark_high_contrast_theme() -> crate::style::Theme {
        crate::style::Theme {
            name: "High Contrast Dark".to_string(),
            primary: Color::White,
            secondary: Color::BrightWhite,
            background: Color::Black,
            foreground: Color::White,
            accent: Color::BrightWhite,
            error: Color::BrightRed,
            warning: Color::BrightYellow,
            success: Color::BrightGreen,
            info: Color::BrightBlue,
            muted: Color::Gray,
            border: Color::White,
        }
    }

    /// Create light high contrast theme
    fn create_light_high_contrast_theme() -> crate::style::Theme {
        crate::style::Theme {
            name: "High Contrast Light".to_string(),
            primary: Color::Black,
            secondary: Color::BrightBlack,
            background: Color::White,
            foreground: Color::Black,
            accent: Color::BrightBlack,
            error: Color::Red,
            warning: Color::DarkYellow,
            success: Color::DarkGreen,
            info: Color::DarkBlue,
            muted: Color::DarkGray,
            border: Color::Black,
        }
    }

    /// Create yellow-on-blue high contrast theme (for specific visual needs)
    fn create_yellow_blue_theme() -> crate::style::Theme {
        crate::style::Theme {
            name: "Yellow on Blue".to_string(),
            primary: Color::BrightYellow,
            secondary: Color::Yellow,
            background: Color::Blue,
            foreground: Color::BrightYellow,
            accent: Color::BrightWhite,
            error: Color::BrightRed,
            warning: Color::BrightWhite,
            success: Color::BrightGreen,
            info: Color::BrightCyan,
            muted: Color::Cyan,
            border: Color::BrightYellow,
        }
    }
}

/// Keyboard shortcut customizer
#[derive(Debug, Clone)]
pub struct KeyboardShortcutCustomizer {
    /// Default shortcuts
    defaults: HashMap<String, Vec<crossterm::event::KeyEvent>>,
    /// User customizations
    customizations: HashMap<String, Vec<crossterm::event::KeyEvent>>,
    /// Conflicts checker
    conflicts: HashMap<Vec<crossterm::event::KeyEvent>, Vec<String>>,
}

impl KeyboardShortcutCustomizer {
    pub fn new() -> Self {
        let mut defaults = HashMap::new();

        // Define default shortcuts
        defaults.insert("mode.chat".to_string(), vec![
            crossterm::event::KeyEvent {
                code: crossterm::event::KeyCode::Char('1'),
                modifiers: crossterm::event::KeyModifiers::CONTROL,
                kind: crossterm::event::KeyEventKind::Press,
                state: crossterm::event::KeyEventState::empty(),
            }
        ]);

        defaults.insert("mode.command".to_string(), vec![
            crossterm::event::KeyEvent {
                code: crossterm::event::KeyCode::Char('2'),
                modifiers: crossterm::event::KeyModifiers::CONTROL,
                kind: crossterm::event::KeyEventKind::Press,
                state: crossterm::event::KeyEventState::empty(),
            }
        ]);

        defaults.insert("focus.next".to_string(), vec![
            crossterm::event::KeyEvent {
                code: crossterm::event::KeyCode::Tab,
                modifiers: crossterm::event::KeyModifiers::empty(),
                kind: crossterm::event::KeyEventKind::Press,
                state: crossterm::event::KeyEventState::empty(),
            }
        ]);

        defaults.insert("focus.previous".to_string(), vec![
            crossterm::event::KeyEvent {
                code: crossterm::event::KeyCode::Tab,
                modifiers: crossterm::event::KeyModifiers::SHIFT,
                kind: crossterm::event::KeyEventKind::Press,
                state: crossterm::event::KeyEventState::empty(),
            }
        ]);

        Self {
            defaults,
            customizations: HashMap::new(),
            conflicts: HashMap::new(),
        }
    }

    /// Get shortcut for action
    pub fn get_shortcut(&self, action: &str) -> Option<&Vec<crossterm::event::KeyEvent>> {
        self.customizations.get(action)
            .or_else(|| self.defaults.get(action))
    }

    /// Set custom shortcut for action
    pub fn set_shortcut(&mut self, action: String, keys: Vec<crossterm::event::KeyEvent>) -> Result<(), String> {
        // Check for conflicts
        for (existing_keys, actions) in &self.conflicts {
            if existing_keys == &keys {
                return Err(format!("Shortcut conflicts with: {}", actions.join(", ")));
            }
        }

        // Remove old shortcut from conflicts
        if let Some(old_keys) = self.customizations.get(&action) {
            if let Some(actions) = self.conflicts.get_mut(old_keys) {
                actions.retain(|a| a != &action);
                if actions.is_empty() {
                    self.conflicts.remove(old_keys);
                }
            }
        }

        // Add new shortcut
        self.customizations.insert(action.clone(), keys.clone());
        self.conflicts.entry(keys).or_insert_with(Vec::new).push(action);

        Ok(())
    }

    /// Reset shortcut to default
    pub fn reset_shortcut(&mut self, action: &str) {
        if let Some(keys) = self.customizations.remove(action) {
            if let Some(actions) = self.conflicts.get_mut(&keys) {
                actions.retain(|a| a != action);
                if actions.is_empty() {
                    self.conflicts.remove(&keys);
                }
            }
        }
    }

    /// Get all available actions
    pub fn available_actions(&self) -> Vec<String> {
        let mut actions: Vec<String> = self.defaults.keys().cloned().collect();
        actions.sort();
        actions
    }

    /// Export shortcuts configuration
    pub fn export_config(&self) -> HashMap<String, Vec<String>> {
        let mut config = HashMap::new();

        for (action, keys) in &self.customizations {
            let key_strings: Vec<String> = keys.iter().map(|k| self.key_to_string(k)).collect();
            config.insert(action.clone(), key_strings);
        }

        config
    }

    /// Import shortcuts configuration
    pub fn import_config(&mut self, config: HashMap<String, Vec<String>>) -> Result<(), String> {
        for (action, key_strings) in config {
            let keys: Result<Vec<crossterm::event::KeyEvent>, String> = key_strings.iter()
                .map(|s| self.string_to_key(s))
                .collect();

            match keys {
                Ok(k) => {
                    self.set_shortcut(action, k)?;
                }
                Err(e) => return Err(format!("Invalid key format: {}", e)),
            }
        }

        Ok(())
    }

    /// Convert key event to string representation
    fn key_to_string(&self, key: &crossterm::event::KeyEvent) -> String {
        let mut parts = Vec::new();

        if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
            parts.push("Ctrl".to_string());
        }
        if key.modifiers.contains(crossterm::event::KeyModifiers::ALT) {
            parts.push("Alt".to_string());
        }
        if key.modifiers.contains(crossterm::event::KeyModifiers::SHIFT) {
            parts.push("Shift".to_string());
        }

        let key_part = match key.code {
            crossterm::event::KeyCode::Char(c) => c.to_string().to_uppercase(),
            crossterm::event::KeyCode::Enter => "Enter".to_string(),
            crossterm::event::KeyCode::Esc => "Escape".to_string(),
            crossterm::event::KeyCode::Backspace => "Backspace".to_string(),
            crossterm::event::KeyCode::Tab => "Tab".to_string(),
            crossterm::event::KeyCode::Up => "Up".to_string(),
            crossterm::event::KeyCode::Down => "Down".to_string(),
            crossterm::event::KeyCode::Left => "Left".to_string(),
            crossterm::event::KeyCode::Right => "Right".to_string(),
            _ => "Unknown".to_string(),
        };

        parts.push(key_part);
        parts.join("+")
    }

    /// Convert string representation to key event
    fn string_to_key(&self, s: &str) -> Result<crossterm::event::KeyEvent, String> {
        let parts: Vec<&str> = s.split('+').collect();
        let mut modifiers = crossterm::event::KeyModifiers::empty();
        let mut key_code = None;

        for part in parts {
            match part.to_lowercase().as_str() {
                "ctrl" => modifiers.insert(crossterm::event::KeyModifiers::CONTROL),
                "alt" => modifiers.insert(crossterm::event::KeyModifiers::ALT),
                "shift" => modifiers.insert(crossterm::event::KeyModifiers::SHIFT),
                "enter" => key_code = Some(crossterm::event::KeyCode::Enter),
                "escape" | "esc" => key_code = Some(crossterm::event::KeyCode::Esc),
                "backspace" => key_code = Some(crossterm::event::KeyCode::Backspace),
                "tab" => key_code = Some(crossterm::event::KeyCode::Tab),
                "up" => key_code = Some(crossterm::event::KeyCode::Up),
                "down" => key_code = Some(crossterm::event::KeyCode::Down),
                "left" => key_code = Some(crossterm::event::KeyCode::Left),
                "right" => key_code = Some(crossterm::event::KeyCode::Right),
                other => {
                    if other.len() == 1 {
                        key_code = Some(crossterm::event::KeyCode::Char(other.chars().next().unwrap()));
                    } else {
                        return Err(format!("Unknown key: {}", other));
                    }
                }
            }
        }

        match key_code {
            Some(code) => Ok(crossterm::event::KeyEvent {
                code,
                modifiers,
                kind: crossterm::event::KeyEventKind::Press,
                state: crossterm::event::KeyEventState::empty(),
            }),
            None => Err("No key code specified".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accessibility_config_default() {
        let config = AccessibilityConfig::default();
        assert!(!config.screen_reader_enabled);
        assert!(!config.high_contrast_enabled);
        assert!(!config.animations_disabled);
        assert!(config.announcements_enabled);
    }

    #[test]
    fn test_focus_indicator_style() {
        assert_eq!(FocusIndicatorStyle::Bracket.prefix(), "[");
        assert_eq!(FocusIndicatorStyle::Bracket.suffix(), "]");
        assert_eq!(FocusIndicatorStyle::Arrow.prefix(), "> ");
        assert_eq!(FocusIndicatorStyle::Arrow.suffix(), "");
    }

    #[test]
    fn test_text_alternative() {
        let alt = TextAlternative::new("btn1", "Submit button", ElementType::Button)
            .with_long_description("Click to submit the form");
        assert_eq!(alt.id, "btn1");
        assert_eq!(alt.short_description, "Submit button");
        assert!(alt.long_description.is_some());
    }

    #[test]
    fn test_element_type_role() {
        assert_eq!(ElementType::Button.role(), "button");
        assert_eq!(ElementType::Input.role(), "textbox");
        assert_eq!(ElementType::List.role(), "list");
    }

    #[test]
    fn test_screen_reader_announcer() {
        let mut announcer = ScreenReaderAnnouncer::new(true);
        announcer.announce("Test announcement", AnnouncementPriority::Normal);
        assert_eq!(announcer.announcements().len(), 1);
        assert_eq!(
            announcer.last_announcement().unwrap().text,
            "Test announcement"
        );
    }

    #[test]
    fn test_screen_reader_announcer_disabled() {
        let mut announcer = ScreenReaderAnnouncer::new(false);
        announcer.announce("Test", AnnouncementPriority::Normal);
        assert_eq!(announcer.announcements().len(), 0);
    }

    #[test]
    fn test_keyboard_navigation_manager() {
        let mut manager = KeyboardNavigationManager::new();
        let alt1 = TextAlternative::new("btn1", "Button 1", ElementType::Button);
        let alt2 = TextAlternative::new("btn2", "Button 2", ElementType::Button);

        manager.register_element(alt1);
        manager.register_element(alt2);

        assert!(manager.focus("btn1"));
        assert_eq!(manager.focused_element, Some("btn1".to_string()));

        let next = manager.focus_next();
        assert!(next.is_some());
        assert_eq!(manager.focused_element, Some("btn2".to_string()));
    }

    #[test]
    fn test_keyboard_navigation_wrap_around() {
        let mut manager = KeyboardNavigationManager::new();
        manager.register_element(TextAlternative::new(
            "btn1",
            "Button 1",
            ElementType::Button,
        ));
        manager.register_element(TextAlternative::new(
            "btn2",
            "Button 2",
            ElementType::Button,
        ));

        manager.focus("btn2");
        let _next = manager.focus_next();
        assert_eq!(manager.focused_element, Some("btn1".to_string()));
    }

    #[test]
    fn test_animation_config_default() {
        let config = AnimationConfig::default();
        assert!(config.enabled);
        assert_eq!(config.speed, 1.0);
        assert!(!config.reduce_motion);
    }

    #[test]
    fn test_animation_duration_calculation() {
        let config = AnimationConfig {
            enabled: true,
            speed: 2.0,
            reduce_motion: false,
        };
        // Base 100ms at 2x speed should be 50ms
        assert_eq!(config.duration_ms(100), 50);
    }

    #[test]
    fn test_animation_disabled() {
        let config = AnimationConfig {
            enabled: false,
            speed: 1.0,
            reduce_motion: false,
        };
        // Disabled animations should return 0 duration
        assert_eq!(config.duration_ms(100), 0);
    }

    #[test]
    fn test_animation_reduce_motion() {
        let config = AnimationConfig {
            enabled: true,
            speed: 1.0,
            reduce_motion: true,
        };
        // Reduce motion should return 0 duration
        assert_eq!(config.duration_ms(100), 0);
        assert!(!config.should_animate());
    }

    #[test]
    fn test_accessibility_config_animations() {
        let config = AccessibilityConfig::default();
        assert!(config.animations.enabled);
        assert!(config.animations.should_animate());
    }

    #[test]
    fn test_state_change_event() {
        let event = StateChangeEvent::new(
            "button",
            "disabled",
            "enabled",
            AnnouncementPriority::Normal,
        );
        assert_eq!(event.component, "button");
        assert_eq!(event.previous_state, "disabled");
        assert_eq!(event.new_state, "enabled");
        assert!(event.announcement_text().contains("button"));
    }

    #[test]
    fn test_focus_manager() {
        let mut manager = FocusManager::new();
        assert!(manager.focused_element.is_none());

        manager.set_focus("btn1");
        assert_eq!(manager.focused_element, Some("btn1".to_string()));

        manager.set_focus("btn2");
        assert_eq!(manager.focused_element, Some("btn2".to_string()));

        let restored = manager.restore_focus();
        assert_eq!(restored, Some("btn1".to_string()));
    }

    #[test]
    fn test_focus_manager_clear() {
        let mut manager = FocusManager::new();
        manager.set_focus("btn1");
        manager.clear_focus();
        assert!(manager.focused_element.is_none());
    }

    #[test]
    fn test_enhanced_keyboard_navigation() {
        let mut nav = super::EnhancedKeyboardNavigation::new();

        nav.register_element("btn1".to_string(), TextAlternative::new("btn1", "Button 1", ElementType::Button));
        nav.register_element("btn2".to_string(), TextAlternative::new("btn2", "Button 2", ElementType::Button));

        // Test tab navigation
        let _ = nav.tab_next();
        assert_eq!(nav.current_focus().map(|alt| alt.id.as_str()), Some("btn1"));

        let _ = nav.tab_next();
        assert_eq!(nav.current_focus().map(|alt| alt.id.as_str()), Some("btn2"));

        // Test wrap around
        let _ = nav.tab_next();
        assert_eq!(nav.current_focus().map(|alt| alt.id.as_str()), Some("btn1"));
    }

    #[test]
    fn test_high_contrast_theme_manager() {
        let manager = super::HighContrastThemeManager::new();

        let themes = manager.available_themes();
        assert!(themes.contains(&"high-contrast-dark".to_string()));
        assert!(themes.contains(&"high-contrast-light".to_string()));
        assert!(themes.contains(&"high-contrast-yellow-blue".to_string()));

        let theme = manager.current_theme();
        assert_eq!(theme.name, "High Contrast Dark");
    }

    #[test]
    fn test_keyboard_shortcut_customizer() {
        let mut customizer = super::KeyboardShortcutCustomizer::new();

        // Test default shortcuts
        let shortcut = customizer.get_shortcut("mode.chat");
        assert!(shortcut.is_some());

        // Test custom shortcut
        let custom_keys = vec![crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Char('x'),
            modifiers: crossterm::event::KeyModifiers::CONTROL,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        }];

        assert!(customizer.set_shortcut("test.action".to_string(), custom_keys.clone()).is_ok());
        assert_eq!(customizer.get_shortcut("test.action"), Some(&custom_keys));
    }

    #[test]
    fn test_keyboard_shortcut_key_conversion() {
        let customizer = super::KeyboardShortcutCustomizer::new();

        let key = crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Char('a'),
            modifiers: crossterm::event::KeyModifiers::CONTROL,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        };

        let key_string = customizer.key_to_string(&key);
        assert_eq!(key_string, "Ctrl+A");

        let converted_back = customizer.string_to_key(&key_string);
        assert!(converted_back.is_ok());
        assert_eq!(converted_back.unwrap().code, crossterm::event::KeyCode::Char('a'));
    }
}
