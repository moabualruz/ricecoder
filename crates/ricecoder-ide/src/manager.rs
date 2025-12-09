//! IDE Integration Manager

use crate::error::{IdeError, IdeResult};
use crate::types::*;
use tracing::{debug, info};

/// IDE Integration Manager
pub struct IdeIntegrationManager {
    config: IdeIntegrationConfig,
}

impl IdeIntegrationManager {
    /// Create a new IDE Integration Manager
    pub fn new(config: IdeIntegrationConfig) -> Self {
        IdeIntegrationManager { config }
    }

    /// Handle completion request
    pub async fn handle_completion_request(
        &self,
        params: &CompletionParams,
    ) -> IdeResult<Vec<CompletionItem>> {
        debug!(
            "Handling completion request for language: {}",
            params.language
        );

        // Validate parameters
        if params.language.is_empty() {
            return Err(IdeError::provider_error(
                "Language parameter is required for completion request",
            ));
        }

        if params.file_path.is_empty() {
            return Err(IdeError::provider_error(
                "File path parameter is required for completion request",
            ));
        }

        // Return empty completions for now (will be implemented with provider chain)
        Ok(vec![])
    }

    /// Handle diagnostics request
    pub async fn handle_diagnostics_request(
        &self,
        params: &DiagnosticsParams,
    ) -> IdeResult<Vec<Diagnostic>> {
        debug!(
            "Handling diagnostics request for language: {}",
            params.language
        );

        // Validate parameters
        if params.language.is_empty() {
            return Err(IdeError::provider_error(
                "Language parameter is required for diagnostics request",
            ));
        }

        if params.file_path.is_empty() {
            return Err(IdeError::provider_error(
                "File path parameter is required for diagnostics request",
            ));
        }

        // Return empty diagnostics for now (will be implemented with provider chain)
        Ok(vec![])
    }

    /// Handle hover request
    pub async fn handle_hover_request(&self, params: &HoverParams) -> IdeResult<Option<Hover>> {
        debug!("Handling hover request for language: {}", params.language);

        // Validate parameters
        if params.language.is_empty() {
            return Err(IdeError::provider_error(
                "Language parameter is required for hover request",
            ));
        }

        if params.file_path.is_empty() {
            return Err(IdeError::provider_error(
                "File path parameter is required for hover request",
            ));
        }

        // Return None for now (will be implemented with provider chain)
        Ok(None)
    }

    /// Handle definition request
    pub async fn handle_definition_request(
        &self,
        params: &DefinitionParams,
    ) -> IdeResult<Option<Location>> {
        debug!(
            "Handling definition request for language: {}",
            params.language
        );

        // Validate parameters
        if params.language.is_empty() {
            return Err(IdeError::provider_error(
                "Language parameter is required for definition request",
            ));
        }

        if params.file_path.is_empty() {
            return Err(IdeError::provider_error(
                "File path parameter is required for definition request",
            ));
        }

        // Return None for now (will be implemented with provider chain)
        Ok(None)
    }

    /// Establish connection with IDE
    pub async fn establish_connection(&self, ide_type: &str) -> IdeResult<()> {
        debug!("Establishing connection with IDE: {}", ide_type);

        match ide_type {
            "vscode" => {
                if let Some(vscode_config) = &self.config.vscode {
                    if !vscode_config.enabled {
                        return Err(IdeError::communication_error(
                            "VS Code integration is not enabled in configuration",
                        ));
                    }
                    info!("VS Code connection established on port {}", vscode_config.port);
                    Ok(())
                } else {
                    Err(IdeError::communication_error(
                        "VS Code configuration not found",
                    ))
                }
            }
            "vim" | "neovim" => {
                if let Some(terminal_config) = &self.config.terminal {
                    if let Some(vim_config) = &terminal_config.vim {
                        if !vim_config.enabled {
                            return Err(IdeError::communication_error(
                                "Vim/Neovim integration is not enabled in configuration",
                            ));
                        }
                        info!("Vim/Neovim connection established");
                        Ok(())
                    } else {
                        Err(IdeError::communication_error(
                            "Vim/Neovim configuration not found",
                        ))
                    }
                } else {
                    Err(IdeError::communication_error(
                        "Terminal configuration not found",
                    ))
                }
            }
            "emacs" => {
                if let Some(terminal_config) = &self.config.terminal {
                    if let Some(emacs_config) = &terminal_config.emacs {
                        if !emacs_config.enabled {
                            return Err(IdeError::communication_error(
                                "Emacs integration is not enabled in configuration",
                            ));
                        }
                        info!("Emacs connection established");
                        Ok(())
                    } else {
                        Err(IdeError::communication_error(
                            "Emacs configuration not found",
                        ))
                    }
                } else {
                    Err(IdeError::communication_error(
                        "Terminal configuration not found",
                    ))
                }
            }
            _ => Err(IdeError::communication_error(format!(
                "Unknown IDE type: {}",
                ide_type
            ))),
        }
    }

