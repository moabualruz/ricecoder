//! Plugin system for RiceCoder TUI
//!
//! This module implements a plugin architecture that allows third-party extensions
//! to customize and extend the TUI functionality.

use crate::error::TuiResult;
use crate::tea::{AppMessage, AppModel};
use ratatui::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use std::fs;
use walkdir::WalkDir;

/// Unique identifier for a plugin
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PluginId(pub String);

impl From<String> for PluginId {
    fn from(s: String) -> Self {
        PluginId(s)
    }
}

impl From<&str> for PluginId {
    fn from(s: &str) -> Self {
        PluginId(s.to_string())
    }
}

/// Plugin metadata
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    pub id: PluginId,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
}

/// Plugin context provided during initialization
#[derive(Debug)]
pub struct PluginContext<'a> {
    pub app_model: &'a AppModel,
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub temp_dir: PathBuf,
}

/// Messages that can be sent between plugins and the main application
#[derive(Debug, Clone)]
pub enum PluginMessage {
    /// Plugin wants to send a message to the main application
    ToApp(AppMessage),
    /// Plugin wants to send a message to another plugin
    ToPlugin {
        target_plugin: PluginId,
        message: Box<PluginMessage>,
    },
    /// Plugin state update
    StateUpdate(String),
    /// Plugin error
    Error(String),
}

/// Core plugin trait that all plugins must implement
#[async_trait::async_trait]
pub trait Plugin: Send + Sync {
    /// Get the plugin's unique identifier
    fn id(&self) -> PluginId;

    /// Get the plugin's human-readable name
    fn name(&self) -> &str;

    /// Get the plugin's version
    fn version(&self) -> &str;

    /// Get plugin metadata
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: self.id(),
            name: self.name().to_string(),
            version: self.version().to_string(),
            description: String::new(),
            author: String::new(),
            homepage: None,
            repository: None,
        }
    }

    /// Initialize the plugin with context
    async fn initialize(&mut self, context: &PluginContext) -> TuiResult<()>;

    /// Handle an incoming message
    async fn handle_message(&mut self, message: &PluginMessage) -> Vec<PluginMessage>;

    /// Render the plugin's UI (if any)
    async fn render(&self, area: Rect, model: &AppModel) -> TuiResult<Vec<Line>>;

    /// Called when the plugin should clean up
    async fn cleanup(&mut self) -> TuiResult<()>;

    /// Get the plugin's configuration schema (JSON Schema)
    fn config_schema(&self) -> Option<serde_json::Value> {
        None
    }

    /// Update plugin configuration
    async fn update_config(&mut self, config: serde_json::Value) -> TuiResult<()> {
        let _ = config;
        Ok(())
    }
}

