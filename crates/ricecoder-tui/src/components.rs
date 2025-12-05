//! Interactive UI components

use crate::app::AppMode;

/// Mode indicator component
#[derive(Debug, Clone)]
pub struct ModeIndicator {
    /// Current mode
    pub mode: AppMode,
    /// Show keyboard shortcut
    pub show_shortcut: bool,
    /// Show mode capabilities
    pub show_capabilities: bool,
}

impl ModeIndicator {
    /// Create a new mode indicator
    pub fn new(mode: AppMode) -> Self {
        Self {
            mode,
            show_shortcut: true,
            show_capabilities: false,
        }
    }

    /// Get the display text for the mode
    pub fn display_text(&self) -> String {
        if self.show_shortcut {
            format!("[{}] {}", self.mode.shortcut(), self.mode.display_name())
        } else {
            format!("[{}]", self.mode.display_name())
        }
    }

    /// Get the short display text
    pub fn short_text(&self) -> &'static str {
        self.mode.display_name()
    }

    /// Get the capabilities for the current mode
    pub fn get_capabilities(&self) -> Vec<&'static str> {
        match self.mode {
            AppMode::Chat => vec!["QuestionAnswering", "FreeformChat"],
            AppMode::Command => vec!["CodeGeneration", "FileOperations", "CommandExecution"],
            AppMode::Diff => vec!["CodeModification", "FileOperations"],
            AppMode::Help => vec!["QuestionAnswering"],
        }
    }

    /// Get capabilities display text
    pub fn capabilities_text(&self) -> String {
        let caps = self.get_capabilities();
        format!("Capabilities: {}", caps.join(", "))
    }

    /// Update the mode
    pub fn set_mode(&mut self, mode: AppMode) {
        self.mode = mode;
    }

    /// Toggle shortcut display
    pub fn toggle_shortcut_display(&mut self) {
        self.show_shortcut = !self.show_shortcut;
    }

    /// Toggle capabilities display
    pub fn toggle_capabilities_display(&mut self) {
        self.show_capabilities = !self.show_capabilities;
    }

    /// Enable capabilities display
    pub fn show_capabilities_enabled(&mut self) {
        self.show_capabilities = true;
    }

    /// Disable capabilities display
    pub fn hide_capabilities_enabled(&mut self) {
        self.show_capabilities = false;
    }
}

impl Default for ModeIndicator {
    fn default() -> Self {
        Self::new(AppMode::Chat)
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
            modes: vec![AppMode::Chat, AppMode::Command, AppMode::Diff, AppMode::Help],
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
        self.modes.get(self.selected).copied().unwrap_or(AppMode::Chat)
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
            let indices: Vec<usize> = self.filtered_items().into_iter().map(|(idx, _)| idx).collect();
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

/// Dialog type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogType {
    /// Input dialog
    Input,
    /// Confirmation dialog
    Confirm,
    /// Message dialog
    Message,
}

/// Dialog result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogResult {
    /// Dialog was confirmed
    Confirmed,
    /// Dialog was cancelled
    Cancelled,
    /// Dialog is still open
    Pending,
}

/// Dialog widget
pub struct DialogWidget {
    /// Dialog type
    pub dialog_type: DialogType,
    /// Dialog title
    pub title: String,
    /// Dialog message
    pub message: String,
    /// Input value (for input dialogs)
    pub input: String,
    /// Cursor position
    pub cursor: usize,
    /// Dialog result
    pub result: DialogResult,
    /// Validation function (for input dialogs)
    pub validator: Option<fn(&str) -> bool>,
    /// Error message (if validation fails)
    pub error_message: Option<String>,
    /// Confirmation state (for confirm dialogs)
    pub confirmed: Option<bool>,
}

