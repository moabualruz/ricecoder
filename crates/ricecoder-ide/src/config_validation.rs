//! Configuration validation for IDE integration
//!
//! This module provides comprehensive configuration validation with clear error messages
//! and remediation steps for IDE integration configuration.

use crate::error::{IdeError, IdeResult};
use crate::types::*;
use tracing::debug;

/// Configuration validator for IDE integration
pub struct ConfigValidator;

impl ConfigValidator {
    /// Validate complete IDE integration configuration
    pub fn validate_complete(config: &IdeIntegrationConfig) -> IdeResult<()> {
        debug!("Validating complete IDE configuration");

        // Validate provider chain
        Self::validate_provider_chain(&config.providers)?;

        // Validate IDE-specific configurations
        if let Some(vscode_config) = &config.vscode {
            Self::validate_vscode_config(vscode_config)?;
        }

        if let Some(terminal_config) = &config.terminal {
            Self::validate_terminal_config(terminal_config)?;
        }

        debug!("Configuration validation passed");
        Ok(())
    }

    /// Validate provider chain configuration
    fn validate_provider_chain(providers: &ProviderChainConfig) -> IdeResult<()> {
        debug!("Validating provider chain configuration");

        // Check that at least one provider is enabled
        let any_enabled = providers.external_lsp.enabled
            || providers
                .configured_rules
                .as_ref()
                .map(|c| c.enabled)
                .unwrap_or(false)
            || providers.builtin_providers.enabled;

        if !any_enabled {
            return Err(IdeError::config_validation_error(
                "Configuration validation failed: At least one provider must be enabled.\n\
                 \n\
                 Remediation steps:\n\
                 1. Enable at least one of: external_lsp, configured_rules, or builtin_providers\n\
                 2. Example configuration:\n\
                    providers:\n\
                      external_lsp:\n\
                        enabled: true\n\
                      builtin_providers:\n\
                        enabled: true\n\
                 3. For more details, see: https://ricecoder.dev/docs/ide-integration/configuration",
            ));
        }

        // Validate external LSP configuration
        Self::validate_external_lsp(&providers.external_lsp)?;

        // Validate configured rules configuration
        if let Some(rules_config) = &providers.configured_rules {
            Self::validate_configured_rules(rules_config)?;
        }

        // Validate built-in providers configuration
        Self::validate_builtin_providers(&providers.builtin_providers)?;

        Ok(())
    }

    /// Validate external LSP configuration
    fn validate_external_lsp(config: &ExternalLspConfig) -> IdeResult<()> {
        if !config.enabled {
            return Ok(());
        }

        debug!("Validating external LSP configuration");

        if config.servers.is_empty() {
            return Err(IdeError::config_validation_error(
                "Configuration validation failed: External LSP is enabled but no servers are configured.\n\
                 \n\
                 Remediation steps:\n\
                 1. Add at least one LSP server configuration, or\n\
                 2. Disable external_lsp if you don't want to use external LSP servers\n\
                 \n\
                 Example configuration:\n\
                 providers:\n\
                   external_lsp:\n\
                     enabled: true\n\
                     servers:\n\
                       rust:\n\
                         language: rust\n\
                         command: rust-analyzer\n\
                         args: []\n\
                         timeout_ms: 5000\n\
                 \n\
                 For more details, see: https://ricecoder.dev/docs/ide-integration/lsp-configuration",
            ));
        }

        // Validate each server configuration
        for (language, server_config) in &config.servers {
            Self::validate_lsp_server_config(language, server_config)?;
        }

        // Validate health check interval
        if config.health_check_interval_ms == 0 {
            return Err(IdeError::config_validation_error(
                "Configuration validation failed: health_check_interval_ms must be greater than 0.\n\
                 \n\
                 Remediation steps:\n\
                 1. Set health_check_interval_ms to a positive value (e.g., 5000 for 5 seconds)\n\
                 2. Example configuration:\n\
                    providers:\n\
                      external_lsp:\n\
                        health_check_interval_ms: 5000",
            ));
        }

        Ok(())
    }