/// Plugin registry for managing available plugins
#[derive(Debug)]
pub struct PluginRegistry {
    plugins: HashMap<PluginId, Box<dyn Plugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    /// Register a plugin
    pub fn register<P: Plugin + 'static>(&mut self, plugin: P) -> TuiResult<()> {
        let id = plugin.id();
        if self.plugins.contains_key(&id) {
            return Err(crate::error::TuiError::Plugin(format!(
                "Plugin with ID '{}' already registered",
                id.0
            )));
        }
        self.plugins.insert(id, Box::new(plugin));
        Ok(())
    }

    /// Unregister a plugin
    pub fn unregister(&mut self, id: &PluginId) -> TuiResult<()> {
        if self.plugins.remove(id).is_none() {
            return Err(crate::error::TuiError::Plugin(format!(
                "Plugin with ID '{}' not found",
                id.0
            )));
        }
        Ok(())
    }

    /// Get a plugin by ID
    pub fn get(&self, id: &PluginId) -> Option<&Box<dyn Plugin>> {
        self.plugins.get(id)
    }

    /// Get a mutable reference to a plugin by ID
    pub fn get_mut(&mut self, id: &PluginId) -> Option<&mut Box<dyn Plugin>> {
        self.plugins.get_mut(id)
    }

    /// List all registered plugins
    pub fn list(&self) -> Vec<&Box<dyn Plugin>> {
        self.plugins.values().collect()
    }

    /// Check if a plugin is registered
    pub fn contains(&self, id: &PluginId) -> bool {
        self.plugins.contains_key(id)
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Plugin sandbox for security isolation
#[derive(Debug)]
pub struct PluginSandbox {
    allowed_paths: Vec<PathBuf>,
    max_memory_mb: usize,
    timeout_ms: u64,
    allowed_commands: Vec<String>,
    max_file_size_mb: usize,
    rate_limit_per_second: usize,
}

impl PluginSandbox {
    pub fn new() -> Self {
        Self {
            allowed_paths: vec![],
            max_memory_mb: 50, // 50MB default limit
            timeout_ms: 5000,  // 5 second timeout
            allowed_commands: vec![],
            max_file_size_mb: 10, // 10MB max file size
            rate_limit_per_second: 100, // 100 operations per second
        }
    }

    pub fn with_allowed_path(mut self, path: PathBuf) -> Self {
        self.allowed_paths.push(path);
        self
    }

    pub fn with_memory_limit(mut self, mb: usize) -> Self {
        self.max_memory_mb = mb;
        self
    }

    pub fn with_timeout(mut self, ms: u64) -> Self {
        self.timeout_ms = ms;
        self
    }

    pub fn with_allowed_command(mut self, command: String) -> Self {
        self.allowed_commands.push(command);
        self
    }

    pub fn with_max_file_size(mut self, mb: usize) -> Self {
        self.max_file_size_mb = mb;
        self
    }

    pub fn with_rate_limit(mut self, ops_per_second: usize) -> Self {
        self.rate_limit_per_second = ops_per_second;
        self
    }

    /// Validate that a plugin operation is allowed
    pub fn validate_operation(&self, operation: &PluginOperation) -> TuiResult<()> {
        match operation {
            PluginOperation::FileAccess(path) => {
                self.validate_file_access(path)?;
            }
            PluginOperation::NetworkAccess => {
                return Err(crate::error::TuiError::Plugin(
                    "Network access not allowed for plugins".to_string()
                ));
            }
            PluginOperation::SystemCommand => {
                return Err(crate::error::TuiError::Plugin(
                    "System commands not allowed for plugins".to_string()
                ));
            }
        }
        Ok(())
    }

    /// Validate file access permissions
    fn validate_file_access(&self, path: &PathBuf) -> TuiResult<()> {
        // Check if path is within allowed directories
        let is_allowed = self.allowed_paths.iter().any(|allowed| {
            path.starts_with(allowed) || path.canonicalize()
                .map(|canonical| canonical.starts_with(allowed))
                .unwrap_or(false)
        });

        if !is_allowed {
            return Err(crate::error::TuiError::Plugin(format!(
                "Plugin attempted to access unauthorized path: {}",
                path.display()
            )));
        }

        // Check file size if it exists
        if let Ok(metadata) = std::fs::metadata(path) {
            let size_mb = metadata.len() / (1024 * 1024);
            if size_mb > self.max_file_size_mb as u64 {
                return Err(crate::error::TuiError::Plugin(format!(
                    "Plugin attempted to access file larger than {}MB limit: {}",
                    self.max_file_size_mb, path.display()
                )));
            }
        }

        Ok(())
    }

    /// Validate plugin manifest for security
    pub fn validate_manifest(&self, manifest: &PluginManifest) -> TuiResult<()> {
        // Check for dangerous permissions
        if let Some(permissions) = &manifest.permissions {
            for permission in permissions {
                match permission.as_str() {
                    "network" | "system" | "admin" => {
                        return Err(crate::error::TuiError::Plugin(format!(
                            "Plugin requests dangerous permission: {}", permission
                        )));
                    }
                    "filesystem" | "config" => {
                        // These are allowed but logged
                        tracing::info!("Plugin {} requests permission: {}", manifest.name, permission);
                    }
                    _ => {
                        tracing::warn!("Plugin {} requests unknown permission: {}", manifest.name, permission);
                    }
                }
            }
        }

        // Validate entry point exists and is safe
        let entry_path = PathBuf::from(&manifest.entry_point);
        if entry_path.is_absolute() || entry_path.starts_with("..") {
            return Err(crate::error::TuiError::Plugin(format!(
                "Plugin entry point contains unsafe path: {}", manifest.entry_point
            )));
        }

        Ok(())
    }

    /// Create a rate limiter for plugin operations
    pub fn create_rate_limiter(&self) -> RateLimiter {
        RateLimiter::new(self.rate_limit_per_second)
    }
}

/// Rate limiter for plugin operations
#[derive(Debug)]
pub struct RateLimiter {
    operations: Vec<std::time::Instant>,
    max_per_second: usize,
}

impl RateLimiter {
    pub fn new(max_per_second: usize) -> Self {
        Self {
            operations: Vec::new(),
            max_per_second,
        }
    }

    /// Check if operation is allowed under rate limit
    pub fn check(&mut self) -> bool {
        let now = std::time::Instant::now();

        // Remove operations older than 1 second
        self.operations.retain(|&time| now.duration_since(time).as_secs() < 1);

        if self.operations.len() >= self.max_per_second {
            false
        } else {
            self.operations.push(now);
            true
        }
    }
}

/// Operations that plugins might attempt
#[derive(Debug)]
pub enum PluginOperation {
    FileAccess(PathBuf),
    NetworkAccess,
    SystemCommand,
}

/// Plugin manifest file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub entry_point: String, // Path to the plugin library
    pub dependencies: Option<HashMap<String, String>>,
    pub permissions: Option<Vec<String>>,
}

