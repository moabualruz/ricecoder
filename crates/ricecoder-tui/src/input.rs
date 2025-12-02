//! Input handling for the TUI

/// Intent types for natural language input
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intent {
    /// Generate code
    Generate,
    /// Explain something
    Explain,
    /// Fix code
    Fix,
    /// Refactor code
    Refactor,
    /// Test code
    Test,
    /// Document code
    Document,
    /// Execute command
    Execute,
    /// Help request
    Help,
    /// General chat
    Chat,
}

/// Input analyzer for intent detection
pub struct InputAnalyzer;

impl InputAnalyzer {
    /// Detect intent from user input
    pub fn detect_intent(input: &str) -> Intent {
        let lower = input.to_lowercase();

        if lower.contains("generate") || lower.contains("create") || lower.contains("write") {
            Intent::Generate
        } else if lower.contains("explain") || lower.contains("what is") || lower.contains("how does") {
            Intent::Explain
        } else if lower.contains("fix") || lower.contains("bug") || lower.contains("error") {
            Intent::Fix
        } else if lower.contains("refactor") || lower.contains("improve") || lower.contains("optimize") {
            Intent::Refactor
        } else if lower.contains("test") || lower.contains("unit test") {
            Intent::Test
        } else if lower.contains("document") || lower.contains("comment") {
            Intent::Document
        } else if lower.contains("execute") || lower.contains("run") || lower.contains("command") {
            Intent::Execute
        } else if lower.contains("help") || lower.contains("?") {
            Intent::Help
        } else {
            Intent::Chat
        }
    }

    /// Get suggested commands based on intent
    pub fn suggest_commands(intent: Intent) -> Vec<&'static str> {
        match intent {
            Intent::Generate => vec!["generate", "create", "scaffold"],
            Intent::Explain => vec!["explain", "describe", "clarify"],
            Intent::Fix => vec!["fix", "debug", "resolve"],
            Intent::Refactor => vec!["refactor", "improve", "optimize"],
            Intent::Test => vec!["test", "unit-test", "validate"],
            Intent::Document => vec!["document", "comment", "annotate"],
            Intent::Execute => vec!["execute", "run", "apply"],
            Intent::Help => vec!["help", "guide", "tutorial"],
            Intent::Chat => vec!["chat", "discuss", "ask"],
        }
    }

    /// Validate input
    pub fn validate_input(input: &str) -> Result<(), String> {
        if input.trim().is_empty() {
            return Err("Input cannot be empty".to_string());
        }

        if input.len() > 10000 {
            return Err("Input is too long (max 10000 characters)".to_string());
        }

        Ok(())
    }
}

/// Chat input widget
pub struct ChatInputWidget {
    /// Current input text
    pub text: String,
    /// Cursor position
    pub cursor: usize,
    /// Input history
    pub history: Vec<String>,
    /// History index
    pub history_index: Option<usize>,
    /// Detected intent
    pub intent: Intent,
}

impl ChatInputWidget {
    /// Create a new chat input widget
    pub fn new() -> Self {
        Self {
            text: String::new(),
            cursor: 0,
            history: Vec::new(),
            history_index: None,
            intent: Intent::Chat,
        }
    }

    /// Insert character at cursor
    pub fn insert_char(&mut self, ch: char) {
        self.text.insert(self.cursor, ch);
        self.cursor += 1;
        self.update_intent();
    }

    /// Delete character before cursor
    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            self.text.remove(self.cursor - 1);
            self.cursor -= 1;
            self.update_intent();
        }
    }

    /// Delete character at cursor
    pub fn delete(&mut self) {
        if self.cursor < self.text.len() {
            self.text.remove(self.cursor);
            self.update_intent();
        }
    }

    /// Move cursor left
    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    /// Move cursor right
    pub fn move_right(&mut self) {
        if self.cursor < self.text.len() {
            self.cursor += 1;
        }
    }

    /// Move cursor to start
    pub fn move_start(&mut self) {
        self.cursor = 0;
    }

    /// Move cursor to end
    pub fn move_end(&mut self) {
        self.cursor = self.text.len();
    }

    /// Submit input
    pub fn submit(&mut self) -> String {
        let input = self.text.clone();
        self.history.push(input.clone());
        self.text.clear();
        self.cursor = 0;
        self.history_index = None;
        self.intent = Intent::Chat;
        input
    }

    /// Navigate history up
    pub fn history_up(&mut self) {
        if self.history.is_empty() {
            return;
        }

        match self.history_index {
            None => {
                self.history_index = Some(self.history.len() - 1);
                self.text = self.history[self.history.len() - 1].clone();
            }
            Some(idx) if idx > 0 => {
                self.history_index = Some(idx - 1);
                self.text = self.history[idx - 1].clone();
            }
            _ => {}
        }

        self.cursor = self.text.len();
    }

    /// Navigate history down
    pub fn history_down(&mut self) {
        match self.history_index {
            Some(idx) if idx < self.history.len() - 1 => {
                self.history_index = Some(idx + 1);
                self.text = self.history[idx + 1].clone();
                self.cursor = self.text.len();
            }
            Some(_) => {
                self.history_index = None;
                self.text.clear();
                self.cursor = 0;
            }
            None => {}
        }
    }

    /// Update detected intent
    pub fn update_intent(&mut self) {
        self.intent = InputAnalyzer::detect_intent(&self.text);
    }

    /// Get suggested commands
    pub fn suggestions(&self) -> Vec<&'static str> {
        InputAnalyzer::suggest_commands(self.intent)
    }
}

impl Default for ChatInputWidget {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_detection() {
        assert_eq!(InputAnalyzer::detect_intent("generate code"), Intent::Generate);
        assert_eq!(InputAnalyzer::detect_intent("explain this"), Intent::Explain);
        assert_eq!(InputAnalyzer::detect_intent("fix the bug"), Intent::Fix);
        assert_eq!(InputAnalyzer::detect_intent("refactor this"), Intent::Refactor);
        assert_eq!(InputAnalyzer::detect_intent("unit test"), Intent::Test);
        assert_eq!(InputAnalyzer::detect_intent("document this"), Intent::Document);
        assert_eq!(InputAnalyzer::detect_intent("execute command"), Intent::Execute);
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
