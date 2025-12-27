//! Chat types for tool calling and message handling
//!
//! These types bridge ricecoder's provider models with tool execution.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Content block in a chat message
///
/// Supports text content, tool use requests, and tool results.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    /// Text content
    #[serde(rename = "text")]
    Text {
        /// The text content
        text: String,
    },

    /// Tool use request from the model
    #[serde(rename = "tool_use")]
    ToolUse {
        /// Unique identifier for this tool use
        id: String,
        /// Name of the tool to invoke
        name: String,
        /// Input parameters for the tool
        input: Value,
    },

    /// Result of a tool execution
    #[serde(rename = "tool_result")]
    ToolResult {
        /// The tool_use_id this result corresponds to
        tool_use_id: String,
        /// The result content
        content: String,
        /// Whether the tool execution resulted in an error
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },
}

impl ContentBlock {
    /// Create a text content block
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text { text: text.into() }
    }

    /// Create a tool use content block
    pub fn tool_use(id: impl Into<String>, name: impl Into<String>, input: Value) -> Self {
        Self::ToolUse {
            id: id.into(),
            name: name.into(),
            input,
        }
    }

    /// Create a tool result content block
    pub fn tool_result(
        tool_use_id: impl Into<String>,
        content: impl Into<String>,
        is_error: bool,
    ) -> Self {
        Self::ToolResult {
            tool_use_id: tool_use_id.into(),
            content: content.into(),
            is_error: if is_error { Some(true) } else { None },
        }
    }

    /// Check if this is a text block
    pub fn is_text(&self) -> bool {
        matches!(self, Self::Text { .. })
    }

    /// Check if this is a tool use block
    pub fn is_tool_use(&self) -> bool {
        matches!(self, Self::ToolUse { .. })
    }

    /// Check if this is a tool result block
    pub fn is_tool_result(&self) -> bool {
        matches!(self, Self::ToolResult { .. })
    }

    /// Extract text content if this is a text block
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text { text } => Some(text),
            _ => None,
        }
    }

    /// Extract tool use info if this is a tool use block
    pub fn as_tool_use(&self) -> Option<(&str, &str, &Value)> {
        match self {
            Self::ToolUse { id, name, input } => Some((id, name, input)),
            _ => None,
        }
    }
}

/// Message role in a conversation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// User message
    User,
    /// Assistant (model) message
    Assistant,
    /// System prompt
    System,
}

impl Role {
    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::System => "system",
        }
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A message in a chat conversation with content blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Message role
    pub role: Role,
    /// Content blocks (can be text, tool use, or tool results)
    pub content: Vec<ContentBlock>,
}

impl ChatMessage {
    /// Create a user message with text content
    pub fn user(text: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: vec![ContentBlock::text(text)],
        }
    }

    /// Create an assistant message with text content
    pub fn assistant(text: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: vec![ContentBlock::text(text)],
        }
    }

    /// Create a system message
    pub fn system(text: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: vec![ContentBlock::text(text)],
        }
    }

    /// Create a message with multiple content blocks
    pub fn with_content(role: Role, content: Vec<ContentBlock>) -> Self {
        Self { role, content }
    }

    /// Extract all text from this message
    pub fn text(&self) -> String {
        self.content
            .iter()
            .filter_map(|c| c.as_text())
            .collect::<Vec<_>>()
            .join("")
    }

    /// Check if this message contains tool uses
    pub fn has_tool_use(&self) -> bool {
        self.content.iter().any(|c| c.is_tool_use())
    }

    /// Get all tool uses in this message
    pub fn tool_uses(&self) -> Vec<(&str, &str, &Value)> {
        self.content
            .iter()
            .filter_map(|c| c.as_tool_use())
            .collect()
    }
}

/// Tool definition for LLM function calling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// JSON schema for input parameters
    pub input_schema: Value,
}

/// Stop reason from the model
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    /// Model finished naturally
    EndTurn,
    /// Model wants to use a tool
    ToolUse,
    /// Maximum tokens reached
    MaxTokens,
    /// Stop sequence hit
    StopSequence,
}

/// Token usage for a request
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Usage {
    /// Input/prompt tokens
    pub input_tokens: usize,
    /// Output/completion tokens
    pub output_tokens: usize,
}

impl Usage {
    /// Create new usage info
    pub fn new(input_tokens: usize, output_tokens: usize) -> Self {
        Self {
            input_tokens,
            output_tokens,
        }
    }

    /// Total tokens used
    pub fn total(&self) -> usize {
        self.input_tokens + self.output_tokens
    }
}

/// Tool approval information for user confirmation
#[derive(Debug, Clone)]
pub struct ToolApprovalInfo {
    /// Tool name
    pub tool_name: String,
    /// Tool description
    pub tool_description: String,
    /// Tool input parameters
    pub tool_input: Value,
    /// Tool capabilities (for display)
    pub capabilities: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_block_text() {
        let block = ContentBlock::text("Hello");
        assert!(block.is_text());
        assert_eq!(block.as_text(), Some("Hello"));
    }

    #[test]
    fn test_content_block_tool_use() {
        let block = ContentBlock::tool_use("123", "read_file", serde_json::json!({"path": "test.txt"}));
        assert!(block.is_tool_use());
        let (id, name, input) = block.as_tool_use().unwrap();
        assert_eq!(id, "123");
        assert_eq!(name, "read_file");
        assert_eq!(input["path"], "test.txt");
    }

    #[test]
    fn test_chat_message_user() {
        let msg = ChatMessage::user("Hello");
        assert_eq!(msg.role, Role::User);
        assert_eq!(msg.text(), "Hello");
    }

    #[test]
    fn test_chat_message_with_tool_use() {
        let msg = ChatMessage::with_content(
            Role::Assistant,
            vec![
                ContentBlock::text("Let me read that file."),
                ContentBlock::tool_use("123", "read_file", serde_json::json!({"path": "test.txt"})),
            ],
        );
        assert!(msg.has_tool_use());
        assert_eq!(msg.tool_uses().len(), 1);
    }

    #[test]
    fn test_usage() {
        let usage = Usage::new(100, 50);
        assert_eq!(usage.total(), 150);
    }
}
