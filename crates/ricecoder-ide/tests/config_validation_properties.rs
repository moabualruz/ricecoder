//! Property-based tests for configuration validation
//!
//! **Feature: ricecoder-ide, Property 4: Configuration Validation**
//! **Validates: Requirements 2.8**

use ricecoder_ide::*;
use std::collections::HashMap;

#[test]
fn test_valid_config_accepted() {
    let mut servers = HashMap::new();
    servers.insert(
        "rust".to_string(),
        LspServerConfig {
            language: "rust".to_string(),
            command: "rust-analyzer".to_string(),
            args: vec![],
            timeout_ms: 5000,
        },
    );

    let config = IdeIntegrationConfig {
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
                servers,
                health_check_interval_ms: 5000,
            },
            configured_rules: None,
            builtin_providers: BuiltinProvidersConfig {
                enabled: true,
                languages: vec!["rust".to_string()],
            },
        },
    };

    // Valid configurations should be accepted
    let result = ConfigValidator::validate_complete(&config);
    assert!(result.is_ok(), "Valid configuration should be accepted");
}

#[test]
fn test_invalid_vscode_port_zero_rejected() {
    let config = IdeIntegrationConfig {
        vscode: Some(VsCodeConfig {
            enabled: true,
            port: 0,
            features: vec!["completion".to_string()],
            settings: serde_json::json!({}),
        }),
        terminal: None,
        providers: ProviderChainConfig {
            external_lsp: ExternalLspConfig {
                enabled: false,
                servers: Default::default(),
                health_check_interval_ms: 5000,
            },
            configured_rules: None,
            builtin_providers: BuiltinProvidersConfig {
                enabled: true,
                languages: vec!["rust".to_string()],
            },
        },
    };

    let result = ConfigValidator::validate_complete(&config);
    assert!(result.is_err(), "Invalid port should be rejected");

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("port") || error_msg.contains("Port"),
        "Error should mention port: {}",
        error_msg
    );
    assert!(
        error_msg.contains("Remediation") || error_msg.contains("remediation"),
        "Error should include remediation steps: {}",
        error_msg
    );
}

#[test]
fn test_no_providers_enabled_rejected() {
    let config = IdeIntegrationConfig {
        vscode: None,
        terminal: None,
        providers: ProviderChainConfig {
            external_lsp: ExternalLspConfig {
                enabled: false,
                servers: Default::default(),
                health_check_interval_ms: 5000,
            },
            configured_rules: None,
            builtin_providers: BuiltinProvidersConfig {
                enabled: false,
                languages: vec![],
            },
        },
    };

    let result = ConfigValidator::validate_complete(&config);
    assert!(result.is_err(), "No providers enabled should be rejected");

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("provider") || error_msg.contains("Provider"),
        "Error should mention provider: {}",
        error_msg
    );
    assert!(
        error_msg.contains("enabled"),
        "Error should mention 'enabled': {}",
        error_msg
    );
    assert!(
        error_msg.contains("Remediation") || error_msg.contains("remediation"),
        "Error should include remediation steps: {}",
        error_msg
    );
}

#[test]
fn test_empty_lsp_servers_rejected() {
    let config = IdeIntegrationConfig {
        vscode: None,
        terminal: None,
        providers: ProviderChainConfig {
            external_lsp: ExternalLspConfig {
                enabled: true,
                servers: HashMap::new(),
                health_check_interval_ms: 5000,
            },
            configured_rules: None,
            builtin_providers: BuiltinProvidersConfig {
                enabled: false,
                languages: vec![],
            },
        },
    };

    let result = ConfigValidator::validate_complete(&config);
    assert!(result.is_err(), "Empty LSP servers should be rejected");

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("server") || error_msg.contains("Server"),
        "Error should mention server: {}",
        error_msg
    );
    assert!(
        error_msg.contains("Remediation") || error_msg.contains("remediation"),
        "Error should include remediation steps: {}",
        error_msg
    );
}

#[test]
fn test_empty_builtin_languages_rejected() {
    let config = IdeIntegrationConfig {
        vscode: None,
        terminal: None,
        providers: ProviderChainConfig {
            external_lsp: ExternalLspConfig {
                enabled: false,
                servers: Default::default(),
                health_check_interval_ms: 5000,
            },
            configured_rules: None,
            builtin_providers: BuiltinProvidersConfig {
                enabled: true,
                languages: vec![],
            },
        },
    };

    let result = ConfigValidator::validate_complete(&config);
    assert!(result.is_err(), "Empty builtin languages should be rejected");

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("language") || error_msg.contains("Language"),
        "Error should mention language: {}",
        error_msg
    );
    assert!(
        error_msg.contains("Remediation") || error_msg.contains("remediation"),
        "Error should include remediation steps: {}",
        error_msg
    );
}

#[test]
fn test_empty_vscode_features_rejected() {
    let config = IdeIntegrationConfig {
        vscode: Some(VsCodeConfig {
            enabled: true,
            port: 8080,
            features: vec![],
            settings: serde_json::json!({}),
        }),
        terminal: None,
        providers: ProviderChainConfig {
            external_lsp: ExternalLspConfig {
                enabled: false,
                servers: Default::default(),
                health_check_interval_ms: 5000,
            },
            configured_rules: None,
            builtin_providers: BuiltinProvidersConfig {
                enabled: true,
                languages: vec!["rust".to_string()],
            },
        },
    };

    let result = ConfigValidator::validate_complete(&config);
    assert!(result.is_err(), "Empty VS Code features should be rejected");

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("feature") || error_msg.contains("Feature"),
        "Error should mention feature: {}",
        error_msg
    );
    assert!(
        error_msg.contains("Remediation") || error_msg.contains("remediation"),
        "Error should include remediation steps: {}",
        error_msg
    );
}

