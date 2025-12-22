//! Core data types for the hooks system
//!
//! This module defines the core data structures for the hooks system, including hooks,
//! actions, events, and execution results.
//!
//! # Examples
//!
//! Creating a simple command hook:
//!
//! ```ignore
//! use ricecoder_hooks::*;
//!
//! let hook = Hook {
//!     id: "format-on-save".to_string(),
//!     name: "Format on Save".to_string(),
//!     description: Some("Format code when file is saved".to_string()),
//!     event: "file_saved".to_string(),
//!     action: Action::Command(CommandAction {
//!         command: "prettier".to_string(),
//!         args: vec!["--write".to_string(), "{{file_path}}".to_string()],
//!         timeout_ms: Some(5000),
//!         capture_output: true,
//!     }),
//!     enabled: true,
//!     tags: vec!["formatting".to_string()],
//!     metadata: serde_json::json!({}),
//!     condition: None,
//! };
//! ```

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// A hook that triggers on specific events
///
/// Hooks are the core building blocks of the hooks system. Each hook is associated with
/// an event type and defines an action to execute when that event occurs.
///
/// # Fields
///
/// * `id` - Unique identifier for the hook (typically a UUID)
/// * `name` - Human-readable name for the hook
/// * `description` - Optional description of what the hook does
/// * `event` - Event type that triggers this hook (e.g., "file_saved", "test_passed")
/// * `action` - Action to execute when the hook is triggered
/// * `enabled` - Whether the hook is currently enabled
/// * `tags` - Tags for categorizing and filtering hooks
/// * `metadata` - Additional metadata stored as JSON
/// * `condition` - Optional condition that must be met for the hook to execute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hook {
    /// Unique identifier for the hook
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Optional description
    pub description: Option<String>,

    /// Event that triggers this hook
    pub event: String,

    /// Action to execute
    pub action: Action,

    /// Whether the hook is enabled
    pub enabled: bool,

    /// Tags for categorizing hooks
    pub tags: Vec<String>,

    /// Additional metadata
    pub metadata: serde_json::Value,

    /// Optional condition for execution
    pub condition: Option<Condition>,
}

/// Action to execute when a hook is triggered
///
/// Actions define what happens when a hook is triggered. There are four types of actions:
///
/// * `Command` - Execute a shell command
/// * `ToolCall` - Call a ricecoder tool with parameters
/// * `AiPrompt` - Send a prompt to an AI assistant
/// * `Chain` - Execute multiple hooks in sequence
///
/// # Examples
///
/// Command action:
/// ```ignore
/// Action::Command(CommandAction {
///     command: "echo".to_string(),
///     args: vec!["Hello, World!".to_string()],
///     timeout_ms: Some(5000),
///     capture_output: true,
/// })
/// ```
///
/// Tool call action:
/// ```ignore
/// Action::ToolCall(ToolCallAction {
///     tool_name: "formatter".to_string(),
///     tool_path: "/usr/local/bin/prettier".to_string(),
///     parameters: ParameterBindings {
///         bindings: vec![
///             ("file_path".to_string(), ParameterValue::Variable("file_path".to_string())),
///         ].into_iter().collect(),
///     },
///     timeout_ms: Some(5000),
/// })
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Action {
    /// Execute a shell command
    #[serde(rename = "command")]
    Command(CommandAction),

    /// Call a tool with parameters
    #[serde(rename = "tool_call")]
    ToolCall(ToolCallAction),

    /// Send a prompt to an AI assistant
    #[serde(rename = "ai_prompt")]
    AiPrompt(AiPromptAction),

    /// Chain multiple hooks
    #[serde(rename = "chain")]
    Chain(ChainAction),
}

/// Command action configuration
///
/// Executes a shell command when the hook is triggered. Supports variable substitution
/// in command arguments using `{{variable_name}}` syntax.
///
/// # Examples
///
/// ```ignore
/// CommandAction {
///     command: "prettier".to_string(),
///     args: vec!["--write".to_string(), "{{file_path}}".to_string()],
///     timeout_ms: Some(5000),
///     capture_output: true,
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandAction {
    /// Command to execute
    pub command: String,

    /// Command arguments (supports variable substitution)
    pub args: Vec<String>,

    /// Optional timeout in milliseconds
    pub timeout_ms: Option<u64>,

    /// Whether to capture output
    pub capture_output: bool,
}

/// Tool call action configuration
///
/// Calls a ricecoder tool with parameters bound from the event context. The tool path
/// can be an absolute path, relative path, or internal handler reference.
///
/// # Examples
///
/// ```ignore
/// ToolCallAction {
///     tool_name: "code_formatter".to_string(),
///     tool_path: "/usr/local/bin/prettier".to_string(),
///     parameters: ParameterBindings {
///         bindings: vec![
///             ("file_path".to_string(), ParameterValue::Variable("file_path".to_string())),
///             ("format".to_string(), ParameterValue::Literal(json!("json"))),
///         ].into_iter().collect(),
///     },
///     timeout_ms: Some(5000),
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallAction {
    /// Name of the tool
    pub tool_name: String,

    /// Path to the tool (absolute, relative, or internal handler)
    pub tool_path: String,

    /// Parameter bindings from event context
    pub parameters: ParameterBindings,

    /// Optional timeout in milliseconds
    pub timeout_ms: Option<u64>,
}

