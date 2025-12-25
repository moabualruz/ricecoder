//! Autocomplete popup widget for RiceCoder TUI
//!
//! Renders the autocomplete suggestions popup:
//! - Positioned relative to cursor/anchor
//! - Category headers
//! - Highlighted matches
//! - Keyboard navigation indicators
//!
//! # DDD Layer: Presentation
//! Widget for rendering autocomplete suggestions.

use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, StatefulWidget, Widget},
};

/// Suggestion category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuggestionCategory {
    File,
    Agent,
    Command,
    History,
    Snippet,
}

impl SuggestionCategory {
    /// Get category display name
    pub fn name(&self) -> &'static str {
        match self {
            Self::File => "Files",
            Self::Agent => "Agents",
            Self::Command => "Commands",
            Self::History => "History",
            Self::Snippet => "Snippets",
        }
    }

    /// Get category color
    pub fn color(&self) -> Color {
        match self {
            Self::File => Color::Cyan,
            Self::Agent => Color::Magenta,
            Self::Command => Color::Yellow,
            Self::History => Color::Blue,
            Self::Snippet => Color::Green,
        }
    }

    /// Get category icon
    pub fn icon(&self) -> &'static str {
        match self {
            Self::File => "󰈔",
            Self::Agent => "󰚩",
            Self::Command => "󰘳",
            Self::History => "󰋚",
            Self::Snippet => "󰅪",
        }
    }
}

/// A single autocomplete suggestion
#[derive(Debug, Clone)]
pub struct Suggestion {
    /// Display text
    pub text: String,
    /// Secondary text (description)
    pub description: Option<String>,
    /// Category
    pub category: SuggestionCategory,
    /// Match score (for sorting)
    pub score: u32,
    /// Matched character indices (for highlighting)
    pub match_indices: Vec<usize>,
    /// Value to insert
    pub value: String,
}

impl Suggestion {
    pub fn new(text: impl Into<String>, category: SuggestionCategory) -> Self {
        let text = text.into();
        Self {
            value: text.clone(),
            text,
            description: None,
            category,
            score: 0,
            match_indices: vec![],
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    pub fn with_score(mut self, score: u32) -> Self {
        self.score = score;
        self
    }

    pub fn with_matches(mut self, indices: Vec<usize>) -> Self {
        self.match_indices = indices;
        self
    }
}

/// Autocomplete popup state
#[derive(Debug, Default)]
pub struct AutocompleteState {
    /// All suggestions
    pub suggestions: Vec<Suggestion>,
    /// Selected index
    pub selected: usize,
    /// Whether popup is visible
    pub visible: bool,
    /// Anchor position (where to render popup)
    pub anchor: Position,
    /// Current input for filtering
    pub input: String,
}

impl AutocompleteState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Show popup with suggestions
    pub fn show(&mut self, suggestions: Vec<Suggestion>, anchor: Position) {
        self.suggestions = suggestions;
        self.selected = 0;
        self.visible = true;
        self.anchor = anchor;
    }

    /// Hide popup
    pub fn hide(&mut self) {
        self.visible = false;
        self.suggestions.clear();
        self.selected = 0;
    }

    /// Move selection up
    pub fn select_prev(&mut self) {
        if !self.suggestions.is_empty() {
            self.selected = self
                .selected
                .checked_sub(1)
                .unwrap_or(self.suggestions.len() - 1);
        }
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        if !self.suggestions.is_empty() {
            self.selected = (self.selected + 1) % self.suggestions.len();
        }
    }

    /// Get selected suggestion
    pub fn selected_suggestion(&self) -> Option<&Suggestion> {
        self.suggestions.get(self.selected)
    }

    /// Update input and filter
    pub fn set_input(&mut self, input: impl Into<String>) {
        self.input = input.into();
    }
}

/// Autocomplete popup widget configuration
#[derive(Debug, Clone)]
pub struct AutocompleteWidgetConfig {
    /// Maximum height in lines
    pub max_height: u16,
    /// Maximum width
    pub max_width: u16,
    /// Background color
    pub bg_color: Color,
    /// Border color
    pub border_color: Color,
    /// Selected background
    pub selected_bg: Color,
    /// Text color
    pub text_color: Color,
    /// Muted text color
    pub text_muted: Color,
    /// Match highlight color
    pub match_color: Color,
}

impl Default for AutocompleteWidgetConfig {
    fn default() -> Self {
        Self {
            max_height: 10,
            max_width: 60,
            bg_color: Color::Rgb(30, 30, 46),
            border_color: Color::Rgb(88, 91, 112),
            selected_bg: Color::Rgb(49, 50, 68),
            text_color: Color::White,
            text_muted: Color::Gray,
            match_color: Color::Yellow,
        }
    }
}

/// Autocomplete popup widget
pub struct AutocompleteWidget {
    config: AutocompleteWidgetConfig,
}

impl AutocompleteWidget {
    pub fn new() -> Self {
        Self {
            config: AutocompleteWidgetConfig::default(),
        }
    }

    pub fn with_config(config: AutocompleteWidgetConfig) -> Self {
        Self { config }
    }