    /// Close connection with IDE
    pub async fn close_connection(&self, ide_type: &str) -> IdeResult<()> {
        debug!("Closing connection with IDE: {}", ide_type);
        info!("Connection closed for IDE: {}", ide_type);
        Ok(())
    }

    /// Get configuration
    pub fn config(&self) -> &IdeIntegrationConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> IdeIntegrationConfig {
        IdeIntegrationConfig {
            vscode: Some(VsCodeConfig {
                enabled: true,
                port: 8080,
                features: vec!["completion".to_string()],
                settings: serde_json::json!({}),
            }),
            terminal: Some(TerminalConfig {
                vim: Some(VimConfig {
                    enabled: true,
                    plugin_manager: "vim-plug".to_string(),
                }),
                emacs: Some(EmacsConfig {
                    enabled: true,
                    package_manager: "use-package".to_string(),
                }),
            }),
            providers: ProviderChainConfig {
                external_lsp: crate::types::ExternalLspConfig {
                    enabled: true,
                    servers: Default::default(),
                    health_check_interval_ms: 5000,
                },
                configured_rules: None,
                builtin_providers: crate::types::BuiltinProvidersConfig {
                    enabled: true,
                    languages: vec!["rust".to_string()],
                },
            },
        }
    }

    #[tokio::test]
    async fn test_handle_completion_request_valid() {
        let config = create_test_config();
        let manager = IdeIntegrationManager::new(config);

        let params = CompletionParams {
            language: "rust".to_string(),
            file_path: "src/main.rs".to_string(),
            position: Position {
                line: 10,
                character: 5,
            },
            context: "fn test".to_string(),
        };

        let result = manager.handle_completion_request(&params).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_completion_request_empty_language() {
        let config = create_test_config();
        let manager = IdeIntegrationManager::new(config);

        let params = CompletionParams {
            language: "".to_string(),
            file_path: "src/main.rs".to_string(),
            position: Position {
                line: 10,
                character: 5,
            },
            context: "fn test".to_string(),
        };

        let result = manager.handle_completion_request(&params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_completion_request_empty_file_path() {
        let config = create_test_config();
        let manager = IdeIntegrationManager::new(config);

        let params = CompletionParams {
            language: "rust".to_string(),
            file_path: "".to_string(),
            position: Position {
                line: 10,
                character: 5,
            },
            context: "fn test".to_string(),
        };

        let result = manager.handle_completion_request(&params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_establish_vscode_connection() {
        let config = create_test_config();
        let manager = IdeIntegrationManager::new(config);

        let result = manager.establish_connection("vscode").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_establish_vim_connection() {
        let config = create_test_config();
        let manager = IdeIntegrationManager::new(config);

        let result = manager.establish_connection("vim").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_establish_emacs_connection() {
        let config = create_test_config();
        let manager = IdeIntegrationManager::new(config);

        let result = manager.establish_connection("emacs").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_establish_unknown_ide_connection() {
        let config = create_test_config();
        let manager = IdeIntegrationManager::new(config);

        let result = manager.establish_connection("unknown").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_close_connection() {
        let config = create_test_config();
        let manager = IdeIntegrationManager::new(config);

        let result = manager.close_connection("vscode").await;
        assert!(result.is_ok());
    }
}
