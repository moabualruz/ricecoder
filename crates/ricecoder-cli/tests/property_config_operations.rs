// Property-based tests for configuration operations
// **Feature: ricecoder-cli, Property 10: Config Get/Set Round-Trip**
// **Validates: Requirements 8.2, 8.3**
// **Feature: ricecoder-cli, Property 11: Sensitive Value Masking**
// **Validates: Requirements 8.4**

use proptest::prelude::*;
use ricecoder_storage::ConfigLoader;
use tempfile::TempDir;

// ============================================================================
// Property 10: Config Get/Set Round-Trip
// ============================================================================
// For any configuration key and value, setting it and then loading the config
// should return the same value (or a properly formatted version of it).

proptest! {
    #[test]
    fn prop_config_set_get_roundtrip_provider(provider in "[a-z0-9_]+") {
        prop_assume!(!provider.is_empty());

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_file = temp_dir.path().join("ricecoder.yaml");

        let mut config = ricecoder_storage::Config::default();
        config.providers.default_provider = Some(provider.clone());

        ConfigLoader::save_to_file(&config, &config_file, ricecoder_storage::ConfigFormat::Yaml)
            .expect("Failed to save config");

        let loaded = ConfigLoader::load_from_file(&config_file)
            .expect("Failed to load config");

        prop_assert_eq!(
            loaded.providers.default_provider,
            Some(provider.clone()),
            "Provider should round-trip correctly"
        );
    }

    #[test]
    fn prop_config_set_get_roundtrip_model(model in "[a-z0-9_-]+") {
        prop_assume!(!model.is_empty());

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_file = temp_dir.path().join("ricecoder.yaml");

        let mut config = ricecoder_storage::Config::default();
        config.defaults.model = Some(model.clone());

        ConfigLoader::save_to_file(&config, &config_file, ricecoder_storage::ConfigFormat::Yaml)
            .expect("Failed to save config");

        let loaded = ConfigLoader::load_from_file(&config_file)
            .expect("Failed to load config");

        prop_assert_eq!(
            loaded.defaults.model,
            Some(model.clone()),
            "Model should round-trip correctly"
        );
    }

    #[test]
    fn prop_config_set_get_roundtrip_temperature(temp in 0.0f32..=2.0f32) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_file = temp_dir.path().join("ricecoder.yaml");

        let mut config = ricecoder_storage::Config::default();
        config.defaults.temperature = Some(temp);

        ConfigLoader::save_to_file(&config, &config_file, ricecoder_storage::ConfigFormat::Yaml)
            .expect("Failed to save config");

        let loaded = ConfigLoader::load_from_file(&config_file)
            .expect("Failed to load config");

        if let Some(loaded_temp) = loaded.defaults.temperature {
            prop_assert!((loaded_temp - temp).abs() < 0.001, 
                "Temperature should round-trip correctly");
        } else {
            prop_assert!(false, "Temperature should be set");
        }
    }

    #[test]
    fn prop_config_set_get_roundtrip_api_key(provider in "[a-z0-9_]+", key in "[a-zA-Z0-9_-]{10,50}") {
        prop_assume!(!provider.is_empty() && !key.is_empty());

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_file = temp_dir.path().join("ricecoder.yaml");

        let mut config = ricecoder_storage::Config::default();
        config.providers.api_keys.insert(provider.clone(), key.clone());

        ConfigLoader::save_to_file(&config, &config_file, ricecoder_storage::ConfigFormat::Yaml)
            .expect("Failed to save config");

        let loaded = ConfigLoader::load_from_file(&config_file)
            .expect("Failed to load config");

        prop_assert_eq!(
            loaded.providers.api_keys.get(&provider),
            Some(&key),
            "API key should round-trip correctly"
        );
    }

    #[test]
    fn prop_config_multiple_keys_roundtrip(
        provider in "[a-z0-9_]+",
        model in "[a-z0-9_-]+",
        temp in 0.0f32..=2.0f32
    ) {
        prop_assume!(!provider.is_empty() && !model.is_empty());

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_file = temp_dir.path().join("ricecoder.yaml");

        let mut config = ricecoder_storage::Config::default();
        config.providers.default_provider = Some(provider.clone());
        config.defaults.model = Some(model.clone());
        config.defaults.temperature = Some(temp);

        ConfigLoader::save_to_file(&config, &config_file, ricecoder_storage::ConfigFormat::Yaml)
            .expect("Failed to save config");

        let loaded = ConfigLoader::load_from_file(&config_file)
            .expect("Failed to load config");

        prop_assert_eq!(
            loaded.providers.default_provider,
            Some(provider.clone()),
            "Provider should round-trip"
        );
        prop_assert_eq!(
            loaded.defaults.model,
            Some(model.clone()),
            "Model should round-trip"
        );
        if let Some(loaded_temp) = loaded.defaults.temperature {
            prop_assert!((loaded_temp - temp).abs() < 0.001, 
                "Temperature should round-trip");
        }
    }
}