impl DialogWidget {
    /// Create a new dialog widget
    pub fn new(dialog_type: DialogType, title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            dialog_type,
            title: title.into(),
            message: message.into(),
            input: String::new(),
            cursor: 0,
            result: DialogResult::Pending,
            validator: None,
            error_message: None,
            confirmed: None,
        }
    }

    /// Set a validator function
    pub fn with_validator(mut self, validator: fn(&str) -> bool) -> Self {
        self.validator = Some(validator);
        self
    }

    /// Insert character
    pub fn insert_char(&mut self, ch: char) {
        if ch.is_ascii_graphic() || ch == ' ' {
            self.input.insert(self.cursor, ch);
            self.cursor += 1;
            self.error_message = None;
        }
    }

    /// Backspace
    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            self.input.remove(self.cursor - 1);
            self.cursor -= 1;
            self.error_message = None;
        }
    }

    /// Delete character at cursor
    pub fn delete(&mut self) {
        if self.cursor < self.input.len() {
            self.input.remove(self.cursor);
        }
    }

    /// Move cursor left
    pub fn cursor_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    /// Move cursor right
    pub fn cursor_right(&mut self) {
        if self.cursor < self.input.len() {
            self.cursor += 1;
        }
    }

    /// Move cursor to start
    pub fn cursor_start(&mut self) {
        self.cursor = 0;
    }

    /// Move cursor to end
    pub fn cursor_end(&mut self) {
        self.cursor = self.input.len();
    }

    /// Get input value
    pub fn get_input(&self) -> String {
        self.input.clone()
    }

    /// Validate input
    pub fn validate(&mut self) -> bool {
        if let Some(validator) = self.validator {
            if validator(&self.input) {
                self.error_message = None;
                true
            } else {
                self.error_message = Some("Invalid input".to_string());
                false
            }
        } else {
            true
        }
    }

    /// Confirm dialog
    pub fn confirm(&mut self) {
        match self.dialog_type {
            DialogType::Input => {
                if self.validate() {
                    self.result = DialogResult::Confirmed;
                }
            }
            DialogType::Confirm => {
                self.confirmed = Some(true);
                self.result = DialogResult::Confirmed;
            }
            DialogType::Message => {
                self.result = DialogResult::Confirmed;
            }
        }
    }

    /// Cancel dialog
    pub fn cancel(&mut self) {
        if self.dialog_type == DialogType::Confirm {
            self.confirmed = Some(false);
        }
        self.result = DialogResult::Cancelled;
    }

    /// Check if dialog is confirmed
    pub fn is_confirmed(&self) -> bool {
        self.result == DialogResult::Confirmed
    }

    /// Check if dialog is cancelled
    pub fn is_cancelled(&self) -> bool {
        self.result == DialogResult::Cancelled
    }

    /// Check if dialog is pending
    pub fn is_pending(&self) -> bool {
        self.result == DialogResult::Pending
    }

    /// Clear input
    pub fn clear_input(&mut self) {
        self.input.clear();
        self.cursor = 0;
        self.error_message = None;
    }
}

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

/// Vim keybinding mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VimMode {
    /// Normal mode (navigation)
    Normal,
    /// Insert mode (text input)
    Insert,
    /// Visual mode (selection)
    Visual,
    /// Command mode (commands)
    Command,
}

/// Vim keybindings configuration
pub struct VimKeybindings {
    /// Whether vim mode is enabled
    pub enabled: bool,
    /// Current vim mode
    pub mode: VimMode,
    /// Command buffer for command mode
    pub command_buffer: String,
}

impl VimKeybindings {
    /// Create a new vim keybindings configuration
    pub fn new() -> Self {
        Self {
            enabled: false,
            mode: VimMode::Normal,
            command_buffer: String::new(),
        }
    }

    /// Enable vim mode
    pub fn enable(&mut self) {
        self.enabled = true;
        self.mode = VimMode::Normal;
    }

    /// Disable vim mode
    pub fn disable(&mut self) {
        self.enabled = false;
        self.mode = VimMode::Normal;
        self.command_buffer.clear();
    }

    /// Toggle vim mode
    pub fn toggle(&mut self) {
        if self.enabled {
            self.disable();
        } else {
            self.enable();
        }
    }

    /// Enter insert mode
    pub fn enter_insert(&mut self) {
        if self.enabled {
            self.mode = VimMode::Insert;
        }
    }

    /// Enter normal mode
    pub fn enter_normal(&mut self) {
        if self.enabled {
            self.mode = VimMode::Normal;
            self.command_buffer.clear();
        }
    }

