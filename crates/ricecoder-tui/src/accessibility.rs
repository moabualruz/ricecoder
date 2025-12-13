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
use crate::model::AppMessage;
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

    /// Merge two accessibility configurations (self takes precedence over other)
    pub fn merge(mut self, other: Self) -> Self {
        // Only override if values are not default
        if other.screen_reader_enabled != Self::default().screen_reader_enabled {
            self.screen_reader_enabled = other.screen_reader_enabled;
        }

        if other.high_contrast_enabled != Self::default().high_contrast_enabled {
            self.high_contrast_enabled = other.high_contrast_enabled;
        }

        if other.animations_disabled != Self::default().animations_disabled {
            self.animations_disabled = other.animations_disabled;
        }

        if other.announcements_enabled != Self::default().announcements_enabled {
            self.announcements_enabled = other.announcements_enabled;
        }

        if other.font_size_multiplier != Self::default().font_size_multiplier {
            self.font_size_multiplier = other.font_size_multiplier;
        }

        if other.large_click_targets != Self::default().large_click_targets {
            self.large_click_targets = other.large_click_targets;
        }

        if other.auto_advance != Self::default().auto_advance {
            self.auto_advance = other.auto_advance;
        }

        self
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

/// Screen reader announcer for state changes with ARIA-like support
#[derive(Debug, Clone)]
pub struct ScreenReaderAnnouncer {
    /// Whether announcements are enabled
    enabled: bool,
    /// Announcement history (for testing)
    history: Vec<Announcement>,
    /// Live regions for dynamic content updates
    live_regions: std::collections::HashMap<String, LiveRegion>,
    /// Announcement queue for ordered delivery
    announcement_queue: std::collections::VecDeque<Announcement>,
    /// Priority-based announcement processing
    processing_priority: bool,
}

/// Live region for dynamic content updates (ARIA live regions)
#[derive(Debug, Clone)]
pub struct LiveRegion {
    pub id: String,
    pub content: String,
    pub aria_live: AriaLive,
    pub aria_atomic: bool,
    pub aria_relevant: AriaRelevant,
    pub last_update: std::time::Instant,
}

/// ARIA live property values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AriaLive {
    Off,      // No announcements
    Polite,   // Announce when user is idle
    Assertive, // Announce immediately
}

/// ARIA relevant property values
#[derive(Debug, Clone)]
pub enum AriaRelevant {
    Additions,      // Only additions
    Removals,       // Only removals
    Text,          // Text changes
    All,           // All changes
}

/// ARIA-like semantic roles for UI elements
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AriaRole {
    Banner,
    Navigation,
    Main,
    Complementary,
    ContentInfo,
    Search,
    Form,
    Button,
    Link,
    Heading,
    List,
    ListItem,
    Status,
    Alert,
    Dialog,
    Menu,
    MenuItem,
    Tab,
    TabPanel,
    ProgressBar,
    TextBox,
    CheckBox,
    Radio,
    Slider,
    SpinButton,
}

/// ARIA-like properties for accessibility
#[derive(Debug, Clone)]
pub struct AriaProperties {
    pub role: Option<AriaRole>,
    pub label: Option<String>,
    pub description: Option<String>,
    pub expanded: Option<bool>,
    pub selected: Option<bool>,
    pub checked: Option<bool>,
    pub disabled: Option<bool>,
    pub required: Option<bool>,
    pub invalid: Option<bool>,
    pub level: Option<u32>, // For headings
    pub value_text: Option<String>,
    pub value_min: Option<f64>,
    pub value_max: Option<f64>,
    pub value_now: Option<f64>,
}

impl Default for AriaProperties {
    fn default() -> Self {
        Self {
            role: None,
            label: None,
            description: None,
            expanded: None,
            selected: None,
            checked: None,
            disabled: None,
            required: None,
            invalid: None,
            level: None,
            value_text: None,
            value_min: None,
            value_max: None,
            value_now: None,
        }
    }
}

impl AriaProperties {
    /// Create a new AriaProperties with a specific role
    pub fn with_role(role: AriaRole) -> Self {
        Self {
            role: Some(role),
            ..Default::default()
        }
    }

