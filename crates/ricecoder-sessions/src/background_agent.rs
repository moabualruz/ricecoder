//! Background agent management

use crate::error::{SessionError, SessionResult};
use crate::models::{BackgroundAgent, AgentStatus};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

/// Event emitted when a background agent completes
#[derive(Debug, Clone)]
pub struct AgentCompletionEvent {
    /// ID of the completed agent
    pub agent_id: String,
    /// Final status of the agent
    pub status: AgentStatus,
    /// Optional result message
    pub message: Option<String>,
}

/// Manages background agents running in sessions
#[derive(Debug, Clone)]
pub struct BackgroundAgentManager {
    /// All background agents indexed by ID
    agents: Arc<RwLock<HashMap<String, BackgroundAgent>>>,
    /// Running tasks indexed by agent ID
    tasks: Arc<RwLock<HashMap<String, JoinHandle<()>>>>,
    /// Completion events for agents
    completion_events: Arc<RwLock<Vec<AgentCompletionEvent>>>,
}

impl BackgroundAgentManager {
    /// Create a new background agent manager
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            tasks: Arc::new(RwLock::new(HashMap::new())),
            completion_events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Start a background agent asynchronously
    pub async fn start_agent(&self, agent: BackgroundAgent) -> SessionResult<String> {
        let agent_id = agent.id.clone();
        let agent_type = agent.agent_type.clone();

        // Store the agent
        {
            let mut agents = self.agents.write().await;
            agents.insert(agent_id.clone(), agent.clone());
        }

        // Spawn a task to simulate agent execution
        let agent_id_clone = agent_id.clone();
        let agents = Arc::clone(&self.agents);
        let completion_events = Arc::clone(&self.completion_events);

        let task = tokio::spawn(async move {
            // Simulate agent work
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            // Update agent status to completed
            {
                let mut agents_lock = agents.write().await;
                if let Some(agent) = agents_lock.get_mut(&agent_id_clone) {
                    agent.status = AgentStatus::Completed;
                    agent.completed_at = Some(chrono::Utc::now());
                }
            }

            // Emit completion event
            {
                let mut events = completion_events.write().await;
                events.push(AgentCompletionEvent {
                    agent_id: agent_id_clone.clone(),
                    status: AgentStatus::Completed,
                    message: Some(format!("Agent {} completed successfully", agent_type)),
                });
            }
        });

        // Store the task
        {
            let mut tasks = self.tasks.write().await;
            tasks.insert(agent_id.clone(), task);
        }

        Ok(agent_id)
    }

    /// Get the status of a background agent
    pub async fn get_agent_status(&self, agent_id: &str) -> SessionResult<AgentStatus> {
        let agents = self.agents.read().await;
        agents
            .get(agent_id)
            .map(|agent| agent.status)
            .ok_or_else(|| SessionError::AgentError(format!("Agent not found: {}", agent_id)))
    }

    /// Get a background agent by ID
    pub async fn get_agent(&self, agent_id: &str) -> SessionResult<BackgroundAgent> {
        let agents = self.agents.read().await;
        agents
            .get(agent_id)
            .cloned()
            .ok_or_else(|| SessionError::AgentError(format!("Agent not found: {}", agent_id)))
    }

    /// Pause a background agent
    pub async fn pause_agent(&self, agent_id: &str) -> SessionResult<()> {
        let mut agents = self.agents.write().await;
        if let Some(agent) = agents.get_mut(agent_id) {
            if agent.status == AgentStatus::Running {
                agent.status = AgentStatus::Cancelled;
                Ok(())
            } else {
                Err(SessionError::AgentError(format!(
                    "Cannot pause agent in {:?} state",
                    agent.status
                )))
            }
        } else {
            Err(SessionError::AgentError(format!("Agent not found: {}", agent_id)))
        }
    }

