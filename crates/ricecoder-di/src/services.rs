//! Service registration for the DI container
//!
//! This module provides functions to register all the services
//! needed by the RiceCoder application across all crates.
//!
//! ## Factory-Return Pattern (Current Approach)
//!
//! Services are registered via the factory-return pattern in each crate's `di.rs`.
//! The `register_discovered_services` function collects all services from all crates
//! and registers them in the container.
//!
//! Each crate uses `inventory::submit!` with a `ServiceFactory` that returns
//! `Vec<ServiceEntry>` containing its services.

use std::sync::Arc;
use ricecoder_common::di::{collect_all_services, discovered_factory_count, list_discovered_factories};

use async_trait::async_trait;
use ricecoder_agents::use_cases::{
    ProviderCommunityUseCase, ProviderFailoverUseCase, ProviderHealthUseCase, ProviderModelUseCase,
    ProviderPerformanceUseCase, ProviderSwitchingUseCase, SessionLifecycleUseCase,
    SessionSharingUseCase, SessionStateManagementUseCase,
};
use ricecoder_providers::provider::manager::ProviderManager;
use ricecoder_sessions::{SessionManager, SessionStore, ShareService};

use crate::{DIContainer, DIResult};

/// Trait for services that need lifecycle management
#[async_trait]
pub trait Lifecycle: Send + Sync {
    /// Initialize the service (called after all dependencies are resolved)
    async fn initialize(&self) -> DIResult<()> {
        Ok(())
    }

    /// Cleanup the service (called during shutdown)
    async fn cleanup(&self) -> DIResult<()> {
        Ok(())
    }
}

/// Lifecycle manager for handling service initialization and cleanup
pub struct LifecycleManager {
    services: Vec<Arc<dyn Lifecycle>>,
}

impl LifecycleManager {
    /// Create a new lifecycle manager
    pub fn new() -> Self {
        Self {
            services: Vec::new(),
        }
    }

    /// Register a service for lifecycle management
    pub fn register_service(&mut self, service: Arc<dyn Lifecycle>) {
        self.services.push(service);
    }

    /// Initialize all registered services
    pub async fn initialize_all(&self) -> DIResult<()> {
        for service in &self.services {
            service.initialize().await?;
        }
        Ok(())
    }

    /// Cleanup all registered services
    pub async fn cleanup_all(&self) -> DIResult<()> {
        // Cleanup in reverse order
        for service in self.services.iter().rev() {
            service.cleanup().await?;
        }
        Ok(())
    }
}

impl Default for LifecycleManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Register all discovered services from crates using the factory-return pattern.
///
/// This function collects all services from crates that use `inventory::submit!`
/// with a `ServiceFactory` and registers them in the container.
///
/// ## Example
///
/// ```rust,ignore
/// use ricecoder_di::{DIContainer, register_discovered_services};
///
/// let container = DIContainer::new();
/// register_discovered_services(&container)?;
/// ```
pub fn register_discovered_services(container: &DIContainer) -> DIResult<()> {
    let factory_count = discovered_factory_count();
    let factory_names = list_discovered_factories();
    
    tracing::info!(
        "Registering services from {} discovered factories: {:?}",
        factory_count,
        factory_names
    );

    let services = collect_all_services();
    let service_count = services.len();

    for entry in services {
        container.register_entry(entry)?;
    }

    tracing::info!(
        "Registered {} services from factory-return pattern",
        service_count
    );

    Ok(())
}

/// Register all infrastructure services
///
/// This function:
/// 1. First registers all discovered services from crates using the factory-return pattern
/// 2. Then registers core session and provider infrastructure
///
/// Most service registration is now handled by the factory-return pattern in each crate's di.rs.
/// This function provides a central entry point that collects all services.
pub fn register_infrastructure_services(container: &DIContainer) -> DIResult<()> {
    // First, register all discovered services from factory-return pattern
    // This collects services from all crates that use inventory::submit!
    register_discovered_services(container)?;

    // Register core session infrastructure (not using factory-return pattern yet)
    container.register(|_| {
        let session_store = Arc::new(SessionStore::new().map_err(|e| {
            crate::DIError::DependencyResolutionFailed {
                message: format!("Failed to create session store: {}", e),
            }
        })?);
        Ok(session_store)
    })?;

    container.register(|_| {
        let session_manager = Arc::new(SessionManager::new(10)); // max 10 sessions
        Ok(session_manager)
    })?;

    container.register(|_| {
        let share_service = Arc::new(ShareService::new());
        Ok(share_service)
    })?;

    // Register provider infrastructure
    container.register(|_| {
        let registry = ricecoder_providers::provider::ProviderRegistry::new();
        let provider_manager = Arc::new(ProviderManager::new(registry, "openai".to_string()));
        Ok(provider_manager)
    })?;

    Ok(())
}