    /// Set the label
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set the description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Generate screen reader description
    pub fn screen_reader_description(&self) -> String {
        let mut parts = Vec::new();

        if let Some(role) = &self.role {
            parts.push(format!("{:?}", role).to_lowercase().replace('_', " "));
        }

        if let Some(label) = &self.label {
            parts.push(label.clone());
        }

        if let Some(description) = &self.description {
            parts.push(description.clone());
        }

        if let Some(expanded) = self.expanded {
            parts.push(if expanded { "expanded".to_string() } else { "collapsed".to_string() });
        }

        if let Some(selected) = self.selected {
            if selected {
                parts.push("selected".to_string());
            }
        }

        if let Some(checked) = self.checked {
            parts.push(if checked { "checked".to_string() } else { "unchecked".to_string() });
        }

        if let Some(disabled) = self.disabled {
            if disabled {
                parts.push("disabled".to_string());
            }
        }

        if let Some(required) = self.required {
            if required {
                parts.push("required".to_string());
            }
        }

        if let Some(invalid) = self.invalid {
            if invalid {
                parts.push("invalid".to_string());
            }
        }

        if let Some(level) = self.level {
            parts.push(format!("level {}", level));
        }

        if let Some(value_text) = &self.value_text {
            parts.push(value_text.clone());
        } else if let Some(value_now) = self.value_now {
            if let (Some(min), Some(max)) = (self.value_min, self.value_max) {
                parts.push(format!("{:.0}%", ((value_now - min) / (max - min)) * 100.0));
            } else {
                parts.push(format!("{:.0}", value_now));
            }
        }

        parts.join(", ")
    }
}

/// Semantic navigation manager for landmark and heading navigation
#[derive(Debug)]
pub struct SemanticNavigator {
    /// Registered landmarks
    landmarks: std::collections::HashMap<String, Landmark>,
    /// Registered headings
    headings: Vec<Heading>,
    /// Current navigation position
    current_position: NavigationPosition,
}

/// Landmark for semantic navigation
#[derive(Debug, Clone)]
pub struct Landmark {
    pub id: String,
    pub role: AriaRole,
    pub label: String,
    pub bounds: Option<ratatui::layout::Rect>,
    pub accessible: bool,
}

/// Heading for semantic navigation
#[derive(Debug, Clone)]
pub struct Heading {
    pub id: String,
    pub level: u32,
    pub text: String,
    pub bounds: Option<ratatui::layout::Rect>,
    pub accessible: bool,
}

/// Current navigation position
#[derive(Debug, Clone)]
pub enum NavigationPosition {
    Landmark(String),
    Heading(usize),
    None,
}

/// Navigation direction
#[derive(Debug, Clone, Copy)]
pub enum NavigationDirection {
    Next,
    Previous,
    First,
    Last,
}

impl SemanticNavigator {
    /// Create a new semantic navigator
    pub fn new() -> Self {
        Self {
            landmarks: std::collections::HashMap::new(),
            headings: Vec::new(),
            current_position: NavigationPosition::None,
        }
    }

    /// Register a landmark
    pub fn register_landmark(&mut self, landmark: Landmark) {
        self.landmarks.insert(landmark.id.clone(), landmark);
    }

    /// Register a heading
    pub fn register_heading(&mut self, heading: Heading) {
        self.headings.push(heading);
        // Keep headings sorted by their position in the document
        self.headings.sort_by_key(|h| h.bounds.map(|b| b.y).unwrap_or(0));
    }

    /// Unregister a landmark
    pub fn unregister_landmark(&mut self, id: &str) {
        self.landmarks.remove(id);
    }

    /// Unregister a heading
    pub fn unregister_heading(&mut self, id: &str) {
        self.headings.retain(|h| h.id != id);
    }

