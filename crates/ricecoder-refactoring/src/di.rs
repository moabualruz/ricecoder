//! Dependency injection support for ricecoder-refactoring

use std::sync::Arc;
use ricecoder_common::di::{ServiceEntry, ServiceFactory};
use crate::{ImpactAnalyzer, PreviewGenerator, SafetyChecker, ValidationEngine};

inventory::submit! {
    ServiceFactory::new("refactoring", create_refactoring_services)
}

fn create_refactoring_services() -> Vec<ServiceEntry> {
    vec![
        ServiceEntry::new::<ImpactAnalyzer>(Arc::new(ImpactAnalyzer::new())),
        ServiceEntry::new::<PreviewGenerator>(Arc::new(PreviewGenerator::new())),
        ServiceEntry::new::<SafetyChecker>(Arc::new(SafetyChecker::new())),
        ServiceEntry::new::<ValidationEngine>(Arc::new(ValidationEngine::new())),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_common::di::list_discovered_factories;

    #[test]
    fn test_refactoring_factory_registered() {
        let factories = list_discovered_factories();
        assert!(factories.contains(&"refactoring"), "Factory should be registered");
    }
}