    /// Enter visual mode
    pub fn enter_visual(&mut self) {
        if self.enabled {
            self.mode = VimMode::Visual;
        }
    }

    /// Enter command mode
    pub fn enter_command(&mut self) {
        if self.enabled {
            self.mode = VimMode::Command;
            self.command_buffer.clear();
        }
    }

    /// Add character to command buffer
    pub fn add_to_command(&mut self, ch: char) {
        self.command_buffer.push(ch);
    }

    /// Clear command buffer
    pub fn clear_command(&mut self) {
        self.command_buffer.clear();
    }

    /// Get command buffer
    pub fn get_command(&self) -> &str {
        &self.command_buffer
    }

    /// Check if in normal mode
    pub fn is_normal(&self) -> bool {
        self.enabled && self.mode == VimMode::Normal
    }

    /// Check if in insert mode
    pub fn is_insert(&self) -> bool {
        self.enabled && self.mode == VimMode::Insert
    }

    /// Check if in visual mode
    pub fn is_visual(&self) -> bool {
        self.enabled && self.mode == VimMode::Visual
    }

    /// Check if in command mode
    pub fn is_command(&self) -> bool {
        self.enabled && self.mode == VimMode::Command
    }
}

impl Default for VimKeybindings {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_widget() {
        let mut menu = MenuWidget::new();
        menu.add_item(MenuItem::new("Option 1"));
        menu.add_item(MenuItem::new("Option 2"));

        assert_eq!(menu.selected, 0);
        menu.select_next();
        assert_eq!(menu.selected, 1);
        menu.select_prev();
        assert_eq!(menu.selected, 0);
    }

    #[test]
    fn test_menu_widget_with_title() {
        let menu = MenuWidget::with_title("Main Menu");
        assert_eq!(menu.title, Some("Main Menu".to_string()));
        assert!(!menu.open);
    }

    #[test]
    fn test_menu_widget_open_close() {
        let mut menu = MenuWidget::new();
        assert!(!menu.open);
        
        menu.open();
        assert!(menu.open);
        
        menu.close();
        assert!(!menu.open);
        
        menu.toggle();
        assert!(menu.open);
    }

    #[test]
    fn test_menu_widget_first_last() {
        let mut menu = MenuWidget::new();
        menu.add_item(MenuItem::new("Item 1"));
        menu.add_item(MenuItem::new("Item 2"));
        menu.add_item(MenuItem::new("Item 3"));
        menu.add_item(MenuItem::new("Item 4"));

        menu.select_last();
        assert_eq!(menu.selected, 3);

        menu.select_first();
        assert_eq!(menu.selected, 0);
    }

    #[test]
    fn test_menu_widget_visible_items() {
        let mut menu = MenuWidget::new();
        for i in 0..10 {
            menu.add_item(MenuItem::new(format!("Item {}", i)));
        }

        let visible = menu.visible_items(5);
        assert_eq!(visible.len(), 5);

        menu.scroll = 3;
        let visible = menu.visible_items(5);
        assert_eq!(visible.len(), 5);
        assert_eq!(visible[0].0, 3);
    }

    #[test]
    fn test_menu_widget_clear() {
        let mut menu = MenuWidget::new();
        menu.add_item(MenuItem::new("Item 1"));
        menu.add_item(MenuItem::new("Item 2"));
        menu.selected = 1;

        menu.clear();
        assert!(menu.items.is_empty());
        assert_eq!(menu.selected, 0);
        assert_eq!(menu.scroll, 0);
    }

    #[test]
    fn test_menu_widget_add_items() {
        let mut menu = MenuWidget::new();
        let items = vec![
            MenuItem::new("Item 1"),
            MenuItem::new("Item 2"),
            MenuItem::new("Item 3"),
        ];
        menu.add_items(items);

        assert_eq!(menu.item_count(), 3);
    }

    #[test]
    fn test_menu_widget_is_empty() {
        let menu = MenuWidget::new();
        assert!(menu.is_empty());

        let mut menu = MenuWidget::new();
        menu.add_item(MenuItem::new("Item"));
        assert!(!menu.is_empty());
    }