/// Plugin discovery result
#[derive(Debug)]
pub struct DiscoveredPlugin {
    pub manifest: PluginManifest,
    pub manifest_path: PathBuf,
    pub plugin_dir: PathBuf,
}

/// Main plugin manager
#[derive(Debug)]
pub struct PluginManager {
    registry: RwLock<PluginRegistry>,
    sandbox: PluginSandbox,
    config_dir: PathBuf,
    data_dir: PathBuf,
    temp_dir: PathBuf,
    active_plugins: RwLock<HashMap<PluginId, PluginState>>,
    extensions: RwLock<ExtensionManager>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PluginState {
    Uninitialized,
    Initializing,
    Active,
    Error,
    Disabled,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new(config_dir: PathBuf, data_dir: PathBuf, temp_dir: PathBuf) -> Self {
        Self {
            registry: RwLock::new(PluginRegistry::new()),
            sandbox: PluginSandbox::new(),
            config_dir,
            data_dir,
            temp_dir,
            active_plugins: RwLock::new(HashMap::new()),
            extensions: RwLock::new(ExtensionManager::new()),
        }
    }

    /// Register a plugin
    pub async fn register_plugin<P: Plugin + 'static>(&self, plugin: P) -> TuiResult<PluginId> {
        let id = plugin.id();
        let mut registry = self.registry.write().await;
        registry.register(plugin)?;
        let mut active_plugins = self.active_plugins.write().await;
        active_plugins.insert(id.clone(), PluginState::Uninitialized);
        Ok(id)
    }

    /// Load and initialize a plugin
    pub async fn load_plugin(&self, id: &PluginId, model: &AppModel) -> TuiResult<()> {
        let mut registry = self.registry.write().await;
        let plugin = registry.get_mut(id).ok_or_else(|| {
            crate::error::TuiError::Plugin { message: format!("Plugin '{}' not found", id.0) }
        })?;

        let mut active_plugins = self.active_plugins.write().await;
        let previous_state = active_plugins.get(id).copied().unwrap_or(PluginState::Uninitialized);

        // Check if plugin is already active
        if previous_state == PluginState::Active {
            return Ok(());
        }

        active_plugins.insert(id.clone(), PluginState::Initializing);

        let context = PluginContext {
            app_model: model,
            config_dir: self.config_dir.clone(),
            data_dir: self.data_dir.clone(),
            temp_dir: self.temp_dir.clone(),
        };

        match plugin.initialize(&context).await {
            Ok(()) => {
                active_plugins.insert(id.clone(), PluginState::Active);
                tracing::info!("Plugin '{}' initialized successfully", plugin.name());
                Ok(())
            }
            Err(e) => {
                active_plugins.insert(id.clone(), PluginState::Error);
                tracing::error!("Plugin '{}' failed to initialize: {}", plugin.name(), e);
                Err(e)
            }
        }
    }

