//! Split view widget implementation

/// Split direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitDirection {
    /// Vertical split (left/right)
    Vertical,
    /// Horizontal split (top/bottom)
    Horizontal,
}

/// Split view widget
pub struct SplitViewWidget {
    /// Left/top panel content
    pub left_content: String,
    /// Right/bottom panel content
    pub right_content: String,
    /// Split ratio (0-100)
    pub split_ratio: u8,
    /// Split direction
    pub direction: SplitDirection,
    /// Active panel (0 = left/top, 1 = right/bottom)
    pub active_panel: usize,
    /// Left/top panel scroll
    pub left_scroll: usize,
    /// Right/bottom panel scroll
    pub right_scroll: usize,
}

impl SplitViewWidget {
    /// Create a new split view widget
    pub fn new() -> Self {
        Self {
            left_content: String::new(),
            right_content: String::new(),
            split_ratio: 50,
            direction: SplitDirection::Vertical,
            active_panel: 0,
            left_scroll: 0,
            right_scroll: 0,
        }
    }

    /// Create a horizontal split view
    pub fn horizontal() -> Self {
        Self {
            left_content: String::new(),
            right_content: String::new(),
            split_ratio: 50,
            direction: SplitDirection::Horizontal,
            active_panel: 0,
            left_scroll: 0,
            right_scroll: 0,
        }
    }

    /// Set left/top content
    pub fn set_left(&mut self, content: impl Into<String>) {
        self.left_content = content.into();
    }

    /// Set right/bottom content
    pub fn set_right(&mut self, content: impl Into<String>) {
        self.right_content = content.into();
    }

    /// Adjust split ratio
    pub fn adjust_split(&mut self, delta: i8) {
        let new_ratio = (self.split_ratio as i16 + delta as i16).clamp(20, 80) as u8;
        self.split_ratio = new_ratio;
    }

    /// Switch active panel
    pub fn switch_panel(&mut self) {
        self.active_panel = 1 - self.active_panel;
    }

    /// Get active panel content
    pub fn active_content(&self) -> &str {
        if self.active_panel == 0 {
            &self.left_content
        } else {
            &self.right_content
        }
    }

    /// Get active panel scroll
    pub fn active_scroll(&self) -> usize {
        if self.active_panel == 0 {
            self.left_scroll
        } else {
            self.right_scroll
        }
    }

    /// Scroll active panel up
    pub fn scroll_up(&mut self) {
        if self.active_panel == 0 {
            if self.left_scroll > 0 {
                self.left_scroll -= 1;
            }
        } else if self.right_scroll > 0 {
            self.right_scroll -= 1;
        }
    }

    /// Scroll active panel down
    pub fn scroll_down(&mut self) {
        if self.active_panel == 0 {
            self.left_scroll += 1;
        } else {
            self.right_scroll += 1;
        }
    }
}

impl Default for SplitViewWidget {
    fn default() -> Self {
        Self::new()
    }
}
