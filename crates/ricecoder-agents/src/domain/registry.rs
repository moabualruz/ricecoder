//! Domain registry for discovering and managing domain agents

use crate::domain::error::{DomainError, DomainResult};
use crate::domain::models::{DomainAgent, DomainCapability};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Registry for domain agents
///
/// This struct manages the registration and discovery of domain agents.
/// It supports auto-discovery of domains from configuration files and
/// provides agent lookup by domain type.
///
/// # Examples
///
/// ```ignore
/// use ricecoder_agents::domain::DomainRegistry;
///
/// let registry = DomainRegistry::new();
/// registry.register_agent("web", agent);
/// let agent = registry.get_agent("web")?;
/// ```
#[derive(Debug, Clone)]
pub struct DomainRegistry {
    agents: Arc<RwLock<HashMap<String, DomainAgent>>>,
}

impl DomainRegistry {
    /// Create a new domain registry
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a domain agent
    ///
    /// # Arguments
    ///
    /// * `domain` - Domain identifier (e.g., "web", "backend", "devops")
    /// * `agent` - Domain agent to register
    ///
    /// # Examples
    ///
    /// ```ignore
    /// registry.register_agent("web", web_agent);
    /// ```
    pub fn register_agent(&self, domain: &str, agent: DomainAgent) -> DomainResult<()> {
        let mut agents = self
            .agents
            .write()
            .map_err(|e| DomainError::internal(format!("Failed to acquire write lock: {}", e)))?;

        agents.insert(domain.to_string(), agent);
        Ok(())
    }

    /// Get a domain agent by domain type
    ///
    /// # Arguments
    ///
    /// * `domain` - Domain identifier
    ///
    /// # Returns
    ///
    /// Returns the domain agent if found, otherwise returns an error
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let agent = registry.get_agent("web")?;
    /// ```
    pub fn get_agent(&self, domain: &str) -> DomainResult<DomainAgent> {
        let agents = self
            .agents
            .read()
            .map_err(|e| DomainError::internal(format!("Failed to acquire read lock: {}", e)))?;

        agents
            .get(domain)
            .cloned()
            .ok_or_else(|| DomainError::agent_not_found(domain))
    }

    /// Discover all registered domains
    ///
    /// # Returns
    ///
    /// Returns a vector of all registered domain identifiers
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let domains = registry.discover_domains()?;
    /// ```
    pub fn discover_domains(&self) -> DomainResult<Vec<String>> {
        let agents = self
            .agents
            .read()
            .map_err(|e| DomainError::internal(format!("Failed to acquire read lock: {}", e)))?;

        Ok(agents.keys().cloned().collect())
    }

    /// List capabilities for a domain
    ///
    /// # Arguments
    ///
    /// * `domain` - Domain identifier
    ///
    /// # Returns
    ///
    /// Returns a vector of capabilities for the domain
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let capabilities = registry.list_capabilities("web")?;
    /// ```
    pub fn list_capabilities(&self, domain: &str) -> DomainResult<Vec<DomainCapability>> {
        let agent = self.get_agent(domain)?;
        Ok(agent.capabilities)
    }

    /// Check if a domain is registered
    ///
    /// # Arguments
    ///
    /// * `domain` - Domain identifier
    ///
    /// # Returns
    ///
    /// Returns true if the domain is registered, false otherwise
    pub fn has_domain(&self, domain: &str) -> DomainResult<bool> {
        let agents = self
            .agents
            .read()
            .map_err(|e| DomainError::internal(format!("Failed to acquire read lock: {}", e)))?;

        Ok(agents.contains_key(domain))
    }

    /// Get all registered agents
    ///
    /// # Returns
    ///
    /// Returns a vector of all registered domain agents
    pub fn get_all_agents(&self) -> DomainResult<Vec<DomainAgent>> {
        let agents = self
            .agents
            .read()
            .map_err(|e| DomainError::internal(format!("Failed to acquire read lock: {}", e)))?;

        Ok(agents.values().cloned().collect())
    }

    /// Clear all registered agents
    pub fn clear(&self) -> DomainResult<()> {
        let mut agents = self
            .agents
            .write()
            .map_err(|e| DomainError::internal(format!("Failed to acquire write lock: {}", e)))?;

        agents.clear();
        Ok(())
    }

    /// Get the number of registered domains
    pub fn domain_count(&self) -> DomainResult<usize> {
        let agents = self
            .agents
            .read()
            .map_err(|e| DomainError::internal(format!("Failed to acquire read lock: {}", e)))?;

        Ok(agents.len())
    }
}

