//! Property-based tests for IDE-specific configuration
//! **Feature: ricecoder-ide, Property 6: IDE-Specific Configuration**
//! **Validates: Requirements 1.6**
//!
//! Tests that IDE-specific settings are applied correctly without affecting other IDEs

use ricecoder_ide::{
    IdeConfigApplicator, IdeIntegrationConfig, IdeSpecificSettings, IdeType, VsCodeConfig,
    TerminalConfig, ProviderChainConfig, ExternalLspConfig,
    BuiltinProvidersConfig, DiagnosticSeverity, Hover, Position,
    Range,
};
use std::collections::HashMap;

/// Create a test IDE configuration for a specific IDE type
fn create_test_config(ide_type: IdeType) -> IdeIntegrationConfig {
    match ide_type {
        IdeType::VsCode => IdeIntegrationConfig {
            vscode: Some(VsCodeConfig {
                enabled: true,
                port: 8080,
                features: vec!["completion".to_string(), "diagnostics".to_string()],
                settings: serde_json::json!({
                    "max_completion_items": 20,
                    "timeout_ms": 5000,
                }),
            }),
            terminal: None,
            providers: ProviderChainConfig {
                external_lsp: ExternalLspConfig {
                    enabled: true,
                    servers: HashMap::new(),
                    health_check_interval_ms: 5000,
                },
                configured_rules: None,
                builtin_providers: BuiltinProvidersConfig {
                    enabled: true,
                    languages: vec!["rust".to_string()],
                },
            },
        },
        IdeType::Vim => IdeIntegrationConfig {
            vscode: None,
            terminal: Some(TerminalConfig {
                vim: Some(ricecoder_ide::types::VimConfig {
                    enabled: true,
                    plugin_manager: "vim-plug".to_string(),
                }),
                emacs: None,
            }),
            providers: ProviderChainConfig {
                external_lsp: ExternalLspConfig {
                    enabled: true,
                    servers: HashMap::new(),
                    health_check_interval_ms: 5000,
                },
                configured_rules: None,
                builtin_providers: BuiltinProvidersConfig {
                    enabled: true,
                    languages: vec!["rust".to_string()],
                },
            },
        },
        IdeType::Emacs => IdeIntegrationConfig {
            vscode: None,
            terminal: Some(TerminalConfig {
                vim: None,
                emacs: Some(ricecoder_ide::types::EmacsConfig {
                    enabled: true,
                    package_manager: "use-package".to_string(),
                }),
            }),
            providers: ProviderChainConfig {
                external_lsp: ExternalLspConfig {
                    enabled: true,
                    servers: HashMap::new(),
                    health_check_interval_ms: 5000,
                },
                configured_rules: None,
                builtin_providers: BuiltinProvidersConfig {
                    enabled: true,
                    languages: vec!["rust".to_string()],
                },
            },
        },
        IdeType::Unknown => IdeIntegrationConfig {
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
                    enabled: true,
                    languages: vec!["rust".to_string()],
                },
            },
        },
    }
}

#[test]
fn test_ide_specific_configuration_vscode_settings() {
    let config = create_test_config(IdeType::VsCode);
    let settings = IdeConfigApplicator::extract_settings(&config, IdeType::VsCode).unwrap();

    assert_eq!(settings.ide_type, IdeType::VsCode);
    assert!(settings.is_feature_enabled("completion"));
    assert!(settings.is_feature_enabled("diagnostics"));
    assert_eq!(settings.port, 8080);
}

#[test]
fn test_ide_specific_configuration_vim_settings() {
    let config = create_test_config(IdeType::Vim);
    let settings = IdeConfigApplicator::extract_settings(&config, IdeType::Vim).unwrap();

    assert_eq!(settings.ide_type, IdeType::Vim);
    assert!(settings.is_feature_enabled("completion"));
    assert!(settings.is_feature_enabled("diagnostics"));
    assert!(settings.is_feature_enabled("hover"));
    assert_eq!(settings.port, 9000);
}

#[test]
fn test_ide_specific_configuration_emacs_settings() {
    let config = create_test_config(IdeType::Emacs);
    let settings = IdeConfigApplicator::extract_settings(&config, IdeType::Emacs).unwrap();

    assert_eq!(settings.ide_type, IdeType::Emacs);
    assert!(settings.is_feature_enabled("completion"));
    assert!(settings.is_feature_enabled("diagnostics"));
    assert!(settings.is_feature_enabled("hover"));
    assert_eq!(settings.port, 9000);
}

