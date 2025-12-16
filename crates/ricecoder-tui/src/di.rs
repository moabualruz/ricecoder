//! DI integration for the TUI
//!
//! This module provides DI container setup and service resolution
//! for the RiceCoder TUI application.

use ricecoder_di::{create_application_container, DIContainer, DIResult};
use std::sync::{Arc, OnceLock};

/// Global DI container for the TUI application
static DI_CONTAINER: OnceLock<Arc<DIContainer>> = OnceLock::new();

/// Initialize the DI container for the TUI
pub fn initialize_di_container() -> DIResult<()> {
    let container = create_application_container()?;
    DI_CONTAINER.set(Arc::new(container)).map_err(|_| ricecoder_di::DIError::ServiceAlreadyRegistered { service_type: "DIContainer".to_string() })?;
    Ok(())
}

/// Get the global DI container
pub fn get_di_container() -> Option<Arc<DIContainer>> {
    DI_CONTAINER.get().cloned()
}

/// Get a service from the DI container
pub fn get_service<T>() -> Option<Arc<T>>
where
    T: Send + Sync + 'static,
{
    get_di_container()
        .and_then(|container| container.resolve::<T>().ok())
}

/// Initialize DI container with specific features
/// This allows TUI to opt into specific service groups
pub fn initialize_di_container_with_features(features: &[&str]) -> DIResult<()> {
    // For now, just use the default container
    // In the future, this could conditionally enable features
    initialize_di_container()
}

/// Check if DI container is initialized
pub fn is_di_initialized() -> bool {
    DI_CONTAINER.get().is_some()
}

/// Reset the DI container (mainly for testing)
#[cfg(test)]
pub fn reset_di_container() {
    // OnceLock cannot be reset, so this is a no-op for now
    // In tests, use separate containers
}

