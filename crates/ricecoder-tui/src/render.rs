//! Rendering logic for the TUI

use ratatui::prelude::*;

use crate::{
    app::App,
    diff::{DiffLineType, DiffViewType, DiffWidget},
    markdown::{MarkdownElement, MarkdownParser},
    style::Theme,
    widgets::{Message, MessageAuthor, ToolStatus},
};

/// Renderer for the TUI
pub struct Renderer;

impl Renderer {
    /// Create a new renderer
    pub fn new() -> Self {
        Self
    }

    /// Render the application
    ///
    /// This method renders the entire TUI application using ratatui.
    /// It sets up the layout and renders all widgets based on the current app state.
    ///
    /// Note: This is a stateless render function that works with the terminal
    /// managed by the main event loop. The terminal is initialized once in main.rs
    /// and reused for all render calls.
    ///
    /// Requirements: 1.2 - Render the TUI interface
    pub fn render(&self, app: &App) -> anyhow::Result<()> {
        // This is a placeholder that demonstrates the rendering structure.
        // In a real implementation, this would be called from within a terminal.draw() closure.
        // The actual rendering happens in the main event loop in main.rs.

        tracing::debug!(
            "Rendering TUI - Mode: {}, Messages: {}, Input: {}",
            app.reactive_state
                .blocking_read()
                .current()
                .mode
                .display_name(),
            app.chat.messages.len(),
            app.chat.input
        );

        Ok(())
    }

