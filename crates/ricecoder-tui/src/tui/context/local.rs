//! Local State Context Provider
//!
//! This module manages local TUI state including model selection, agent selection,
//! and MCP server status. Provides reactive state management with watchers and
//! favorites tracking.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Model identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModelId {
    pub provider_id: String,
    pub model_id: String,
}

impl ModelId {
    pub fn new(provider_id: impl Into<String>, model_id: impl Into<String>) -> Self {
        Self {
            provider_id: provider_id.into(),
            model_id: model_id.into(),
        }
    }
}

/// Agent information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub name: String,
    pub model: Option<ModelId>,
    pub mode: String,
    pub hidden: bool,
    pub default: bool,
    pub color: Option<String>,
}

/// MCP server status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum McpStatus {
    Connected,
    Disconnected,
    Failed,
    Disabled,
}

/// Local state data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalState {
    /// Current selected agent
    pub current_agent: String,
    /// Current selected model per agent
    pub agent_models: HashMap<String, ModelId>,
    /// Recent models
    pub recent_models: Vec<ModelId>,
    /// Favorite models
    pub favorite_models: Vec<ModelId>,
    /// MCP server statuses
    pub mcp_statuses: HashMap<String, McpStatus>,
}

impl Default for LocalState {
    fn default() -> Self {
        Self {
            current_agent: "build".to_string(),
            agent_models: HashMap::new(),
            recent_models: Vec::new(),
            favorite_models: Vec::new(),
            mcp_statuses: HashMap::new(),
        }
    }
}

/// Local state provider
#[derive(Debug, Clone)]
pub struct LocalProvider {
    state: Arc<RwLock<LocalState>>,
    available_agents: Arc<RwLock<Vec<AgentInfo>>>,
}

impl LocalProvider {
    /// Create new local provider
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(LocalState::default())),
            available_agents: Arc::new(RwLock::new(vec![
                AgentInfo {
                    name: "build".to_string(),
                    model: None,
                    mode: "agent".to_string(),
                    hidden: false,
                    default: true,
                    color: None,
                },
                AgentInfo {
                    name: "plan".to_string(),
                    model: None,
                    mode: "agent".to_string(),
                    hidden: false,
                    default: false,
                    color: None,
                },
            ])),
        }
    }

    /// Get current agent name
    pub async fn current_agent(&self) -> String {
        self.state.read().await.current_agent.clone()
    }

    /// Set current agent
    pub async fn set_agent(&self, name: String) {
        let mut state = self.state.write().await;
        state.current_agent = name;
    }

    /// Move to next/previous agent
    pub async fn move_agent(&self, direction: i32) {
        let agents = self.available_agents.read().await;
        let mut state = self.state.write().await;

        let current_idx = agents
            .iter()
            .position(|a| a.name == state.current_agent)
            .unwrap_or(0);

        let new_idx = if direction > 0 {
            (current_idx + 1) % agents.len()
        } else {
            (current_idx + agents.len() - 1) % agents.len()
        };

        if let Some(agent) = agents.get(new_idx) {
            state.current_agent = agent.name.clone();
        }
    }

    /// Get available agents
    pub async fn available_agents(&self) -> Vec<AgentInfo> {
        self.available_agents.read().await.clone()
    }

    /// Get current model for current agent
    pub async fn current_model(&self) -> Option<ModelId> {
        let state = self.state.read().await;
        state.agent_models.get(&state.current_agent).cloned()
    }

    /// Set model for current agent
    pub async fn set_model(&self, model: ModelId) {
        let mut state = self.state.write().await;
        let agent = state.current_agent.clone();
        state.agent_models.insert(agent, model.clone());

        // Add to recent (unique, max 10)
        state.recent_models.retain(|m| m != &model);
        state.recent_models.insert(0, model);
        if state.recent_models.len() > 10 {
            state.recent_models.truncate(10);
        }
    }

    /// Get recent models
    pub async fn recent_models(&self) -> Vec<ModelId> {
        self.state.read().await.recent_models.clone()
    }

    /// Get favorite models
    pub async fn favorite_models(&self) -> Vec<ModelId> {
        self.state.read().await.favorite_models.clone()
    }

    /// Toggle model as favorite
    pub async fn toggle_favorite(&self, model: ModelId) {
        let mut state = self.state.write().await;
        if let Some(pos) = state.favorite_models.iter().position(|m| m == &model) {
            state.favorite_models.remove(pos);
        } else {
            state.favorite_models.push(model);
        }
    }

    /// Check if MCP server is enabled
    pub async fn is_mcp_enabled(&self, name: &str) -> bool {
        self.state
            .read()
            .await
            .mcp_statuses
            .get(name)
            .map(|s| *s == McpStatus::Connected)
            .unwrap_or(false)
    }

    /// Get MCP status
    pub async fn mcp_status(&self, name: &str) -> Option<McpStatus> {
        self.state.read().await.mcp_statuses.get(name).cloned()
    }

    /// Update MCP status
    pub async fn set_mcp_status(&self, name: String, status: McpStatus) {
        let mut state = self.state.write().await;
        state.mcp_statuses.insert(name, status);
    }

    /// Load state from storage
    pub async fn load(&self, state: LocalState) {
        let mut current = self.state.write().await;
        *current = state;
    }

    /// Save state to storage
    pub async fn save(&self) -> LocalState {
        self.state.read().await.clone()
    }
}