/// Parameter bindings for tool calls
///
/// Maps parameter names to values that can be either literals or references to
/// event context variables.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterBindings {
    /// Map of parameter names to values
    pub bindings: HashMap<String, ParameterValue>,
}

/// Parameter value (literal or variable reference)
///
/// Parameters can be either literal values or references to event context variables.
/// Variable references use the format `{{variable_name}}` and are substituted at
/// execution time.
///
/// # Examples
///
/// Literal value:
/// ```ignore
/// ParameterValue::Literal(json!("json"))
/// ```
///
/// Variable reference:
/// ```ignore
/// ParameterValue::Variable("file_path".to_string())
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ParameterValue {
    /// Literal value
    Literal(serde_json::Value),

    /// Variable reference (substituted from event context)
    Variable(String),
}

/// AI prompt action configuration
///
/// Sends a prompt to an AI assistant with variables substituted from the event context.
/// Supports streaming responses and custom model configuration.
///
/// # Examples
///
/// ```ignore
/// AiPromptAction {
///     prompt_template: "Format the following code:\n{{code}}".to_string(),
///     variables: vec![
///         ("code".to_string(), "file_content".to_string()),
///     ].into_iter().collect(),
///     model: Some("gpt-4".to_string()),
///     temperature: Some(0.7),
///     max_tokens: Some(2000),
///     stream: true,
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiPromptAction {
    /// Prompt template with variable placeholders
    pub prompt_template: String,

    /// Variables for substitution (maps placeholder to context key)
    pub variables: HashMap<String, String>,

    /// Optional model name
    pub model: Option<String>,

    /// Optional temperature (0.0 to 2.0)
    pub temperature: Option<f32>,

    /// Optional max tokens for response
    pub max_tokens: Option<u32>,

    /// Whether to stream responses
    pub stream: bool,
}

/// Chain action configuration
///
/// Executes multiple hooks in sequence. Optionally passes the output of one hook
/// as context to the next hook in the chain.
///
/// # Examples
///
/// ```ignore
/// ChainAction {
///     hook_ids: vec![
///         "analyze-code".to_string(),
///         "generate-suggestions".to_string(),
///         "apply-suggestions".to_string(),
///     ],
///     pass_output: true,
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainAction {
    /// IDs of hooks to execute in sequence
    pub hook_ids: Vec<String>,

    /// Whether to pass output between hooks
    pub pass_output: bool,
}

/// Condition for hook execution
///
/// Optional condition that must be met for a hook to execute. Conditions are evaluated
/// against the event context and can filter hooks based on context values.
///
/// # Examples
///
/// ```ignore
/// Condition {
///     expression: "file_path.ends_with('.rs')".to_string(),
///     context_keys: vec!["file_path".to_string()],
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    /// Condition expression (evaluated against event context)
    pub expression: String,

    /// Context keys used in the expression
    pub context_keys: Vec<String>,
}

/// Event that triggers hooks
///
/// Events are emitted by the system when something happens (e.g., file saved, test passed).
/// Each event has a type, context, and timestamp.
///
/// # Examples
///
/// ```ignore
/// Event {
///     event_type: "file_saved".to_string(),
///     context: EventContext {
///         data: json!({
///             "file_path": "/path/to/file.rs",
///             "size": 1024,
///         }),
///         metadata: json!({
///             "user": "alice",
///             "project": "my-project",
///         }),
///     },
///     timestamp: "2024-01-01T12:00:00Z".to_string(),
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Event type (e.g., "file_saved", "test_passed")
    pub event_type: String,

    /// Event context with data and metadata
    pub context: EventContext,

    /// Event timestamp (ISO 8601 format)
    pub timestamp: String,
}

/// Context passed to hooks
///
/// Contains the data and metadata associated with an event. This context is passed
/// to hooks and can be used for variable substitution in actions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventContext {
    /// Event data (varies by event type)
    pub data: serde_json::Value,

    /// Event metadata (user, project, etc.)
    pub metadata: serde_json::Value,
}

/// Result of hook execution
///
/// Contains the result of executing a hook, including status, output, and any errors.
///
/// # Examples
///
/// ```ignore
/// HookResult {
///     hook_id: "format-on-save".to_string(),
///     status: HookStatus::Success,
///     output: Some("Formatted 5 files".to_string()),
///     error: None,
///     duration_ms: 1234,
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookResult {
    /// Hook ID
    pub hook_id: String,

    /// Execution status
    pub status: HookStatus,

    /// Optional output from hook execution
    pub output: Option<String>,

    /// Optional error message
    pub error: Option<String>,

    /// Duration in milliseconds
    pub duration_ms: u64,
}

/// Status of hook execution
///
/// Indicates the outcome of executing a hook.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HookStatus {
    /// Hook executed successfully
    Success,

    /// Hook execution failed
    Failed,

    /// Hook execution timed out
    Timeout,

    /// Hook was skipped (condition not met or hook disabled)
    Skipped,
}
