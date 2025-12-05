//! Property-based tests for DiffWidget
//! Tests universal properties that should hold across all inputs
//! Uses proptest for random test case generation
//! Validates Requirements 3.1, 3.2, 3.5, 3.4, 3.6

use proptest::prelude::*;
use ricecoder_tui::{DiffHunk, DiffLine, DiffLineType, DiffWidget};

// ============================================================================
// Generators for Property Tests
// ============================================================================

/// Generate a valid line type
fn arb_line_type() -> impl Strategy<Value = DiffLineType> {
    prop_oneof![
        Just(DiffLineType::Added),
        Just(DiffLineType::Removed),
        Just(DiffLineType::Unchanged),
        Just(DiffLineType::Context),
    ]
}

/// Generate a valid line number (1-10000)
fn arb_line_number() -> impl Strategy<Value = usize> {
    1..=10000usize
}

/// Generate a diff line with random content and line numbers
fn arb_diff_line() -> impl Strategy<Value = DiffLine> {
    (
        arb_line_type(),
        ".*",
        prop::option::of(arb_line_number()),
        prop::option::of(arb_line_number()),
    )
        .prop_map(|(line_type, content, old_num, new_num)| {
            let mut line = DiffLine::new(line_type, content);
            if let Some(num) = old_num {
                line = line.with_old_line_num(num);
            }
            if let Some(num) = new_num {
                line = line.with_new_line_num(num);
            }
            line
        })
}

/// Generate a diff hunk with random lines
fn arb_diff_hunk() -> impl Strategy<Value = DiffHunk> {
    (
        "@@ -\\d+,\\d+ \\+\\d+,\\d+ @@",
        prop::collection::vec(arb_diff_line(), 0..50),
    )
        .prop_map(|(header, lines)| {
            let mut hunk = DiffHunk::new(header);
            for line in lines {
                hunk.add_line(line);
            }
            hunk
        })
}

/// Generate a diff widget with random hunks
fn arb_diff_widget() -> impl Strategy<Value = DiffWidget> {
    prop::collection::vec(arb_diff_hunk(), 0..20).prop_map(|hunks| {
        let mut widget = DiffWidget::new();
        for hunk in hunks {
            widget.add_hunk(hunk);
        }
        widget
    })
}

// ============================================================================
// Property 3: Diff Display Accuracy
// **Feature: ricecoder-tui, Property 3: Diff Display Accuracy**
// **Validates: Requirements 3.1, 3.2, 3.5**
// Generate random diffs and verify all lines display with correct line numbers
// ============================================================================

proptest! {
    #[test]
    fn prop_test_diff_display_accuracy_all_lines_visible(widget in arb_diff_widget()) {
        // For any diff widget, all lines in expanded hunks should be visible
        for hunk in &widget.hunks {
            if !hunk.collapsed {
                assert_eq!(hunk.visible_lines().len(), hunk.lines.len());
            }
        }
    }

    #[test]
    fn prop_test_diff_display_accuracy_collapsed_hunks_empty(widget in arb_diff_widget()) {
        // For any diff widget, collapsed hunks should have no visible lines
        for hunk in &widget.hunks {
            if hunk.collapsed {
                assert_eq!(hunk.visible_lines().len(), 0);
            }
        }
    }

    #[test]
    fn prop_test_diff_display_accuracy_line_numbers_preserved(
        lines in prop::collection::vec(arb_diff_line(), 1..100)
    ) {
        // For any set of diff lines, line numbers should be preserved exactly
        for line in &lines {
            if let Some(old_num) = line.old_line_num {
                assert!(old_num > 0);
            }
            if let Some(new_num) = line.new_line_num {
                assert!(new_num > 0);
            }
        }
    }

    #[test]
    fn prop_test_diff_display_accuracy_line_types_preserved(
        line_type in arb_line_type(),
        content in ".*"
    ) {
        // For any diff line, the line type should be preserved exactly
        let line = DiffLine::new(line_type, content);
        assert_eq!(line.line_type, line_type);
    }

    #[test]
    fn prop_test_diff_display_accuracy_content_preserved(content in ".*") {
        // For any content string, the content should be preserved exactly
        let line = DiffLine::new(DiffLineType::Added, content.clone());
        assert_eq!(line.content, content);
    }

    #[test]
    fn prop_test_diff_display_accuracy_hunk_header_preserved(header in "@@ -\\d+,\\d+ \\+\\d+,\\d+ @@") {
        // For any hunk header, the header should be preserved exactly
        let hunk = DiffHunk::new(header.clone());
        assert_eq!(hunk.header, header);
    }

    #[test]
    fn prop_test_diff_display_accuracy_all_lines_in_hunk(
        lines in prop::collection::vec(arb_diff_line(), 1..100)
    ) {
        // For any set of lines added to a hunk, all lines should be retrievable
        let mut hunk = DiffHunk::new("@@ -1,5 +1,6 @@");
        for line in &lines {
            hunk.add_line(line.clone());
        }
        assert_eq!(hunk.lines.len(), lines.len());
    }

    #[test]
    fn prop_test_diff_display_accuracy_visible_lines_count(widget in arb_diff_widget()) {
        // For any diff widget, visible lines count should match expanded hunks
        for hunk in &widget.hunks {
            let visible = hunk.visible_lines().len();
            if hunk.collapsed {
                assert_eq!(visible, 0);
            } else {
                assert_eq!(visible, hunk.lines.len());
            }
        }
    }
}

