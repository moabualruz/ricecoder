//! IDE-specific configuration applicator
//! Applies IDE-specific settings and behavior based on IDE type

use crate::error::{IdeError, IdeResult};
use crate::types::*;
use std::collections::HashMap;
use tracing::{debug, info};

/// IDE type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdeType {
    /// VS Code
    VsCode,
    /// Vim/Neovim
    Vim,
    /// Emacs
    Emacs,
    /// Unknown IDE
    Unknown,
}

impl IdeType {
    /// Parse IDE type from string
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "vscode" | "vs-code" | "vs_code" => IdeType::VsCode,
            "vim" | "neovim" | "nvim" => IdeType::Vim,
            "emacs" => IdeType::Emacs,
            _ => IdeType::Unknown,
        }
    }

    /// Get string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            IdeType::VsCode => "vscode",
            IdeType::Vim => "vim",
            IdeType::Emacs => "emacs",
            IdeType::Unknown => "unknown",
        }
    }
}

/// IDE-specific settings
#[derive(Debug, Clone)]
pub struct IdeSpecificSettings {
    /// IDE type
    pub ide_type: IdeType,
    /// Enabled features
    pub enabled_features: Vec<String>,
    /// Custom settings
    pub custom_settings: HashMap<String, serde_json::Value>,
    /// Timeout in milliseconds
    pub timeout_ms: u64,
    /// Port for communication
    pub port: u16,
}

impl IdeSpecificSettings {
    /// Create new IDE-specific settings
    pub fn new(ide_type: IdeType) -> Self {
        IdeSpecificSettings {
            ide_type,
            enabled_features: Vec::new(),
            custom_settings: HashMap::new(),
            timeout_ms: 5000,
            port: 0,
        }
    }

    /// Add enabled feature
    pub fn with_feature(mut self, feature: String) -> Self {
        self.enabled_features.push(feature);
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// Set port
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Add custom setting
    pub fn with_setting(mut self, key: String, value: serde_json::Value) -> Self {
        self.custom_settings.insert(key, value);
        self
    }

    /// Check if feature is enabled
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        self.enabled_features.iter().any(|f| f == feature)
    }

    /// Get custom setting
    pub fn get_setting(&self, key: &str) -> Option<&serde_json::Value> {
        self.custom_settings.get(key)
    }
}

/// IDE configuration applicator
pub struct IdeConfigApplicator;

impl IdeConfigApplicator {
    /// Extract IDE-specific settings from configuration
    pub fn extract_settings(
        config: &IdeIntegrationConfig,
        ide_type: IdeType,
    ) -> IdeResult<IdeSpecificSettings> {
        debug!("Extracting IDE-specific settings for: {}", ide_type.as_str());

        match ide_type {
            IdeType::VsCode => Self::extract_vscode_settings(config),
            IdeType::Vim => Self::extract_vim_settings(config),
            IdeType::Emacs => Self::extract_emacs_settings(config),
            IdeType::Unknown => Err(IdeError::config_error(
                "Unknown IDE type. Supported IDEs: vscode, vim, emacs",
            )),
        }
    }

    /// Extract VS Code specific settings
    fn extract_vscode_settings(config: &IdeIntegrationConfig) -> IdeResult<IdeSpecificSettings> {
        debug!("Extracting VS Code specific settings");

        let vscode_config = config
            .vscode
            .as_ref()
            .ok_or_else(|| IdeError::config_error("VS Code configuration not found"))?;

        if !vscode_config.enabled {
            return Err(IdeError::config_error("VS Code integration is disabled"));
        }

        let mut settings = IdeSpecificSettings::new(IdeType::VsCode)
            .with_port(vscode_config.port)
            .with_timeout(5000); // Default timeout for VS Code

        // Add enabled features
        for feature in &vscode_config.features {
            settings = settings.with_feature(feature.clone());
        }

        // Add custom settings from VS Code settings
        if let Some(settings_obj) = vscode_config.settings.as_object() {
            for (key, value) in settings_obj {
                settings = settings.with_setting(key.clone(), value.clone());
            }
        }

        info!("Extracted VS Code settings: {} features enabled", settings.enabled_features.len());
        Ok(settings)
    }