    /// Unload a plugin
    pub async fn unload_plugin(&self, id: &PluginId) -> TuiResult<()> {
        let mut registry = self.registry.write().await;
        let plugin_name = if let Some(plugin) = registry.get(id) {
            plugin.name().to_string()
        } else {
            id.0.clone()
        };

        // Set state to disabled before cleanup
        let mut active_plugins = self.active_plugins.write().await;
        active_plugins.insert(id.clone(), PluginState::Disabled);

        // Perform cleanup
        if let Some(plugin) = registry.get_mut(id) {
            match plugin.cleanup().await {
                Ok(()) => {
                    tracing::info!("Plugin '{}' cleaned up successfully", plugin_name);
                }
                Err(e) => {
                    tracing::warn!("Plugin '{}' cleanup failed: {}", plugin_name, e);
                    // Continue with unloading even if cleanup fails
                }
            }
        }

        // Remove from registry and active plugins
        registry.unregister(id)?;
        active_plugins.remove(id);

        tracing::info!("Plugin '{}' unloaded", plugin_name);
        Ok(())
    }

    /// Send a message to a plugin
    pub async fn send_message(&self, target: &PluginId, message: PluginMessage) -> TuiResult<Vec<PluginMessage>> {
        let mut registry = self.registry.write().await;
        let plugin = registry.get_mut(target).ok_or_else(|| {
            crate::error::TuiError::Plugin { message: format!("Plugin '{}' not found", target.0) }
        })?;

        let responses = plugin.handle_message(&message).await;
        Ok(responses)
    }

    /// Render all active plugins
    pub async fn render_plugins(&self, area: Rect, model: &AppModel) -> TuiResult<Vec<Line<'static>>> {
        let registry = self.registry.read().await;
        let active_plugins = self.active_plugins.read().await;

