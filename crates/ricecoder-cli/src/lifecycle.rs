//! Component lifecycle management
//!
//! This module provides lifecycle management for application components,
//! ensuring proper startup, initialization, and shutdown procedures.

use std::sync::{Arc, OnceLock, RwLock};
use tracing::{debug, info, warn};
use ricecoder_di::DIContainer;

/// Component lifecycle states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleState {
    /// Component is not yet initialized
    Uninitialized,
    /// Component is initializing
    Initializing,
    /// Component is ready to use
    Ready,
    /// Component is shutting down
    ShuttingDown,
    /// Component has shut down
    ShutDown,
    /// Component encountered an error
    Error,
}

/// Component lifecycle trait
#[async_trait::async_trait]
pub trait LifecycleComponent: Send + Sync {
    /// Get the component name
    fn name(&self) -> &'static str;

    /// Initialize the component
    async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Start the component (if it has background tasks)
    async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }

    /// Stop the component
    async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }

    /// Check if component is healthy
    async fn health_check(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}

/// Component registration info
#[derive(Clone)]
struct ComponentInfo {
    component: Arc<RwLock<dyn LifecycleComponent>>,
    state: LifecycleState,
    dependencies: Vec<String>,
}

/// Component lifecycle manager
pub struct LifecycleManager {
    components: RwLock<Vec<ComponentInfo>>,
}

impl LifecycleManager {
    /// Create a new lifecycle manager
    pub fn new() -> Self {
        Self {
            components: RwLock::new(Vec::new()),
        }
    }

    /// Register a component with optional dependencies
    pub fn register_component<C>(
        &self,
        component: C,
        dependencies: Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        C: LifecycleComponent + 'static,
    {
        let info = ComponentInfo {
            component: Arc::new(RwLock::new(component)),
            state: LifecycleState::Uninitialized,
            dependencies,
        };

        let mut components = self.components.write().unwrap();
        components.push(info.clone());

        info!("Registered component: {}", info.component.read().unwrap().name());
        Ok(())
    }

    /// Initialize all components in dependency order
    pub async fn initialize_all(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Initializing all components...");

        // Sort components by dependencies (simple topological sort)
        self.sort_by_dependencies()?;

        let components_guard = self.components.read().unwrap();
        let components: Vec<_> = components_guard.iter().map(|info| info.component.clone()).collect();
        drop(components_guard);

        for comp_arc in components.into_iter() {
            let mut component = comp_arc.write().unwrap();
            let name = component.name();

            debug!("Initializing component: {}", name);

            // Update state
            self.update_component_state(name, LifecycleState::Initializing);

            match component.initialize().await {
                Ok(()) => {
                    debug!("Component initialized successfully: {}", name);
                    drop(component);
                    self.update_component_state(name, LifecycleState::Ready);
                }
                Err(e) => {
                    warn!("Component initialization failed: {} - {}", name, e);
                    drop(component);
                    self.update_component_state(name, LifecycleState::Error);
                    return Err(e);
                }
            }
        }

        info!("All components initialized successfully");
        Ok(())
    }

    /// Start all components
    pub async fn start_all(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting all components...");

        let components = self.components.read().unwrap();

        for info in components.iter() {
            if info.state != LifecycleState::Ready {
                continue;
            }

            let mut component = info.component.write().unwrap();
            let name = component.name();

            debug!("Starting component: {}", name);

            match component.start().await {
                Ok(()) => {
                    debug!("Component started successfully: {}", name);
                }
                Err(e) => {
                    warn!("Component start failed: {} - {}", name, e);
                    return Err(e);
                }
            }
        }

        info!("All components started successfully");
        Ok(())
    }

    /// Stop all components in reverse order
    pub async fn stop_all(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Stopping all components...");

        let components_guard = self.components.read().unwrap();
        let components: Vec<_> = components_guard.iter().map(|info| info.component.clone()).collect();
        drop(components_guard);

        // Stop in reverse order
        for comp_arc in components.into_iter().rev() {
            let mut component = comp_arc.write().unwrap();
            let name = component.name();
            info!("Stopping component: {}", name);
            if let Err(e) = component.stop().await {
                warn!("Component stop failed: {} - {}", name, e);
            }
        }

        info!("All components stopped");
        Ok(())
    }

    /// Perform health checks on all components
    pub async fn health_check_all(&self) -> Vec<(String, Result<(), Box<dyn std::error::Error + Send + Sync>>)> {
        let components_guard = self.components.read().unwrap();
        let infos: Vec<_> = components_guard.iter().cloned().collect();
        drop(components_guard);
        let mut results = Vec::new();

        for info in infos.into_iter() {
            if info.state != LifecycleState::Ready {
                continue;
            }

            let component = info.component.read().unwrap();
            let name = component.name().to_string();

            let result = component.health_check().await;
            results.push((name, result));
        }

        results
    }

    /// Get component state
    pub fn get_component_state(&self, name: &str) -> Option<LifecycleState> {
        let components = self.components.read().unwrap();
        components.iter()
            .find(|info| info.component.read().unwrap().name() == name)
            .map(|info| info.state)
    }

    /// Get all component states
    pub fn get_all_component_states(&self) -> Vec<(String, LifecycleState)> {
        let components = self.components.read().unwrap();
        components.iter()
            .map(|info| {
                let name = info.component.read().unwrap().name().to_string();
                (name, info.state)
            })
            .collect()
    }

    /// Update component state
    fn update_component_state(&self, name: &str, state: LifecycleState) {
        let mut components = self.components.write().unwrap();
        if let Some(info) = components.iter_mut()
            .find(|info| info.component.read().unwrap().name() == name) {
            info.state = state;
        }
    }

    /// Sort components by dependencies (simple topological sort)
    fn sort_by_dependencies(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // For now, just ensure components with dependencies come after their dependencies
        // A full topological sort would be more complex
        let mut components = self.components.write().unwrap();

        // Simple bubble sort by dependency count (components with fewer dependencies first)
        for i in 0..components.len() {
            for j in 0..components.len() - i - 1 {
                if components[j].dependencies.len() > components[j + 1].dependencies.len() {
                    components.swap(j, j + 1);
                }
            }
        }

        Ok(())
    }
}

impl Default for LifecycleManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Global lifecycle manager
static LIFECYCLE_MANAGER: OnceLock<Arc<LifecycleManager>> = OnceLock::new();

/// Initialize the global lifecycle manager
pub fn initialize_lifecycle_manager() -> Arc<LifecycleManager> {
    let manager = Arc::new(LifecycleManager::new());
    let _ = LIFECYCLE_MANAGER.set(manager.clone());
    manager
}

/// Get the global lifecycle manager
pub fn get_lifecycle_manager() -> Option<Arc<LifecycleManager>> {
    LIFECYCLE_MANAGER.get().cloned()
}

/// Register a component with the global lifecycle manager
pub fn register_component<C>(
    component: C,
    dependencies: Vec<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    C: LifecycleComponent + 'static,
{
    if let Some(manager) = get_lifecycle_manager() {
        manager.register_component(component, dependencies)
    } else {
        Err("Lifecycle manager not initialized".into())
    }
}
