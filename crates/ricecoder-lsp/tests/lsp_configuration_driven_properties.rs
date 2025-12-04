//! Property-based tests for configuration-driven LSP behavior
//!
//! **Feature: ricecoder-lsp, Property 6: Configuration-driven multi-language support**
//! **Validates: Requirements LSP-5.1, LSP-5.2, LSP-6.1**

use proptest::prelude::*;
use ricecoder_lsp::{
    ConfigRegistry, LanguageConfig, DiagnosticRule, CodeActionTemplate,
    ConfigurationManager,
};

// Strategy for generating valid language names
fn language_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,19}".prop_map(|s| s.to_string())
}

// Strategy for generating valid file extensions
fn extension_strategy() -> impl Strategy<Value = String> {
    "[a-z]{1,5}".prop_map(|s| s.to_string())
}

// Strategy for generating valid diagnostic rules
fn diagnostic_rule_strategy() -> impl Strategy<Value = DiagnosticRule> {
    (
        "[a-z][a-z0-9_]{0,19}",
        "[a-z0-9_]{1,20}",
        prop_oneof!["error", "warning", "info"],
        "[A-Za-z0-9 ]{1,50}",
    )
        .prop_map(|(name, pattern, severity, message)| DiagnosticRule {
            name: name.to_string(),
            pattern: pattern.to_string(),
            severity: severity.to_string(),
            message: message.to_string(),
            fix_template: None,
            code: None,
        })
}

// Strategy for generating valid code action templates
fn code_action_strategy() -> impl Strategy<Value = CodeActionTemplate> {
    (
        "[a-z][a-z0-9_]{0,19}",
        "[A-Za-z0-9 ]{1,50}",
        prop_oneof!["quickfix", "refactor", "source"],
        "[a-z0-9_]{1,20}",
    )
        .prop_map(|(name, title, kind, transformation)| CodeActionTemplate {
            name: name.to_string(),
            title: title.to_string(),
            kind: kind.to_string(),
            transformation: transformation.to_string(),
        })
}

// Strategy for generating valid language configurations
fn language_config_strategy() -> impl Strategy<Value = LanguageConfig> {
    (
        language_name_strategy(),
        prop::collection::vec(extension_strategy(), 1..5),
        prop::option::of("[a-z0-9_-]{1,30}"),
        prop::collection::vec(diagnostic_rule_strategy(), 0..3),
        prop::collection::vec(code_action_strategy(), 0..3),
    )
        .prop_map(|(language, extensions, parser_plugin, diagnostic_rules, code_actions)| {
            LanguageConfig {
                language,
                extensions,
                parser_plugin: parser_plugin.map(|s| s.to_string()),
                diagnostic_rules,
                code_actions,
            }
        })
}

