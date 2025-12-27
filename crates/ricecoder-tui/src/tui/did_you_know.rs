//! Did You Know widget for displaying rotating tips
//!
//! This module provides a widget that displays rotating tips to help users
//! discover features. Tips support highlighting syntax and are displayed
//! in a bordered box.
//!
//! Tips are loaded from `config/tips.txt` via ricecoder-storage loaders.
//!
//! # Examples
//!
//! ```ignore
//! use ricecoder_tui::tui::did_you_know::DidYouKnow;
//! use ratatui::Frame;
//!
//! let mut widget = DidYouKnow::new();
//! widget.randomize(); // Show a random tip
//! frame.render_widget(widget, area);
//! ```

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};
use ricecoder_storage::TipsLoader;
use std::sync::OnceLock;

/// Cached tips loaded from config/tips.txt
static LOADED_TIPS: OnceLock<Vec<String>> = OnceLock::new();

/// Get tips, loading from config/tips.txt on first access
fn get_tips() -> &'static [String] {
    LOADED_TIPS.get_or_init(|| {
        let loader = TipsLoader::with_default_path();
        loader.load_all().unwrap_or_else(|_| default_tips())
    })
}

/// Default tips when config file is not available
fn default_tips() -> Vec<String> {
    vec![
        "Type {highlight}@{/highlight} followed by a filename to fuzzy search and attach files to your prompt.".to_string(),
        "Start a message with {highlight}!{/highlight} to run shell commands directly (e.g., {highlight}!ls -la{/highlight}).".to_string(),
        "Press {highlight}Tab{/highlight} to cycle between Build (full access) and Plan (read-only) agents.".to_string(),
        "Use {highlight}/undo{/highlight} to revert the last message and any file changes made by RiceCoder.".to_string(),
        "Use {highlight}/redo{/highlight} to restore previously undone messages and file changes.".to_string(),
        "Run {highlight}/share{/highlight} to create a public link to your conversation.".to_string(),
        "Drag and drop images into the terminal to add them as context for your prompts.".to_string(),
        "Press {highlight}Ctrl+V{/highlight} to paste images from your clipboard directly into the prompt.".to_string(),
        "Press {highlight}Ctrl+X E{/highlight} or {highlight}/editor{/highlight} to compose messages in your external editor.".to_string(),
        "Run {highlight}/init{/highlight} to auto-generate project rules based on your codebase structure.".to_string(),
        "Run {highlight}/models{/highlight} or {highlight}Ctrl+X M{/highlight} to see and switch between available AI models.".to_string(),
        "Use {highlight}/theme{/highlight} or {highlight}Ctrl+X T{/highlight} to preview and switch between 50+ built-in themes.".to_string(),
        "Press {highlight}Ctrl+X N{/highlight} or {highlight}/new{/highlight} to start a fresh conversation session.".to_string(),
        "Use {highlight}/sessions{/highlight} or {highlight}Ctrl+X L{/highlight} to list and continue previous conversations.".to_string(),
    ]
}

const BOX_WIDTH: u16 = 42;
const TITLE: &str = " 🅘 Did you know? ";

/// Part of a tip with optional highlighting
#[derive(Debug, Clone)]
struct TipPart {
    text: String,
    highlight: bool,
}

/// Parse a tip string into parts with highlighting
fn parse_tip(tip: &str) -> Vec<TipPart> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_highlight = false;

    let mut chars = tip.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '{' {
            // Check if this is a highlight tag
            let peek: String = chars.clone().take(11).collect();
            if peek.starts_with("highlight}") {
                // Save current text as non-highlighted
                if !current.is_empty() {
                    parts.push(TipPart {
                        text: current.clone(),
                        highlight: false,
                    });
                    current.clear();
                }
                // Skip "highlight}"
                for _ in 0..10 {
                    chars.next();
                }
                in_highlight = true;
                continue;
            } else if peek.starts_with("/highlight}") {
                // Save current text as highlighted
                if !current.is_empty() {
                    parts.push(TipPart {
                        text: current.clone(),
                        highlight: true,
                    });
                    current.clear();
                }
                // Skip "/highlight}"
                for _ in 0..11 {
                    chars.next();
                }
                in_highlight = false;
                continue;
            }
        }
        current.push(ch);
    }

    // Push remaining text
    if !current.is_empty() {
        parts.push(TipPart {
            text: current,
            highlight: in_highlight,
        });
    }

    parts
}

