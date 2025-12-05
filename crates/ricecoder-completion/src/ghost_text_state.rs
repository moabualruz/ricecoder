/// Ghost text state management for acceptance and dismissal
///
/// Manages the lifecycle of ghost text suggestions, including acceptance,
/// dismissal, and updates based on context changes.
use crate::types::GhostText;

/// Represents the state of ghost text
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum GhostTextState {
    /// No ghost text is currently displayed
    #[default]
    Dismissed,
    /// Ghost text is displayed and can be accepted
    Displayed(GhostText),
    /// Ghost text has been accepted
    Accepted(GhostText),
}

/// Partial acceptance mode for ghost text
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartialAcceptanceMode {
    /// Accept the entire ghost text
    Full,
    /// Accept only the current word
    Word,
    /// Accept only the current line
    Line,
    /// Accept a specific number of characters
    Characters(usize),
}

/// Ghost text state manager
pub trait GhostTextStateManager: Send + Sync {
    /// Display ghost text
    fn display(&mut self, ghost_text: GhostText);

    /// Dismiss ghost text
    fn dismiss(&mut self);

    /// Accept ghost text (full acceptance)
    fn accept(&mut self) -> Option<GhostText>;

    /// Accept ghost text partially
    fn accept_partial(&mut self, mode: PartialAcceptanceMode) -> Option<String>;

    /// Update ghost text based on context change
    fn update(&mut self, new_ghost_text: GhostText);

    /// Get current ghost text state
    fn get_state(&self) -> &GhostTextState;

    /// Check if ghost text is currently displayed
    fn is_displayed(&self) -> bool;
}

/// Basic ghost text state manager implementation
pub struct BasicGhostTextStateManager {
    state: GhostTextState,
}

impl BasicGhostTextStateManager {
    pub fn new() -> Self {
        Self {
            state: GhostTextState::Dismissed,
        }
    }
}

impl Default for BasicGhostTextStateManager {
    fn default() -> Self {
        Self::new()
    }
}

impl GhostTextStateManager for BasicGhostTextStateManager {
    fn display(&mut self, ghost_text: GhostText) {
        self.state = GhostTextState::Displayed(ghost_text);
    }

    fn dismiss(&mut self) {
        self.state = GhostTextState::Dismissed;
    }

    fn accept(&mut self) -> Option<GhostText> {
        match self.state.clone() {
            GhostTextState::Displayed(ghost_text) => {
                self.state = GhostTextState::Accepted(ghost_text.clone());
                Some(ghost_text)
            }
            _ => None,
        }
    }

    fn accept_partial(&mut self, mode: PartialAcceptanceMode) -> Option<String> {
        match &self.state {
            GhostTextState::Displayed(ghost_text) => {
                let text = match mode {
                    PartialAcceptanceMode::Full => ghost_text.text.clone(),
                    PartialAcceptanceMode::Word => {
                        // Accept until the first space or special character
                        ghost_text
                            .text
                            .split_whitespace()
                            .next()
                            .unwrap_or("")
                            .to_string()
                    }
                    PartialAcceptanceMode::Line => {
                        // Accept until the first newline
                        ghost_text.text.lines().next().unwrap_or("").to_string()
                    }
                    PartialAcceptanceMode::Characters(n) => {
                        ghost_text.text.chars().take(n).collect()
                    }
                };
                Some(text)
            }
            _ => None,
        }
    }

    fn update(&mut self, new_ghost_text: GhostText) {
        if matches!(self.state, GhostTextState::Displayed(_)) {
            self.state = GhostTextState::Displayed(new_ghost_text);
        }
    }

    fn get_state(&self) -> &GhostTextState {
        &self.state
    }

    fn is_displayed(&self) -> bool {
        matches!(self.state, GhostTextState::Displayed(_))
    }
}

/// Key handler for ghost text acceptance and dismissal
pub trait GhostTextKeyHandler: Send + Sync {
    /// Handle Tab key press (accept ghost text)
    fn handle_tab(&mut self) -> Option<String>;

    /// Handle Escape key press (dismiss ghost text)
    fn handle_escape(&mut self);

    /// Handle character input (update ghost text)
    fn handle_character_input(&mut self, _char: char) {
        // Default: dismiss ghost text on any character input
        self.handle_escape();
    }
}

/// Basic ghost text key handler
pub struct BasicGhostTextKeyHandler {
    state_manager: Box<dyn GhostTextStateManager>,
}

impl BasicGhostTextKeyHandler {
    pub fn new(state_manager: Box<dyn GhostTextStateManager>) -> Self {
        Self { state_manager }
    }
}

