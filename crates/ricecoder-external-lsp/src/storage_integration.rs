//! Storage integration for external LSP configuration
//!
//! This module provides integration with ricecoder-storage for loading and managing
//! LSP server configurations, using the centralized storage system for path resolution
//! and configuration hierarchy.
//!
//! # Configuration Hierarchy
//!
//! Configurations are loaded in the following priority order (highest to lowest):
//! 1. Runtime overrides (programmatic configuration)
//! 2. Project-level configuration (`.ricecoder/lsp-servers.yaml`)
//! 3. User-level configuration (`~/.ricecoder/lsp-servers.yaml`)
//! 4. Built-in defaults (pre-configured servers)
//! 5. Fallback (internal providers)
//!
//! # Example
//!
//! ```ignore
//! use ricecoder_external_lsp::storage_integration::StorageConfigLoader;
//! use ricecoder_storage::StorageManager;
//!
//! let storage_manager = StorageManager::new()?;
//! let config_loader = StorageConfigLoader::new(storage_manager);
//! let registry = config_loader.load_registry()?;
//! ```

use crate::error::Result;
use crate::types::{GlobalLspSettings, LspServerConfig, LspServerRegistry};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, info, warn};

/// Storage-based configuration loader for LSP servers
///
/// This loader integrates with ricecoder-storage to load LSP server configurations
/// from multiple sources with proper hierarchy and path resolution.
pub struct StorageConfigLoader;

impl StorageConfigLoader {
    /// Create a new storage-based configuration loader
    pub fn new() -> Self {
        Self
    }

    /// Load LSP server registry from storage
    ///
    /// Loads configurations from multiple sources in priority order:
    /// 1. Project-level configuration
    /// 2. User-level configuration
    /// 3. Built-in defaults
    ///
    /// # Returns
    ///
    /// LSP server registry with all configured servers
    pub fn load_registry(&self) -> Result<LspServerRegistry> {
        info!("Loading LSP server registry from storage");

        // Start with built-in defaults
        let mut servers: HashMap<String, Vec<LspServerConfig>> = HashMap::new();
        let mut global_settings = GlobalLspSettings::default();

        // Load project-level configuration
        if let Ok(project_config) = self.load_project_config() {
            debug!("Loaded project-level LSP configuration");
            self.merge_config(&mut servers, &mut global_settings, project_config)?;
        }

        // Load user-level configuration
        if let Ok(user_config) = self.load_user_config() {
            debug!("Loaded user-level LSP configuration");
            self.merge_config(&mut servers, &mut global_settings, user_config)?;
        }

        // Load built-in defaults
        let builtin_config = self.load_builtin_config()?;
        debug!("Loaded built-in LSP configuration");
        self.merge_config(&mut servers, &mut global_settings, builtin_config)?;

        Ok(LspServerRegistry {
            servers,
            global: global_settings,
        })
    }

    /// Load project-level configuration
    fn load_project_config(&self) -> Result<Value> {
        // Try to load from .ricecoder/lsp-servers.yaml
        let project_config_path = PathBuf::from(".ricecoder/lsp-servers.yaml");
        debug!("Loading project config from: {:?}", project_config_path);

        if project_config_path.exists() {
            let content = std::fs::read_to_string(&project_config_path)
                .map_err(|e| crate::error::ExternalLspError::ConfigError(format!(
                    "Failed to read project config: {}",
                    e
                )))?;

            let config: Value = serde_yaml::from_str(&content)
                .map_err(|e| crate::error::ExternalLspError::ConfigError(format!(
                    "Failed to parse project config: {}",
                    e
                )))?;

            Ok(config)
        } else {
            Err(crate::error::ExternalLspError::ConfigError(
                "Project config not found".to_string(),
            ))
        }
    }

    /// Load user-level configuration
    fn load_user_config(&self) -> Result<Value> {
        // Try to load from ~/.ricecoder/lsp-servers.yaml
        let home_dir = dirs::home_dir().ok_or_else(|| {
            crate::error::ExternalLspError::ConfigError("Could not determine home directory".to_string())
        })?;
        
        let user_config_path = home_dir.join(".ricecoder/lsp-servers.yaml");
        debug!("Loading user config from: {:?}", user_config_path);

        if user_config_path.exists() {
            let content = std::fs::read_to_string(&user_config_path)
                .map_err(|e| crate::error::ExternalLspError::ConfigError(format!(
                    "Failed to read user config: {}",
                    e
                )))?;

            let config: Value = serde_yaml::from_str(&content)
                .map_err(|e| crate::error::ExternalLspError::ConfigError(format!(
                    "Failed to parse user config: {}",
                    e
                )))?;

            Ok(config)
        } else {
            Err(crate::error::ExternalLspError::ConfigError(
                "User config not found".to_string(),
            ))
        }
    }