/// Did You Know widget
pub struct DidYouKnow {
    tip_index: usize,
    text_color: Color,
    muted_color: Color,
    border_color: Color,
}

impl DidYouKnow {
    /// Create a new Did You Know widget with random tip
    pub fn new() -> Self {
        let tips = get_tips();
        let tip_count = tips.len().max(1);
        // Simple random using time-based seed
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        let tip_index = (now.as_nanos() as usize) % tip_count;
        
        Self {
            tip_index,
            text_color: Color::White,
            muted_color: Color::DarkGray,
            border_color: Color::Gray,
        }
    }

    /// Randomize the current tip
    pub fn randomize(&mut self) {
        let tips = get_tips();
        if tips.is_empty() {
            return;
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        self.tip_index = (now.as_nanos() as usize) % tips.len();
    }

    /// Set the text color
    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }

    /// Set the muted color
    pub fn muted_color(mut self, color: Color) -> Self {
        self.muted_color = color;
        self
    }

    /// Set the border color
    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = color;
        self
    }

    /// Get the current tip as styled spans
    fn tip_spans(&self) -> Vec<Span> {
        let tips = get_tips();
        let tip = tips.get(self.tip_index).map(|s| s.as_str()).unwrap_or("");
        let parts = parse_tip(tip);

        parts
            .into_iter()
            .map(|part| {
                let color = if part.highlight {
                    self.text_color
                } else {
                    self.muted_color
                };
                Span::styled(part.text, Style::default().fg(color))
            })
            .collect()
    }

    /// Create the title line with borders
    fn title_line(&self) -> Line {
        let dashes = BOX_WIDTH.saturating_sub(TITLE.len() as u16 + 3);
        vec![
            Span::styled("╭─", Style::default().fg(self.border_color)),
            Span::styled(TITLE, Style::default().fg(self.text_color)),
            Span::styled(
                format!("{}╮", "─".repeat(dashes as usize)),
                Style::default().fg(self.border_color),
            ),
        ]
        .into()
    }
}

impl Default for DidYouKnow {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for DidYouKnow {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Create layout: title, content, footer
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(3),
                Constraint::Length(1),
            ])
            .split(area);

        // Render title
        Paragraph::new(self.title_line())
            .alignment(Alignment::Left)
            .render(chunks[0], buf);

        // Render content with border
        let content_block = Block::default()
            .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
            .border_style(Style::default().fg(self.border_color));

        let inner_area = content_block.inner(chunks[1]);
        content_block.render(chunks[1], buf);

        // Render tip text with padding
        let tip_area = Rect {
            x: inner_area.x + 1,
            y: inner_area.y + 1,
            width: inner_area.width.saturating_sub(2),
            height: inner_area.height.saturating_sub(1),
        };

        Paragraph::new(Line::from(self.tip_spans()))
            .wrap(ratatui::widgets::Wrap { trim: false })
            .render(tip_area, buf);

        // Footer with hide hint
        let footer = Line::from(vec![
            Span::styled("Ctrl+X T", Style::default().fg(self.text_color)),
            Span::styled(" hide tips", Style::default().fg(self.muted_color)),
        ]);

        Paragraph::new(footer)
            .alignment(Alignment::Right)
            .render(chunks[2], buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tip() {
        let tip = "Press {highlight}Tab{/highlight} to cycle";
        let parts = parse_tip(tip);
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0].text, "Press ");
        assert!(!parts[0].highlight);
        assert_eq!(parts[1].text, "Tab");
        assert!(parts[1].highlight);
        assert_eq!(parts[2].text, " to cycle");
        assert!(!parts[2].highlight);
    }

    #[test]
    fn test_did_you_know_creation() {
        let widget = DidYouKnow::new();
        assert!(widget.tip_index < super::get_tips().len());
    }

    #[test]
    fn test_did_you_know_randomize() {
        let mut widget = DidYouKnow::new();
        let first_index = widget.tip_index;
        widget.randomize();
        // Might be same by chance, but verifies it's valid
        assert!(widget.tip_index < super::get_tips().len());
    }
}
