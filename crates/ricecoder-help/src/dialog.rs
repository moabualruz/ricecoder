//! Help dialog widget implementation

use crate::{HelpContent, HelpSearch, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    widgets::{
        Block, Borders, Clear, List, ListItem, ListState, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Wrap,
    },
    Frame,
};

/// Help dialog state
#[derive(Debug, Clone, PartialEq)]
pub enum HelpDialogState {
    Browse,
    Search,
}

/// Help dialog widget with scrollable content and search functionality
pub struct HelpDialog {
    content: HelpContent,
    search: HelpSearch,
    state: HelpDialogState,
    selected_category: usize,
    selected_item: usize,
    scroll_offset: usize,
    search_input: String,
    list_state: ListState,
    scrollbar_state: ScrollbarState,
    visible: bool,
}

impl HelpDialog {
    /// Create a new help dialog
    pub fn new(content: HelpContent) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        
        Self {
            content,
            search: HelpSearch::new(),
            state: HelpDialogState::Browse,
            selected_category: 0,
            selected_item: 0,
            scroll_offset: 0,
            search_input: String::new(),
            list_state,
            scrollbar_state: ScrollbarState::default(),
            visible: false,
        }
    }
    
    /// Create help dialog with default RiceCoder content
    pub fn default_ricecoder() -> Self {
        Self::new(HelpContent::default_ricecoder_help())
    }
    
    /// Show the help dialog
    pub fn show(&mut self) {
        self.visible = true;
    }
    
    /// Hide the help dialog
    pub fn hide(&mut self) {
        self.visible = false;
        self.state = HelpDialogState::Browse;
        self.search_input.clear();
        self.search.clear();
    }
    
    /// Check if dialog is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }
    
    /// Toggle dialog visibility
    pub fn toggle(&mut self) {
        if self.visible {
            self.hide();
        } else {
            self.show();
        }
    }
    
    /// Handle keyboard input
    pub fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        if !self.visible {
            return Ok(false);
        }
        
        match self.state {
            HelpDialogState::Browse => self.handle_browse_key(key),
            HelpDialogState::Search => self.handle_search_key(key),
        }
    }
    
    /// Handle keyboard input in browse mode
    fn handle_browse_key(&mut self, key: KeyEvent) -> Result<bool> {
        match (key.code, key.modifiers) {
            // Close dialog
            (KeyCode::Esc, _) => {
                self.hide();
                Ok(true)
            }
            
            // Start search
            (KeyCode::Char('f'), KeyModifiers::CONTROL) => {
                self.state = HelpDialogState::Search;
                self.search_input.clear();
                Ok(true)
            }
            
            // Navigation
            (KeyCode::Up, _) => {
                self.navigate_up();
                Ok(true)
            }
            (KeyCode::Down, _) => {
                self.navigate_down();
                Ok(true)
            }
            (KeyCode::Left, _) => {
                self.navigate_left();
                Ok(true)
            }
            (KeyCode::Right, _) => {
                self.navigate_right();
                Ok(true)
            }
            
            // Page navigation
            (KeyCode::PageUp, _) => {
                self.page_up();
                Ok(true)
            }
            (KeyCode::PageDown, _) => {
                self.page_down();
                Ok(true)
            }
            (KeyCode::Home, _) => {
                self.go_to_top();
                Ok(true)
            }
            (KeyCode::End, _) => {
                self.go_to_bottom();
                Ok(true)
            }
            
            _ => Ok(false),
        }
    }
    
    /// Handle keyboard input in search mode
    fn handle_search_key(&mut self, key: KeyEvent) -> Result<bool> {
        match (key.code, key.modifiers) {
            // Exit search mode
            (KeyCode::Esc, _) => {
                self.state = HelpDialogState::Browse;
                self.search_input.clear();
                self.search.clear();
                Ok(true)
            }
            
            // Perform search
            (KeyCode::Enter, _) => {
                self.search.search(&self.content, &self.search_input)?;
                Ok(true)
            }
            
            // Navigate search results
            (KeyCode::Up, _) => {
                self.search.select_previous();
                Ok(true)
            }
            (KeyCode::Down, _) => {
                self.search.select_next();
                Ok(true)
            }
            
            // Edit search query
            (KeyCode::Char(c), _) => {
                self.search_input.push(c);
                // Auto-search as user types
                self.search.search(&self.content, &self.search_input)?;
                Ok(true)
            }
            (KeyCode::Backspace, _) => {
                self.search_input.pop();
                if self.search_input.is_empty() {
                    self.search.clear();
                } else {
                    self.search.search(&self.content, &self.search_input)?;
                }
                Ok(true)
            }
            
            _ => Ok(false),
        }
    }
    
    /// Navigate up in browse mode
    fn navigate_up(&mut self) {
        if self.selected_item > 0 {
            self.selected_item -= 1;
        } else if self.selected_category > 0 {
            self.selected_category -= 1;
            if let Some(category) = self.content.categories.get(self.selected_category) {
                self.selected_item = category.items.len().saturating_sub(1);
            }
        }
        self.update_scroll();
    }
    
    /// Navigate down in browse mode
    fn navigate_down(&mut self) {
        if let Some(category) = self.content.categories.get(self.selected_category) {
            if self.selected_item + 1 < category.items.len() {
                self.selected_item += 1;
            } else if self.selected_category + 1 < self.content.categories.len() {
                self.selected_category += 1;
                self.selected_item = 0;
            }
        }
        self.update_scroll();
    }
    
    /// Navigate left (previous category)
    fn navigate_left(&mut self) {
        if self.selected_category > 0 {
            self.selected_category -= 1;
            self.selected_item = 0;
        }
        self.update_scroll();
    }
    
    /// Navigate right (next category)
    fn navigate_right(&mut self) {
        if self.selected_category + 1 < self.content.categories.len() {
            self.selected_category += 1;
            self.selected_item = 0;
        }
        self.update_scroll();
    }
    
    /// Page up
    fn page_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(10);
    }
    
    /// Page down
    fn page_down(&mut self) {
        self.scroll_offset += 10;
    }
    
    /// Go to top
    fn go_to_top(&mut self) {
        self.selected_category = 0;
        self.selected_item = 0;
        self.scroll_offset = 0;
    }
    
    /// Go to bottom
    fn go_to_bottom(&mut self) {
        if !self.content.categories.is_empty() {
            self.selected_category = self.content.categories.len() - 1;
            if let Some(category) = self.content.categories.get(self.selected_category) {
                self.selected_item = category.items.len().saturating_sub(1);
            }
        }
        self.update_scroll();
    }
    
    /// Update scroll position based on selection
    fn update_scroll(&mut self) {
        // Calculate total items before current selection
        let mut total_items = 0;
        for (i, category) in self.content.categories.iter().enumerate() {
            if i < self.selected_category {
                total_items += category.items.len() + 1; // +1 for category header
            } else if i == self.selected_category {
                total_items += self.selected_item + 1; // +1 for category header
                break;
            }
        }
        
        // Adjust scroll to keep selection visible
        const VISIBLE_ITEMS: usize = 20; // Approximate visible items
        if total_items >= self.scroll_offset + VISIBLE_ITEMS {
            self.scroll_offset = total_items.saturating_sub(VISIBLE_ITEMS - 1);
        } else if total_items < self.scroll_offset {
            self.scroll_offset = total_items;
        }
    }
    
    /// Render the help dialog
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }
        
        // Clear the area
        frame.render_widget(Clear, area);
        
        // Create main layout
        let popup_area = self.centered_rect(90, 85, area);
        
        let main_block = Block::default()
            .title("RiceCoder Help")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        
        frame.render_widget(main_block, popup_area);
        
        let inner_area = popup_area.inner(Margin {
            horizontal: 1,
            vertical: 1,
        });
        
        match self.state {
            HelpDialogState::Browse => self.render_browse_mode(frame, inner_area),
            HelpDialogState::Search => self.render_search_mode(frame, inner_area),
        }
        
        // Render footer with shortcuts
        self.render_footer(frame, popup_area);
    }
    
    /// Render browse mode
    fn render_browse_mode(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(area);
        
        // Render category list
        self.render_category_list(frame, chunks[0]);
        
        // Render content area
        self.render_content_area(frame, chunks[1]);
    }
    
    /// Render search mode
    fn render_search_mode(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);
        
        // Render search input
        self.render_search_input(frame, chunks[0]);
        
        // Render search results
        self.render_search_results(frame, chunks[1]);
    }
    
    /// Render category list
    fn render_category_list(&mut self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .content
            .categories
            .iter()
            .enumerate()
            .map(|(i, category)| {
                let style = if i == self.selected_category {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                
                ListItem::new(category.name.clone()).style(style)
            })
            .collect();
        
        let list = List::new(items)
            .block(
                Block::default()
                    .title("Categories")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue)),
            )
            .highlight_style(Style::default().bg(Color::DarkGray));
        
        self.list_state.select(Some(self.selected_category));
        frame.render_stateful_widget(list, area, &mut self.list_state);
    }
    
    /// Render content area
    fn render_content_area(&mut self, frame: &mut Frame, area: Rect) {
        if let Some(category) = self.content.categories.get(self.selected_category) {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0)])
                .split(area);
            
            // Category description
            let desc_paragraph = Paragraph::new(category.description.clone())
                .block(
                    Block::default()
                        .title(category.name.as_str())
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Green)),
                )
                .wrap(Wrap { trim: true });
            
            frame.render_widget(desc_paragraph, chunks[0]);
            
            // Items list
            if let Some(item) = category.items.get(self.selected_item) {
                let content = format!("{}\n\n{}", item.title, item.content);
                let paragraph = Paragraph::new(content)
                    .block(
                        Block::default()
                            .title("Content")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Magenta)),
                    )
                    .wrap(Wrap { trim: true })
                    .scroll((self.scroll_offset as u16, 0));
                
                frame.render_widget(paragraph, chunks[1]);
                
                // Render scrollbar
                let scrollbar = Scrollbar::default()
                    .orientation(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(Some("↑"))
                    .end_symbol(Some("↓"));
                
                let content_height = item.content.lines().count() + 2; // +2 for title
                let visible_height = chunks[1].height as usize;
                
                self.scrollbar_state = self.scrollbar_state
                    .content_length(content_height)
                    .viewport_content_length(visible_height)
                    .position(self.scroll_offset);
                
                frame.render_stateful_widget(scrollbar, chunks[1], &mut self.scrollbar_state);
            }
        }
    }
    
    /// Render search input
    fn render_search_input(&self, frame: &mut Frame, area: Rect) {
        let input = Paragraph::new(self.search_input.clone())
            .block(
                Block::default()
                    .title("Search (Ctrl+F)")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow)),
            );
        
        frame.render_widget(input, area);
    }
    
    /// Render search results
    fn render_search_results(&mut self, frame: &mut Frame, area: Rect) {
        let results = self.search.results();
        
        if results.is_empty() {
            let no_results = Paragraph::new("No results found")
                .block(
                    Block::default()
                        .title("Search Results")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Red)),
                )
                .alignment(Alignment::Center);
            
            frame.render_widget(no_results, area);
            return;
        }
        
        let items: Vec<ListItem> = results
            .iter()
            .enumerate()
            .map(|(i, result)| {
                let style = if i == self.search.selected_index() {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                
                let text = format!("[{}] {}", result.category_name, result.item_title);
                ListItem::new(text).style(style)
            })
            .collect();
        
        let list = List::new(items)
            .block(
                Block::default()
                    .title(format!("Search Results ({})", results.len()))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green)),
            )
            .highlight_style(Style::default().bg(Color::DarkGray));
        
        let mut list_state = ListState::default();
        list_state.select(Some(self.search.selected_index()));
        frame.render_stateful_widget(list, area, &mut list_state);
    }
    
    /// Render footer with shortcuts
    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let footer_area = Rect {
            x: area.x,
            y: area.y + area.height - 1,
            width: area.width,
            height: 1,
        };
        
        let shortcuts = match self.state {
            HelpDialogState::Browse => {
                "ESC: Close | Ctrl+F: Search | ↑↓←→: Navigate | PgUp/PgDn: Scroll"
            }
            HelpDialogState::Search => "ESC: Back to Browse | Enter: Search | ↑↓: Navigate Results",
        };
        
        let footer = Paragraph::new(shortcuts)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        
        frame.render_widget(footer, footer_area);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_dialog_creation() {
        let content = HelpContent::default_ricecoder_help();
        let dialog = HelpDialog::new(content);
        
        assert!(!dialog.is_visible());
        assert_eq!(dialog.state, HelpDialogState::Browse);
        assert_eq!(dialog.selected_category, 0);
        assert_eq!(dialog.selected_item, 0);
    }

    #[test]
    fn test_help_dialog_visibility() {
        let mut dialog = HelpDialog::default_ricecoder();
        
        assert!(!dialog.is_visible());
        
        dialog.show();
        assert!(dialog.is_visible());
        
        dialog.hide();
        assert!(!dialog.is_visible());
        
        dialog.toggle();
        assert!(dialog.is_visible());
        
        dialog.toggle();
        assert!(!dialog.is_visible());
    }

    #[test]
    fn test_help_dialog_navigation() {
        let mut dialog = HelpDialog::default_ricecoder();
        dialog.show();
        
        // Test basic navigation
        assert_eq!(dialog.selected_category, 0);
        assert_eq!(dialog.selected_item, 0);
        
        dialog.navigate_down();
        // Should move to next item or category
        
        dialog.navigate_up();
        // Should move back
        
        dialog.navigate_right();
        // Should move to next category if available
        
        dialog.navigate_left();
        // Should move to previous category if available
    }

    #[test]
    fn test_help_dialog_search_mode() {
        let mut dialog = HelpDialog::default_ricecoder();
        dialog.show();
        
        assert_eq!(dialog.state, HelpDialogState::Browse);
        
        // Simulate Ctrl+F
        let key = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL);
        dialog.handle_key(key).unwrap();
        
        assert_eq!(dialog.state, HelpDialogState::Search);
        assert_eq!(dialog.search_input, "");
        
        // Simulate typing
        let key = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE);
        dialog.handle_key(key).unwrap();
        
        assert_eq!(dialog.search_input, "h");
        
        // Simulate escape
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        dialog.handle_key(key).unwrap();
        
        assert_eq!(dialog.state, HelpDialogState::Browse);
        assert_eq!(dialog.search_input, "");
    }
}