    /// Render the UI frame using ratatui's Frame
    ///
    /// This method is called from within the terminal.draw() closure in main.rs.
    /// It handles all the actual rendering of widgets.
    ///
    /// Requirements: 1.2 - Render the TUI interface
    pub fn render_frame(f: &mut ratatui::Frame, app: &mut App) {
        // Get the available area
        let area = f.size();

        // Create the main layout
        let chunks = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Mode indicator
                Constraint::Min(5),    // Chat area
                Constraint::Length(3), // Input area
                Constraint::Length(1), // Status bar
            ])
            .split(area);

        // Render mode indicator
        let mode_text = format!(
            "Mode: {}",
            app.reactive_state
                .blocking_read()
                .current()
                .mode
                .display_name()
        );
        let mode_block = ratatui::widgets::Block::default()
            .title("RiceCoder")
            .borders(ratatui::widgets::Borders::BOTTOM);
        let mode_paragraph = ratatui::widgets::Paragraph::new(mode_text)
            .block(mode_block)
            .style(Style::default().fg(Color::Cyan));
        f.render_widget(mode_paragraph, chunks[0]);

        // Render chat area
        let chat_block = ratatui::widgets::Block::default()
            .title("Chat")
            .borders(ratatui::widgets::Borders::ALL);

        let messages = &app.chat.messages;
        let mut text_lines = Vec::new();

        if messages.is_empty() {
            text_lines.push(Line::from(
                "Welcome to RiceCoder TUI! Type your message below.",
            ));
        } else {
            for (i, msg) in messages.iter().enumerate() {
                // Simple message rendering - just show the message text
                let author_str = if i % 2 == 0 { "User" } else { "RiceCoder" };
                let author_style = if i % 2 == 0 {
                    Style::default()
                        .fg(app.reactive_state.blocking_read().current().theme.primary)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                        .fg(app.reactive_state.blocking_read().current().theme.secondary)
                        .add_modifier(Modifier::BOLD)
                };

                text_lines.push(Line::from(Span::styled(
                    format!("[{}] {}", "12:34", author_str), // TODO: Add timestamps
                    author_style,
                )));

                // Render message content as simple text
                let content_style = if i % 2 == 0 {
                    Style::default().fg(app.reactive_state.blocking_read().current().theme.primary)
                } else {
                    Style::default().fg(app
                        .reactive_state
                        .blocking_read()
                        .current()
                        .theme
                        .secondary)
                };

                // Split message into lines and render
                for line in msg.content.lines() {
                    text_lines.push(Line::from(Span::styled(line.to_string(), content_style)));
                }

                // Add spacing between messages
                text_lines.push(Line::from(""));
            }
        }

        let chat_paragraph = ratatui::widgets::Paragraph::new(text_lines)
            .block(chat_block)
            .wrap(ratatui::widgets::Wrap { trim: true })
            .scroll((0, 0)); // TODO: Add scroll support

        f.render_widget(chat_paragraph, chunks[1]);

        // Render input area
        let input_block = ratatui::widgets::Block::default()
            .title("Input")
            .borders(ratatui::widgets::Borders::ALL);
        let input_text = format!("> {}", app.chat.input);
        let input_paragraph = ratatui::widgets::Paragraph::new(input_text)
            .block(input_block)
            .style(Style::default().fg(Color::Green));
        f.render_widget(input_paragraph, chunks[2]);

        // TODO: Render help dialog if visible - needs integration
        // app.help_dialog.render(f, area);

        // TODO: Render file picker if visible - needs integration
        // app.file_picker.render(f, area);

        // TODO: Render status bar with token usage - needs session integration
        let token_usage = Default::default(); // Placeholder
        let status_bar = crate::status_bar::StatusBarWidget::new()
            .with_provider("RiceCoder") // TODO: Get from provider integration
            .with_model("gpt-4") // TODO: Get from provider integration
            .with_connection_status(crate::status_bar::ConnectionStatus::Connected) // TODO: Get from provider integration
            .with_session_name("default") // TODO: Get current session name
            .with_message_count(app.chat.messages.len())
            .with_token_usage(token_usage)
            .with_input_mode(crate::status_bar::InputMode::Insert);
        f.render_widget(status_bar, chunks[3]);
    }

    /// Helper to render markdown content
    fn render_markdown(
        &self,
        content: &str,
        theme: &Theme,
        text_lines: &mut Vec<Line>,
        base_style: Style,
    ) {
        let elements = MarkdownParser::parse(content);
        let mut current_line_spans = Vec::new();

        for element in elements {
            if element.is_block() {
                if !current_line_spans.is_empty() {
                    text_lines.push(Line::from(std::mem::take(&mut current_line_spans)));
                }

                match element {
                    MarkdownElement::Header(level, text) => {
                        let style = Style::default()
                            .fg(theme.accent.to_ratatui())
                            .add_modifier(Modifier::BOLD);
                        let prefix = "#".repeat(level as usize);
                        text_lines.push(Line::from(Span::styled(
                            format!("{} {}", prefix, text),
                            style,
                        )));
                    }
                    MarkdownElement::CodeBlock(lang, code) => {
                        let highlighted = MarkdownParser::highlight(&code, lang.as_deref());
                        text_lines.extend(highlighted);
                    }
                    MarkdownElement::ListItem(text) => {
                        text_lines.push(Line::from(vec![
                            Span::styled("• ", Style::default().fg(theme.secondary.to_ratatui())),
                            Span::raw(text),
                        ]));
                    }
                    _ => {}
                }
            } else {
                match element {
                    MarkdownElement::Text(text) => {
                        current_line_spans.push(Span::styled(text, base_style));
                    }
                    MarkdownElement::Bold(text) => {
                        current_line_spans
                            .push(Span::styled(text, base_style.add_modifier(Modifier::BOLD)));
                    }
                    MarkdownElement::Italic(text) => {
                        current_line_spans.push(Span::styled(
                            text,
                            base_style.add_modifier(Modifier::ITALIC),
                        ));
                    }
                    MarkdownElement::Code(text) => {
                        current_line_spans.push(Span::styled(
                            text,
                            Style::default().bg(Color::DarkGray).fg(Color::White),
                        ));
                    }
                    MarkdownElement::Link(text, _url) => {
                        current_line_spans.push(Span::styled(
                            text,
                            Style::default()
                                .fg(Color::Blue)
                                .add_modifier(Modifier::UNDERLINED),
                        ));
                    }
                    MarkdownElement::Newline => {
                        text_lines.push(Line::from(std::mem::take(&mut current_line_spans)));
                    }
                    _ => {}
                }
            }
        }

        if !current_line_spans.is_empty() {
            text_lines.push(Line::from(current_line_spans));
        }
    }

    /// Render a diff widget in unified view
    pub fn render_diff_unified(
        &self,
        diff: &DiffWidget,
        _area: Rect,
        _theme: &Theme,
    ) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        // Add header showing view type and stats
        let total_lines: usize = diff.hunks.iter().map(|h| h.lines.len()).sum();
        let added_count = diff
            .hunks
            .iter()
            .flat_map(|h| &h.lines)
            .filter(|l| l.line_type == DiffLineType::Added)
            .count();
        let removed_count = diff
            .hunks
            .iter()
            .flat_map(|h| &h.lines)
            .filter(|l| l.line_type == DiffLineType::Removed)
            .count();

        let header = format!(
            "Unified Diff View | {} lines | +{} -{} | Approved: {}",
            total_lines,
            added_count,
            removed_count,
            diff.approved_hunks().len()
        );
        lines.push(Line::from(header));
        lines.push(Line::from(""));

        // Render each hunk
        for (hunk_idx, hunk) in diff.hunks.iter().enumerate() {
            let is_selected = diff.selected_hunk == Some(hunk_idx);
            let is_approved = diff.approvals.get(hunk_idx).copied().unwrap_or(false);

            // Hunk header
            let header_style = if is_selected {
                Style::default().fg(Color::Cyan).bold()
            } else if is_approved {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Yellow)
            };

            let approval_indicator = if is_approved { "✓" } else { " " };
            let collapse_indicator = if hunk.collapsed { "▶" } else { "▼" };

            let hunk_header = format!(
                "{} {} {} {}",
                approval_indicator, collapse_indicator, hunk.header, ""
            );
            lines.push(Line::from(Span::styled(hunk_header, header_style)));

            // Render lines if not collapsed
            if !hunk.collapsed {
                for line in &hunk.lines {
                    let (prefix, style) = match line.line_type {
                        DiffLineType::Added => ("+", Style::default().fg(Color::Green)),
                        DiffLineType::Removed => ("-", Style::default().fg(Color::Red)),
                        DiffLineType::Context => (" ", Style::default()),
                        DiffLineType::Unchanged => (" ", Style::default()),
                    };

                    let line_num_str = match (line.old_line_num, line.new_line_num) {
                        (Some(old), Some(new)) => format!("{:4} {:4}", old, new),
                        (Some(old), None) => format!("{:4}     ", old),
                        (None, Some(new)) => format!("     {:4}", new),
                        (None, None) => "          ".to_string(),
                    };

                    let content = format!("{} {} {}", prefix, line_num_str, line.content);
                    lines.push(Line::from(Span::styled(content, style)));
                }
            }

            lines.push(Line::from(""));
        }

        lines
    }

    /// Render a diff widget in side-by-side view
    pub fn render_diff_side_by_side(
        &self,
        diff: &DiffWidget,
        area: Rect,
        _theme: &Theme,
    ) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        // Add header
        let total_lines: usize = diff.hunks.iter().map(|h| h.lines.len()).sum();
        let added_count = diff
            .hunks
            .iter()
            .flat_map(|h| &h.lines)
            .filter(|l| l.line_type == DiffLineType::Added)
            .count();
        let removed_count = diff
            .hunks
            .iter()
            .flat_map(|h| &h.lines)
            .filter(|l| l.line_type == DiffLineType::Removed)
            .count();

        let header = format!(
            "Side-by-Side Diff View | {} lines | +{} -{} | Approved: {}",
            total_lines,
            added_count,
            removed_count,
            diff.approved_hunks().len()
        );
        lines.push(Line::from(header));
        lines.push(Line::from(""));

        // Column headers
        let col_width = (area.width as usize).saturating_sub(20) / 2;
        let header_left = format!("Original ({:width$})", "", width = col_width);
        let header_right = format!("Modified ({:width$})", "", width = col_width);
        lines.push(Line::from(format!("{} | {}", header_left, header_right)));
        lines.push(Line::from("─".repeat(area.width as usize)));

        // Render each hunk
        for (hunk_idx, hunk) in diff.hunks.iter().enumerate() {
            let is_selected = diff.selected_hunk == Some(hunk_idx);
            let is_approved = diff.approvals.get(hunk_idx).copied().unwrap_or(false);

            // Hunk header
            let header_style = if is_selected {
                Style::default().fg(Color::Cyan).bold()
            } else if is_approved {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Yellow)
            };

            let approval_indicator = if is_approved { "✓" } else { " " };
            let collapse_indicator = if hunk.collapsed { "▶" } else { "▼" };

            let hunk_header = format!(
                "{} {} {}",
                approval_indicator, collapse_indicator, hunk.header
            );
            lines.push(Line::from(Span::styled(hunk_header, header_style)));

            // Render lines if not collapsed
            if !hunk.collapsed {
                for line in &hunk.lines {
                    let (prefix, style) = match line.line_type {
                        DiffLineType::Added => ("+", Style::default().fg(Color::Green)),
                        DiffLineType::Removed => ("-", Style::default().fg(Color::Red)),
                        DiffLineType::Context => (" ", Style::default()),
                        DiffLineType::Unchanged => (" ", Style::default()),
                    };

                    let line_num = line.new_line_num.map(|n| n.to_string()).unwrap_or_default();
                    let content = format!("{} {:4} {}", prefix, line_num, line.content);

                    // For side-by-side, we'd need to track old vs new separately
                    // For now, show on the right side
                    let padded = format!("{:<width$} | {}", "", content, width = col_width);
                    lines.push(Line::from(Span::styled(padded, style)));
                }
            }

            lines.push(Line::from(""));
        }

        lines
    }

    /// Render diff widget based on view type
    pub fn render_diff(&self, diff: &DiffWidget, area: Rect, theme: &Theme) -> Vec<Line<'static>> {
        match diff.view_type {
            DiffViewType::Unified => self.render_diff_unified(diff, area, theme),
            DiffViewType::SideBySide => self.render_diff_side_by_side(diff, area, theme),
        }
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}
