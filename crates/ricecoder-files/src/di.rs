//! Dependency injection support for ricecoder-files
//!
//! This module provides automatic service registration using the inventory crate's
//! factory-return pattern. Services are created and returned for registration by
//! ricecoder-di during container initialization.

use std::sync::Arc;

use ricecoder_common::di::{ServiceEntry, ServiceFactory};

use crate::manager::FileManager;

// Auto-register file services with the DI container
inventory::submit! {
    ServiceFactory::new("files", create_files_services)
}

/// Create all file services for registration.
///
/// This factory function creates instances of all file services and returns them
/// as `ServiceEntry` items. ricecoder-di collects these and registers them in its container.
fn create_files_services() -> Vec<ServiceEntry> {
    vec![
        // FileManager - Main file operations coordinator
        ServiceEntry::new::<FileManager>(Arc::new(FileManager::new())),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_common::di::list_discovered_factories;

    #[test]
    fn test_files_factory_registered() {
        let factories = list_discovered_factories();
        assert!(
            factories.contains(&"files"),
            "Files factory should be registered"
        );
    }

    #[test]
    fn test_create_files_services() {
        let services = create_files_services();
        assert!(!services.is_empty(), "Should create at least one service");

        // Check that FileManager is in the list
        let has_file_manager = services.iter().any(|s| {
            s.type_name.contains("FileManager")
        });
        assert!(has_file_manager, "Should include FileManager");
    }
}