    #[test]
    fn test_list_widget() {
        let mut list = ListWidget::new();
        list.add_item("apple");
        list.add_item("banana");
        list.add_item("cherry");

        assert_eq!(list.filtered_items().len(), 3);

        list.set_filter("app");
        assert_eq!(list.filtered_items().len(), 1);
    }

    #[test]
    fn test_list_widget_selection() {
        let mut list = ListWidget::new();
        list.add_item("item1");
        list.add_item("item2");
        list.add_item("item3");

        list.select_next();
        assert_eq!(list.selected, Some(0));

        list.select_next();
        assert_eq!(list.selected, Some(1));
    }

    #[test]
    fn test_list_widget_multi_select() {
        let mut list = ListWidget::new().with_multi_select();
        list.add_item("item1");
        list.add_item("item2");
        list.add_item("item3");

        assert!(list.multi_select);

        list.select_next();
        list.toggle_selection();
        assert!(list.selected_items.contains(&0));

        list.select_next();
        list.toggle_selection();
        assert!(list.selected_items.contains(&1));

        let selected = list.get_selected_items();
        assert_eq!(selected.len(), 2);
    }

    #[test]
    fn test_list_widget_select_all() {
        let mut list = ListWidget::new().with_multi_select();
        list.add_item("item1");
        list.add_item("item2");
        list.add_item("item3");

        list.select_all();
        assert_eq!(list.selected_items.len(), 3);
    }

    #[test]
    fn test_list_widget_deselect_all() {
        let mut list = ListWidget::new().with_multi_select();
        list.add_item("item1");
        list.add_item("item2");
        list.add_item("item3");

        list.select_all();
        assert_eq!(list.selected_items.len(), 3);

        list.deselect_all();
        assert!(list.selected_items.is_empty());
    }

    #[test]
    fn test_list_widget_filter() {
        let mut list = ListWidget::new();
        list.add_item("apple");
        list.add_item("apricot");
        list.add_item("banana");
        list.add_item("blueberry");

        list.set_filter("ap");
        assert_eq!(list.filtered_items().len(), 2);

        list.set_filter("b");
        assert_eq!(list.filtered_items().len(), 2);

        list.clear_filter();
        assert_eq!(list.filtered_items().len(), 4);
    }

    #[test]
    fn test_list_widget_visible_items() {
        let mut list = ListWidget::new();
        for i in 0..10 {
            list.add_item(format!("item{}", i));
        }

        let visible = list.visible_items(5);
        assert_eq!(visible.len(), 5);

        list.scroll = 3;
        let visible = list.visible_items(5);
        assert_eq!(visible.len(), 5);
    }

    #[test]
    fn test_list_widget_add_items() {
        let mut list = ListWidget::new();
        let items = vec!["item1".to_string(), "item2".to_string(), "item3".to_string()];
        list.add_items(items);

        assert_eq!(list.item_count(), 3);
    }

    #[test]
    fn test_list_widget_clear() {
        let mut list = ListWidget::new();
        list.add_item("item1");
        list.add_item("item2");
        list.selected = Some(0);

        list.clear();
        assert!(list.items.is_empty());
        assert!(list.selected.is_none());
    }

    #[test]
    fn test_list_widget_is_empty() {
        let list = ListWidget::new();
        assert!(list.is_empty());

        let mut list = ListWidget::new();
        list.add_item("item");
        assert!(!list.is_empty());
    }

    #[test]
    fn test_dialog_widget() {
        let dialog = DialogWidget::new(DialogType::Input, "Title", "Message");
        assert_eq!(dialog.dialog_type, DialogType::Input);
        assert_eq!(dialog.title, "Title");
        assert!(dialog.is_pending());
    }

    #[test]
    fn test_dialog_widget_input() {
        let mut dialog = DialogWidget::new(DialogType::Input, "Title", "Message");
        dialog.insert_char('h');
        dialog.insert_char('i');

        assert_eq!(dialog.get_input(), "hi");
    }

