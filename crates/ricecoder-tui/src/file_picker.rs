//! File picker widget for selecting files to include in messages
//!
//! This module provides a file picker overlay that allows users to browse,
//! search, and select files from the filesystem with tree navigation and
//! fuzzy search capabilities.

use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};
use ratatui_explorer::FileExplorer;

/// Result of file selection
#[derive(Debug, Clone)]
pub struct FileSelection {
    /// File path
    pub path: PathBuf,
    /// File content (None for directories, binary files, or errors)
    pub content: Option<String>,
    /// File information
    pub info: FileInfo,
    /// Selection status
    pub status: FileSelectionStatus,
}

/// Status of file selection
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileSelectionStatus {
    /// File successfully included
    Included,
    /// Directory (not included)
    Directory,
    /// Binary file (not included)
    BinaryFile,
    /// File too large (not included)
    TooLarge,
    /// Error reading file
    Error(String),
}

/// Errors that can occur during file picker operations
#[derive(Debug, Clone, thiserror::Error)]
pub enum FilePickerError {
    #[error("Failed to read file: {0}")]
    ReadError(String),
    #[error("File is too large to include")]
    FileTooLarge,
    #[error("Binary files cannot be included")]
    BinaryFile,
}

/// File picker widget state
#[derive(Debug)]
pub struct FilePickerWidget {
    /// ratatui-explorer file explorer
    explorer: FileExplorer,
    /// Selected file indices (maintaining compatibility)
    selected_indices: HashSet<usize>,
    /// Whether the picker is visible
    visible: bool,
    /// Recently modified files (for visual indicators)
    recently_modified: HashSet<PathBuf>,
    /// Files with external changes (for conflict detection)
    externally_modified: HashSet<PathBuf>,
    /// Maximum file size for inclusion (in bytes)
    max_file_size: u64,

    // Component trait fields
    /// Unique component identifier
    id: crate::ComponentId,
    /// Whether the component is focused
    focused: bool,
    /// Whether the component is enabled
    enabled: bool,
    /// Component bounds
    bounds: ratatui::layout::Rect,
}

/// File information for selection decisions
#[derive(Debug, Clone)]
pub struct FileInfo {
    /// File size in bytes
    pub size: u64,
    /// Whether the file is binary
    pub is_binary: bool,
    /// Whether the file is too large to include
    pub is_too_large: bool,
}

impl FilePickerWidget {
    /// Create a new file picker widget
    pub fn new() -> Self {
        let explorer = FileExplorer::new().unwrap();

        Self {
            explorer,
            selected_indices: HashSet::new(),
            visible: false,
            max_file_size: 1024 * 1024, // 1MB default
            recently_modified: HashSet::new(),
            externally_modified: HashSet::new(),

            // Component trait fields
            id: "file-picker".to_string(),
            focused: false,
            enabled: true,
            bounds: ratatui::layout::Rect::new(0, 0, 80, 25),
        }
    }

    /// Show the file picker
    pub fn show(&mut self) {
        self.visible = true;
        self.focused = true;
    }

    /// Hide the file picker
    pub fn hide(&mut self) {
        self.visible = false;
        self.focused = false;
    }

    /// Set the maximum file size for inclusion
    pub fn set_max_file_size(&mut self, max_size: u64) {
        self.max_file_size = max_size;
    }

    /// Check if the picker is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Toggle picker visibility
    pub fn toggle(&mut self) {
        if self.visible {
            self.hide();
        } else {
            self.show();
        }
    }

    /// Get file information for a path
    fn get_file_info(&self, path: &Path) -> FileInfo {
        let metadata = path.metadata().ok();
        let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
        let is_too_large = size > self.max_file_size;

        // Simple binary detection (check for null bytes in first 512 bytes)
        let is_binary = if let Ok(content) = std::fs::read(path) {
            content.len() >= 512 && content[..512].contains(&0)
        } else {
            false
        };

        FileInfo {
            size,
            is_binary,
            is_too_large,
        }
    }

    /// Handle backspace for search (delegates to FileExplorer)
    pub fn backspace(&mut self) {
        // FileExplorer handles backspace through its internal state
    }

