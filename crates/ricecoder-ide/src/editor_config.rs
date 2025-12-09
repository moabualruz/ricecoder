/// Editor-specific configuration module
/// Handles loading and managing configuration for vim, neovim, and emacs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

/// Editor configuration error
#[derive(Debug, Error)]
pub enum EditorConfigError {
    #[error("Failed to load editor configuration: {0}")]
    LoadError(String),
    
    #[error("Invalid editor configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Unsupported editor: {0}")]
    UnsupportedEditor(String),
    
    #[error("Configuration validation failed: {0}")]
    ValidationError(String),
}

/// Vim/Neovim configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VimConfig {
    /// Enable vim integration
    pub enabled: bool,
    
    /// RiceCoder host
    pub host: String,
    
    /// RiceCoder port
    pub port: u16,
    
    /// Request timeout in milliseconds
    pub timeout_ms: u64,
    
    /// Completion settings
    pub completion: CompletionSettings,
    
    /// Diagnostics settings
    pub diagnostics: DiagnosticsSettings,
    
    /// Hover settings
    pub hover: HoverSettings,
    
    /// Custom keybindings
    pub keybindings: HashMap<String, String>,
}

/// Emacs configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmacsConfig {
    /// Enable emacs integration
    pub enabled: bool,
    
    /// RiceCoder host
    pub host: String,
    
    /// RiceCoder port
    pub port: u16,
    
    /// Request timeout in milliseconds
    pub timeout_ms: u64,
    
    /// Completion settings
    pub completion: CompletionSettings,
    
    /// Diagnostics settings
    pub diagnostics: DiagnosticsSettings,
    
    /// Hover settings
    pub hover: HoverSettings,
    
    /// Custom keybindings
    pub keybindings: HashMap<String, String>,
}

/// Completion settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionSettings {
    /// Enable completion
    pub enabled: bool,
    
    /// Maximum number of completion items
    pub max_items: usize,
    
    /// Trigger completion on these characters
    pub trigger_characters: Vec<String>,
}

/// Diagnostics settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsSettings {
    /// Enable diagnostics
    pub enabled: bool,
    
    /// Show diagnostics on file change
    pub show_on_change: bool,
    
    /// Minimum severity level to display (1=error, 2=warning, 3=info)
    pub min_severity: u8,
}

/// Hover settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoverSettings {
    /// Enable hover
    pub enabled: bool,
    
    /// Show hover on cursor move
    pub show_on_move: bool,
    
    /// Hover delay in milliseconds
    pub delay_ms: u64,
}

/// Terminal editor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalEditorConfig {
    /// Vim/Neovim configuration
    pub vim: Option<VimConfig>,
    
    /// Emacs configuration
    pub emacs: Option<EmacsConfig>,
}

impl Default for VimConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            host: "localhost".to_string(),
            port: 9000,
            timeout_ms: 5000,
            completion: CompletionSettings::default(),
            diagnostics: DiagnosticsSettings::default(),
            hover: HoverSettings::default(),
            keybindings: HashMap::new(),
        }
    }
}

impl Default for EmacsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            host: "localhost".to_string(),
            port: 9000,
            timeout_ms: 5000,
            completion: CompletionSettings::default(),
            diagnostics: DiagnosticsSettings::default(),
            hover: HoverSettings::default(),
            keybindings: HashMap::new(),
        }
    }
}

impl Default for CompletionSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            max_items: 20,
            trigger_characters: vec![".".to_string(), ":".to_string()],
        }
    }
}

impl Default for DiagnosticsSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            show_on_change: true,
            min_severity: 1,
        }
    }
}

impl Default for HoverSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            show_on_move: false,
            delay_ms: 500,
        }
    }
}

impl Default for TerminalEditorConfig {
    fn default() -> Self {
        Self {
            vim: Some(VimConfig::default()),
            emacs: Some(EmacsConfig::default()),
        }
    }
}

impl TerminalEditorConfig {
    /// Load configuration from YAML file
    pub fn from_yaml(path: &Path) -> Result<Self, EditorConfigError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| EditorConfigError::LoadError(e.to_string()))?;
        
        let config: TerminalEditorConfig = serde_yaml::from_str(&content)
            .map_err(|e| EditorConfigError::InvalidConfig(e.to_string()))?;
        
        config.validate()?;
        Ok(config)
    }
    
    /// Load configuration from JSON file
    pub fn from_json(path: &Path) -> Result<Self, EditorConfigError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| EditorConfigError::LoadError(e.to_string()))?;
        
        let config: TerminalEditorConfig = serde_json::from_str(&content)
            .map_err(|e| EditorConfigError::InvalidConfig(e.to_string()))?;
        
        config.validate()?;
        Ok(config)
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<(), EditorConfigError> {
        if let Some(vim_config) = &self.vim {
            vim_config.validate()?;
        }
        
        if let Some(emacs_config) = &self.emacs {
            emacs_config.validate()?;
        }
        
        Ok(())
    }
    
    /// Get vim configuration
    pub fn vim(&self) -> Option<&VimConfig> {
        self.vim.as_ref()
    }
    
    /// Get emacs configuration
    pub fn emacs(&self) -> Option<&EmacsConfig> {
        self.emacs.as_ref()
    }
}

impl VimConfig {
    /// Validate vim configuration
    pub fn validate(&self) -> Result<(), EditorConfigError> {
        if self.port == 0 {
            return Err(EditorConfigError::ValidationError(
                "Port must be greater than 0".to_string(),
            ));
        }
        
        if self.timeout_ms == 0 {
            return Err(EditorConfigError::ValidationError(
                "Timeout must be greater than 0".to_string(),
            ));
        }
        
        if self.completion.max_items == 0 {
            return Err(EditorConfigError::ValidationError(
                "Max completion items must be greater than 0".to_string(),
            ));
        }
        
        Ok(())
    }
}

impl EmacsConfig {
    /// Validate emacs configuration
    pub fn validate(&self) -> Result<(), EditorConfigError> {
        if self.port == 0 {
            return Err(EditorConfigError::ValidationError(
                "Port must be greater than 0".to_string(),
            ));
        }
        
        if self.timeout_ms == 0 {
            return Err(EditorConfigError::ValidationError(
                "Timeout must be greater than 0".to_string(),
            ));
        }
        
        if self.completion.max_items == 0 {
            return Err(EditorConfigError::ValidationError(
                "Max completion items must be greater than 0".to_string(),
            ));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vim_config_default() {
        let config = VimConfig::default();
        assert!(config.enabled);
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 9000);
        assert_eq!(config.timeout_ms, 5000);
    }
    
    #[test]
    fn test_emacs_config_default() {
        let config = EmacsConfig::default();
        assert!(config.enabled);
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 9000);
        assert_eq!(config.timeout_ms, 5000);
    }
    
    #[test]
    fn test_vim_config_validation() {
        let mut config = VimConfig::default();
        assert!(config.validate().is_ok());
        
        config.port = 0;
        assert!(config.validate().is_err());
        
        config.port = 9000;
        config.timeout_ms = 0;
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_emacs_config_validation() {
        let mut config = EmacsConfig::default();
        assert!(config.validate().is_ok());
        
        config.port = 0;
        assert!(config.validate().is_err());
        
        config.port = 9000;
        config.timeout_ms = 0;
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_terminal_editor_config_default() {
        let config = TerminalEditorConfig::default();
        assert!(config.vim.is_some());
        assert!(config.emacs.is_some());
        assert!(config.validate().is_ok());
    }
}
