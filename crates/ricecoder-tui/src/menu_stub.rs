//! Minimal stub for old menu component types
//! These are used by dialog files but not critical for TUI functionality

/// Menu item stub
#[derive(Debug, Clone)]
pub struct MenuItem {
    pub label: String,
    pub value: String,
    pub description: Option<String>,
}

impl MenuItem {
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            description: None,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// Menu widget stub
#[derive(Debug, Clone)]
pub struct MenuWidget {
    pub title: Option<String>,
    pub items: Vec<MenuItem>,
    pub selected_idx: usize,
    pub is_open: bool,
}

impl MenuWidget {
    pub fn new(items: Vec<MenuItem>) -> Self {
        Self {
            title: None,
            items,
            selected_idx: 0,
            is_open: false,
        }
    }

    pub fn with_title(title: impl Into<String>) -> Self {
        Self {
            title: Some(title.into()),
            items: Vec::new(),
            selected_idx: 0,
            is_open: false,
        }
    }

    pub fn add_item(&mut self, item: MenuItem) {
        self.items.push(item);
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.selected_idx = 0;
    }

    pub fn open(&mut self) {
        self.is_open = true;
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }

    pub fn selected_index(&self) -> usize {
        self.selected_idx
    }

    pub fn select_next(&mut self) {
        if !self.items.is_empty() {
            self.selected_idx = (self.selected_idx + 1) % self.items.len();
        }
    }

    pub fn select_prev(&mut self) {
        if !self.items.is_empty() {
            self.selected_idx = if self.selected_idx == 0 {
                self.items.len() - 1
            } else {
                self.selected_idx - 1
            };
        }
    }

    pub fn selected(&self) -> Option<&MenuItem> {
        self.items.get(self.selected_idx)
    }
}
