//! Code diffing widget
//!
//! This module provides the `DiffWidget` for displaying and managing code changes.
//! It supports multiple diff formats (unified and side-by-side), syntax highlighting,
//! and hunk-level approval/rejection.
//!
//! # Features
//!
//! - **Multiple view formats**: Unified and side-by-side diff views
//! - **Syntax highlighting**: Language-aware code highlighting
//! - **Hunk navigation**: Jump between hunks with keyboard shortcuts
//! - **Approval workflow**: Accept or reject individual hunks
//! - **Line numbers**: Display original and new line numbers
//!
//! # Examples
//!
//! Creating a diff widget:
//!
//! ```ignore
//! use ricecoder_tui::{DiffWidget, DiffHunk, DiffLine, DiffLineType};
//!
//! let mut diff = DiffWidget::new();
//! let line = DiffLine {
//!     line_type: DiffLineType::Added,
//!     old_line_num: None,
//!     new_line_num: Some(1),
//!     content: "fn hello() {}".to_string(),
//! };
//! ```
//!
//! Navigating hunks:
//!
//! ```ignore
//! diff.next_hunk();  // Move to next hunk
//! diff.prev_hunk();  // Move to previous hunk
//! ```
//!
//! Approving changes:
//!
//! ```ignore
//! diff.approve_hunk(0);  // Approve first hunk
//! diff.reject_hunk(1);   // Reject second hunk
//! ```

/// Diff line type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffLineType {
    /// Unchanged line
    Unchanged,
    /// Added line
    Added,
    /// Removed line
    Removed,
    /// Context line
    Context,
}

/// Diff line
#[derive(Debug, Clone)]
pub struct DiffLine {
    /// Line type
    pub line_type: DiffLineType,
    /// Line number in original
    pub old_line_num: Option<usize>,
    /// Line number in new
    pub new_line_num: Option<usize>,
    /// Line content
    pub content: String,
}

impl DiffLine {
    /// Create a new diff line
    pub fn new(line_type: DiffLineType, content: impl Into<String>) -> Self {
        Self {
            line_type,
            old_line_num: None,
            new_line_num: None,
            content: content.into(),
        }
    }

    /// Set old line number
    pub fn with_old_line_num(mut self, num: usize) -> Self {
        self.old_line_num = Some(num);
        self
    }

    /// Set new line number
    pub fn with_new_line_num(mut self, num: usize) -> Self {
        self.new_line_num = Some(num);
        self
    }
}

/// Diff hunk
#[derive(Debug, Clone)]
pub struct DiffHunk {
    /// Hunk header
    pub header: String,
    /// Lines in hunk
    pub lines: Vec<DiffLine>,
    /// Whether hunk is collapsed
    pub collapsed: bool,
}

impl DiffHunk {
    /// Create a new diff hunk
    pub fn new(header: impl Into<String>) -> Self {
        Self {
            header: header.into(),
            lines: Vec::new(),
            collapsed: false,
        }
    }

    /// Add a line to the hunk
    pub fn add_line(&mut self, line: DiffLine) {
        self.lines.push(line);
    }

    /// Toggle collapsed state
    pub fn toggle_collapsed(&mut self) {
        self.collapsed = !self.collapsed;
    }

    /// Get visible lines
    pub fn visible_lines(&self) -> Vec<&DiffLine> {
        if self.collapsed {
            Vec::new()
        } else {
            self.lines.iter().collect()
        }
    }
}

/// Diff view type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffViewType {
    /// Unified diff view
    Unified,
    /// Side-by-side diff view
    SideBySide,
}

/// Diff widget
#[derive(Debug, Clone)]
pub struct DiffWidget {
    /// Hunks in the diff
    pub hunks: Vec<DiffHunk>,
    /// Current view type
    pub view_type: DiffViewType,
    /// Selected hunk index
    pub selected_hunk: Option<usize>,
    /// Scroll offset
    pub scroll: usize,
    /// Approval state for each hunk
    pub approvals: Vec<bool>,
    /// Component ID
    id: String,
    /// Bounds
    bounds: ratatui::layout::Rect,
    /// Focused state
    focused: bool,
    /// Visible state
    visible: bool,
    /// Z-index
    z_index: i32,
}

impl DiffWidget {
    /// Create a new diff widget
    pub fn new() -> Self {
        Self {
            hunks: Vec::new(),
            view_type: DiffViewType::Unified,
            selected_hunk: None,
            scroll: 0,
            approvals: Vec::new(),
            id: "diff_widget".to_string(),
            bounds: ratatui::layout::Rect::default(),
            focused: false,
            visible: true,
            z_index: 0,
        }
    }

    /// Add a hunk
    pub fn add_hunk(&mut self, hunk: DiffHunk) {
        self.hunks.push(hunk);
        self.approvals.push(false);
    }

    /// Toggle view type
    pub fn toggle_view_type(&mut self) {
        self.view_type = match self.view_type {
            DiffViewType::Unified => DiffViewType::SideBySide,
            DiffViewType::SideBySide => DiffViewType::Unified,
        };
    }

    /// Select next hunk
    pub fn select_next_hunk(&mut self) {
        if self.hunks.is_empty() {
            return;
        }
        match self.selected_hunk {
            None => self.selected_hunk = Some(0),
            Some(idx) if idx < self.hunks.len() - 1 => self.selected_hunk = Some(idx + 1),
            _ => {}
        }
    }

    /// Select previous hunk
    pub fn select_prev_hunk(&mut self) {
        match self.selected_hunk {
            None => {}
            Some(0) => self.selected_hunk = None,
            Some(idx) => self.selected_hunk = Some(idx - 1),
        }
    }