    /// Render a suggestion with match highlighting
    fn render_suggestion<'a>(&self, suggestion: &'a Suggestion, selected: bool) -> ListItem<'a> {
        let mut spans = Vec::new();

        // Category icon
        spans.push(Span::styled(
            format!("{} ", suggestion.category.icon()),
            Style::default().fg(suggestion.category.color()),
        ));

        // Text with match highlighting
        let text = &suggestion.text;
        let mut last_idx = 0;
        for &idx in &suggestion.match_indices {
            if idx > last_idx && idx < text.len() {
                // Non-matched portion
                spans.push(Span::styled(
                    &text[last_idx..idx],
                    Style::default().fg(self.config.text_color),
                ));
            }
            if idx < text.len() {
                // Matched character
                let end = (idx + 1).min(text.len());
                spans.push(Span::styled(
                    &text[idx..end],
                    Style::default()
                        .fg(self.config.match_color)
                        .add_modifier(Modifier::BOLD),
                ));
                last_idx = end;
            }
        }
        // Remaining text
        if last_idx < text.len() {
            spans.push(Span::styled(
                &text[last_idx..],
                Style::default().fg(self.config.text_color),
            ));
        }

        // Description
        if let Some(desc) = &suggestion.description {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                desc.clone(),
                Style::default().fg(self.config.text_muted),
            ));
        }

        let style = if selected {
            Style::default().bg(self.config.selected_bg)
        } else {
            Style::default()
        };

        ListItem::new(Line::from(spans)).style(style)
    }

    /// Calculate popup position
    fn calculate_rect(&self, state: &AutocompleteState, area: Rect) -> Rect {
        let height = (state.suggestions.len() as u16 + 2).min(self.config.max_height);
        let width = self.config.max_width.min(area.width.saturating_sub(2));

        // Position below anchor, or above if not enough space
        let x = state.anchor.x.min(area.width.saturating_sub(width));
        let y = if state.anchor.y + height + 1 < area.height {
            state.anchor.y + 1
        } else {
            state.anchor.y.saturating_sub(height)
        };

        Rect::new(x, y, width, height)
    }
}

impl Default for AutocompleteWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl StatefulWidget for AutocompleteWidget {
    type State = AutocompleteState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if !state.visible || state.suggestions.is_empty() {
            return;
        }

        let popup_rect = self.calculate_rect(state, area);

        // Clear background
        Clear.render(popup_rect, buf);

        // Render border
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.config.border_color))
            .style(Style::default().bg(self.config.bg_color));

        let inner = block.inner(popup_rect);
        block.render(popup_rect, buf);

        // Render suggestions
        let items: Vec<ListItem> = state
            .suggestions
            .iter()
            .enumerate()
            .map(|(i, s)| self.render_suggestion(s, i == state.selected))
            .collect();

        let list = List::new(items);

        // Render with scroll offset
        let mut list_state = ListState::default();
        list_state.select(Some(state.selected));

        StatefulWidget::render(list, inner, buf, &mut list_state);
    }
}

/// Helper to render hint text below popup
pub fn render_autocomplete_hints(area: Rect, buf: &mut Buffer, visible: bool) {
    if !visible {
        return;
    }

    let hints = Line::from(vec![
        Span::styled("↑↓", Style::default().fg(Color::White)),
        Span::styled(" navigate  ", Style::default().fg(Color::Gray)),
        Span::styled("Tab", Style::default().fg(Color::White)),
        Span::styled(" select  ", Style::default().fg(Color::Gray)),
        Span::styled("Esc", Style::default().fg(Color::White)),
        Span::styled(" close", Style::default().fg(Color::Gray)),
    ]);

    Paragraph::new(hints).render(area, buf);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suggestion_category() {
        assert_eq!(SuggestionCategory::File.name(), "Files");
        assert_eq!(SuggestionCategory::Agent.color(), Color::Magenta);
    }

    #[test]
    fn test_suggestion_new() {
        let s = Suggestion::new("test.rs", SuggestionCategory::File)
            .with_description("A test file")
            .with_score(100);

        assert_eq!(s.text, "test.rs");
        assert_eq!(s.description, Some("A test file".to_string()));
        assert_eq!(s.score, 100);
    }

    #[test]
    fn test_state_navigation() {
        let mut state = AutocompleteState::new();
        state.suggestions = vec![
            Suggestion::new("a", SuggestionCategory::File),
            Suggestion::new("b", SuggestionCategory::File),
            Suggestion::new("c", SuggestionCategory::File),
        ];

        assert_eq!(state.selected, 0);
        state.select_next();
        assert_eq!(state.selected, 1);
        state.select_next();
        assert_eq!(state.selected, 2);
        state.select_next();
        assert_eq!(state.selected, 0); // wraps

        state.select_prev();
        assert_eq!(state.selected, 2); // wraps back
    }

    #[test]
    fn test_show_hide() {
        let mut state = AutocompleteState::new();
        assert!(!state.visible);

        state.show(
            vec![Suggestion::new("test", SuggestionCategory::Command)],
            Position::new(0, 0),
        );
        assert!(state.visible);
        assert_eq!(state.suggestions.len(), 1);

        state.hide();
        assert!(!state.visible);
        assert!(state.suggestions.is_empty());
    }
}
