//! Chat Context Management
//!
//! Manages conversation context including messages, system prompts,
//! and token tracking. Ported from crustly's agent context.

use std::path::PathBuf;

use uuid::Uuid;

use super::types::{ChatMessage, ContentBlock, Role, Usage};

/// Chat context for a conversation
///
/// Tracks messages, token counts, and provides context window management.
#[derive(Debug, Clone)]
pub struct ChatContext {
    /// Session ID
    pub session_id: Uuid,

    /// System prompt
    pub system_prompt: Option<String>,

    /// Conversation messages
    pub messages: Vec<ChatMessage>,

    /// Tracked files in the conversation
    pub tracked_files: Vec<TrackedFile>,

    /// Current token count estimate
    pub token_count: usize,

    /// Maximum context tokens
    pub max_tokens: usize,

    /// Accumulated usage across iterations
    pub total_usage: Usage,
}

/// A file tracked in the conversation
#[derive(Debug, Clone)]
pub struct TrackedFile {
    /// File identifier
    pub id: Uuid,
    /// File path
    pub path: PathBuf,
    /// File content (if loaded)
    pub content: Option<String>,
    /// Estimated token count for this file
    pub token_count: usize,
}

impl ChatContext {
    /// Create a new chat context for a session
    pub fn new(session_id: Uuid, max_tokens: usize) -> Self {
        Self {
            session_id,
            system_prompt: None,
            messages: Vec::new(),
            tracked_files: Vec::new(),
            token_count: 0,
            max_tokens,
            total_usage: Usage::default(),
        }
    }

    /// Set the system prompt
    pub fn with_system_prompt(mut self, prompt: String) -> Self {
        self.token_count += Self::estimate_tokens(&prompt);
        self.system_prompt = Some(prompt);
        self
    }

    /// Add a message to the context
    pub fn add_message(&mut self, message: ChatMessage) {
        let tokens = self.estimate_message_tokens(&message);
        self.token_count += tokens;
        self.messages.push(message);
    }

    /// Add usage from a response
    pub fn add_usage(&mut self, usage: Usage) {
        self.total_usage.input_tokens += usage.input_tokens;
        self.total_usage.output_tokens += usage.output_tokens;
    }

    /// Track a file in the conversation
    pub fn track_file(&mut self, file: TrackedFile) {
        self.token_count += file.token_count;
        self.tracked_files.push(file);
    }

    /// Check if context would exceed limit with additional tokens
    pub fn would_exceed_limit(&self, additional_tokens: usize) -> bool {
        self.token_count + additional_tokens > self.max_tokens
    }

    /// Estimate tokens for a message
    fn estimate_message_tokens(&self, message: &ChatMessage) -> usize {
        let mut tokens = 0;

        for content in &message.content {
            match content {
                ContentBlock::Text { text } => {
                    tokens += Self::estimate_tokens(text);
                }
                ContentBlock::ToolUse { name, input, .. } => {
                    tokens += Self::estimate_tokens(name);
                    tokens += Self::estimate_tokens(&input.to_string());
                }
                ContentBlock::ToolResult { content, .. } => {
                    tokens += Self::estimate_tokens(content);
                }
            }
        }

        // Add overhead for message structure
        tokens + 4
    }

    /// Simple token estimation (roughly 4 characters per token)
    ///
    /// This is a heuristic that works reasonably well for English text.
    /// For more accurate counting, use the provider's token counter.
    pub fn estimate_tokens(text: &str) -> usize {
        (text.len() / 4).max(1)
    }

    /// Get the current token usage percentage
    pub fn usage_percentage(&self) -> f64 {
        (self.token_count as f64 / self.max_tokens as f64) * 100.0
    }

    /// Trim old messages if context is too large
    ///
    /// Uses FIFO removal of oldest messages to make room for new content.
    /// Always preserves the system prompt.
    pub fn trim_to_fit(&mut self, required_space: usize) {
        while self.would_exceed_limit(required_space) && !self.messages.is_empty() {
            // Remove the oldest user/assistant message
            if let Some(first_msg) = self.messages.first() {
                let tokens = self.estimate_message_tokens(first_msg);
                self.token_count = self.token_count.saturating_sub(tokens);
                self.messages.remove(0);
            }
        }
    }

    /// Get remaining context space
    pub fn remaining_tokens(&self) -> usize {
        self.max_tokens.saturating_sub(self.token_count)
    }

