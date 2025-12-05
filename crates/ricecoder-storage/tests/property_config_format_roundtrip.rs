//! Property-based tests for configuration format round-trip
//!
//! **Feature: ricecoder-storage, Property 3: Configuration Format Round-Trip**
//! **Validates: Requirements 4.1, 4.2, 4.3**

use proptest::prelude::*;
use ricecoder_storage::config::{Config, ConfigLoader, DefaultsConfig, ProvidersConfig};
use ricecoder_storage::types::ConfigFormat;
use std::collections::HashMap;

/// Strategy for generating valid configurations
fn config_strategy() -> impl Strategy<Value = Config> {
    (
        prop::option::of("[a-z_]+"),
        prop::option::of("[a-z_]+"),
        prop::option::of(0.0f32..1.0f32),
        prop::option::of(1u32..4096u32),
    )
        .prop_map(|(provider, model, temp, tokens)| {
            let mut providers = ProvidersConfig {
                api_keys: HashMap::new(),
                endpoints: HashMap::new(),
                default_provider: provider.clone(),
            };

            // Add some API keys
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
                custom: HashMap::new(),
            }
        })
}

proptest! {
    /// Property: YAML round-trip preserves configuration
    ///
    /// For any valid configuration, serializing to YAML and deserializing
    /// should produce an equivalent configuration.
    #[test]
    fn prop_yaml_roundtrip(config in config_strategy()) {
        let yaml = ConfigLoader::serialize(&config, ConfigFormat::Yaml)
            .expect("Failed to serialize to YAML");
        let deserialized = ConfigLoader::load_from_string(&yaml, ConfigFormat::Yaml, "test.yaml")
            .expect("Failed to deserialize from YAML");

        prop_assert_eq!(config, deserialized);
    }

    /// Property: TOML round-trip preserves configuration
    ///
    /// For any valid configuration, serializing to TOML and deserializing
    /// should produce an equivalent configuration.
    #[test]
    fn prop_toml_roundtrip(config in config_strategy()) {
        let toml = ConfigLoader::serialize(&config, ConfigFormat::Toml)
            .expect("Failed to serialize to TOML");
        let deserialized = ConfigLoader::load_from_string(&toml, ConfigFormat::Toml, "test.toml")
            .expect("Failed to deserialize from TOML");

        prop_assert_eq!(config, deserialized);
    }

    /// Property: JSON round-trip preserves configuration
    ///
    /// For any valid configuration, serializing to JSON and deserializing
    /// should produce an equivalent configuration.
    #[test]
    fn prop_json_roundtrip(config in config_strategy()) {
        let json = ConfigLoader::serialize(&config, ConfigFormat::Json)
            .expect("Failed to serialize to JSON");
        let deserialized = ConfigLoader::load_from_string(&json, ConfigFormat::Json, "test.json")
            .expect("Failed to deserialize from JSON");

        prop_assert_eq!(config, deserialized);
    }

    /// Property: All formats produce equivalent configurations
    ///
    /// For any valid configuration, serializing to different formats and
    /// deserializing should produce equivalent configurations.
    #[test]
    fn prop_format_equivalence(config in config_strategy()) {
        let yaml = ConfigLoader::serialize(&config, ConfigFormat::Yaml)
            .expect("Failed to serialize to YAML");
        let toml = ConfigLoader::serialize(&config, ConfigFormat::Toml)
            .expect("Failed to serialize to TOML");
        let json = ConfigLoader::serialize(&config, ConfigFormat::Json)
            .expect("Failed to serialize to JSON");

        let from_yaml = ConfigLoader::load_from_string(&yaml, ConfigFormat::Yaml, "test.yaml")
            .expect("Failed to deserialize from YAML");
        let from_toml = ConfigLoader::load_from_string(&toml, ConfigFormat::Toml, "test.toml")
            .expect("Failed to deserialize from TOML");
        let from_json = ConfigLoader::load_from_string(&json, ConfigFormat::Json, "test.json")
            .expect("Failed to deserialize from JSON");

        prop_assert_eq!(from_yaml.clone(), from_toml.clone());
        prop_assert_eq!(from_toml.clone(), from_json.clone());
        prop_assert_eq!(from_yaml, from_json);
    }
}
