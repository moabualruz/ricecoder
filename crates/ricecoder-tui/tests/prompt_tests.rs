use ricecoder_tui::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_indicators() {
        let context = ContextIndicators::new()
            .with_git_branch("main")
            .with_project_name("ricecoder")
            .with_provider("openai", "gpt-4");

        let formatted = context.format();
        assert!(formatted.contains("main"));
        assert!(formatted.contains("ricecoder"));
        assert!(formatted.contains("openai"));
    }

    #[test]
    fn test_prompt_config() {
        let config = PromptConfig::new().with_prefix("$").with_suffix(" ");

        assert_eq!(config.prefix, "$");
        assert_eq!(config.suffix, " ");
    }

    #[test]
    fn test_prompt_widget_creation() {
        let widget = PromptWidget::new();
        assert!(widget.input.is_empty());
        assert_eq!(widget.cursor, 0);
    }

    #[test]
    fn test_prompt_widget_input() {
        let mut widget = PromptWidget::new();
        widget.insert_char('h');
        widget.insert_char('i');

        assert_eq!(widget.input, "hi");
        assert_eq!(widget.cursor, 2);
    }

    #[test]
    fn test_prompt_widget_backspace() {
        let mut widget = PromptWidget::new();
        widget.input = "hello".to_string();
        widget.cursor = 5;

        widget.backspace();
        assert_eq!(widget.input, "hell");
        assert_eq!(widget.cursor, 4);
    }

    #[test]
    fn test_prompt_widget_cursor_movement() {
        let mut widget = PromptWidget::new();
        widget.input = "hello".to_string();
        widget.cursor = 2;

        widget.move_left();
        assert_eq!(widget.cursor, 1);

        widget.move_right();
        assert_eq!(widget.cursor, 2);

        widget.move_start();
        assert_eq!(widget.cursor, 0);

        widget.move_end();
        assert_eq!(widget.cursor, 5);
    }

    #[test]
    fn test_prompt_widget_submit() {
        let mut widget = PromptWidget::new();
        widget.input = "test command".to_string();

        let submitted = widget.submit();
        assert_eq!(submitted, "test command");
        assert!(widget.input.is_empty());
        assert_eq!(widget.history.len(), 1);
    }

    #[test]
    fn test_prompt_widget_history() {
        let mut widget = PromptWidget::new();

        widget.input = "first".to_string();
        widget.submit();

        widget.input = "second".to_string();
        widget.submit();

        widget.history_up();
        assert_eq!(widget.input, "second");

        widget.history_up();
        assert_eq!(widget.input, "first");

        widget.history_down();
        assert_eq!(widget.input, "second");

        widget.history_down();
        assert!(widget.input.is_empty());
    }

    #[test]
    fn test_prompt_formatting() {
        let mut widget = PromptWidget::new();
        widget.context = ContextIndicators::new()
            .with_git_branch("main")
            .with_project_name("ricecoder");

        let prompt = widget.format_prompt();
        assert!(prompt.contains("main"));
        assert!(prompt.contains("ricecoder"));
        assert!(prompt.contains("❯"));
    }

    #[test]
    fn test_display_line() {
        let mut widget = PromptWidget::new();
        widget.input = "hello".to_string();

        let display = widget.display_line();
        assert!(display.contains("hello"));
        assert!(display.contains("❯"));
    }
}
