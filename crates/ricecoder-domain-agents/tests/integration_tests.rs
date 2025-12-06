//! Integration tests for domain-specific agents

use ricecoder_domain_agents::{
    DomainAgentInput, DomainAgentRegistryManager, KnowledgeBase, KnowledgeEntry,
};
use std::collections::HashMap;

#[tokio::test]
async fn test_registry_with_all_agents() {
    let registry = DomainAgentRegistryManager::with_defaults();

    // Verify all agents are registered
    assert!(registry.has_agent("frontend"));
    assert!(registry.has_agent("backend"));
    assert!(registry.has_agent("devops"));

    // Verify domains
    let domains = registry.get_registered_domains();
    assert_eq!(domains.len(), 3);
}

#[tokio::test]
async fn test_execute_frontend_agent() {
    let registry = DomainAgentRegistryManager::with_defaults();

    let input = DomainAgentInput {
        domain: "frontend".to_string(),
        task: "Design a React component".to_string(),
        context: "User profile component".to_string(),
        parameters: HashMap::new(),
    };

    let output = registry.execute_agent("frontend", input).await.unwrap();
    assert_eq!(output.domain, "frontend");
    assert!(!output.response.is_empty());
    assert!(!output.suggestions.is_empty());
    assert!(output.confidence > 0.0 && output.confidence <= 1.0);
}

#[tokio::test]
async fn test_execute_backend_agent() {
    let registry = DomainAgentRegistryManager::with_defaults();

    let input = DomainAgentInput {
        domain: "backend".to_string(),
        task: "Design an API endpoint".to_string(),
        context: "User management API".to_string(),
        parameters: HashMap::new(),
    };

    let output = registry.execute_agent("backend", input).await.unwrap();
    assert_eq!(output.domain, "backend");
    assert!(!output.response.is_empty());
    assert!(!output.suggestions.is_empty());
}

#[tokio::test]
async fn test_execute_devops_agent() {
    let registry = DomainAgentRegistryManager::with_defaults();

    let input = DomainAgentInput {
        domain: "devops".to_string(),
        task: "Set up deployment pipeline".to_string(),
        context: "Kubernetes cluster".to_string(),
        parameters: HashMap::new(),
    };

    let output = registry.execute_agent("devops", input).await.unwrap();
    assert_eq!(output.domain, "devops");
    assert!(!output.response.is_empty());
    assert!(!output.suggestions.is_empty());
}

#[tokio::test]
async fn test_execute_nonexistent_agent() {
    let registry = DomainAgentRegistryManager::with_defaults();

    let input = DomainAgentInput {
        domain: "nonexistent".to_string(),
        task: "Some task".to_string(),
        context: "Some context".to_string(),
        parameters: HashMap::new(),
    };

    let result = registry.execute_agent("nonexistent", input).await;
    assert!(result.is_err());
}

#[test]
fn test_knowledge_base_operations() {
    let mut kb = KnowledgeBase::new("frontend", "1.0.0");

    // Add entries
    let entry1 = KnowledgeEntry {
        id: "react-hooks".to_string(),
        category: "patterns".to_string(),
        title: "React Hooks".to_string(),
        description: "Best practices for using React Hooks".to_string(),
        tags: vec!["react".to_string(), "hooks".to_string()],
        example: Some("const [count, setCount] = useState(0);".to_string()),
        references: vec!["https://react.dev".to_string()],
    };

    let entry2 = KnowledgeEntry {
        id: "vue-composition".to_string(),
        category: "patterns".to_string(),
        title: "Vue Composition API".to_string(),
        description: "Best practices for Vue Composition API".to_string(),
        tags: vec!["vue".to_string(), "composition".to_string()],
        example: None,
        references: vec![],
    };

    kb.add_entry(entry1);
    kb.add_entry(entry2);

    // Test search by tag
    let react_results = kb.search_by_tag("react");
    assert_eq!(react_results.len(), 1);

    let vue_results = kb.search_by_tag("vue");
    assert_eq!(vue_results.len(), 1);

    // Test search by category
    let pattern_results = kb.search_by_category("patterns");
    assert_eq!(pattern_results.len(), 2);
}

#[test]
fn test_agent_metadata() {
    let registry = DomainAgentRegistryManager::with_defaults();

    let frontend_metadata = registry.get_agent_metadata("frontend").unwrap();
    assert_eq!(frontend_metadata.domain, "frontend");
    assert_eq!(frontend_metadata.name, "Frontend Agent");
    assert!(!frontend_metadata.capabilities.is_empty());

    let backend_metadata = registry.get_agent_metadata("backend").unwrap();
    assert_eq!(backend_metadata.domain, "backend");
    assert_eq!(backend_metadata.name, "Backend Agent");

    let devops_metadata = registry.get_agent_metadata("devops").unwrap();
    assert_eq!(devops_metadata.domain, "devops");
    assert_eq!(devops_metadata.name, "DevOps Agent");
}

#[test]
fn test_get_all_agents_metadata() {
    let registry = DomainAgentRegistryManager::with_defaults();
    let all_metadata = registry.get_all_agents_metadata();

    assert_eq!(all_metadata.len(), 3);

    let domains: Vec<_> = all_metadata.iter().map(|m| m.domain.as_str()).collect();
    assert!(domains.contains(&"frontend"));
    assert!(domains.contains(&"backend"));
    assert!(domains.contains(&"devops"));
}
