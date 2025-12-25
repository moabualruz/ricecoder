//! Dependency injection support for ricecoder-domain-agents

use std::sync::Arc;
use ricecoder_common::di::{ServiceEntry, ServiceFactory};
use crate::{
    DomainAgentRegistryManager,
    KnowledgeBaseManager,
};

inventory::submit! {
    ServiceFactory::new("domain-agents", create_domain_agents_services)
}

fn create_domain_agents_services() -> Vec<ServiceEntry> {
    vec![
        ServiceEntry::new::<DomainAgentRegistryManager>(Arc::new(DomainAgentRegistryManager::with_defaults())),
        ServiceEntry::new::<KnowledgeBaseManager>(Arc::new(KnowledgeBaseManager::new())),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_common::di::list_discovered_factories;

    #[test]
    fn test_domain_agents_factory_registered() {
        let factories = list_discovered_factories();
        assert!(factories.contains(&"domain-agents"), "Factory should be registered");
    }
}
