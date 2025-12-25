//! Task tool for background agent execution
//!
//! This module provides the Task tool that enables agents to spawn subagents for complex tasks.
//! It matches the functionality of task.ts from the reference implementation.

use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::{
    error::{AgentError, Result},
    tool_registry::{ToolInvoker, ToolMetadata},
};

/// Parameters for task tool invocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskParams {
    /// Short description (3-5 words)
    pub description: String,
    /// Task instructions for the subagent
    pub prompt: String,
    /// Agent identifier to execute
    pub subagent_type: String,
    /// Existing session ID to continue (optional)
    pub session_id: Option<String>,
    /// Command that triggered this task (optional)
    pub command: Option<String>,
}

/// Progress update from a running task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProgress {
    /// Tool part ID
    pub id: String,
    /// Tool name
    pub tool: String,
    /// Execution status
    pub status: String,
    /// Optional title when completed
    pub title: Option<String>,
}

/// Result of task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// Task description
    pub title: String,
    /// Session ID for the subagent execution
    pub session_id: String,
    /// Progress summary
    pub summary: Vec<TaskProgress>,
    /// Text output from the subagent
    pub output: String,
}

/// Subagent type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentType {
    /// Unique name identifier
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Execution mode (subagent, primary, all)
    pub mode: String,
    /// Tool permissions for this agent
    pub tools: HashMap<String, bool>,
    /// Optional model override
    pub model: Option<ModelConfig>,
}

/// Model configuration for subagent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Model ID (e.g., "gpt-4")
    pub model_id: String,
    /// Provider ID (e.g., "openai")
    pub provider_id: String,
}

/// Task execution context
pub struct TaskExecutionContext {
    /// Parent session ID
    pub session_id: String,
    /// Parent message ID
    pub message_id: String,
    /// Current model configuration
    pub model: ModelConfig,
    /// Metadata update callback
    pub metadata_callback: Arc<dyn Fn(Value) + Send + Sync>,
    /// Abort signal receiver
    pub abort_rx: Option<tokio::sync::watch::Receiver<bool>>,
}

/// Task tool for background agent execution
pub struct TaskTool {
    /// Available subagents
    subagents: Arc<RwLock<HashMap<String, SubagentType>>>,
    /// Running tasks
    tasks: Arc<RwLock<HashMap<String, TaskHandle>>>,
    /// Session manager
    session_manager: Arc<RwLock<Option<Arc<dyn SessionManager + Send + Sync>>>>,
}

/// Handle to a running task
struct TaskHandle {
    /// Task ID
    id: String,
    /// Session ID for this task
    session_id: String,
    /// Abort sender
    abort_tx: tokio::sync::watch::Sender<bool>,
    /// Join handle
    handle: tokio::task::JoinHandle<Result<TaskResult>>,
}

/// Trait for session management
#[async_trait]
pub trait SessionManager {
    /// Create a new child session
    async fn create_child_session(
        &self,
        parent_id: &str,
        title: &str,
    ) -> Result<String>;

    /// Get or create session
    async fn get_or_create_session(
        &self,
        session_id: Option<&str>,
        parent_id: &str,
        title: &str,
    ) -> Result<String>;

    /// Execute prompt in session
    async fn execute_prompt(
        &self,
        session_id: &str,
        message_id: &str,
        prompt: &str,
        agent: &str,
        model: &ModelConfig,
        tools: &HashMap<String, bool>,
        progress_tx: mpsc::UnboundedSender<TaskProgress>,
        abort_rx: tokio::sync::watch::Receiver<bool>,
    ) -> Result<String>;

    /// Get session messages
    async fn get_session_messages(&self, session_id: &str) -> Result<Vec<Value>>;
}

impl TaskTool {
    /// Create a new task tool with default subagents
    pub fn new() -> Self {
        let mut subagents = HashMap::new();

        // Register default subagents
        subagents.insert(
            "general".to_string(),
            SubagentType {
                name: "general".to_string(),
                description: "General-purpose agent for complex multi-step tasks".to_string(),
                mode: "subagent".to_string(),
                tools: HashMap::new(),
                model: None,
            },
        );

        subagents.insert(
            "explore".to_string(),
            SubagentType {
                name: "explore".to_string(),
                description: "Fast agent specialized for exploring codebases".to_string(),
                mode: "subagent".to_string(),
                tools: HashMap::new(),
                model: None,
            },
        );

        subagents.insert(
            "librarian".to_string(),
            SubagentType {
                name: "librarian".to_string(),
                description: "Specialized agent for searching remote repos and documentation".to_string(),
                mode: "subagent".to_string(),
                tools: HashMap::new(),
                model: None,
            },
        );

        subagents.insert(
            "oracle".to_string(),
            SubagentType {
                name: "oracle".to_string(),
                description: "Expert technical advisor with deep reasoning".to_string(),
                mode: "subagent".to_string(),
                tools: HashMap::new(),
                model: None,
            },
        );

        Self {
            subagents: Arc::new(RwLock::new(subagents)),
            tasks: Arc::new(RwLock::new(HashMap::new())),
            session_manager: Arc::new(RwLock::new(None)),
        }
    }

