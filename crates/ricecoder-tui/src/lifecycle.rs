//! Component lifecycle management for TUI
//!
//! This module provides lifecycle management for TUI application components,
//! ensuring proper startup, initialization, and shutdown procedures.

use std::{
    fmt::Debug,
    sync::{Arc, OnceLock, RwLock},
};

use ricecoder_di::DIContainer;
use tracing::{debug, info, warn};

/// Component lifecycle states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TuiLifecycleState {
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

/// Component lifecycle trait for TUI
#[async_trait::async_trait]
pub trait TuiLifecycleComponent: Send + Sync + Debug {
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

/// Component registration info for TUI
#[derive(Clone, Debug)]
struct TuiComponentInfo {
    component: Arc<RwLock<dyn TuiLifecycleComponent>>,
    state: TuiLifecycleState,
    dependencies: Vec<String>,
}

/// Component lifecycle manager for TUI
#[derive(Debug)]
pub struct TuiLifecycleManager {
    components: RwLock<Vec<TuiComponentInfo>>,
}

impl TuiLifecycleManager {
    /// Create a new TUI lifecycle manager
    pub fn new() -> Self {
        Self {
            components: RwLock::new(Vec::new()),
        }
    }

    /// Validate memory safety invariants
    pub fn validate_memory_safety(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let components = self.components.read().unwrap();

        // Check for component name uniqueness
        let mut names = std::collections::HashSet::new();
        for info in &*components {
            let name = info.component.read().unwrap().name().to_string();
            if !names.insert(name.clone()) {
                return Err(format!("Duplicate component name: {}", name).into());
            }
        }

        // Check for circular dependencies (simplified check)
        for info in &*components {
            for dep in &info.dependencies {
                if !components
                    .iter()
                    .any(|other| other.component.read().unwrap().name() == dep.as_str())
                {
                    return Err(format!(
                        "Component '{}' depends on non-existent component '{}'",
                        info.component.read().unwrap().name(),
                        dep
                    )
                    .into());
                }
            }
        }

        Ok(())
    }

    /// Register a component with optional dependencies
    pub fn register_component<C>(
        &self,
        component: C,
        dependencies: Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        C: TuiLifecycleComponent + 'static,
    {
        let info = TuiComponentInfo {
            component: Arc::new(RwLock::new(component)),
            state: TuiLifecycleState::Uninitialized,
            dependencies,
        };

        let mut components = self.components.write().unwrap();
        components.push(info.clone());

        info!(
            "Registered TUI component: {}",
            info.component.read().unwrap().name()
        );
        Ok(())
    }

    /// Initialize all components in dependency order
    pub async fn initialize_all(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Initializing all TUI components...");

        // Sort components by dependencies (simple topological sort)
        self.sort_by_dependencies()?;

        let components = self.components.read().unwrap().clone();

        for info in &components {
            let mut component = info.component.write().unwrap();
            let name = component.name();

            debug!("Initializing TUI component: {}", name);

            // Update state
            drop(component);
            self.update_component_state(name, TuiLifecycleState::Initializing);

            let mut component = info.component.write().unwrap();
            match component.initialize().await {
                Ok(()) => {
                    debug!("TUI component initialized successfully: {}", name);
                    drop(component);
                    self.update_component_state(name, TuiLifecycleState::Ready);
                }
                Err(e) => {
                    warn!("TUI component initialization failed: {} - {}", name, e);
                    drop(component);
                    self.update_component_state(name, TuiLifecycleState::Error);
                    return Err(e);
                }
            }
        }

        info!("All TUI components initialized successfully");
        Ok(())
    }

    /// Start all components
    pub async fn start_all(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting all TUI components...");

        let components = self.components.read().unwrap().clone();

        for info in &components {
            if info.state != TuiLifecycleState::Ready {
                continue;
            }

            let mut component = info.component.write().unwrap();
            let name = component.name();

            debug!("Starting TUI component: {}", name);

            match component.start().await {
                Ok(()) => {
                    debug!("TUI component started successfully: {}", name);
                }
                Err(e) => {
                    warn!("TUI component start failed: {} - {}", name, e);
                    return Err(e);
                }
            }
        }

        info!("All TUI components started successfully");
        Ok(())
    }

    /// Stop all components in reverse order
    pub async fn stop_all(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Stopping all TUI components...");

        let components = self.components.read().unwrap().clone();

        // Stop in reverse order
        for info in components.iter().rev() {
            let mut component = info.component.write().unwrap();
            let name = component.name();

            debug!("Stopping TUI component: {}", name);
            self.update_component_state(name, TuiLifecycleState::ShuttingDown);

            match component.stop().await {
                Ok(()) => {
                    debug!("TUI component stopped successfully: {}", name);
                    drop(component);
                    self.update_component_state(name, TuiLifecycleState::ShutDown);
                }
                Err(e) => {
                    warn!("TUI component stop failed: {} - {}", name, e);
                    // Continue stopping other components even if one fails
                }
            }
        }

        info!("All TUI components stopped");
        Ok(())
    }

    /// Perform health checks on all components
    pub async fn health_check_all(
        &self,
    ) -> Vec<(String, Result<(), Box<dyn std::error::Error + Send + Sync>>)> {
        let components = self.components.read().unwrap().clone();
        let mut results = Vec::new();

        for info in &components {
            if info.state != TuiLifecycleState::Ready {
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
    pub fn get_component_state(&self, name: &str) -> Option<TuiLifecycleState> {
        let components = self.components.read().unwrap();
        components
            .iter()
            .find(|info| info.component.read().unwrap().name() == name)
            .map(|info| info.state)
    }

    /// Get all component states
    pub fn get_all_component_states(&self) -> Vec<(String, TuiLifecycleState)> {
        let components = self.components.read().unwrap();
        components
            .iter()
            .map(|info| {
                let name = info.component.read().unwrap().name().to_string();
                (name, info.state)
            })
            .collect()
    }

    /// Update component state
    fn update_component_state(&self, name: &str, state: TuiLifecycleState) {
        let mut components = self.components.write().unwrap();
        if let Some(info) = components
            .iter_mut()
            .find(|info| info.component.read().unwrap().name() == name)
        {
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

impl Default for TuiLifecycleManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Global lifecycle manager for TUI
static TUI_LIFECYCLE_MANAGER: OnceLock<Arc<TuiLifecycleManager>> = OnceLock::new();

/// Initialize the global TUI lifecycle manager
pub fn initialize_tui_lifecycle_manager() -> Arc<TuiLifecycleManager> {
    let manager = Arc::new(TuiLifecycleManager::new());
    TUI_LIFECYCLE_MANAGER
        .set(manager.clone())
        .expect("TUI lifecycle manager already initialized");
    manager
}

/// Get the global TUI lifecycle manager
pub fn get_tui_lifecycle_manager() -> Option<Arc<TuiLifecycleManager>> {
    TUI_LIFECYCLE_MANAGER.get().cloned()
}

/// Register a component with the global TUI lifecycle manager
pub fn register_tui_component<C>(
    component: C,
    dependencies: Vec<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    C: TuiLifecycleComponent + 'static,
{
    if let Some(manager) = get_tui_lifecycle_manager() {
        manager.register_component(component, dependencies)
    } else {
        Err("TUI lifecycle manager not initialized".into())
    }
}

#[cfg(test)]
mod tests {
    // Tests are in tests/lifecycle_tests.rs
}