    /// Validate individual LSP server configuration
    fn validate_lsp_server_config(language: &str, config: &LspServerConfig) -> IdeResult<()> {
        debug!("Validating LSP server configuration for language: {}", language);

        // Validate language field
        if config.language.is_empty() {
            return Err(IdeError::config_validation_error(format!(
                "Configuration validation failed: LSP server for '{}' has empty language field.\n\
                 \n\
                 Remediation steps:\n\
                 1. Ensure the language field matches the key (e.g., 'rust' for rust-analyzer)\n\
                 2. Example configuration:\n\
                    servers:\n\
                      rust:\n\
                        language: rust\n\
                        command: rust-analyzer",
                language
            )));
        }

        // Validate command field
        if config.command.is_empty() {
            return Err(IdeError::config_validation_error(format!(
                "Configuration validation failed: LSP server for '{}' has empty command.\n\
                 \n\
                 Remediation steps:\n\
                 1. Specify the command to start the LSP server\n\
                 2. Common LSP servers:\n\
                    - Rust: rust-analyzer\n\
                    - TypeScript: typescript-language-server\n\
                    - Python: pylsp\n\
                 3. Example configuration:\n\
                    servers:\n\
                      {}:\n\
                        command: <lsp-server-command>\n\
                 \n\
                 For more details, see: https://ricecoder.dev/docs/ide-integration/lsp-servers",
                language, language
            )));
        }

        // Validate timeout
        if config.timeout_ms == 0 {
            return Err(IdeError::config_validation_error(format!(
                "Configuration validation failed: LSP server for '{}' has invalid timeout (0ms).\n\
                 \n\
                 Remediation steps:\n\
                 1. Set timeout_ms to a positive value (e.g., 5000 for 5 seconds)\n\
                 2. Recommended values:\n\
                    - Fast operations: 1000-2000ms\n\
                    - Normal operations: 5000-10000ms\n\
                    - Slow operations: 15000-30000ms\n\
                 3. Example configuration:\n\
                    servers:\n\
                      {}:\n\
                        timeout_ms: 5000",
                language, language
            )));
        }

        if config.timeout_ms > 120000 {
            return Err(IdeError::config_validation_error(format!(
                "Configuration validation failed: LSP server for '{}' has excessive timeout ({}ms > 120000ms).\n\
                 \n\
                 Remediation steps:\n\
                 1. Reduce timeout_ms to a reasonable value (max 120000ms = 2 minutes)\n\
                 2. Example configuration:\n\
                    servers:\n\
                      {}:\n\
                        timeout_ms: 30000",
                language, config.timeout_ms, language
            )));
        }

        Ok(())
    }

    /// Validate configured rules configuration
    fn validate_configured_rules(config: &ConfiguredRulesConfig) -> IdeResult<()> {
        if !config.enabled {
            return Ok(());
        }

        debug!("Validating configured rules configuration");

        if config.rules_path.is_empty() {
            return Err(IdeError::config_validation_error(
                "Configuration validation failed: Configured rules are enabled but rules_path is empty.\n\
                 \n\
                 Remediation steps:\n\
                 1. Specify a valid path to the rules file, or\n\
                 2. Disable configured_rules if you don't want to use custom rules\n\
                 \n\
                 Example configuration:\n\
                 providers:\n\
                   configured_rules:\n\
                     enabled: true\n\
                     rules_path: config/ide-rules.yaml\n\
                 \n\
                 For more details, see: https://ricecoder.dev/docs/ide-integration/custom-rules",
            ));
        }

        Ok(())
    }

    /// Validate built-in providers configuration
    fn validate_builtin_providers(config: &BuiltinProvidersConfig) -> IdeResult<()> {
        if !config.enabled {
            return Ok(());
        }

        debug!("Validating built-in providers configuration");

        if config.languages.is_empty() {
            return Err(IdeError::config_validation_error(
                "Configuration validation failed: Built-in providers are enabled but no languages are configured.\n\
                 \n\
                 Remediation steps:\n\
                 1. Add at least one language to the languages list, or\n\
                 2. Disable builtin_providers if you don't want to use built-in providers\n\
                 \n\
                 Supported languages:\n\
                 - rust\n\
                 - typescript\n\
                 - python\n\
                 \n\
                 Example configuration:\n\
                 providers:\n\
                   builtin_providers:\n\
                     enabled: true\n\
                     languages:\n\
                       - rust\n\
                       - typescript\n\
                       - python",
            ));
        }

        // Validate language names
        let valid_languages = ["rust", "typescript", "python"];
        for lang in &config.languages {
            if !valid_languages.contains(&lang.as_str()) {
                return Err(IdeError::config_validation_error(format!(
                    "Configuration validation failed: Unknown language '{}' in builtin_providers.\n\
                     \n\
                     Supported languages:\n\
                     - rust\n\
                     - typescript\n\
                     - python\n\
                     \n\
                     Remediation steps:\n\
                     1. Use only supported language names\n\
                     2. Example configuration:\n\
                        providers:\n\
                          builtin_providers:\n\
                            languages:\n\
                              - rust\n\
                              - typescript",
                    lang
                )));
            }
        }

        Ok(())
    }