    /// Register a new subagent type
    pub async fn register_subagent(&self, subagent: SubagentType) {
        let mut agents = self.subagents.write().await;
        agents.insert(subagent.name.clone(), subagent);
    }

    /// Get list of available subagents
    pub async fn list_subagents(&self) -> Vec<SubagentType> {
        let agents = self.subagents.read().await;
        agents
            .values()
            .filter(|a| a.mode != "primary")
            .cloned()
            .collect()
    }

    /// Set session manager
    pub async fn set_session_manager(&self, manager: Arc<dyn SessionManager + Send + Sync>) {
        let mut mgr = self.session_manager.write().await;
        *mgr = Some(manager);
    }

    /// Execute task with given parameters and context
    pub async fn execute_task(
        &self,
        params: TaskParams,
        context: TaskExecutionContext,
    ) -> Result<TaskResult> {
        info!(
            description = %params.description,
            subagent = %params.subagent_type,
            "Starting task execution"
        );

        // Validate subagent type
        let subagent = {
            let agents = self.subagents.read().await;
            agents
                .get(&params.subagent_type)
                .cloned()
                .ok_or_else(|| {
                    AgentError::ValidationError(format!(
                        "Unknown agent type: {}",
                        params.subagent_type
                    ))
                })?
        };

        // Get session manager
        let session_mgr = {
            let mgr = self.session_manager.read().await;
            mgr.clone().ok_or_else(|| {
                AgentError::config_error("No session manager configured")
            })?
        };

        // Create or get session
        let session_id = session_mgr
            .get_or_create_session(
                params.session_id.as_deref(),
                &context.session_id,
                &format!("{} (@{} subagent)", params.description, subagent.name),
            )
            .await?;

        debug!(
            session_id = %session_id,
            parent_session = %context.session_id,
            "Session created"
        );

        // Build tool permissions (disable task recursion, todowrite, todoread)
        let mut tools = HashMap::new();
        tools.insert("task".to_string(), false);
        tools.insert("todowrite".to_string(), false);
        tools.insert("todoread".to_string(), false);

        // Merge agent-specific tool overrides
        for (tool, enabled) in &subagent.tools {
            tools.insert(tool.clone(), *enabled);
        }

        // Determine model (agent override or inherit from parent)
        let model = subagent.model.as_ref().unwrap_or(&context.model).clone();

        // Create progress channel
        let (progress_tx, mut progress_rx) = mpsc::unbounded_channel::<TaskProgress>();

        // Create abort channel
        let (abort_tx, abort_rx) = tokio::sync::watch::channel(false);

        // Set up abort propagation if provided
        if let Some(mut parent_abort) = context.abort_rx {
            let abort_tx_clone = abort_tx.clone();
            tokio::spawn(async move {
                while parent_abort.changed().await.is_ok() {
                    if *parent_abort.borrow() {
                        let _ = abort_tx_clone.send(true);
                        break;
                    }
                }
            });
        }

        // Track progress updates
        let session_id_clone = session_id.clone();
        let metadata_callback = Arc::clone(&context.metadata_callback);
        let progress_handle = tokio::spawn(async move {
            let mut summary: Vec<TaskProgress> = Vec::new();

            while let Some(progress) = progress_rx.recv().await {
                debug!(tool = %progress.tool, status = %progress.status, "Progress update");

                // Update summary
                if let Some(existing) = summary.iter_mut().find(|p| p.id == progress.id) {
                    *existing = progress.clone();
                } else {
                    summary.push(progress.clone());
                }

                // Call metadata callback
                metadata_callback(json!({
                    "title": "Task execution",
                    "metadata": {
                        "sessionId": session_id_clone,
                        "summary": summary.clone()
                    }
                }));
            }
        });

        // Execute prompt in session
        let output = session_mgr
            .execute_prompt(
                &session_id,
                &context.message_id,
                &params.prompt,
                &subagent.name,
                &model,
                &tools,
                progress_tx,
                abort_rx,
            )
            .await?;

        // Wait for progress tracking to finish
        drop(abort_tx); // Signal completion
        let _ = progress_handle.await;

        // Get final messages for summary
        let messages = session_mgr.get_session_messages(&session_id).await?;

        let empty_vec: Vec<Value> = vec![];
        let summary: Vec<TaskProgress> = messages
            .iter()
            .filter(|msg| msg["role"] == "assistant")
            .flat_map(|msg| msg["parts"].as_array().unwrap_or(&empty_vec))
            .filter(|part| part["type"] == "tool")
            .map(|part| TaskProgress {
                id: part["id"].as_str().unwrap_or("").to_string(),
                tool: part["tool"].as_str().unwrap_or("").to_string(),
                status: part["state"]["status"]
                    .as_str()
                    .unwrap_or("unknown")
                    .to_string(),
                title: part["state"]["title"].as_str().map(|s| s.to_string()),
            })
            .collect();

        info!(
            session_id = %session_id,
            tool_count = summary.len(),
            "Task execution completed"
        );

        // Build result with embedded session ID
        let output_with_metadata = format!(
            "{}\n\n<task_metadata>\nsession_id: {}\n</task_metadata>",
            output, session_id
        );

        Ok(TaskResult {
            title: params.description,
            session_id,
            summary,
            output: output_with_metadata,
        })
    }

