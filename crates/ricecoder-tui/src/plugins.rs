//! Plugin system for RiceCoder TUI
//!
//! This module implements a plugin architecture that allows third-party extensions
//! to customize and extend the TUI functionality.

use crate::error::TuiResult;
use crate::model::{AppMessage, AppModel};
use crate::Component;
use ratatui::prelude::*;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
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
    /// Custom message with string payload
    Custom(String),
    /// JSON message
    Json(serde_json::Value),
    /// Binary message
    Binary(Vec<u8>),
}

/// Plugin capabilities for enhanced functionality
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PluginCapability {
    /// Can provide UI components
    UiComponent,
    /// Can provide commands
    Command,
    /// Can provide themes
    Theme,
    /// Can handle file types
    FileHandler,
    /// Can provide keyboard shortcuts
    Keybinding,
    /// Can provide accessibility features
    Accessibility,
    /// Can provide integrations
    Integration,
}

/// Plugin version information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl std::fmt::Display for PluginVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Enhanced plugin metadata
#[derive(Debug, Clone)]
pub struct EnhancedPluginMetadata {
    pub base: PluginMetadata,
    pub version: PluginVersion,
    pub license: Option<String>,
    pub min_tui_version: Option<PluginVersion>,
    pub max_tui_version: Option<PluginVersion>,
    pub capabilities: Vec<PluginCapability>,
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
    async fn initialize<'a>(&mut self, context: &PluginContext<'a>) -> TuiResult<()>;

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

    /// Get enhanced plugin metadata
    fn enhanced_metadata(&self) -> EnhancedPluginMetadata {
        EnhancedPluginMetadata {
            base: self.metadata(),
            version: PluginVersion {
                major: 1,
                minor: 0,
                patch: 0,
            },
            license: None,
            min_tui_version: None,
            max_tui_version: None,
            capabilities: vec![],
        }
    }
}

/// UI Component plugin trait for plugins that provide UI components
#[async_trait::async_trait]
pub trait UiComponentPlugin: Plugin {
    /// Render the plugin's UI component
    fn render_component(&self, frame: &mut Frame, area: Rect, model: &AppModel);

    /// Handle input events for the component
    fn handle_component_input(&mut self, message: &AppMessage, model: &AppModel) -> bool;

    /// Get the component's preferred size
    fn preferred_size(&self) -> Option<(u16, u16)> {
        None
    }

    /// Check if component is currently focused
    fn is_component_focused(&self) -> bool {
        false
    }
}

/// Command plugin trait for plugins that provide commands
#[async_trait::async_trait]
pub trait CommandPlugin: Plugin {
    /// Get available commands
    fn available_commands(&self) -> Vec<PluginCommand>;

    /// Execute a command
    async fn execute_command(
        &mut self,
        command: &str,
        args: &[String],
        context: &PluginContext,
    ) -> TuiResult<CommandResult>;
}

/// Theme plugin trait for plugins that provide themes
#[async_trait::async_trait]
pub trait ThemePlugin: Plugin {
    /// Get available themes
    fn available_themes(&self) -> Vec<PluginTheme>;

    /// Apply a theme
    async fn apply_theme(&mut self, theme_id: &str) -> TuiResult<()>;
}

/// Plugin command definition
#[derive(Debug, Clone)]
pub struct PluginCommandInfo {
    pub name: String,
    pub description: String,
    pub usage: String,
    pub examples: Vec<String>,
}

/// Plugin theme definition
#[derive(Debug, Clone)]
pub struct PluginTheme {
    pub id: String,
    pub name: String,
    pub description: String,
    pub theme_data: serde_json::Value,
    pub author: String,
    pub version: String,
    pub tags: Vec<String>,
}

/// Theme plugin implementation
#[derive(Debug, Clone)]
pub struct ThemePluginImpl {
    metadata: EnhancedPluginMetadata,
    themes: Vec<PluginTheme>,
    active_theme: Option<String>,
}

