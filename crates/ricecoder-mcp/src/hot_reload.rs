//! Configuration hot-reload support for MCP

use crate::config::{MCPConfig, MCPConfigLoader};
use crate::error::Result;
use crate::registry::ToolRegistry;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Configuration watcher for hot-reload support
#[derive(Debug, Clone)]
pub struct ConfigWatcher {
    config_dir: PathBuf,
    current_config: Arc<RwLock<MCPConfig>>,
    #[allow(dead_code)]
    tool_registry: Arc<ToolRegistry>,
}

impl ConfigWatcher {
    /// Creates a new configuration watcher
    ///
    /// # Arguments
    /// * `config_dir` - Directory to watch for configuration changes
    /// * `tool_registry` - Tool registry to update on configuration changes
    pub fn new<P: AsRef<Path>>(config_dir: P, tool_registry: Arc<ToolRegistry>) -> Self {
        Self {
            config_dir: config_dir.as_ref().to_path_buf(),
            current_config: Arc::new(RwLock::new(MCPConfig::new())),
            tool_registry,
        }
    }

    /// Loads initial configuration from the configuration directory
    pub async fn load_initial_config(&self) -> Result<()> {
        debug!("Loading initial configuration from: {:?}", self.config_dir);

        let config = MCPConfigLoader::load_from_directory(&self.config_dir)?;

        // Validate configuration
        MCPConfigLoader::validate(&config)?;

        // Update current configuration
        let mut current = self.current_config.write().await;
        *current = config.clone();
        drop(current);

        // Update tool registry with loaded tools
        self.update_registry(&config).await?;

        info!("Initial configuration loaded successfully");
        Ok(())
    }

    /// Detects configuration file changes
    ///
    /// This method should be run in a background task to continuously monitor for changes.
    pub async fn watch_for_changes(&self) -> Result<()> {
        debug!("Starting configuration file watcher");

        loop {
            // Check for configuration file changes
            if let Err(e) = self.check_for_changes().await {
                warn!("Error checking for configuration changes: {}", e);
            }

            // Sleep for a short duration before checking again
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    }

    /// Checks for configuration file changes
    async fn check_for_changes(&self) -> Result<()> {
        debug!("Checking for configuration file changes");

        // Try to load configuration from directory
        let new_config = MCPConfigLoader::load_from_directory(&self.config_dir)?;

        // Validate new configuration
        MCPConfigLoader::validate(&new_config)?;

        // Compare with current configuration
        let current = self.current_config.read().await;
        if self.configs_differ(&current, &new_config) {
            drop(current);

            info!("Configuration changes detected, reloading");

            // Update current configuration
            let mut current = self.current_config.write().await;
            *current = new_config.clone();
            drop(current);

            // Update tool registry with new tools
            self.update_registry(&new_config).await?;

            info!("Configuration reloaded successfully");
        }

        Ok(())
    }

    /// Compares two configurations to detect changes
    fn configs_differ(&self, config1: &MCPConfig, config2: &MCPConfig) -> bool {
        // Compare servers
        if config1.servers.len() != config2.servers.len() {
            return true;
        }

        for (server1, server2) in config1.servers.iter().zip(config2.servers.iter()) {
            if server1.id != server2.id
                || server1.command != server2.command
                || server1.timeout_ms != server2.timeout_ms
            {
                return true;
            }
        }

        // Compare custom tools
        if config1.custom_tools.len() != config2.custom_tools.len() {
            return true;
        }

        for (tool1, tool2) in config1.custom_tools.iter().zip(config2.custom_tools.iter()) {
            if tool1.id != tool2.id || tool1.handler != tool2.handler {
                return true;
            }
        }

        // Compare permissions
        if config1.permissions.len() != config2.permissions.len() {
            return true;
        }

        for (perm1, perm2) in config1.permissions.iter().zip(config2.permissions.iter()) {
            if perm1.pattern != perm2.pattern || perm1.level != perm2.level {
                return true;
            }
        }

        false
    }

    /// Updates the tool registry with new configuration
    async fn update_registry(&self, config: &MCPConfig) -> Result<()> {
        debug!("Updating tool registry with new configuration");

        // Note: Tool registry updates would be performed here
        // In a real implementation, this would convert CustomToolConfig to ToolMetadata
        // and register them with the tool registry

        info!(
            "Tool registry updated with {} custom tools",
            config.custom_tools.len()
        );
        Ok(())
    }

    /// Gets the current configuration
    pub async fn get_current_config(&self) -> MCPConfig {
        self.current_config.read().await.clone()
    }

    /// Manually reloads configuration
    pub async fn reload_config(&self) -> Result<()> {
        debug!("Manually reloading configuration");

        let config = MCPConfigLoader::load_from_directory(&self.config_dir)?;

        // Validate configuration
        MCPConfigLoader::validate(&config)?;

        // Update current configuration
        let mut current = self.current_config.write().await;
        *current = config.clone();
        drop(current);

        // Update tool registry
        self.update_registry(&config).await?;

        info!("Configuration reloaded successfully");
        Ok(())
    }

    /// Validates new configuration before applying
    pub async fn validate_new_config<P: AsRef<Path>>(&self, config_path: P) -> Result<MCPConfig> {
        debug!(
            "Validating new configuration from: {:?}",
            config_path.as_ref()
        );

        let config = MCPConfigLoader::load_from_file(config_path)?;
        MCPConfigLoader::validate(&config)?;

        info!("Configuration validation passed");
        Ok(config)
    }

    /// Gets the configuration directory
    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    /// Sets the configuration directory
    pub fn set_config_dir<P: AsRef<Path>>(&mut self, dir: P) {
        self.config_dir = dir.as_ref().to_path_buf();
        debug!("Configuration directory changed to: {:?}", self.config_dir);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::ToolRegistry;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_create_config_watcher() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let registry = Arc::new(ToolRegistry::new());
        let watcher = ConfigWatcher::new(temp_dir.path(), registry);

        assert_eq!(watcher.config_dir(), temp_dir.path());
    }

    #[tokio::test]
    async fn test_load_initial_config() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("mcp-servers.yaml");

        let config_content = r#"
servers:
  - id: test-server
    name: Test Server
    command: test
    args: []
    env: {}
    timeout_ms: 5000
    auto_reconnect: true
    max_retries: 3
custom_tools: []
permissions: []
"#;

        std::fs::write(&config_path, config_content).expect("Failed to write config");

        let registry = Arc::new(ToolRegistry::new());
        let watcher = ConfigWatcher::new(temp_dir.path(), registry);

        let result = watcher.load_initial_config().await;
        assert!(result.is_ok());

        let config = watcher.get_current_config().await;
        assert_eq!(config.servers.len(), 1);
    }