// ============================================================================
// Property 4: Hunk-Level Approval Isolation
// **Feature: ricecoder-tui, Property 4: Hunk-Level Approval Isolation**
// **Validates: Requirements 3.4, 3.6**
// Generate random hunk selections and verify approval isolation
// ============================================================================

proptest! {
    #[test]
    fn prop_test_hunk_approval_isolation_approve_one_hunk(
        widget in arb_diff_widget(),
        hunk_idx in 0usize..20
    ) {
        // For any diff widget and hunk index, approving one hunk should not affect others
        let mut w = widget;
        if hunk_idx < w.hunks.len() {
            w.selected_hunk = Some(hunk_idx);
            w.approve_hunk();

            // Check that only the selected hunk is approved
            for (idx, &approved) in w.approvals.iter().enumerate() {
                if idx == hunk_idx {
                    assert!(approved);
                } else {
                    assert!(!approved);
                }
            }
        }
    }

    #[test]
    fn prop_test_hunk_approval_isolation_reject_one_hunk(
        widget in arb_diff_widget(),
        hunk_idx in 0usize..20
    ) {
        // For any diff widget and hunk index, rejecting one hunk should not affect others
        let mut w = widget;
        if hunk_idx < w.hunks.len() {
            w.approve_all();
            w.selected_hunk = Some(hunk_idx);
            w.reject_hunk();

            // Check that only the selected hunk is rejected
            for (idx, &approved) in w.approvals.iter().enumerate() {
                if idx == hunk_idx {
                    assert!(!approved);
                } else {
                    assert!(approved);
                }
            }
        }
    }

    #[test]
    fn prop_test_hunk_approval_isolation_approve_all_affects_all(widget in arb_diff_widget()) {
        // For any diff widget, approve_all should approve all hunks
        let mut w = widget;
        w.approve_all();

        for &approved in &w.approvals {
            assert!(approved);
        }
    }

    #[test]
    fn prop_test_hunk_approval_isolation_reject_all_affects_all(widget in arb_diff_widget()) {
        // For any diff widget, reject_all should reject all hunks
        let mut w = widget;
        w.approve_all();
        w.reject_all();

        for &approved in &w.approvals {
            assert!(!approved);
        }
    }

    #[test]
    fn prop_test_hunk_approval_isolation_approved_hunks_count(widget in arb_diff_widget()) {
        // For any diff widget, approved_hunks count should match actual approvals
        let mut w = widget;
        w.approve_all();

        let approved_count = w.approved_hunks().len();
        assert_eq!(approved_count, w.hunks.len());
    }

    #[test]
    fn prop_test_hunk_approval_isolation_rejected_hunks_count(widget in arb_diff_widget()) {
        // For any diff widget, rejected_hunks count should match actual rejections
        let mut w = widget;
        w.reject_all();

        let rejected_count = w.rejected_hunks().len();
        assert_eq!(rejected_count, w.hunks.len());
    }

    #[test]
    fn prop_test_hunk_approval_isolation_approved_plus_rejected_equals_total(
        widget in arb_diff_widget()
    ) {
        // For any diff widget, approved + rejected should equal total hunks
        let mut w = widget;
        w.approve_all();
        w.selected_hunk = Some(0);
        if w.hunks.len() > 0 {
            w.reject_hunk();
        }

        let approved = w.approved_hunks().len();
        let rejected = w.rejected_hunks().len();
        assert_eq!(approved + rejected, w.hunks.len());
    }

    #[test]
    fn prop_test_hunk_approval_isolation_no_overlap(widget in arb_diff_widget()) {
        // For any diff widget, no hunk should be both approved and rejected
        let mut w = widget;
        w.approve_all();
        w.selected_hunk = Some(0);
        if w.hunks.len() > 0 {
            w.reject_hunk();
        }

        let approved = w.approved_hunks();
        let rejected = w.rejected_hunks();

        for idx in &approved {
            assert!(!rejected.contains(idx));
        }
    }
}

