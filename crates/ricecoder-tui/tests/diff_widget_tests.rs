//! Unit tests for DiffWidget
//! Tests unified and side-by-side diff rendering, syntax highlighting, hunk navigation,
//! line number display, approval logic, and edge cases.
//! Validates Requirements 3.1, 3.2, 3.3, 3.4, 3.5, 3.6

use ricecoder_tui::{DiffWidget, DiffHunk, DiffLine, DiffLineType, DiffViewType};

// ============================================================================
// DiffLine Tests
// ============================================================================

#[test]
fn test_diff_line_creation_with_added_type() {
    let line = DiffLine::new(DiffLineType::Added, "let x = 5;");
    assert_eq!(line.line_type, DiffLineType::Added);
    assert_eq!(line.content, "let x = 5;");
    assert_eq!(line.old_line_num, None);
    assert_eq!(line.new_line_num, None);
}

#[test]
fn test_diff_line_creation_with_removed_type() {
    let line = DiffLine::new(DiffLineType::Removed, "let y = 10;");
    assert_eq!(line.line_type, DiffLineType::Removed);
    assert_eq!(line.content, "let y = 10;");
}

#[test]
fn test_diff_line_creation_with_unchanged_type() {
    let line = DiffLine::new(DiffLineType::Unchanged, "fn main() {");
    assert_eq!(line.line_type, DiffLineType::Unchanged);
    assert_eq!(line.content, "fn main() {");
}

#[test]
fn test_diff_line_with_line_numbers() {
    let line = DiffLine::new(DiffLineType::Unchanged, "code")
        .with_old_line_num(1)
        .with_new_line_num(1);

    assert_eq!(line.old_line_num, Some(1));
    assert_eq!(line.new_line_num, Some(1));
}

#[test]
fn test_diff_line_with_different_line_numbers() {
    let line = DiffLine::new(DiffLineType::Added, "new code")
        .with_old_line_num(5)
        .with_new_line_num(6);

    assert_eq!(line.old_line_num, Some(5));
    assert_eq!(line.new_line_num, Some(6));
}

#[test]
fn test_diff_line_with_only_old_line_number() {
    let line = DiffLine::new(DiffLineType::Removed, "old code")
        .with_old_line_num(10);

    assert_eq!(line.old_line_num, Some(10));
    assert_eq!(line.new_line_num, None);
}

#[test]
fn test_diff_line_with_only_new_line_number() {
    let line = DiffLine::new(DiffLineType::Added, "new code")
        .with_new_line_num(15);

    assert_eq!(line.old_line_num, None);
    assert_eq!(line.new_line_num, Some(15));
}

// ============================================================================
// DiffHunk Tests
// ============================================================================

#[test]
fn test_diff_hunk_creation() {
    let hunk = DiffHunk::new("@@ -1,5 +1,6 @@");
    assert_eq!(hunk.header, "@@ -1,5 +1,6 @@");
    assert!(!hunk.collapsed);
    assert!(hunk.lines.is_empty());
}

#[test]
fn test_diff_hunk_add_single_line() {
    let mut hunk = DiffHunk::new("@@ -1,5 +1,6 @@");
    hunk.add_line(DiffLine::new(DiffLineType::Added, "new line"));

    assert_eq!(hunk.lines.len(), 1);
    assert_eq!(hunk.lines[0].line_type, DiffLineType::Added);
}