        let mut all_lines = Vec::new();
        for (id, state) in &*active_plugins {
            if *state == PluginState::Active {
                if let Some(plugin) = registry.get(id) {
                    let lines = plugin.render(area, model).await?;
                    all_lines.extend(lines);
                }
            }
        }
        Ok(all_lines)
    }

    /// Get the state of a plugin
    pub async fn get_plugin_state(&self, id: &PluginId) -> PluginState {
        self.active_plugins.read().await
            .get(id)
            .copied()
            .unwrap_or(PluginState::Uninitialized)
    }

    /// List all registered plugins with their states
    pub async fn list_plugins(&self) -> Vec<(PluginId, PluginState)> {
        let active_plugins = self.active_plugins.read().await;
        active_plugins.iter()
            .map(|(id, state)| (id.clone(), *state))
            .collect()
    }

    /// Configure the plugin sandbox
    pub fn configure_sandbox<F>(&mut self, f: F)
    where
        F: FnOnce(PluginSandbox) -> PluginSandbox,
    {
        self.sandbox = f(std::mem::take(&mut self.sandbox));
    }

    /// Discover plugins in a directory
    pub async fn discover_plugins(&self, search_paths: &[PathBuf]) -> TuiResult<Vec<DiscoveredPlugin>> {
        let mut discovered = Vec::new();

        for search_path in search_paths {
            if !search_path.exists() {
                continue;
            }

            for entry in WalkDir::new(search_path).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() && entry.file_name() == "plugin.json" {
                    match self.load_plugin_manifest(&entry.path().to_path_buf()).await {
                        Ok(manifest) => {
                            let plugin_dir = entry.path().parent().unwrap_or(search_path).to_path_buf();
                            discovered.push(DiscoveredPlugin {
                                manifest,
                                manifest_path: entry.path().to_path_buf(),
                                plugin_dir,
                            });
                        }
                        Err(e) => {
                            tracing::warn!("Failed to load plugin manifest {}: {}", entry.path().display(), e);
                        }
                    }
                }
            }
        }

        Ok(discovered)
    }

    /// Load a plugin manifest from file
    async fn load_plugin_manifest(&self, path: &PathBuf) -> TuiResult<PluginManifest> {
        let content = tokio::fs::read_to_string(path).await
            .map_err(|e| crate::error::TuiError::Plugin { message: format!(
                "Failed to read plugin manifest {}: {}", path.display(), e
            ) })?;

        let manifest: PluginManifest = serde_json::from_str(&content)
            .map_err(|e| crate::error::TuiError::Plugin { message: format!(
                "Failed to parse plugin manifest {}: {}", path.display(), e
            ) })?;

        Ok(manifest)
    }

    /// Load a discovered plugin
    pub async fn load_discovered_plugin(&self, discovered: &DiscoveredPlugin, model: &AppModel) -> TuiResult<PluginId> {
        // Validate plugin manifest for security
        self.sandbox.validate_manifest(&discovered.manifest)?;

        // For now, we'll create a placeholder plugin since we can't dynamically load Rust code
        // In a real implementation, this would load the plugin library and instantiate the plugin
        let plugin = PlaceholderPlugin::new(discovered.manifest.clone());

        self.register_plugin(plugin).await?;
        self.load_plugin(&PluginId::from(discovered.manifest.id.clone()), model).await?;

        Ok(PluginId::from(discovered.manifest.id.clone()))
    }

    /// Discover and load all plugins from default search paths
    pub async fn discover_and_load_all(&self, model: &AppModel) -> TuiResult<Vec<PluginId>> {
        let search_paths = vec![
            self.config_dir.join("plugins"),
            PathBuf::from("./plugins"), // Relative to current directory
        ];

        let discovered = self.discover_plugins(&search_paths).await?;
        let mut loaded = Vec::new();

        for plugin in discovered {
            match self.load_discovered_plugin(&plugin, model).await {
                Ok(id) => {
                    tracing::info!("Loaded plugin: {}", plugin.manifest.name);
                    loaded.push(id);
                }
                Err(e) => {
                    tracing::warn!("Failed to load plugin {}: {}", plugin.manifest.name, e);
                }
            }
        }

        Ok(loaded)
    }

    /// Reload a plugin (unload and load again)
    pub async fn reload_plugin(&self, id: &PluginId, model: &AppModel) -> TuiResult<()> {
        tracing::info!("Reloading plugin: {}", id.0);

        // Unload first
        self.unload_plugin(id).await?;

        // Check if plugin is still registered (might have been removed)
        let registry = self.registry.read().await;
        if registry.contains(id) {
            // Load again
            drop(registry); // Release read lock
            self.load_plugin(id, model).await?;
            tracing::info!("Plugin '{}' reloaded successfully", id.0);
        } else {
            tracing::info!("Plugin '{}' was removed during reload", id.0);
        }

        Ok(())
    }

    /// Get detailed plugin information
    pub async fn get_plugin_info(&self, id: &PluginId) -> Option<PluginInfo> {
        let registry = self.registry.read().await;
        let active_plugins = self.active_plugins.read().await;

        registry.get(id).map(|plugin| {
            PluginInfo {
                id: plugin.id(),
                name: plugin.name().to_string(),
                version: plugin.version().to_string(),
                metadata: plugin.metadata(),
                state: active_plugins.get(&plugin.id()).copied().unwrap_or(PluginState::Uninitialized),
            }
        })
    }

    /// Enable a disabled plugin
    pub async fn enable_plugin(&self, id: &PluginId, model: &AppModel) -> TuiResult<()> {
        let active_plugins = self.active_plugins.read().await;
        if let Some(&PluginState::Disabled) = active_plugins.get(id) {
            drop(active_plugins);
            self.load_plugin(id, model).await
        } else {
            Err(crate::error::TuiError::Plugin { message: format!(
                "Plugin '{}' is not disabled", id.0
            ) })
        }
    }

    /// Disable an active plugin
    pub async fn disable_plugin(&self, id: &PluginId) -> TuiResult<()> {
        let mut active_plugins = self.active_plugins.write().await;
        if let Some(&PluginState::Active) = active_plugins.get(id) {
            active_plugins.insert(id.clone(), PluginState::Disabled);
            tracing::info!("Plugin '{}' disabled", id.0);
            Ok(())
        } else {
            Err(crate::error::TuiError::Plugin { message: format!(
                "Plugin '{}' is not active", id.0
            ) })
        }
    }

    /// Get access to the extension manager
    pub fn extensions(&self) -> &RwLock<ExtensionManager> {
        &self.extensions
    }

    /// Register component extensions from a plugin
    pub async fn register_component_extensions(&self, plugin_id: &PluginId, extensions: Vec<(String, Box<dyn ComponentExtension>)>) -> TuiResult<()> {
        let mut ext_manager = self.extensions.write().await;
        let count = extensions.len();
        for (component_id, extension) in extensions {
            ext_manager.register_component_extension(&component_id, extension);
        }
        tracing::info!("Plugin '{}' registered {} component extensions", plugin_id.0, count);
        Ok(())
    }

    /// Register command extensions from a plugin
    pub async fn register_command_extensions(&self, plugin_id: &PluginId, extensions: Vec<(String, Box<dyn CommandExtension>)>) -> TuiResult<()> {
        let mut ext_manager = self.extensions.write().await;
        let count = extensions.len();
        for (command, extension) in extensions {
            ext_manager.register_command_extension(&command, extension);
        }
        tracing::info!("Plugin '{}' registered {} command extensions", plugin_id.0, count);
        Ok(())
    }

    /// Register theme extensions from a plugin
    pub async fn register_theme_extensions(&self, plugin_id: &PluginId, extensions: Vec<Box<dyn ThemeExtension>>) -> TuiResult<()> {
        let mut ext_manager = self.extensions.write().await;
        let count = extensions.len();
        for extension in extensions {
            ext_manager.register_theme_extension(extension);
        }
        tracing::info!("Plugin '{}' registered {} theme extensions", plugin_id.0, count);
        Ok(())
    }
}

