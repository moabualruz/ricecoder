//! Prompt state management
//!
//! Implements the state machine for the prompt component:
//! - PromptMode (normal/shell)
//! - PromptInfo (input + parts)
//! - PromptState (full state container)
//!
//! # DDD Layer: Application
//! State management for the prompt widget.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::tui::prompt::parts::{AgentPart, FilePart, PromptPart, TextPart};

/// Input mode for the prompt
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PromptMode {
    /// Normal text input mode
    #[default]
    Normal,
    /// Shell command mode (triggered by ! at start)
    Shell,
}

/// Prompt content with attached parts
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromptInfo {
    /// The raw input text
    pub input: String,
    /// Attached parts (files, agents, pasted text)
    pub parts: Vec<PromptPart>,
    /// Input mode at time of creation (for history)
    #[serde(default)]
    pub mode: Option<PromptMode>,
}

impl PromptInfo {
    /// Create empty prompt info
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with input text
    pub fn with_input(input: impl Into<String>) -> Self {
        Self {
            input: input.into(),
            ..Default::default()
        }
    }

    /// Add a part
    pub fn add_part(&mut self, part: PromptPart) {
        self.parts.push(part);
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.input.is_empty() && self.parts.is_empty()
    }

    /// Clear all content
    pub fn clear(&mut self) {
        self.input.clear();
        self.parts.clear();
    }

    /// Get text parts only
    pub fn text_parts(&self) -> impl Iterator<Item = &TextPart> {
        self.parts.iter().filter_map(|p| match p {
            PromptPart::Text(t) => Some(t),
            _ => None,
        })
    }

    /// Get file parts only
    pub fn file_parts(&self) -> impl Iterator<Item = &FilePart> {
        self.parts.iter().filter_map(|p| match p {
            PromptPart::File(f) => Some(f),
            _ => None,
        })
    }

    /// Get agent parts only
    pub fn agent_parts(&self) -> impl Iterator<Item = &AgentPart> {
        self.parts.iter().filter_map(|p| match p {
            PromptPart::Agent(a) => Some(a),
            _ => None,
        })
    }

    /// Expand all pasted text parts inline
    pub fn expand_text_parts(&self) -> String {
        // Implementation: replace virtual text with actual text
        let mut result = self.input.clone();
        // Sort parts by position descending to avoid offset issues
        let mut text_parts: Vec<_> = self
            .text_parts()
            .filter_map(|p| p.source.as_ref().map(|s| (s.start, s.end, &p.text)))
            .collect();
        text_parts.sort_by(|a, b| b.0.cmp(&a.0));

        for (start, end, text) in text_parts {
            if start <= result.len() && end <= result.len() {
                result.replace_range(start..end, text);
            }
        }
        result
    }

    /// Get non-text parts (for submission)
    pub fn non_text_parts(&self) -> Vec<&PromptPart> {
        self.parts.iter().filter(|p| !p.is_text()).collect()
    }
}

/// Full prompt state container
#[derive(Debug, Clone)]
pub struct PromptState {
    /// Current prompt content
    pub prompt: PromptInfo,
    /// Current input mode
    pub mode: PromptMode,
    /// Mapping from extmark ID to part index
    pub extmark_to_part_index: HashMap<u32, usize>,
    /// Interrupt counter (for double-esc handling)
    pub interrupt_count: u8,
    /// Placeholder index for empty prompt
    pub placeholder_index: usize,
    /// Whether the prompt is disabled
    pub disabled: bool,
    /// Whether the prompt is focused
    pub focused: bool,
}

impl Default for PromptState {
    fn default() -> Self {
        Self::new()
    }
}

impl PromptState {
    /// Create new prompt state
    pub fn new() -> Self {
        Self {
            prompt: PromptInfo::new(),
            mode: PromptMode::Normal,
            extmark_to_part_index: HashMap::new(),
            interrupt_count: 0,
            placeholder_index: 0,
            disabled: false,
            focused: true,
        }
    }

    /// Reset to initial state
    pub fn reset(&mut self) {
        self.prompt.clear();
        self.mode = PromptMode::Normal;
        self.extmark_to_part_index.clear();
        self.interrupt_count = 0;
    }

    /// Toggle shell mode
    pub fn toggle_shell_mode(&mut self) {
        self.mode = match self.mode {
            PromptMode::Normal => PromptMode::Shell,
            PromptMode::Shell => PromptMode::Normal,
        };
    }

    /// Increment interrupt counter, returns true if should abort
    pub fn increment_interrupt(&mut self) -> bool {
        self.interrupt_count += 1;
        self.interrupt_count >= 2
    }

    /// Reset interrupt counter
    pub fn reset_interrupt(&mut self) {
        self.interrupt_count = 0;
    }

    /// Set prompt from history entry
    pub fn set_from_history(&mut self, info: PromptInfo) {
        self.mode = info.mode.unwrap_or(PromptMode::Normal);
        self.prompt = info;
        self.extmark_to_part_index.clear();
    }

    /// Register an extmark for a part
    pub fn register_extmark(&mut self, extmark_id: u32, part_index: usize) {
        self.extmark_to_part_index.insert(extmark_id, part_index);
    }

