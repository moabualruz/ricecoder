//! Core data models for sessions, messages, and background agents

use std::{collections::HashMap, path::PathBuf};

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
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
    /// Tenant/organization ID for multi-tenant isolation
    pub tenant_id: Option<String>,
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
            tenant_id: None,
        }
    }

    /// Create a new session context with tenant isolation
    pub fn with_tenant(
        provider: String,
        model: String,
        mode: SessionMode,
        tenant_id: String,
    ) -> Self {
        Self {
            project_path: None,
            provider,
            model,
            mode,
            files: Vec::new(),
            custom: HashMap::new(),
            tenant_id: Some(tenant_id),
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
        self.parts
            .push(MessagePart::ToolInvocation(ToolInvocationPart {
                tool_name: tool_name.into(),
                parameters,
                status: ToolStatus::Pending,
                started_at: None,
            }));
    }

    /// Add tool result to the message
    pub fn add_tool_result(
        &mut self,
        tool_name: impl Into<String>,
        result: Value,
        status: ToolStatus,
        duration_ms: u64,
    ) {
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
                    result.push_str(&format!(
                        "🔧 {}({})\n",
                        invocation.tool_name, invocation.parameters
                    ));
                }
                MessagePart::ToolResult(result_part) => {
                    result.push_str(&format!(
                        "✅ {}: {}\n",
                        result_part.tool_name, result_part.result
                    ));
                }
                MessagePart::FileReference(file_ref) => {
                    result.push_str(&format!("📁 {}\n", file_ref.path.display()));
                }
                MessagePart::Image(image) => {
                    result.push_str(&format!(
                        "🖼️ {} ({}x{})\n",
                        image.mime_type,
                        image.width.unwrap_or(0),
                        image.height.unwrap_or(0)
                    ));
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

/// Enterprise compliance event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceEventType {
    /// Session shared with external user
    SessionShared,
    /// Session accessed by unauthorized user
    UnauthorizedAccess,
    /// Data retention policy violation
    RetentionViolation,
    /// Encryption policy violation
    EncryptionViolation,
    /// Audit logging failure
    AuditFailure,
}

/// Enterprise compliance alert levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplianceAlertLevel {
    /// Informational event
    Info,
    /// Warning that requires attention
    Warning,
    /// Critical violation requiring immediate action
    Critical,
}

/// Enterprise compliance event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceEvent {
    /// Unique event ID
    pub id: String,
    /// Event type
    pub event_type: ComplianceEventType,
    /// Alert level
    pub alert_level: ComplianceAlertLevel,
    /// User ID associated with the event
    pub user_id: Option<String>,
    /// Session ID associated with the event
    pub session_id: Option<String>,
    /// Description of the event
    pub description: String,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// When the event occurred
    pub timestamp: DateTime<Utc>,
}

/// Enterprise session analytics data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseSessionAnalytics {
    /// Total number of sessions created
    pub total_sessions_created: u64,
    /// Total number of sessions shared
    pub total_sessions_shared: u64,
    /// Total number of session accesses
    pub total_session_accesses: u64,
    /// Sessions by tenant/organization
    pub sessions_by_tenant: HashMap<String, u64>,
    /// Sessions by data classification
    pub sessions_by_classification: HashMap<String, u64>,
    /// Average session duration
    pub average_session_duration_minutes: f64,
    /// Compliance events by type
    pub compliance_events_by_type: HashMap<String, u64>,
    /// Top users by session creation
    pub top_users_by_sessions: Vec<(String, u64)>,
    /// Session sharing trends over time
    pub sharing_trends: Vec<SharingTrendPoint>,
}

/// Data point for session sharing trends
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharingTrendPoint {
    /// Date of the data point
    pub date: DateTime<Utc>,
    /// Number of shares created on this date
    pub shares_created: u64,
    /// Number of shares accessed on this date
    pub shares_accessed: u64,
    /// Number of compliance events on this date
    pub compliance_events: u64,
}

/// GDPR/HIPAA compliance data retention policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRetentionPolicy {
    /// Maximum retention period for session data
    pub session_data_retention_days: u32,
    /// Maximum retention period for audit logs
    pub audit_log_retention_days: u32,
    /// Maximum retention period for backup data
    pub backup_retention_days: u32,
    /// Whether to enable automatic data deletion
    pub auto_delete_expired_data: bool,
    /// Data minimization settings
    pub data_minimization: DataMinimizationSettings,
}

/// Data minimization settings for GDPR compliance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataMinimizationSettings {
    /// Whether to anonymize IP addresses
    pub anonymize_ip_addresses: bool,
    /// Whether to limit collection of unnecessary data
    pub limit_unnecessary_collection: bool,
    /// Whether to enable data purging on user request
    pub enable_data_purging: bool,
    /// Whether to enable data export for portability
    pub enable_data_export: bool,
}

/// Data portability export format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataExportFormat {
    /// JSON format
    Json,
    /// XML format
    Xml,
    /// CSV format
    Csv,
    /// PDF report format
    Pdf,
}

/// Data export request for GDPR Article 20
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataExportRequest {
    /// User ID requesting export
    pub user_id: String,
    /// Export format
    pub format: DataExportFormat,
    /// Include audit logs
    pub include_audit_logs: bool,
    /// Include session data
    pub include_session_data: bool,
    /// Include sharing history
    pub include_sharing_history: bool,
    /// Requested at timestamp
    pub requested_at: DateTime<Utc>,
    /// Export completed at timestamp
    pub completed_at: Option<DateTime<Utc>>,
}

/// Right to erasure (GDPR Article 17) request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataErasureRequest {
    /// User ID requesting erasure
    pub user_id: String,
    /// Reason for erasure request
    pub reason: ErasureReason,
    /// Whether to erase all user data
    pub erase_all_data: bool,
    /// Specific data types to erase
    pub data_types_to_erase: Vec<DataType>,
    /// Requested at timestamp
    pub requested_at: DateTime<Utc>,
    /// Erasure completed at timestamp
    pub completed_at: Option<DateTime<Utc>>,
}

/// Reason for data erasure request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErasureReason {
    /// User withdrew consent
    ConsentWithdrawn,
    /// Data no longer needed
    NoLongerNeeded,
    /// Legal obligation
    LegalObligation,
    /// User requested deletion
    UserRequest,
}

/// Types of data that can be erased
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DataType {
    /// Session data
    Sessions,
    /// Audit logs
    AuditLogs,
    /// Sharing history
    SharingHistory,
    /// User preferences
    UserPreferences,
    /// All data types
    All,
}

/// Privacy-preserving session handling settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacySettings {
    /// Enable differential privacy for analytics
    pub enable_differential_privacy: bool,
    /// Enable data anonymization
    pub enable_data_anonymization: bool,
    /// Enable consent management
    pub enable_consent_management: bool,
    /// Enable privacy audit logging
    pub enable_privacy_auditing: bool,
}
