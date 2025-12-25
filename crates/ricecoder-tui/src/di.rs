//! Dependency injection support for ricecoder-tui
//!
//! This module provides factory-return DI pattern for TUI services.
//! Services are registered via `inventory::submit!` and collected by ricecoder-di.
//!
//! ## Architecture
//!
//! The TUI crate uses the factory-return pattern instead of global statics:
//! - Services are created and returned as `ServiceEntry` items
//! - ricecoder-di collects all entries at container initialization
//! - No global `OnceLock` needed - container is passed through the app
//!
//! ## Migration from Global Static
//!
//! Previously: `static DI_CONTAINER: OnceLock<Arc<DIContainer>>`
//! Now: Container passed as parameter or resolved from parent container

use std::sync::Arc;

use ricecoder_common::di::{ServiceEntry, ServiceFactory};
use ricecoder_di::{DIContainer, DIResult};

use crate::lifecycle::TuiLifecycleManager;

// Auto-register TUI services with the DI container
inventory::submit! {
    ServiceFactory::new("tui", create_tui_services)
}

/// Create all TUI services for registration.
///
/// This factory function creates instances of all TUI services and returns them
/// as `ServiceEntry` items. ricecoder-di collects these and registers them in its container.
fn create_tui_services() -> Vec<ServiceEntry> {
    vec![
        // TuiLifecycleManager - Component lifecycle management
        ServiceEntry::new::<TuiLifecycleManager>(Arc::new(TuiLifecycleManager::new())),
    ]
}

/// Get a service from a DI container
///
/// Helper function to resolve services from a container reference.
/// The container should be passed through the application, not accessed globally.
pub fn get_service<T>(container: &DIContainer) -> Option<Arc<T>>
where
    T: Send + Sync + 'static,
{
    container.resolve::<T>().ok()
}

/// Get the TUI lifecycle manager from the container
pub fn get_lifecycle_manager(container: &DIContainer) -> Option<Arc<TuiLifecycleManager>> {
    get_service::<TuiLifecycleManager>(container)
}

// =============================================================================
// Legacy API - Kept for backward compatibility during migration
// =============================================================================

use std::sync::OnceLock;

/// Legacy: Global DI container for the TUI application
/// 
/// **DEPRECATED**: Pass container through application instead.
/// This is kept for backward compatibility during migration.
#[deprecated(since = "0.2.0", note = "Pass DIContainer through application instead of using global static")]
static DI_CONTAINER: OnceLock<Arc<DIContainer>> = OnceLock::new();

/// Legacy: Initialize the DI container for the TUI
/// 
/// **DEPRECATED**: Use `ricecoder_di::create_application_container()` and pass the container.
#[deprecated(since = "0.2.0", note = "Use create_application_container() and pass container through app")]
#[allow(deprecated)]
pub fn initialize_di_container() -> DIResult<()> {
    let container = ricecoder_di::create_application_container()?;
    DI_CONTAINER.set(Arc::new(container)).map_err(|_| {
        ricecoder_di::DIError::ServiceAlreadyRegistered {
            service_type: "DIContainer".to_string(),
        }
    })?;
    Ok(())
}

/// Legacy: Get the global DI container
/// 
/// **DEPRECATED**: Pass container through application instead.
#[deprecated(since = "0.2.0", note = "Pass DIContainer through application instead of using global static")]
#[allow(deprecated)]
pub fn get_di_container() -> Option<Arc<DIContainer>> {
    DI_CONTAINER.get().cloned()
}

/// Legacy: Get a service from the global DI container
/// 
/// **DEPRECATED**: Use `get_service(container)` instead.
#[deprecated(since = "0.2.0", note = "Use get_service(container) with explicit container reference")]
#[allow(deprecated)]
pub fn get_service_global<T>() -> Option<Arc<T>>
where
    T: Send + Sync + 'static,
{
    get_di_container().and_then(|container| container.resolve::<T>().ok())
}

/// Legacy: Initialize DI container with specific features
/// 
/// **DEPRECATED**: Features are controlled by cargo features and linking.
#[deprecated(since = "0.2.0", note = "Features controlled by cargo features, not runtime")]
#[allow(deprecated)]
pub fn initialize_di_container_with_features(_features: &[&str]) -> DIResult<()> {
    initialize_di_container()
}

/// Legacy: Check if DI container is initialized
/// 
/// **DEPRECATED**: Container state is managed by the application.
#[deprecated(since = "0.2.0", note = "Container state managed by application")]
#[allow(deprecated)]
pub fn is_di_initialized() -> bool {
    DI_CONTAINER.get().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_common::di::list_discovered_factories;

    #[test]
    fn test_tui_factory_registered() {
        let factories = list_discovered_factories();
        assert!(factories.contains(&"tui"), "TUI factory should be registered");
    }

    #[test]
    fn test_create_tui_services() {
        let services = create_tui_services();
        assert!(!services.is_empty(), "Should create at least one service");
        
        // Check that TuiLifecycleManager is present
        let has_lifecycle = services.iter().any(|s| s.type_name.contains("TuiLifecycleManager"));
        assert!(has_lifecycle, "Should include TuiLifecycleManager");
    }
}
