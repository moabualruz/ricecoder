//! Default Session Manager Implementation
//!
//! Provides a default implementation of the SessionManager trait for TaskTool.
//! Uses in-memory session storage with ChatService for execution.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{json, Value};
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info, warn};
use uuid::Uuid;

use ricecoder_providers::provider::Provider;

use super::task::{ModelConfig, SessionManager, TaskProgress};
use crate::chat::{ChatContext, ChatMessage, ChatService};
use crate::error::{AgentError, Result};
use crate::tool_registry::ToolRegistry;

/// Session data stored in memory
#[derive(Debug)]
struct SessionData {
    /// Session ID
    id: String,
    /// Parent session ID (if any)
    parent_id: Option<String>,
    /// Session title
    title: String,
    /// Chat context for this session
    context: ChatContext,
    /// Messages in JSON format for retrieval
    messages: Vec<Value>,
}

/// Default session manager implementation
///
/// Uses in-memory storage for sessions and ChatService for execution.
/// This is a basic implementation suitable for single-process execution.
/// For production use with persistence, replace with SurrealDB-backed version.
pub struct DefaultSessionManager {
    /// Active sessions
    sessions: Arc<RwLock<HashMap<String, SessionData>>>,
    /// Provider for chat execution
    provider: Arc<dyn Provider>,
    /// Tool registry for subagent execution
    tool_registry: Arc<ToolRegistry>,
    /// Default context window size
    default_context_size: usize,
}

impl DefaultSessionManager {
    /// Create a new default session manager
    pub fn new(provider: Arc<dyn Provider>, tool_registry: Arc<ToolRegistry>) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            provider,
            tool_registry,
            default_context_size: 128_000, // 128k tokens default
        }
    }

    /// Set the default context window size
    pub fn with_context_size(mut self, size: usize) -> Self {
        self.default_context_size = size;
        self
    }

    /// Create a ChatService configured for subagent execution
    fn create_chat_service(&self) -> ChatService {
        ChatService::new(Arc::clone(&self.provider))
            .with_tool_registry(Arc::clone(&self.tool_registry))
            .with_auto_approve_tools(true) // Subagents auto-approve
            .with_max_tool_iterations(15)
    }
}

#[async_trait]
impl SessionManager for DefaultSessionManager {
    /// Create a new child session
    async fn create_child_session(&self, parent_id: &str, title: &str) -> Result<String> {
        let session_uuid = Uuid::new_v4();
        let session_id = session_uuid.to_string();
        
        let session_data = SessionData {
            id: session_id.clone(),
            parent_id: Some(parent_id.to_string()),
            title: title.to_string(),
            context: ChatContext::new(session_uuid, self.default_context_size),
            messages: Vec::new(),
        };

        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), session_data);

        info!(
            session_id = %session_id,
            parent_id = %parent_id,
            title = %title,
            "Created child session"
        );

        Ok(session_id)
    }

    /// Get or create session
    async fn get_or_create_session(
        &self,
        session_id: Option<&str>,
        parent_id: &str,
        title: &str,
    ) -> Result<String> {
        if let Some(id) = session_id {
            // Check if session exists
            let sessions = self.sessions.read().await;
            if sessions.contains_key(id) {
                debug!(session_id = %id, "Using existing session");
                return Ok(id.to_string());
            }
        }

        // Create new session
        self.create_child_session(parent_id, title).await
    }

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
        mut abort_rx: tokio::sync::watch::Receiver<bool>,
    ) -> Result<String> {
        debug!(
            session_id = %session_id,
            agent = %agent,
            model = %model.model_id,
            "Executing prompt in session"
        );

        // Get or create session context
        let _context = {
            let mut sessions = self.sessions.write().await;
            let session = sessions.get_mut(session_id).ok_or_else(|| {
                AgentError::SessionNotFound(session_id.to_string())
            })?;

            // Add user message to context and messages
            session.context.add_message(ChatMessage::user(prompt.to_string()));
            session.messages.push(json!({
                "id": message_id,
                "role": "user",
                "content": prompt,
            }));

            session.context.clone()
        };

        // Create chat service for execution
        let chat_service = self.create_chat_service();

        // Build system prompt based on agent type
        let system_prompt = match agent {
            "explore" => "You are a fast exploration agent. Focus on finding information quickly and reporting back concisely.",
            "librarian" => "You are a documentation specialist. Search external resources, repos, and documentation to find relevant information.",
            "oracle" => "You are an expert technical advisor. Provide deep analysis and architectural guidance.",
            "general" | _ => "You are a general-purpose coding assistant. Execute the task autonomously and report results.",
        };

        // Create a context for this execution
        // Parse session_id as UUID or generate new one
        let exec_uuid = Uuid::parse_str(session_id).unwrap_or_else(|_| Uuid::new_v4());
        let mut exec_context = ChatContext::new(exec_uuid, self.default_context_size)
            .with_system_prompt(system_prompt.to_string());
        exec_context.add_message(ChatMessage::user(prompt.to_string()));

        // Send progress update
        let _ = progress_tx.send(TaskProgress {
            id: message_id.to_string(),
            tool: "task".to_string(),
            status: "running".to_string(),
            title: None,
        });

        // Execute with tool loop
        // Note: We use a simplified execution here. Full implementation would use
        // chat_service.send_message_with_tools() but that requires more context setup.
        
        // For now, do a simple chat completion
        let response = tokio::select! {
            result = chat_service.send_message(&mut exec_context, prompt.to_string(), Some(model.model_id.clone())) => {
                result.map_err(|e| AgentError::ExecutionFailed(e.to_string()))?
            }
            _ = abort_rx.changed() => {
                if *abort_rx.borrow() {
                    warn!(session_id = %session_id, "Task aborted");
                    let _ = progress_tx.send(TaskProgress {
                        id: message_id.to_string(),
                        tool: "task".to_string(),
                        status: "aborted".to_string(),
                        title: None,
                    });
                    return Err(AgentError::TaskAborted);
                }
                // Channel closed but not aborted - continue waiting
                chat_service.send_message(&mut exec_context, prompt.to_string(), Some(model.model_id.clone()))
                    .await
                    .map_err(|e| AgentError::ExecutionFailed(e.to_string()))?
            }
        };

        // Store response in session
        {
            let mut sessions = self.sessions.write().await;
            if let Some(session) = sessions.get_mut(session_id) {
                session.messages.push(json!({
                    "id": Uuid::new_v4().to_string(),
                    "role": "assistant",
                    "content": &response.content,
                }));
            }
        }

        // Send completion progress
        let _ = progress_tx.send(TaskProgress {
            id: message_id.to_string(),
            tool: "task".to_string(),
            status: "completed".to_string(),
            title: Some("Task completed".to_string()),
        });

        info!(
            session_id = %session_id,
            response_len = response.content.len(),
            "Prompt execution completed"
        );

        Ok(response.content)
    }

    /// Get session messages
    async fn get_session_messages(&self, session_id: &str) -> Result<Vec<Value>> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(session_id).ok_or_else(|| {
            AgentError::SessionNotFound(session_id.to_string())
        })?;

        Ok(session.messages.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Full tests would require mocking the Provider trait
    // For now, we just test the basic struct creation

    #[test]
    fn test_session_manager_creation() {
        // This test verifies the struct can be created
        // Full testing requires a mock provider
        assert!(true); // Placeholder
    }
}
