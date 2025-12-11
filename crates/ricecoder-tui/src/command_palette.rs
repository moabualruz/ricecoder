//! Command palette widget for fuzzy command search and execution
//!
//! This module provides a command palette widget that allows users to search for
//! and execute commands using fuzzy matching, with descriptions and keyboard shortcuts.

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};
use unicode_width::UnicodeWidthStr;

/// A command that can be executed from the palette
#[derive(Debug, Clone)]
pub struct PaletteCommand {
    /// Command name/identifier
    pub name: String,
    /// Human-readable display name
    pub display_name: String,
    /// Command description
    pub description: String,
    /// Keyboard shortcut (if any)
    pub shortcut: Option<String>,
    /// Command category for grouping
    pub category: String,
}

/// Command palette widget state
#[derive(Debug, Clone)]
pub struct CommandPaletteWidget {
    /// All available commands
    commands: Vec<PaletteCommand>,
    /// Current search query
    query: String,
    /// Currently selected command index
    selected_index: usize,
    /// Filtered commands based on search
    filtered_commands: Vec<(PaletteCommand, Vec<(usize, usize)>)>, // (command, match_positions)
    /// Whether the palette is visible
    visible: bool,
    /// Maximum number of visible items
    max_visible_items: usize,
}

impl Default for CommandPaletteWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandPaletteWidget {
    /// Create a new command palette widget
    pub fn new() -> Self {
        CommandPaletteWidget {
            commands: Vec::new(),
            query: String::new(),
            selected_index: 0,
            filtered_commands: Vec::new(),
            visible: false,
            max_visible_items: 10,
        }
    }

    /// Add a command to the palette
    pub fn add_command(&mut self, command: PaletteCommand) {
        self.commands.push(command);
        self.update_filtered_commands();
    }

    /// Add multiple commands to the palette
    pub fn add_commands(&mut self, commands: Vec<PaletteCommand>) {
        self.commands.extend(commands);
        self.update_filtered_commands();
    }

    /// Set the search query
    pub fn set_query(&mut self, query: String) {
        self.query = query;
        self.selected_index = 0;
        self.update_filtered_commands();
    }

    /// Get the current search query
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Move selection up
    pub fn select_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Move selection down
    pub fn select_down(&mut self) {
        if self.selected_index < self.filtered_commands.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    /// Get the currently selected command
    pub fn selected_command(&self) -> Option<&PaletteCommand> {
        self.filtered_commands
            .get(self.selected_index)
            .map(|(cmd, _)| cmd)
    }

    /// Show the command palette
    pub fn show(&mut self) {
        self.visible = true;
        self.query.clear();
        self.selected_index = 0;
        self.update_filtered_commands();
    }

    /// Hide the command palette
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Check if the palette is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Get the number of filtered commands
    pub fn filtered_count(&self) -> usize {
        self.filtered_commands.len()
    }

    /// Get the total number of commands
    pub fn total_commands(&self) -> usize {
        self.commands.len()
    }

    /// Update the filtered commands based on the current query
    fn update_filtered_commands(&mut self) {
        if self.query.is_empty() {
            // Show all commands when no query
            self.filtered_commands = self
                .commands
                .iter()
                .map(|cmd| (cmd.clone(), Vec::new()))
                .collect();
        } else {
            // Fuzzy search
            let query_lower = self.query.to_lowercase();
            let mut scored_commands = Vec::new();

            for command in &self.commands {
                let name_lower = command.display_name.to_lowercase();
                let desc_lower = command.description.to_lowercase();

                // Check if query matches name or description
                let (text, matches) = if let Some(matches) = fuzzy_match(&query_lower, &name_lower) {
                    (&command.display_name, matches)
                } else if let Some(matches) = fuzzy_match(&query_lower, &desc_lower) {
                    (&command.description, matches)
                } else {
                    continue; // No match
                };

                scored_commands.push((command.clone(), matches));
            }

            // Sort by relevance (simplified - just by match count)
            scored_commands.sort_by(|a, b| {
                let a_score = a.1.len();
                let b_score = b.1.len();
                b_score.cmp(&a_score) // Higher match count first
            });

            self.filtered_commands = scored_commands;
        }

        // Ensure selected index is valid
        if self.selected_index >= self.filtered_commands.len() && !self.filtered_commands.is_empty() {
            self.selected_index = self.filtered_commands.len() - 1;
        }
    }

    /// Render the command palette overlay
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        // Create overlay area (centered, taking up most of the screen)
        let overlay_width = (area.width * 3 / 4).min(80);
        let overlay_height = (area.height * 3 / 4).min(20);
        let overlay_x = (area.width - overlay_width) / 2;
        let overlay_y = (area.height - overlay_height) / 2;

        let overlay_area = Rect {
            x: overlay_x,
            y: overlay_y,
            width: overlay_width,
            height: overlay_height,
        };

        // Clear background and draw border
        frame.render_widget(Clear, overlay_area);
        let block = Block::default()
            .title("Command Palette")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        frame.render_widget(block, overlay_area);

        let inner_area = overlay_area.inner(&Margin {
            vertical: 1,
            horizontal: 1,
        });

        // Split into search bar and results
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Search bar
                Constraint::Min(1),    // Results
            ])
            .split(inner_area);

        // Search bar
        let search_text = format!("> {}", self.query);
        let search_paragraph = Paragraph::new(search_text)
            .style(Style::default().fg(Color::White))
            .block(Block::default().borders(Borders::BOTTOM));
        frame.render_widget(search_paragraph, chunks[0]);

        // Results list
        let visible_items = self.filtered_commands.len().min(self.max_visible_items);
        let start_index = if self.selected_index >= self.max_visible_items {
            self.selected_index.saturating_sub(self.max_visible_items - 1)
        } else {
            0
        };

        let items: Vec<ListItem> = self
            .filtered_commands
            .iter()
            .skip(start_index)
            .take(visible_items)
            .enumerate()
            .map(|(i, (cmd, matches))| {
                let actual_index = start_index + i;
                let is_selected = actual_index == self.selected_index;

                // Create highlighted text
                let display_text = self.create_highlighted_text(&cmd.display_name, matches);
                let desc_text = if cmd.description.len() > 60 {
                    format!("{}...", &cmd.description[..57])
                } else {
                    cmd.description.clone()
                };

                let shortcut_text = cmd
                    .shortcut
                    .as_ref()
                    .map(|s| format!(" ({})", s))
                    .unwrap_or_default();

                let full_text = format!("{}{}", display_text, shortcut_text);
                let desc_line = Line::from(vec![
                    Span::styled(desc_text, Style::default().fg(Color::Gray)),
                ]);

                let style = if is_selected {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                };

                ListItem::new(vec![
                    Line::from(full_text).style(style),
                    desc_line,
                ])
            })
            .collect();

        let results_list = List::new(items)
            .block(Block::default())
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::White));

        frame.render_widget(results_list, chunks[1]);

        // Show result count
        if self.filtered_commands.len() > visible_items {
            let count_text = format!(
                "{} / {} commands",
                self.filtered_commands.len(),
                self.commands.len()
            );
            let count_paragraph = Paragraph::new(count_text)
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Right);
            frame.render_widget(count_paragraph, chunks[1]);
        }
    }

    /// Create highlighted text with fuzzy match positions
    fn create_highlighted_text(&self, text: &str, matches: &[(usize, usize)]) -> String {
        let mut result = String::new();
        let chars: Vec<char> = text.chars().collect();
        let mut last_end = 0;

        for &(start, end) in matches {
            // Add non-matching text
            if last_end < start {
                for i in last_end..start {
                    if i < chars.len() {
                        result.push(chars[i]);
                    }
                }
            }

            // Add highlighted matching text
            for i in start..end {
                if i < chars.len() {
                    result.push(chars[i]);
                }
            }

            last_end = end;
        }

        // Add remaining text
        for i in last_end..chars.len() {
            result.push(chars[i]);
        }

        result
    }
}

