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
        }
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
    pub fn new(id: impl Into<String>, short_desc: impl Into<String>, element_type: ElementType) -> Self {
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
        self.element_descriptions.insert(alternative.id.clone(), alternative);
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
        assert_eq!(announcer.last_announcement().unwrap().text, "Test announcement");
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
        manager.register_element(TextAlternative::new("btn1", "Button 1", ElementType::Button));
        manager.register_element(TextAlternative::new("btn2", "Button 2", ElementType::Button));

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
}