impl Default for LocalProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_local_provider_creation() {
        let provider = LocalProvider::new();
        assert_eq!(provider.current_agent().await, "build");
    }

    #[tokio::test]
    async fn test_agent_switching() {
        let provider = LocalProvider::new();
        provider.set_agent("plan".to_string()).await;
        assert_eq!(provider.current_agent().await, "plan");
    }

    #[tokio::test]
    async fn test_agent_cycling() {
        let provider = LocalProvider::new();
        provider.move_agent(1).await;
        assert_eq!(provider.current_agent().await, "plan");

        provider.move_agent(1).await;
        assert_eq!(provider.current_agent().await, "build");

        provider.move_agent(-1).await;
        assert_eq!(provider.current_agent().await, "plan");
    }

    #[tokio::test]
    async fn test_model_selection() {
        let provider = LocalProvider::new();
        let model = ModelId::new("openai", "gpt-4");
        provider.set_model(model.clone()).await;
        assert_eq!(provider.current_model().await, Some(model));
    }

    #[tokio::test]
    async fn test_recent_models() {
        let provider = LocalProvider::new();
        let m1 = ModelId::new("openai", "gpt-4");
        let m2 = ModelId::new("anthropic", "claude-3");

        provider.set_model(m1.clone()).await;
        provider.set_model(m2.clone()).await;

        let recent = provider.recent_models().await;
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0], m2);
        assert_eq!(recent[1], m1);
    }

    #[tokio::test]
    async fn test_favorite_toggle() {
        let provider = LocalProvider::new();
        let model = ModelId::new("openai", "gpt-4");

        provider.toggle_favorite(model.clone()).await;
        assert_eq!(provider.favorite_models().await.len(), 1);

        provider.toggle_favorite(model.clone()).await;
        assert_eq!(provider.favorite_models().await.len(), 0);
    }

    #[tokio::test]
    async fn test_mcp_status() {
        let provider = LocalProvider::new();
        assert!(!provider.is_mcp_enabled("filesystem").await);

        provider
            .set_mcp_status("filesystem".to_string(), McpStatus::Connected)
            .await;
        assert!(provider.is_mcp_enabled("filesystem").await);

        provider
            .set_mcp_status("filesystem".to_string(), McpStatus::Disabled)
            .await;
        assert!(!provider.is_mcp_enabled("filesystem").await);
    }

    #[tokio::test]
    async fn test_state_persistence() {
        let provider = LocalProvider::new();
        let model = ModelId::new("openai", "gpt-4");
        provider.set_model(model.clone()).await;
        provider.toggle_favorite(model.clone()).await;

        let saved = provider.save().await;
        assert_eq!(saved.agent_models.len(), 1);
        assert_eq!(saved.favorite_models.len(), 1);

        let provider2 = LocalProvider::new();
        provider2.load(saved).await;
        assert_eq!(provider2.current_model().await, Some(model.clone()));
        assert_eq!(provider2.favorite_models().await.len(), 1);
    }
}