    /// Navigate to next landmark
    pub fn next_landmark(&mut self) -> Option<&Landmark> {
        let current_id = match &self.current_position {
            NavigationPosition::Landmark(id) => Some(id.clone()),
            _ => None,
        };

        let landmark_ids: Vec<String> = self.landmarks.keys().cloned().collect();
        let next_id = if let Some(current) = current_id {
            if let Some(pos) = landmark_ids.iter().position(|id| id == &current) {
                let next_pos = (pos + 1) % landmark_ids.len();
                landmark_ids.get(next_pos).cloned()
            } else {
                landmark_ids.first().cloned()
            }
        } else {
            landmark_ids.first().cloned()
        };

        if let Some(id) = next_id {
            self.current_position = NavigationPosition::Landmark(id.clone());
            self.landmarks.get(&id)
        } else {
            None
        }
    }

    /// Navigate to previous landmark
    pub fn previous_landmark(&mut self) -> Option<&Landmark> {
        let current_id = match &self.current_position {
            NavigationPosition::Landmark(id) => Some(id.clone()),
            _ => None,
        };

        let landmark_ids: Vec<String> = self.landmarks.keys().cloned().collect();
        let prev_id = if let Some(current) = current_id {
            if let Some(pos) = landmark_ids.iter().position(|id| id == &current) {
                let prev_pos = if pos == 0 { landmark_ids.len() - 1 } else { pos - 1 };
                landmark_ids.get(prev_pos).cloned()
            } else {
                landmark_ids.last().cloned()
            }
        } else {
            landmark_ids.last().cloned()
        };

        if let Some(id) = prev_id {
            self.current_position = NavigationPosition::Landmark(id.clone());
            self.landmarks.get(&id)
        } else {
            None
        }
    }

    /// Navigate to next heading
    pub fn next_heading(&mut self) -> Option<&Heading> {
        let current_idx = match &self.current_position {
            NavigationPosition::Heading(idx) => Some(*idx),
            _ => None,
        };

        let next_idx = if let Some(current) = current_idx {
            if current + 1 < self.headings.len() {
                current + 1
            } else {
                0
            }
        } else {
            0
        };

        if let Some(heading) = self.headings.get(next_idx) {
            self.current_position = NavigationPosition::Heading(next_idx);
            Some(heading)
        } else {
            None
        }
    }

    /// Navigate to previous heading
    pub fn previous_heading(&mut self) -> Option<&Heading> {
        let current_idx = match &self.current_position {
            NavigationPosition::Heading(idx) => Some(*idx),
            _ => None,
        };

        let prev_idx = if let Some(current) = current_idx {
            if current > 0 {
                current - 1
            } else {
                self.headings.len().saturating_sub(1)
            }
        } else {
            self.headings.len().saturating_sub(1)
        };

        if let Some(heading) = self.headings.get(prev_idx) {
            self.current_position = NavigationPosition::Heading(prev_idx);
            Some(heading)
        } else {
            None
        }
    }

    /// Navigate to next heading of specific level
    pub fn next_heading_level(&mut self, level: u32) -> Option<&Heading> {
        let current_idx = match &self.current_position {
            NavigationPosition::Heading(idx) => Some(*idx),
            _ => None,
        };

        let start_idx = current_idx.map(|i| i + 1).unwrap_or(0);

        for (idx, heading) in self.headings.iter().enumerate().skip(start_idx) {
            if heading.level == level {
                self.current_position = NavigationPosition::Heading(idx);
                return Some(heading);
            }
        }

        // Wrap around to beginning
        for (idx, heading) in self.headings.iter().enumerate() {
            if heading.level == level {
                self.current_position = NavigationPosition::Heading(idx);
                return Some(heading);
            }
        }

        None
    }

    /// Get all landmarks
    pub fn landmarks(&self) -> &std::collections::HashMap<String, Landmark> {
        &self.landmarks
    }

    /// Get all headings
    pub fn headings(&self) -> &[Heading] {
        &self.headings
    }

    /// Get current navigation position
    pub fn current_position(&self) -> &NavigationPosition {
        &self.current_position
    }

    /// Clear all registered elements
    pub fn clear(&mut self) {
        self.landmarks.clear();
        self.headings.clear();
        self.current_position = NavigationPosition::None;
    }
}

