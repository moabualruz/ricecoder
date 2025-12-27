//! Dependency injection support for ricecoder-cli
//!
//! This module provides factory-return DI pattern for CLI services.
//! Services are registered via `inventory::submit!` and collected by ricecoder-di.

use std::sync::{Arc, OnceLock};
use ricecoder_common::di::{ServiceEntry, ServiceFactory};
use ricecoder_di::DIContainer;
use crate::{
    BrandingManager,
    CommandRouter,
    AccessibilitySettings,
    lifecycle::LifecycleManager,
};

/// Global DI container instance
static CONTAINER: OnceLock<DIContainer> = OnceLock::new();

/// Get a service from the global DI container.
pub fn get_service<T>() -> Option<Arc<T>>
where
    T: Send + Sync + 'static,
{
    CONTAINER.get().and_then(|container| container.resolve::<T>().ok())
}

/// Initialize the DI container.
///
/// Creates and configures the application container with all required services.
pub fn initialize_di_container() -> Result<(), String> {
    let container = ricecoder_di::create_application_container()
        .map_err(|e| format!("Failed to create DI container: {}", e))?;
    
    CONTAINER.set(container).map_err(|_| "DI container already initialized".to_string())?;
    
    Ok(())
}

inventory::submit! {
    ServiceFactory::new("cli", create_cli_services)
}

/// Create all CLI services for registration.
///
/// This factory function creates instances of all CLI services and returns them
/// as `ServiceEntry` items. ricecoder-di collects these and registers them in its container.
fn create_cli_services() -> Vec<ServiceEntry> {
    vec![
        // LifecycleManager - Component lifecycle management (replaces global static)
        ServiceEntry::new::<LifecycleManager>(Arc::new(LifecycleManager::new())),
        // BrandingManager - Application branding (unit struct)
        ServiceEntry::new::<BrandingManager>(Arc::new(BrandingManager)),
        // CommandRouter - Command routing (unit struct)
        ServiceEntry::new::<CommandRouter>(Arc::new(CommandRouter)),
        // AccessibilitySettings - Accessibility configuration
        ServiceEntry::new::<AccessibilitySettings>(Arc::new(AccessibilitySettings::default())),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_common::di::list_discovered_factories;

    #[test]
    fn test_cli_factory_registered() {
        let factories = list_discovered_factories();
        assert!(factories.contains(&"cli"), "Factory should be registered");
    }
}
