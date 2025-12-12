//! LSP Diagnostics display widget for TUI
//!
//! This module provides widgets for displaying LSP diagnostics (errors, warnings, hints)
//! in the terminal user interface with proper formatting and navigation.

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Widget, Wrap},
};
use std::collections::HashMap;

/// Severity levels for diagnostics
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiagnosticSeverity {
    Error = 1,
    Warning = 2,
    Information = 3,
    Hint = 4,
}

impl DiagnosticSeverity {
    /// Get the display color for this severity
    pub fn color(&self) -> Color {
        match self {
            DiagnosticSeverity::Error => Color::Red,
            DiagnosticSeverity::Warning => Color::Yellow,
            DiagnosticSeverity::Information => Color::Blue,
            DiagnosticSeverity::Hint => Color::Green,
        }
    }

    /// Get the display symbol for this severity
    pub fn symbol(&self) -> &'static str {
        match self {
            DiagnosticSeverity::Error => "✗",
            DiagnosticSeverity::Warning => "⚠",
            DiagnosticSeverity::Information => "ℹ",
            DiagnosticSeverity::Hint => "💡",
        }
    }
}

/// A single diagnostic item
#[derive(Debug, Clone)]
pub struct DiagnosticItem {
    /// Unique identifier for the diagnostic
    pub id: String,
    /// Severity level
    pub severity: DiagnosticSeverity,
    /// Human-readable message
    pub message: String,
    /// Source of the diagnostic (e.g., "rustc", "eslint")
    pub source: Option<String>,
    /// Code identifier (e.g., "E0425", "no-unused-vars")
    pub code: Option<String>,
    /// File path where the diagnostic occurs
    pub file_path: String,
    /// Line number (0-based)
    pub line: usize,
    /// Column number (0-based)
    pub column: usize,
    /// End line (optional, for multi-line diagnostics)
    pub end_line: Option<usize>,
    /// End column (optional)
    pub end_column: Option<usize>,
    /// Related information (additional context)
    pub related_information: Vec<DiagnosticRelatedInformation>,
}

impl DiagnosticItem {
    /// Create a new diagnostic item
    pub fn new(
        severity: DiagnosticSeverity,
        message: impl Into<String>,
        file_path: impl Into<String>,
        line: usize,
        column: usize,
    ) -> Self {
        let file_path = file_path.into();
        Self {
            id: format!("{}:{}:{}", file_path, line, column),
            severity,
            message: message.into(),
            source: None,
            code: None,
            file_path,
            line,
            column,
            end_line: None,
            end_column: None,
            related_information: Vec::new(),
        }
    }

    /// Set the source
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Set the code
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Set the end position
    pub fn with_end_position(mut self, end_line: usize, end_column: usize) -> Self {
        self.end_line = Some(end_line);
        self.end_column = Some(end_column);
        self
    }

    /// Add related information
    pub fn with_related_info(mut self, info: DiagnosticRelatedInformation) -> Self {
        self.related_information.push(info);
        self
    }
}

/// Related information for a diagnostic
#[derive(Debug, Clone)]
pub struct DiagnosticRelatedInformation {
    /// Location of the related information
    pub location: DiagnosticLocation,
    /// Message describing the relationship
    pub message: String,
}

/// Location information
#[derive(Debug, Clone)]
pub struct DiagnosticLocation {
    pub file_path: String,
    pub line: usize,
    pub column: usize,
}

/// Widget for displaying diagnostics in a list
pub struct DiagnosticsWidget<'a> {
    diagnostics: &'a [DiagnosticItem],
    selected_index: Option<usize>,
    show_file_paths: bool,
    max_height: Option<usize>,
}

impl<'a> DiagnosticsWidget<'a> {
    /// Create a new diagnostics widget
    pub fn new(diagnostics: &'a [DiagnosticItem]) -> Self {
        Self {
            diagnostics,
            selected_index: None,
            show_file_paths: true,
            max_height: None,
        }
    }

    /// Set the selected diagnostic index
    pub fn select(mut self, index: usize) -> Self {
        self.selected_index = Some(index);
        self
    }

    /// Show or hide file paths in the display
    pub fn show_file_paths(mut self, show: bool) -> Self {
        self.show_file_paths = show;
        self
    }

    /// Set maximum height for the widget
    pub fn max_height(mut self, height: usize) -> Self {
        self.max_height = Some(height);
        self
    }

