//! Dependency injection support for ricecoder-learning
//!
//! This module provides automatic service registration using the inventory crate's
//! factory-return pattern. Services are created and returned for registration by
//! ricecoder-di during container initialization.

use std::sync::Arc;

use ricecoder_common::di::{ServiceEntry, ServiceFactory};

use crate::{manager::LearningManager, models::RuleScope};

// Auto-register learning services with the DI container
inventory::submit! {
    ServiceFactory::new("learning", create_learning_services)
}

/// Create all learning services for registration.
///
/// This factory function creates instances of all learning services and returns them
/// as `ServiceEntry` items. ricecoder-di collects these and registers them in its container.
fn create_learning_services() -> Vec<ServiceEntry> {
    vec![
        // LearningManager - Main learning system coordinator (Global scope)
        ServiceEntry::new::<LearningManager>(Arc::new(LearningManager::new(RuleScope::Global))),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_common::di::list_discovered_factories;

    #[test]
    fn test_learning_factory_registered() {
        let factories = list_discovered_factories();
        assert!(
            factories.contains(&"learning"),
            "Learning factory should be registered"
        );
    }

    #[test]
    fn test_create_learning_services() {
        let services = create_learning_services();
        assert!(!services.is_empty(), "Should create at least one service");

        // Check that LearningManager is in the list
        let has_learning_manager = services.iter().any(|s| {
            s.type_name.contains("LearningManager")
        });
        assert!(has_learning_manager, "Should include LearningManager");
    }
}