#[test]
fn test_ide_specific_configuration_isolation() {
    let vscode_config = create_test_config(IdeType::VsCode);
    let vim_config = create_test_config(IdeType::Vim);

    let vscode_settings = IdeConfigApplicator::extract_settings(&vscode_config, IdeType::VsCode);
    let vim_settings = IdeConfigApplicator::extract_settings(&vim_config, IdeType::Vim);

    assert!(vscode_settings.is_ok());
    assert!(vim_settings.is_ok());

    let vscode_settings = vscode_settings.unwrap();
    let vim_settings = vim_settings.unwrap();

    assert_eq!(vscode_settings.ide_type, IdeType::VsCode);
    assert_eq!(vim_settings.ide_type, IdeType::Vim);
    assert_ne!(vscode_settings.port, vim_settings.port);
}

#[test]
fn test_completion_behavior_truncation() {
    let mut items: Vec<_> = (0..50)
        .map(|i| ricecoder_ide::CompletionItem {
            label: format!("item_{}", i),
            kind: ricecoder_ide::CompletionItemKind::Text,
            detail: None,
            documentation: None,
            insert_text: format!("item_{}", i),
        })
        .collect();

    let settings = IdeSpecificSettings::new(IdeType::VsCode)
        .with_port(8080)
        .with_timeout(5000)
        .with_setting("max_completion_items".to_string(), serde_json::json!(20));

    IdeConfigApplicator::apply_completion_behavior(&mut items, &settings);

    assert!(items.len() <= 20);
}

#[test]
fn test_diagnostics_behavior_filtering() {
    let mut diagnostics = vec![
        ricecoder_ide::Diagnostic {
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 10,
                },
            },
            severity: DiagnosticSeverity::Error,
            message: "error".to_string(),
            source: "test".to_string(),
        },
        ricecoder_ide::Diagnostic {
            range: Range {
                start: Position {
                    line: 1,
                    character: 0,
                },
                end: Position {
                    line: 1,
                    character: 10,
                },
            },
            severity: DiagnosticSeverity::Warning,
            message: "warning".to_string(),
            source: "test".to_string(),
        },
        ricecoder_ide::Diagnostic {
            range: Range {
                start: Position {
                    line: 2,
                    character: 0,
                },
                end: Position {
                    line: 2,
                    character: 10,
                },
            },
            severity: DiagnosticSeverity::Information,
            message: "info".to_string(),
            source: "test".to_string(),
        },
    ];

    let settings = IdeSpecificSettings::new(IdeType::Vim)
        .with_port(9000)
        .with_timeout(5000)
        .with_setting("min_severity".to_string(), serde_json::json!(1));

    IdeConfigApplicator::apply_diagnostics_behavior(&mut diagnostics, &settings);

    // Should only keep errors (severity 1)
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].message, "error");
}

#[test]
fn test_ide_type_parsing() {
    assert_eq!(IdeType::from_str("vscode"), IdeType::VsCode);
    assert_eq!(IdeType::from_str("vs-code"), IdeType::VsCode);
    assert_eq!(IdeType::from_str("vim"), IdeType::Vim);
    assert_eq!(IdeType::from_str("neovim"), IdeType::Vim);
    assert_eq!(IdeType::from_str("emacs"), IdeType::Emacs);
}

#[test]
fn test_ide_type_as_str() {
    assert_eq!(IdeType::VsCode.as_str(), "vscode");
    assert_eq!(IdeType::Vim.as_str(), "vim");
    assert_eq!(IdeType::Emacs.as_str(), "emacs");
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

#[test]
fn test_completion_behavior_preserves_items() {
    let mut items: Vec<_> = (0..10)
        .map(|i| ricecoder_ide::CompletionItem {
            label: format!("item_{}", i),
            kind: ricecoder_ide::CompletionItemKind::Text,
            detail: None,
            documentation: None,
            insert_text: format!("item_{}", i),
        })
        .collect();

    let settings = IdeSpecificSettings::new(IdeType::VsCode)
        .with_port(8080)
        .with_timeout(5000);

    let original_count = items.len();
    IdeConfigApplicator::apply_completion_behavior(&mut items, &settings);

    // Items should not increase
    assert!(items.len() <= original_count);

    // All items should still be valid
    for item in &items {
        assert!(!item.label.is_empty());
        assert!(!item.insert_text.is_empty());
    }
}

#[test]
fn test_diagnostics_behavior_filters_correctly() {
    let mut diagnostics = vec![
        ricecoder_ide::Diagnostic {
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 10,
                },
            },
            severity: DiagnosticSeverity::Error,
            message: "error".to_string(),
            source: "test".to_string(),
        },
        ricecoder_ide::Diagnostic {
            range: Range {
                start: Position {
                    line: 1,
                    character: 0,
                },
                end: Position {
                    line: 1,
                    character: 10,
                },
            },
            severity: DiagnosticSeverity::Warning,
            message: "warning".to_string(),
            source: "test".to_string(),
        },
    ];

    let settings = IdeSpecificSettings::new(IdeType::Vim)
        .with_port(9000)
        .with_timeout(5000);

    let original_count = diagnostics.len();
    IdeConfigApplicator::apply_diagnostics_behavior(&mut diagnostics, &settings);

    // Diagnostics should not increase
    assert!(diagnostics.len() <= original_count);

    // All remaining diagnostics should be valid
    for diag in &diagnostics {
        assert!(!diag.message.is_empty());
    }
}