    /// Extract Vim specific settings
    fn extract_vim_settings(config: &IdeIntegrationConfig) -> IdeResult<IdeSpecificSettings> {
        debug!("Extracting Vim specific settings");

        let terminal_config = config
            .terminal
            .as_ref()
            .ok_or_else(|| IdeError::config_error("Terminal configuration not found"))?;

        let vim_config = terminal_config
            .vim
            .as_ref()
            .ok_or_else(|| IdeError::config_error("Vim configuration not found"))?;

        if !vim_config.enabled {
            return Err(IdeError::config_error("Vim integration is disabled"));
        }

        let mut settings = IdeSpecificSettings::new(IdeType::Vim)
            .with_port(9000) // Default port for Vim
            .with_timeout(5000); // Default timeout

        // Add default features for Vim
        settings = settings
            .with_feature("completion".to_string())
            .with_feature("diagnostics".to_string())
            .with_feature("hover".to_string());

        // Add plugin manager setting
        settings = settings.with_setting(
            "plugin_manager".to_string(),
            serde_json::json!(vim_config.plugin_manager),
        );

        info!("Extracted Vim settings: {} features enabled", settings.enabled_features.len());
        Ok(settings)
    }

    /// Extract Emacs specific settings
    fn extract_emacs_settings(config: &IdeIntegrationConfig) -> IdeResult<IdeSpecificSettings> {
        debug!("Extracting Emacs specific settings");

        let terminal_config = config
            .terminal
            .as_ref()
            .ok_or_else(|| IdeError::config_error("Terminal configuration not found"))?;

        let emacs_config = terminal_config
            .emacs
            .as_ref()
            .ok_or_else(|| IdeError::config_error("Emacs configuration not found"))?;

        if !emacs_config.enabled {
            return Err(IdeError::config_error("Emacs integration is disabled"));
        }

        let mut settings = IdeSpecificSettings::new(IdeType::Emacs)
            .with_port(9000) // Default port for Emacs
            .with_timeout(5000); // Default timeout

        // Add default features for Emacs
        settings = settings
            .with_feature("completion".to_string())
            .with_feature("diagnostics".to_string())
            .with_feature("hover".to_string());

        // Add package manager setting
        settings = settings.with_setting(
            "package_manager".to_string(),
            serde_json::json!(emacs_config.package_manager),
        );

        info!("Extracted Emacs settings: {} features enabled", settings.enabled_features.len());
        Ok(settings)
    }

    /// Apply IDE-specific behavior to completion items
    pub fn apply_completion_behavior(
        items: &mut Vec<CompletionItem>,
        settings: &IdeSpecificSettings,
    ) {
        debug!("Applying IDE-specific completion behavior for: {}", settings.ide_type.as_str());

        // Limit completion items based on IDE settings
        if let Some(max_items_value) = settings.get_setting("max_completion_items") {
            if let Some(max_items) = max_items_value.as_u64() {
                let max_items = max_items as usize;
                if items.len() > max_items {
                    items.truncate(max_items);
                    debug!("Truncated completion items to {} for IDE", max_items);
                }
            }
        }

        // Apply IDE-specific formatting
        match settings.ide_type {
            IdeType::VsCode => {
                // VS Code prefers detailed documentation
                for item in items.iter_mut() {
                    if item.documentation.is_none() {
                        if let Some(detail) = &item.detail {
                            if !detail.is_empty() {
                                item.documentation = Some(detail.clone());
                            }
                        }
                    }
                }
            }
            IdeType::Vim => {
                // Vim prefers concise labels
                for item in items.iter_mut() {
                    if item.label.len() > 50 {
                        item.label.truncate(50);
                        item.label.push_str("...");
                    }
                }
            }
            IdeType::Emacs => {
                // Emacs prefers detailed information
                for item in items.iter_mut() {
                    if item.documentation.is_none() {
                        if let Some(detail) = &item.detail {
                            if !detail.is_empty() {
                                item.documentation = Some(detail.clone());
                            }
                        }
                    }
                }
            }
            IdeType::Unknown => {}
        }
    }

    /// Apply IDE-specific behavior to diagnostics
    pub fn apply_diagnostics_behavior(
        diagnostics: &mut Vec<Diagnostic>,
        settings: &IdeSpecificSettings,
    ) {
        debug!("Applying IDE-specific diagnostics behavior for: {}", settings.ide_type.as_str());

        // Filter diagnostics based on minimum severity
        if let Some(min_severity_value) = settings.get_setting("min_severity") {
            if let Some(min_severity) = min_severity_value.as_u64() {
                diagnostics.retain(|d| {
                    let severity = match d.severity {
                        DiagnosticSeverity::Error => 1,
                        DiagnosticSeverity::Warning => 2,
                        DiagnosticSeverity::Information => 3,
                        DiagnosticSeverity::Hint => 4,
                    };
                    severity <= min_severity as u8
                });
                debug!("Filtered diagnostics by severity: {}", min_severity);
            }
        }
    }

