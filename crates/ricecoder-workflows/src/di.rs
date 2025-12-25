//! Dependency injection support for ricecoder-workflows
//!
//! This module provides automatic service registration using the inventory crate's
//! factory-return pattern. Services are created and returned for registration by
//! ricecoder-di during container initialization.

use std::sync::Arc;

use ricecoder_common::di::{ServiceEntry, ServiceFactory};

use crate::WorkflowEngine;

// Auto-register workflow services with the DI container
inventory::submit! {
    ServiceFactory::new("workflows", create_workflow_services)
}

/// Create all workflow services for registration.
///
/// This factory function creates instances of all workflow services and returns them
/// as `ServiceEntry` items. ricecoder-di collects these and registers them in its container.
fn create_workflow_services() -> Vec<ServiceEntry> {
    vec![
        // WorkflowEngine - Main workflow execution coordinator
        ServiceEntry::new::<WorkflowEngine>(Arc::new(WorkflowEngine::new())),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_common::di::list_discovered_factories;

    #[test]
    fn test_workflow_factory_registered() {
        let factories = list_discovered_factories();
        assert!(
            factories.contains(&"workflows"),
            "Workflows factory should be registered"
        );
    }

    #[test]
    fn test_create_workflow_services() {
        let services = create_workflow_services();
        assert!(!services.is_empty(), "Should create at least one service");

        // Check that WorkflowEngine is in the list
        let has_workflow_engine = services.iter().any(|s| {
            s.type_name.contains("WorkflowEngine")
        });
        assert!(has_workflow_engine, "Should include WorkflowEngine");
    }
}
