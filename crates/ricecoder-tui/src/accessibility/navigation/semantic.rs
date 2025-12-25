//! Semantic navigation manager for landmark and heading navigation

use crate::accessibility::AriaRole;

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
        self.headings
            .sort_by_key(|h| h.bounds.map(|b| b.y).unwrap_or(0));
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
                let prev_pos = if pos == 0 {
                    landmark_ids.len() - 1
                } else {
                    pos - 1
                };
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
