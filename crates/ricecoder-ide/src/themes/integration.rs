//! IDE Theme Integration
//!
//! This module provides integration between the IDE and the theme system,
//! including theme initialization, configuration loading, and persistence.

use super::IdeThemeManager;
use crate::error::{IdeError, IdeResult};
use ricecoder_storage::PathResolver;
use tracing::{debug, info};

/// IDE Theme Configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IdeThemeConfig {
    /// Current theme name
    pub current_theme: String,
    /// Enable custom themes
    pub enable_custom_themes: bool,
    /// Custom themes directory
    pub custom_themes_dir: Option<String>,
}

impl Default for IdeThemeConfig {
    fn default() -> Self {
        Self {
            current_theme: "dark".to_string(),
            enable_custom_themes: true,
            custom_themes_dir: None,
        }
    }
}

/// IDE Theme Integration Manager
pub struct IdeThemeIntegration {
    theme_manager: IdeThemeManager,
    config: IdeThemeConfig,
}

impl IdeThemeIntegration {
    /// Create a new IDE theme integration
    pub fn new(config: IdeThemeConfig) -> Self {
        Self {
            theme_manager: IdeThemeManager::new(),
            config,
        }
    }

    /// Initialize theme system on IDE startup
    pub async fn initialize(&mut self) -> IdeResult<()> {
        debug!("Initializing IDE theme system");

        // Load saved theme preference from storage
        self.load_theme_preference().await?;

        // Load custom themes if enabled
        if self.config.enable_custom_themes {
            self.load_custom_themes().await?;
        }

        info!(
            "IDE theme system initialized with theme: {}",
            self.config.current_theme
        );

        Ok(())
    }

    /// Load theme preference from storage
    async fn load_theme_preference(&mut self) -> IdeResult<()> {
        debug!("Loading theme preference from storage");

        // Try to load from storage
        let storage_path = PathResolver::resolve_global_path()
            .map_err(|e| IdeError::config_error(format!("Failed to get storage path: {}", e)))?
            .join("theme.yaml");

        if storage_path.exists() {
            match tokio::fs::read_to_string(&storage_path).await {
                Ok(content) => {
                    match serde_yaml::from_str::<IdeThemeConfig>(&content) {
                        Ok(loaded_config) => {
                            self.config = loaded_config;
                            debug!("Loaded theme preference: {}", self.config.current_theme);
                        }
                        Err(e) => {
                            debug!("Failed to parse theme config: {}", e);
                            // Fall back to default
                        }
                    }
                }
                Err(e) => {
                    debug!("Failed to read theme config: {}", e);
                    // Fall back to default
                }
            }
        }

        // Apply the theme
        self.theme_manager
            .switch_by_name(&self.config.current_theme)
            .map_err(|e| IdeError::config_error(format!("Failed to switch theme: {}", e)))?;

        Ok(())
    }

    /// Load custom themes from directory
    async fn load_custom_themes(&mut self) -> IdeResult<()> {
        debug!("Loading custom themes");

        let custom_themes_dir = if let Some(dir) = &self.config.custom_themes_dir {
            PathResolver::expand_home(&std::path::PathBuf::from(dir))
                .map_err(|e| IdeError::config_error(format!("Failed to resolve path: {}", e)))?
        } else {
            IdeThemeManager::custom_themes_directory()
                .map_err(|e| IdeError::config_error(format!("Failed to get themes dir: {}", e)))?
        };

        if custom_themes_dir.exists() {
            match self
                .theme_manager
                .load_and_register_custom_themes(&custom_themes_dir)
            {
                Ok(themes) => {
                    info!("Loaded {} custom themes", themes.len());
                    debug!("Custom themes: {:?}", themes);
                }
                Err(e) => {
                    debug!("Failed to load custom themes: {}", e);
                    // Continue without custom themes
                }
            }
        }

        Ok(())
    }

    /// Switch to a theme by name
    pub fn switch_theme(&mut self, theme_name: &str) -> IdeResult<()> {
        debug!("Switching to theme: {}", theme_name);

        self.theme_manager
            .switch_by_name(theme_name)
            .map_err(|e| IdeError::config_error(format!("Failed to switch theme: {}", e)))?;

        self.config.current_theme = theme_name.to_string();

        // Save preference
        if let Err(e) = self.save_theme_preference() {
            debug!("Failed to save theme preference: {}", e);
            // Continue anyway
        }

        info!("Switched to theme: {}", theme_name);

        Ok(())
    }

