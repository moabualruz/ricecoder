//! Dependency injection support for ricecoder-specs

use std::sync::Arc;
use ricecoder_common::di::{ServiceEntry, ServiceFactory};
use crate::{
    SpecManager, ApprovalManager, ChangeTracker, ConversationManager, WorkflowOrchestrator,
};

inventory::submit! {
    ServiceFactory::new("specs", create_specs_services)
}

fn create_specs_services() -> Vec<ServiceEntry> {
    vec![
        ServiceEntry::new::<SpecManager>(Arc::new(SpecManager::default())),
        ServiceEntry::new::<ApprovalManager>(Arc::new(ApprovalManager::default())),
        ServiceEntry::new::<ChangeTracker>(Arc::new(ChangeTracker::default())),
        ServiceEntry::new::<ConversationManager>(Arc::new(ConversationManager::default())),
        ServiceEntry::new::<WorkflowOrchestrator>(Arc::new(WorkflowOrchestrator::default())),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_common::di::list_discovered_factories;

    #[test]
    fn test_specs_factory_registered() {
        let factories = list_discovered_factories();
        assert!(factories.contains(&"specs"), "Factory should be registered");
    }
}