    /// Cancel a running task
    pub async fn cancel_task(&self, task_id: &str) -> Result<()> {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.remove(task_id) {
            let _ = task.abort_tx.send(true);
            task.handle.abort();
            Ok(())
        } else {
            Err(AgentError::NotFound(format!("Task not found: {}", task_id)))
        }
    }

    /// Get task status
    pub async fn get_task_status(&self, task_id: &str) -> Result<String> {
        let tasks = self.tasks.read().await;
        if tasks.contains_key(task_id) {
            Ok("running".to_string())
        } else {
            Err(AgentError::NotFound(format!("Task not found: {}", task_id)))
        }
    }
}

impl Default for TaskTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolInvoker for TaskTool {
    async fn invoke(&self, input: Value) -> std::result::Result<Value, String> {
        // Parse parameters
        let params: TaskParams =
            serde_json::from_value(input).map_err(|e| format!("Invalid parameters: {}", e))?;

        // Create minimal execution context (in production, this would come from the tool framework)
        let context = TaskExecutionContext {
            session_id: Uuid::new_v4().to_string(),
            message_id: Uuid::new_v4().to_string(),
            model: ModelConfig {
                model_id: "gpt-4".to_string(),
                provider_id: "openai".to_string(),
            },
            metadata_callback: Arc::new(|_| {}),
            abort_rx: None,
        };

        // Execute task
        let result = self
            .execute_task(params, context)
            .await
            .map_err(|e| format!("Task execution failed: {}", e))?;

        // Return result
        Ok(json!({
            "title": result.title,
            "metadata": {
                "sessionId": result.session_id,
                "summary": result.summary
            },
            "output": result.output
        }))
    }

    fn metadata(&self) -> ToolMetadata {
        // Generate dynamic description with available subagents
        let subagents_desc = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let agents = self.list_subagents().await;
                agents
                    .iter()
                    .map(|a| format!("- {}: {}", a.name, a.description))
                    .collect::<Vec<_>>()
                    .join("\n")
            })
        });

        let description = format!(
            r#"Launch a new agent to handle complex, multi-step tasks autonomously.

Available agent types and the tools they have access to:
{}

When using the Task tool, you must specify a subagent_type parameter to select which agent type to use.

Usage notes:
1. Launch multiple agents concurrently whenever possible, to maximize performance
2. When the agent is done, it will return a single message back to you
3. Each agent invocation is stateless unless you provide a session_id
4. Your prompt should contain a highly detailed task description for the agent to perform autonomously
"#,
            subagents_desc
        );

        ToolMetadata {
            id: "task".to_string(),
            name: "Task".to_string(),
            description,
            input_schema: json!({
                "type": "object",
                "properties": {
                    "description": {
                        "type": "string",
                        "description": "A short (3-5 words) description of the task"
                    },
                    "prompt": {
                        "type": "string",
                        "description": "The task for the agent to perform"
                    },
                    "subagent_type": {
                        "type": "string",
                        "description": "The type of specialized agent to use for this task"
                    },
                    "session_id": {
                        "type": "string",
                        "description": "Existing Task session to continue (optional)"
                    },
                    "command": {
                        "type": "string",
                        "description": "The command that triggered this task (optional)"
                    }
                },
                "required": ["description", "prompt", "subagent_type"]
            }),
            output_schema: json!({
                "type": "object",
                "properties": {
                    "title": { "type": "string" },
                    "metadata": {
                        "type": "object",
                        "properties": {
                            "sessionId": { "type": "string" },
                            "summary": {
                                "type": "array",
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "id": { "type": "string" },
                                        "tool": { "type": "string" },
                                        "status": { "type": "string" },
                                        "title": { "type": "string" }
                                    }
                                }
                            }
                        }
                    },
                    "output": { "type": "string" }
                }
            }),
            available: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_task_tool() {
        let tool = TaskTool::new();
        let subagents = tool.list_subagents().await;

        assert!(!subagents.is_empty());
        assert!(subagents.iter().any(|s| s.name == "general"));
        assert!(subagents.iter().any(|s| s.name == "explore"));
    }

    #[tokio::test]
    async fn test_register_subagent() {
        let tool = TaskTool::new();
        let custom_agent = SubagentType {
            name: "custom".to_string(),
            description: "Custom agent".to_string(),
            mode: "subagent".to_string(),
            tools: HashMap::new(),
            model: None,
        };

        tool.register_subagent(custom_agent).await;
        let subagents = tool.list_subagents().await;

        assert!(subagents.iter().any(|s| s.name == "custom"));
    }

    #[test]
    fn test_tool_metadata() {
        let tool = TaskTool::new();
        let metadata = tool.metadata();

        assert_eq!(metadata.id, "task");
        assert_eq!(metadata.name, "Task");
        assert!(metadata.available);
        assert!(metadata.description.contains("Launch a new agent"));
    }
}
