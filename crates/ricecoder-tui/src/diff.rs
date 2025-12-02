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
#[derive(Debug)]
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
}

impl Default for DiffWidget {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_line_creation() {
        let line = DiffLine::new(DiffLineType::Added, "let x = 5;");
        assert_eq!(line.line_type, DiffLineType::Added);
        assert_eq!(line.content, "let x = 5;");
    }

    #[test]
    fn test_diff_line_numbers() {
        let line = DiffLine::new(DiffLineType::Unchanged, "code")
            .with_old_line_num(1)
            .with_new_line_num(1);

        assert_eq!(line.old_line_num, Some(1));
        assert_eq!(line.new_line_num, Some(1));
    }

    #[test]
    fn test_diff_hunk_creation() {
        let hunk = DiffHunk::new("@@ -1,5 +1,6 @@");
        assert_eq!(hunk.header, "@@ -1,5 +1,6 @@");
        assert!(!hunk.collapsed);
    }

    #[test]
    fn test_diff_hunk_add_line() {
        let mut hunk = DiffHunk::new("@@ -1,5 +1,6 @@");
        hunk.add_line(DiffLine::new(DiffLineType::Added, "new line"));

        assert_eq!(hunk.lines.len(), 1);
    }

    #[test]
    fn test_diff_hunk_toggle_collapsed() {
        let mut hunk = DiffHunk::new("@@ -1,5 +1,6 @@");
        assert!(!hunk.collapsed);

        hunk.toggle_collapsed();
        assert!(hunk.collapsed);

        hunk.toggle_collapsed();
        assert!(!hunk.collapsed);
    }

    #[test]
    fn test_diff_hunk_visible_lines() {
        let mut hunk = DiffHunk::new("@@ -1,5 +1,6 @@");
        hunk.add_line(DiffLine::new(DiffLineType::Added, "line 1"));
        hunk.add_line(DiffLine::new(DiffLineType::Removed, "line 2"));

        assert_eq!(hunk.visible_lines().len(), 2);

        hunk.toggle_collapsed();
        assert_eq!(hunk.visible_lines().len(), 0);
    }

    #[test]
    fn test_diff_widget_creation() {
        let widget = DiffWidget::new();
        assert!(widget.hunks.is_empty());
        assert_eq!(widget.view_type, DiffViewType::Unified);
    }

    #[test]
    fn test_diff_widget_add_hunk() {
        let mut widget = DiffWidget::new();
        let hunk = DiffHunk::new("@@ -1,5 +1,6 @@");
        widget.add_hunk(hunk);

        assert_eq!(widget.hunks.len(), 1);
        assert_eq!(widget.approvals.len(), 1);
    }

    #[test]
    fn test_diff_widget_toggle_view_type() {
        let mut widget = DiffWidget::new();
        assert_eq!(widget.view_type, DiffViewType::Unified);

        widget.toggle_view_type();
        assert_eq!(widget.view_type, DiffViewType::SideBySide);

        widget.toggle_view_type();
        assert_eq!(widget.view_type, DiffViewType::Unified);
    }

    #[test]
    fn test_diff_widget_hunk_selection() {
        let mut widget = DiffWidget::new();
        widget.add_hunk(DiffHunk::new("@@ -1,5 +1,6 @@"));
        widget.add_hunk(DiffHunk::new("@@ -10,5 +11,6 @@"));

        widget.select_next_hunk();
        assert_eq!(widget.selected_hunk, Some(0));

        widget.select_next_hunk();
        assert_eq!(widget.selected_hunk, Some(1));

        widget.select_prev_hunk();
        assert_eq!(widget.selected_hunk, Some(0));
    }

    #[test]
    fn test_diff_widget_approval() {
        let mut widget = DiffWidget::new();
        widget.add_hunk(DiffHunk::new("@@ -1,5 +1,6 @@"));
        widget.add_hunk(DiffHunk::new("@@ -10,5 +11,6 @@"));

        widget.approve_all();
        assert_eq!(widget.approved_hunks().len(), 2);
        assert_eq!(widget.rejected_hunks().len(), 0);

        widget.reject_all();
        assert_eq!(widget.approved_hunks().len(), 0);
        assert_eq!(widget.rejected_hunks().len(), 2);
    }

    #[test]
    fn test_diff_widget_hunk_approval() {
        let mut widget = DiffWidget::new();
        widget.add_hunk(DiffHunk::new("@@ -1,5 +1,6 @@"));
        widget.add_hunk(DiffHunk::new("@@ -10,5 +11,6 @@"));

        widget.select_next_hunk();
        widget.approve_hunk();

        assert_eq!(widget.approved_hunks().len(), 1);
        assert_eq!(widget.approved_hunks()[0], 0);
    }

    #[test]
    fn test_diff_widget_scroll() {
        let mut widget = DiffWidget::new();
        let mut hunk = DiffHunk::new("@@ -1,5 +1,6 @@");
        for i in 0..20 {
            hunk.add_line(DiffLine::new(DiffLineType::Unchanged, format!("line {}", i)));
        }
        widget.add_hunk(hunk);

        assert_eq!(widget.scroll, 0);

        widget.scroll_down(10);
        assert_eq!(widget.scroll, 1);

        widget.scroll_up();
        assert_eq!(widget.scroll, 0);

        widget.scroll_up();
        assert_eq!(widget.scroll, 0);
    }
}