/// Fuzzy matching function
/// Returns Some(vec of (start, end) positions) if query matches text, None otherwise
fn fuzzy_match(query: &str, text: &str) -> Option<Vec<(usize, usize)>> {
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
            let start = text_idx;
            // Find consecutive matching characters
            while query_idx < query_chars.len()
                && text_idx < text_chars.len()
                && query_chars[query_idx].eq_ignore_ascii_case(&text_chars[text_idx])
            {
                query_idx += 1;
                text_idx += 1;
            }
            let end = text_idx;
            matches.push((start, end));
        } else {
            text_idx += 1;
        }
    }

    if query_idx == query_chars.len() {
        Some(matches)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_palette_creation() {
        let palette = CommandPaletteWidget::new();
        assert!(!palette.is_visible());
        assert_eq!(palette.query(), "");
        assert_eq!(palette.filtered_count(), 0);
    }

    #[test]
    fn test_add_commands() {
        let mut palette = CommandPaletteWidget::new();

        let cmd1 = PaletteCommand {
            name: "help".to_string(),
            display_name: "Help".to_string(),
            description: "Show help information".to_string(),
            shortcut: Some("F1".to_string()),
            category: "General".to_string(),
        };

        let cmd2 = PaletteCommand {
            name: "quit".to_string(),
            display_name: "Quit".to_string(),
            description: "Exit the application".to_string(),
            shortcut: Some("Ctrl+Q".to_string()),
            category: "General".to_string(),
        };

        palette.add_commands(vec![cmd1, cmd2]);
        assert_eq!(palette.filtered_count(), 2);
    }

    #[test]
    fn test_fuzzy_matching() {
        assert_eq!(fuzzy_match("he", "help"), Some(vec![(0, 2)]));
        assert_eq!(fuzzy_match("hp", "help"), Some(vec![(0, 1), (3, 4)]));
        assert_eq!(fuzzy_match("xyz", "help"), None);
        assert_eq!(fuzzy_match("", "help"), Some(vec![]));
    }

    #[test]
    fn test_selection_navigation() {
        let mut palette = CommandPaletteWidget::new();

        let cmd = PaletteCommand {
            name: "test".to_string(),
            display_name: "Test".to_string(),
            description: "Test command".to_string(),
            shortcut: None,
            category: "Test".to_string(),
        };

        palette.add_command(cmd);
        palette.show();

        assert_eq!(palette.selected_index, 0);
        palette.select_down();
        assert_eq!(palette.selected_index, 0); // Can't go down with only one item

        palette.select_up();
        assert_eq!(palette.selected_index, 0); // Can't go up from first item
    }
}