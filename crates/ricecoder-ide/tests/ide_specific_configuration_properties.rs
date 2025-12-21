//! Property-based tests for IDE-specific configuration
//! **Feature: ricecoder-ide, Property 6: IDE-Specific Configuration**
//! **Validates: Requirements 1.6**
//!
//! Tests that IDE-specific settings are applied correctly without affecting other IDEs.
//! For any IDE-specific setting (VS Code settings, vim config, emacs config), the system
//! SHALL support IDE-specific configuration and apply it correctly without affecting other IDEs.

use proptest::prelude::*;
use ricecoder_ide::{
    BuiltinProvidersConfig, CompletionItem, CompletionItemKind, Diagnostic, DiagnosticSeverity,
    ExternalLspConfig, Hover, IdeConfigApplicator, IdeIntegrationConfig, IdeSpecificSettings,
    IdeType, Position, ProviderChainConfig, Range, TerminalConfig, VsCodeConfig,
};
use std::collections::HashMap;

/// Strategy for generating IDE types
fn ide_type_strategy() -> impl Strategy<Value = IdeType> {
    prop_oneof![
        Just(IdeType::VsCode),
        Just(IdeType::Vim),
        Just(IdeType::Emacs),
    ]
}

/// Strategy for generating port numbers (valid range: 1-65535)
fn port_strategy() -> impl Strategy<Value = u16> {
    1u16..=65535u16
}

/// Strategy for generating timeout values (valid range: 100-60000 ms)
fn timeout_strategy() -> impl Strategy<Value = u64> {
    100u64..=60000u64
}

/// Strategy for generating feature names
fn feature_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("completion".to_string()),
        Just("diagnostics".to_string()),
        Just("hover".to_string()),
        Just("definition".to_string()),
        Just("references".to_string()),
    ]
}

/// Strategy for generating completion items
fn completion_item_strategy() -> impl Strategy<Value = CompletionItem> {
    (
        "[a-z_][a-z0-9_]{0,20}",
        prop_oneof![
            Just(CompletionItemKind::Function),
            Just(CompletionItemKind::Variable),
            Just(CompletionItemKind::Class),
            Just(CompletionItemKind::Text),
        ],
    )
        .prop_map(|(label, kind)| CompletionItem {
            label: label.clone(),
            kind,
            detail: Some(format!("Detail for {}", label)),
            documentation: Some(format!("Documentation for {}", label)),
            insert_text: label,
        })
}

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

