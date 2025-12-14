//! DI integration for the TUI
//!
//! This module provides DI container setup and service resolution
//! for the RiceCoder TUI application.

use ricecoder_di::{create_application_container, DIContainer, DIResult};
use std::sync::Arc;

/// Global DI container for the TUI application
static mut DI_CONTAINER: Option<Arc<DIContainer>> = None;

/// Initialize the DI container for the TUI
pub fn initialize_di_container() -> DIResult<()> {
    let container = create_application_container()?;
    unsafe {
        DI_CONTAINER = Some(Arc::new(container));
    }
    Ok(())
}

/// Get the global DI container
pub fn get_di_container() -> Option<Arc<DIContainer>> {
    unsafe { DI_CONTAINER.clone() }
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
    unsafe { DI_CONTAINER.is_some() }
}

/// Reset the DI container (mainly for testing)
#[cfg(test)]
pub fn reset_di_container() {
    unsafe {
        DI_CONTAINER = None;
    }
}