    /// Apply IDE-specific behavior to hover information
    pub fn apply_hover_behavior(
        hover: &mut Option<Hover>,
        settings: &IdeSpecificSettings,
    ) {
        debug!("Applying IDE-specific hover behavior for: {}", settings.ide_type.as_str());

        if let Some(hover_info) = hover {
            match settings.ide_type {
                IdeType::VsCode => {
                    // VS Code supports markdown in hover
                    // Keep content as-is
                }
                IdeType::Vim => {
                    // Vim prefers plain text, strip markdown
                    hover_info.contents = Self::strip_markdown(&hover_info.contents);
                }
                IdeType::Emacs => {
                    // Emacs supports some markup
                    // Keep content as-is
                }
                IdeType::Unknown => {}
            }
        }
    }

    /// Strip markdown formatting from text
    fn strip_markdown(text: &str) -> String {
        // Simple markdown stripping
        text.lines()
            .map(|line| {
                // Remove markdown headers
                let line = line.trim_start_matches('#').trim();
                // Remove markdown bold/italic
                let line = line.replace("**", "").replace("__", "").replace("*", "").replace("_", "");
                // Remove markdown code blocks
                
                line.replace("`", "")
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Validate IDE-specific settings
    pub fn validate_settings(settings: &IdeSpecificSettings) -> IdeResult<()> {
        debug!("Validating IDE-specific settings for: {}", settings.ide_type.as_str());

        if settings.port == 0 && settings.ide_type != IdeType::Unknown {
            return Err(IdeError::config_error(
                "IDE port must be greater than 0",
            ));
        }

        if settings.timeout_ms == 0 {
            return Err(IdeError::config_error(
                "IDE timeout must be greater than 0",
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ide_type_from_str() {
        assert_eq!(IdeType::parse("vscode"), IdeType::VsCode);
        assert_eq!(IdeType::parse("vs-code"), IdeType::VsCode);
        assert_eq!(IdeType::parse("vim"), IdeType::Vim);
        assert_eq!(IdeType::parse("neovim"), IdeType::Vim);
        assert_eq!(IdeType::parse("emacs"), IdeType::Emacs);
        assert_eq!(IdeType::parse("unknown"), IdeType::Unknown);
    }

    #[test]
    fn test_ide_type_as_str() {
        assert_eq!(IdeType::VsCode.as_str(), "vscode");
        assert_eq!(IdeType::Vim.as_str(), "vim");
        assert_eq!(IdeType::Emacs.as_str(), "emacs");
        assert_eq!(IdeType::Unknown.as_str(), "unknown");
    }

    #[test]
    fn test_ide_specific_settings_creation() {
        let settings = IdeSpecificSettings::new(IdeType::VsCode);
        assert_eq!(settings.ide_type, IdeType::VsCode);
        assert!(settings.enabled_features.is_empty());
        assert_eq!(settings.timeout_ms, 5000);
    }

    #[test]
    fn test_ide_specific_settings_builder() {
        let settings = IdeSpecificSettings::new(IdeType::Vim)
            .with_feature("completion".to_string())
            .with_feature("diagnostics".to_string())
            .with_timeout(10000)
            .with_port(9000);

        assert_eq!(settings.enabled_features.len(), 2);
        assert!(settings.is_feature_enabled("completion"));
        assert!(settings.is_feature_enabled("diagnostics"));
        assert_eq!(settings.timeout_ms, 10000);
        assert_eq!(settings.port, 9000);
    }

    #[test]
    fn test_ide_specific_settings_custom_settings() {
        let settings = IdeSpecificSettings::new(IdeType::Emacs)
            .with_setting("key1".to_string(), serde_json::json!("value1"))
            .with_setting("key2".to_string(), serde_json::json!(42));

        assert_eq!(settings.get_setting("key1").unwrap().as_str().unwrap(), "value1");
        assert_eq!(settings.get_setting("key2").unwrap().as_u64().unwrap(), 42);
        assert!(settings.get_setting("key3").is_none());
    }

    #[test]
    fn test_strip_markdown() {
        let text = "# Header\n**bold** and *italic* and `code`";
        let stripped = IdeConfigApplicator::strip_markdown(text);
        assert!(!stripped.contains("**"));
        assert!(!stripped.contains("*"));
        assert!(!stripped.contains("`"));
    }

    #[test]
    fn test_validate_settings_valid() {
        let settings = IdeSpecificSettings::new(IdeType::VsCode)
            .with_port(8080)
            .with_timeout(5000);

        assert!(IdeConfigApplicator::validate_settings(&settings).is_ok());
    }

    #[test]
    fn test_validate_settings_invalid_port() {
        let settings = IdeSpecificSettings::new(IdeType::VsCode)
            .with_port(0)
            .with_timeout(5000);

        assert!(IdeConfigApplicator::validate_settings(&settings).is_err());
    }

    #[test]
    fn test_validate_settings_invalid_timeout() {
        let settings = IdeSpecificSettings::new(IdeType::Vim)
            .with_port(9000)
            .with_timeout(0);

        assert!(IdeConfigApplicator::validate_settings(&settings).is_err());
    }
}