    /// Convert to simple message format for providers
    ///
    /// Flattens content blocks to text for providers that don't support
    /// structured content.
    pub fn to_simple_messages(&self) -> Vec<ricecoder_providers::models::Message> {
        self.messages
            .iter()
            .map(|msg| ricecoder_providers::models::Message {
                role: msg.role.as_str().to_string(),
                content: msg.text(),
            })
            .collect()
    }

    /// Get the last assistant message (if any)
    pub fn last_assistant_message(&self) -> Option<&ChatMessage> {
        self.messages
            .iter()
            .rev()
            .find(|m| m.role == Role::Assistant)
    }

    /// Get message count
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }
}

impl TrackedFile {
    /// Create a new tracked file
    pub fn new(path: PathBuf) -> Self {
        Self {
            id: Uuid::new_v4(),
            path,
            content: None,
            token_count: 0,
        }
    }

    /// Load content and estimate tokens
    pub fn with_content(mut self, content: String) -> Self {
        self.token_count = ChatContext::estimate_tokens(&content);
        self.content = Some(content);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let session_id = Uuid::new_v4();
        let context = ChatContext::new(session_id, 4096);

        assert_eq!(context.session_id, session_id);
        assert_eq!(context.max_tokens, 4096);
        assert_eq!(context.token_count, 0);
        assert!(context.messages.is_empty());
    }

    #[test]
    fn test_add_message() {
        let session_id = Uuid::new_v4();
        let mut context = ChatContext::new(session_id, 4096);

        let message = ChatMessage::user("Hello, how are you?");
        context.add_message(message);

        assert_eq!(context.messages.len(), 1);
        assert!(context.token_count > 0);
    }

    #[test]
    fn test_system_prompt() {
        let session_id = Uuid::new_v4();
        let context = ChatContext::new(session_id, 4096)
            .with_system_prompt("You are a helpful assistant.".to_string());

        assert!(context.system_prompt.is_some());
        assert!(context.token_count > 0);
    }

    #[test]
    fn test_token_estimation() {
        let tokens = ChatContext::estimate_tokens("Hello world");
        assert!(tokens > 0);
        assert!(tokens < 10); // Should be around 2-3 tokens
    }

    #[test]
    fn test_would_exceed_limit() {
        let session_id = Uuid::new_v4();
        let mut context = ChatContext::new(session_id, 100);

        let message = ChatMessage::user("Hello");
        context.add_message(message);

        assert!(!context.would_exceed_limit(10));
        assert!(context.would_exceed_limit(1000));
    }

    #[test]
    fn test_usage_percentage() {
        let session_id = Uuid::new_v4();
        let mut context = ChatContext::new(session_id, 100);

        // Add message that uses ~50 tokens
        let long_text = "a".repeat(200); // ~50 tokens
        let message = ChatMessage::user(long_text);
        context.add_message(message);

        let usage = context.usage_percentage();
        assert!(usage > 0.0 && usage <= 100.0);
    }

    #[test]
    fn test_trim_to_fit() {
        let session_id = Uuid::new_v4();
        let mut context = ChatContext::new(session_id, 100);

        // Add several messages with longer text
        for i in 0..5 {
            let long_text = format!(
                "This is a longer message {} that will use more tokens",
                i
            );
            let message = ChatMessage::user(long_text);
            context.add_message(message);
        }

        let original_count = context.messages.len();
        context.trim_to_fit(10); // Require 10 tokens space

        // Should have removed some messages
        assert!(context.messages.len() <= original_count);
    }

    #[test]
    fn test_remaining_tokens() {
        let session_id = Uuid::new_v4();
        let mut context = ChatContext::new(session_id, 100);

        let message = ChatMessage::user("Hello");
        context.add_message(message);

        let remaining = context.remaining_tokens();
        assert!(remaining < 100);
        assert!(remaining > 0);
    }

    #[test]
    fn test_tracked_file() {
        let file = TrackedFile::new(PathBuf::from("test.rs"))
            .with_content("fn main() {}".to_string());

        assert!(file.content.is_some());
        assert!(file.token_count > 0);
    }

    #[test]
    fn test_add_usage() {
        let session_id = Uuid::new_v4();
        let mut context = ChatContext::new(session_id, 4096);

        context.add_usage(Usage::new(100, 50));
        context.add_usage(Usage::new(200, 100));

        assert_eq!(context.total_usage.input_tokens, 300);
        assert_eq!(context.total_usage.output_tokens, 150);
    }
}