/// Detailed plugin information
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub id: PluginId,
    pub name: String,
    pub version: String,
    pub metadata: PluginMetadata,
    pub state: PluginState,
}

/// Extension point for component customization
#[async_trait::async_trait]
pub trait ComponentExtension: Send + Sync {
    /// Get the extension identifier
    fn extension_id(&self) -> &str;

    /// Called before component rendering to allow modifications
    async fn pre_render(&self, component_id: &str, model: &AppModel) -> TuiResult<()> {
        let _ = (component_id, model);
        Ok(())
    }

    /// Called after component rendering to allow modifications
    async fn post_render(&self, component_id: &str, lines: &mut Vec<Line<'static>>) -> TuiResult<()> {
        let _ = (component_id, lines);
        Ok(())
    }

    /// Called when component receives input
    async fn on_input(&self, component_id: &str, input: &str, model: &AppModel) -> TuiResult<Option<String>> {
        let _ = (component_id, input, model);
        Ok(None)
    }
}

/// Extension point for command customization
#[async_trait::async_trait]
pub trait CommandExtension: Send + Sync {
    /// Get the extension identifier
    fn extension_id(&self) -> &str;

    /// Called before command execution
    async fn pre_execute(&self, command: &str, args: &[String], model: &AppModel) -> TuiResult<()> {
        let _ = (command, args, model);
        Ok(())
    }

    /// Called after command execution
    async fn post_execute(&self, command: &str, result: &TuiResult<()>, model: &AppModel) -> TuiResult<()> {
        let _ = (command, result, model);
        Ok(())
    }

    /// Register additional commands
    async fn register_commands(&self) -> Vec<PluginCommand> {
        vec![]
    }
}

/// Plugin-defined command
#[derive(Debug, Clone)]
pub struct PluginCommand {
    pub name: String,
    pub description: String,
    pub handler: PluginCommandHandler,
}

/// Command handler type for plugins
pub type PluginCommandHandler = Box<dyn Fn(&[String], &AppModel) -> TuiResult<()> + Send + Sync>;

/// Extension point for theme customization
#[async_trait::async_trait]
pub trait ThemeExtension: Send + Sync {
    /// Get the extension identifier
    fn extension_id(&self) -> &str;

    /// Called when theme is loaded to allow modifications
    async fn customize_theme(&self, theme: &mut crate::Theme) -> TuiResult<()> {
        let _ = theme;
        Ok(())
    }

    /// Register additional theme variants
    async fn register_themes(&self) -> Vec<PluginTheme> {
        vec![]
    }
}

/// Plugin-defined theme
#[derive(Debug, Clone)]
pub struct PluginTheme {
    pub name: String,
    pub theme: crate::Theme,
}

/// Extension point manager
#[derive(Debug)]
pub struct ExtensionManager {
    component_extensions: HashMap<String, Vec<Box<dyn ComponentExtension>>>,
    command_extensions: HashMap<String, Vec<Box<dyn CommandExtension>>>,
    theme_extensions: Vec<Box<dyn ThemeExtension>>,
}

impl ExtensionManager {
    pub fn new() -> Self {
        Self {
            component_extensions: HashMap::new(),
            command_extensions: HashMap::new(),
            theme_extensions: Vec::new(),
        }
    }

