//! Core data models for sessions, messages, and background agents

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

/// Different types of content that can be part of a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePart {
    /// Plain text content
    Text(String),
    /// Reasoning or thinking content (can be collapsed)
    Reasoning(String),
    /// Tool invocation with parameters
    ToolInvocation(ToolInvocationPart),
    /// Result from a tool execution
    ToolResult(ToolResultPart),
    /// Reference to a file
    FileReference(FileReferencePart),
    /// Image content
    Image(ImagePart),
    /// Code block with syntax highlighting
    Code(CodePart),
    /// Error message
    Error(String),
}

/// Tool invocation part
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInvocationPart {
    /// Name of the tool being invoked
    pub tool_name: String,
    /// Parameters passed to the tool
    pub parameters: Value,
    /// Current status of the tool invocation
    pub status: ToolStatus,
    /// When the tool was started
    pub started_at: Option<DateTime<Utc>>,
}

/// Tool result part
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResultPart {
    /// Name of the tool that was executed
    pub tool_name: String,
    /// Result from the tool execution
    pub result: Value,
    /// Status of the tool execution
    pub status: ToolStatus,
    /// Duration of the tool execution in milliseconds
    pub duration_ms: u64,
    /// Error message if the tool failed
    pub error: Option<String>,
}

/// File reference part
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileReferencePart {
    /// Path to the file
    pub path: PathBuf,
    /// Size of the file in bytes
    pub size: u64,
    /// Content of the file (if small enough)
    pub content: Option<String>,
    /// Specific line range being referenced
    pub line_range: Option<(usize, usize)>,
}

/// Image part
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImagePart {
    /// Path to the image file (if saved locally)
    pub path: Option<PathBuf>,
    /// Raw image data
    pub data: Option<Vec<u8>>,
    /// MIME type of the image
    pub mime_type: String,
    /// Width of the image
    pub width: Option<u32>,
    /// Height of the image
    pub height: Option<u32>,
}

/// Code part
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodePart {
    /// Programming language for syntax highlighting
    pub language: String,
    /// Code content
    pub content: String,
    /// Optional filename
    pub filename: Option<String>,
    /// Whether to show line numbers
    pub line_numbers: bool,
}

/// Status of a tool execution
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ToolStatus {
    /// Tool is pending execution
    Pending,
    /// Tool is currently running
    Running,
    /// Tool completed successfully
    Complete,
    /// Tool execution failed
    Error,
    /// Tool execution was cancelled
    Cancelled,
}

/// Represents a session with its context, history, and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique identifier for the session
    pub id: String,
    /// Human-readable name for the session
    pub name: String,
    /// When the session was created
    pub created_at: DateTime<Utc>,
    /// When the session was last updated
    pub updated_at: DateTime<Utc>,
    /// Current status of the session
    pub status: SessionStatus,
    /// Session context (project, provider, model, etc.)
    pub context: SessionContext,
    /// Conversation history
    pub history: Vec<Message>,
    /// Background agents running in this session
    pub background_agents: Vec<BackgroundAgent>,
}

impl Session {
    /// Create a new session with default values
    pub fn new(name: String, context: SessionContext) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            created_at: now,
            updated_at: now,
            status: SessionStatus::Active,
            context,
            history: Vec::new(),
            background_agents: Vec::new(),
        }
    }
}

/// Status of a session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionStatus {
    /// Session is currently active
    Active,
    /// Session is paused
    Paused,
    /// Session is archived
    Archived,
}

/// Context for a session (project, provider, model, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    /// Path to the project directory
    pub project_path: Option<String>,
    /// AI provider (e.g., "openai", "anthropic")
    pub provider: String,
    /// Model name (e.g., "gpt-4", "claude-3")
    pub model: String,
    /// Session mode (Chat, Code, Vibe)
    pub mode: SessionMode,
    /// Files included in the session context
    pub files: Vec<String>,
    /// Custom context data
    pub custom: HashMap<String, serde_json::Value>,
}

impl SessionContext {
    /// Create a new session context
    pub fn new(provider: String, model: String, mode: SessionMode) -> Self {
        Self {
            project_path: None,
            provider,
            model,
            mode,
            files: Vec::new(),
            custom: HashMap::new(),
        }
    }
}

/// Session mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionMode {
    /// Chat mode for conversations
    Chat,
    /// Code mode for code generation and analysis
    Code,
    /// Vibe mode for creative tasks
    Vibe,
}

/// A message in the conversation history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique identifier for the message
    pub id: String,
    /// Role of the message sender
    pub role: MessageRole,
    /// Message content parts (replaces simple content field)
    pub parts: Vec<MessagePart>,
    /// When the message was created
    pub timestamp: DateTime<Utc>,
    /// Additional metadata about the message
    pub metadata: MessageMetadata,
}