    /// Get part index for extmark
    pub fn get_part_for_extmark(&self, extmark_id: u32) -> Option<usize> {
        self.extmark_to_part_index.get(&extmark_id).copied()
    }

    /// Check if can submit
    pub fn can_submit(&self) -> bool {
        !self.disabled && !self.prompt.input.trim().is_empty()
    }
}

/// Placeholder messages for empty prompt
pub const PLACEHOLDERS: &[&str] = &[
    "Fix a TODO in the codebase",
    "What is the tech stack of this project?",
    "Fix broken tests",
    "Explain how the auth system works",
    "Add logging to this function",
];

/// Get a random placeholder
pub fn random_placeholder() -> &'static str {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as usize)
        .unwrap_or(0);
    PLACEHOLDERS[seed % PLACEHOLDERS.len()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::prompt::parts::TextSource;

    #[test]
    fn test_prompt_mode_default() {
        assert_eq!(PromptMode::default(), PromptMode::Normal);
    }

    #[test]
    fn test_prompt_info_empty() {
        let info = PromptInfo::new();
        assert!(info.is_empty());
    }

    #[test]
    fn test_prompt_info_with_input() {
        let info = PromptInfo::with_input("hello");
        assert!(!info.is_empty());
        assert_eq!(info.input, "hello");
    }

    #[test]
    fn test_prompt_state_reset() {
        let mut state = PromptState::new();
        state.prompt.input = "test".to_string();
        state.mode = PromptMode::Shell;
        state.interrupt_count = 1;

        state.reset();

        assert!(state.prompt.is_empty());
        assert_eq!(state.mode, PromptMode::Normal);
        assert_eq!(state.interrupt_count, 0);
    }

    #[test]
    fn test_interrupt_counter() {
        let mut state = PromptState::new();
        assert!(!state.increment_interrupt());
        assert!(state.increment_interrupt());
        state.reset_interrupt();
        assert_eq!(state.interrupt_count, 0);
    }

    #[test]
    fn test_can_submit() {
        let mut state = PromptState::new();
        assert!(!state.can_submit()); // empty

        state.prompt.input = "  ".to_string();
        assert!(!state.can_submit()); // whitespace only

        state.prompt.input = "hello".to_string();
        assert!(state.can_submit());

        state.disabled = true;
        assert!(!state.can_submit());
    }

    #[test]
    fn test_expand_text_parts() {
        let mut info = PromptInfo::with_input("[Pasted ~3 lines] more text");
        info.parts.push(PromptPart::Text(TextPart {
            text: "line1\nline2\nline3".to_string(),
            source: Some(TextSource::new(0, 17, "[Pasted ~3 lines]")),
        }));

        let expanded = info.expand_text_parts();
        assert_eq!(expanded, "line1\nline2\nline3 more text");
    }

    #[test]
    fn test_toggle_shell_mode() {
        let mut state = PromptState::new();
        assert_eq!(state.mode, PromptMode::Normal);

        state.toggle_shell_mode();
        assert_eq!(state.mode, PromptMode::Shell);

        state.toggle_shell_mode();
        assert_eq!(state.mode, PromptMode::Normal);
    }

    #[test]
    fn test_set_from_history() {
        let mut state = PromptState::new();
        state.register_extmark(1, 0);

        let mut info = PromptInfo::with_input("history input");
        info.mode = Some(PromptMode::Shell);

        state.set_from_history(info);

        assert_eq!(state.prompt.input, "history input");
        assert_eq!(state.mode, PromptMode::Shell);
        assert!(state.extmark_to_part_index.is_empty());
    }

    #[test]
    fn test_extmark_registration() {
        let mut state = PromptState::new();

        state.register_extmark(42, 0);
        state.register_extmark(43, 1);

        assert_eq!(state.get_part_for_extmark(42), Some(0));
        assert_eq!(state.get_part_for_extmark(43), Some(1));
        assert_eq!(state.get_part_for_extmark(99), None);
    }

    #[test]
    fn test_non_text_parts() {
        let mut info = PromptInfo::new();
        info.parts.push(PromptPart::Text(TextPart::new("text")));
        info.parts
            .push(PromptPart::File(FilePart::from_base64("image/png", "abc")));
        info.parts.push(PromptPart::Agent(AgentPart::new("build")));

        let non_text = info.non_text_parts();
        assert_eq!(non_text.len(), 2);
        assert!(non_text[0].is_file());
        assert!(non_text[1].is_agent());
    }

    #[test]
    fn test_random_placeholder() {
        let placeholder1 = random_placeholder();
        assert!(PLACEHOLDERS.contains(&placeholder1));

        // Verify it's one of the expected placeholders
        let mut found = false;
        for &expected in PLACEHOLDERS {
            if placeholder1 == expected {
                found = true;
                break;
            }
        }
        assert!(found);
    }

    #[test]
    fn test_placeholder_variety() {
        // Test that we have multiple placeholders available
        assert!(PLACEHOLDERS.len() > 1);
        assert!(PLACEHOLDERS.len() >= 5);
    }
}
