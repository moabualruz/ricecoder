//! Auto-discovery service registration using inventory crate
//!
//! This module provides the infrastructure for distributed service registration.
//! Each crate can register its services using `inventory::submit!` and they will
//! be automatically discovered at runtime.
//!
//! ## Usage
//!
//! In a feature crate (e.g., ricecoder-storage):
//!
//! ```rust,ignore
//! use ricecoder_di::{DIContainer, DIResult, ServiceRegistration};
//! use std::sync::Arc;
//!
//! inventory::submit! {
//!     ServiceRegistration::new(
//!         "storage",
//!         |container| {
//!             container.register(|_| Ok(Arc::new(StorageManager::new())))?;
//!             Ok(())
//!         }
//!     )
//! }
//! ```
//!
//! In ricecoder-di, all registrations are collected and executed:
//!
//! ```rust,ignore
//! use ricecoder_di::registration::register_all_discovered_services;
//!
//! let container = DIContainer::new();
//! register_all_discovered_services(&container)?;
//! ```

use crate::{DIContainer, DIResult};
use tracing::{debug, info};

/// A service registration descriptor that can be collected via inventory.
///
/// Each crate submits one or more `ServiceRegistration` entries that will be
/// automatically discovered and executed when the DI container is initialized.
pub struct ServiceRegistration {
    /// Name of the service group (e.g., "storage", "research", "mcp")
    pub name: &'static str,

    /// Registration function that registers services with the container
    pub register_fn: fn(&DIContainer) -> DIResult<()>,

    /// Priority for registration order (lower = earlier, default = 100)
    pub priority: u32,

    /// Dependencies - names of other service groups that must be registered first
    pub dependencies: &'static [&'static str],
}

impl ServiceRegistration {
    /// Create a new service registration with default priority
    pub const fn new(name: &'static str, register_fn: fn(&DIContainer) -> DIResult<()>) -> Self {
        Self {
            name,
            register_fn,
            priority: 100,
            dependencies: &[],
        }
    }

    /// Create a new service registration with custom priority
    pub const fn with_priority(
        name: &'static str,
        register_fn: fn(&DIContainer) -> DIResult<()>,
        priority: u32,
    ) -> Self {
        Self {
            name,
            register_fn,
            priority,
            dependencies: &[],
        }
    }

    /// Create a new service registration with dependencies
    pub const fn with_dependencies(
        name: &'static str,
        register_fn: fn(&DIContainer) -> DIResult<()>,
        dependencies: &'static [&'static str],
    ) -> Self {
        Self {
            name,
            register_fn,
            priority: 100,
            dependencies,
        }
    }

    /// Create a new service registration with priority and dependencies
    pub const fn full(
        name: &'static str,
        register_fn: fn(&DIContainer) -> DIResult<()>,
        priority: u32,
        dependencies: &'static [&'static str],
    ) -> Self {
        Self {
            name,
            register_fn,
            priority,
            dependencies,
        }
    }
}

// Collect all ServiceRegistration instances across all crates
inventory::collect!(ServiceRegistration);

/// Register all discovered services with the container.
///
/// This function iterates over all `ServiceRegistration` entries that were
/// submitted via `inventory::submit!` across all linked crates and executes
/// their registration functions.
///
/// Services are registered in priority order (lower priority value = earlier).
pub fn register_all_discovered_services(container: &DIContainer) -> DIResult<()> {
    // Collect all registrations
    let mut registrations: Vec<&ServiceRegistration> = inventory::iter::<ServiceRegistration>().collect();

    // Sort by priority (stable sort to preserve insertion order for equal priorities)
    registrations.sort_by_key(|r| r.priority);

    info!(
        "Discovered {} service registrations via inventory",
        registrations.len()
    );

    // Execute each registration
    for registration in registrations {
        debug!(
            "Registering services for '{}' (priority: {})",
            registration.name, registration.priority
        );
        (registration.register_fn)(container)?;
    }

    info!("All discovered services registered successfully");
    Ok(())
}

/// Get the count of discovered service registrations.
///
/// Useful for debugging and testing.
pub fn discovered_registration_count() -> usize {
    inventory::iter::<ServiceRegistration>().count()
}

/// List all discovered service registration names.
///
/// Useful for debugging and diagnostics.
pub fn list_discovered_registrations() -> Vec<&'static str> {
    inventory::iter::<ServiceRegistration>()
        .map(|r| r.name)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test registration (will be collected by inventory)
    inventory::submit! {
        ServiceRegistration::new("test_service", |_container| {
            Ok(())
        })
    }

    #[test]
    fn test_discovered_registrations_include_test() {
        let names = list_discovered_registrations();
        assert!(
            names.contains(&"test_service"),
            "Should discover test_service registration"
        );
    }

    #[test]
    fn test_register_all_discovered() {
        let container = DIContainer::new();
        let result = register_all_discovered_services(&container);
        assert!(result.is_ok(), "Should register all discovered services");
    }
}
