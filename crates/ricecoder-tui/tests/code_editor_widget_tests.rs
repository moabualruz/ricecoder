use ricecoder_tui::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_from_extension() {
        assert_eq!(Language::from_extension("rs"), Language::Rust);
        assert_eq!(Language::from_extension("py"), Language::Python);
        assert_eq!(Language::from_extension("js"), Language::JavaScript);
        assert_eq!(Language::from_extension("ts"), Language::TypeScript);
        assert_eq!(Language::from_extension("go"), Language::Go);
        assert_eq!(Language::from_extension("java"), Language::Java);
        assert_eq!(Language::from_extension("unknown"), Language::PlainText);
    }

    #[test]
    fn test_language_name() {
        assert_eq!(Language::Rust.name(), "Rust");
        assert_eq!(Language::Python.name(), "Python");
        assert_eq!(Language::JavaScript.name(), "JavaScript");
    }

    #[test]
    fn test_code_line_creation() {
        let line = CodeLine::new(1, "fn main() {}", Language::Rust);
        assert_eq!(line.line_number, 1);
        assert_eq!(line.content, "fn main() {}");
        assert_eq!(line.language, Language::Rust);
        assert!(!line.highlighted);
    }

    #[test]
    fn test_code_line_highlight() {
        let mut line = CodeLine::new(1, "code", Language::Rust);
        assert!(!line.highlighted);

        line.highlight();
        assert!(line.highlighted);

        line.unhighlight();
        assert!(!line.highlighted);
    }

    #[test]
    fn test_code_line_display_text() {
        let line = CodeLine::new(1, "fn main() {}", Language::Rust);
        let display = line.display_text();
        assert!(display.contains("1"));
        assert!(display.contains("fn main() {}"));
    }

    #[test]
    fn test_code_editor_creation() {
        let editor = CodeEditorWidget::new(Language::Rust);
        assert_eq!(editor.language(), Language::Rust);
        assert_eq!(editor.line_count(), 0);
    }

    #[test]
    fn test_code_editor_set_code() {
        let mut editor = CodeEditorWidget::new(Language::Rust);
        editor.set_code("fn main() {\n    println!(\"Hello\");\n}");

        assert_eq!(editor.line_count(), 3);
    }

    #[test]
    fn test_code_editor_add_line() {
        let mut editor = CodeEditorWidget::new(Language::Rust);
        editor.add_line("fn main() {}");
        editor.add_line("    println!(\"Hello\");");

        assert_eq!(editor.line_count(), 2);
    }

    #[test]
    fn test_code_editor_clear() {
        let mut editor = CodeEditorWidget::new(Language::Rust);
        editor.add_line("code");
        editor.select_line(0);

        editor.clear();
        assert_eq!(editor.line_count(), 0);
        assert!(editor.selected_line().is_none());
    }

    #[test]
    fn test_code_editor_highlight() {
        let mut editor = CodeEditorWidget::new(Language::Rust);
        editor.add_line("line 1");
        editor.add_line("line 2");

        editor.highlight_line(0);
        assert!(editor.get_line(0).unwrap().highlighted);
        assert!(!editor.get_line(1).unwrap().highlighted);

        editor.clear_highlights();
        assert!(!editor.get_line(0).unwrap().highlighted);
        assert!(!editor.get_line(1).unwrap().highlighted);
    }

    #[test]
    fn test_code_editor_selection() {
        let mut editor = CodeEditorWidget::new(Language::Rust);
        editor.add_line("line 1");
        editor.add_line("line 2");

        editor.select_line(0);
        assert_eq!(editor.selected_line(), Some(0));

        editor.deselect_line();
        assert!(editor.selected_line().is_none());
    }

    #[test]
    fn test_code_editor_scroll() {
        let mut editor = CodeEditorWidget::new(Language::Rust);
        for i in 0..10 {
            editor.add_line(format!("line {}", i));
        }

        editor.scroll_down(5);
        assert_eq!(editor.scroll_offset(), 1);

        editor.scroll_up();
        assert_eq!(editor.scroll_offset(), 0);

        editor.scroll_to_bottom(5);
        assert!(editor.is_at_bottom(5));
    }

    #[test]
    fn test_code_editor_visible_lines() {
        let mut editor = CodeEditorWidget::new(Language::Rust);
        for i in 0..10 {
            editor.add_line(format!("line {}", i));
        }

        let visible = editor.visible_lines(5);
        assert_eq!(visible.len(), 5);
    }

    #[test]
    fn test_code_editor_set_language() {
        let mut editor = CodeEditorWidget::new(Language::Rust);
        editor.add_line("code");

        editor.set_language(Language::Python);
        assert_eq!(editor.language(), Language::Python);
        assert_eq!(editor.get_line(0).unwrap().language, Language::Python);
    }

    #[test]
    fn test_code_editor_theme() {
        let mut editor = CodeEditorWidget::new(Language::Rust);
        assert_eq!(editor.theme(), SyntaxTheme::Dark);

        editor.set_theme(SyntaxTheme::Light);
        assert_eq!(editor.theme(), SyntaxTheme::Light);
    }

    #[test]
    fn test_code_editor_scroll_percentage() {
        let mut editor = CodeEditorWidget::new(Language::Rust);
        for i in 0..10 {
            editor.add_line(format!("line {}", i));
        }

        assert_eq!(editor.scroll_percentage(5), 0);

        editor.scroll_to_bottom(5);
        assert_eq!(editor.scroll_percentage(5), 100);
    }

    #[test]
    fn test_code_editor_title() {
        let mut editor = CodeEditorWidget::new(Language::Rust);
        assert_eq!(editor.title(), "Code");

        editor.set_title("main.rs");
        assert_eq!(editor.title(), "main.rs");
    }

    #[test]
    fn test_code_editor_line_numbers() {
        let mut editor = CodeEditorWidget::new(Language::Rust);
        assert!(editor.show_line_numbers());

        editor.set_show_line_numbers(false);
        assert!(!editor.show_line_numbers());
    }

    #[test]
    fn test_code_editor_borders() {
        let mut editor = CodeEditorWidget::new(Language::Rust);
        assert!(editor.show_borders());

        editor.set_show_borders(false);
        assert!(!editor.show_borders());
    }

    #[test]
    fn test_code_editor_tab_width() {
        let mut editor = CodeEditorWidget::new(Language::Rust);
        assert_eq!(editor.tab_width(), 4);

        editor.set_tab_width(2);
        assert_eq!(editor.tab_width(), 2);
    }
}