impl Default for SemanticNavigator {
    fn default() -> Self {
        Self::new()
    }
}

/// Vim-like input mode manager
#[derive(Debug, Clone)]
pub struct VimModeManager {
    /// Current input mode
    current_mode: InputMode,
    /// Previous mode (for switching back)
    previous_mode: InputMode,
    /// Mode-specific keybindings
    mode_keybindings: std::collections::HashMap<InputMode, std::collections::HashMap<String, ModeAction>>,
    /// Mode indicators
    mode_indicators: std::collections::HashMap<InputMode, ModeIndicator>,
}

/// Input modes (vim-like)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    /// Normal mode (navigation and commands)
    Normal,
    /// Insert mode (text input)
    Insert,
    /// Visual mode (text selection)
    Visual,
    /// Command mode (ex commands)
    Command,
    /// Replace mode (replace text)
    Replace,
}

/// Actions that can be performed in different modes
#[derive(Debug, Clone)]
pub enum ModeAction {
    SwitchMode(InputMode),
    ExecuteCommand(String),
    MoveCursor(Movement),
    InsertText(String),
    DeleteText(DeleteOperation),
    Custom(String),
}

/// Cursor movement operations
#[derive(Debug, Clone)]
pub enum Movement {
    Left,
    Right,
    Up,
    Down,
    WordForward,
    WordBackward,
    LineStart,
    LineEnd,
    DocumentStart,
    DocumentEnd,
}

/// Text deletion operations
#[derive(Debug, Clone)]
pub enum DeleteOperation {
    Character,
    Word,
    Line,
    Selection,
}

/// Mode indicator for UI display
#[derive(Debug, Clone)]
pub struct ModeIndicator {
    pub text: String,
    pub style: ModeIndicatorStyle,
}

/// Visual style for mode indicators
#[derive(Debug, Clone)]
pub enum ModeIndicatorStyle {
    Normal,
    Insert,
    Visual,
    Command,
    Replace,
}

impl VimModeManager {
    /// Create a new vim mode manager
    pub fn new() -> Self {
        let mut manager = Self {
            current_mode: InputMode::Normal,
            previous_mode: InputMode::Normal,
            mode_keybindings: std::collections::HashMap::new(),
            mode_indicators: std::collections::HashMap::new(),
        };

        manager.initialize_default_keybindings();
        manager.initialize_mode_indicators();
        manager
    }

    /// Initialize default keybindings for each mode
    fn initialize_default_keybindings(&mut self) {
        // Normal mode keybindings
        let mut normal_bindings = std::collections::HashMap::new();
        normal_bindings.insert("i".to_string(), ModeAction::SwitchMode(InputMode::Insert));
        normal_bindings.insert("v".to_string(), ModeAction::SwitchMode(InputMode::Visual));
        normal_bindings.insert(":".to_string(), ModeAction::SwitchMode(InputMode::Command));
        normal_bindings.insert("R".to_string(), ModeAction::SwitchMode(InputMode::Replace));
        normal_bindings.insert("h".to_string(), ModeAction::MoveCursor(Movement::Left));
        normal_bindings.insert("j".to_string(), ModeAction::MoveCursor(Movement::Down));
        normal_bindings.insert("k".to_string(), ModeAction::MoveCursor(Movement::Up));
        normal_bindings.insert("l".to_string(), ModeAction::MoveCursor(Movement::Right));
        normal_bindings.insert("w".to_string(), ModeAction::MoveCursor(Movement::WordForward));
        normal_bindings.insert("b".to_string(), ModeAction::MoveCursor(Movement::WordBackward));
        normal_bindings.insert("0".to_string(), ModeAction::MoveCursor(Movement::LineStart));
        normal_bindings.insert("$".to_string(), ModeAction::MoveCursor(Movement::LineEnd));
        normal_bindings.insert("gg".to_string(), ModeAction::MoveCursor(Movement::DocumentStart));
        normal_bindings.insert("G".to_string(), ModeAction::MoveCursor(Movement::DocumentEnd));
        normal_bindings.insert("x".to_string(), ModeAction::DeleteText(DeleteOperation::Character));
        normal_bindings.insert("dw".to_string(), ModeAction::DeleteText(DeleteOperation::Word));
        normal_bindings.insert("dd".to_string(), ModeAction::DeleteText(DeleteOperation::Line));

        self.mode_keybindings.insert(InputMode::Normal, normal_bindings);

        // Insert mode keybindings (minimal - mostly for escaping back to normal)
        let mut insert_bindings = std::collections::HashMap::new();
        insert_bindings.insert("Esc".to_string(), ModeAction::SwitchMode(InputMode::Normal));

        self.mode_keybindings.insert(InputMode::Insert, insert_bindings);

        // Visual mode keybindings
        let mut visual_bindings = std::collections::HashMap::new();
        visual_bindings.insert("Esc".to_string(), ModeAction::SwitchMode(InputMode::Normal));
        visual_bindings.insert("d".to_string(), ModeAction::DeleteText(DeleteOperation::Selection));

        self.mode_keybindings.insert(InputMode::Visual, visual_bindings);

        // Command mode keybindings
        let mut command_bindings = std::collections::HashMap::new();
        command_bindings.insert("Esc".to_string(), ModeAction::SwitchMode(InputMode::Normal));
        command_bindings.insert("Enter".to_string(), ModeAction::ExecuteCommand("execute_command".to_string()));

        self.mode_keybindings.insert(InputMode::Command, command_bindings);
    }