    /// Validate VS Code configuration
    fn validate_vscode_config(config: &VsCodeConfig) -> IdeResult<()> {
        if !config.enabled {
            return Ok(());
        }

        debug!("Validating VS Code configuration");

        // Validate port
        if config.port == 0 {
            return Err(IdeError::config_validation_error(
                "Configuration validation failed: VS Code integration is enabled but port is 0.\n\
                 \n\
                 Remediation steps:\n\
                 1. Specify a valid port number (1-65535)\n\
                 2. Recommended ports:\n\
                    - Development: 8000-9000\n\
                    - Production: 3000-5000\n\
                 3. Example configuration:\n\
                    vscode:\n\
                      enabled: true\n\
                      port: 8080",
            ));
        }

        // Note: u16 max is 65535, so this check is not needed but kept for clarity
        // if config.port > 65535 {
        //     return Err(...);
        // }

        // Validate features
        if config.features.is_empty() {
            return Err(IdeError::config_validation_error(
                "Configuration validation failed: VS Code integration is enabled but no features are configured.\n\
                 \n\
                 Remediation steps:\n\
                 1. Add at least one feature to the features list, or\n\
                 2. Disable VS Code integration if you don't want to use it\n\
                 \n\
                 Available features:\n\
                 - completion\n\
                 - diagnostics\n\
                 - hover\n\
                 - definition\n\
                 \n\
                 Example configuration:\n\
                 vscode:\n\
                   enabled: true\n\
                   features:\n\
                     - completion\n\
                     - diagnostics\n\
                     - hover",
            ));
        }

        // Validate feature names
        let valid_features = ["completion", "diagnostics", "hover", "definition"];
        for feature in &config.features {
            if !valid_features.contains(&feature.as_str()) {
                return Err(IdeError::config_validation_error(format!(
                    "Configuration validation failed: Unknown feature '{}' in VS Code configuration.\n\
                     \n\
                     Available features:\n\
                     - completion\n\
                     - diagnostics\n\
                     - hover\n\
                     - definition\n\
                     \n\
                     Remediation steps:\n\
                     1. Use only supported feature names\n\
                     2. Example configuration:\n\
                        vscode:\n\
                          features:\n\
                            - completion\n\
                            - diagnostics",
                    feature
                )));
            }
        }

        Ok(())
    }

