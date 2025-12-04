//! Core data models for sessions, messages, and background agents

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

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
    /// Message content
    pub content: String,
    /// When the message was created
    pub timestamp: DateTime<Utc>,
    /// Additional metadata about the message
    pub metadata: MessageMetadata,
}

impl Message {
    /// Create a new message
    pub fn new(role: MessageRole, content: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            role,
            content,
            timestamp: Utc::now(),
            metadata: MessageMetadata::default(),
        }
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