    /// Initialize mode indicators
    fn initialize_mode_indicators(&mut self) {
        self.mode_indicators.insert(InputMode::Normal, ModeIndicator {
            text: "NORMAL".to_string(),
            style: ModeIndicatorStyle::Normal,
        });

        self.mode_indicators.insert(InputMode::Insert, ModeIndicator {
            text: "INSERT".to_string(),
            style: ModeIndicatorStyle::Insert,
        });

        self.mode_indicators.insert(InputMode::Visual, ModeIndicator {
            text: "VISUAL".to_string(),
            style: ModeIndicatorStyle::Visual,
        });

        self.mode_indicators.insert(InputMode::Command, ModeIndicator {
            text: "COMMAND".to_string(),
            style: ModeIndicatorStyle::Command,
        });

        self.mode_indicators.insert(InputMode::Replace, ModeIndicator {
            text: "REPLACE".to_string(),
            style: ModeIndicatorStyle::Replace,
        });
    }

    /// Get current input mode
    pub fn current_mode(&self) -> InputMode {
        self.current_mode
    }

    /// Switch to a different input mode
    pub fn switch_mode(&mut self, mode: InputMode) {
        self.previous_mode = self.current_mode;
        self.current_mode = mode;
    }

    /// Switch back to previous mode
    pub fn switch_to_previous_mode(&mut self) {
        std::mem::swap(&mut self.current_mode, &mut self.previous_mode);
    }

    /// Handle key input based on current mode
    pub fn handle_key(&mut self, key: &str) -> Option<ModeAction> {
        if let Some(bindings) = self.mode_keybindings.get(&self.current_mode) {
            if let Some(action) = bindings.get(key) {
                return Some(action.clone());
            }
        }
        None
    }

    /// Add a custom keybinding for a mode
    pub fn add_keybinding(&mut self, mode: InputMode, key: String, action: ModeAction) {
        self.mode_keybindings.entry(mode).or_insert_with(std::collections::HashMap::new).insert(key, action);
    }

    /// Remove a keybinding
    pub fn remove_keybinding(&mut self, mode: InputMode, key: &str) {
        if let Some(bindings) = self.mode_keybindings.get_mut(&mode) {
            bindings.remove(key);
        }
    }

    /// Get mode indicator for current mode
    pub fn current_mode_indicator(&self) -> Option<&ModeIndicator> {
        self.mode_indicators.get(&self.current_mode)
    }

    /// Get all available keybindings for current mode
    pub fn current_mode_keybindings(&self) -> Option<&std::collections::HashMap<String, ModeAction>> {
        self.mode_keybindings.get(&self.current_mode)
    }