    #[test]
    fn test_dialog_widget_cursor_movement() {
        let mut dialog = DialogWidget::new(DialogType::Input, "Title", "Message");
        dialog.insert_char('h');
        dialog.insert_char('e');
        dialog.insert_char('l');
        dialog.insert_char('l');
        dialog.insert_char('o');

        assert_eq!(dialog.cursor, 5);

        dialog.cursor_left();
        assert_eq!(dialog.cursor, 4);

        dialog.cursor_right();
        assert_eq!(dialog.cursor, 5);

        dialog.cursor_start();
        assert_eq!(dialog.cursor, 0);

        dialog.cursor_end();
        assert_eq!(dialog.cursor, 5);
    }

    #[test]
    fn test_dialog_widget_delete() {
        let mut dialog = DialogWidget::new(DialogType::Input, "Title", "Message");
        dialog.insert_char('h');
        dialog.insert_char('e');
        dialog.insert_char('l');
        dialog.insert_char('l');
        dialog.insert_char('o');

        dialog.cursor_start();
        dialog.delete();
        assert_eq!(dialog.get_input(), "ello");
    }

    #[test]
    fn test_dialog_widget_backspace() {
        let mut dialog = DialogWidget::new(DialogType::Input, "Title", "Message");
        dialog.insert_char('h');
        dialog.insert_char('e');
        dialog.insert_char('l');
        dialog.insert_char('l');
        dialog.insert_char('o');

        dialog.backspace();
        assert_eq!(dialog.get_input(), "hell");
        assert_eq!(dialog.cursor, 4);
    }

    #[test]
    fn test_dialog_widget_confirm() {
        let mut dialog = DialogWidget::new(DialogType::Input, "Title", "Message");
        dialog.insert_char('t');
        dialog.insert_char('e');
        dialog.insert_char('s');
        dialog.insert_char('t');

        dialog.confirm();
        assert!(dialog.is_confirmed());
        assert!(!dialog.is_pending());
    }

    #[test]
    fn test_dialog_widget_cancel() {
        let mut dialog = DialogWidget::new(DialogType::Input, "Title", "Message");
        dialog.cancel();
        assert!(dialog.is_cancelled());
        assert!(!dialog.is_pending());
    }

    #[test]
    fn test_dialog_widget_validation() {
        let mut dialog = DialogWidget::new(DialogType::Input, "Title", "Message")
            .with_validator(|input| !input.is_empty() && input.len() >= 3);

        dialog.insert_char('a');
        dialog.insert_char('b');
        assert!(!dialog.validate());

        dialog.insert_char('c');
        assert!(dialog.validate());
    }

    #[test]
    fn test_dialog_widget_confirm_dialog() {
        let mut dialog = DialogWidget::new(DialogType::Confirm, "Confirm", "Are you sure?");
        assert!(dialog.confirmed.is_none());

        dialog.confirm();
        assert_eq!(dialog.confirmed, Some(true));
        assert!(dialog.is_confirmed());

        let mut dialog = DialogWidget::new(DialogType::Confirm, "Confirm", "Are you sure?");
        dialog.cancel();
        assert_eq!(dialog.confirmed, Some(false));
        assert!(dialog.is_cancelled());
    }

    #[test]
    fn test_dialog_widget_clear_input() {
        let mut dialog = DialogWidget::new(DialogType::Input, "Title", "Message");
        dialog.insert_char('t');
        dialog.insert_char('e');
        dialog.insert_char('s');
        dialog.insert_char('t');

        dialog.clear_input();
        assert!(dialog.get_input().is_empty());
        assert_eq!(dialog.cursor, 0);
    }

    #[test]
    fn test_split_view_widget() {
        let mut split = SplitViewWidget::new();
        split.set_left("left content");
        split.set_right("right content");

        assert_eq!(split.left_content, "left content");
        assert_eq!(split.right_content, "right content");
        assert_eq!(split.split_ratio, 50);
    }

    #[test]
    fn test_split_view_adjust() {
        let mut split = SplitViewWidget::new();
        split.adjust_split(10);
        assert_eq!(split.split_ratio, 60);

        split.adjust_split(-20);
        assert_eq!(split.split_ratio, 40);
    }

    #[test]
    fn test_split_view_direction() {
        let split = SplitViewWidget::new();
        assert_eq!(split.direction, SplitDirection::Vertical);

        let split = SplitViewWidget::horizontal();
        assert_eq!(split.direction, SplitDirection::Horizontal);
    }