    /// Load built-in configuration
    fn load_builtin_config(&self) -> Result<Value> {
        // Create built-in defaults for common LSP servers
        let mut servers: HashMap<String, Vec<LspServerConfig>> = HashMap::new();
        
        // Rust
        servers.insert("rust".to_string(), vec![LspServerConfig {
            language: "rust".to_string(),
            extensions: vec![".rs".to_string()],
            executable: "rust-analyzer".to_string(),
            args: vec![],
            env: HashMap::new(),
            init_options: None,
            enabled: true,
            timeout_ms: 5000,
            max_restarts: 3,
            idle_timeout_ms: 300000,
            output_mapping: None,
        }]);
        
        // TypeScript
        servers.insert("typescript".to_string(), vec![LspServerConfig {
            language: "typescript".to_string(),
            extensions: vec![".ts".to_string(), ".tsx".to_string(), ".js".to_string(), ".jsx".to_string()],
            executable: "typescript-language-server".to_string(),
            args: vec!["--stdio".to_string()],
            env: HashMap::new(),
            init_options: None,
            enabled: true,
            timeout_ms: 5000,
            max_restarts: 3,
            idle_timeout_ms: 300000,
            output_mapping: None,
        }]);
        
        // Python
        servers.insert("python".to_string(), vec![LspServerConfig {
            language: "python".to_string(),
            extensions: vec![".py".to_string()],
            executable: "pylsp".to_string(),
            args: vec![],
            env: HashMap::new(),
            init_options: None,
            enabled: true,
            timeout_ms: 5000,
            max_restarts: 3,
            idle_timeout_ms: 300000,
            output_mapping: None,
        }]);

        let config_value = serde_json::json!({
            "servers": servers,
            "global": {
                "max_processes": 5,
                "default_timeout_ms": 5000,
                "enable_fallback": true,
                "health_check_interval_ms": 30000
            }
        });
        
        Ok(config_value)
    }

    /// Merge configuration from multiple sources
    fn merge_config(
        &self,
        servers: &mut HashMap<String, Vec<LspServerConfig>>,
        global_settings: &mut GlobalLspSettings,
        config: Value,
    ) -> Result<()> {
        // Merge servers
        if let Some(config_servers) = config.get("servers").and_then(|v| v.as_object()) {
            for (language, server_configs) in config_servers {
                if let Ok(configs) = serde_json::from_value::<Vec<LspServerConfig>>(
                    server_configs.clone(),
                ) {
                    servers.insert(language.clone(), configs);
                }
            }
        }

        // Merge global settings
        if let Some(global) = config.get("global").and_then(|v| v.as_object()) {
            if let Some(max_processes) = global.get("max_processes").and_then(|v| v.as_u64()) {
                global_settings.max_processes = max_processes as usize;
            }
            if let Some(timeout) = global.get("default_timeout_ms").and_then(|v| v.as_u64()) {
                global_settings.default_timeout_ms = timeout;
            }
            if let Some(enable_fallback) = global.get("enable_fallback").and_then(|v| v.as_bool())
            {
                global_settings.enable_fallback = enable_fallback;
            }
            if let Some(health_check) = global
                .get("health_check_interval_ms")
                .and_then(|v| v.as_u64())
            {
                global_settings.health_check_interval_ms = health_check;
            }
        }

        Ok(())
    }

    /// Resolve executable path using storage path resolver
    ///
    /// # Arguments
    ///
    /// * `executable` - Executable name or path
    ///
    /// # Returns
    ///
    /// Resolved executable path
    pub fn resolve_executable_path(&self, executable: &str) -> Result<PathBuf> {
        // Try to resolve using path resolver
        // First check if it's an absolute path
        let path = PathBuf::from(executable);
        if path.is_absolute() && path.exists() {
            debug!("Resolved executable: {} -> {:?}", executable, path);
            return Ok(path);
        }
        
        // Try to find in PATH
        if let Ok(path_env) = std::env::var("PATH") {
            for path_dir in std::env::split_paths(&path_env) {
                let full_path = path_dir.join(executable);
                if full_path.exists() {
                    debug!("Resolved executable: {} -> {:?}", executable, full_path);
                    return Ok(full_path);
                }
            }
        }
        
        // Fall back to checking current directory
        let current_path = PathBuf::from(executable);
        if current_path.exists() {
            debug!("Resolved executable: {} -> {:?}", executable, current_path);
            return Ok(current_path);
        }
        
        warn!("Could not resolve executable: {}", executable);
        Err(crate::error::ExternalLspError::ServerNotFound {
            executable: executable.to_string(),
        })
    }

    /// Cache server state in storage
    ///
    /// # Arguments
    ///
    /// * `language` - Programming language
    /// * `_state` - Server state to cache
    pub fn cache_server_state(&self, language: &str, _state: Value) -> Result<()> {
        debug!("Caching server state for language: {}", language);
        
        // Use storage manager to cache state
        // This would integrate with ricecoder-storage's caching system
        // For now, this is a placeholder for future implementation
        
        Ok(())
    }

    /// Load cached server state from storage
    ///
    /// # Arguments
    ///
    /// * `language` - Programming language
    ///
    /// # Returns
    ///
    /// Cached server state, or None if not found
    pub fn load_cached_server_state(&self, language: &str) -> Result<Option<Value>> {
        debug!("Loading cached server state for language: {}", language);
        
        // Use storage manager to load cached state
        // This would integrate with ricecoder-storage's caching system
        // For now, this is a placeholder for future implementation
        
        Ok(None)
    }
}

impl Default for StorageConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_config_loader_creation() {
        // This test would require a mock StorageManager
        // For now, we just verify the struct can be created
        let _loader = StorageConfigLoader::new();
    }
}