    /// Get the number of diagnostics
    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    /// Check if there are no diagnostics
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    /// Get diagnostics grouped by file
    pub fn diagnostics_by_file(&self) -> HashMap<String, Vec<&DiagnosticItem>> {
        let mut grouped = HashMap::new();
        for diagnostic in self.diagnostics {
            grouped.entry(diagnostic.file_path.clone())
                  .or_insert_with(Vec::new)
                  .push(diagnostic);
        }
        grouped
    }

    /// Get diagnostics by severity
    pub fn diagnostics_by_severity(&self, severity: DiagnosticSeverity) -> Vec<&DiagnosticItem> {
        self.diagnostics.iter()
            .filter(|d| d.severity == severity)
            .collect()
    }
}

impl<'a> Widget for DiagnosticsWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.diagnostics.is_empty() {
            let block = Block::default()
                .title("Diagnostics")
                .borders(Borders::ALL);
            let inner_area = block.inner(area);
            block.render(area, buf);

            let no_diagnostics = Paragraph::new("No diagnostics found")
                .style(Style::default().fg(Color::Gray))
                .alignment(ratatui::layout::Alignment::Center);
            no_diagnostics.render(inner_area, buf);
            return;
        }

        let block = Block::default()
            .title(format!("Diagnostics ({})", self.diagnostics.len()))
            .borders(Borders::ALL);
        let inner_area = block.inner(area);
        block.render(area, buf);

        // Create list items for diagnostics
        let items: Vec<ListItem> = self.diagnostics.iter().enumerate().map(|(i, diagnostic)| {
            let mut spans = Vec::new();

            // Severity symbol
            spans.push(Span::styled(
                diagnostic.severity.symbol(),
                Style::default().fg(diagnostic.severity.color())
            ));
            spans.push(Span::raw(" "));

            // Message
            spans.push(Span::raw(&diagnostic.message));

            // Code if available
            if let Some(ref code) = diagnostic.code {
                spans.push(Span::raw(" "));
                spans.push(Span::styled(
                    format!("[{}]", code),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::ITALIC)
                ));
            }

            // Location
            let location = if self.show_file_paths {
                format!(" ({}:{}:{})", diagnostic.file_path, diagnostic.line + 1, diagnostic.column + 1)
            } else {
                format!(" ({}:{})", diagnostic.line + 1, diagnostic.column + 1)
            };
            spans.push(Span::styled(
                location,
                Style::default().fg(Color::Gray)
            ));

            let style = if Some(i) == self.selected_index {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(spans)).style(style)
        }).collect();

        let list = List::new(items)
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::White));

        list.render(inner_area, buf);
    }
}

/// Widget for displaying detailed diagnostic information
pub struct DiagnosticDetailWidget<'a> {
    diagnostic: Option<&'a DiagnosticItem>,
}

impl<'a> DiagnosticDetailWidget<'a> {
    /// Create a new diagnostic detail widget
    pub fn new(diagnostic: Option<&'a DiagnosticItem>) -> Self {
        Self { diagnostic }
    }
}

