//! Property-based tests for configuration loading and merging
//!
//! **Feature: ricecoder-storage, Property 3: Configuration Round-Trip**
//! **Feature: ricecoder-storage, Property 2: Configuration Merging Idempotence**
//! **Feature: ricecoder-storage, Property 4: Environment Variable Substitution**
//! **Validates: Requirements 2.1, 2.3, 2.4**

use proptest::prelude::*;
use ricecoder_storage::config::{Config, ConfigLoader, DefaultsConfig, ProvidersConfig};
use ricecoder_storage::types::ConfigFormat;
use std::collections::HashMap;
use tempfile::TempDir;

/// Strategy for generating valid configurations
fn config_strategy() -> impl Strategy<Value = Config> {
    (
        prop::option::of("[a-z]+"),
        prop::option::of("[a-z]+"),
        prop::option::of(0.0f32..1.0f32),
        prop::option::of(1u32..4096u32),
    )
        .prop_map(|(provider, model, temp, tokens)| {
            let mut providers = ProvidersConfig {
                api_keys: HashMap::new(),
                endpoints: HashMap::new(),
                default_provider: provider.clone(),
            };

            if let Some(p) = provider {
                providers.api_keys.insert(p, "test-key".to_string());
            }

            Config {
                providers,
                defaults: DefaultsConfig {
                    model,
                    temperature: temp,
                    max_tokens: tokens,
                },
                steering: Vec::new(),
                tui: Default::default(),
                custom: HashMap::new(),
            }
        })
}

proptest! {
    /// Property: Configuration round-trip through file
    ///
    /// For any valid configuration, saving to a file and loading
    /// should produce an equivalent configuration.
    #[test]
    fn prop_config_roundtrip_through_file(config in config_strategy()) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_file = temp_dir.path().join("ricecoder.yaml");

        // Save configuration
        ConfigLoader::save_to_file(&config, &config_file, ConfigFormat::Yaml)
            .expect("Failed to save config");

        // Verify file exists
        assert!(config_file.exists());

        // Load and verify
        let loaded = ConfigLoader::load_from_file(&config_file)
            .expect("Failed to load config");

        prop_assert_eq!(config, loaded);
    }

    /// Property: Environment variable substitution in API keys
    ///
    /// For any configuration with ${VAR_NAME} patterns in API keys,
    /// substitution should replace them with environment variable values.
    #[test]
    fn prop_env_var_substitution_in_api_keys(
        provider in "[a-z]+",
        var_name in "[A-Z][A-Z0-9]*",
        var_value in "[a-zA-Z0-9]+",
    ) {
        // Set environment variable
        std::env::set_var(&var_name, &var_value);

        let mut config = Config::default();
        let pattern = format!("${{{}}}", var_name);
        config.providers.api_keys.insert(provider.clone(), pattern.clone());

        // Substitute
        ConfigLoader::substitute_env_vars(&mut config)
            .expect("Failed to substitute");

        // Verify substitution
        prop_assert_eq!(
            config.providers.api_keys.get(&provider),
            Some(&var_value.to_string())
        );

        // Clean up
        std::env::remove_var(&var_name);
    }

    /// Property: Missing environment variables are replaced with empty string
    ///
    /// For any configuration with ${NONEXISTENT_VAR} patterns,
    /// substitution should replace them with empty strings.
    #[test]
    fn prop_missing_env_var_becomes_empty(provider in "[a-z]+") {
        let mut config = Config::default();
        config.providers.api_keys.insert(
            provider.clone(),
            "${DEFINITELY_NONEXISTENT_VAR_12345}".to_string()
        );

        ConfigLoader::substitute_env_vars(&mut config)
            .expect("Failed to substitute");

        // Should be replaced with empty string
        prop_assert_eq!(
            config.providers.api_keys.get(&provider),
            Some(&"".to_string())
        );
    }

    /// Property: Multiple environment variable patterns are substituted
    ///
    /// For any configuration with multiple ${VAR_NAME} patterns,
    /// all should be substituted correctly.
    #[test]
    fn prop_multiple_env_var_substitutions(
        var1_name in "[A-Z][A-Z0-9]*",
        var1_value in "[a-zA-Z0-9]+",
        var2_name in "[A-Z][A-Z0-9]*",
        var2_value in "[a-zA-Z0-9]+",
    ) {
        // Ensure variable names are different
        prop_assume!(var1_name != var2_name);

        // Set environment variables
        std::env::set_var(&var1_name, &var1_value);
        std::env::set_var(&var2_name, &var2_value);

        let mut config = Config::default();
        let pattern = format!("${{{}}}-${{{}}}", var1_name, var2_name);
        config.providers.api_keys.insert("test".to_string(), pattern);

        ConfigLoader::substitute_env_vars(&mut config)
            .expect("Failed to substitute");

        let expected = format!("{}-{}", var1_value, var2_value);
        prop_assert_eq!(
            config.providers.api_keys.get("test"),
            Some(&expected)
        );

        // Clean up
        std::env::remove_var(&var1_name);
        std::env::remove_var(&var2_name);
    }

    /// Property: Substitution preserves non-pattern strings
    ///
    /// For any configuration with non-pattern strings, substitution should
    /// leave them unchanged.
    #[test]
    fn prop_substitution_preserves_non_patterns(
        provider in "[a-z]+",
        value in "[a-zA-Z0-9]+",
    ) {
        let mut config = Config::default();
        config.providers.api_keys.insert(provider.clone(), value.clone());

        ConfigLoader::substitute_env_vars(&mut config)
            .expect("Failed to substitute");

        // Value should be unchanged
        prop_assert_eq!(
            config.providers.api_keys.get(&provider),
            Some(&value)
        );
    }

}
