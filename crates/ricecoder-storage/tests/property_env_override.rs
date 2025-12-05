//! Property-based tests for environment variable override
//!
//! **Feature: ricecoder-storage, Property 8: Environment Variable Override**
//! **Validates: Requirements 4.4**

use proptest::prelude::*;
use ricecoder_storage::config::{Config, EnvOverrides};
use std::collections::HashMap;

/// Strategy for generating valid environment variable overrides
fn env_override_strategy() -> impl Strategy<Value = HashMap<String, String>> {
    prop::collection::hash_map(r"[a-z_]+", r"[a-zA-Z0-9_\-\.]+", 0..5)
}

proptest! {
    /// Property: Environment variables override configuration values
    ///
    /// For any environment variable override, the configuration value should be
    /// overridden by the environment variable value.
    #[test]
    fn prop_env_overrides_config(overrides in env_override_strategy()) {
        let mut config = Config::default();
        config.defaults.model = Some("gpt-4".to_string());
        config.providers.default_provider = Some("openai".to_string());

        EnvOverrides::apply_overrides(&mut config, &overrides);

        // Verify that overrides were applied
        for (key, value) in &overrides {
            if key == "defaults.model" {
                assert_eq!(config.defaults.model, Some(value.clone()));
            }
            if key == "providers.default_provider" {
                assert_eq!(config.providers.default_provider, Some(value.clone()));
            }
        }
    }

    /// Property: Provider default override works
    ///
    /// For any provider name, setting RICECODER_PROVIDERS_DEFAULT_PROVIDER should
    /// override the default provider.
    #[test]
    fn prop_provider_default_override(provider_name in "[a-z_]+") {
        let mut config = Config::default();
        let mut overrides = HashMap::new();
        overrides.insert("providers.default_provider".to_string(), provider_name.clone());

        EnvOverrides::apply_overrides(&mut config, &overrides);

        assert_eq!(config.providers.default_provider, Some(provider_name));
    }

    /// Property: Model default override works
    ///
    /// For any model name, setting RICECODER_DEFAULTS_MODEL should override the model.
    #[test]
    fn prop_model_default_override(model_name in "[a-z_]+") {
        let mut config = Config::default();
        let mut overrides = HashMap::new();
        overrides.insert("defaults.model".to_string(), model_name.clone());

        EnvOverrides::apply_overrides(&mut config, &overrides);

        assert_eq!(config.defaults.model, Some(model_name));
    }

    /// Property: Temperature override works
    ///
    /// For any valid temperature value, setting RICECODER_DEFAULTS_TEMPERATURE should
    /// override the temperature.
    #[test]
    fn prop_temperature_override(temp in 0.0f32..1.0f32) {
        let mut config = Config::default();
        let mut overrides = HashMap::new();
        overrides.insert("defaults.temperature".to_string(), temp.to_string());

        EnvOverrides::apply_overrides(&mut config, &overrides);

        assert_eq!(config.defaults.temperature, Some(temp));
    }

    /// Property: Max tokens override works
    ///
    /// For any valid max tokens value, setting RICECODER_DEFAULTS_MAX_TOKENS should
    /// override the max tokens.
    #[test]
    fn prop_max_tokens_override(tokens in 1u32..4096u32) {
        let mut config = Config::default();
        let mut overrides = HashMap::new();
        overrides.insert("defaults.max_tokens".to_string(), tokens.to_string());

        EnvOverrides::apply_overrides(&mut config, &overrides);

        assert_eq!(config.defaults.max_tokens, Some(tokens));
    }

    /// Property: API key override works
    ///
    /// For any provider and API key, setting RICECODER_PROVIDERS_API_KEYS_<PROVIDER>
    /// should override the API key.
    #[test]
    fn prop_api_key_override(provider in "[a-z_]+", api_key in "[a-zA-Z0-9_\\-]+") {
        let mut config = Config::default();
        let mut overrides = HashMap::new();
        let key = format!("providers.api_keys.{}", provider);
        overrides.insert(key, api_key.clone());

        EnvOverrides::apply_overrides(&mut config, &overrides);

        assert_eq!(config.providers.api_keys.get(&provider), Some(&api_key));
    }

    /// Property: Multiple overrides work together
    ///
    /// For any combination of overrides, all should be applied correctly.
    #[test]
    fn prop_multiple_overrides_work(
        provider in "[a-z_]+",
        model in "[a-z_]+",
        temp in 0.0f32..1.0f32,
    ) {
        let mut config = Config::default();
        let mut overrides = HashMap::new();
        overrides.insert("providers.default_provider".to_string(), provider.clone());
        overrides.insert("defaults.model".to_string(), model.clone());
        overrides.insert("defaults.temperature".to_string(), temp.to_string());

        EnvOverrides::apply_overrides(&mut config, &overrides);

        assert_eq!(config.providers.default_provider, Some(provider));
        assert_eq!(config.defaults.model, Some(model));
        assert_eq!(config.defaults.temperature, Some(temp));
    }

    /// Property: Unknown overrides are stored in custom map
    ///
    /// For any unknown configuration path, the override should be stored in the
    /// custom map.
    #[test]
    fn prop_unknown_overrides_stored(path in "[a-z_]+\\.[a-z_]+", value in "[a-zA-Z0-9_\\-]+") {
        let mut config = Config::default();
        let mut overrides = HashMap::new();
        overrides.insert(path.clone(), value.clone());

        EnvOverrides::apply_overrides(&mut config, &overrides);

        // Unknown paths should be stored in custom map
        if !path.starts_with("providers") && !path.starts_with("defaults") {
            assert!(config.custom.contains_key(&path));
        }
    }
}
