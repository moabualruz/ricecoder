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

            // Component trait fields
            id: "command-palette".to_string(),
            focused: false,
            enabled: true,
            bounds: ratatui::layout::Rect::new(0, 0, 80, 20),
        }
    }

    /// Show the command palette
    pub fn show(&mut self) {
        self.visible = true;
        self.focused = true;
        self.query.clear();
        self.selected_index = 0;
        self.update_filtered_commands();
    }

    /// Hide the command palette
    pub fn hide(&mut self) {
        self.visible = false;
        self.focused = false;
        self.query.clear();
        self.selected_index = 0;
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
                let (text, matches) = if let Some(matches) = fuzzy_match(&query_lower, &name_lower)
                {
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
        if self.selected_index >= self.filtered_commands.len() && !self.filtered_commands.is_empty()
        {
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

        let inner_area = overlay_area.inner(Margin {
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
            self.selected_index
                .saturating_sub(self.max_visible_items - 1)
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
                let desc_line = Line::from(vec![Span::styled(
                    desc_text,
                    Style::default().fg(Color::Gray),
                )]);

                let style = if is_selected {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                };

                ListItem::new(vec![Line::from(full_text).style(style), desc_line])
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

impl crate::Component for CommandPaletteWidget {
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

        // Create the main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Search input
                Constraint::Min(5),    // Command list
            ])
            .split(area);

        // Render search input
        let search_text = if self.query.is_empty() {
            Span::styled(
                "Type to search commands...",
                Style::default().fg(Color::Gray),
            )
        } else {
            Span::raw(&self.query)
        };

        let search_block = Block::default()
            .title("Command Palette")
            .borders(Borders::ALL);

        let search_paragraph = Paragraph::new(Line::from(search_text)).block(search_block);

        frame.render_widget(search_paragraph, chunks[0]);

        // Render command list
        let mut list_items = Vec::new();
        for (idx, (command, matches)) in self.filtered_commands.iter().enumerate() {
            let mut spans = Vec::new();

            // Highlight matching characters
            let mut last_end = 0;
            for &(start, end) in matches {
                // Add non-matching text before this match
                if start > last_end {
                    spans.push(Span::raw(&command.display_name[last_end..start]));
                }
                // Add matching text with highlight
                spans.push(Span::styled(
                    &command.display_name[start..end],
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ));
                last_end = end;
            }

            // Add remaining text
            if last_end < command.display_name.len() {
                spans.push(Span::raw(&command.display_name[last_end..]));
            }

            // Add description and shortcut
            let mut line_spans = spans;
            line_spans.push(Span::raw(" - "));
            line_spans.push(Span::styled(
                &command.description,
                Style::default().fg(Color::Gray),
            ));

            if let Some(shortcut) = &command.shortcut {
                line_spans.push(Span::raw(" ("));
                line_spans.push(Span::styled(shortcut, Style::default().fg(Color::Cyan)));
                line_spans.push(Span::raw(")"));
            }

            let style = if idx == self.selected_index {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default()
            };

            list_items.push(ListItem::new(Line::from(line_spans)).style(style));
        }

        let list = List::new(list_items).block(Block::default().borders(Borders::ALL));

        frame.render_widget(list, chunks[1]);
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
                        // Execute selected command
                        if let Some((command, _)) = self.filtered_commands.get(self.selected_index)
                        {
                            // TODO: Execute command
                            tracing::info!("Executing command: {}", command.name);
                            self.hide();
                            return true;
                        }
                    }
                    crossterm::event::KeyCode::Up => {
                        if self.selected_index > 0 {
                            self.selected_index -= 1;
                        }
                        return true;
                    }
                    crossterm::event::KeyCode::Down => {
                        if self.selected_index < self.filtered_commands.len().saturating_sub(1) {
                            self.selected_index += 1;
                        }
                        return true;
                    }
                    crossterm::event::KeyCode::Char(c) => {
                        self.query.push(c);
                        self.update_filtered_commands();
                        self.selected_index = 0;
                        return true;
                    }
                    crossterm::event::KeyCode::Backspace => {
                        self.query.pop();
                        self.update_filtered_commands();
                        self.selected_index = 0;
                        return true;
                    }
                    _ => {}
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
        if self.selected_index >= self.filtered_commands.len() && !self.filtered_commands.is_empty()
        {
            return Err("Selected index out of bounds".to_string());
        }
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn crate::Component> {
        Box::new(self.clone())
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
        // Command palette doesn't support children
    }

    fn remove_child(&mut self, _id: &crate::ComponentId) -> Option<Box<dyn crate::Component>> {
        None
    }

    fn z_index(&self) -> i32 {
        100 // High z-index for overlay
    }

    fn set_z_index(&mut self, _z_index: i32) {
        // z-index is fixed for command palette
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn tab_order(&self) -> Option<usize> {
        Some(1)
    }

    fn set_tab_order(&mut self, _order: Option<usize>) {
        // Tab order is fixed
    }
}

/// Fuzzy matching function using nucleo (Helix editor's fuzzy matcher)
/// Returns Some(vec of (start, end) positions) if query matches text, None otherwise
fn fuzzy_match(query: &str, text: &str) -> Option<Vec<(usize, usize)>> {
    use nucleo::pattern::{AtomKind, CaseMatching, Normalization, Pattern};
    use nucleo::{Config, Matcher, Utf32Str};

    if query.is_empty() {
        return Some(Vec::new());
    }

    let mut matcher = Matcher::new(Config::DEFAULT);
    let pattern = Pattern::new(
        query,
        CaseMatching::Smart,
        Normalization::Smart,
        AtomKind::Fuzzy,
    );

    let mut buf = Vec::new();
    let haystack = Utf32Str::new(text, &mut buf);
    let mut indices = Vec::new();

    if pattern.indices(haystack, &mut matcher, &mut indices).is_some() {
        // Convert indices to (start, end) pairs
        let mut matches = Vec::new();
        let mut i = 0;
        while i < indices.len() {
            let start = indices[i] as usize;
            let mut end = start + 1;
            // Group consecutive indices
            while i + 1 < indices.len() && indices[i + 1] == indices[i] + 1 {
                i += 1;
                end = indices[i] as usize + 1;
            }
            matches.push((start, end));
            i += 1;
        }
        Some(matches)
    } else {
        None
    }
}
