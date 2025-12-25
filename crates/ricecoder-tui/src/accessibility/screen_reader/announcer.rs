//! Screen reader announcer for state changes with ARIA-like support

use std::collections::{HashMap, VecDeque};

use crate::accessibility::{AriaLive, AriaRelevant, LiveRegion};

/// Screen reader announcer for state changes with ARIA-like support
#[derive(Debug, Clone, PartialEq)]
pub struct ScreenReaderAnnouncer {
    /// Whether announcements are enabled
    enabled: bool,
    /// Announcement history (for testing)
    history: Vec<Announcement>,
    /// Live regions for dynamic content updates
    live_regions: HashMap<String, LiveRegion>,
    /// Announcement queue for ordered delivery
    announcement_queue: VecDeque<Announcement>,
    /// Priority-based announcement processing
    processing_priority: bool,
}

/// An announcement for screen readers
#[derive(Debug, Clone, PartialEq)]
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
            live_regions: HashMap::new(),
            announcement_queue: VecDeque::new(),
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
    pub fn generate_aria_label(
        &self,
        element_id: &str,
        base_label: &str,
        state_info: Option<&str>,
    ) -> String {
        let mut label = base_label.to_string();
        if let Some(state) = state_info {
            label.push_str(&format!(", {}", state));
        }
        label
    }

    /// Generate ARIA-like description for an element
    pub fn generate_aria_description(
        &self,
        element_id: &str,
        description: &str,
        instructions: Option<&str>,
    ) -> String {
        let mut desc = description.to_string();
        if let Some(instr) = instructions {
            desc.push_str(&format!(". {}", instr));
        }
        desc
    }

    /// Announce focus changes with ARIA-like information
    pub fn announce_focus_change(
        &mut self,
        element_id: &str,
        element_type: &str,
        element_label: &str,
    ) {
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
        let priority = if success {
            AnnouncementPriority::Normal
        } else {
            AnnouncementPriority::High
        };
        self.announce(message, priority);
    }

    /// Create or update a live region
    pub fn update_live_region(
        &mut self,
        id: &str,
        content: &str,
        aria_live: AriaLive,
        atomic: bool,
        relevant: AriaRelevant,
    ) {
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
                self.announce(
                    format!("Live region {}: {}", id, content),
                    AnnouncementPriority::High,
                );
            }
            AriaLive::Polite => {
                self.announce(
                    format!("Live region {}: {}", id, content),
                    AnnouncementPriority::Low,
                );
            }
            AriaLive::Off => {
                // No announcement for live=off
            }
        }

        // Announce new live regions
        if is_new {
            self.announce(
                format!("Live region {} created", id),
                AnnouncementPriority::Low,
            );
        }
    }

    /// Remove a live region
    pub fn remove_live_region(&mut self, id: &str) {
        if self.live_regions.remove(id).is_some() {
            self.announce(
                format!("Live region {} removed", id),
                AnnouncementPriority::Low,
            );
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
            for announcement in high_priority
                .into_iter()
                .chain(normal_priority)
                .chain(low_priority)
            {
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
    pub fn live_regions(&self) -> &HashMap<String, LiveRegion> {
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

impl crate::Component for ScreenReaderAnnouncer {
    fn id(&self) -> crate::ComponentId {
        "screen_reader".to_string()
    }

    fn render(
        &self,
        _frame: &mut ratatui::Frame,
        _area: ratatui::layout::Rect,
        _model: &crate::AppModel,
    ) {
        // Screen reader doesn't render visually
    }

    fn update(&mut self, _message: &crate::AppMessage, _model: &crate::AppModel) -> bool {
        // Screen reader doesn't handle messages directly
        false
    }

    fn is_focused(&self) -> bool {
        false
    }

    fn set_focused(&mut self, _focused: bool) {
        // Screen reader doesn't have focus
    }

    fn is_visible(&self) -> bool {
        true
    }

    fn set_visible(&mut self, _visible: bool) {
        // Screen reader visibility is controlled by enabled state
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn bounds(&self) -> ratatui::layout::Rect {
        ratatui::layout::Rect::default()
    }

    fn set_bounds(&mut self, _bounds: ratatui::layout::Rect) {
        // Screen reader doesn't have bounds
    }

    fn handle_focus(
        &mut self,
        _direction: crate::components::FocusDirection,
    ) -> crate::components::FocusResult {
        crate::components::FocusResult::Handled
    }

    fn children(&self) -> Vec<&dyn crate::Component> {
        Vec::new()
    }

    fn children_mut(&mut self) -> Vec<&mut dyn crate::Component> {
        Vec::new()
    }

    fn find_child(&self, _id: &crate::ComponentId) -> Option<&dyn crate::Component> {
        None
    }

    fn find_child_mut(&mut self, _id: &crate::ComponentId) -> Option<&mut dyn crate::Component> {
        None
    }

    fn add_child(&mut self, _child: Box<dyn crate::Component>) {
        // Screen reader doesn't have children
    }

    fn remove_child(&mut self, _id: &crate::ComponentId) -> Option<Box<dyn crate::Component>> {
        None
    }

    fn z_index(&self) -> i32 {
        0
    }

    fn set_z_index(&mut self, _z_index: i32) {
        // Screen reader doesn't have z-index
    }

    fn can_focus(&self) -> bool {
        false
    }

    fn tab_order(&self) -> Option<usize> {
        None
    }

    fn set_tab_order(&mut self, _order: Option<usize>) {
        // Screen reader doesn't participate in tab order
    }

    fn clone_box(&self) -> Box<dyn crate::Component> {
        Box::new(ScreenReaderAnnouncer {
            enabled: self.enabled,
            history: self.history.clone(),
            live_regions: self.live_regions.clone(),
            announcement_queue: self.announcement_queue.clone(),
            processing_priority: self.processing_priority,
        })
    }
}