    #[test]
    fn test_split_view_panel_switching() {
        let mut split = SplitViewWidget::new();
        split.set_left("left");
        split.set_right("right");

        assert_eq!(split.active_panel, 0);
        assert_eq!(split.active_content(), "left");

        split.switch_panel();
        assert_eq!(split.active_panel, 1);
        assert_eq!(split.active_content(), "right");

        split.switch_panel();
        assert_eq!(split.active_panel, 0);
    }

    #[test]
    fn test_split_view_scrolling() {
        let mut split = SplitViewWidget::new();
        split.set_left("left content");
        split.set_right("right content");

        assert_eq!(split.left_scroll, 0);
        split.scroll_down();
        assert_eq!(split.left_scroll, 1);

        split.scroll_up();
        assert_eq!(split.left_scroll, 0);

        split.switch_panel();
        split.scroll_down();
        assert_eq!(split.right_scroll, 1);
    }

    #[test]
    fn test_tab_widget() {
        let mut tabs = TabWidget::new();
        tabs.add_tab("Tab 1");
        tabs.add_tab("Tab 2");
        tabs.add_tab("Tab 3");

        assert_eq!(tabs.active, 0);
        tabs.select_next();
        assert_eq!(tabs.active, 1);
        tabs.select_prev();
        assert_eq!(tabs.active, 0);
    }

    #[test]
    fn test_tab_widget_with_content() {
        let mut tabs = TabWidget::new();
        tabs.add_tab_with_content("Tab 1", "Content 1");
        tabs.add_tab_with_content("Tab 2", "Content 2");

        assert_eq!(tabs.active_content(), Some(&"Content 1".to_string()));

        tabs.select_next();
        assert_eq!(tabs.active_content(), Some(&"Content 2".to_string()));
    }

    #[test]
    fn test_tab_widget_select_by_index() {
        let mut tabs = TabWidget::new();
        tabs.add_tab("Tab 1");
        tabs.add_tab("Tab 2");
        tabs.add_tab("Tab 3");

        tabs.select_tab(2);
        assert_eq!(tabs.active, 2);

        tabs.select_tab(0);
        assert_eq!(tabs.active, 0);
    }

    #[test]
    fn test_tab_widget_close_tab() {
        let mut tabs = TabWidget::new();
        tabs.add_tab("Tab 1");
        tabs.add_tab("Tab 2");
        tabs.add_tab("Tab 3");

        assert_eq!(tabs.tab_count(), 3);

        tabs.close_tab(1);
        assert_eq!(tabs.tab_count(), 2);
        assert_eq!(tabs.active_tab(), Some(&"Tab 1".to_string()));
    }

    #[test]
    fn test_tab_widget_close_active_tab() {
        let mut tabs = TabWidget::new();
        tabs.add_tab("Tab 1");
        tabs.add_tab("Tab 2");
        tabs.add_tab("Tab 3");

        // Select last tab
        tabs.select_tab(2);
        assert_eq!(tabs.active, 2);
        tabs.close_active_tab();
        assert_eq!(tabs.tab_count(), 2);
        // After closing tab at index 2 (last), active should be adjusted to 1
        assert_eq!(tabs.active, 1);
    }

    #[test]
    fn test_tab_widget_set_content() {
        let mut tabs = TabWidget::new();
        tabs.add_tab("Tab 1");
        tabs.set_active_content("New content");

        assert_eq!(tabs.active_content(), Some(&"New content".to_string()));
    }

    #[test]
    fn test_tab_widget_visible_tabs() {
        let mut tabs = TabWidget::new();
        for i in 0..10 {
            tabs.add_tab(format!("Tab {}", i));
        }

        let visible = tabs.visible_tabs(5);
        assert_eq!(visible.len(), 5);

        tabs.scroll = 3;
        let visible = tabs.visible_tabs(5);
        assert_eq!(visible.len(), 5);
    }

    #[test]
    fn test_tab_widget_clear() {
        let mut tabs = TabWidget::new();
        tabs.add_tab("Tab 1");
        tabs.add_tab("Tab 2");

        tabs.clear();
        assert!(tabs.is_empty());
        assert_eq!(tabs.active, 0);
    }