#[test]
fn test_hover_behavior_preserves_content() {
    let mut hover = Some(Hover {
        contents: "test content".to_string(),
        range: Some(Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 0,
                character: 10,
            },
        }),
    });

    let settings = IdeSpecificSettings::new(IdeType::VsCode)
        .with_port(8080)
        .with_timeout(5000);

    let original_has_content = hover.is_some();
    IdeConfigApplicator::apply_hover_behavior(&mut hover, &settings);

    // Hover presence should not change
    assert_eq!(hover.is_some(), original_has_content);

    // If hover exists, content should not be empty
    if let Some(h) = &hover {
        assert!(!h.contents.is_empty());
    }
}

#[test]
fn test_ide_settings_dont_affect_other_ides() {
    let vscode_config = create_test_config(IdeType::VsCode);
    let vim_config = create_test_config(IdeType::Vim);
    let emacs_config = create_test_config(IdeType::Emacs);

    let vscode_settings = IdeConfigApplicator::extract_settings(&vscode_config, IdeType::VsCode);
    let vim_settings = IdeConfigApplicator::extract_settings(&vim_config, IdeType::Vim);
    let emacs_settings = IdeConfigApplicator::extract_settings(&emacs_config, IdeType::Emacs);

    // All should succeed
    assert!(vscode_settings.is_ok());
    assert!(vim_settings.is_ok());
    assert!(emacs_settings.is_ok());

    let vscode_settings = vscode_settings.unwrap();
    let vim_settings = vim_settings.unwrap();
    let emacs_settings = emacs_settings.unwrap();

    // Each should have correct IDE type
    assert_eq!(vscode_settings.ide_type, IdeType::VsCode);
    assert_eq!(vim_settings.ide_type, IdeType::Vim);
    assert_eq!(emacs_settings.ide_type, IdeType::Emacs);

    // Settings should be different (VS Code uses different port than terminal editors)
    assert_ne!(vscode_settings.port, vim_settings.port);
    // Vim and Emacs both use port 9000 by default, so they're the same
    assert_eq!(vim_settings.port, emacs_settings.port);
}

#[test]
fn test_settings_validation_consistency() {
    let vscode_settings = IdeSpecificSettings::new(IdeType::VsCode)
        .with_port(8080)
        .with_timeout(5000);

    let vim_settings = IdeSpecificSettings::new(IdeType::Vim)
        .with_port(9000)
        .with_timeout(5000);

    let emacs_settings = IdeSpecificSettings::new(IdeType::Emacs)
        .with_port(9000)
        .with_timeout(5000);

    // All should validate successfully
    assert!(IdeConfigApplicator::validate_settings(&vscode_settings).is_ok());
    assert!(IdeConfigApplicator::validate_settings(&vim_settings).is_ok());
    assert!(IdeConfigApplicator::validate_settings(&emacs_settings).is_ok());
}

#[test]
fn test_custom_settings_storage_and_retrieval() {
    let settings = IdeSpecificSettings::new(IdeType::VsCode)
        .with_port(8080)
        .with_timeout(5000)
        .with_setting("key1".to_string(), serde_json::json!("value1"))
        .with_setting("key2".to_string(), serde_json::json!(42))
        .with_setting("key3".to_string(), serde_json::json!(true));

    // All settings should be retrievable
    assert_eq!(settings.get_setting("key1").unwrap().as_str().unwrap(), "value1");
    assert_eq!(settings.get_setting("key2").unwrap().as_u64().unwrap(), 42);
    assert_eq!(settings.get_setting("key3").unwrap().as_bool().unwrap(), true);
    assert!(settings.get_setting("key4").is_none());
}
