//! Dependency injection support for ricecoder-execution
//!
//! This module provides automatic service registration using the inventory crate's
//! factory-return pattern. Services are created and returned for registration by
//! ricecoder-di during container initialization.

use std::sync::Arc;

use ricecoder_common::di::{ServiceEntry, ServiceFactory};

use crate::ExecutionManager;

// Auto-register execution services with the DI container
inventory::submit! {
    ServiceFactory::new("execution", create_execution_services)
}

/// Create all execution services for registration.
///
/// This factory function creates instances of all execution services and returns them
/// as `ServiceEntry` items. ricecoder-di collects these and registers them in its container.
fn create_execution_services() -> Vec<ServiceEntry> {
    vec![
        // ExecutionManager - Main execution coordination
        ServiceEntry::new::<ExecutionManager>(Arc::new(ExecutionManager::new())),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_common::di::list_discovered_factories;

    #[test]
    fn test_execution_factory_registered() {
        let factories = list_discovered_factories();
        assert!(
            factories.contains(&"execution"),
            "Execution factory should be registered"
        );
    }

    #[test]
    fn test_create_execution_services() {
        let services = create_execution_services();
        assert!(!services.is_empty(), "Should create at least one service");

        // Check that ExecutionManager is in the list
        let has_execution_manager = services.iter().any(|s| {
            s.type_name.contains("ExecutionManager")
        });
        assert!(has_execution_manager, "Should include ExecutionManager");
    }
}
