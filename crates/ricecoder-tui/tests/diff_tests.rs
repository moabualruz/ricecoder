use ricecoder_tui::*;

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
            hunk.add_line(DiffLine::new(
                DiffLineType::Unchanged,
                format!("line {}", i),
            ));
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