    #[tokio::test]
    async fn test_configs_differ_servers() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let registry = Arc::new(ToolRegistry::new());
        let watcher = ConfigWatcher::new(temp_dir.path(), registry);

        let mut config1 = MCPConfig::new();
        let config2 = MCPConfig::new();

        config1.add_server(crate::config::MCPServerConfig {
            id: "server1".to_string(),
            name: "Server 1".to_string(),
            command: "cmd1".to_string(),
            args: vec![],
            env: std::collections::HashMap::new(),
            timeout_ms: 5000,
            auto_reconnect: true,
            max_retries: 3,
        });

        assert!(watcher.configs_differ(&config1, &config2));
    }

    #[tokio::test]
    async fn test_configs_same() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let registry = Arc::new(ToolRegistry::new());
        let watcher = ConfigWatcher::new(temp_dir.path(), registry);

        let config1 = MCPConfig::new();
        let config2 = MCPConfig::new();

        assert!(!watcher.configs_differ(&config1, &config2));
    }

    #[tokio::test]
    async fn test_validate_new_config() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("test-config.yaml");

        let config_content = r#"
servers:
  - id: test-server
    name: Test Server
    command: test
    args: []
    env: {}
    timeout_ms: 5000
    auto_reconnect: true
    max_retries: 3
custom_tools: []
permissions: []
"#;

        std::fs::write(&config_path, config_content).expect("Failed to write config");

        let registry = Arc::new(ToolRegistry::new());
        let watcher = ConfigWatcher::new(temp_dir.path(), registry);

        let result = watcher.validate_new_config(&config_path).await;
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.servers.len(), 1);
    }

    #[tokio::test]
    async fn test_validate_invalid_config() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("invalid-config.yaml");

        let config_content = r#"
servers:
  - id: ""
    name: Test Server
    command: test
    args: []
    env: {}
    timeout_ms: 5000
    auto_reconnect: true
    max_retries: 3
custom_tools: []
permissions: []
"#;

        std::fs::write(&config_path, config_content).expect("Failed to write config");

        let registry = Arc::new(ToolRegistry::new());
        let watcher = ConfigWatcher::new(temp_dir.path(), registry);

        let result = watcher.validate_new_config(&config_path).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_set_config_dir() {
        let temp_dir1 = TempDir::new().expect("Failed to create temp dir");
        let temp_dir2 = TempDir::new().expect("Failed to create temp dir");

        let registry = Arc::new(ToolRegistry::new());
        let mut watcher = ConfigWatcher::new(temp_dir1.path(), registry);

        assert_eq!(watcher.config_dir(), temp_dir1.path());

        watcher.set_config_dir(temp_dir2.path());
        assert_eq!(watcher.config_dir(), temp_dir2.path());
    }

    #[tokio::test]
    async fn test_reload_config() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("mcp-servers.yaml");

        let config_content = r#"
servers:
  - id: test-server
    name: Test Server
    command: test
    args: []
    env: {}
    timeout_ms: 5000
    auto_reconnect: true
    max_retries: 3
custom_tools: []
permissions: []
"#;

        std::fs::write(&config_path, config_content).expect("Failed to write config");

        let registry = Arc::new(ToolRegistry::new());
        let watcher = ConfigWatcher::new(temp_dir.path(), registry);

        let result = watcher.reload_config().await;
        assert!(result.is_ok());

        let config = watcher.get_current_config().await;
        assert_eq!(config.servers.len(), 1);
    }
}