    /// Check if vim mode is enabled
    pub fn is_vim_mode_enabled(&self) -> bool {
        true // Always enabled when this manager is used
    }

    /// Get mode-specific help text
    pub fn mode_help(&self) -> String {
        match self.current_mode {
            InputMode::Normal => "Normal mode: h/j/k/l to move, i to insert, v for visual, : for command".to_string(),
            InputMode::Insert => "Insert mode: Type to insert text, Esc to return to normal mode".to_string(),
            InputMode::Visual => "Visual mode: Select text, d to delete selection, Esc to exit".to_string(),
            InputMode::Command => "Command mode: Type commands, Enter to execute, Esc to cancel".to_string(),
            InputMode::Replace => "Replace mode: Type to replace text, Esc to return to normal mode".to_string(),
        }
    }
}

impl Default for VimModeManager {
    fn default() -> Self {
        Self::new()
    }
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
            live_regions: std::collections::HashMap::new(),
            announcement_queue: std::collections::VecDeque::new(),
            processing_priority: true,
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

    /// Create or update a live region
    pub fn update_live_region(&mut self, id: &str, content: &str, aria_live: AriaLive, atomic: bool, relevant: AriaRelevant) {
        if !self.enabled {
            return;
        }

        let region = LiveRegion {
            id: id.to_string(),
            content: content.to_string(),
            aria_live,
            aria_atomic: atomic,
            aria_relevant: relevant,
            last_update: std::time::Instant::now(),
        };

        let is_new = !self.live_regions.contains_key(id);
        self.live_regions.insert(id.to_string(), region.clone());

        // Announce live region updates based on ARIA live property
        match aria_live {
            AriaLive::Assertive => {
                self.announce(format!("Live region {}: {}", id, content), AnnouncementPriority::High);
            }
            AriaLive::Polite => {
                self.announce(format!("Live region {}: {}", id, content), AnnouncementPriority::Low);
            }
            AriaLive::Off => {
                // No announcement for live=off
            }
        }

        // Announce new live regions
        if is_new {
            self.announce(format!("Live region {} created", id), AnnouncementPriority::Low);
        }
    }

    /// Remove a live region
    pub fn remove_live_region(&mut self, id: &str) {
        if self.live_regions.remove(id).is_some() {
            self.announce(format!("Live region {} removed", id), AnnouncementPriority::Low);
        }
    }

    /// Queue an announcement for later delivery
    pub fn queue_announcement(&mut self, text: impl Into<String>, priority: AnnouncementPriority) {
        let announcement = Announcement {
            text: text.into(),
            priority,
        };
        self.announcement_queue.push_back(announcement);
    }

    /// Process queued announcements
    pub fn process_queue(&mut self) {
        if !self.enabled {
            self.announcement_queue.clear();
            return;
        }

        // Process high priority announcements first if priority processing is enabled
        if self.processing_priority {
            let mut high_priority = Vec::new();
            let mut normal_priority = Vec::new();
            let mut low_priority = Vec::new();

            while let Some(announcement) = self.announcement_queue.pop_front() {
                match announcement.priority {
                    AnnouncementPriority::High => high_priority.push(announcement),
                    AnnouncementPriority::Normal => normal_priority.push(announcement),
                    AnnouncementPriority::Low => low_priority.push(announcement),
                }
            }

            // Process in priority order
            for announcement in high_priority.into_iter().chain(normal_priority).chain(low_priority) {
                self.announce(announcement.text, announcement.priority);
            }
        } else {
            // Process in FIFO order
            while let Some(announcement) = self.announcement_queue.pop_front() {
                self.announce(announcement.text, announcement.priority);
            }
        }
    }

    /// Get all live regions
    pub fn live_regions(&self) -> &std::collections::HashMap<String, LiveRegion> {
        &self.live_regions
    }

    /// Get announcement queue length
    pub fn queue_length(&self) -> usize {
        self.announcement_queue.len()
    }

    /// Enable priority processing of announcements
    pub fn enable_priority_processing(&mut self) {
        self.processing_priority = true;
    }