    /// Register a component extension
    pub fn register_component_extension(&mut self, component_id: &str, extension: Box<dyn ComponentExtension>) {
        self.component_extensions
            .entry(component_id.to_string())
            .or_insert_with(Vec::new)
            .push(extension);
    }

    /// Register a command extension
    pub fn register_command_extension(&mut self, command: &str, extension: Box<dyn CommandExtension>) {
        self.command_extensions
            .entry(command.to_string())
            .or_insert_with(Vec::new)
            .push(extension);
    }

    /// Register a theme extension
    pub fn register_theme_extension(&mut self, extension: Box<dyn ThemeExtension>) {
        self.theme_extensions.push(extension);
    }

    /// Get component extensions for a specific component
    pub fn get_component_extensions(&self, component_id: &str) -> &[Box<dyn ComponentExtension>] {
        self.component_extensions
            .get(component_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Get command extensions for a specific command
    pub fn get_command_extensions(&self, command: &str) -> &[Box<dyn CommandExtension>] {
        self.command_extensions
            .get(command)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Get all theme extensions
    pub fn get_theme_extensions(&self) -> &[Box<dyn ThemeExtension>] {
        &self.theme_extensions
    }

    /// Execute component pre-render extensions
    pub async fn execute_component_pre_render(&self, component_id: &str, model: &AppModel) -> TuiResult<()> {
        for extension in self.get_component_extensions(component_id) {
            extension.pre_render(component_id, model).await?;
        }
        Ok(())
    }

    /// Execute component post-render extensions
    pub async fn execute_component_post_render(&self, component_id: &str, lines: &mut Vec<Line<'static>>) -> TuiResult<()> {
        for extension in self.get_component_extensions(component_id) {
            extension.post_render(component_id, lines).await?;
        }
        Ok(())
    }

    /// Execute command pre-execute extensions
    pub async fn execute_command_pre_execute(&self, command: &str, args: &[String], model: &AppModel) -> TuiResult<()> {
        for extension in self.get_command_extensions(command) {
            extension.pre_execute(command, args, model).await?;
        }
        Ok(())
    }

    /// Execute command post-execute extensions
    pub async fn execute_command_post_execute(&self, command: &str, result: &TuiResult<()>, model: &AppModel) -> TuiResult<()> {
        for extension in self.get_command_extensions(command) {
            extension.post_execute(command, result, model).await?;
        }
        Ok(())
    }
}

impl Default for ExtensionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new(
            PathBuf::from("~/.config/ricecoder/plugins"),
            PathBuf::from("~/.local/share/ricecoder/plugins"),
            std::env::temp_dir().join("ricecoder-plugins"),
        )
    }
}

/// Placeholder plugin implementation for discovered plugins
/// In a real implementation, this would be replaced by dynamically loaded plugin code
#[derive(Debug)]
pub struct PlaceholderPlugin {
    manifest: PluginManifest,
}

impl PlaceholderPlugin {
    pub fn new(manifest: PluginManifest) -> Self {
        Self { manifest }
    }
}

#[async_trait::async_trait]
impl Plugin for PlaceholderPlugin {
    fn id(&self) -> PluginId {
        PluginId::from(self.manifest.id.clone())
    }

    fn name(&self) -> &str {
        &self.manifest.name
    }

    fn version(&self) -> &str {
        &self.manifest.version
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: self.id(),
            name: self.manifest.name.clone(),
            version: self.manifest.version.clone(),
            description: self.manifest.description.clone(),
            author: self.manifest.author.clone(),
            homepage: self.manifest.homepage.clone(),
            repository: self.manifest.repository.clone(),
        }
    }

    async fn initialize(&mut self, _context: &PluginContext) -> TuiResult<()> {
        tracing::info!("Initializing placeholder plugin: {}", self.name());
        Ok(())
    }

    async fn handle_message(&mut self, message: &PluginMessage) -> Vec<PluginMessage> {
        tracing::debug!("Placeholder plugin {} received message: {:?}", self.name(), message);
        // For now, just echo the message back
        vec![message.clone()]
    }

    async fn render(&self, _area: Rect, _model: &AppModel) -> TuiResult<Vec<Line>> {
        // Placeholder plugins don't render anything by default
        Ok(vec![])
    }

    async fn cleanup(&mut self) -> TuiResult<()> {
        tracing::info!("Cleaning up placeholder plugin: {}", self.name());
        Ok(())
    }
}