impl ThemePluginImpl {
    /// Create a new theme plugin
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            metadata: EnhancedPluginMetadata {
                base: PluginMetadata {
                    id: PluginId::from(id),
                    name: name.to_string(),
                    version: "1.0.0".to_string(),
                    description: format!("Theme plugin: {}", name),
                    author: "Unknown".to_string(),
                    homepage: None,
                    repository: None,
                },
                version: PluginVersion {
                    major: 1,
                    minor: 0,
                    patch: 0,
                },
                license: Some("MIT".to_string()),
                min_tui_version: None,
                max_tui_version: None,
                capabilities: vec![PluginCapability::Theme],
            },
            themes: Vec::new(),
            active_theme: None,
        }
    }

    /// Add a theme to the plugin
    pub fn add_theme(&mut self, theme: PluginTheme) {
        self.themes.push(theme);
    }

    /// Load themes from a directory
    pub fn load_themes_from_dir(&mut self, dir_path: &std::path::Path) -> TuiResult<()> {
        if !dir_path.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("yaml")
                || path.extension().and_then(|s| s.to_str()) == Some("yml")
            {
                match self.load_theme_from_file(&path) {
                    Ok(theme) => self.add_theme(theme),
                    Err(e) => tracing::warn!("Failed to load theme from {}: {}", path.display(), e),
                }
            }
        }

        Ok(())
    }

    /// Load a single theme from a YAML file
    fn load_theme_from_file(&self, file_path: &std::path::Path) -> TuiResult<PluginTheme> {
        let content = std::fs::read_to_string(file_path)?;
        let theme_data: serde_json::Value = serde_yaml::from_str(&content)?;

        // Extract theme metadata from the YAML
        let id = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let name = theme_data
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(&id)
            .to_string();

        let description = theme_data
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("A theme")
            .to_string();

        let author = theme_data
            .get("author")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let version = theme_data
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("1.0.0")
            .to_string();

        let tags = theme_data
            .get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        Ok(PluginTheme {
            id,
            name,
            description,
            theme_data,
            author,
            version,
            tags,
        })
    }

    /// Validate a theme
    pub fn validate_theme(&self, theme: &PluginTheme) -> TuiResult<()> {
        // Check required fields
        if theme.id.is_empty() {
            return Err(crate::error::TuiError::Plugin {
                message: "Theme ID cannot be empty".to_string(),
            });
        }

        if theme.name.is_empty() {
            return Err(crate::error::TuiError::Plugin {
                message: "Theme name cannot be empty".to_string(),
            });
        }

        // Validate theme data structure (basic check)
        if !theme.theme_data.is_object() {
            return Err(crate::error::TuiError::Plugin {
                message: "Theme data must be a JSON object".to_string(),
            });
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Plugin for ThemePluginImpl {
    fn id(&self) -> PluginId {
        "theme".into()
    }

    fn name(&self) -> &str {
        "Theme Plugin"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }
    fn metadata(&self) -> PluginMetadata {
        self.metadata.base.clone()
    }

    async fn initialize<'a>(&mut self, context: &PluginContext<'a>) -> TuiResult<()> {
        // Load themes from plugin data directory
        let themes_dir = context.data_dir.join("themes");
        self.load_themes_from_dir(&themes_dir)?;

        // Validate all loaded themes
        for theme in &self.themes {
            self.validate_theme(theme)?;
        }

        tracing::info!(
            "Theme plugin {} initialized with {} themes",
            self.metadata().name,
            self.themes.len()
        );
        Ok(())
    }

    async fn handle_message(&mut self, message: &PluginMessage) -> Vec<PluginMessage> {
        match message {
            PluginMessage::Custom(msg) if msg.starts_with("activate_theme:") => {
                let theme_id = msg.trim_start_matches("activate_theme:");
                if let Some(theme) = self.themes.iter().find(|t| t.id == theme_id) {
                    self.active_theme = Some(theme_id.to_string());
                    vec![PluginMessage::Custom(format!(
                        "theme_activated:{}",
                        theme_id
                    ))]
                } else {
                    vec![PluginMessage::Error(format!(
                        "Theme not found: {}",
                        theme_id
                    ))]
                }
            }
            _ => vec![],
        }
    }

    async fn render(&self, area: Rect, model: &AppModel) -> TuiResult<Vec<Line>> {
        let mut lines = Vec::new();

        lines.push(Line::from(format!(
            "🎨 {} - {} themes",
            self.metadata().name,
            self.themes.len()
        )));

        if let Some(active) = &self.active_theme {
            lines.push(Line::from(format!("Active theme: {}", active)));
        }

        lines.push(Line::from(""));
        lines.push(Line::from("Available themes:"));

        for theme in &self.themes {
            let active_marker = if self.active_theme.as_ref() == Some(&theme.id) {
                " ✓"
            } else {
                ""
            };
            lines.push(Line::from(format!(
                "  • {} - {}{}",
                theme.name, theme.description, active_marker
            )));
        }

        Ok(lines)
    }

    async fn cleanup(&mut self) -> TuiResult<()> {
        self.themes.clear();
        self.active_theme = None;
        Ok(())
    }
}

#[async_trait::async_trait]
impl ThemePlugin for ThemePluginImpl {
    fn available_themes(&self) -> Vec<PluginTheme> {
        self.themes.clone()
    }

    async fn apply_theme(&mut self, theme_id: &str) -> TuiResult<()> {
        if let Some(theme) = self.themes.iter().find(|t| t.id == theme_id) {
            self.active_theme = Some(theme_id.to_string());
            // In a real implementation, this would apply the theme to the TUI
            tracing::info!("Applied theme: {}", theme.name);
            Ok(())
        } else {
            Err(crate::error::TuiError::Plugin {
                message: format!("Theme not found: {}", theme_id),
            })
        }
    }
}

/// Command execution result
#[derive(Debug, Clone)]
pub enum CommandResult {
    Success(String),
    Error(String),
    Output(String),
}

/// Enhanced plugin registry for managing different plugin types
pub struct EnhancedPluginRegistry {
    plugins: HashMap<PluginId, Box<dyn Plugin>>,
    ui_components: HashMap<PluginId, Box<dyn UiComponentPlugin>>,
    command_plugins: HashMap<PluginId, Box<dyn CommandPlugin>>,
    theme_plugins: HashMap<PluginId, Box<dyn ThemePlugin>>,
    plugin_states: HashMap<PluginId, PluginState>,
}

impl EnhancedPluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            ui_components: HashMap::new(),
            command_plugins: HashMap::new(),
            theme_plugins: HashMap::new(),
            plugin_states: HashMap::new(),
        }
    }

    /// Register a generic plugin
    pub fn register_plugin(&mut self, plugin: Box<dyn Plugin>) -> TuiResult<()> {
        let id = plugin.id();
        self.plugins.insert(id.clone(), plugin);
        self.plugin_states.insert(id, PluginState::Active);
        Ok(())
    }

    /// Register a UI component plugin
    pub fn register_ui_component(
        &mut self,
        component: Box<dyn UiComponentPlugin>,
    ) -> TuiResult<()> {
        let id = component.id();
        // TODO: Fix clone_box method
        // self.register_plugin(component.clone_box())?;
        self.ui_components.insert(id, component);
        Ok(())
    }

    /// Register a command plugin
    pub fn register_command_plugin(&mut self, plugin: Box<dyn CommandPlugin>) -> TuiResult<()> {
        let id = plugin.id();
        // TODO: Fix clone_box method
        // self.register_plugin(plugin.clone_box())?;
        self.command_plugins.insert(id, plugin);
        Ok(())
    }

    /// Register a theme plugin
    pub fn register_theme_plugin(&mut self, plugin: Box<dyn ThemePlugin>) -> TuiResult<()> {
        let id = plugin.id();
        // TODO: Fix clone_box method
        // self.register_plugin(plugin.clone_box())?;
        self.theme_plugins.insert(id, plugin);
        Ok(())
    }

    pub fn unregister(&mut self, id: &PluginId) -> Option<Box<dyn Plugin>> {
        self.ui_components.remove(id);
        self.command_plugins.remove(id);
        self.theme_plugins.remove(id);
        self.plugin_states.remove(id);
        self.plugins.remove(id)
    }

    pub fn get(&self, id: &PluginId) -> Option<&dyn Plugin> {
        self.plugins.get(id).map(|p| p.as_ref())
    }

    pub fn get_state(&self, id: &PluginId) -> Option<PluginState> {
        self.plugin_states.get(id).copied()
    }

    pub fn set_state(&mut self, id: PluginId, state: PluginState) {
        self.plugin_states.insert(id, state);
    }

    pub fn all_plugins(&self) -> Vec<&dyn Plugin> {
        self.plugins.values().map(|p| p.as_ref()).collect()
    }

    pub fn ui_components(&self) -> Vec<&dyn UiComponentPlugin> {
        self.ui_components.values().map(|p| p.as_ref()).collect()
    }

    pub fn command_plugins(&self) -> Vec<&dyn CommandPlugin> {
        self.command_plugins.values().map(|p| p.as_ref()).collect()
    }

    pub fn theme_plugins(&self) -> Vec<&dyn ThemePlugin> {
        self.theme_plugins.values().map(|p| p.as_ref()).collect()
    }
}

