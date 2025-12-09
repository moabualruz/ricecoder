//! Configuration management for IDE integration

use crate::error::{IdeError, IdeResult};
use crate::types::IdeIntegrationConfig;
use ricecoder_storage::PathResolver;
use std::path::PathBuf;
use tracing::{debug, info};

/// Configuration manager for IDE integration
pub struct ConfigManager;

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new() -> Self {
        ConfigManager
    }

    /// Load configuration from a YAML file
    pub async fn load_from_yaml_file(file_path: &str) -> IdeResult<IdeIntegrationConfig> {
        debug!("Loading IDE configuration from YAML: {}", file_path);

        // Expand home directory if needed
        let resolved_path = PathResolver::expand_home(&PathBuf::from(file_path))
            .map_err(|e| IdeError::path_resolution_error(format!("Failed to resolve path: {}", e)))?;

        // Read the file
        let content = tokio::fs::read_to_string(&resolved_path)
            .await
            .map_err(|e| {
                IdeError::config_error(format!(
                    "Failed to read configuration file '{}': {}",
                    resolved_path.display(),
                    e
                ))
            })?;

        // Parse YAML
        let config: IdeIntegrationConfig = serde_yaml::from_str(&content).map_err(|e| {
            IdeError::config_error(format!(
                "Failed to parse YAML configuration: {}. Please check the file format.",
                e
            ))
        })?;

        // Validate configuration
        Self::validate_config(&config)?;

        info!("Successfully loaded IDE configuration from {}", file_path);
        Ok(config)
    }

    /// Load configuration from a JSON file
    pub async fn load_from_json_file(file_path: &str) -> IdeResult<IdeIntegrationConfig> {
        debug!("Loading IDE configuration from JSON: {}", file_path);

        // Expand home directory if needed
        let resolved_path = PathResolver::expand_home(&PathBuf::from(file_path))
            .map_err(|e| IdeError::path_resolution_error(format!("Failed to resolve path: {}", e)))?;

        // Read the file
        let content = tokio::fs::read_to_string(&resolved_path)
            .await
            .map_err(|e| {
                IdeError::config_error(format!(
                    "Failed to read configuration file '{}': {}",
                    resolved_path.display(),
                    e
                ))
            })?;

        // Parse JSON
        let config: IdeIntegrationConfig = serde_json::from_str(&content).map_err(|e| {
            IdeError::config_error(format!(
                "Failed to parse JSON configuration: {}. Please check the file format.",
                e
            ))
        })?;

        // Validate configuration
        Self::validate_config(&config)?;

        info!("Successfully loaded IDE configuration from {}", file_path);
        Ok(config)
    }

    /// Load configuration from a file (auto-detect format)
    pub async fn load_from_file(file_path: &str) -> IdeResult<IdeIntegrationConfig> {
        if file_path.ends_with(".yaml") || file_path.ends_with(".yml") {
            Self::load_from_yaml_file(file_path).await
        } else if file_path.ends_with(".json") {
            Self::load_from_json_file(file_path).await
        } else {
            Err(IdeError::config_error(
                "Unsupported configuration file format. Use .yaml, .yml, or .json",
            ))
        }
    }

    /// Validate configuration
    fn validate_config(config: &IdeIntegrationConfig) -> IdeResult<()> {
        debug!("Validating IDE configuration");

        // Validate provider chain configuration
        if !config.providers.external_lsp.enabled
            && !config
                .providers
                .configured_rules
                .as_ref()
                .map(|c| c.enabled)
                .unwrap_or(false)
            && !config.providers.builtin_providers.enabled
        {
            return Err(IdeError::config_validation_error(
                "At least one provider must be enabled (external_lsp, configured_rules, or builtin_providers). \
                 Please enable at least one provider in your configuration.",
            ));
        }

        // Validate external LSP configuration
        if config.providers.external_lsp.enabled {
            if config.providers.external_lsp.servers.is_empty() {
                return Err(IdeError::config_validation_error(
                    "External LSP is enabled but no servers are configured. \
                     Please add at least one LSP server configuration or disable external_lsp.",
                ));
            }

            for (language, server_config) in &config.providers.external_lsp.servers {
                if server_config.command.is_empty() {
                    return Err(IdeError::config_validation_error(format!(
                        "LSP server for language '{}' has empty command. \
                         Please specify a valid command to start the LSP server.",
                        language
                    )));
                }

                if server_config.timeout_ms == 0 {
                    return Err(IdeError::config_validation_error(format!(
                        "LSP server for language '{}' has invalid timeout (0ms). \
                         Please set a positive timeout value.",
                        language
                    )));
                }
            }
        }

        // Validate configured rules configuration
        if let Some(rules_config) = &config.providers.configured_rules {
            if rules_config.enabled && rules_config.rules_path.is_empty() {
                return Err(IdeError::config_validation_error(
                    "Configured rules are enabled but rules_path is empty. \
                     Please specify a valid path to the rules file or disable configured_rules.",
                ));
            }
        }

        // Validate VS Code configuration
        if let Some(vscode_config) = &config.vscode {
            if vscode_config.enabled && vscode_config.port == 0 {
                return Err(IdeError::config_validation_error(
                    "VS Code integration is enabled but port is 0. \
                     Please specify a valid port number (1-65535).",
                ));
            }
        }

        debug!("Configuration validation passed");
        Ok(())
    }

    /// Get default configuration
    pub fn default_config() -> IdeIntegrationConfig {
        IdeIntegrationConfig {
            vscode: None,
            terminal: None,
            providers: crate::types::ProviderChainConfig {
                external_lsp: crate::types::ExternalLspConfig {
                    enabled: true,
                    servers: Default::default(),
                    health_check_interval_ms: 5000,
                },
                configured_rules: None,
                builtin_providers: crate::types::BuiltinProvidersConfig {
                    enabled: true,
                    languages: vec![
                        "rust".to_string(),
                        "typescript".to_string(),
                        "python".to_string(),
                    ],
                },
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_config_valid() {
        let config = IdeIntegrationConfig {
            vscode: Some(crate::types::VsCodeConfig {
                enabled: true,
                port: 8080,
                features: vec!["completion".to_string()],
                settings: serde_json::json!({}),
            }),
            terminal: None,
            providers: crate::types::ProviderChainConfig {
                external_lsp: crate::types::ExternalLspConfig {
                    enabled: true,
                    servers: {
                        let mut map = std::collections::HashMap::new();
                        map.insert(
                            "rust".to_string(),
                            crate::types::LspServerConfig {
                                language: "rust".to_string(),
                                command: "rust-analyzer".to_string(),
                                args: vec![],
                                timeout_ms: 5000,
                            },
                        );
                        map
                    },
                    health_check_interval_ms: 5000,
                },
                configured_rules: None,
                builtin_providers: crate::types::BuiltinProvidersConfig {
                    enabled: true,
                    languages: vec!["rust".to_string()],
                },
            },
        };

        assert!(ConfigManager::validate_config(&config).is_ok());
    }

    #[test]
    fn test_validate_config_no_providers_enabled() {
        let config = IdeIntegrationConfig {
            vscode: None,
            terminal: None,
            providers: crate::types::ProviderChainConfig {
                external_lsp: crate::types::ExternalLspConfig {
                    enabled: false,
                    servers: Default::default(),
                    health_check_interval_ms: 5000,
                },
                configured_rules: None,
                builtin_providers: crate::types::BuiltinProvidersConfig {
                    enabled: false,
                    languages: vec![],
                },
            },
        };

        let result = ConfigManager::validate_config(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("At least one provider must be enabled"));
    }

    #[test]
    fn test_validate_config_empty_lsp_servers() {
        let config = IdeIntegrationConfig {
            vscode: None,
            terminal: None,
            providers: crate::types::ProviderChainConfig {
                external_lsp: crate::types::ExternalLspConfig {
                    enabled: true,
                    servers: Default::default(),
                    health_check_interval_ms: 5000,
                },
                configured_rules: None,
                builtin_providers: crate::types::BuiltinProvidersConfig {
                    enabled: false,
                    languages: vec![],
                },
            },
        };

        let result = ConfigManager::validate_config(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("no servers are configured"));
    }

    #[test]
    fn test_validate_config_invalid_lsp_command() {
        let config = IdeIntegrationConfig {
            vscode: None,
            terminal: None,
            providers: crate::types::ProviderChainConfig {
                external_lsp: crate::types::ExternalLspConfig {
                    enabled: true,
                    servers: {
                        let mut map = std::collections::HashMap::new();
                        map.insert(
                            "rust".to_string(),
                            crate::types::LspServerConfig {
                                language: "rust".to_string(),
                                command: "".to_string(),
                                args: vec![],
                                timeout_ms: 5000,
                            },
                        );
                        map
                    },
                    health_check_interval_ms: 5000,
                },
                configured_rules: None,
                builtin_providers: crate::types::BuiltinProvidersConfig {
                    enabled: false,
                    languages: vec![],
                },
            },
        };

        let result = ConfigManager::validate_config(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("empty command"));
    }

    #[test]
    fn test_validate_config_invalid_vscode_port() {
        let config = IdeIntegrationConfig {
            vscode: Some(crate::types::VsCodeConfig {
                enabled: true,
                port: 0,
                features: vec![],
                settings: serde_json::json!({}),
            }),
            terminal: None,
            providers: crate::types::ProviderChainConfig {
                external_lsp: crate::types::ExternalLspConfig {
                    enabled: false,
                    servers: Default::default(),
                    health_check_interval_ms: 5000,
                },
                configured_rules: None,
                builtin_providers: crate::types::BuiltinProvidersConfig {
                    enabled: true,
                    languages: vec!["rust".to_string()],
                },
            },
        };

        let result = ConfigManager::validate_config(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("port is 0"));
    }

    #[test]
    fn test_default_config() {
        let config = ConfigManager::default_config();
        assert!(config.providers.external_lsp.enabled);
        assert!(config.providers.builtin_providers.enabled);
        assert_eq!(config.providers.builtin_providers.languages.len(), 3);
    }
}
