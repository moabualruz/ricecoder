//! Dependency injection support for ricecoder-storage
//!
//! This module provides automatic service registration using the inventory crate's
//! factory-return pattern. Services are created and returned for registration by
//! ricecoder-di during container initialization.
//!
//! ## Pattern
//!
//! ```rust,ignore
//! inventory::submit! {
//!     ServiceFactory::new("storage", create_storage_services)
//! }
//!
//! fn create_storage_services() -> Vec<ServiceEntry> {
//!     vec![
//!         ServiceEntry::new::<StorageManager>(Arc::new(StorageManager::new())),
//!     ]
//! }
//! ```

use std::sync::Arc;

use ricecoder_common::di::{ServiceEntry, ServiceFactory};

use crate::{CacheManager, GlobalStore, PathResolver, ProjectStore, SessionManager};

// Auto-register storage services with the DI container
inventory::submit! {
    ServiceFactory::new("storage", create_storage_services)
}

/// Create all storage services for registration.
///
/// This factory function creates instances of all storage services and returns them
/// as `ServiceEntry` items. ricecoder-di collects these and registers them in its container.
fn create_storage_services() -> Vec<ServiceEntry> {
    let mut services = Vec::new();

    // Try to resolve global path for cache and session storage
    if let Ok(global_path) = PathResolver::resolve_global_path() {
        // CacheManager - Main caching infrastructure
        let cache_dir = global_path.join("cache");
        if let Ok(cache_manager) = CacheManager::new(&cache_dir) {
            services.push(ServiceEntry::new::<CacheManager>(Arc::new(cache_manager)));
        }

        // SessionManager - Session persistence
        let sessions_dir = global_path.join("sessions");
        services.push(ServiceEntry::new::<SessionManager>(Arc::new(
            SessionManager::new(sessions_dir),
        )));
    }

    // GlobalStore - Global knowledge base storage (if path resolution succeeds)
    if let Ok(global_store) = GlobalStore::with_default_path() {
        services.push(ServiceEntry::new::<GlobalStore>(Arc::new(global_store)));
    }

    // ProjectStore - Project-local storage
    let project_store = ProjectStore::with_default_path();
    services.push(ServiceEntry::new::<ProjectStore>(Arc::new(project_store)));

    services
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_common::di::list_discovered_factories;

    #[test]
    fn test_storage_factory_registered() {
        let factories = list_discovered_factories();
        assert!(
            factories.contains(&"storage"),
            "Storage factory should be registered"
        );
    }

    #[test]
    fn test_create_storage_services() {
        let services = create_storage_services();
        assert!(!services.is_empty(), "Should create at least one service");

        // Check that CacheManager is in the list
        let has_cache_manager = services.iter().any(|s| {
            s.type_name.contains("CacheManager")
        });
        assert!(has_cache_manager, "Should include CacheManager");

        // Check that SessionManager is in the list
        let has_session_manager = services.iter().any(|s| {
            s.type_name.contains("SessionManager")
        });
        assert!(has_session_manager, "Should include SessionManager");
    }
}