impl<'a> Widget for DiagnosticDetailWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title("Diagnostic Details")
            .borders(Borders::ALL);
        let inner_area = block.inner(area);
        block.render(area, buf);

        if let Some(diagnostic) = self.diagnostic {
            let mut lines = Vec::new();

            // Severity and message
            lines.push(Line::from(vec![
                Span::styled(diagnostic.severity.symbol(), Style::default().fg(diagnostic.severity.color())),
                Span::raw(" "),
                Span::styled(&diagnostic.message, Style::default().fg(Color::White)),
            ]));

            // Source and code
            if diagnostic.source.is_some() || diagnostic.code.is_some() {
                let mut source_spans = Vec::new();
                source_spans.push(Span::styled("Source: ", Style::default().fg(Color::Cyan)));

                if let Some(ref source) = diagnostic.source {
                    source_spans.push(Span::raw(source));
                }

                if let Some(ref code) = diagnostic.code {
                    if diagnostic.source.is_some() {
                        source_spans.push(Span::raw(" "));
                    }
                    source_spans.push(Span::styled(format!("({})", code), Style::default().fg(Color::Yellow)));
                }

                lines.push(Line::from(source_spans));
            }

            // Location
            lines.push(Line::from(vec![
                Span::styled("Location: ", Style::default().fg(Color::Cyan)),
                Span::raw(&diagnostic.file_path),
                Span::raw(format!(":{}:{}", diagnostic.line + 1, diagnostic.column + 1)),
            ]));

            // Range if multi-line
            if let (Some(end_line), Some(end_column)) = (diagnostic.end_line, diagnostic.end_column) {
                if end_line != diagnostic.line || end_column != diagnostic.column {
                    lines.push(Line::from(vec![
                        Span::styled("Range: ", Style::default().fg(Color::Cyan)),
                        Span::raw(format!("{}:{}-{}:{}", diagnostic.line + 1, diagnostic.column + 1, end_line + 1, end_column + 1)),
                    ]));
                }
            }

            // Related information
            if !diagnostic.related_information.is_empty() {
                lines.push(Line::from(Span::styled("Related:", Style::default().fg(Color::Cyan))));
                for related in &diagnostic.related_information {
                    lines.push(Line::from(vec![
                        Span::raw("  • "),
                        Span::raw(&related.location.file_path),
                        Span::raw(format!(":{}:{}", related.location.line + 1, related.location.column + 1)),
                        Span::raw(" - "),
                        Span::raw(&related.message),
                    ]));
                }
            }

            let paragraph = Paragraph::new(lines)
                .wrap(Wrap { trim: true });

            paragraph.render(inner_area, buf);
        } else {
            let no_selection = Paragraph::new("No diagnostic selected")
                .style(Style::default().fg(Color::Gray))
                .alignment(ratatui::layout::Alignment::Center);
            no_selection.render(inner_area, buf);
        }
    }
}

/// Hover information widget for displaying LSP hover responses
pub struct HoverWidget<'a> {
    content: Option<&'a str>,
    language: Option<&'a str>,
}

impl<'a> HoverWidget<'a> {
    /// Create a new hover widget
    pub fn new() -> Self {
        Self {
            content: None,
            language: None,
        }
    }

    /// Set the hover content
    pub fn content(mut self, content: &'a str) -> Self {
        self.content = Some(content);
        self
    }

    /// Set the language for syntax highlighting hint
    pub fn language(mut self, language: &'a str) -> Self {
        self.language = Some(language);
        self
    }
}

impl<'a> Widget for HoverWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title("Hover Information")
            .borders(Borders::ALL);
        let inner_area = block.inner(area);
        block.render(area, buf);

        if let Some(content) = self.content {
            let paragraph = Paragraph::new(content)
                .wrap(Wrap { trim: true })
                .style(Style::default().fg(Color::White));

            paragraph.render(inner_area, buf);
        } else {
            let no_hover = Paragraph::new("No hover information available")
                .style(Style::default().fg(Color::Gray))
                .alignment(ratatui::layout::Alignment::Center);
            no_hover.render(inner_area, buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_item_creation() {
        let diagnostic = DiagnosticItem::new(
            DiagnosticSeverity::Error,
            "Undefined variable 'x'",
            "main.rs",
            10,
            5,
        )
        .with_source("rustc")
        .with_code("E0425");

        assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
        assert_eq!(diagnostic.message, "Undefined variable 'x'");
        assert_eq!(diagnostic.file_path, "main.rs");
        assert_eq!(diagnostic.line, 10);
        assert_eq!(diagnostic.column, 5);
        assert_eq!(diagnostic.source, Some("rustc".to_string()));
        assert_eq!(diagnostic.code, Some("E0425".to_string()));
    }

    #[test]
    fn test_diagnostic_severity_ordering() {
        assert!(DiagnosticSeverity::Error < DiagnosticSeverity::Warning);
        assert!(DiagnosticSeverity::Warning < DiagnosticSeverity::Information);
        assert!(DiagnosticSeverity::Information < DiagnosticSeverity::Hint);
    }

    #[test]
    fn test_diagnostics_widget_empty() {
        let diagnostics = Vec::new();
        let widget = DiagnosticsWidget::new(&diagnostics);
        assert_eq!(widget.len(), 0);
        assert!(widget.is_empty());
    }

    #[test]
    fn test_diagnostics_widget_with_items() {
        let diagnostics = vec![
            DiagnosticItem::new(DiagnosticSeverity::Error, "Error message", "file.rs", 1, 0),
            DiagnosticItem::new(DiagnosticSeverity::Warning, "Warning message", "file.rs", 2, 0),
        ];
        let widget = DiagnosticsWidget::new(&diagnostics);
        assert_eq!(widget.len(), 2);
        assert!(!widget.is_empty());
    }
}