    #[test]
    fn test_mode_indicator_creation() {
        let indicator = ModeIndicator::new(AppMode::Chat);
        assert_eq!(indicator.mode, AppMode::Chat);
        assert!(indicator.show_shortcut);
    }

    #[test]
    fn test_mode_indicator_display_text() {
        let indicator = ModeIndicator::new(AppMode::Chat);
        let text = indicator.display_text();
        assert!(text.contains("Chat"));
        assert!(text.contains("Ctrl+1"));
    }

    #[test]
    fn test_mode_indicator_short_text() {
        let indicator = ModeIndicator::new(AppMode::Command);
        assert_eq!(indicator.short_text(), "Command");
    }

    #[test]
    fn test_mode_indicator_set_mode() {
        let mut indicator = ModeIndicator::new(AppMode::Chat);
        indicator.set_mode(AppMode::Diff);
        assert_eq!(indicator.mode, AppMode::Diff);
    }

    #[test]
    fn test_mode_indicator_toggle_shortcut() {
        let mut indicator = ModeIndicator::new(AppMode::Chat);
        assert!(indicator.show_shortcut);
        indicator.toggle_shortcut_display();
        assert!(!indicator.show_shortcut);
        indicator.toggle_shortcut_display();
        assert!(indicator.show_shortcut);
    }

    #[test]
    fn test_mode_indicator_get_capabilities() {
        let indicator = ModeIndicator::new(AppMode::Chat);
        let caps = indicator.get_capabilities();
        assert!(caps.contains(&"QuestionAnswering"));
        assert!(caps.contains(&"FreeformChat"));
    }

    #[test]
    fn test_mode_indicator_capabilities_text() {
        let indicator = ModeIndicator::new(AppMode::Command);
        let text = indicator.capabilities_text();
        assert!(text.contains("Capabilities:"));
        assert!(text.contains("CodeGeneration"));
    }

    #[test]
    fn test_mode_indicator_toggle_capabilities() {
        let mut indicator = ModeIndicator::new(AppMode::Chat);
        assert!(!indicator.show_capabilities);
        indicator.toggle_capabilities_display();
        assert!(indicator.show_capabilities);
        indicator.toggle_capabilities_display();
        assert!(!indicator.show_capabilities);
    }

    #[test]
    fn test_mode_indicator_show_hide_capabilities() {
        let mut indicator = ModeIndicator::new(AppMode::Chat);
        assert!(!indicator.show_capabilities);
        
        indicator.show_capabilities_enabled();
        assert!(indicator.show_capabilities);
        
        indicator.hide_capabilities_enabled();
        assert!(!indicator.show_capabilities);
    }

    #[test]
    fn test_mode_selection_menu_creation() {
        let menu = ModeSelectionMenu::new();
        assert!(!menu.open);
        assert_eq!(menu.modes.len(), 4);
        assert_eq!(menu.selected_mode(), AppMode::Chat);
    }

    #[test]
    fn test_mode_selection_menu_open_close() {
        let mut menu = ModeSelectionMenu::new();
        assert!(!menu.open);
        
        menu.open(AppMode::Chat);
        assert!(menu.open);
        
        menu.close();
        assert!(!menu.open);
    }

    #[test]
    fn test_mode_selection_menu_navigation() {
        let mut menu = ModeSelectionMenu::new();
        menu.open(AppMode::Chat);
        
        assert_eq!(menu.selected_mode(), AppMode::Chat);
        menu.select_next();
        assert_eq!(menu.selected_mode(), AppMode::Command);
        menu.select_next();
        assert_eq!(menu.selected_mode(), AppMode::Diff);
        menu.select_prev();
        assert_eq!(menu.selected_mode(), AppMode::Command);
    }

    #[test]
    fn test_mode_selection_menu_wrap_around() {
        let mut menu = ModeSelectionMenu::new();
        menu.open(AppMode::Help);
        
        assert_eq!(menu.selected_mode(), AppMode::Help);
        menu.select_next();
        assert_eq!(menu.selected_mode(), AppMode::Chat);
        
        menu.select_prev();
        assert_eq!(menu.selected_mode(), AppMode::Help);
    }