    /// Save theme preference to storage
    fn save_theme_preference(&self) -> IdeResult<()> {
        debug!("Saving theme preference to storage");

        let storage_path = PathResolver::resolve_global_path()
            .map_err(|e| IdeError::config_error(format!("Failed to get storage path: {}", e)))?
            .join("theme.yaml");

        // Create parent directory if needed
        if let Some(parent) = storage_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| IdeError::config_error(format!("Failed to create dir: {}", e)))?;
        }

        let yaml = serde_yaml::to_string(&self.config)
            .map_err(|e| IdeError::config_error(format!("Failed to serialize config: {}", e)))?;

        std::fs::write(&storage_path, yaml)
            .map_err(|e| IdeError::config_error(format!("Failed to write config: {}", e)))?;

        debug!("Theme preference saved to {}", storage_path.display());

        Ok(())
    }

    /// Get the current theme manager
    pub fn theme_manager(&self) -> &IdeThemeManager {
        &self.theme_manager
    }

    /// Get the current theme manager (mutable)
    pub fn theme_manager_mut(&mut self) -> &mut IdeThemeManager {
        &mut self.theme_manager
    }

    /// Get the current theme configuration
    pub fn config(&self) -> &IdeThemeConfig {
        &self.config
    }

    /// Get the current theme configuration (mutable)
    pub fn config_mut(&mut self) -> &mut IdeThemeConfig {
        &mut self.config
    }

    /// Get all available themes
    pub fn available_themes(&self) -> Vec<&'static str> {
        self.theme_manager.available_themes()
    }

    /// Get the current theme name
    pub fn current_theme_name(&self) -> String {
        self.config.current_theme.clone()
    }

    /// List all available themes (built-in and custom)
    pub fn list_all_themes(&self) -> IdeResult<Vec<String>> {
        self.theme_manager
            .list_all_themes()
            .map_err(|e| IdeError::config_error(format!("Failed to list themes: {}", e)))
    }

    /// List all built-in themes
    pub fn list_builtin_themes(&self) -> Vec<String> {
        self.theme_manager.list_builtin_themes()
    }

    /// List all custom themes
    pub fn list_custom_themes(&self) -> IdeResult<Vec<String>> {
        self.theme_manager
            .list_custom_themes()
            .map_err(|e| IdeError::config_error(format!("Failed to list custom themes: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ide_theme_config_default() {
        let config = IdeThemeConfig::default();
        assert_eq!(config.current_theme, "dark");
        assert!(config.enable_custom_themes);
    }

    #[test]
    fn test_ide_theme_integration_creation() {
        let config = IdeThemeConfig::default();
        let integration = IdeThemeIntegration::new(config);
        assert_eq!(integration.current_theme_name(), "dark");
    }

    #[test]
    fn test_ide_theme_integration_available_themes() {
        let config = IdeThemeConfig::default();
        let integration = IdeThemeIntegration::new(config);
        let themes = integration.available_themes();
        assert_eq!(themes.len(), 6);
    }

    #[test]
    fn test_ide_theme_integration_list_builtin_themes() {
        let config = IdeThemeConfig::default();
        let integration = IdeThemeIntegration::new(config);
        let themes = integration.list_builtin_themes();
        assert_eq!(themes.len(), 6);
        assert!(themes.contains(&"dark".to_string()));
        assert!(themes.contains(&"light".to_string()));
    }

    #[tokio::test]
    async fn test_ide_theme_integration_switch_theme() {
        let config = IdeThemeConfig::default();
        let mut integration = IdeThemeIntegration::new(config);

        integration.switch_theme("light").unwrap();
        assert_eq!(integration.current_theme_name(), "light");

        integration.switch_theme("dracula").unwrap();
        assert_eq!(integration.current_theme_name(), "dracula");
    }

    #[tokio::test]
    async fn test_ide_theme_integration_switch_invalid_theme() {
        let config = IdeThemeConfig::default();
        let mut integration = IdeThemeIntegration::new(config);

        let result = integration.switch_theme("invalid");
        assert!(result.is_err());
    }
}