    /// Cancel a background agent
    pub async fn cancel_agent(&self, agent_id: &str) -> SessionResult<()> {
        let mut agents = self.agents.write().await;
        if let Some(agent) = agents.get_mut(agent_id) {
            agent.status = AgentStatus::Cancelled;
            agent.completed_at = Some(chrono::Utc::now());

            // Emit cancellation event
            let mut events = self.completion_events.write().await;
            events.push(AgentCompletionEvent {
                agent_id: agent_id.to_string(),
                status: AgentStatus::Cancelled,
                message: Some("Agent was cancelled".to_string()),
            });

            Ok(())
        } else {
            Err(SessionError::AgentError(format!("Agent not found: {}", agent_id)))
        }
    }

    /// List all background agents
    pub async fn list_agents(&self) -> Vec<BackgroundAgent> {
        let agents = self.agents.read().await;
        agents.values().cloned().collect()
    }

    /// Get all completion events
    pub async fn get_completion_events(&self) -> Vec<AgentCompletionEvent> {
        let events = self.completion_events.read().await;
        events.clone()
    }

    /// Clear completion events
    pub async fn clear_completion_events(&self) {
        let mut events = self.completion_events.write().await;
        events.clear();
    }

    /// Check if an agent is running
    pub async fn is_agent_running(&self, agent_id: &str) -> bool {
        if let Ok(status) = self.get_agent_status(agent_id).await {
            status == AgentStatus::Running
        } else {
            false
        }
    }

    /// Wait for an agent to complete
    pub async fn wait_for_agent(&self, agent_id: &str) -> SessionResult<AgentStatus> {
        loop {
            let status = self.get_agent_status(agent_id).await?;
            if status != AgentStatus::Running {
                return Ok(status);
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    }
}

impl Default for BackgroundAgentManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_start_agent() {
        let manager = BackgroundAgentManager::new();
        let agent = BackgroundAgent::new("test_agent".to_string(), Some("test task".to_string()));

        let agent_id = manager.start_agent(agent).await.unwrap();
        assert!(!agent_id.is_empty());

        // Wait for agent to complete
        let status = manager.wait_for_agent(&agent_id).await.unwrap();
        assert_eq!(status, AgentStatus::Completed);
    }

    #[tokio::test]
    async fn test_get_agent_status() {
        let manager = BackgroundAgentManager::new();
        let agent = BackgroundAgent::new("test_agent".to_string(), None);
        let agent_id = agent.id.clone();

        manager.start_agent(agent).await.unwrap();

        let status = manager.get_agent_status(&agent_id).await.unwrap();
        assert_eq!(status, AgentStatus::Running);
    }

    #[tokio::test]
    async fn test_cancel_agent() {
        let manager = BackgroundAgentManager::new();
        let agent = BackgroundAgent::new("test_agent".to_string(), None);
        let agent_id = agent.id.clone();

        manager.start_agent(agent).await.unwrap();
        manager.cancel_agent(&agent_id).await.unwrap();

        let status = manager.get_agent_status(&agent_id).await.unwrap();
        assert_eq!(status, AgentStatus::Cancelled);
    }

    #[tokio::test]
    async fn test_list_agents() {
        let manager = BackgroundAgentManager::new();
        let agent1 = BackgroundAgent::new("agent1".to_string(), None);
        let agent2 = BackgroundAgent::new("agent2".to_string(), None);

        manager.start_agent(agent1).await.unwrap();
        manager.start_agent(agent2).await.unwrap();

        let agents = manager.list_agents().await;
        assert_eq!(agents.len(), 2);
    }

    #[tokio::test]
    async fn test_completion_events() {
        let manager = BackgroundAgentManager::new();
        let agent = BackgroundAgent::new("test_agent".to_string(), None);

        manager.start_agent(agent).await.unwrap();

        // Wait for completion
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        let events = manager.get_completion_events().await;
        assert!(!events.is_empty());
        assert_eq!(events[0].status, AgentStatus::Completed);
    }
}