    #[test]
    fn test_mode_selection_menu_confirm() {
        let mut menu = ModeSelectionMenu::new();
        menu.open(AppMode::Chat);
        menu.select_next();
        
        let selected = menu.confirm_switch();
        assert_eq!(selected, AppMode::Command);
        assert!(!menu.open);
    }

    #[test]
    fn test_mode_selection_menu_descriptions() {
        let menu = ModeSelectionMenu::new();
        let descriptions = menu.get_mode_descriptions();
        assert_eq!(descriptions.len(), 4);
        assert!(descriptions[0].1.contains("Chat"));
    }

    #[test]
    fn test_mode_selection_menu_shortcuts() {
        let menu = ModeSelectionMenu::new();
        let shortcuts = menu.get_shortcuts();
        assert_eq!(shortcuts.len(), 4);
        assert_eq!(shortcuts[0].0, "Ctrl+1");
    }

    #[test]
    fn test_vim_keybindings_creation() {
        let vim = VimKeybindings::new();
        assert!(!vim.enabled);
        assert_eq!(vim.mode, VimMode::Normal);
        assert!(vim.command_buffer.is_empty());
    }

    #[test]
    fn test_vim_keybindings_enable_disable() {
        let mut vim = VimKeybindings::new();
        assert!(!vim.enabled);

        vim.enable();
        assert!(vim.enabled);
        assert_eq!(vim.mode, VimMode::Normal);

        vim.disable();
        assert!(!vim.enabled);
    }

    #[test]
    fn test_vim_keybindings_toggle() {
        let mut vim = VimKeybindings::new();
        assert!(!vim.enabled);

        vim.toggle();
        assert!(vim.enabled);

        vim.toggle();
        assert!(!vim.enabled);
    }

    #[test]
    fn test_vim_keybindings_mode_switching() {
        let mut vim = VimKeybindings::new();
        vim.enable();

        assert_eq!(vim.mode, VimMode::Normal);

        vim.enter_insert();
        assert_eq!(vim.mode, VimMode::Insert);

        vim.enter_visual();
        assert_eq!(vim.mode, VimMode::Visual);

        vim.enter_command();
        assert_eq!(vim.mode, VimMode::Command);

        vim.enter_normal();
        assert_eq!(vim.mode, VimMode::Normal);
    }

    #[test]
    fn test_vim_keybindings_command_buffer() {
        let mut vim = VimKeybindings::new();
        vim.enable();
        vim.enter_command();

        vim.add_to_command('w');
        vim.add_to_command('q');

        assert_eq!(vim.get_command(), "wq");

        vim.clear_command();
        assert!(vim.get_command().is_empty());
    }

    #[test]
    fn test_vim_keybindings_mode_checks() {
        let mut vim = VimKeybindings::new();
        vim.enable();

        assert!(vim.is_normal());
        assert!(!vim.is_insert());
        assert!(!vim.is_visual());
        assert!(!vim.is_command());

        vim.enter_insert();
        assert!(!vim.is_normal());
        assert!(vim.is_insert());

        vim.enter_visual();
        assert!(vim.is_visual());

        vim.enter_command();
        assert!(vim.is_command());
    }

    #[test]
    fn test_vim_keybindings_disabled_mode_checks() {
        let vim = VimKeybindings::new();
        assert!(!vim.is_normal());
        assert!(!vim.is_insert());
        assert!(!vim.is_visual());
        assert!(!vim.is_command());
    }

    #[test]
    fn test_vim_keybindings_enter_normal_clears_command() {
        let mut vim = VimKeybindings::new();
        vim.enable();
        vim.enter_command();
        vim.add_to_command('w');
        vim.add_to_command('q');

        assert!(!vim.get_command().is_empty());

        vim.enter_normal();
        assert!(vim.get_command().is_empty());
    }

    #[test]
    fn test_vim_keybindings_disable_resets_mode() {
        let mut vim = VimKeybindings::new();
        vim.enable();
        vim.enter_command();
        vim.add_to_command('t');
        vim.add_to_command('e');
        vim.add_to_command('s');
        vim.add_to_command('t');

        vim.disable();
        assert_eq!(vim.mode, VimMode::Normal);
        assert!(vim.get_command().is_empty());
    }
}