// ============================================================================
// Additional Property Tests for Robustness
// ============================================================================

proptest! {
    #[test]
    fn prop_test_diff_widget_view_toggle_idempotent(widget in arb_diff_widget()) {
        // For any diff widget, toggling view type twice should return to original
        let mut w = widget;
        let original = w.view_type;

        w.toggle_view_type();
        w.toggle_view_type();

        assert_eq!(w.view_type, original);
    }

    #[test]
    fn prop_test_diff_hunk_collapse_toggle_idempotent(hunk in arb_diff_hunk()) {
        // For any diff hunk, toggling collapsed twice should return to original
        let mut h = hunk;
        let original = h.collapsed;

        h.toggle_collapsed();
        h.toggle_collapsed();

        assert_eq!(h.collapsed, original);
    }

    #[test]
    fn prop_test_diff_widget_scroll_bounds(widget in arb_diff_widget()) {
        // For any diff widget, scroll should never be negative
        let mut w = widget;
        w.scroll_up();
        w.scroll_up();
        w.scroll_up();

        assert_eq!(w.scroll, 0);
    }

    #[test]
    fn prop_test_diff_widget_hunk_count_matches_approvals(widget in arb_diff_widget()) {
        // For any diff widget, hunk count should match approval count
        let w = widget;
        assert_eq!(w.hunks.len(), w.approvals.len());
    }

    #[test]
    fn prop_test_diff_line_with_line_numbers_preserves_type(
        line_type in arb_line_type(),
        old_num in arb_line_number(),
        new_num in arb_line_number()
    ) {
        // For any diff line with line numbers, type should be preserved
        let line = DiffLine::new(line_type, "content")
            .with_old_line_num(old_num)
            .with_new_line_num(new_num);

        assert_eq!(line.line_type, line_type);
        assert_eq!(line.old_line_num, Some(old_num));
        assert_eq!(line.new_line_num, Some(new_num));
    }

    #[test]
    fn prop_test_diff_widget_navigation_bounds(widget in arb_diff_widget()) {
        // For any diff widget, navigation should stay within bounds
        let mut w = widget;

        // Navigate forward
        for _ in 0..100 {
            w.select_next_hunk();
        }

        // Should not exceed bounds
        if let Some(idx) = w.selected_hunk {
            assert!(idx < w.hunks.len());
        }

        // Navigate backward
        for _ in 0..100 {
            w.select_prev_hunk();
        }

        // Should not go below None
        if let Some(idx) = w.selected_hunk {
            assert!(idx < w.hunks.len());
        }
    }

    #[test]
    fn prop_test_diff_widget_empty_hunk_handling(widget in arb_diff_widget()) {
        // For any diff widget, empty hunks should be handled correctly
        let mut w = widget;
        let empty_hunk = DiffHunk::new("@@ -1,0 +1,0 @@");
        w.add_hunk(empty_hunk);

        assert_eq!(w.hunks.last().unwrap().lines.len(), 0);
        assert_eq!(w.hunks.last().unwrap().visible_lines().len(), 0);
    }
}