    /// Validate terminal editor configuration
    fn validate_terminal_config(config: &TerminalConfig) -> IdeResult<()> {
        debug!("Validating terminal editor configuration");

        // At least one editor should be configured
        let any_enabled = config
            .vim
            .as_ref()
            .map(|c| c.enabled)
            .unwrap_or(false)
            || config
                .emacs
                .as_ref()
                .map(|c| c.enabled)
                .unwrap_or(false);

        if !any_enabled {
            return Ok(()); // Terminal config is optional
        }

        // Validate individual editor configurations
        if let Some(vim_config) = &config.vim {
            if vim_config.enabled && vim_config.plugin_manager.is_empty() {
                return Err(IdeError::config_validation_error(
                    "Configuration validation failed: Vim/Neovim integration is enabled but plugin_manager is empty.\n\
                     \n\
                     Remediation steps:\n\
                     1. Specify a valid plugin manager\n\
                     2. Supported plugin managers:\n\
                        - vim-plug\n\
                        - vundle\n\
                        - pathogen\n\
                        - packer (for neovim)\n\
                        - lazy.nvim (for neovim)\n\
                     3. Example configuration:\n\
                        terminal:\n\
                          vim:\n\
                            enabled: true\n\
                            plugin_manager: vim-plug",
                ));
            }
        }

        if let Some(emacs_config) = &config.emacs {
            if emacs_config.enabled && emacs_config.package_manager.is_empty() {
                return Err(IdeError::config_validation_error(
                    "Configuration validation failed: Emacs integration is enabled but package_manager is empty.\n\
                     \n\
                     Remediation steps:\n\
                     1. Specify a valid package manager\n\
                     2. Supported package managers:\n\
                        - use-package\n\
                        - straight.el\n\
                        - quelpa\n\
                     3. Example configuration:\n\
                        terminal:\n\
                          emacs:\n\
                            enabled: true\n\
                            package_manager: use-package",
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_valid_config() -> IdeIntegrationConfig {
        IdeIntegrationConfig {
            vscode: Some(VsCodeConfig {
                enabled: true,
                port: 8080,
                features: vec!["completion".to_string()],
                settings: serde_json::json!({}),
            }),
            terminal: None,
            providers: ProviderChainConfig {
                external_lsp: ExternalLspConfig {
                    enabled: true,
                    servers: {
                        let mut map = HashMap::new();
                        map.insert(
                            "rust".to_string(),
                            LspServerConfig {
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
                builtin_providers: BuiltinProvidersConfig {
                    enabled: true,
                    languages: vec!["rust".to_string()],
                },
            },
        }
    }

    #[test]
    fn test_validate_complete_valid_config() {
        let config = create_valid_config();
        assert!(ConfigValidator::validate_complete(&config).is_ok());
    }

    #[test]
    fn test_validate_no_providers_enabled() {
        let mut config = create_valid_config();
        config.providers.external_lsp.enabled = false;
        config.providers.builtin_providers.enabled = false;

        let result = ConfigValidator::validate_complete(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("At least one provider must be enabled"));
    }

    #[test]
    fn test_validate_empty_lsp_servers() {
        let mut config = create_valid_config();
        config.providers.external_lsp.servers.clear();

        let result = ConfigValidator::validate_complete(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("no servers are configured"));
    }

    #[test]
    fn test_validate_invalid_lsp_command() {
        let mut config = create_valid_config();
        config
            .providers
            .external_lsp
            .servers
            .get_mut("rust")
            .unwrap()
            .command = String::new();

        let result = ConfigValidator::validate_complete(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("empty command"));
    }

    #[test]
    fn test_validate_invalid_lsp_timeout() {
        let mut config = create_valid_config();
        config
            .providers
            .external_lsp
            .servers
            .get_mut("rust")
            .unwrap()
            .timeout_ms = 0;

        let result = ConfigValidator::validate_complete(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("invalid timeout"));
    }

    #[test]
    fn test_validate_excessive_lsp_timeout() {
        let mut config = create_valid_config();
        config
            .providers
            .external_lsp
            .servers
            .get_mut("rust")
            .unwrap()
            .timeout_ms = 200000;

        let result = ConfigValidator::validate_complete(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("excessive timeout"));
    }

    #[test]
    fn test_validate_invalid_vscode_port() {
        let mut config = create_valid_config();
        config.vscode.as_mut().unwrap().port = 0;

        let result = ConfigValidator::validate_complete(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("port is 0"));
    }

    #[test]
    fn test_validate_empty_vscode_features() {
        let mut config = create_valid_config();
        config.vscode.as_mut().unwrap().features.clear();

        let result = ConfigValidator::validate_complete(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("no features are configured"));
    }

    #[test]
    fn test_validate_invalid_vscode_feature() {
        let mut config = create_valid_config();
        config.vscode.as_mut().unwrap().features = vec!["invalid_feature".to_string()];

        let result = ConfigValidator::validate_complete(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown feature"));
    }

    #[test]
    fn test_validate_empty_builtin_languages() {
        let mut config = create_valid_config();
        config.providers.builtin_providers.languages.clear();

        let result = ConfigValidator::validate_complete(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("no languages are configured"));
    }

    #[test]
    fn test_validate_invalid_builtin_language() {
        let mut config = create_valid_config();
        config.providers.builtin_providers.languages = vec!["invalid_lang".to_string()];

        let result = ConfigValidator::validate_complete(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown language"));
    }

    #[test]
    fn test_validate_vim_empty_plugin_manager() {
        let mut config = create_valid_config();
        config.terminal = Some(TerminalConfig {
            vim: Some(VimConfig {
                enabled: true,
                plugin_manager: String::new(),
            }),
            emacs: None,
        });

        let result = ConfigValidator::validate_complete(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("plugin_manager is empty"));
    }

    #[test]
    fn test_validate_vim_neovim_empty_plugin_manager() {
        let mut config = create_valid_config();
        config.terminal = Some(TerminalConfig {
            vim: Some(VimConfig {
                enabled: true,
                plugin_manager: String::new(),
            }),
            emacs: None,
        });

        let result = ConfigValidator::validate_complete(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("plugin_manager is empty"));
    }

    #[test]
    fn test_validate_emacs_empty_package_manager() {
        let mut config = create_valid_config();
        config.terminal = Some(TerminalConfig {
            vim: None,
            emacs: Some(EmacsConfig {
                enabled: true,
                package_manager: String::new(),
            }),
        });

        let result = ConfigValidator::validate_complete(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("package_manager is empty"));
    }
}
