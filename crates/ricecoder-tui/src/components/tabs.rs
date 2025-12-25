//! Tab widget implementation

/// Tab widget
pub struct TabWidget {
    /// Tab titles
    pub tabs: Vec<String>,
    /// Active tab index
    pub active: usize,
    /// Tab content
    pub content: Vec<String>,
    /// Scroll offset for tab bar
    pub scroll: usize,
}

impl TabWidget {
    /// Create a new tab widget
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active: 0,
            content: Vec::new(),
            scroll: 0,
        }
    }

    /// Add a tab
    pub fn add_tab(&mut self, title: impl Into<String>) {
        self.tabs.push(title.into());
        self.content.push(String::new());
    }

    /// Add a tab with content
    pub fn add_tab_with_content(&mut self, title: impl Into<String>, content: impl Into<String>) {
        self.tabs.push(title.into());
        self.content.push(content.into());
    }

    /// Select next tab
    pub fn select_next(&mut self) {
        if self.active < self.tabs.len().saturating_sub(1) {
            self.active += 1;
            self.ensure_visible(10);
        }
    }

    /// Select previous tab
    pub fn select_prev(&mut self) {
        if self.active > 0 {
            self.active -= 1;
            self.ensure_visible(10);
        }
    }

    /// Select tab by index
    pub fn select_tab(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.active = index;
            self.ensure_visible(10);
        }
    }

    /// Ensure active tab is visible
    fn ensure_visible(&mut self, visible_width: usize) {
        if self.active < self.scroll {
            self.scroll = self.active;
        } else if self.active >= self.scroll + visible_width {
            self.scroll = self.active.saturating_sub(visible_width - 1);
        }
    }

    /// Get active tab title
    pub fn active_tab(&self) -> Option<&String> {
        self.tabs.get(self.active)
    }

    /// Get active tab content
    pub fn active_content(&self) -> Option<&String> {
        self.content.get(self.active)
    }

    /// Set content for active tab
    pub fn set_active_content(&mut self, content: impl Into<String>) {
        if let Some(c) = self.content.get_mut(self.active) {
            *c = content.into();
        }
    }

    /// Get visible tabs
    pub fn visible_tabs(&self, width: usize) -> Vec<(usize, &String)> {
        self.tabs
            .iter()
            .enumerate()
            .skip(self.scroll)
            .take(width)
            .collect()
    }

    /// Close tab
    pub fn close_tab(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.tabs.remove(index);
            self.content.remove(index);

            if self.active >= self.tabs.len() && self.active > 0 {
                self.active -= 1;
            }
        }
    }

    /// Close active tab
    pub fn close_active_tab(&mut self) {
        self.close_tab(self.active);
    }

    /// Get tab count
    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    /// Check if tabs are empty
    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }

    /// Clear all tabs
    pub fn clear(&mut self) {
        self.tabs.clear();
        self.content.clear();
        self.active = 0;
        self.scroll = 0;
    }
}

impl Default for TabWidget {
    fn default() -> Self {
        Self::new()
    }
}