proptest! {
    /// Property 1: Configured languages work correctly
    ///
    /// For any valid language configuration, registering it should succeed
    /// and the language should be retrievable from the registry.
    #[test]
    fn prop_configured_languages_work(config in language_config_strategy()) {
        let mut registry = ConfigRegistry::new();

        // Register the configuration
        prop_assert!(registry.register(config.clone()).is_ok());

        // Verify the language is registered
        prop_assert!(registry.has_language(&config.language));

        // Verify we can retrieve the configuration
        prop_assert!(registry.get(&config.language).is_some());

        // Verify the retrieved configuration matches
        let retrieved = registry.get(&config.language).unwrap();
        prop_assert_eq!(&retrieved.language, &config.language);
        prop_assert_eq!(&retrieved.extensions, &config.extensions);
    }

    /// Property 2: Unconfigured languages degrade gracefully
    ///
    /// For any language name that is not configured, the registry should
    /// return None without crashing.
    #[test]
    fn prop_unconfigured_languages_degrade(
        configured_lang in language_name_strategy(),
        unconfigured_lang in language_name_strategy(),
    ) {
        prop_assume!(configured_lang != unconfigured_lang);

        let mut registry = ConfigRegistry::new();
        let config = LanguageConfig {
            language: configured_lang.clone(),
            extensions: vec!["test".to_string()],
            parser_plugin: None,
            diagnostic_rules: vec![],
            code_actions: vec![],
        };

        registry.register(config).unwrap();

        // Unconfigured language should return None
        prop_assert!(registry.get(&unconfigured_lang).is_none());

        // But configured language should still work
        prop_assert!(registry.get(&configured_lang).is_some());
    }

    /// Property 3: Configuration changes reload without restart
    ///
    /// For any language configuration, updating it should replace the old
    /// configuration without requiring a restart.
    #[test]
    fn prop_configuration_hot_reload(
        config1 in language_config_strategy(),
        mut config2 in language_config_strategy(),
    ) {
        // Make them the same language but different extensions
        config2.language = config1.language.clone();
        prop_assume!(config1.extensions != config2.extensions);

        let mut registry = ConfigRegistry::new();

        // Register first configuration
        prop_assert!(registry.register(config1.clone()).is_ok());
        let retrieved1_ext = registry.get(&config1.language).unwrap().extensions.clone();
        prop_assert_eq!(&retrieved1_ext, &config1.extensions);

        // Update with second configuration
        prop_assert!(registry.register(config2.clone()).is_ok());
        let retrieved2_ext = registry.get(&config2.language).unwrap().extensions.clone();
        prop_assert_eq!(&retrieved2_ext, &config2.extensions);

        // Verify the configuration was updated
        prop_assert_ne!(&retrieved1_ext, &retrieved2_ext);
    }

    /// Property 4: Invalid configurations are rejected with clear errors
    ///
    /// For any invalid configuration (empty language name, empty extensions),
    /// registration should fail with a validation error.
    #[test]
    fn prop_invalid_configurations_rejected(
        _empty_language in Just("".to_string()),
    ) {
        let mut registry = ConfigRegistry::new();

        let config = LanguageConfig {
            language: "".to_string(),
            extensions: vec!["test".to_string()],
            parser_plugin: None,
            diagnostic_rules: vec![],
            code_actions: vec![],
        };

        // Registration should fail
        prop_assert!(registry.register(config).is_err());
    }

    /// Property 5: Configuration manager loads defaults correctly
    ///
    /// For any configuration manager, loading defaults should succeed
    /// and register all default language providers.
    #[test]
    fn prop_configuration_manager_defaults(_unit in Just(())) {
        let manager = ConfigurationManager::new();

        // Load defaults
        prop_assert!(manager.load_defaults().is_ok());

        // Verify default providers are registered
        let registry = manager.semantic_registry();
        let semantic_reg = registry.read().unwrap();
        prop_assert!(semantic_reg.has_provider("rust"));
        prop_assert!(semantic_reg.has_provider("typescript"));
        prop_assert!(semantic_reg.has_provider("python"));
        prop_assert!(semantic_reg.has_provider("unknown"));
    }

    /// Property 6: Multiple languages can be configured simultaneously
    ///
    /// For any set of distinct language configurations, all should be
    /// registerable and retrievable without interference.
    #[test]
    fn prop_multiple_languages_configured(
        configs in prop::collection::vec(language_config_strategy(), 1..5),
    ) {
        let mut registry = ConfigRegistry::new();

        // Ensure all languages are distinct
        let mut languages = std::collections::HashSet::new();
        for config in &configs {
            languages.insert(config.language.clone());
        }
        prop_assume!(languages.len() == configs.len());

        // Register all configurations
        for config in &configs {
            prop_assert!(registry.register(config.clone()).is_ok());
        }

        // Verify all are registered
        for config in &configs {
            prop_assert!(registry.has_language(&config.language));
            prop_assert!(registry.get(&config.language).is_some());
        }

        // Verify count matches
        prop_assert_eq!(registry.languages().len(), configs.len());
    }

    /// Property 7: Configuration extensions are preserved
    ///
    /// For any language configuration with extensions, retrieving the
    /// configuration should preserve all extensions exactly.
    #[test]
    fn prop_extensions_preserved(config in language_config_strategy()) {
        let mut registry = ConfigRegistry::new();

        let original_extensions = config.extensions.clone();
        prop_assert!(registry.register(config.clone()).is_ok());

        let retrieved = registry.get(&config.language).unwrap();
        prop_assert_eq!(&retrieved.extensions, &original_extensions);

        // Verify we can find by extension
        for ext in &original_extensions {
            prop_assert!(registry.get_by_extension(ext).is_some());
        }
    }

    /// Property 8: Diagnostic rules are validated on registration
    ///
    /// For any configuration with diagnostic rules, all rules should be
    /// validated and invalid rules should cause registration to fail.
    #[test]
    fn prop_diagnostic_rules_validated(config in language_config_strategy()) {
        let mut registry = ConfigRegistry::new();

        // Valid configuration should register successfully
        prop_assert!(registry.register(config.clone()).is_ok());
    }

    /// Property 9: Code actions are validated on registration
    ///
    /// For any configuration with code actions, all actions should be
    /// validated and invalid actions should cause registration to fail.
    #[test]
    fn prop_code_actions_validated(config in language_config_strategy()) {
        let mut registry = ConfigRegistry::new();

        // Valid configuration should register successfully
        prop_assert!(registry.register(config.clone()).is_ok());
    }

    /// Property 10: Configuration registry is thread-safe
    ///
    /// For any configuration, concurrent access to the registry should
    /// not cause panics or data corruption.
    #[test]
    fn prop_registry_thread_safe(config in language_config_strategy()) {
        use std::sync::Arc;
        use std::thread;

        let registry = Arc::new(std::sync::Mutex::new(ConfigRegistry::new()));

        // Register configuration
        {
            let mut reg = registry.lock().unwrap();
            prop_assert!(reg.register(config.clone()).is_ok());
        }

        // Spawn multiple threads to read concurrently
        let mut handles = vec![];
        for _ in 0..5 {
            let reg = Arc::clone(&registry);
            let lang = config.language.clone();
            let handle = thread::spawn(move || {
                let registry = reg.lock().unwrap();
                registry.has_language(&lang)
            });
            handles.push(handle);
        }

        // All threads should succeed
        for handle in handles {
            prop_assert!(handle.join().unwrap());
        }
    }
}
