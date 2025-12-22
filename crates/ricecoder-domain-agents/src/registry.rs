//! Domain agent registry and management

use std::{collections::HashMap, sync::Arc};

use tracing::{debug, info};

use crate::{
    domain_agents::{BackendAgent, DevOpsAgent, DomainAgent, DomainAgentInput, FrontendAgent},
    error::{DomainAgentError, Result},
    models::{DomainAgentMetadata, DomainAgentRegistry},
};

/// Registry for domain-specific agents
pub struct DomainAgentRegistryManager {
    agents: HashMap<String, Arc<dyn DomainAgent>>,
    registry: DomainAgentRegistry,
}

impl DomainAgentRegistryManager {
    /// Create a new registry manager
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
            registry: DomainAgentRegistry::new(),
        }
    }

    /// Initialize with default agents
    pub fn with_defaults() -> Self {
        let mut manager = Self::new();
        manager.register_default_agents();
        manager
    }

    /// Register default agents
    fn register_default_agents(&mut self) {
        debug!("Registering default domain agents");

        // Register frontend agent
        let frontend_agent = Arc::new(FrontendAgent::new());
        self.agents
            .insert("frontend".to_string(), frontend_agent.clone());
        self.registry
            .register_agent("frontend", frontend_agent.metadata().clone());

        // Register backend agent
        let backend_agent = Arc::new(BackendAgent::new());
        self.agents
            .insert("backend".to_string(), backend_agent.clone());
        self.registry
            .register_agent("backend", backend_agent.metadata().clone());

        // Register DevOps agent
        let devops_agent = Arc::new(DevOpsAgent::new());
        self.agents
            .insert("devops".to_string(), devops_agent.clone());
        self.registry
            .register_agent("devops", devops_agent.metadata().clone());

        info!("Registered {} default domain agents", self.agents.len());
    }

    /// Register a custom agent
    pub fn register_agent(&mut self, domain: &str, agent: Arc<dyn DomainAgent>) {
        debug!("Registering agent for domain: {}", domain);
        self.agents.insert(domain.to_string(), agent.clone());
        self.registry
            .register_agent(domain, agent.metadata().clone());
    }

    /// Get agent for domain
    pub fn get_agent(&self, domain: &str) -> Result<Arc<dyn DomainAgent>> {
        self.agents
            .get(domain)
            .cloned()
            .ok_or_else(|| DomainAgentError::AgentNotFound(domain.to_string()))
    }

    /// Execute agent for domain
    pub async fn execute_agent(
        &self,
        domain: &str,
        input: DomainAgentInput,
    ) -> Result<crate::domain_agents::DomainAgentOutput> {
        let agent = self.get_agent(domain)?;
        agent.execute(input).await
    }

    /// Get all registered domains
    pub fn get_registered_domains(&self) -> Vec<&str> {
        self.agents.keys().map(|s| s.as_str()).collect()
    }

    /// Check if domain is registered
    pub fn has_agent(&self, domain: &str) -> bool {
        self.agents.contains_key(domain)
    }

    /// Get agent metadata
    pub fn get_agent_metadata(&self, domain: &str) -> Result<&DomainAgentMetadata> {
        self.registry
            .get_agent(domain)
            .ok_or_else(|| DomainAgentError::AgentNotFound(domain.to_string()))
    }

    /// Get all agent metadata
    pub fn get_all_agents_metadata(&self) -> Vec<&DomainAgentMetadata> {
        self.registry.agents.values().collect()
    }

    /// Get registry
    pub fn get_registry(&self) -> &DomainAgentRegistry {
        &self.registry
    }
}

impl Default for DomainAgentRegistryManager {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = DomainAgentRegistryManager::new();
        assert!(registry.agents.is_empty());
    }

    #[test]
    fn test_register_default_agents() {
        let registry = DomainAgentRegistryManager::with_defaults();
        assert_eq!(registry.agents.len(), 3);
        assert!(registry.has_agent("frontend"));
        assert!(registry.has_agent("backend"));
        assert!(registry.has_agent("devops"));
    }

    #[test]
    fn test_get_agent() {
        let registry = DomainAgentRegistryManager::with_defaults();
        let agent = registry.get_agent("frontend");
        assert!(agent.is_ok());
    }

    #[test]
    fn test_get_nonexistent_agent() {
        let registry = DomainAgentRegistryManager::with_defaults();
        let agent = registry.get_agent("nonexistent");
        assert!(agent.is_err());
    }

    #[test]
    fn test_get_registered_domains() {
        let registry = DomainAgentRegistryManager::with_defaults();
        let domains = registry.get_registered_domains();
        assert_eq!(domains.len(), 3);
    }

    #[test]
    fn test_has_agent() {
        let registry = DomainAgentRegistryManager::with_defaults();
        assert!(registry.has_agent("frontend"));
        assert!(!registry.has_agent("nonexistent"));
    }

    #[test]
    fn test_get_agent_metadata() {
        let registry = DomainAgentRegistryManager::with_defaults();
        let metadata = registry.get_agent_metadata("frontend");
        assert!(metadata.is_ok());
        assert_eq!(metadata.unwrap().domain, "frontend");
    }

    #[test]
    fn test_get_all_agents_metadata() {
        let registry = DomainAgentRegistryManager::with_defaults();
        let all_metadata = registry.get_all_agents_metadata();
        assert_eq!(all_metadata.len(), 3);
    }
}