/// Basic plugin registry for backward compatibility
pub struct PluginRegistry {
    plugins: HashMap<PluginId, Box<dyn Plugin>>,
}

// Keep the original PluginRegistry for backward compatibility

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    /// Register a plugin
    pub fn register<P: Plugin + 'static>(&mut self, plugin: P) -> TuiResult<()> {
        let id = plugin.id();
        if self.plugins.contains_key(&id) {
            return Err(crate::error::TuiError::Plugin {
                message: format!("Plugin with ID '{}' already registered", id.0),
            });
        }
        self.plugins.insert(id, Box::new(plugin));
        Ok(())
    }

    /// Unregister a plugin
    pub fn unregister(&mut self, id: &PluginId) -> TuiResult<()> {
        if self.plugins.remove(id).is_none() {
            return Err(crate::error::TuiError::Plugin {
                message: format!("Plugin with ID '{}' not found", id.0),
            });
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

impl Default for PluginSandbox {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginSandbox {
    pub fn new() -> Self {
        Self {
            allowed_paths: vec![],
            max_memory_mb: 50, // 50MB default limit
            timeout_ms: 5000,  // 5 second timeout
            allowed_commands: vec![],
            max_file_size_mb: 10,       // 10MB max file size
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
                return Err(crate::error::TuiError::Plugin {
                    message: "Network access not allowed for plugins".to_string(),
                });
            }
            PluginOperation::SystemCommand => {
                return Err(crate::error::TuiError::Plugin {
                    message: "System commands not allowed for plugins".to_string(),
                });
            }
        }
        Ok(())
    }

    /// Validate file access permissions
    fn validate_file_access(&self, path: &PathBuf) -> TuiResult<()> {
        // Check if path is within allowed directories
        let is_allowed = self.allowed_paths.iter().any(|allowed| {
            path.starts_with(allowed)
                || path
                    .canonicalize()
                    .map(|canonical| canonical.starts_with(allowed))
                    .unwrap_or(false)
        });

        if !is_allowed {
            return Err(crate::error::TuiError::Plugin {
                message: format!(
                    "Plugin attempted to access unauthorized path: {}",
                    path.display()
                ),
            });
        }

        // Check file size if it exists
        if let Ok(metadata) = std::fs::metadata(path) {
            let size_mb = metadata.len() / (1024 * 1024);
            if size_mb > self.max_file_size_mb as u64 {
                return Err(crate::error::TuiError::Plugin {
                    message: format!(
                        "Plugin attempted to access file larger than {}MB limit: {}",
                        self.max_file_size_mb,
                        path.display()
                    ),
                });
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
                        return Err(crate::error::TuiError::Plugin {
                            message: format!(
                                "Plugin requests dangerous permission: {}",
                                permission
                            ),
                        });
                    }
                    "filesystem" | "config" => {
                        // These are allowed but logged
                        tracing::info!(
                            "Plugin {} requests permission: {}",
                            manifest.name,
                            permission
                        );
                    }
                    _ => {
                        tracing::warn!(
                            "Plugin {} requests unknown permission: {}",
                            manifest.name,
                            permission
                        );
                    }
                }
            }
        }

        // Validate entry point exists and is safe
        let entry_path = PathBuf::from(&manifest.entry_point);
        if entry_path.is_absolute() || entry_path.starts_with("..") {
            return Err(crate::error::TuiError::Plugin {
                message: format!(
                    "Plugin entry point contains unsafe path: {}",
                    manifest.entry_point
                ),
            });
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
        self.operations
            .retain(|&time| now.duration_since(time).as_secs() < 1);

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
        let plugin = registry
            .get_mut(id)
            .ok_or_else(|| crate::error::TuiError::Plugin {
                message: format!("Plugin '{}' not found", id.0),
            })?;

        let mut active_plugins = self.active_plugins.write().await;
        let previous_state = active_plugins
            .get(id)
            .copied()
            .unwrap_or(PluginState::Uninitialized);

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
    pub async fn send_message(
        &self,
        target: &PluginId,
        message: PluginMessage,
    ) -> TuiResult<Vec<PluginMessage>> {
        let mut registry = self.registry.write().await;
        let plugin = registry
            .get_mut(target)
            .ok_or_else(|| crate::error::TuiError::Plugin {
                message: format!("Plugin '{}' not found", target.0),
            })?;

        let responses = plugin.handle_message(&message).await;
        Ok(responses)
    }

    /// Render all active plugins
    pub async fn render_plugins(&self, area: Rect, model: &AppModel) -> TuiResult<Vec<Line>> {
        let registry = self.registry.read().await;
        let active_plugins = self.active_plugins.read().await;

        let mut all_lines = Vec::new();
        {
            let registry = self.registry.read().await;
            let active_plugins = self.active_plugins.read().await;

            for (id, state) in &*active_plugins {
                if *state == PluginState::Active {
                    if let Some(plugin) = registry.get(id) {
                        // Clone plugin data to avoid lifetime issues
                        let plugin_id = plugin.id();
                        let plugin_name = plugin.name();
                        let plugin_version = plugin.version();

                        // Create a simple line with plugin info instead of calling render
                        let line = Line::from(format!(
                            "Plugin: {} v{} (ID: {})",
                            plugin_name, plugin_version, plugin_id.0
                        ));
                        all_lines.push(line);
                    }
                }
            }
        }

        Ok(all_lines)
    }

    /// Get the state of a plugin
    pub async fn get_plugin_state(&self, id: &PluginId) -> PluginState {
        self.active_plugins
            .read()
            .await
            .get(id)
            .copied()
            .unwrap_or(PluginState::Uninitialized)
    }

    /// List all registered plugins with their states
    pub async fn list_plugins(&self) -> Vec<(PluginId, PluginState)> {
        let active_plugins = self.active_plugins.read().await;
        active_plugins
            .iter()
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
    pub async fn discover_plugins(
        &self,
        search_paths: &[PathBuf],
    ) -> TuiResult<Vec<DiscoveredPlugin>> {
        let mut discovered = Vec::new();

        for search_path in search_paths {
            if !search_path.exists() {
                continue;
            }

            for entry in WalkDir::new(search_path).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() && entry.file_name() == "plugin.json" {
                    match self.load_plugin_manifest(&entry.path().to_path_buf()).await {
                        Ok(manifest) => {
                            let plugin_dir =
                                entry.path().parent().unwrap_or(search_path).to_path_buf();
                            discovered.push(DiscoveredPlugin {
                                manifest,
                                manifest_path: entry.path().to_path_buf(),
                                plugin_dir,
                            });
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Failed to load plugin manifest {}: {}",
                                entry.path().display(),
                                e
                            );
                        }
                    }
                }
            }
        }

        Ok(discovered)
    }

    /// Load a plugin manifest from file
    async fn load_plugin_manifest(&self, path: &PathBuf) -> TuiResult<PluginManifest> {
        let content =
            tokio::fs::read_to_string(path)
                .await
                .map_err(|e| crate::error::TuiError::Plugin {
                    message: format!("Failed to read plugin manifest {}: {}", path.display(), e),
                })?;

        let manifest: PluginManifest =
            serde_json::from_str(&content).map_err(|e| crate::error::TuiError::Plugin {
                message: format!("Failed to parse plugin manifest {}: {}", path.display(), e),
            })?;

        Ok(manifest)
    }

    /// Load a discovered plugin
    pub async fn load_discovered_plugin(
        &self,
        discovered: &DiscoveredPlugin,
        model: &AppModel,
    ) -> TuiResult<PluginId> {
        // Validate plugin manifest for security
        self.sandbox.validate_manifest(&discovered.manifest)?;

        // For now, we'll create a placeholder plugin since we can't dynamically load Rust code
        // In a real implementation, this would load the plugin library and instantiate the plugin
        let plugin = PlaceholderPlugin::new(discovered.manifest.clone());

        self.register_plugin(plugin).await?;
        self.load_plugin(&PluginId::from(discovered.manifest.id.clone()), model)
            .await?;

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

        registry.get(id).map(|plugin| PluginInfo {
            id: plugin.id(),
            name: plugin.name().to_string(),
            version: plugin.version().to_string(),
            metadata: plugin.metadata(),
            state: active_plugins
                .get(&plugin.id())
                .copied()
                .unwrap_or(PluginState::Uninitialized),
        })
    }

    /// Enable a disabled plugin
    pub async fn enable_plugin(&self, id: &PluginId, model: &AppModel) -> TuiResult<()> {
        let active_plugins = self.active_plugins.read().await;
        if let Some(&PluginState::Disabled) = active_plugins.get(id) {
            drop(active_plugins);
            self.load_plugin(id, model).await
        } else {
            Err(crate::error::TuiError::Plugin {
                message: format!("Plugin '{}' is not disabled", id.0),
            })
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
            Err(crate::error::TuiError::Plugin {
                message: format!("Plugin '{}' is not active", id.0),
            })
        }
    }

    /// Get access to the extension manager
    pub fn extensions(&self) -> &RwLock<ExtensionManager> {
        &self.extensions
    }

    /// Register component extensions from a plugin
    pub async fn register_component_extensions(
        &self,
        plugin_id: &PluginId,
        extensions: Vec<(String, Box<dyn ComponentExtension>)>,
    ) -> TuiResult<()> {
        let mut ext_manager = self.extensions.write().await;
        let count = extensions.len();
        for (component_id, extension) in extensions {
            ext_manager.register_component_extension(&component_id, extension);
        }
        tracing::info!(
            "Plugin '{}' registered {} component extensions",
            plugin_id.0,
            count
        );
        Ok(())
    }

    /// Register command extensions from a plugin
    pub async fn register_command_extensions(
        &self,
        plugin_id: &PluginId,
        extensions: Vec<(String, Box<dyn CommandExtension>)>,
    ) -> TuiResult<()> {
        let mut ext_manager = self.extensions.write().await;
        let count = extensions.len();
        for (command, extension) in extensions {
            ext_manager.register_command_extension(&command, extension);
        }
        tracing::info!(
            "Plugin '{}' registered {} command extensions",
            plugin_id.0,
            count
        );
        Ok(())
    }

    /// Register theme extensions from a plugin
    pub async fn register_theme_extensions(
        &self,
        plugin_id: &PluginId,
        extensions: Vec<Box<dyn ThemeExtension>>,
    ) -> TuiResult<()> {
        let mut ext_manager = self.extensions.write().await;
        let count = extensions.len();
        for extension in extensions {
            ext_manager.register_theme_extension(extension);
        }
        tracing::info!(
            "Plugin '{}' registered {} theme extensions",
            plugin_id.0,
            count
        );
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
    async fn post_render(
        &self,
        component_id: &str,
        lines: &mut Vec<Line<'static>>,
    ) -> TuiResult<()> {
        let _ = (component_id, lines);
        Ok(())
    }

    /// Called when component receives input
    async fn on_input(
        &self,
        component_id: &str,
        input: &str,
        model: &AppModel,
    ) -> TuiResult<Option<String>> {
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
    async fn post_execute(
        &self,
        command: &str,
        result: &TuiResult<()>,
        model: &AppModel,
    ) -> TuiResult<()> {
        let _ = (command, result, model);
        Ok(())
    }

    /// Register additional commands
    async fn register_commands(&self) -> Vec<PluginCommand> {
        vec![]
    }
}

/// Plugin-defined command
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
pub struct PluginThemeExtension {
    pub name: String,
    pub theme: crate::Theme,
}

/// Extension point manager
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
    pub fn register_component_extension(
        &mut self,
        component_id: &str,
        extension: Box<dyn ComponentExtension>,
    ) {
        self.component_extensions
            .entry(component_id.to_string())
            .or_insert_with(Vec::new)
            .push(extension);
    }

    /// Register a command extension
    pub fn register_command_extension(
        &mut self,
        command: &str,
        extension: Box<dyn CommandExtension>,
    ) {
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
    pub async fn execute_component_pre_render(
        &self,
        component_id: &str,
        model: &AppModel,
    ) -> TuiResult<()> {
        for extension in self.get_component_extensions(component_id) {
            extension.pre_render(component_id, model).await?;
        }
        Ok(())
    }

    /// Execute component post-render extensions
    pub async fn execute_component_post_render(
        &self,
        component_id: &str,
        lines: &mut Vec<Line<'static>>,
    ) -> TuiResult<()> {
        for extension in self.get_component_extensions(component_id) {
            extension.post_render(component_id, lines).await?;
        }
        Ok(())
    }

    /// Execute command pre-execute extensions
    pub async fn execute_command_pre_execute(
        &self,
        command: &str,
        args: &[String],
        model: &AppModel,
    ) -> TuiResult<()> {
        for extension in self.get_command_extensions(command) {
            extension.pre_execute(command, args, model).await?;
        }
        Ok(())
    }

    /// Execute command post-execute extensions
    pub async fn execute_command_post_execute(
        &self,
        command: &str,
        result: &TuiResult<()>,
        model: &AppModel,
    ) -> TuiResult<()> {
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

    async fn initialize<'a>(&mut self, _context: &PluginContext<'a>) -> TuiResult<()> {
        tracing::info!("Initializing placeholder plugin: {}", self.name());
        Ok(())
    }

    async fn handle_message(&mut self, message: &PluginMessage) -> Vec<PluginMessage> {
        tracing::debug!(
            "Placeholder plugin {} received message: {:?}",
            self.name(),
            message
        );
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

/// Theme marketplace for discovering and installing themes
pub struct ThemeMarketplace {
    registry: EnhancedPluginRegistry,
    marketplace_url: Option<String>,
    installed_themes: std::collections::HashMap<String, PluginTheme>,
    available_themes: Vec<MarketplaceTheme>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct MarketplaceTheme {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub download_url: String,
    pub tags: Vec<String>,
    pub preview_image: Option<String>,
    pub rating: Option<f32>,
    pub downloads: Option<u32>,
}

impl ThemeMarketplace {
    /// Create a new theme marketplace
    pub fn new() -> Self {
        Self {
            registry: EnhancedPluginRegistry::new(),
            marketplace_url: Some("https://api.example.com/themes".to_string()),
            installed_themes: std::collections::HashMap::new(),
            available_themes: Vec::new(),
        }
    }

    /// Discover available themes from the marketplace
    pub async fn discover_themes(&mut self) -> TuiResult<()> {
        // In a real implementation, this would fetch from the marketplace API
        // For now, simulate some available themes
        self.available_themes = vec![
            MarketplaceTheme {
                id: "dracula-pro".to_string(),
                name: "Dracula Pro".to_string(),
                description: "Professional Dracula theme with enhanced colors".to_string(),
                author: "Dracula Team".to_string(),
                version: "2.1.0".to_string(),
                download_url: "https://example.com/themes/dracula-pro.zip".to_string(),
                tags: vec!["dark".to_string(), "professional".to_string()],
                preview_image: Some("https://example.com/previews/dracula-pro.png".to_string()),
                rating: Some(4.8),
                downloads: Some(15420),
            },
            MarketplaceTheme {
                id: "nord-extended".to_string(),
                name: "Nord Extended".to_string(),
                description: "Extended Nord theme with additional color variants".to_string(),
                author: "Nord Theme".to_string(),
                version: "1.3.2".to_string(),
                download_url: "https://example.com/themes/nord-extended.zip".to_string(),
                tags: vec![
                    "cold".to_string(),
                    "arctic".to_string(),
                    "extended".to_string(),
                ],
                preview_image: Some("https://example.com/previews/nord-extended.png".to_string()),
                rating: Some(4.6),
                downloads: Some(8920),
            },
        ];

        Ok(())
    }

    /// Install a theme from the marketplace
    pub async fn install_theme(&mut self, theme_id: &str) -> TuiResult<()> {
        let marketplace_theme = self
            .available_themes
            .iter()
            .find(|t| t.id == theme_id)
            .ok_or_else(|| crate::error::TuiError::Plugin {
                message: format!("Theme not found in marketplace: {}", theme_id),
            })?
            .clone();

        // In a real implementation, this would download and extract the theme
        // For now, simulate installation by creating a theme plugin
        let mut theme_plugin = ThemePluginImpl::new(&marketplace_theme.id, &marketplace_theme.name);

        // Create a plugin theme from marketplace data
        let plugin_theme = PluginTheme {
            id: marketplace_theme.id.clone(),
            name: marketplace_theme.name.clone(),
            description: marketplace_theme.description.clone(),
            theme_data: serde_json::json!({
                "name": marketplace_theme.name,
                "author": marketplace_theme.author,
                "version": marketplace_theme.version,
                "description": marketplace_theme.description,
                "tags": marketplace_theme.tags,
                // Add default theme colors (would be in the downloaded theme)
                "colors": {
                    "background": "#282a36",
                    "foreground": "#f8f8f2",
                    "primary": "#bd93f9",
                    "secondary": "#6272a4",
                    "accent": "#ffb86c"
                }
            }),
            author: marketplace_theme.author.clone(),
            version: marketplace_theme.version.clone(),
            tags: marketplace_theme.tags.clone(),
        };

        theme_plugin.add_theme(plugin_theme.clone());

        // Register the theme plugin
        self.registry
            .register_theme_plugin(Box::new(theme_plugin))?;

        // Mark as installed
        self.installed_themes
            .insert(theme_id.to_string(), plugin_theme);

        tracing::info!("Installed theme: {}", marketplace_theme.name);
        Ok(())
    }

    /// Uninstall a theme
    pub async fn uninstall_theme(&mut self, theme_id: &str) -> TuiResult<()> {
        if self.installed_themes.remove(theme_id).is_some() {
            self.registry
                .unregister(&PluginId::from(theme_id.to_string()));
            tracing::info!("Uninstalled theme: {}", theme_id);
            Ok(())
        } else {
            Err(crate::error::TuiError::Plugin {
                message: format!("Theme not installed: {}", theme_id),
            })
        }
    }

    /// Get installed themes
    pub fn installed_themes(&self) -> Vec<&PluginTheme> {
        self.installed_themes.values().collect()
    }

    /// Get available themes from marketplace
    pub fn available_themes(&self) -> &[MarketplaceTheme] {
        &self.available_themes
    }

    /// Search themes by query
    pub fn search_themes(&self, query: &str) -> Vec<&MarketplaceTheme> {
        let query_lower = query.to_lowercase();
        self.available_themes
            .iter()
            .filter(|theme| {
                theme.name.to_lowercase().contains(&query_lower)
                    || theme.description.to_lowercase().contains(&query_lower)
                    || theme
                        .tags
                        .iter()
                        .any(|tag| tag.to_lowercase().contains(&query_lower))
            })
            .collect()
    }

    /// Get theme plugin registry
    pub fn registry(&self) -> &EnhancedPluginRegistry {
        &self.registry
    }

    /// Get theme plugin registry mutably
    pub fn registry_mut(&mut self) -> &mut EnhancedPluginRegistry {
        &mut self.registry
    }
}

impl Default for ThemeMarketplace {
    fn default() -> Self {
        Self::new()
    }
}
