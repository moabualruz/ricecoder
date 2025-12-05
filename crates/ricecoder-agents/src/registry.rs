//! Agent registry for discovering and managing agents

use crate::agents::Agent;
use crate::error::{AgentError, Result};
use crate::models::TaskType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

/// Metadata about a registered agent
///
/// This struct contains metadata about an agent, including its ID, name, description,
/// and the task types it supports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetadataInfo {
    /// Agent identifier
    pub id: String,
    /// Agent name
    pub name: String,
    /// Agent description
    pub description: String,
    /// Supported task types
    pub supported_task_types: Vec<TaskType>,
}

/// Registry for discovering and managing agents
///
/// The `AgentRegistry` maintains a registry of all available agents and provides
/// methods to discover agents by ID or task type. It also manages agent metadata
/// and configuration.
///
/// # Examples
///
/// ```ignore
/// use ricecoder_agents::{AgentRegistry, Agent, TaskType};
/// use std::sync::Arc;
///
/// let mut registry = AgentRegistry::new();
/// // Register agents...
///
/// // Find agents by task type
/// let agents = registry.find_agents_by_task_type(TaskType::CodeReview);
/// ```
pub struct AgentRegistry {
    agents: HashMap<String, Arc<dyn Agent>>,
    task_type_map: HashMap<TaskType, Vec<String>>,
    metadata: HashMap<String, AgentMetadataInfo>,
}

impl AgentRegistry {
    /// Create a new agent registry
    ///
    /// # Returns
    ///
    /// A new empty `AgentRegistry`
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
            task_type_map: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// Register an agent
    ///
    /// Registers an agent with the registry, making it available for task execution.
    /// The agent is indexed by ID and by each task type it supports.
    ///
    /// # Arguments
    ///
    /// * `agent` - The agent to register
    pub fn register(&mut self, agent: Arc<dyn Agent>) {
        let agent_id = agent.id().to_string();
        let agent_name = agent.name().to_string();

        debug!(agent_id = %agent_id, agent_name = %agent_name, "Registering agent");

        self.agents.insert(agent_id.clone(), agent.clone());

        // Collect supported task types
        let supported_task_types: Vec<TaskType> = [
            TaskType::CodeReview,
            TaskType::TestGeneration,
            TaskType::Documentation,
            TaskType::Refactoring,
            TaskType::SecurityAnalysis,
        ]
        .iter()
        .copied()
        .filter(|task_type| agent.supports(*task_type))
        .collect();

        debug!(
            agent_id = %agent_id,
            task_count = supported_task_types.len(),
            "Agent supports task types"
        );

        // Register for each supported task type
        for task_type in &supported_task_types {
            self.task_type_map
                .entry(*task_type)
                .or_default()
                .push(agent_id.clone());
        }

        // Store metadata
        let metadata = AgentMetadataInfo {
            id: agent_id.clone(),
            name: agent_name.clone(),
            description: agent.description().to_string(),
            supported_task_types,
        };
        self.metadata.insert(metadata.id.clone(), metadata);

        info!(agent_id = %agent_id, agent_name = %agent_name, "Agent registered successfully");
    }

    /// Find an agent by ID
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The ID of the agent to find
    ///
    /// # Returns
    ///
    /// A `Result` containing the agent or an error if not found
    pub fn find_agent(&self, agent_id: &str) -> Result<Arc<dyn Agent>> {
        self.agents
            .get(agent_id)
            .cloned()
            .ok_or_else(|| AgentError::not_found(agent_id))
    }