impl Default for DomainRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::{DomainKnowledge, DomainAgent};

    fn create_test_agent(domain: &str) -> DomainAgent {
        DomainAgent {
            id: format!("{}-agent", domain),
            domain: domain.to_string(),
            capabilities: vec![],
            knowledge: DomainKnowledge::default(),
        }
    }

    #[test]
    fn test_registry_creation() {
        let registry = DomainRegistry::new();
        assert!(registry.discover_domains().unwrap().is_empty());
    }

    #[test]
    fn test_register_agent() {
        let registry = DomainRegistry::new();
        let agent = create_test_agent("web");

        registry.register_agent("web", agent).unwrap();
        assert!(registry.has_domain("web").unwrap());
    }

    #[test]
    fn test_get_agent() {
        let registry = DomainRegistry::new();
        let agent = create_test_agent("web");

        registry.register_agent("web", agent.clone()).unwrap();
        let retrieved = registry.get_agent("web").unwrap();

        assert_eq!(retrieved.id, agent.id);
        assert_eq!(retrieved.domain, agent.domain);
    }

    #[test]
    fn test_get_nonexistent_agent() {
        let registry = DomainRegistry::new();
        let result = registry.get_agent("nonexistent");

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DomainError::AgentNotFound(_)));
    }

    #[test]
    fn test_discover_domains() {
        let registry = DomainRegistry::new();

        registry.register_agent("web", create_test_agent("web")).unwrap();
        registry.register_agent("backend", create_test_agent("backend")).unwrap();
        registry.register_agent("devops", create_test_agent("devops")).unwrap();

        let domains = registry.discover_domains().unwrap();
        assert_eq!(domains.len(), 3);
        assert!(domains.contains(&"web".to_string()));
        assert!(domains.contains(&"backend".to_string()));
        assert!(domains.contains(&"devops".to_string()));
    }

    #[test]
    fn test_list_capabilities() {
        let registry = DomainRegistry::new();
        let agent = create_test_agent("web");

        registry.register_agent("web", agent).unwrap();
        let capabilities = registry.list_capabilities("web").unwrap();

        assert!(capabilities.is_empty());
    }

    #[test]
    fn test_has_domain() {
        let registry = DomainRegistry::new();

        registry.register_agent("web", create_test_agent("web")).unwrap();

        assert!(registry.has_domain("web").unwrap());
        assert!(!registry.has_domain("backend").unwrap());
    }

    #[test]
    fn test_get_all_agents() {
        let registry = DomainRegistry::new();

        registry.register_agent("web", create_test_agent("web")).unwrap();
        registry.register_agent("backend", create_test_agent("backend")).unwrap();

        let agents = registry.get_all_agents().unwrap();
        assert_eq!(agents.len(), 2);
    }

    #[test]
    fn test_clear() {
        let registry = DomainRegistry::new();

        registry.register_agent("web", create_test_agent("web")).unwrap();
        registry.register_agent("backend", create_test_agent("backend")).unwrap();

        assert_eq!(registry.domain_count().unwrap(), 2);

        registry.clear().unwrap();
        assert_eq!(registry.domain_count().unwrap(), 0);
    }

    #[test]
    fn test_domain_count() {
        let registry = DomainRegistry::new();

        assert_eq!(registry.domain_count().unwrap(), 0);

        registry.register_agent("web", create_test_agent("web")).unwrap();
        assert_eq!(registry.domain_count().unwrap(), 1);

        registry.register_agent("backend", create_test_agent("backend")).unwrap();
        assert_eq!(registry.domain_count().unwrap(), 2);
    }

    #[test]
    fn test_register_multiple_agents() {
        let registry = DomainRegistry::new();

        registry.register_agent("web", create_test_agent("web")).unwrap();
        registry.register_agent("backend", create_test_agent("backend")).unwrap();
        registry.register_agent("devops", create_test_agent("devops")).unwrap();

        assert_eq!(registry.domain_count().unwrap(), 3);
    }

    #[test]
    fn test_overwrite_agent() {
        let registry = DomainRegistry::new();

        let agent1 = create_test_agent("web");
        registry.register_agent("web", agent1).unwrap();

        let mut agent2 = create_test_agent("web");
        agent2.id = "web-agent-v2".to_string();
        registry.register_agent("web", agent2).unwrap();

        let retrieved = registry.get_agent("web").unwrap();
        assert_eq!(retrieved.id, "web-agent-v2");
    }

    #[test]
    fn test_default_registry() {
        let registry = DomainRegistry::default();
        assert_eq!(registry.domain_count().unwrap(), 0);
    }
}