#[test]
fn test_diff_hunk_add_multiple_lines() {
    let mut hunk = DiffHunk::new("@@ -1,5 +1,6 @@");
    hunk.add_line(DiffLine::new(DiffLineType::Added, "line 1"));
    hunk.add_line(DiffLine::new(DiffLineType::Removed, "line 2"));
    hunk.add_line(DiffLine::new(DiffLineType::Unchanged, "line 3"));

    assert_eq!(hunk.lines.len(), 3);
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
fn test_diff_hunk_visible_lines_when_expanded() {
    let mut hunk = DiffHunk::new("@@ -1,5 +1,6 @@");
    hunk.add_line(DiffLine::new(DiffLineType::Added, "line 1"));
    hunk.add_line(DiffLine::new(DiffLineType::Removed, "line 2"));

    assert_eq!(hunk.visible_lines().len(), 2);
}

#[test]
fn test_diff_hunk_visible_lines_when_collapsed() {
    let mut hunk = DiffHunk::new("@@ -1,5 +1,6 @@");
    hunk.add_line(DiffLine::new(DiffLineType::Added, "line 1"));
    hunk.add_line(DiffLine::new(DiffLineType::Removed, "line 2"));

    hunk.toggle_collapsed();
    assert_eq!(hunk.visible_lines().len(), 0);
}

#[test]
fn test_diff_hunk_visible_lines_empty() {
    let hunk = DiffHunk::new("@@ -1,5 +1,6 @@");
    assert_eq!(hunk.visible_lines().len(), 0);
}

// ============================================================================
// DiffWidget Creation and Basic Operations
// ============================================================================

#[test]
fn test_diff_widget_creation() {
    let widget = DiffWidget::new();
    assert!(widget.hunks.is_empty());
    assert_eq!(widget.view_type, DiffViewType::Unified);
    assert_eq!(widget.selected_hunk, None);
    assert_eq!(widget.scroll, 0);
    assert!(widget.approvals.is_empty());
}

#[test]
fn test_diff_widget_default() {
    let widget = DiffWidget::default();
    assert!(widget.hunks.is_empty());
    assert_eq!(widget.view_type, DiffViewType::Unified);
}

#[test]
fn test_diff_widget_add_single_hunk() {
    let mut widget = DiffWidget::new();
    let hunk = DiffHunk::new("@@ -1,5 +1,6 @@");
    widget.add_hunk(hunk);

    assert_eq!(widget.hunks.len(), 1);
    assert_eq!(widget.approvals.len(), 1);
    assert!(!widget.approvals[0]);
}

#[test]
fn test_diff_widget_add_multiple_hunks() {
    let mut widget = DiffWidget::new();
    widget.add_hunk(DiffHunk::new("@@ -1,5 +1,6 @@"));
    widget.add_hunk(DiffHunk::new("@@ -10,5 +11,6 @@"));
    widget.add_hunk(DiffHunk::new("@@ -20,5 +21,6 @@"));

    assert_eq!(widget.hunks.len(), 3);
    assert_eq!(widget.approvals.len(), 3);
}

// ============================================================================
// DiffWidget View Type Tests
// ============================================================================

#[test]
fn test_diff_widget_toggle_view_type_unified_to_side_by_side() {
    let mut widget = DiffWidget::new();
    assert_eq!(widget.view_type, DiffViewType::Unified);

    widget.toggle_view_type();
    assert_eq!(widget.view_type, DiffViewType::SideBySide);
}

#[test]
fn test_diff_widget_toggle_view_type_side_by_side_to_unified() {
    let mut widget = DiffWidget::new();
    widget.view_type = DiffViewType::SideBySide;

    widget.toggle_view_type();
    assert_eq!(widget.view_type, DiffViewType::Unified);
}

#[test]
fn test_diff_widget_toggle_view_type_multiple_times() {
    let mut widget = DiffWidget::new();
    let initial = widget.view_type;

    widget.toggle_view_type();
    widget.toggle_view_type();
    assert_eq!(widget.view_type, initial);

    widget.toggle_view_type();
    widget.toggle_view_type();
    widget.toggle_view_type();
    assert_ne!(widget.view_type, initial);
}

// ============================================================================
// DiffWidget Hunk Navigation Tests
// ============================================================================

#[test]
fn test_diff_widget_select_next_hunk_from_none() {
    let mut widget = DiffWidget::new();
    widget.add_hunk(DiffHunk::new("@@ -1,5 +1,6 @@"));

    widget.select_next_hunk();
    assert_eq!(widget.selected_hunk, Some(0));
}

#[test]
fn test_diff_widget_select_next_hunk_forward() {
    let mut widget = DiffWidget::new();
    widget.add_hunk(DiffHunk::new("@@ -1,5 +1,6 @@"));
    widget.add_hunk(DiffHunk::new("@@ -10,5 +11,6 @@"));

    widget.select_next_hunk();
    assert_eq!(widget.selected_hunk, Some(0));

    widget.select_next_hunk();
    assert_eq!(widget.selected_hunk, Some(1));
}

#[test]
fn test_diff_widget_select_next_hunk_at_end() {
    let mut widget = DiffWidget::new();
    widget.add_hunk(DiffHunk::new("@@ -1,5 +1,6 @@"));
    widget.add_hunk(DiffHunk::new("@@ -10,5 +11,6 @@"));

    widget.select_next_hunk();
    widget.select_next_hunk();
    let last_selected = widget.selected_hunk;

    widget.select_next_hunk();
    assert_eq!(widget.selected_hunk, last_selected);
}

#[test]
fn test_diff_widget_select_prev_hunk_from_none() {
    let mut widget = DiffWidget::new();
    widget.add_hunk(DiffHunk::new("@@ -1,5 +1,6 @@"));

    widget.select_prev_hunk();
    assert_eq!(widget.selected_hunk, None);
}

#[test]
fn test_diff_widget_select_prev_hunk_backward() {
    let mut widget = DiffWidget::new();
    widget.add_hunk(DiffHunk::new("@@ -1,5 +1,6 @@"));
    widget.add_hunk(DiffHunk::new("@@ -10,5 +11,6 @@"));

    widget.select_next_hunk();
    widget.select_next_hunk();
    assert_eq!(widget.selected_hunk, Some(1));

    widget.select_prev_hunk();
    assert_eq!(widget.selected_hunk, Some(0));
}

#[test]
fn test_diff_widget_select_prev_hunk_from_first() {
    let mut widget = DiffWidget::new();
    widget.add_hunk(DiffHunk::new("@@ -1,5 +1,6 @@"));
    widget.add_hunk(DiffHunk::new("@@ -10,5 +11,6 @@"));

    widget.select_next_hunk();
    assert_eq!(widget.selected_hunk, Some(0));

    widget.select_prev_hunk();
    assert_eq!(widget.selected_hunk, None);
}

#[test]
fn test_diff_widget_toggle_selected_hunk_collapsed() {
    let mut widget = DiffWidget::new();
    let mut hunk = DiffHunk::new("@@ -1,5 +1,6 @@");
    hunk.add_line(DiffLine::new(DiffLineType::Added, "line"));
    widget.add_hunk(hunk);

    widget.select_next_hunk();
    assert!(!widget.hunks[0].collapsed);

    widget.toggle_selected_hunk();
    assert!(widget.hunks[0].collapsed);

    widget.toggle_selected_hunk();
    assert!(!widget.hunks[0].collapsed);
}

#[test]
fn test_diff_widget_toggle_selected_hunk_when_none_selected() {
    let mut widget = DiffWidget::new();
    widget.add_hunk(DiffHunk::new("@@ -1,5 +1,6 @@"));

    widget.toggle_selected_hunk();
    assert!(!widget.hunks[0].collapsed);
}

// ============================================================================
// DiffWidget Approval Tests
// ============================================================================

#[test]
fn test_diff_widget_approve_all() {
    let mut widget = DiffWidget::new();
    widget.add_hunk(DiffHunk::new("@@ -1,5 +1,6 @@"));
    widget.add_hunk(DiffHunk::new("@@ -10,5 +11,6 @@"));

    widget.approve_all();
    assert_eq!(widget.approved_hunks().len(), 2);
    assert_eq!(widget.rejected_hunks().len(), 0);
}

#[test]
fn test_diff_widget_reject_all() {
    let mut widget = DiffWidget::new();
    widget.add_hunk(DiffHunk::new("@@ -1,5 +1,6 @@"));
    widget.add_hunk(DiffHunk::new("@@ -10,5 +11,6 @@"));

    widget.approve_all();
    widget.reject_all();
    assert_eq!(widget.approved_hunks().len(), 0);
    assert_eq!(widget.rejected_hunks().len(), 2);
}

#[test]
fn test_diff_widget_approve_single_hunk() {
    let mut widget = DiffWidget::new();
    widget.add_hunk(DiffHunk::new("@@ -1,5 +1,6 @@"));
    widget.add_hunk(DiffHunk::new("@@ -10,5 +11,6 @@"));

    widget.select_next_hunk();
    widget.approve_hunk();

    assert_eq!(widget.approved_hunks().len(), 1);
    assert_eq!(widget.approved_hunks()[0], 0);
    assert_eq!(widget.rejected_hunks().len(), 1);
}

#[test]
fn test_diff_widget_reject_single_hunk() {
    let mut widget = DiffWidget::new();
    widget.add_hunk(DiffHunk::new("@@ -1,5 +1,6 @@"));
    widget.add_hunk(DiffHunk::new("@@ -10,5 +11,6 @@"));

    widget.approve_all();
    widget.select_next_hunk();
    widget.select_next_hunk();
    widget.reject_hunk();

    assert_eq!(widget.approved_hunks().len(), 1);
    assert_eq!(widget.rejected_hunks().len(), 1);
    assert_eq!(widget.rejected_hunks()[0], 1);
}

#[test]
fn test_diff_widget_approve_hunk_when_none_selected() {
    let mut widget = DiffWidget::new();
    widget.add_hunk(DiffHunk::new("@@ -1,5 +1,6 @@"));

    widget.approve_hunk();
    assert_eq!(widget.approved_hunks().len(), 0);
}

#[test]
fn test_diff_widget_reject_hunk_when_none_selected() {
    let mut widget = DiffWidget::new();
    widget.add_hunk(DiffHunk::new("@@ -1,5 +1,6 @@"));

    widget.reject_hunk();
    assert_eq!(widget.rejected_hunks().len(), 1);
}

#[test]
fn test_diff_widget_approved_hunks_returns_correct_indices() {
    let mut widget = DiffWidget::new();
    widget.add_hunk(DiffHunk::new("@@ -1,5 +1,6 @@"));
    widget.add_hunk(DiffHunk::new("@@ -10,5 +11,6 @@"));
    widget.add_hunk(DiffHunk::new("@@ -20,5 +21,6 @@"));

    widget.select_next_hunk();
    widget.approve_hunk();
    widget.select_next_hunk();
    widget.select_next_hunk();
    widget.approve_hunk();

    let approved = widget.approved_hunks();
    assert_eq!(approved.len(), 2);
    assert!(approved.contains(&0));
    assert!(approved.contains(&2));
}

#[test]
fn test_diff_widget_rejected_hunks_returns_correct_indices() {
    let mut widget = DiffWidget::new();
    widget.add_hunk(DiffHunk::new("@@ -1,5 +1,6 @@"));
    widget.add_hunk(DiffHunk::new("@@ -10,5 +11,6 @@"));
    widget.add_hunk(DiffHunk::new("@@ -20,5 +21,6 @@"));

    let rejected = widget.rejected_hunks();
    assert_eq!(rejected.len(), 3);
    assert!(rejected.contains(&0));
    assert!(rejected.contains(&1));
    assert!(rejected.contains(&2));
}

// ============================================================================
// DiffWidget Scrolling Tests
// ============================================================================

#[test]
fn test_diff_widget_scroll_initial_position() {
    let widget = DiffWidget::new();
    assert_eq!(widget.scroll, 0);
}

#[test]
fn test_diff_widget_scroll_down() {
    let mut widget = DiffWidget::new();
    let mut hunk = DiffHunk::new("@@ -1,5 +1,6 @@");
    for i in 0..20 {
        hunk.add_line(DiffLine::new(DiffLineType::Unchanged, format!("line {}", i)));
    }
    widget.add_hunk(hunk);

    widget.scroll_down(10);
    assert_eq!(widget.scroll, 1);

    widget.scroll_down(10);
    assert_eq!(widget.scroll, 2);
}

#[test]
fn test_diff_widget_scroll_up() {
    let mut widget = DiffWidget::new();
    let mut hunk = DiffHunk::new("@@ -1,5 +1,6 @@");
    for i in 0..20 {
        hunk.add_line(DiffLine::new(DiffLineType::Unchanged, format!("line {}", i)));
    }
    widget.add_hunk(hunk);

    widget.scroll_down(10);
    widget.scroll_down(10);
    assert_eq!(widget.scroll, 2);

    widget.scroll_up();
    assert_eq!(widget.scroll, 1);

    widget.scroll_up();
    assert_eq!(widget.scroll, 0);
}

#[test]
fn test_diff_widget_scroll_up_at_top() {
    let mut widget = DiffWidget::new();
    widget.add_hunk(DiffHunk::new("@@ -1,5 +1,6 @@"));

    widget.scroll_up();
    assert_eq!(widget.scroll, 0);
}

#[test]
fn test_diff_widget_scroll_down_at_bottom() {
    let mut widget = DiffWidget::new();
    let mut hunk = DiffHunk::new("@@ -1,5 +1,6 @@");
    for i in 0..10 {
        hunk.add_line(DiffLine::new(DiffLineType::Unchanged, format!("line {}", i)));
    }
    widget.add_hunk(hunk);

    widget.scroll_down(10);
    let scroll_at_bottom = widget.scroll;

    widget.scroll_down(10);
    assert_eq!(widget.scroll, scroll_at_bottom);
}

// ============================================================================
// Edge Cases and Complex Scenarios
// ============================================================================

#[test]
fn test_diff_widget_empty_diff() {
    let widget = DiffWidget::new();
    assert!(widget.hunks.is_empty());
    assert!(widget.approvals.is_empty());
    assert_eq!(widget.approved_hunks().len(), 0);
    assert_eq!(widget.rejected_hunks().len(), 0);
}

#[test]
fn test_diff_widget_large_diff() {
    let mut widget = DiffWidget::new();
    for i in 0..100 {
        let mut hunk = DiffHunk::new(format!("@@ -{},{} +{},{} @@", i * 10, 5, i * 10, 6));
        for j in 0..50 {
            hunk.add_line(DiffLine::new(
                DiffLineType::Unchanged,
                format!("line {} in hunk {}", j, i),
            ));
        }
        widget.add_hunk(hunk);
    }

    assert_eq!(widget.hunks.len(), 100);
    assert_eq!(widget.approvals.len(), 100);
}

#[test]
fn test_diff_widget_mixed_line_types() {
    let mut widget = DiffWidget::new();
    let mut hunk = DiffHunk::new("@@ -1,10 +1,11 @@");

    hunk.add_line(DiffLine::new(DiffLineType::Unchanged, "fn main() {").with_old_line_num(1).with_new_line_num(1));
    hunk.add_line(DiffLine::new(DiffLineType::Removed, "    let x = 5;").with_old_line_num(2));
    hunk.add_line(DiffLine::new(DiffLineType::Added, "    let x = 10;").with_new_line_num(2));
    hunk.add_line(DiffLine::new(DiffLineType::Unchanged, "    println!(\"{}\", x);").with_old_line_num(3).with_new_line_num(3));
    hunk.add_line(DiffLine::new(DiffLineType::Added, "    let y = 20;").with_new_line_num(4));
    hunk.add_line(DiffLine::new(DiffLineType::Unchanged, "}").with_old_line_num(4).with_new_line_num(5));

    widget.add_hunk(hunk);

    assert_eq!(widget.hunks[0].lines.len(), 6);
    assert_eq!(widget.hunks[0].visible_lines().len(), 6);
}

#[test]
fn test_diff_widget_hunk_with_empty_content() {
    let mut widget = DiffWidget::new();
    let mut hunk = DiffHunk::new("@@ -1,5 +1,6 @@");
    hunk.add_line(DiffLine::new(DiffLineType::Added, ""));
    hunk.add_line(DiffLine::new(DiffLineType::Removed, ""));

    widget.add_hunk(hunk);

    assert_eq!(widget.hunks[0].lines.len(), 2);
    assert_eq!(widget.hunks[0].lines[0].content, "");
}

#[test]
fn test_diff_widget_hunk_with_special_characters() {
    let mut widget = DiffWidget::new();
    let mut hunk = DiffHunk::new("@@ -1,5 +1,6 @@");
    hunk.add_line(DiffLine::new(DiffLineType::Added, "let s = \"hello\\nworld\";"));
    hunk.add_line(DiffLine::new(DiffLineType::Added, "let emoji = \"🦀\";"));
    hunk.add_line(DiffLine::new(DiffLineType::Added, "let tab = \"a\\tb\";"));

    widget.add_hunk(hunk);

    assert_eq!(widget.hunks[0].lines.len(), 3);
    assert!(widget.hunks[0].lines[0].content.contains("\\n"));
    assert!(widget.hunks[0].lines[1].content.contains("🦀"));
}

#[test]
fn test_diff_widget_approval_isolation() {
    let mut widget = DiffWidget::new();
    widget.add_hunk(DiffHunk::new("@@ -1,5 +1,6 @@"));
    widget.add_hunk(DiffHunk::new("@@ -10,5 +11,6 @@"));
    widget.add_hunk(DiffHunk::new("@@ -20,5 +21,6 @@"));

    widget.select_next_hunk();
    widget.approve_hunk();

    assert_eq!(widget.approvals[0], true);
    assert_eq!(widget.approvals[1], false);
    assert_eq!(widget.approvals[2], false);

    widget.select_next_hunk();
    widget.approve_hunk();

    assert_eq!(widget.approvals[0], true);
    assert_eq!(widget.approvals[1], true);
    assert_eq!(widget.approvals[2], false);
}

#[test]
fn test_diff_widget_line_number_accuracy() {
    let mut widget = DiffWidget::new();
    let mut hunk = DiffHunk::new("@@ -1,5 +1,6 @@");

    for i in 1..=5 {
        hunk.add_line(
            DiffLine::new(DiffLineType::Unchanged, format!("line {}", i))
                .with_old_line_num(i)
                .with_new_line_num(i),
        );
    }
    hunk.add_line(
        DiffLine::new(DiffLineType::Added, "new line")
            .with_new_line_num(6),
    );

    widget.add_hunk(hunk);

    let lines = &widget.hunks[0].lines;
    for (idx, line) in lines.iter().enumerate() {
        if idx < 5 {
            assert_eq!(line.old_line_num, Some(idx + 1));
            assert_eq!(line.new_line_num, Some(idx + 1));
        } else {
            assert_eq!(line.old_line_num, None);
            assert_eq!(line.new_line_num, Some(6));
        }
    }
}

#[test]
fn test_diff_widget_navigation_with_collapsed_hunks() {
    let mut widget = DiffWidget::new();
    widget.add_hunk(DiffHunk::new("@@ -1,5 +1,6 @@"));
    widget.add_hunk(DiffHunk::new("@@ -10,5 +11,6 @@"));

    widget.select_next_hunk();
    widget.toggle_selected_hunk();
    assert!(widget.hunks[0].collapsed);

    widget.select_next_hunk();
    assert_eq!(widget.selected_hunk, Some(1));
    assert!(!widget.hunks[1].collapsed);
}

#[test]
fn test_diff_widget_view_toggle_preserves_state() {
    let mut widget = DiffWidget::new();
    widget.add_hunk(DiffHunk::new("@@ -1,5 +1,6 @@"));
    widget.select_next_hunk();
    widget.approve_hunk();

    let selected_before = widget.selected_hunk;
    let approved_before = widget.approved_hunks();

    widget.toggle_view_type();

    assert_eq!(widget.selected_hunk, selected_before);
    assert_eq!(widget.approved_hunks(), approved_before);
}
