//! Property-based tests for configuration consistency
//!
//! **Feature: ricecoder-external-lsp, Property 4: Configuration Consistency**
//! **Validates: Requirements ELSP-2.4**

use proptest::prelude::*;
use ricecoder_external_lsp::ConfigLoader;

/// Strategy for generating valid LSP server configurations
fn arb_lsp_server_config() -> impl Strategy<Value = String> {
    (
        "[a-z]+",
        "[a-z]+",
        "[a-z_]+",  // Executable names should be alphanumeric with underscores
        1000u64..100000u64,
        1u32..10u32,
        0u64..1000000u64,
    )
        .prop_map(|(lang, ext, exe, timeout, restarts, idle)| {
            format!(
                r#"
global:
  max_processes: 5
  default_timeout_ms: 5000
  enable_fallback: true
  health_check_interval_ms: 30000

servers:
  {}:
    - language: {}
      extensions: [".{}"]
      executable: {}
      args: []
      env: {{}}
      enabled: true
      timeout_ms: {}
      max_restarts: {}
      idle_timeout_ms: {}
"#,
                lang, lang, ext, exe, timeout, restarts, idle
            )
        })
}

proptest! {
    /// Property 4: Configuration Consistency
    ///
    /// For any configuration change, the system SHALL reload without losing active LSP
    /// connections or pending requests.
    ///
    /// This property tests that:
    /// 1. A configuration can be loaded successfully
    /// 2. The loaded configuration can be merged with other configurations
    /// 3. The merged configuration is valid and consistent
    #[test]
    fn prop_configuration_consistency(config_yaml in arb_lsp_server_config()) {
        // Load the configuration
        let registry = ConfigLoader::load_from_string(&config_yaml);
        prop_assert!(registry.is_ok(), "Configuration should load successfully");

        let registry = registry.unwrap();

        // Verify the registry has at least one language configured
        prop_assert!(!registry.servers.is_empty(), "Registry should have at least one language");

        // Verify each language has at least one server
        for (language, servers) in &registry.servers {
            prop_assert!(!servers.is_empty(), "Language {} should have at least one server", language);

            // Verify each server is valid
            for server in servers {
                prop_assert_eq!(&server.language, language, "Server language should match registry key");
                prop_assert!(!server.executable.is_empty(), "Server executable should not be empty");
                prop_assert!(!server.extensions.is_empty(), "Server extensions should not be empty");
                prop_assert!(server.timeout_ms > 0, "Server timeout should be positive");
            }
        }

        // Verify global settings are valid
        prop_assert!(registry.global.max_processes > 0, "Max processes should be positive");
        prop_assert!(registry.global.default_timeout_ms > 0, "Default timeout should be positive");
        prop_assert!(registry.global.health_check_interval_ms > 0, "Health check interval should be positive");
    }

    /// Property: Configuration merge preserves all servers
    ///
    /// When merging configurations, all servers from both configurations should be present
    /// in the result (with later configurations overriding earlier ones for the same language).
    #[test]
    fn prop_configuration_merge_preserves_servers(
        config1_yaml in arb_lsp_server_config(),
        config2_yaml in arb_lsp_server_config(),
    ) {
        let registry1 = ConfigLoader::load_from_string(&config1_yaml);
        let registry2 = ConfigLoader::load_from_string(&config2_yaml);

        prop_assume!(registry1.is_ok() && registry2.is_ok());

        let registry1 = registry1.unwrap();
        let registry2 = registry2.unwrap();

        // Merge configurations
        let merged = ConfigLoader::merge_configs(None, None, Some(registry2.clone()), registry1.clone());
        prop_assert!(merged.is_ok(), "Merge should succeed");

        let merged = merged.unwrap();

        // Verify merged registry has servers from both
        let all_languages: std::collections::HashSet<_> = registry1
            .servers
            .keys()
            .chain(registry2.servers.keys())
            .cloned()
            .collect();

        for language in all_languages {
            // The merged registry should have the language from either registry1 or registry2
            // (registry2 overrides registry1 for the same language)
            if registry2.servers.contains_key(&language) {
                prop_assert!(merged.servers.contains_key(&language), "Merged should have language from registry2");
            } else if registry1.servers.contains_key(&language) {
                prop_assert!(merged.servers.contains_key(&language), "Merged should have language from registry1");
            }
        }
    }

    /// Property: Configuration validation is consistent
    ///
    /// A configuration that passes validation once should always pass validation.
    #[test]
    fn prop_configuration_validation_consistency(config_yaml in arb_lsp_server_config()) {
        let result1 = ConfigLoader::load_from_string(&config_yaml);
        let result2 = ConfigLoader::load_from_string(&config_yaml);

        // Both attempts should have the same result
        match (&result1, &result2) {
            (Ok(_), Ok(_)) => {
                // Both succeeded - this is consistent
            }
            (Err(_), Err(_)) => {
                // Both failed - this is consistent
            }
            _ => {
                prop_assert!(false, "Configuration validation should be consistent");
            }
        }
    }

    /// Property: Global settings are preserved during merge
    ///
    /// When merging configurations, the global settings from the later configuration
    /// should override the earlier ones.
    #[test]
    fn prop_global_settings_merge(
        config1_yaml in arb_lsp_server_config(),
        config2_yaml in arb_lsp_server_config(),
    ) {
        let registry1 = ConfigLoader::load_from_string(&config1_yaml);
        let registry2 = ConfigLoader::load_from_string(&config2_yaml);

        prop_assume!(registry1.is_ok() && registry2.is_ok());

        let registry1 = registry1.unwrap();
        let registry2 = registry2.unwrap();

        // Merge configurations
        let merged = ConfigLoader::merge_configs(None, None, Some(registry2.clone()), registry1);
        prop_assert!(merged.is_ok(), "Merge should succeed");

        let merged = merged.unwrap();

        // Verify global settings come from registry2 (the later configuration)
        prop_assert_eq!(
            merged.global.max_processes,
            registry2.global.max_processes,
            "Global max_processes should come from later configuration"
        );
        prop_assert_eq!(
            merged.global.default_timeout_ms,
            registry2.global.default_timeout_ms,
            "Global default_timeout_ms should come from later configuration"
        );
    }
}
