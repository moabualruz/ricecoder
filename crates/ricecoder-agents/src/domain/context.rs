//! Shared context manager for cross-domain coordination

use crate::domain::error::{DomainError, DomainResult};
use crate::domain::models::{Recommendation, SharedContext};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Manages shared context across domain agents
///
/// This struct maintains cross-domain context that is shared between agents,
/// enabling coordination and dependency tracking between recommendations.
///
/// # Examples
///
/// ```ignore
/// use ricecoder_agents::domain::SharedContextManager;
///
/// let manager = SharedContextManager::new();
/// manager.update_context("project_type", serde_json::json!("web-app"))?;
/// let value = manager.get_context("project_type")?;
/// ```
#[derive(Debug, Clone)]
pub struct SharedContextManager {
    context: Arc<RwLock<SharedContext>>,
    agent_recommendations: Arc<RwLock<HashMap<String, Vec<Recommendation>>>>,
}

impl SharedContextManager {
    /// Create a new shared context manager
    pub fn new() -> Self {
        Self {
            context: Arc::new(RwLock::new(SharedContext::default())),
            agent_recommendations: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Update a context value
    ///
    /// # Arguments
    ///
    /// * `key` - Context key
    /// * `value` - Context value
    pub fn update_context(&self, key: &str, value: serde_json::Value) -> DomainResult<()> {
        let mut context = self
            .context
            .write()
            .map_err(|e| DomainError::internal(format!("Failed to acquire write lock: {}", e)))?;

        context.cross_domain_state.insert(key.to_string(), value);
        Ok(())
    }

    /// Get a context value
    ///
    /// # Arguments
    ///
    /// * `key` - Context key
    ///
    /// # Returns
    ///
    /// Returns the context value if found
    pub fn get_context(&self, key: &str) -> DomainResult<serde_json::Value> {
        let context = self
            .context
            .read()
            .map_err(|e| DomainError::internal(format!("Failed to acquire read lock: {}", e)))?;

        context
            .cross_domain_state
            .get(key)
            .cloned()
            .ok_or_else(|| DomainError::context_error(format!("Context key not found: {}", key)))
    }

    /// Update project type
    ///
    /// # Arguments
    ///
    /// * `project_type` - Project type
    pub fn set_project_type(&self, project_type: &str) -> DomainResult<()> {
        let mut context = self
            .context
            .write()
            .map_err(|e| DomainError::internal(format!("Failed to acquire write lock: {}", e)))?;

        context.project_type = project_type.to_string();
        Ok(())
    }

    /// Get project type
    pub fn get_project_type(&self) -> DomainResult<String> {
        let context = self
            .context
            .read()
            .map_err(|e| DomainError::internal(format!("Failed to acquire read lock: {}", e)))?;

        Ok(context.project_type.clone())
    }

    /// Add technology to tech stack
    ///
    /// # Arguments
    ///
    /// * `technology` - Technology to add
    pub fn add_technology(&self, technology: &str) -> DomainResult<()> {
        let mut context = self
            .context
            .write()
            .map_err(|e| DomainError::internal(format!("Failed to acquire write lock: {}", e)))?;

        if !context.tech_stack.contains(&technology.to_string()) {
            context.tech_stack.push(technology.to_string());
        }

        Ok(())
    }

    /// Get tech stack
    pub fn get_tech_stack(&self) -> DomainResult<Vec<String>> {
        let context = self
            .context
            .read()
            .map_err(|e| DomainError::internal(format!("Failed to acquire read lock: {}", e)))?;

        Ok(context.tech_stack.clone())
    }

    /// Add constraint
    ///
    /// # Arguments
    ///
    /// * `constraint` - Constraint to add
    pub fn add_constraint(&self, constraint: &str) -> DomainResult<()> {
        let mut context = self
            .context
            .write()
            .map_err(|e| DomainError::internal(format!("Failed to acquire write lock: {}", e)))?;

        if !context.constraints.contains(&constraint.to_string()) {
            context.constraints.push(constraint.to_string());
        }

        Ok(())
    }

    /// Get constraints
    pub fn get_constraints(&self) -> DomainResult<Vec<String>> {
        let context = self
            .context
            .read()
            .map_err(|e| DomainError::internal(format!("Failed to acquire read lock: {}", e)))?;

        Ok(context.constraints.clone())
    }

    /// Store agent recommendations
    ///
    /// # Arguments
    ///
    /// * `agent_id` - Agent identifier
    /// * `recommendations` - Recommendations from the agent
    pub fn store_agent_recommendations(
        &self,
        agent_id: &str,
        recommendations: Vec<Recommendation>,
    ) -> DomainResult<()> {
        let mut agent_recs = self
            .agent_recommendations
            .write()
            .map_err(|e| DomainError::internal(format!("Failed to acquire write lock: {}", e)))?;

        agent_recs.insert(agent_id.to_string(), recommendations);
        Ok(())
    }

    /// Get agent recommendations
    ///
    /// # Arguments
    ///
    /// * `agent_id` - Agent identifier
    ///
    /// # Returns
    ///
    /// Returns recommendations from the agent
    pub fn get_agent_recommendations(&self, agent_id: &str) -> DomainResult<Vec<Recommendation>> {
        let agent_recs = self
            .agent_recommendations
            .read()
            .map_err(|e| DomainError::internal(format!("Failed to acquire read lock: {}", e)))?;

        Ok(agent_recs.get(agent_id).cloned().unwrap_or_default())
    }

    /// Get all agent recommendations
    pub fn get_all_recommendations(&self) -> DomainResult<Vec<Recommendation>> {
        let agent_recs = self
            .agent_recommendations
            .read()
            .map_err(|e| DomainError::internal(format!("Failed to acquire read lock: {}", e)))?;

        let mut all_recs = Vec::new();
        for recs in agent_recs.values() {
            all_recs.extend(recs.clone());
        }

        Ok(all_recs)
    }

    /// Get shared context
    pub fn get_shared_context(&self) -> DomainResult<SharedContext> {
        let context = self
            .context
            .read()
            .map_err(|e| DomainError::internal(format!("Failed to acquire read lock: {}", e)))?;

        Ok(context.clone())
    }

    /// Clear all context
    pub fn clear(&self) -> DomainResult<()> {
        let mut context = self
            .context
            .write()
            .map_err(|e| DomainError::internal(format!("Failed to acquire write lock: {}", e)))?;

        *context = SharedContext::default();

        let mut agent_recs = self
            .agent_recommendations
            .write()
            .map_err(|e| DomainError::internal(format!("Failed to acquire write lock: {}", e)))?;

        agent_recs.clear();

        Ok(())
    }
}

impl Default for SharedContextManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_recommendation(domain: &str) -> Recommendation {
        Recommendation {
            domain: domain.to_string(),
            category: "test".to_string(),
            content: "Test recommendation".to_string(),
            technologies: vec!["Tech1".to_string()],
            rationale: "Test rationale".to_string(),
        }
    }

    #[test]
    fn test_context_manager_creation() {
        let manager = SharedContextManager::new();
        assert!(manager.get_project_type().unwrap().is_empty());
    }

    #[test]
    fn test_update_context() {
        let manager = SharedContextManager::new();
        manager
            .update_context("key", serde_json::json!("value"))
            .unwrap();

        let value = manager.get_context("key").unwrap();
        assert_eq!(value, serde_json::json!("value"));
    }

    #[test]
    fn test_get_nonexistent_context() {
        let manager = SharedContextManager::new();
        let result = manager.get_context("nonexistent");

        assert!(result.is_err());
    }

    #[test]
    fn test_set_project_type() {
        let manager = SharedContextManager::new();
        manager.set_project_type("web-app").unwrap();

        let project_type = manager.get_project_type().unwrap();
        assert_eq!(project_type, "web-app");
    }

    #[test]
    fn test_add_technology() {
        let manager = SharedContextManager::new();
        manager.add_technology("React").unwrap();

        let tech_stack = manager.get_tech_stack().unwrap();
        assert_eq!(tech_stack.len(), 1);
        assert_eq!(tech_stack[0], "React");
    }

    #[test]
    fn test_add_multiple_technologies() {
        let manager = SharedContextManager::new();
        manager.add_technology("React").unwrap();
        manager.add_technology("Node.js").unwrap();

        let tech_stack = manager.get_tech_stack().unwrap();
        assert_eq!(tech_stack.len(), 2);
    }

    #[test]
    fn test_add_duplicate_technology() {
        let manager = SharedContextManager::new();
        manager.add_technology("React").unwrap();
        manager.add_technology("React").unwrap();

        let tech_stack = manager.get_tech_stack().unwrap();
        assert_eq!(tech_stack.len(), 1);
    }

    #[test]
    fn test_add_constraint() {
        let manager = SharedContextManager::new();
        manager.add_constraint("Must support IE11").unwrap();

        let constraints = manager.get_constraints().unwrap();
        assert_eq!(constraints.len(), 1);
        assert_eq!(constraints[0], "Must support IE11");
    }

    #[test]
    fn test_add_multiple_constraints() {
        let manager = SharedContextManager::new();
        manager.add_constraint("Must support IE11").unwrap();
        manager.add_constraint("Must be mobile-friendly").unwrap();

        let constraints = manager.get_constraints().unwrap();
        assert_eq!(constraints.len(), 2);
    }

    #[test]
    fn test_store_agent_recommendations() {
        let manager = SharedContextManager::new();
        let recommendations = vec![create_test_recommendation("web")];

        manager
            .store_agent_recommendations("web-agent", recommendations)
            .unwrap();

        let retrieved = manager.get_agent_recommendations("web-agent").unwrap();
        assert_eq!(retrieved.len(), 1);
    }

    #[test]
    fn test_get_all_recommendations() {
        let manager = SharedContextManager::new();

        manager
            .store_agent_recommendations("web-agent", vec![create_test_recommendation("web")])
            .unwrap();
        manager
            .store_agent_recommendations(
                "backend-agent",
                vec![create_test_recommendation("backend")],
            )
            .unwrap();

        let all_recs = manager.get_all_recommendations().unwrap();
        assert_eq!(all_recs.len(), 2);
    }

    #[test]
    fn test_get_shared_context() {
        let manager = SharedContextManager::new();
        manager.set_project_type("web-app").unwrap();
        manager.add_technology("React").unwrap();

        let context = manager.get_shared_context().unwrap();
        assert_eq!(context.project_type, "web-app");
        assert_eq!(context.tech_stack.len(), 1);
    }

    #[test]
    fn test_clear() {
        let manager = SharedContextManager::new();
        manager.set_project_type("web-app").unwrap();
        manager.add_technology("React").unwrap();
        manager
            .store_agent_recommendations("web-agent", vec![create_test_recommendation("web")])
            .unwrap();

        manager.clear().unwrap();

        assert!(manager.get_project_type().unwrap().is_empty());
        assert!(manager.get_tech_stack().unwrap().is_empty());
        assert!(manager
            .get_agent_recommendations("web-agent")
            .unwrap()
            .is_empty());
    }

    #[test]
    fn test_default_manager() {
        let manager = SharedContextManager::default();
        assert!(manager.get_project_type().unwrap().is_empty());
    }

    #[test]
    fn test_context_isolation() {
        let manager1 = SharedContextManager::new();
        let manager2 = SharedContextManager::new();

        manager1.set_project_type("web-app").unwrap();
        manager2.set_project_type("mobile-app").unwrap();

        assert_eq!(manager1.get_project_type().unwrap(), "web-app");
        assert_eq!(manager2.get_project_type().unwrap(), "mobile-app");
    }
}