    /// Find agents by task type
    ///
    /// # Arguments
    ///
    /// * `task_type` - The task type to find agents for
    ///
    /// # Returns
    ///
    /// A vector of agents that support the given task type
    pub fn find_agents_by_task_type(&self, task_type: TaskType) -> Vec<Arc<dyn Agent>> {
        self.task_type_map
            .get(&task_type)
            .map(|agent_ids| {
                agent_ids
                    .iter()
                    .filter_map(|id| self.agents.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all registered agents
    ///
    /// # Returns
    ///
    /// A vector of all registered agents
    pub fn all_agents(&self) -> Vec<Arc<dyn Agent>> {
        self.agents.values().cloned().collect()
    }

    /// Get the number of registered agents
    ///
    /// # Returns
    ///
    /// The total number of registered agents
    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }

    /// Get metadata for a specific agent
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The ID of the agent
    ///
    /// # Returns
    ///
    /// An `Option` containing the agent metadata if found
    pub fn get_agent_metadata(&self, agent_id: &str) -> Option<AgentMetadataInfo> {
        self.metadata.get(agent_id).cloned()
    }

    /// Get metadata for all registered agents
    ///
    /// # Returns
    ///
    /// A vector of metadata for all registered agents
    pub fn all_agent_metadata(&self) -> Vec<AgentMetadataInfo> {
        self.metadata.values().cloned().collect()
    }

    /// Check if an agent is registered
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The ID of the agent to check
    ///
    /// # Returns
    ///
    /// `true` if the agent is registered, `false` otherwise
    pub fn has_agent(&self, agent_id: &str) -> bool {
        self.agents.contains_key(agent_id)
    }

    /// Get agents that support a specific task type
    ///
    /// # Arguments
    ///
    /// * `task_type` - The task type to find agents for
    ///
    /// # Returns
    ///
    /// A vector of metadata for agents that support the given task type
    pub fn agents_for_task_type(&self, task_type: TaskType) -> Vec<AgentMetadataInfo> {
        self.task_type_map
            .get(&task_type)
            .map(|agent_ids| {
                agent_ids
                    .iter()
                    .filter_map(|id| self.metadata.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Discover built-in agents at startup
    ///
    /// This method initializes the registry with built-in agents.
    /// In the current implementation, this is a placeholder for future
    /// agent discovery mechanisms.
    pub fn discover_builtin_agents(&mut self) -> Result<()> {
        info!("Discovering built-in agents");
        // Built-in agents will be registered here as they are implemented
        // For now, this is a placeholder that can be extended
        debug!("Built-in agent discovery completed");
        Ok(())
    }

    /// Load agent configuration from project settings
    ///
    /// This method loads agent configuration from a configuration source.
    /// The configuration can be used to enable/disable agents or customize
    /// their behavior.
    pub fn load_configuration(&mut self, config: HashMap<String, serde_json::Value>) -> Result<()> {
        info!(config_count = config.len(), "Loading agent configuration");
        // Configuration loading logic can be implemented here
        // This allows agents to be configured at runtime
        debug!("Agent configuration loaded successfully");
        Ok(())
    }

    /// Get all task types that have registered agents
    ///
    /// # Returns
    ///
    /// A vector of all task types that have at least one registered agent
    pub fn supported_task_types(&self) -> Vec<TaskType> {
        let mut types: Vec<TaskType> = self.task_type_map.keys().copied().collect();
        types.sort_by_key(|t| format!("{:?}", t));
        types
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AgentInput, AgentOutput};

    struct TestAgent {
        id: String,
        name: String,
        description: String,
        task_types: Vec<TaskType>,
    }

    #[async_trait::async_trait]
    impl Agent for TestAgent {
        fn id(&self) -> &str {
            &self.id
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            &self.description
        }

        fn supports(&self, task_type: TaskType) -> bool {
            self.task_types.contains(&task_type)
        }

        async fn execute(&self, _input: AgentInput) -> Result<AgentOutput> {
            Ok(AgentOutput::default())
        }
    }

    #[test]
    fn test_register_agent() {
        let mut registry = AgentRegistry::new();
        let agent = Arc::new(TestAgent {
            id: "test-agent".to_string(),
            name: "Test Agent".to_string(),
            description: "A test agent".to_string(),
            task_types: vec![TaskType::CodeReview],
        });

        registry.register(agent);
        assert_eq!(registry.agent_count(), 1);
    }

    #[test]
    fn test_find_agent_by_id() {
        let mut registry = AgentRegistry::new();
        let agent = Arc::new(TestAgent {
            id: "test-agent".to_string(),
            name: "Test Agent".to_string(),
            description: "A test agent".to_string(),
            task_types: vec![TaskType::CodeReview],
        });

        registry.register(agent);
        let found = registry.find_agent("test-agent");
        assert!(found.is_ok());
    }

    #[test]
    fn test_find_agent_not_found() {
        let registry = AgentRegistry::new();
        let found = registry.find_agent("nonexistent");
        assert!(found.is_err());
    }

    #[test]
    fn test_find_agents_by_task_type() {
        let mut registry = AgentRegistry::new();
        let agent = Arc::new(TestAgent {
            id: "test-agent".to_string(),
            name: "Test Agent".to_string(),
            description: "A test agent".to_string(),
            task_types: vec![TaskType::CodeReview],
        });

        registry.register(agent);
        let agents = registry.find_agents_by_task_type(TaskType::CodeReview);
        assert_eq!(agents.len(), 1);
    }

    #[test]
    fn test_get_agent_metadata() {
        let mut registry = AgentRegistry::new();
        let agent = Arc::new(TestAgent {
            id: "test-agent".to_string(),
            name: "Test Agent".to_string(),
            description: "A test agent for metadata".to_string(),
            task_types: vec![TaskType::CodeReview, TaskType::SecurityAnalysis],
        });

        registry.register(agent);
        let metadata = registry.get_agent_metadata("test-agent");
        assert!(metadata.is_some());

        let meta = metadata.unwrap();
        assert_eq!(meta.id, "test-agent");
        assert_eq!(meta.name, "Test Agent");
        assert_eq!(meta.description, "A test agent for metadata");
        assert_eq!(meta.supported_task_types.len(), 2);
        assert!(meta.supported_task_types.contains(&TaskType::CodeReview));
        assert!(meta
            .supported_task_types
            .contains(&TaskType::SecurityAnalysis));
    }

    #[test]
    fn test_all_agent_metadata() {
        let mut registry = AgentRegistry::new();
        let agent1 = Arc::new(TestAgent {
            id: "agent-1".to_string(),
            name: "Agent 1".to_string(),
            description: "First agent".to_string(),
            task_types: vec![TaskType::CodeReview],
        });
        let agent2 = Arc::new(TestAgent {
            id: "agent-2".to_string(),
            name: "Agent 2".to_string(),
            description: "Second agent".to_string(),
            task_types: vec![TaskType::TestGeneration],
        });

        registry.register(agent1);
        registry.register(agent2);

        let all_metadata = registry.all_agent_metadata();
        assert_eq!(all_metadata.len(), 2);
    }

    #[test]
    fn test_has_agent() {
        let mut registry = AgentRegistry::new();
        let agent = Arc::new(TestAgent {
            id: "test-agent".to_string(),
            name: "Test Agent".to_string(),
            description: "A test agent".to_string(),
            task_types: vec![TaskType::CodeReview],
        });

        registry.register(agent);
        assert!(registry.has_agent("test-agent"));
        assert!(!registry.has_agent("nonexistent"));
    }

    #[test]
    fn test_agents_for_task_type() {
        let mut registry = AgentRegistry::new();
        let agent1 = Arc::new(TestAgent {
            id: "agent-1".to_string(),
            name: "Agent 1".to_string(),
            description: "First agent".to_string(),
            task_types: vec![TaskType::CodeReview],
        });
        let agent2 = Arc::new(TestAgent {
            id: "agent-2".to_string(),
            name: "Agent 2".to_string(),
            description: "Second agent".to_string(),
            task_types: vec![TaskType::CodeReview, TaskType::SecurityAnalysis],
        });

        registry.register(agent1);
        registry.register(agent2);

        let code_review_agents = registry.agents_for_task_type(TaskType::CodeReview);
        assert_eq!(code_review_agents.len(), 2);

        let security_agents = registry.agents_for_task_type(TaskType::SecurityAnalysis);
        assert_eq!(security_agents.len(), 1);

        let doc_agents = registry.agents_for_task_type(TaskType::Documentation);
        assert_eq!(doc_agents.len(), 0);
    }

    #[test]
    fn test_multiple_agents_same_task_type() {
        let mut registry = AgentRegistry::new();
        let agent1 = Arc::new(TestAgent {
            id: "agent-1".to_string(),
            name: "Agent 1".to_string(),
            description: "First agent".to_string(),
            task_types: vec![TaskType::CodeReview],
        });
        let agent2 = Arc::new(TestAgent {
            id: "agent-2".to_string(),
            name: "Agent 2".to_string(),
            description: "Second agent".to_string(),
            task_types: vec![TaskType::CodeReview],
        });

        registry.register(agent1);
        registry.register(agent2);

        let agents = registry.find_agents_by_task_type(TaskType::CodeReview);
        assert_eq!(agents.len(), 2);
    }

    #[test]
    fn test_agent_metadata_serialization() {
        let metadata = AgentMetadataInfo {
            id: "test-agent".to_string(),
            name: "Test Agent".to_string(),
            description: "A test agent".to_string(),
            supported_task_types: vec![TaskType::CodeReview, TaskType::SecurityAnalysis],
        };

        let json = serde_json::to_string(&metadata).expect("serialization failed");
        let deserialized: AgentMetadataInfo =
            serde_json::from_str(&json).expect("deserialization failed");

        assert_eq!(deserialized.id, metadata.id);
        assert_eq!(deserialized.name, metadata.name);
        assert_eq!(deserialized.description, metadata.description);
        assert_eq!(
            deserialized.supported_task_types,
            metadata.supported_task_types
        );
    }

    #[test]
    fn test_discover_builtin_agents() {
        let mut registry = AgentRegistry::new();
        let result = registry.discover_builtin_agents();
        assert!(result.is_ok());
    }

    #[test]
    fn test_load_configuration() {
        let mut registry = AgentRegistry::new();
        let mut config = HashMap::new();
        config.insert("agent-1".to_string(), serde_json::json!({"enabled": true}));

        let result = registry.load_configuration(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_supported_task_types() {
        let mut registry = AgentRegistry::new();
        let agent1 = Arc::new(TestAgent {
            id: "agent-1".to_string(),
            name: "Agent 1".to_string(),
            description: "First agent".to_string(),
            task_types: vec![TaskType::CodeReview],
        });
        let agent2 = Arc::new(TestAgent {
            id: "agent-2".to_string(),
            name: "Agent 2".to_string(),
            description: "Second agent".to_string(),
            task_types: vec![TaskType::TestGeneration, TaskType::SecurityAnalysis],
        });

        registry.register(agent1);
        registry.register(agent2);

        let supported = registry.supported_task_types();
        assert!(supported.contains(&TaskType::CodeReview));
        assert!(supported.contains(&TaskType::TestGeneration));
        assert!(supported.contains(&TaskType::SecurityAnalysis));
        assert!(!supported.contains(&TaskType::Documentation));
    }

    #[test]
    fn test_registry_empty_discovery() {
        let registry = AgentRegistry::new();
        assert_eq!(registry.agent_count(), 0);
        assert!(registry.all_agent_metadata().is_empty());
        assert!(registry.supported_task_types().is_empty());
    }

    #[test]
    fn test_registry_with_multiple_agents() {
        let mut registry = AgentRegistry::new();
        let agents: Vec<Arc<dyn Agent>> = (1..=5)
            .map(|i| {
                Arc::new(TestAgent {
                    id: format!("agent-{}", i),
                    name: format!("Agent {}", i),
                    description: format!("Test agent {}", i),
                    task_types: vec![TaskType::CodeReview],
                }) as Arc<dyn Agent>
            })
            .collect();

        for agent in agents {
            registry.register(agent);
        }

        assert_eq!(registry.agent_count(), 5);
        assert_eq!(registry.all_agent_metadata().len(), 5);
    }
}