proptest! {
    /// Property 6: IDE-Specific Configuration
    /// For any IDE-specific setting (VS Code settings, vim config, emacs config),
    /// the system SHALL support IDE-specific configuration and apply it correctly
    /// without affecting other IDEs.
    ///
    /// This property tests that IDE settings are correctly extracted and applied
    /// for each IDE type without cross-contamination.
    #[test]
    fn prop_ide_settings_extracted_correctly(ide_type in ide_type_strategy()) {
        let config = create_test_config(ide_type);
        let result = IdeConfigApplicator::extract_settings(&config, ide_type);

        // Settings should be successfully extracted for all IDE types
        prop_assert!(result.is_ok(), "Failed to extract settings for IDE: {:?}", ide_type);

        let settings = result.unwrap();

        // IDE type should match
        prop_assert_eq!(settings.ide_type, ide_type, "IDE type mismatch");

        // Port should be valid (non-zero for known IDEs)
        if ide_type != IdeType::Unknown {
            prop_assert!(settings.port > 0, "Port should be non-zero for IDE: {:?}", ide_type);
        }

        // Timeout should be valid
        prop_assert!(settings.timeout_ms > 0, "Timeout should be positive");
    }

    /// Property 6 (continued): IDE settings isolation
    /// For any pair of different IDE types, their settings should not affect each other.
    /// Each IDE should have its own isolated configuration.
    #[test]
    fn prop_ide_settings_isolation(
        ide_type_1 in ide_type_strategy(),
        ide_type_2 in ide_type_strategy()
    ) {
        // Skip if same IDE type
        if ide_type_1 == ide_type_2 {
            return Ok(());
        }

        let config_1 = create_test_config(ide_type_1);
        let config_2 = create_test_config(ide_type_2);

        let settings_1 = IdeConfigApplicator::extract_settings(&config_1, ide_type_1);
        let settings_2 = IdeConfigApplicator::extract_settings(&config_2, ide_type_2);

        // Both should succeed
        prop_assert!(settings_1.is_ok(), "Failed to extract settings for IDE 1");
        prop_assert!(settings_2.is_ok(), "Failed to extract settings for IDE 2");

        let settings_1 = settings_1.unwrap();
        let settings_2 = settings_2.unwrap();

        // IDE types should be different
        prop_assert_ne!(settings_1.ide_type, settings_2.ide_type, "IDE types should be different");

        // Settings should be independent
        prop_assert_eq!(settings_1.ide_type, ide_type_1, "IDE 1 type mismatch");
        prop_assert_eq!(settings_2.ide_type, ide_type_2, "IDE 2 type mismatch");
    }

    /// Property 6 (continued): IDE settings validation
    /// For any valid IDE settings with valid port and timeout,
    /// validation should succeed.
    #[test]
    fn prop_ide_settings_validation(
        ide_type in ide_type_strategy(),
        port in port_strategy(),
        timeout in timeout_strategy()
    ) {
        let settings = IdeSpecificSettings::new(ide_type)
            .with_port(port)
            .with_timeout(timeout);

        // Validation should succeed for valid settings
        let result = IdeConfigApplicator::validate_settings(&settings);
        prop_assert!(result.is_ok(), "Validation failed for valid settings");
    }

    /// Property 6 (continued): IDE settings with features
    /// For any IDE type and any set of features, settings should correctly
    /// track which features are enabled.
    #[test]
    fn prop_ide_settings_features(
        ide_type in ide_type_strategy(),
        features in prop::collection::vec(feature_strategy(), 0..5)
    ) {
        let mut settings = IdeSpecificSettings::new(ide_type)
            .with_port(8080)
            .with_timeout(5000);

        // Add all features
        for feature in &features {
            settings = settings.with_feature(feature.clone());
        }

        // Verify all features are tracked
        for feature in &features {
            prop_assert!(
                settings.is_feature_enabled(feature),
                "Feature {} should be enabled",
                feature
            );
        }

        // Verify feature count matches
        prop_assert_eq!(
            settings.enabled_features.len(),
            features.len(),
            "Feature count mismatch"
        );
    }

    /// Property 6 (continued): IDE settings custom settings storage
    /// For any IDE type and any custom settings, settings should correctly
    /// store and retrieve custom values.
    #[test]
    fn prop_ide_settings_custom_storage(
        ide_type in ide_type_strategy(),
        key1 in "[a-z_][a-z0-9_]{0,10}",
        key2 in "[a-z_][a-z0-9_]{0,10}",
        value1 in "[a-z0-9]{1,20}",
        value2 in 0u32..1000u32
    ) {
        // Skip if keys are the same (can't test both retrievals)
        if key1 == key2 {
            return Ok(());
        }

        let settings = IdeSpecificSettings::new(ide_type)
            .with_port(8080)
            .with_timeout(5000)
            .with_setting(key1.clone(), serde_json::json!(value1.clone()))
            .with_setting(key2.clone(), serde_json::json!(value2));

        // Verify settings are stored and retrievable
        prop_assert_eq!(
            settings.get_setting(&key1).unwrap().as_str().unwrap(),
            value1,
            "String setting mismatch"
        );

        prop_assert_eq!(
            settings.get_setting(&key2).unwrap().as_u64().unwrap() as u32,
            value2,
            "Numeric setting mismatch"
        );

        // Verify non-existent setting returns None
        prop_assert!(
            settings.get_setting("nonexistent").is_none(),
            "Non-existent setting should return None"
        );
    }

    /// Property 6 (continued): Completion behavior preserves validity
    /// For any IDE type and any set of completion items, applying IDE-specific
    /// behavior should preserve item validity (non-empty labels and insert text).
    #[test]
    fn prop_completion_behavior_preserves_validity(
        ide_type in ide_type_strategy(),
        mut items in prop::collection::vec(completion_item_strategy(), 1..20)
    ) {
        let settings = IdeSpecificSettings::new(ide_type)
            .with_port(8080)
            .with_timeout(5000);

        let original_count = items.len();
        IdeConfigApplicator::apply_completion_behavior(&mut items, &settings);

        // Items should not increase
        prop_assert!(
            items.len() <= original_count,
            "Completion items should not increase"
        );

        // All remaining items should be valid
        for item in &items {
            prop_assert!(!item.label.is_empty(), "Item label should not be empty");
            prop_assert!(!item.insert_text.is_empty(), "Item insert_text should not be empty");
        }
    }

    /// Property 6 (continued): Diagnostics behavior preserves validity
    /// For any IDE type and any set of diagnostics, applying IDE-specific
    /// behavior should preserve diagnostic validity.
    #[test]
    fn prop_diagnostics_behavior_preserves_validity(
        ide_type in ide_type_strategy(),
        diagnostics in prop::collection::vec(
            (
                0u32..100u32,
                0u32..100u32,
                0u32..100u32,
                0u32..100u32,
                "[a-z ]{1,50}",
                "[a-z]{1,20}"
            ),
            1..10
        )
    ) {
        let mut diags: Vec<Diagnostic> = diagnostics
            .into_iter()
            .map(|(l1, c1, l2, c2, msg, src)| Diagnostic {
                range: Range {
                    start: Position {
                        line: l1,
                        character: c1,
                    },
                    end: Position {
                        line: l2,
                        character: c2,
                    },
                },
                severity: DiagnosticSeverity::Warning,
                message: msg,
                source: src,
            })
            .collect();

        let settings = IdeSpecificSettings::new(ide_type)
            .with_port(8080)
            .with_timeout(5000);

        let original_count = diags.len();
        IdeConfigApplicator::apply_diagnostics_behavior(&mut diags, &settings);

        // Diagnostics should not increase
        prop_assert!(
            diags.len() <= original_count,
            "Diagnostics should not increase"
        );

        // All remaining diagnostics should be valid
        for diag in &diags {
            prop_assert!(!diag.message.is_empty(), "Diagnostic message should not be empty");
            prop_assert!(!diag.source.is_empty(), "Diagnostic source should not be empty");
        }
    }

    /// Property 6 (continued): Hover behavior preserves content
    /// For any IDE type and any hover information, applying IDE-specific
    /// behavior should preserve hover content validity.
    #[test]
    fn prop_hover_behavior_preserves_content(
        ide_type in ide_type_strategy(),
        content in "[a-z]{1,100}"  // Non-whitespace content only
    ) {
        let mut hover = Some(Hover {
            contents: content.clone(),
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

        let settings = IdeSpecificSettings::new(ide_type)
            .with_port(8080)
            .with_timeout(5000);

        let original_has_content = hover.is_some();
        IdeConfigApplicator::apply_hover_behavior(&mut hover, &settings);

        // Hover presence should not change
        prop_assert_eq!(
            hover.is_some(),
            original_has_content,
            "Hover presence should not change"
        );

        // If hover exists, content should not be empty
        if let Some(h) = &hover {
            prop_assert!(!h.contents.is_empty(), "Hover content should not be empty");
        }
    }

    /// Property 6 (continued): IDE type parsing consistency
    /// For any IDE type, parsing its string representation should return the same type.
    #[test]
    fn prop_ide_type_parsing_consistency(ide_type in ide_type_strategy()) {
        let ide_str = ide_type.as_str();
        let parsed = IdeType::parse(ide_str);

        prop_assert_eq!(parsed, ide_type, "IDE type parsing should be consistent");
    }

    /// Property 6 (continued): Settings builder idempotence
    /// For any IDE type and settings, building settings multiple times
    /// with the same values should produce equivalent results.
    #[test]
    fn prop_settings_builder_idempotence(
        ide_type in ide_type_strategy(),
        port in port_strategy(),
        timeout in timeout_strategy()
    ) {
        let settings1 = IdeSpecificSettings::new(ide_type)
            .with_port(port)
            .with_timeout(timeout);

        let settings2 = IdeSpecificSettings::new(ide_type)
            .with_port(port)
            .with_timeout(timeout);

        // Both should have same values
        prop_assert_eq!(settings1.ide_type, settings2.ide_type);
        prop_assert_eq!(settings1.port, settings2.port);
        prop_assert_eq!(settings1.timeout_ms, settings2.timeout_ms);
    }
}
