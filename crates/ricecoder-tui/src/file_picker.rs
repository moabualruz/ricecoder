//! File picker widget for selecting files to include in messages
//!
//! This module provides a file picker overlay that allows users to browse,
//! search, and select files from the filesystem with tree navigation and
//! fuzzy search capabilities.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

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
#[derive(Debug, Clone)]
pub struct FilePickerWidget {
    /// Current search query
    search_query: String,
    /// Filtered file paths based on search
    filtered_files: Vec<(PathBuf, FileInfo)>,
    /// Selected file indices
    selected_indices: HashSet<usize>,
    /// Whether the picker is visible
    visible: bool,
    /// Current working directory
    cwd: PathBuf,
    /// Maximum number of visible items
    max_visible_items: usize,
    /// Scroll offset
    scroll_offset: usize,
    /// Maximum file size to include (in bytes)
    max_file_size: u64,
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
        Self {
            search_query: String::new(),
            filtered_files: Vec::new(),
            selected_indices: HashSet::new(),
            visible: false,
            cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            max_visible_items: 15,
            scroll_offset: 0,
            max_file_size: 1024 * 1024, // 1MB default
        }
    }

    /// Set the maximum file size for inclusion
    pub fn set_max_file_size(&mut self, max_size: u64) {
        self.max_file_size = max_size;
    }

    /// Show the file picker
    pub fn show(&mut self) {
        self.visible = true;
        self.refresh_file_list();
        self.update_filtered_files();
    }

    /// Hide the file picker
    pub fn hide(&mut self) {
        self.visible = false;
        self.search_query.clear();
        self.selected_indices.clear();
        self.scroll_offset = 0;
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

    /// Get selected file paths
    pub fn selected_files(&self) -> Vec<PathBuf> {
        self.selected_indices
            .iter()
            .filter_map(|&idx| self.filtered_files.get(idx))
            .map(|(path, _)| path.clone())
            .collect()
    }

    /// Get selected files with their information
    pub fn selected_files_with_info(&self) -> Vec<(PathBuf, FileInfo)> {
        self.selected_indices
            .iter()
            .filter_map(|&idx| self.filtered_files.get(idx))
            .cloned()
            .collect()
    }

    /// Handle character input for search
    pub fn input_char(&mut self, c: char) {
        self.search_query.push(c);
        self.update_filtered_files();
        self.selected_indices.clear();
        self.scroll_offset = 0;
    }

    /// Handle backspace for search
    pub fn backspace(&mut self) {
        self.search_query.pop();
        self.update_filtered_files();
        self.selected_indices.clear();
        self.scroll_offset = 0;
    }

    /// Navigate up
    pub fn navigate_up(&mut self) {
        if self.filtered_files.is_empty() {
            return;
        }

        // If no selection, select last item
        if self.selected_indices.is_empty() {
            let last_idx = self.filtered_files.len().saturating_sub(1);
            self.selected_indices.insert(last_idx);
            self.adjust_scroll(last_idx);
            return;
        }

        // Move to previous item
        let current = *self.selected_indices.iter().next().unwrap();
        if current > 0 {
            self.selected_indices.clear();
            self.selected_indices.insert(current - 1);
            self.adjust_scroll(current - 1);
        }
    }

    /// Navigate down
    pub fn navigate_down(&mut self) {
        if self.filtered_files.is_empty() {
            return;
        }

        // If no selection, select first item
        if self.selected_indices.is_empty() {
            self.selected_indices.insert(0);
            self.adjust_scroll(0);
            return;
        }

        // Move to next item
        let current = *self.selected_indices.iter().next().unwrap();
        if current + 1 < self.filtered_files.len() {
            self.selected_indices.clear();
            self.selected_indices.insert(current + 1);
            self.adjust_scroll(current + 1);
        }
    }

    /// Toggle selection of current item
    pub fn toggle_selection(&mut self) {
        if self.filtered_files.is_empty() || self.selected_indices.is_empty() {
            return;
        }

        let current = *self.selected_indices.iter().next().unwrap();
        if self.selected_indices.contains(&current) {
            self.selected_indices.remove(&current);
        } else {
            self.selected_indices.insert(current);
        }
    }

    /// Select all visible items
    pub fn select_all(&mut self) {
        self.selected_indices.clear();
        for i in 0..self.filtered_files.len() {
            self.selected_indices.insert(i);
        }
    }

    /// Clear all selections
    pub fn clear_selection(&mut self) {
        self.selected_indices.clear();
    }

    /// Clear search query
    pub fn clear_search(&mut self) {
        self.search_query.clear();
        self.update_filtered_files();
        self.selected_indices.clear();
        self.scroll_offset = 0;
    }

    /// Confirm selection and return selected files with content
    pub fn confirm_selection(&mut self) -> Result<Vec<FileSelection>, FilePickerError> {
        let selected = self.selected_files_with_info();
        let mut results = Vec::new();

        for (path, info) in selected {
            let selection = if path.is_dir() {
                // Directories are not included as content
                FileSelection {
                    path,
                    content: None,
                    info,
                    status: FileSelectionStatus::Directory,
                }
            } else if info.is_binary {
                FileSelection {
                    path,
                    content: None,
                    info,
                    status: FileSelectionStatus::BinaryFile,
                }
            } else if info.is_too_large {
                FileSelection {
                    path,
                    content: None,
                    info,
                    status: FileSelectionStatus::TooLarge,
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
        let content = std::fs::read_to_string(path)
            .map_err(|e| FilePickerError::ReadError(e.to_string()))?;

        // Double-check size limit
        if content.len() as u64 > self.max_file_size {
            return Err(FilePickerError::FileTooLarge);
        }

        Ok(content)
    }

    /// Refresh the file list from current directory
    fn refresh_file_list(&mut self) {
        self.filtered_files.clear();

        if let Ok(entries) = std::fs::read_dir(&self.cwd) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    let name = entry.file_name().to_string_lossy().to_string();

                    // Skip hidden files unless search query starts with .
                    if name.starts_with('.') && !self.search_query.starts_with('.') {
                        continue;
                    }

                    let path = entry.path();
                    let file_info = self.get_file_info(&path);
                    self.filtered_files.push((path, file_info));
                }
            }
        }

        // Sort files: directories first, then files, alphabetically
        self.filtered_files.sort_by(|a, b| {
            let a_is_dir = a.0.is_dir();
            let b_is_dir = b.0.is_dir();

            if a_is_dir && !b_is_dir {
                std::cmp::Ordering::Less
            } else if !a_is_dir && b_is_dir {
                std::cmp::Ordering::Greater
            } else {
                a.0.file_name()
                    .unwrap_or_default()
                    .cmp(b.0.file_name().unwrap_or_default())
            }
        });
    }

    /// Get file information including size and binary detection
    fn get_file_info(&self, path: &Path) -> FileInfo {
        let mut size = 0;
        let mut is_binary = false;

        if path.is_file() {
            // Get file size
            if let Ok(metadata) = std::fs::metadata(path) {
                size = metadata.len();
            }

            // Check if file is binary by reading first 512 bytes
            if let Ok(content) = std::fs::read(path) {
                // Check for null bytes or non-ASCII characters
                let sample = &content[..content.len().min(512)];
                is_binary = sample.iter().any(|&b| b == 0) ||
                           sample.iter().any(|&b| b < 32 && b != 9 && b != 10 && b != 13);
            }
        }

        let is_too_large = size > self.max_file_size;

        FileInfo {
            size,
            is_binary,
            is_too_large,
        }
    }

    /// Update filtered files based on search query
    fn update_filtered_files(&mut self) {
        if self.search_query.is_empty() {
            self.refresh_file_list();
            return;
        }

        let query_lower = self.search_query.to_lowercase();
        self.filtered_files.retain(|(path, _)| {
            let name = path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_lowercase();
            fuzzy_match(&query_lower, &name).is_some()
        });

        // Re-sort after filtering
        self.filtered_files.sort_by(|a, b| {
            let a_name = a.0.file_name().unwrap_or_default().to_string_lossy();
            let b_name = b.0.file_name().unwrap_or_default().to_string_lossy();

            // Exact matches first
            let a_exact = a_name.to_lowercase().starts_with(&query_lower);
            let b_exact = b_name.to_lowercase().starts_with(&query_lower);

            if a_exact && !b_exact {
                std::cmp::Ordering::Less
            } else if !a_exact && b_exact {
                std::cmp::Ordering::Greater
            } else {
                a_name.cmp(&b_name)
            }
        });
    }

    /// Adjust scroll to keep selected item visible
    fn adjust_scroll(&mut self, selected_idx: usize) {
        if selected_idx < self.scroll_offset {
            self.scroll_offset = selected_idx;
        } else if selected_idx >= self.scroll_offset + self.max_visible_items {
            self.scroll_offset = selected_idx.saturating_sub(self.max_visible_items - 1);
        }
    }

    /// Render the file picker
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        // Clear the area
        frame.render_widget(Clear, area);

        // Create main layout
        let popup_area = self.centered_rect(80, 80, area);

        let main_block = Block::default()
            .title("File Picker - Type to search, ↑↓ to navigate, Space to select, Enter to confirm")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        frame.render_widget(main_block, popup_area);

        let inner_area = popup_area.inner(&Margin {
            horizontal: 1,
            vertical: 1,
        });

        // Create layout for search and file list
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Search input
                Constraint::Min(5),    // File list
                Constraint::Length(1), // Footer
            ])
            .split(inner_area);

        // Render search input
        self.render_search_input(frame, chunks[0]);

        // Render file list
        self.render_file_list(frame, chunks[1]);

        // Render footer
        self.render_footer(frame, chunks[2]);
    }

    /// Render search input
    fn render_search_input(&self, frame: &mut Frame, area: Rect) {
        let search_text = if self.search_query.is_empty() {
            "Search files...".to_string()
        } else {
            self.search_query.clone()
        };

        let input = Paragraph::new(search_text)
            .block(
                Block::default()
                    .title("Search")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .style(Style::default().fg(Color::White));

        frame.render_widget(input, area);
    }

    /// Render file list
    fn render_file_list(&self, frame: &mut Frame, area: Rect) {
        let visible_files: Vec<ListItem> = self
            .filtered_files
            .iter()
            .skip(self.scroll_offset)
            .take(self.max_visible_items)
            .enumerate()
            .map(|(i, (path, file_info))| {
                let actual_idx = i + self.scroll_offset;
                let file_name = path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy();

                let is_selected = self.selected_indices.contains(&actual_idx);
                let is_current = self.selected_indices.len() == 1 && self.selected_indices.contains(&actual_idx);

                // Create styled spans
                let mut spans = Vec::new();

                if is_selected {
                    spans.push(Span::styled("[✓] ", Style::default().fg(Color::Green)));
                } else {
                    spans.push(Span::styled("[ ] ", Style::default().fg(Color::Gray)));
                }

                // Add directory indicator
                if path.is_dir() {
                    spans.push(Span::styled("📁 ", Style::default().fg(Color::Blue)));
                } else {
                    spans.push(Span::styled("📄 ", Style::default().fg(Color::White)));
                }

                // Add warning indicators
                if file_info.is_binary {
                    spans.push(Span::styled("🔒 ", Style::default().fg(Color::Red)));
                } else if file_info.is_too_large {
                    spans.push(Span::styled("⚠️ ", Style::default().fg(Color::Yellow)));
                }

                // Highlight matches
                if self.search_query.is_empty() {
                    spans.push(Span::styled(file_name, Style::default()));
                } else {
                    let query_lower = self.search_query.to_lowercase();
                    let name_lower = file_name.to_lowercase();

                    if let Some(matches) = fuzzy_match(&query_lower, &name_lower) {
                        let chars: Vec<char> = file_name.chars().collect();
                        let mut last_end = 0;

                        for &(start, end) in &matches {
                            if start > last_end {
                                let normal_text: String = chars[last_end..start].iter().collect();
                                spans.push(Span::styled(normal_text, Style::default()));
                            }

                            let highlight_text: String = chars[start..end].iter().collect();
                            spans.push(Span::styled(
                                highlight_text,
                                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                            ));
                            last_end = end;
                        }

                        if last_end < chars.len() {
                            let remaining: String = chars[last_end..].iter().collect();
                            spans.push(Span::styled(remaining, Style::default()));
                        }
                    } else {
                        spans.push(Span::styled(file_name, Style::default()));
                    }
                }

                // Add file size info for files
                if !path.is_dir() {
                    let size_str = self.format_file_size(file_info.size);
                    spans.push(Span::styled(
                        format!(" ({})", size_str),
                        Style::default().fg(Color::Gray)
                    ));
                }

                let mut style = Style::default();
                if is_current {
                    style = style.bg(Color::DarkGray);
                }

                ListItem::new(Line::from(spans)).style(style)
            })
            .collect();

        let list = List::new(visible_files)
            .block(
                Block::default()
                    .title(format!("Files ({}/{})", self.filtered_files.len(), self.filtered_files.len()))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue)),
            );

        frame.render_widget(list, area);
    }

    /// Format file size for display
    fn format_file_size(&self, size: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB"];

        if size == 0 {
            return "0B".to_string();
        }

        let mut size = size as f64;
        let mut unit_idx = 0;

        while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
            size /= 1024.0;
            unit_idx += 1;
        }

        if unit_idx == 0 {
            format!("{}B", size as u64)
        } else {
            format!("{:.1}{}", size, UNITS[unit_idx])
        }
    }

    /// Render footer
    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let selected_count = self.selected_indices.len();
        let footer_text = if selected_count > 0 {
            format!("Selected: {} files | Enter: Confirm | Esc: Cancel", selected_count)
        } else {
            "↑↓: Navigate | Space: Select | Enter: Confirm | Esc: Cancel".to_string()
        };

        let footer = Paragraph::new(footer_text)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);

        frame.render_widget(footer, area);
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

