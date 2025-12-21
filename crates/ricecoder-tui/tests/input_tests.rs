use ricecoder_tui::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_detection() {
        assert_eq!(
            InputAnalyzer::detect_intent("generate code"),
            Intent::Generate
        );
        assert_eq!(
            InputAnalyzer::detect_intent("explain this"),
            Intent::Explain
        );
        assert_eq!(InputAnalyzer::detect_intent("fix the bug"), Intent::Fix);
        assert_eq!(
            InputAnalyzer::detect_intent("refactor this"),
            Intent::Refactor
        );
        assert_eq!(InputAnalyzer::detect_intent("unit test"), Intent::Test);
        assert_eq!(
            InputAnalyzer::detect_intent("document this"),
            Intent::Document
        );
        assert_eq!(
            InputAnalyzer::detect_intent("execute command"),
            Intent::Execute
        );
        assert_eq!(InputAnalyzer::detect_intent("help me"), Intent::Help);
        assert_eq!(InputAnalyzer::detect_intent("hello"), Intent::Chat);
    }

    #[test]
    fn test_input_validation() {
        assert!(InputAnalyzer::validate_input("hello").is_ok());
        assert!(InputAnalyzer::validate_input("").is_err());
        assert!(InputAnalyzer::validate_input("   ").is_err());
        assert!(InputAnalyzer::validate_input(&"x".repeat(10001)).is_err());
    }

    #[test]
    fn test_chat_input_widget() {
        let widget = ChatInputWidget::new();
        assert!(widget.text.is_empty());
        assert_eq!(widget.cursor, 0);
    }

    #[test]
    fn test_insert_char() {
        let mut widget = ChatInputWidget::new();
        widget.insert_char('h');
        widget.insert_char('i');
        assert_eq!(widget.text, "hi");
        assert_eq!(widget.cursor, 2);
    }

    #[test]
    fn test_backspace() {
        let mut widget = ChatInputWidget::new();
        widget.insert_char('h');
        widget.insert_char('i');
        widget.backspace();
        assert_eq!(widget.text, "h");
        assert_eq!(widget.cursor, 1);
    }

    #[test]
    fn test_cursor_movement() {
        let mut widget = ChatInputWidget::new();
        widget.text = "hello".to_string();
        widget.cursor = 5;

        widget.move_left();
        assert_eq!(widget.cursor, 4);

        widget.move_right();
        assert_eq!(widget.cursor, 5);

        widget.move_start();
        assert_eq!(widget.cursor, 0);

        widget.move_end();
        assert_eq!(widget.cursor, 5);
    }

    #[test]
    fn test_submit() {
        let mut widget = ChatInputWidget::new();
        widget.text = "hello".to_string();
        widget.cursor = 5;

        let submitted = widget.submit();
        assert_eq!(submitted, "hello");
        assert!(widget.text.is_empty());
        assert_eq!(widget.cursor, 0);
        assert_eq!(widget.history.len(), 1);
    }

    #[test]
    fn test_history_navigation() {
        let mut widget = ChatInputWidget::new();
        widget.submit_text("first");
        widget.submit_text("second");
        widget.submit_text("third");

        widget.history_up();
        assert_eq!(widget.text, "third");

        widget.history_up();
        assert_eq!(widget.text, "second");

        widget.history_down();
        assert_eq!(widget.text, "third");

        widget.history_down();
        assert!(widget.text.is_empty());
    }
}

impl ChatInputWidget {
    /// Helper for tests
    #[cfg(test)]
    fn submit_text(&mut self, text: &str) {
        self.text = text.to_string();
        self.submit();
    }
}