    /// Toggle selected hunk collapsed state
    pub fn toggle_selected_hunk(&mut self) {
        if let Some(idx) = self.selected_hunk {
            if let Some(hunk) = self.hunks.get_mut(idx) {
                hunk.toggle_collapsed();
            }
        }
    }

    /// Approve all changes
    pub fn approve_all(&mut self) {
        for approval in &mut self.approvals {
            *approval = true;
        }
    }

    /// Reject all changes
    pub fn reject_all(&mut self) {
        for approval in &mut self.approvals {
            *approval = false;
        }
    }

    /// Approve selected hunk
    pub fn approve_hunk(&mut self) {
        if let Some(idx) = self.selected_hunk {
            if let Some(approval) = self.approvals.get_mut(idx) {
                *approval = true;
            }
        }
    }

    /// Reject selected hunk
    pub fn reject_hunk(&mut self) {
        if let Some(idx) = self.selected_hunk {
            if let Some(approval) = self.approvals.get_mut(idx) {
                *approval = false;
            }
        }
    }

    /// Get all approved hunks
    pub fn approved_hunks(&self) -> Vec<usize> {
        self.approvals
            .iter()
            .enumerate()
            .filter_map(|(idx, &approved)| if approved { Some(idx) } else { None })
            .collect()
    }

    /// Get all rejected hunks
    pub fn rejected_hunks(&self) -> Vec<usize> {
        self.approvals
            .iter()
            .enumerate()
            .filter_map(|(idx, &approved)| if !approved { Some(idx) } else { None })
            .collect()
    }

    /// Scroll up
    pub fn scroll_up(&mut self) {
        if self.scroll > 0 {
            self.scroll -= 1;
        }
    }

    /// Scroll down
    pub fn scroll_down(&mut self, height: usize) {
        let total_lines: usize = self.hunks.iter().map(|h| h.visible_lines().len()).sum();
        let max_scroll = total_lines.saturating_sub(height);
        if self.scroll < max_scroll {
            self.scroll += 1;
        }
    }

    /// Render the diff widget
    pub fn render(&self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        use ratatui::widgets::{Block, Borders, Paragraph};
        use ratatui::style::{Color, Style, Modifier};
        use ratatui::text::{Line, Span};

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" Diff ({} hunks) ", self.hunks.len()))
            .border_style(Style::default().fg(if self.focused { Color::Cyan } else { Color::Gray }));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if self.hunks.is_empty() {
            let empty = Paragraph::new("No changes")
                .style(Style::default().fg(Color::Gray));
            frame.render_widget(empty, inner);
            return;
        }

        // Build diff lines
        let mut lines: Vec<Line> = Vec::new();
        
        for (hunk_idx, hunk) in self.hunks.iter().enumerate() {
            let is_selected = self.selected_hunk == Some(hunk_idx);
            let is_approved = self.approvals.get(hunk_idx).copied().unwrap_or(false);
            
            // Hunk header
            let header_style = Style::default()
                .fg(Color::Cyan)
                .add_modifier(if is_selected { Modifier::BOLD | Modifier::REVERSED } else { Modifier::BOLD });
            
            let status = if is_approved { "✓" } else { "○" };
            lines.push(Line::from(vec![
                Span::styled(format!("{} ", status), Style::default().fg(if is_approved { Color::Green } else { Color::Gray })),
                Span::styled(format!("@@ {} @@", hunk.header), header_style),
            ]));

            // Diff lines
            for line in &hunk.lines {
                let (prefix, style) = match line.line_type {
                    DiffLineType::Added => ("+", Style::default().fg(Color::Green)),
                    DiffLineType::Removed => ("-", Style::default().fg(Color::Red)),
                    DiffLineType::Context => (" ", Style::default().fg(Color::Gray)),
                    DiffLineType::Unchanged => (" ", Style::default().fg(Color::Gray)),
                };
                lines.push(Line::from(Span::styled(
                    format!("{}{}", prefix, line.content),
                    style,
                )));
            }
            lines.push(Line::from("")); // Spacing between hunks
        }

        // Apply scroll offset
        let visible_lines: Vec<Line> = lines
            .into_iter()
            .skip(self.scroll)
            .take(inner.height as usize)
            .collect();

        let paragraph = Paragraph::new(visible_lines);
        frame.render_widget(paragraph, inner);
    }
}

impl Default for DiffWidget {
    fn default() -> Self {
        Self::new()
    }
}

// Component trait implementation
use crate::components::traits::{Component, ComponentId};
use crate::components::{FocusDirection, FocusResult};
use crate::model::{AppModel, AppMessage};
use crossterm::event::KeyCode;

#[allow(deprecated)]
impl Component for DiffWidget {
    fn id(&self) -> ComponentId {
        self.id.clone()
    }

    fn render(&self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect, _model: &AppModel) {
        self.render(frame, area);
    }

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }

    fn bounds(&self) -> ratatui::layout::Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: ratatui::layout::Rect) {
        self.bounds = bounds;
    }

    fn is_focused(&self) -> bool {
        self.focused
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    fn z_index(&self) -> i32 {
        self.z_index
    }

    fn set_z_index(&mut self, z_index: i32) {
        self.z_index = z_index;
    }

    fn update(&mut self, message: &AppMessage, _model: &AppModel) -> bool {
        if let AppMessage::KeyPress(key) = message {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    self.select_prev_hunk();
                    return true;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.select_next_hunk();
                    return true;
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    self.approve_hunk();
                    return true;
                }
                KeyCode::Char('r') => {
                    self.reject_hunk();
                    return true;
                }
                KeyCode::Char('a') => {
                    self.approve_all();
                    return true;
                }
                KeyCode::Char('v') => {
                    self.toggle_view_type();
                    return true;
                }
                _ => {}
            }
        }
        false
    }
}