#[test]
fn test_invalid_lsp_timeout_zero_rejected() {
    let mut servers = HashMap::new();
    servers.insert(
        "rust".to_string(),
        LspServerConfig {
            language: "rust".to_string(),
            command: "rust-analyzer".to_string(),
            args: vec![],
            timeout_ms: 0,
        },
    );

    let config = IdeIntegrationConfig {
        vscode: None,
        terminal: None,
        providers: ProviderChainConfig {
            external_lsp: ExternalLspConfig {
                enabled: true,
                servers,
                health_check_interval_ms: 5000,
            },
            configured_rules: None,
            builtin_providers: BuiltinProvidersConfig {
                enabled: false,
                languages: vec![],
            },
        },
    };

    let result = ConfigValidator::validate_complete(&config);
    assert!(result.is_err(), "Invalid timeout should be rejected");

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("timeout") || error_msg.contains("Timeout"),
        "Error should mention timeout: {}",
        error_msg
    );
    assert!(
        error_msg.contains("Remediation") || error_msg.contains("remediation"),
        "Error should include remediation steps: {}",
        error_msg
    );
}

#[test]
fn test_invalid_lsp_timeout_excessive_rejected() {
    let mut servers = HashMap::new();
    servers.insert(
        "rust".to_string(),
        LspServerConfig {
            language: "rust".to_string(),
            command: "rust-analyzer".to_string(),
            args: vec![],
            timeout_ms: 200000,
        },
    );

    let config = IdeIntegrationConfig {
        vscode: None,
        terminal: None,
        providers: ProviderChainConfig {
            external_lsp: ExternalLspConfig {
                enabled: true,
                servers,
                health_check_interval_ms: 5000,
            },
            configured_rules: None,
            builtin_providers: BuiltinProvidersConfig {
                enabled: false,
                languages: vec![],
            },
        },
    };

    let result = ConfigValidator::validate_complete(&config);
    assert!(result.is_err(), "Excessive timeout should be rejected");

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("timeout") || error_msg.contains("Timeout"),
        "Error should mention timeout: {}",
        error_msg
    );
    assert!(
        error_msg.contains("Remediation") || error_msg.contains("remediation"),
        "Error should include remediation steps: {}",
        error_msg
    );
}

#[test]
fn test_invalid_lsp_empty_command_rejected() {
    let mut servers = HashMap::new();
    servers.insert(
        "rust".to_string(),
        LspServerConfig {
            language: "rust".to_string(),
            command: String::new(),
            args: vec![],
            timeout_ms: 5000,
        },
    );

    let config = IdeIntegrationConfig {
        vscode: None,
        terminal: None,
        providers: ProviderChainConfig {
            external_lsp: ExternalLspConfig {
                enabled: true,
                servers,
                health_check_interval_ms: 5000,
            },
            configured_rules: None,
            builtin_providers: BuiltinProvidersConfig {
                enabled: false,
                languages: vec![],
            },
        },
    };

    let result = ConfigValidator::validate_complete(&config);
    assert!(result.is_err(), "Empty command should be rejected");

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("command") || error_msg.contains("Command"),
        "Error should mention command: {}",
        error_msg
    );
    assert!(
        error_msg.contains("Remediation") || error_msg.contains("remediation"),
        "Error should include remediation steps: {}",
        error_msg
    );
}

#[test]
fn test_invalid_builtin_language_rejected() {
    let config = IdeIntegrationConfig {
        vscode: None,
        terminal: None,
        providers: ProviderChainConfig {
            external_lsp: ExternalLspConfig {
                enabled: false,
                servers: Default::default(),
                health_check_interval_ms: 5000,
            },
            configured_rules: None,
            builtin_providers: BuiltinProvidersConfig {
                enabled: true,
                languages: vec!["invalid_language".to_string()],
            },
        },
    };

    let result = ConfigValidator::validate_complete(&config);
    assert!(result.is_err(), "Invalid language should be rejected");

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("language") || error_msg.contains("Language"),
        "Error should mention language: {}",
        error_msg
    );
    assert!(
        error_msg.contains("invalid_language"),
        "Error should include the invalid language name: {}",
        error_msg
    );
    assert!(
        error_msg.contains("Remediation") || error_msg.contains("remediation"),
        "Error should include remediation steps: {}",
        error_msg
    );
}

#[test]
fn test_invalid_vscode_feature_rejected() {
    let config = IdeIntegrationConfig {
        vscode: Some(VsCodeConfig {
            enabled: true,
            port: 8080,
            features: vec!["invalid_feature".to_string()],
            settings: serde_json::json!({}),
        }),
        terminal: None,
        providers: ProviderChainConfig {
            external_lsp: ExternalLspConfig {
                enabled: false,
                servers: Default::default(),
                health_check_interval_ms: 5000,
            },
            configured_rules: None,
            builtin_providers: BuiltinProvidersConfig {
                enabled: true,
                languages: vec!["rust".to_string()],
            },
        },
    };

    let result = ConfigValidator::validate_complete(&config);
    assert!(result.is_err(), "Invalid feature should be rejected");

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("feature") || error_msg.contains("Feature"),
        "Error should mention feature: {}",
        error_msg
    );
    assert!(
        error_msg.contains("invalid_feature"),
        "Error should include the invalid feature name: {}",
        error_msg
    );
    assert!(
        error_msg.contains("Remediation") || error_msg.contains("remediation"),
        "Error should include remediation steps: {}",
        error_msg
    );
}