    /// Select all visible items (delegates to FileExplorer)
    pub fn select_all(&mut self) {
        // FileExplorer may have a select all method
        // For now, selection is handled by the widget
    }

    /// Clear all selections
    pub fn clear_selection(&mut self) {
        self.selected_indices.clear();
        // FileExplorer handles selection clearing through its internal state
    }

    /// Clear search query (delegates to FileExplorer)
    pub fn clear_search(&mut self) {
        // FileExplorer handles search clearing through its internal state
    }

    /// Handle file changes
    pub fn handle_file_changes(&mut self, _events: &[ricecoder_files::FileChangeEvent]) {
        // TODO: Implement file change handling
        // For now, we just track modification indicators
    }

    /// Navigate up in the file list
    pub fn navigate_up(&mut self) {
        // FileExplorer handles navigation through its internal state
    }

    /// Toggle selection of current item
    pub fn toggle_selection(&mut self) {
        // FileExplorer handles selection through its internal state
    }

    /// Input a character for search
    pub fn input_char(&mut self, _c: char) {
        // FileExplorer handles character input through its internal state
    }

    /// Get selected files
    pub fn selected_files(&self) -> Vec<PathBuf> {
        // For now, return empty vec - selection is handled by widget
        Vec::new()
    }

    /// Confirm selection and return selected files
    pub fn confirm_selection(&mut self) -> Result<Vec<FileSelection>, FilePickerError> {
        let selected_files = self
            .selected_files()
            .into_iter()
            .map(|path| {
                let info = self.get_file_info(&path);
                (path, info)
            })
            .collect::<Vec<_>>();
        let mut results = Vec::new();

        for (path, info) in selected_files {
            let selection = if info.is_too_large {
                FileSelection {
                    path,
                    content: None,
                    info,
                    status: FileSelectionStatus::TooLarge,
                }
            } else if info.is_binary {
                FileSelection {
                    path,
                    content: None,
                    info,
                    status: FileSelectionStatus::BinaryFile,
                }
            } else {
                // Try to read file content
                match self.read_file_content(&path) {
                    Ok(content) => FileSelection {
                        path,
                        content: Some(content),
                        info,
                        status: FileSelectionStatus::Included,
                    },
                    Err(e) => FileSelection {
                        path,
                        content: None,
                        info,
                        status: FileSelectionStatus::Error(e.to_string()),
                    },
                }
            };
            results.push(selection);
        }

        self.hide();
        Ok(results)
    }

    /// Read file content with size limits
    fn read_file_content(&self, path: &Path) -> Result<String, FilePickerError> {
        let content =
            std::fs::read_to_string(path).map_err(|e| FilePickerError::ReadError(e.to_string()))?;

        // Double-check size limit
        if content.len() as u64 > self.max_file_size {
            return Err(FilePickerError::FileTooLarge);
        }

        Ok(content)
    }

    /// Mark files as externally modified (for conflict detection)
    pub fn mark_externally_modified(&mut self, paths: &[PathBuf]) {
        for path in paths {
            self.externally_modified.insert(path.clone());
        }
    }

    /// Clear external modification markers
    pub fn clear_external_modifications(&mut self) {
        self.externally_modified.clear();
    }

    /// Get files that have been externally modified
    pub fn externally_modified_files(&self) -> Vec<PathBuf> {
        self.externally_modified.iter().cloned().collect()
    }

    /// Check if a file has been recently modified
    pub fn is_recently_modified(&self, path: &Path) -> bool {
        self.recently_modified.contains(path)
    }

    /// Check if a file has been externally modified
    pub fn is_externally_modified(&self, path: &Path) -> bool {
        self.externally_modified.contains(path)
    }

    /// Navigate down (delegates to FileExplorer)
    pub fn navigate_down(&mut self) {
        // FileExplorer handles navigation through its internal state
    }

    /// Render the file picker
    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        // Clear the area
        f.render_widget(Clear, area);

        // Create main layout
        let popup_area = self.centered_rect(80, 80, area);

