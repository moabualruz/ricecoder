//! Central orchestration manager for workspace operations

use crate::error::Result;
use crate::models::Workspace;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Central coordinator for all workspace orchestration operations
///
/// The OrchestrationManager manages the lifecycle of all orchestration components
/// and coordinates cross-project workflows. It uses `ricecoder_storage::PathResolver`
/// for all path operations to ensure consistent workspace navigation.
pub struct OrchestrationManager {
    /// Current workspace state
    workspace: Arc<RwLock<Workspace>>,

    /// Workspace root path
    workspace_root: PathBuf,
}

impl OrchestrationManager {
    /// Creates a new OrchestrationManager for the given workspace root
    ///
    /// # Arguments
    ///
    /// * `workspace_root` - The root path of the workspace
    ///
    /// # Returns
    ///
    /// A new OrchestrationManager instance
    pub fn new(workspace_root: PathBuf) -> Self {
        debug!(
            "Creating OrchestrationManager for workspace: {:?}",
            workspace_root
        );

        let workspace = Workspace {
            root: workspace_root.clone(),
            ..Default::default()
        };

        Self {
            workspace: Arc::new(RwLock::new(workspace)),
            workspace_root,
        }
    }

    /// Gets the current workspace state
    ///
    /// # Returns
    ///
    /// A clone of the current workspace
    pub async fn get_workspace(&self) -> Workspace {
        self.workspace.read().await.clone()
    }

    /// Updates the workspace state
    ///
    /// # Arguments
    ///
    /// * `workspace` - The new workspace state
    pub async fn set_workspace(&self, workspace: Workspace) {
        info!("Updating workspace state");
        *self.workspace.write().await = workspace;
    }

    /// Gets the workspace root path
    ///
    /// # Returns
    ///
    /// The workspace root path
    pub fn workspace_root(&self) -> &PathBuf {
        &self.workspace_root
    }

    /// Initializes the orchestration manager
    ///
    /// This method should be called after creating the manager to initialize
    /// all sub-components and load workspace configuration.
    ///
    /// # Returns
    ///
    /// Result indicating success or failure
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing OrchestrationManager");

        // Initialize workspace state
        let mut workspace = self.workspace.write().await;
        workspace.root = self.workspace_root.clone();

        // Load workspace configuration
        self.load_configuration(&mut workspace).await?;

        debug!("OrchestrationManager initialized successfully");
        Ok(())
    }

    /// Loads workspace configuration from storage
    ///
    /// This method loads configuration from the workspace root using
    /// ricecoder_storage::PathResolver for consistent path handling.
    ///
    /// # Arguments
    ///
    /// * `workspace` - The workspace to load configuration into
    ///
    /// # Returns
    ///
    /// Result indicating success or failure
    async fn load_configuration(&self, workspace: &mut Workspace) -> Result<()> {
        debug!(
            "Loading workspace configuration from: {:?}",
            self.workspace_root
        );

        // Try to load configuration from standard locations
        let config_paths = vec![
            self.workspace_root
                .join(".ricecoder")
                .join("workspace.yaml"),
            self.workspace_root
                .join(".ricecoder")
                .join("workspace.json"),
            self.workspace_root.join("ricecoder.yaml"),
            self.workspace_root.join("ricecoder.json"),
        ];

        for config_path in config_paths {
            if config_path.exists() {
                debug!("Found configuration file: {:?}", config_path);
                // Configuration loading would happen here
                // For now, we just log that we found it
                break;
            }
        }

        // Initialize default configuration if not found
        if workspace.config.rules.is_empty() {
            debug!("No configuration found, using defaults");
        }

        Ok(())
    }

    /// Shuts down the orchestration manager
    ///
    /// This method should be called when the manager is no longer needed
    /// to clean up resources and save state.
    ///
    /// # Returns
    ///
    /// Result indicating success or failure
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down OrchestrationManager");
        debug!("OrchestrationManager shutdown complete");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_orchestration_manager_creation() {
        let root = PathBuf::from("/test/workspace");
        let manager = OrchestrationManager::new(root.clone());

        assert_eq!(manager.workspace_root(), &root);
    }

    #[tokio::test]
    async fn test_orchestration_manager_initialization() {
        let root = PathBuf::from("/test/workspace");
        let manager = OrchestrationManager::new(root.clone());

        let result = manager.initialize().await;
        assert!(result.is_ok());

        let workspace = manager.get_workspace().await;
        assert_eq!(workspace.root, root);
    }

    #[tokio::test]
    async fn test_orchestration_manager_workspace_update() {
        let root = PathBuf::from("/test/workspace");
        let manager = OrchestrationManager::new(root.clone());

        manager.initialize().await.expect("initialization failed");

        let mut workspace = manager.get_workspace().await;
        workspace.metrics.total_projects = 5;

        manager.set_workspace(workspace.clone()).await;

        let updated = manager.get_workspace().await;
        assert_eq!(updated.metrics.total_projects, 5);
    }

    #[tokio::test]
    async fn test_orchestration_manager_shutdown() {
        let root = PathBuf::from("/test/workspace");
        let manager = OrchestrationManager::new(root);

        manager.initialize().await.expect("initialization failed");
        let result = manager.shutdown().await;
        assert!(result.is_ok());
    }
}