    /// Disable priority processing of announcements
    pub fn disable_priority_processing(&mut self) {
        self.processing_priority = false;
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

/// Keyboard shortcut help system
#[derive(Debug)]
pub struct KeyboardShortcutHelp {
    /// Available shortcuts organized by category
    shortcuts: std::collections::HashMap<String, Vec<KeyboardShortcut>>,
    /// Current search filter
    search_filter: String,
    /// Filtered shortcuts based on search
    filtered_shortcuts: Vec<(String, Vec<KeyboardShortcut>)>,
}

/// Keyboard shortcut definition
#[derive(Debug, Clone)]
pub struct KeyboardShortcut {
    pub keys: String,
    pub description: String,
    pub category: String,
    pub context: Option<String>, // Optional context (e.g., "when file is open")
}

impl KeyboardShortcutHelp {
    /// Create a new keyboard shortcut help system
    pub fn new() -> Self {
        Self {
            shortcuts: std::collections::HashMap::new(),
            search_filter: String::new(),
            filtered_shortcuts: Vec::new(),
        }
    }

    /// Add a keyboard shortcut
    pub fn add_shortcut(&mut self, shortcut: KeyboardShortcut) {
        self.shortcuts.entry(shortcut.category.clone())
            .or_insert_with(Vec::new)
            .push(shortcut);
        self.update_filtered_shortcuts();
    }

    /// Add multiple shortcuts
    pub fn add_shortcuts(&mut self, shortcuts: Vec<KeyboardShortcut>) {
        for shortcut in shortcuts {
            self.add_shortcut(shortcut);
        }
    }

    /// Remove shortcuts by category
    pub fn remove_category(&mut self, category: &str) {
        self.shortcuts.remove(category);
        self.update_filtered_shortcuts();
    }

    /// Set search filter
    pub fn set_search_filter(&mut self, filter: String) {
        self.search_filter = filter;
        self.update_filtered_shortcuts();
    }

    /// Get filtered shortcuts
    pub fn filtered_shortcuts(&self) -> &[(String, Vec<KeyboardShortcut>)] {
        &self.filtered_shortcuts
    }

    /// Get all shortcuts
    pub fn all_shortcuts(&self) -> &std::collections::HashMap<String, Vec<KeyboardShortcut>> {
        &self.shortcuts
    }

    /// Get shortcuts for a specific category
    pub fn category_shortcuts(&self, category: &str) -> Option<&[KeyboardShortcut]> {
        self.shortcuts.get(category).map(|v| v.as_slice())
    }

    /// Search shortcuts by description or keys
    pub fn search(&self, query: &str) -> Vec<&KeyboardShortcut> {
        let query_lower = query.to_lowercase();
        self.shortcuts.values()
            .flatten()
            .filter(|shortcut|
                shortcut.description.to_lowercase().contains(&query_lower) ||
                shortcut.keys.to_lowercase().contains(&query_lower) ||
                shortcut.category.to_lowercase().contains(&query_lower)
            )
            .collect()
    }

    /// Get context-sensitive shortcuts
    pub fn context_shortcuts(&self, context: &str) -> Vec<&KeyboardShortcut> {
        self.shortcuts.values()
            .flatten()
            .filter(|shortcut| {
                shortcut.context.as_ref().map_or(true, |ctx| ctx == context)
            })
            .collect()
    }

    /// Generate help text for display
    pub fn generate_help_text(&self, max_width: usize) -> Vec<String> {
        let mut lines = Vec::new();

        if !self.search_filter.is_empty() {
            lines.push(format!("Search: {}", self.search_filter));
            lines.push("".to_string());
        }

        for (category, shortcuts) in &self.filtered_shortcuts {
            lines.push(format!("{}:", category.to_uppercase()));
            lines.push("".to_string());

            for shortcut in shortcuts {
                let key_part = format!("  {:<15}", shortcut.keys);
                let desc_part = if key_part.len() + shortcut.description.len() > max_width {
                    format!("{}...", &shortcut.description[..max_width.saturating_sub(key_part.len() + 3)])
                } else {
                    shortcut.description.clone()
                };

                lines.push(format!("{}{}", key_part, desc_part));

                if let Some(context) = &shortcut.context {
                    lines.push(format!("    ({})", context));
                }
            }

            lines.push("".to_string());
        }

        lines
    }

    /// Clear all shortcuts
    pub fn clear(&mut self) {
        self.shortcuts.clear();
        self.filtered_shortcuts.clear();
        self.search_filter.clear();
    }

    /// Update filtered shortcuts based on search filter
    fn update_filtered_shortcuts(&mut self) {
        self.filtered_shortcuts.clear();

        if self.search_filter.is_empty() {
            // Show all categories
            for (category, shortcuts) in &self.shortcuts {
                self.filtered_shortcuts.push((category.clone(), shortcuts.clone()));
            }
        } else {
            // Filter by search
            let filter_lower = self.search_filter.to_lowercase();
            for (category, shortcuts) in &self.shortcuts {
                let filtered: Vec<KeyboardShortcut> = shortcuts.iter()
                    .filter(|shortcut|
                        shortcut.description.to_lowercase().contains(&filter_lower) ||
                        shortcut.keys.to_lowercase().contains(&filter_lower) ||
                        category.to_lowercase().contains(&filter_lower)
                    )
                    .cloned()
                    .collect();

                if !filtered.is_empty() {
                    self.filtered_shortcuts.push((category.clone(), filtered));
                }
            }
        }

        // Sort categories alphabetically
        self.filtered_shortcuts.sort_by(|a, b| a.0.cmp(&b.0));
    }
}

impl Default for KeyboardShortcutHelp {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize default keyboard shortcuts for RiceCoder
pub fn initialize_default_shortcuts() -> Vec<KeyboardShortcut> {
    vec![
        // General shortcuts
        KeyboardShortcut {
            keys: "Ctrl+C".to_string(),
            description: "Exit application".to_string(),
            category: "General".to_string(),
            context: None,
        },
        KeyboardShortcut {
            keys: "Ctrl+Z".to_string(),
            description: "Undo last action".to_string(),
            category: "General".to_string(),
            context: None,
        },
        KeyboardShortcut {
            keys: "Ctrl+Y".to_string(),
            description: "Redo last action".to_string(),
            category: "General".to_string(),
            context: None,
        },
        KeyboardShortcut {
            keys: "Ctrl+S".to_string(),
            description: "Save current file".to_string(),
            category: "File".to_string(),
            context: Some("when file is open".to_string()),
        },
        KeyboardShortcut {
            keys: "Ctrl+O".to_string(),
            description: "Open file".to_string(),
            category: "File".to_string(),
            context: None,
        },
        KeyboardShortcut {
            keys: "Ctrl+N".to_string(),
            description: "New file".to_string(),
            category: "File".to_string(),
            context: None,
        },
        KeyboardShortcut {
            keys: "F1".to_string(),
            description: "Show help".to_string(),
            category: "Help".to_string(),
            context: None,
        },
        KeyboardShortcut {
            keys: "Ctrl+/".to_string(),
            description: "Toggle comment".to_string(),
            category: "Editing".to_string(),
            context: Some("when text is selected".to_string()),
        },
        KeyboardShortcut {
            keys: "Ctrl+F".to_string(),
            description: "Find in file".to_string(),
            category: "Search".to_string(),
            context: Some("when file is open".to_string()),
        },
        KeyboardShortcut {
            keys: "Ctrl+H".to_string(),
            description: "Replace in file".to_string(),
            category: "Search".to_string(),
            context: Some("when file is open".to_string()),
        },
        KeyboardShortcut {
            keys: "Ctrl+P".to_string(),
            description: "Command palette".to_string(),
            category: "Navigation".to_string(),
            context: None,
        },
        KeyboardShortcut {
            keys: "Ctrl+B".to_string(),
            description: "Toggle sidebar".to_string(),
            category: "View".to_string(),
            context: None,
        },
        KeyboardShortcut {
            keys: "F11".to_string(),
            description: "Toggle fullscreen".to_string(),
            category: "View".to_string(),
            context: None,
        },
    ]
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


