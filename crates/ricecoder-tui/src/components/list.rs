//! List widget implementation

/// List widget
pub struct ListWidget {
    /// List items
    pub items: Vec<String>,
    /// Selected item index
    pub selected: Option<usize>,
    /// Filter text
    pub filter: String,
    /// Multi-select enabled
    pub multi_select: bool,
    /// Selected items (for multi-select)
    pub selected_items: std::collections::HashSet<usize>,
    /// Scroll offset
    pub scroll: usize,
}

impl ListWidget {
    /// Create a new list widget
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected: None,
            filter: String::new(),
            multi_select: false,
            selected_items: std::collections::HashSet::new(),
            scroll: 0,
        }
    }

    /// Enable multi-select mode
    pub fn with_multi_select(mut self) -> Self {
        self.multi_select = true;
        self
    }

    /// Add an item
    pub fn add_item(&mut self, item: impl Into<String>) {
        self.items.push(item.into());
    }

    /// Add multiple items
    pub fn add_items(&mut self, items: Vec<String>) {
        self.items.extend(items);
    }

    /// Set filter
    pub fn set_filter(&mut self, filter: impl Into<String>) {
        self.filter = filter.into();
        self.scroll = 0; // Reset scroll when filtering
    }

    /// Clear filter
    pub fn clear_filter(&mut self) {
        self.filter.clear();
        self.scroll = 0;
    }

    /// Get filtered items
    pub fn filtered_items(&self) -> Vec<(usize, &String)> {
        self.items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.to_lowercase().contains(&self.filter.to_lowercase()))
            .collect()
    }

    /// Get visible items based on scroll
    pub fn visible_items(&self, height: usize) -> Vec<(usize, &String)> {
        self.filtered_items()
            .into_iter()
            .skip(self.scroll)
            .take(height)
            .collect()
    }

    /// Select next item
    pub fn select_next(&mut self) {
        let filtered = self.filtered_items();
        if filtered.is_empty() {
            return;
        }

        match self.selected {
            None => {
                self.selected = Some(filtered[0].0);
                self.scroll = 0;
            }
            Some(idx) => {
                if let Some(pos) = filtered.iter().position(|(i, _)| *i == idx) {
                    if pos < filtered.len() - 1 {
                        self.selected = Some(filtered[pos + 1].0);
                    }
                }
            }
        }
    }

    /// Select previous item
    pub fn select_prev(&mut self) {
        let filtered = self.filtered_items();
        if filtered.is_empty() {
            return;
        }

        match self.selected {
            None => {}
            Some(idx) => {
                if let Some(pos) = filtered.iter().position(|(i, _)| *i == idx) {
                    if pos > 0 {
                        self.selected = Some(filtered[pos - 1].0);
                    }
                }
            }
        }
    }

    /// Toggle selection for current item (multi-select)
    pub fn toggle_selection(&mut self) {
        if self.multi_select {
            if let Some(idx) = self.selected {
                if self.selected_items.contains(&idx) {
                    self.selected_items.remove(&idx);
                } else {
                    self.selected_items.insert(idx);
                }
            }
        }
    }

    /// Select all items
    pub fn select_all(&mut self) {
        if self.multi_select {
            let indices: Vec<usize> = self
                .filtered_items()
                .into_iter()
                .map(|(idx, _)| idx)
                .collect();
            for idx in indices {
                self.selected_items.insert(idx);
            }
        }
    }

    /// Deselect all items
    pub fn deselect_all(&mut self) {
        self.selected_items.clear();
    }

    /// Get selected item
    pub fn selected_item(&self) -> Option<&String> {
        self.selected.and_then(|idx| self.items.get(idx))
    }

    /// Get all selected items (multi-select)
    pub fn get_selected_items(&self) -> Vec<&String> {
        self.selected_items
            .iter()
            .filter_map(|idx| self.items.get(*idx))
            .collect()
    }

    /// Clear all items
    pub fn clear(&mut self) {
        self.items.clear();
        self.selected = None;
        self.selected_items.clear();
        self.scroll = 0;
    }

    /// Get total item count
    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    /// Check if list is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl Default for ListWidget {
    fn default() -> Self {
        Self::new()
    }
}
