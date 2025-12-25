//! Dependency injection support for ricecoder-orchestration

use std::sync::Arc;
use ricecoder_common::di::{ServiceEntry, ServiceFactory};
use crate::{
    DependencyAnalyzer, ImpactAnalyzer, ChangePropagationTracker, DependencyValidator, SyncManager,
};

inventory::submit! {
    ServiceFactory::new("orchestration", create_orchestration_services)
}

fn create_orchestration_services() -> Vec<ServiceEntry> {
    vec![
        ServiceEntry::new::<DependencyAnalyzer>(Arc::new(DependencyAnalyzer::new())),
        ServiceEntry::new::<ImpactAnalyzer>(Arc::new(ImpactAnalyzer::new())),
        ServiceEntry::new::<ChangePropagationTracker>(Arc::new(ChangePropagationTracker::new())),
        ServiceEntry::new::<DependencyValidator>(Arc::new(DependencyValidator::new())),
        ServiceEntry::new::<SyncManager>(Arc::new(SyncManager::new())),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_common::di::list_discovered_factories;

    #[test]
    fn test_orchestration_factory_registered() {
        let factories = list_discovered_factories();
        assert!(factories.contains(&"orchestration"), "Factory should be registered");
    }
}
