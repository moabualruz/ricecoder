//! Dependency injection support for ricecoder-research
//!
//! This module provides automatic service registration using the inventory crate's
//! factory-return pattern. Services are created and returned for registration by
//! ricecoder-di during container initialization.

use std::sync::Arc;

use ricecoder_common::di::{ServiceEntry, ServiceFactory};

use crate::ResearchManager;

// Auto-register research services with the DI container
inventory::submit! {
    ServiceFactory::new("research", create_research_services)
}

/// Create all research services for registration.
///
/// This factory function creates instances of all research services and returns them
/// as `ServiceEntry` items. ricecoder-di collects these and registers them in its container.
fn create_research_services() -> Vec<ServiceEntry> {
    vec![
        // ResearchManager - Main research coordination
        ServiceEntry::new::<ResearchManager>(Arc::new(ResearchManager::new())),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_common::di::list_discovered_factories;

    #[test]
    fn test_research_factory_registered() {
        let factories = list_discovered_factories();
        assert!(
            factories.contains(&"research"),
            "Research factory should be registered"
        );
    }

    #[test]
    fn test_create_research_services() {
        let services = create_research_services();
        assert!(!services.is_empty(), "Should create at least one service");

        // Check that ResearchManager is in the list
        let has_research_manager = services.iter().any(|s| {
            s.type_name.contains("ResearchManager")
        });
        assert!(has_research_manager, "Should include ResearchManager");
    }
}
