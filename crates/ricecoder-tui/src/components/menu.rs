//! Menu widgets and mode selection

use crate::model::AppMode;

/// Menu item
#[derive(Debug, Clone)]
pub struct MenuItem {
    /// Item label
    pub label: String,
    /// Item description
    pub description: Option<String>,
}

impl MenuItem {
    /// Create a new menu item
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            description: None,
        }
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

/// Menu widget
pub struct MenuWidget {
    /// Menu items
    pub items: Vec<MenuItem>,
    /// Selected item index
    pub selected: usize,
    /// Whether menu is open
    pub open: bool,
    /// Menu title
    pub title: Option<String>,
    /// Scroll offset for large menus
    pub scroll: usize,
}

impl MenuWidget {
    /// Create a new menu widget
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected: 0,
            open: false,
            title: None,
            scroll: 0,
        }
    }

    /// Create a menu with a title
    pub fn with_title(title: impl Into<String>) -> Self {
        Self {
            items: Vec::new(),
            selected: 0,
            open: false,
            title: Some(title.into()),
            scroll: 0,
        }
    }

    /// Add a menu item
    pub fn add_item(&mut self, item: MenuItem) {
        self.items.push(item);
    }

    /// Add multiple items
    pub fn add_items(&mut self, items: Vec<MenuItem>) {
        self.items.extend(items);
    }

    /// Select next item
    pub fn select_next(&mut self) {
        if self.selected < self.items.len().saturating_sub(1) {
            self.selected += 1;
            self.ensure_visible(10); // Assume 10 visible items
        }
    }

    /// Select previous item
    pub fn select_prev(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.ensure_visible(10);
        }
    }

    /// Jump to first item
    pub fn select_first(&mut self) {
        self.selected = 0;
        self.scroll = 0;
    }

    /// Jump to last item
    pub fn select_last(&mut self) {
        self.selected = self.items.len().saturating_sub(1);
        self.ensure_visible(10);
    }

    /// Ensure selected item is visible
    fn ensure_visible(&mut self, visible_height: usize) {
        if self.selected < self.scroll {
            self.scroll = self.selected;
        } else if self.selected >= self.scroll + visible_height {
            self.scroll = self.selected.saturating_sub(visible_height - 1);
        }
    }

    /// Get selected item
    pub fn selected_item(&self) -> Option<&MenuItem> {
        self.items.get(self.selected)
    }

    /// Get selected item index
    pub fn selected_index(&self) -> usize {
        self.selected
    }

    /// Open the menu
    pub fn open(&mut self) {
        self.open = true;
    }

    /// Close the menu
    pub fn close(&mut self) {
        self.open = false;
    }

    /// Toggle menu open state
    pub fn toggle(&mut self) {
        self.open = !self.open;
    }

    /// Get visible items based on scroll
    pub fn visible_items(&self, height: usize) -> Vec<(usize, &MenuItem)> {
        self.items
            .iter()
            .enumerate()
            .skip(self.scroll)
            .take(height)
            .collect()
    }

    /// Clear all items
    pub fn clear(&mut self) {
        self.items.clear();
        self.selected = 0;
        self.scroll = 0;
    }

    /// Get total item count
    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    /// Check if menu is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl Default for MenuWidget {
    fn default() -> Self {
        Self::new()
    }
}

/// Mode selection menu for switching between modes
#[derive(Debug, Clone)]
pub struct ModeSelectionMenu {
    /// Available modes
    pub modes: Vec<AppMode>,
    /// Currently selected mode index
    pub selected: usize,
    /// Whether the menu is open
    pub open: bool,
    /// Whether to show confirmation dialog
    pub show_confirmation: bool,
    /// Previous mode (for cancellation)
    pub previous_mode: AppMode,
}

impl ModeSelectionMenu {
    /// Create a new mode selection menu
    pub fn new() -> Self {
        Self {
            modes: vec![
                AppMode::Chat,
                AppMode::Command,
                AppMode::Diff,
                AppMode::Mcp,
                AppMode::Provider,
                AppMode::Help,
            ],
            selected: 0,
            open: false,
            show_confirmation: false,
            previous_mode: AppMode::Chat,
        }
    }

    /// Open the mode selection menu
    pub fn open(&mut self, current_mode: AppMode) {
        self.open = true;
        self.previous_mode = current_mode;
        // Find and select the current mode
        if let Some(pos) = self.modes.iter().position(|&m| m == current_mode) {
            self.selected = pos;
        }
    }

    /// Close the mode selection menu
    pub fn close(&mut self) {
        self.open = false;
        self.show_confirmation = false;
    }

    /// Get the currently selected mode
    pub fn selected_mode(&self) -> AppMode {
        self.modes
            .get(self.selected)
            .copied()
            .unwrap_or(AppMode::Chat)
    }

    /// Move selection to next mode
    pub fn select_next(&mut self) {
        if self.selected < self.modes.len().saturating_sub(1) {
            self.selected += 1;
        } else {
            self.selected = 0;
        }
    }

    /// Move selection to previous mode
    pub fn select_prev(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        } else {
            self.selected = self.modes.len().saturating_sub(1);
        }
    }

    /// Confirm mode switch
    pub fn confirm_switch(&mut self) -> AppMode {
        let mode = self.selected_mode();
        self.close();
        mode
    }

    /// Cancel mode switch
    pub fn cancel_switch(&mut self) {
        self.close();
    }

    /// Get mode descriptions for display
    pub fn get_mode_descriptions(&self) -> Vec<(&AppMode, &'static str)> {
        self.modes
            .iter()
            .map(|mode| {
                let desc = match mode {
                    AppMode::Chat => "Chat with the AI assistant",
                    AppMode::Command => "Execute commands and generate code",
                    AppMode::Diff => "Review and apply code changes",
                    AppMode::Mcp => "Manage MCP servers and tools",
                    AppMode::Provider => "Configure AI providers",
                    AppMode::Session => "Manage and share sessions",
                    AppMode::Help => "Get help and documentation",
                };
                (mode, desc)
            })
            .collect()
    }

    /// Get keyboard shortcuts for mode switching
    pub fn get_shortcuts(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("Ctrl+1", "Chat Mode"),
            ("Ctrl+2", "Command Mode"),
            ("Ctrl+3", "Diff Mode"),
            ("Ctrl+4", "Help Mode"),
        ]
    }
}

impl Default for ModeSelectionMenu {
    fn default() -> Self {
        Self::new()
    }
}