impl GhostTextKeyHandler for BasicGhostTextKeyHandler {
    fn handle_tab(&mut self) -> Option<String> {
        self.state_manager.accept().map(|gt| gt.text)
    }

    fn handle_escape(&mut self) {
        self.state_manager.dismiss();
    }

    fn handle_character_input(&mut self, _char: char) {
        // Dismiss ghost text on character input
        self.handle_escape();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Position, Range};

    #[test]
    fn test_ghost_text_state_display() {
        let mut manager = BasicGhostTextStateManager::new();
        let ghost_text = GhostText::new(
            "test".to_string(),
            Range::new(Position::new(0, 0), Position::new(0, 4)),
        );

        manager.display(ghost_text.clone());
        assert!(manager.is_displayed());
        assert_eq!(manager.get_state(), &GhostTextState::Displayed(ghost_text));
    }

    #[test]
    fn test_ghost_text_state_dismiss() {
        let mut manager = BasicGhostTextStateManager::new();
        let ghost_text = GhostText::new(
            "test".to_string(),
            Range::new(Position::new(0, 0), Position::new(0, 4)),
        );

        manager.display(ghost_text);
        assert!(manager.is_displayed());

        manager.dismiss();
        assert!(!manager.is_displayed());
        assert_eq!(manager.get_state(), &GhostTextState::Dismissed);
    }

    #[test]
    fn test_ghost_text_state_accept() {
        let mut manager = BasicGhostTextStateManager::new();
        let ghost_text = GhostText::new(
            "test".to_string(),
            Range::new(Position::new(0, 0), Position::new(0, 4)),
        );

        manager.display(ghost_text.clone());
        let accepted = manager.accept();

        assert!(accepted.is_some());
        assert_eq!(accepted.unwrap().text, "test");
        assert_eq!(manager.get_state(), &GhostTextState::Accepted(ghost_text));
    }

    #[test]
    fn test_ghost_text_partial_acceptance_word() {
        let mut manager = BasicGhostTextStateManager::new();
        let ghost_text = GhostText::new(
            "hello world".to_string(),
            Range::new(Position::new(0, 0), Position::new(0, 11)),
        );

        manager.display(ghost_text);
        let partial = manager.accept_partial(PartialAcceptanceMode::Word);

        assert_eq!(partial, Some("hello".to_string()));
    }

    #[test]
    fn test_ghost_text_partial_acceptance_line() {
        let mut manager = BasicGhostTextStateManager::new();
        let ghost_text = GhostText::new(
            "hello\nworld".to_string(),
            Range::new(Position::new(0, 0), Position::new(1, 5)),
        );

        manager.display(ghost_text);
        let partial = manager.accept_partial(PartialAcceptanceMode::Line);

        assert_eq!(partial, Some("hello".to_string()));
    }

    #[test]
    fn test_ghost_text_partial_acceptance_characters() {
        let mut manager = BasicGhostTextStateManager::new();
        let ghost_text = GhostText::new(
            "hello world".to_string(),
            Range::new(Position::new(0, 0), Position::new(0, 11)),
        );

        manager.display(ghost_text);
        let partial = manager.accept_partial(PartialAcceptanceMode::Characters(5));

        assert_eq!(partial, Some("hello".to_string()));
    }

    #[test]
    fn test_ghost_text_update() {
        let mut manager = BasicGhostTextStateManager::new();
        let ghost_text1 = GhostText::new(
            "test".to_string(),
            Range::new(Position::new(0, 0), Position::new(0, 4)),
        );
        let ghost_text2 = GhostText::new(
            "updated".to_string(),
            Range::new(Position::new(0, 0), Position::new(0, 7)),
        );

        manager.display(ghost_text1);
        manager.update(ghost_text2.clone());

        assert_eq!(manager.get_state(), &GhostTextState::Displayed(ghost_text2));
    }

    #[test]
    fn test_key_handler_tab() {
        let state_manager = Box::new(BasicGhostTextStateManager::new());
        let mut handler = BasicGhostTextKeyHandler::new(state_manager);

        // We need to manually set up the state since we can't access the manager directly
        // This test demonstrates the interface
        let result = handler.handle_tab();
        assert_eq!(result, None); // No ghost text displayed yet
    }

    #[test]
    fn test_key_handler_escape() {
        let state_manager = Box::new(BasicGhostTextStateManager::new());
        let mut handler = BasicGhostTextKeyHandler::new(state_manager);

        // Should not panic
        handler.handle_escape();
    }
}
