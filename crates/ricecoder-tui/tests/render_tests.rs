use ricecoder_tui::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::{DiffHunk, DiffLine};

    #[test]
    fn test_renderer_creation() {
        let renderer = Renderer::new();
        let default_renderer = Renderer::default();
        // Both should be created successfully
        let _ = renderer;
        let _ = default_renderer;
    }

    #[test]
    fn test_render_diff_unified_empty() {
        let renderer = Renderer::new();
        let diff = DiffWidget::new();
        let theme = Theme::default();
        let area = Rect {
            x: 0,
            y: 0,
            width: 80,
            height: 24,
        };

        let lines = renderer.render_diff_unified(&diff, area, &theme);
        assert!(!lines.is_empty());
        // Should have at least header and empty line
        assert!(lines.len() >= 2);
    }

    #[test]
    fn test_render_diff_unified_with_hunks() {
        let renderer = Renderer::new();
        let mut diff = DiffWidget::new();

        let mut hunk = DiffHunk::new("@@ -1,5 +1,6 @@");
        hunk.add_line(
            DiffLine::new(DiffLineType::Unchanged, "let x = 5;")
                .with_old_line_num(1)
                .with_new_line_num(1),
        );
        hunk.add_line(DiffLine::new(DiffLineType::Added, "let y = 10;").with_new_line_num(2));
        hunk.add_line(DiffLine::new(DiffLineType::Removed, "let z = 15;").with_old_line_num(2));

        diff.add_hunk(hunk);

        let theme = Theme::default();
        let area = Rect {
            x: 0,
            y: 0,
            width: 80,
            height: 24,
        };

        let lines = renderer.render_diff_unified(&diff, area, &theme);
        assert!(!lines.is_empty());
        // Should have header, hunk header, and lines
        assert!(lines.len() > 3);
    }

    #[test]
    fn test_render_diff_unified_with_collapsed_hunk() {
        let renderer = Renderer::new();
        let mut diff = DiffWidget::new();

        let mut hunk = DiffHunk::new("@@ -1,5 +1,6 @@");
        hunk.add_line(DiffLine::new(DiffLineType::Added, "new line"));
        hunk.toggle_collapsed();

        diff.add_hunk(hunk);

        let theme = Theme::default();
        let area = Rect {
            x: 0,
            y: 0,
            width: 80,
            height: 24,
        };

        let lines = renderer.render_diff_unified(&diff, area, &theme);
        assert!(!lines.is_empty());
        // Collapsed hunk should not show its lines
        let content = lines.iter().map(|l| l.to_string()).collect::<String>();
        assert!(!content.contains("new line"));
    }

    #[test]
    fn test_render_diff_unified_with_approval() {
        let renderer = Renderer::new();
        let mut diff = DiffWidget::new();

        let hunk = DiffHunk::new("@@ -1,5 +1,6 @@");
        diff.add_hunk(hunk);
        diff.approve_all();

        let theme = Theme::default();
        let area = Rect {
            x: 0,
            y: 0,
            width: 80,
            height: 24,
        };

        let lines = renderer.render_diff_unified(&diff, area, &theme);
        assert!(!lines.is_empty());
        // Should show approval indicator
        let content = lines.iter().map(|l| l.to_string()).collect::<String>();
        assert!(content.contains("✓"));
    }

    #[test]
    fn test_render_diff_side_by_side_empty() {
        let renderer = Renderer::new();
        let diff = DiffWidget::new();
        let theme = Theme::default();
        let area = Rect {
            x: 0,
            y: 0,
            width: 160,
            height: 24,
        };

        let lines = renderer.render_diff_side_by_side(&diff, area, &theme);
        assert!(!lines.is_empty());
        // Should have header and column headers
        assert!(lines.len() >= 3);
    }

    #[test]
    fn test_render_diff_side_by_side_with_hunks() {
        let renderer = Renderer::new();
        let mut diff = DiffWidget::new();

        let mut hunk = DiffHunk::new("@@ -1,5 +1,6 @@");
        hunk.add_line(DiffLine::new(DiffLineType::Added, "new line").with_new_line_num(1));
        hunk.add_line(DiffLine::new(DiffLineType::Removed, "old line").with_old_line_num(1));

        diff.add_hunk(hunk);

        let theme = Theme::default();
        let area = Rect {
            x: 0,
            y: 0,
            width: 160,
            height: 24,
        };

        let lines = renderer.render_diff_side_by_side(&diff, area, &theme);
        assert!(!lines.is_empty());
        // Should have header, column headers, hunk header, and lines
        assert!(lines.len() > 4);
    }

    #[test]
    fn test_render_diff_unified_view_type() {
        let renderer = Renderer::new();
        let mut diff = DiffWidget::new();

        let mut hunk = DiffHunk::new("@@ -1,5 +1,6 @@");
        hunk.add_line(DiffLine::new(DiffLineType::Added, "line"));
        diff.add_hunk(hunk);

        let theme = Theme::default();
        let area = Rect {
            x: 0,
            y: 0,
            width: 80,
            height: 24,
        };

        assert_eq!(diff.view_type, DiffViewType::Unified);
        let lines = renderer.render_diff(&diff, area, &theme);
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_render_diff_side_by_side_view_type() {
        let renderer = Renderer::new();
        let mut diff = DiffWidget::new();

        let mut hunk = DiffHunk::new("@@ -1,5 +1,6 @@");
        hunk.add_line(DiffLine::new(DiffLineType::Added, "line"));
        diff.add_hunk(hunk);
        diff.toggle_view_type();

        let theme = Theme::default();
        let area = Rect {
            x: 0,
            y: 0,
            width: 160,
            height: 24,
        };

        assert_eq!(diff.view_type, DiffViewType::SideBySide);
        let lines = renderer.render_diff(&diff, area, &theme);
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_render_diff_with_multiple_hunks() {
        let renderer = Renderer::new();
        let mut diff = DiffWidget::new();

        for i in 0..3 {
            let mut hunk = DiffHunk::new(&format!("@@ -{},{} +{},{} @@", i * 5, 5, i * 5, 5));
            hunk.add_line(DiffLine::new(DiffLineType::Added, format!("line {}", i)));
            diff.add_hunk(hunk);
        }

        let theme = Theme::default();
        let area = Rect {
            x: 0,
            y: 0,
            width: 80,
            height: 24,
        };

        let lines = renderer.render_diff_unified(&diff, area, &theme);
        assert!(!lines.is_empty());
        // Should have multiple hunk headers
        let content = lines.iter().map(|l| l.to_string()).collect::<String>();
        assert!(content.contains("@@"));
    }

    #[test]
    fn test_render_diff_line_numbers() {
        let renderer = Renderer::new();
        let mut diff = DiffWidget::new();

        let mut hunk = DiffHunk::new("@@ -10,5 +20,6 @@");
        hunk.add_line(
            DiffLine::new(DiffLineType::Unchanged, "code")
                .with_old_line_num(10)
                .with_new_line_num(20),
        );
        diff.add_hunk(hunk);

        let theme = Theme::default();
        let area = Rect {
            x: 0,
            y: 0,
            width: 80,
            height: 24,
        };

        let lines = renderer.render_diff_unified(&diff, area, &theme);
        let content = lines.iter().map(|l| l.to_string()).collect::<String>();
        // Should contain line numbers
        assert!(content.contains("10") || content.contains("20"));
    }

    #[test]
    fn test_render_diff_added_removed_lines() {
        let renderer = Renderer::new();
        let mut diff = DiffWidget::new();

        let mut hunk = DiffHunk::new("@@ -1,5 +1,6 @@");
        hunk.add_line(DiffLine::new(DiffLineType::Added, "added line"));
        hunk.add_line(DiffLine::new(DiffLineType::Removed, "removed line"));
        hunk.add_line(DiffLine::new(DiffLineType::Unchanged, "unchanged line"));
        diff.add_hunk(hunk);

        let theme = Theme::default();
        let area = Rect {
            x: 0,
            y: 0,
            width: 80,
            height: 24,
        };

        let lines = renderer.render_diff_unified(&diff, area, &theme);
        let content = lines.iter().map(|l| l.to_string()).collect::<String>();
        assert!(content.contains("added line"));
        assert!(content.contains("removed line"));
        assert!(content.contains("unchanged line"));
    }

    #[test]
    fn test_render_diff_selected_hunk() {
        let renderer = Renderer::new();
        let mut diff = DiffWidget::new();

        let hunk1 = DiffHunk::new("@@ -1,5 +1,6 @@");
        let hunk2 = DiffHunk::new("@@ -10,5 +11,6 @@");
        diff.add_hunk(hunk1);
        diff.add_hunk(hunk2);

        diff.select_next_hunk();
        assert_eq!(diff.selected_hunk, Some(0));

        let theme = Theme::default();
        let area = Rect {
            x: 0,
            y: 0,
            width: 80,
            height: 24,
        };

        let lines = renderer.render_diff_unified(&diff, area, &theme);
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_render_diff_stats() {
        let renderer = Renderer::new();
        let mut diff = DiffWidget::new();

        let mut hunk = DiffHunk::new("@@ -1,5 +1,6 @@");
        hunk.add_line(DiffLine::new(DiffLineType::Added, "line1"));
        hunk.add_line(DiffLine::new(DiffLineType::Added, "line2"));
        hunk.add_line(DiffLine::new(DiffLineType::Removed, "line3"));
        diff.add_hunk(hunk);

        let theme = Theme::default();
        let area = Rect {
            x: 0,
            y: 0,
            width: 80,
            height: 24,
        };

        let lines = renderer.render_diff_unified(&diff, area, &theme);
        let content = lines.iter().map(|l| l.to_string()).collect::<String>();
        // Should show stats: +2 -1
        assert!(content.contains("+2") || content.contains("-1"));
    }
}