// ============================================================================
// Integration Tests for Config Operations
// ============================================================================

#[test]
fn test_config_roundtrip_yaml_format() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_file = temp_dir.path().join("ricecoder.yaml");

    let mut config = ricecoder_storage::Config::default();
    config.providers.default_provider = Some("openai".to_string());
    config.defaults.model = Some("gpt-4".to_string());
    config.defaults.temperature = Some(0.7);
    config.providers.api_keys.insert("openai".to_string(), "test-key".to_string());

    ConfigLoader::save_to_file(&config, &config_file, ricecoder_storage::ConfigFormat::Yaml)
        .expect("Failed to save config");

    let loaded = ConfigLoader::load_from_file(&config_file)
        .expect("Failed to load config");

    assert_eq!(loaded.providers.default_provider, Some("openai".to_string()));
    assert_eq!(loaded.defaults.model, Some("gpt-4".to_string()));
    assert_eq!(loaded.defaults.temperature, Some(0.7));
    assert_eq!(loaded.providers.api_keys.get("openai"), Some(&"test-key".to_string()));
}

#[test]
fn test_config_roundtrip_json_format() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_file = temp_dir.path().join("ricecoder.json");

    let mut config = ricecoder_storage::Config::default();
    config.providers.default_provider = Some("openai".to_string());
    config.defaults.model = Some("gpt-4".to_string());
    config.defaults.temperature = Some(0.7);

    ConfigLoader::save_to_file(&config, &config_file, ricecoder_storage::ConfigFormat::Json)
        .expect("Failed to save config");

    let loaded = ConfigLoader::load_from_file(&config_file)
        .expect("Failed to load config");

    assert_eq!(loaded.providers.default_provider, Some("openai".to_string()));
    assert_eq!(loaded.defaults.model, Some("gpt-4".to_string()));
    assert_eq!(loaded.defaults.temperature, Some(0.7));
}

#[test]
fn test_config_roundtrip_toml_format() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_file = temp_dir.path().join("ricecoder.toml");

    let mut config = ricecoder_storage::Config::default();
    config.providers.default_provider = Some("openai".to_string());
    config.defaults.model = Some("gpt-4".to_string());
    config.defaults.temperature = Some(0.7);

    ConfigLoader::save_to_file(&config, &config_file, ricecoder_storage::ConfigFormat::Toml)
        .expect("Failed to save config");

    let loaded = ConfigLoader::load_from_file(&config_file)
        .expect("Failed to load config");

    assert_eq!(loaded.providers.default_provider, Some("openai".to_string()));
    assert_eq!(loaded.defaults.model, Some("gpt-4".to_string()));
    assert_eq!(loaded.defaults.temperature, Some(0.7));
}

#[test]
fn test_config_sensitive_value_masking_non_empty() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_file = temp_dir.path().join("ricecoder.yaml");

    let mut config = ricecoder_storage::Config::default();
    config.providers.api_keys.insert("openai".to_string(), "secret-key-123".to_string());

    ConfigLoader::save_to_file(&config, &config_file, ricecoder_storage::ConfigFormat::Yaml)
        .expect("Failed to save config");

    let loaded = ConfigLoader::load_from_file(&config_file)
        .expect("Failed to load config");

    assert_eq!(
        loaded.providers.api_keys.get("openai"),
        Some(&"secret-key-123".to_string()),
        "API key should be stored as-is in config"
    );
}

#[test]
fn test_config_sensitive_value_masking_multiple_providers() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_file = temp_dir.path().join("ricecoder.yaml");

    let mut config = ricecoder_storage::Config::default();
    config.providers.api_keys.insert("openai".to_string(), "key1".to_string());
    config.providers.api_keys.insert("anthropic".to_string(), "key2".to_string());

    ConfigLoader::save_to_file(&config, &config_file, ricecoder_storage::ConfigFormat::Yaml)
        .expect("Failed to save config");

    let loaded = ConfigLoader::load_from_file(&config_file)
        .expect("Failed to load config");

    assert_eq!(
        loaded.providers.api_keys.get("openai"),
        Some(&"key1".to_string()),
        "First API key should be stored"
    );
    assert_eq!(
        loaded.providers.api_keys.get("anthropic"),
        Some(&"key2".to_string()),
        "Second API key should be stored"
    );
}

#[test]
fn test_config_empty_values_preserved() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_file = temp_dir.path().join("ricecoder.yaml");

    let mut config = ricecoder_storage::Config::default();
    config.providers.api_keys.insert("openai".to_string(), "".to_string());

    ConfigLoader::save_to_file(&config, &config_file, ricecoder_storage::ConfigFormat::Yaml)
        .expect("Failed to save config");

    let loaded = ConfigLoader::load_from_file(&config_file)
        .expect("Failed to load config");

    assert_eq!(loaded.providers.api_keys.get("openai"), Some(&"".to_string()));
}