/// Register all application use cases
pub fn register_use_cases(container: &DIContainer) -> DIResult<()> {
    // Register session use cases
    container.register(|container| {
        let session_manager = container.resolve::<SessionManager>()?;
        let session_store = container.resolve::<SessionStore>()?;
        let use_case = Arc::new(SessionLifecycleUseCase::new(session_manager, session_store));
        Ok(use_case)
    })?;

    container.register(|container| {
        let share_service = container.resolve::<ShareService>()?;
        let session_store = container.resolve::<SessionStore>()?;
        let use_case = Arc::new(SessionSharingUseCase::new(share_service, session_store));
        Ok(use_case)
    })?;

    container.register(|container| {
        let session_manager = container.resolve::<SessionManager>()?;
        let use_case = Arc::new(SessionStateManagementUseCase::new(session_manager));
        Ok(use_case)
    })?;

    // Register provider use cases
    container.register(|container| {
        let provider_manager = container.resolve::<ProviderManager>()?;
        let use_case = Arc::new(ProviderSwitchingUseCase::new(provider_manager));
        Ok(use_case)
    })?;

    container.register(|container| {
        let provider_manager = container.resolve::<ProviderManager>()?;
        let use_case = Arc::new(ProviderPerformanceUseCase::new(provider_manager));
        Ok(use_case)
    })?;

    container.register(|container| {
        let provider_manager = container.resolve::<ProviderManager>()?;
        let use_case = Arc::new(ProviderFailoverUseCase::new(provider_manager));
        Ok(use_case)
    })?;

    container.register(|container| {
        let provider_manager = container.resolve::<ProviderManager>()?;
        let use_case = Arc::new(ProviderModelUseCase::new(provider_manager));
        Ok(use_case)
    })?;

    container.register(|container| {
        let provider_manager = container.resolve::<ProviderManager>()?;
        let use_case = Arc::new(ProviderHealthUseCase::new(provider_manager));
        Ok(use_case)
    })?;

    container.register(|container| {
        let provider_manager = container.resolve::<ProviderManager>()?;
        let use_case = Arc::new(ProviderCommunityUseCase::new(provider_manager));
        Ok(use_case)
    })?;

    Ok(())
}

/// Register all services using the builder pattern
pub fn create_application_container() -> DIResult<DIContainer> {
    let mut builder = crate::DIContainerBuilder::new();

    // Register core services (always available)
    builder = builder.register_infrastructure_services()?;
    builder = builder.register_use_cases()?;

    let container = builder.build();
    Ok(container)
}

/// Create a full-featured container with all optional services enabled
#[cfg(feature = "full")]
pub fn create_full_application_container() -> DIResult<DIContainer> {
    let container = crate::DIContainerBuilder::new()
        .register_infrastructure_services()?
        .register_use_cases()?
        .build();

    Ok(container)
}

/// Create a CLI-only container with minimal services
pub fn create_cli_container() -> DIResult<DIContainer> {
    let builder = crate::DIContainerBuilder::new();

    // Register core infrastructure (includes all discovered services)
    let builder = builder.register_infrastructure_services()?;

    let container = builder.build();
    Ok(container)
}

/// Create a TUI-only container with UI-focused services
pub fn create_tui_container() -> DIResult<DIContainer> {
    let builder = crate::DIContainerBuilder::new();

    // Register core infrastructure (includes all discovered services)
    let builder = builder.register_infrastructure_services()?;

    let container = builder.build();
    Ok(container)
}

/// Create a development container with additional debugging services
pub fn create_development_container() -> DIResult<DIContainer> {
    let builder = crate::DIContainerBuilder::new();

    // Register all core services (includes all discovered services)
    let builder = builder.register_infrastructure_services()?;
    let builder = builder.register_use_cases()?;

    let container = builder.build();
    Ok(container)
}

/// Create a minimal container for testing
pub fn create_test_container() -> DIResult<DIContainer> {
    let builder = crate::DIContainerBuilder::new();

    // Register minimal infrastructure for testing
    let builder = builder.register_infrastructure_services()?;

    let container = builder.build();
    Ok(container)
}

/// Configuration for container creation
///
/// Note: With the factory-return pattern, all services registered via `inventory::submit!`
/// are automatically included. This configuration is kept for API compatibility but
/// the flags have no effect - all linked services are registered.
#[derive(Debug, Clone, Default)]
pub struct ContainerConfig {
    // All fields are kept for API compatibility but are no longer used.
    // Services are automatically registered via factory-return pattern.
    _private: (),
}

/// Create a container based on configuration
///
/// Note: With the factory-return pattern, all services registered via `inventory::submit!`
/// are automatically included. The configuration parameter is kept for API compatibility.
pub fn create_configured_container(_config: &ContainerConfig) -> DIResult<DIContainer> {
    let builder = crate::DIContainerBuilder::new();

    // Register all core services (includes all discovered services)
    let builder = builder.register_infrastructure_services()?;
    let builder = builder.register_use_cases()?;

    let container = builder.build();
    Ok(container)
}

// ============================================================================
// Extension trait for DIContainerBuilder
// ============================================================================

/// Extension trait for DIContainerBuilder to add service registration methods
pub trait DIContainerBuilderExt {
    /// Register infrastructure services
    fn register_infrastructure_services(self) -> DIResult<Self> where Self: Sized;
    /// Register use cases
    fn register_use_cases(self) -> DIResult<Self> where Self: Sized;
}

impl DIContainerBuilderExt for crate::DIContainerBuilder {
    fn register_infrastructure_services(self) -> DIResult<Self> {
        register_infrastructure_services(&self.container)?;
        Ok(self)
    }

    fn register_use_cases(self) -> DIResult<Self> {
        register_use_cases(&self.container)?;
        Ok(self)
    }
}

// Private field access for DIContainerBuilder
impl crate::DIContainerBuilder {
    /// Get a reference to the inner container
    pub(crate) fn container(&self) -> &DIContainer {
        &self.container
    }
}