impl Message {
    /// Create a new message with text content (backwards compatibility)
    pub fn new(role: MessageRole, content: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            role,
            parts: vec![MessagePart::Text(content)],
            timestamp: Utc::now(),
            metadata: MessageMetadata::default(),
        }
    }

    /// Create a new empty message
    pub fn new_empty(role: MessageRole) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            role,
            parts: Vec::new(),
            timestamp: Utc::now(),
            metadata: MessageMetadata::default(),
        }
    }

    /// Add text content to the message
    pub fn add_text(&mut self, text: impl Into<String>) {
        self.parts.push(MessagePart::Text(text.into()));
    }

    /// Add reasoning content to the message
    pub fn add_reasoning(&mut self, reasoning: impl Into<String>) {
        self.parts.push(MessagePart::Reasoning(reasoning.into()));
    }

    /// Add code content to the message
    pub fn add_code(&mut self, language: impl Into<String>, content: impl Into<String>) {
        self.parts.push(MessagePart::Code(CodePart {
            language: language.into(),
            content: content.into(),
            filename: None,
            line_numbers: true,
        }));
    }

    /// Add tool invocation to the message
    pub fn add_tool_invocation(&mut self, tool_name: impl Into<String>, parameters: Value) {
        self.parts.push(MessagePart::ToolInvocation(ToolInvocationPart {
            tool_name: tool_name.into(),
            parameters,
            status: ToolStatus::Pending,
            started_at: None,
        }));
    }

    /// Add tool result to the message
    pub fn add_tool_result(&mut self, tool_name: impl Into<String>, result: Value, status: ToolStatus, duration_ms: u64) {
        self.parts.push(MessagePart::ToolResult(ToolResultPart {
            tool_name: tool_name.into(),
            result,
            status,
            duration_ms,
            error: None,
        }));
    }

    /// Add error content to the message
    pub fn add_error(&mut self, error: impl Into<String>) {
        self.parts.push(MessagePart::Error(error.into()));
    }

    /// Get the primary text content (first text part, for backwards compatibility)
    pub fn content(&self) -> String {
        for part in &self.parts {
            if let MessagePart::Text(text) = part {
                return text.clone();
            }
        }
        String::new()
    }

    /// Get all text content concatenated
    pub fn full_content(&self) -> String {
        let mut result = String::new();
        for part in &self.parts {
            match part {
                MessagePart::Text(text) => {
                    result.push_str(text);
                    result.push('\n');
                }
                MessagePart::Code(code) => {
                    result.push_str(&format!("```{}\n{}\n```\n", code.language, code.content));
                }
                MessagePart::Reasoning(reasoning) => {
                    result.push_str(&format!("💭 {}\n", reasoning));
                }
                MessagePart::Error(error) => {
                    result.push_str(&format!("❌ {}\n", error));
                }
                MessagePart::ToolInvocation(invocation) => {
                    result.push_str(&format!("🔧 {}({})\n", invocation.tool_name, invocation.parameters));
                }
                MessagePart::ToolResult(result_part) => {
                    result.push_str(&format!("✅ {}: {}\n", result_part.tool_name, result_part.result));
                }
                MessagePart::FileReference(file_ref) => {
                    result.push_str(&format!("📁 {}\n", file_ref.path.display()));
                }
                MessagePart::Image(image) => {
                    result.push_str(&format!("🖼️ {} ({}x{})\n", image.mime_type, image.width.unwrap_or(0), image.height.unwrap_or(0)));
                }
            }
        }
        result.trim_end().to_string()
    }
}

/// Role of a message sender
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageRole {
    /// User message
    User,
    /// Assistant message
    Assistant,
    /// System message
    System,
}

impl std::fmt::Display for MessageRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageRole::User => write!(f, "user"),
            MessageRole::Assistant => write!(f, "assistant"),
            MessageRole::System => write!(f, "system"),
        }
    }
}

/// Metadata about a message
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessageMetadata {
    /// Number of tokens in the message
    pub tokens: Option<usize>,
    /// Model used to generate the message
    pub model: Option<String>,
    /// Duration of message generation
    pub duration: Option<Duration>,
}

/// A background agent running in a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundAgent {
    /// Unique identifier for the agent
    pub id: String,
    /// Type of agent (e.g., "code_review", "diff_analysis")
    pub agent_type: String,
    /// Current status of the agent
    pub status: AgentStatus,
    /// Task being executed
    pub task: Option<String>,
    /// When the agent was started
    pub started_at: DateTime<Utc>,
    /// When the agent completed (if finished)
    pub completed_at: Option<DateTime<Utc>>,
}

impl BackgroundAgent {
    /// Create a new background agent
    pub fn new(agent_type: String, task: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            agent_type,
            status: AgentStatus::Running,
            task,
            started_at: Utc::now(),
            completed_at: None,
        }
    }
}

/// Status of a background agent
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    /// Agent is currently running
    Running,
    /// Agent has completed successfully
    Completed,
    /// Agent failed
    Failed,
    /// Agent was cancelled
    Cancelled,
}