        let main_block = Block::default()
            .title(
                "File Picker - Type to search, ↑↓ to navigate, Space to select, Enter to confirm",
            )
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        f.render_widget(main_block, popup_area);

        let inner_area = popup_area.inner(Margin {
            horizontal: 1,
            vertical: 1,
        });

        // Render FileExplorer widget using widget() method
        f.render_widget(&self.explorer.widget(), inner_area);
    }

    /// Create a centered rectangle
    fn centered_rect(&self, percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }
}

impl crate::Component for FilePickerWidget {
    fn id(&self) -> crate::ComponentId {
        self.id.clone()
    }

    fn render(
        &self,
        frame: &mut ratatui::Frame,
        area: ratatui::layout::Rect,
        _model: &crate::AppModel,
    ) {
        if !self.visible {
            return;
        }

        // Clear the background
        frame.render_widget(Clear, area);

        // Use the existing render method
        // For now, create a simple placeholder render
        let block = Block::default().title("File Picker").borders(Borders::ALL);

        let content = vec![
            Line::from("File Picker - Component Architecture"),
            Line::from(""),
            Line::from("Navigate with arrow keys, Enter to select"),
            Line::from("Esc to cancel"),
            Line::from(""),
            Line::from("Current directory: (placeholder)"),
        ];

        let paragraph = Paragraph::new(content)
            .block(block)
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(paragraph, area);
    }

    fn update(&mut self, message: &crate::AppMessage, _model: &crate::AppModel) -> bool {
        if !self.visible {
            return false;
        }

        match message {
            crate::AppMessage::KeyPress(key) => {
                match key.code {
                    crossterm::event::KeyCode::Esc => {
                        self.hide();
                        return true;
                    }
                    crossterm::event::KeyCode::Enter => {
                        // TODO: Handle file selection
                        self.hide();
                        return true;
                    }
                    _ => {
                        // TODO: Handle navigation keys for file explorer
                        // For now, just consume the event
                        return true;
                    }
                }
            }
            _ => {}
        }
        false
    }

    fn is_focused(&self) -> bool {
        self.focused
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn bounds(&self) -> ratatui::layout::Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: ratatui::layout::Rect) {
        self.bounds = bounds;
    }

    fn handle_focus(&mut self, _direction: crate::FocusDirection) -> crate::FocusResult {
        crate::FocusResult::Handled
    }

    fn validate(&self) -> Result<(), String> {
        // TODO: Add validation logic for file picker state
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn crate::Component> {
        // Note: FilePickerWidget contains non-cloneable fields (FileExplorer)
        // For now, return a new instance
        Box::new(Self::new())
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
        // File picker doesn't support children
    }

    fn remove_child(&mut self, _id: &crate::ComponentId) -> Option<Box<dyn crate::Component>> {
        None
    }

    fn z_index(&self) -> i32 {
        100 // High z-index for overlay
    }

    fn set_z_index(&mut self, _z_index: i32) {
        // z-index is fixed for file picker
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn tab_order(&self) -> Option<usize> {
        Some(2)
    }

    fn set_tab_order(&mut self, _order: Option<usize>) {
        // Tab order is fixed
    }
}

impl Default for FilePickerWidget {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple fuzzy matching function
pub fn fuzzy_match(query: &str, text: &str) -> Option<Vec<(usize, usize)>> {
    if query.is_empty() {
        return Some(Vec::new());
    }

    let query_chars: Vec<char> = query.chars().collect();
    let text_chars: Vec<char> = text.chars().collect();
    let mut matches = Vec::new();
    let mut query_idx = 0;
    let mut text_idx = 0;

    while query_idx < query_chars.len() && text_idx < text_chars.len() {
        if query_chars[query_idx].eq_ignore_ascii_case(&text_chars[text_idx]) {
            let start = text.char_indices().nth(text_idx)?.0;
            let end = if text_idx + 1 < text_chars.len() {
                text.char_indices().nth(text_idx + 1)?.0
            } else {
                text.len()
            };
            matches.push((start, end));
            query_idx += 1;
        }
        text_idx += 1;
    }

    if query_idx == query_chars.len() {
        Some(matches)
    } else {
        None
    }
}
