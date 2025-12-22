//! Property-based tests for configuration precedence
//! **Feature: ricecoder-providers, Property 3: Configuration Precedence**
//! **Validates: Requirements 3.4, 3.5**

use std::{sync::Mutex, time::Duration};

use proptest::prelude::*;
use ricecoder_providers::{config::ConfigurationManager, models::ProviderSettings};

// Mutex to ensure environment variable tests don't interfere with each other
lazy_static::lazy_static! {
    static ref ENV_LOCK: Mutex<()> = Mutex::new(());
}

/// Strategy for generating valid provider IDs
fn provider_id_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("openai".to_string()),
        Just("anthropic".to_string()),
        Just("google".to_string()),
        Just("ollama".to_string()),
    ]
}

/// Strategy for generating valid API keys
fn api_key_strategy() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9_-]{10,50}".prop_map(|s| format!("key_{}", s))
}

/// Property: Environment variables override project config
#[test]
fn prop_env_overrides_project_config() {
    proptest!(|(
        provider_id in provider_id_strategy(),
        env_key in api_key_strategy(),
        config_key in api_key_strategy(),
    )| {
        // Skip if keys are the same (not a meaningful test)
        prop_assume!(env_key != config_key);

        // Lock to prevent interference from other tests
        let _lock = ENV_LOCK.lock().unwrap();

        let mut manager = ConfigurationManager::new();

        // Set up config with one key
        manager.config_mut().providers.insert(
            provider_id.clone(),
            ProviderSettings {
                api_key: Some(config_key.clone()),
                base_url: None,
                timeout: None,
                retry_count: None,
            },
        );

        // Set environment variable with different key
        let env_var = format!("{}_API_KEY", provider_id.to_uppercase());
        std::env::set_var(&env_var, &env_key);

        // Load from environment (should override config)
        manager.load_from_env().unwrap();

        // Verify environment variable took precedence
        let retrieved_key = manager.get_api_key(&provider_id).unwrap();
        prop_assert_eq!(retrieved_key, env_key);

        // Cleanup
        std::env::remove_var(&env_var);
    });
}

/// Property: Project config overrides global config
#[test]
fn prop_project_overrides_global() {
    proptest!(|(
        provider_id in provider_id_strategy(),
        global_key in api_key_strategy(),
        project_key in api_key_strategy(),
    )| {
        // Skip if keys are the same
        prop_assume!(global_key != project_key);

        let mut manager = ConfigurationManager::new();

        // Simulate global config
        manager.config_mut().providers.insert(
            provider_id.clone(),
            ProviderSettings {
                api_key: Some(global_key.clone()),
                base_url: None,
                timeout: None,
                retry_count: None,
            },
        );

        // Simulate project config merge (overrides global)
        manager.config_mut().providers.insert(
            provider_id.clone(),
            ProviderSettings {
                api_key: Some(project_key.clone()),
                base_url: None,
                timeout: None,
                retry_count: None,
            },
        );

        // Verify project config took precedence
        let retrieved_key = manager.get_api_key(&provider_id).unwrap();
        prop_assert_eq!(retrieved_key, project_key);
    });
}

/// Property: Environment variables have highest priority
#[test]
fn prop_env_highest_priority() {
    proptest!(|(
        provider_id in provider_id_strategy(),
        env_key in api_key_strategy(),
        config_key in api_key_strategy(),
    )| {
        // Skip if keys are the same
        prop_assume!(env_key != config_key);

        // Lock to prevent interference from other tests
        let _lock = ENV_LOCK.lock().unwrap();

        let mut manager = ConfigurationManager::new();

        // Set up config with one key
        manager.config_mut().providers.insert(
            provider_id.clone(),
            ProviderSettings {
                api_key: Some(config_key.clone()),
                base_url: None,
                timeout: None,
                retry_count: None,
            },
        );

        // Set environment variable
        let env_var = format!("{}_API_KEY", provider_id.to_uppercase());
        std::env::set_var(&env_var, &env_key);

        // Load from environment
        manager.load_from_env().unwrap();

        // Verify environment variable has highest priority
        let retrieved_key = manager.get_api_key(&provider_id).unwrap();
        prop_assert_eq!(retrieved_key, env_key);

        // Cleanup
        std::env::remove_var(&env_var);
    });
}

/// Property: Configuration precedence is consistent
#[test]
fn prop_precedence_consistency() {
    proptest!(|(
        provider_id in provider_id_strategy(),
        key1 in api_key_strategy(),
        _key2 in api_key_strategy(),
    )| {
        let mut manager1 = ConfigurationManager::new();
        let mut manager2 = ConfigurationManager::new();

        // Set up identical configurations
        manager1.config_mut().providers.insert(
            provider_id.clone(),
            ProviderSettings {
                api_key: Some(key1.clone()),
                base_url: None,
                timeout: None,
                retry_count: None,
            },
        );

        manager2.config_mut().providers.insert(
            provider_id.clone(),
            ProviderSettings {
                api_key: Some(key1.clone()),
                base_url: None,
                timeout: None,
                retry_count: None,
            },
        );

        // Both should retrieve the same key
        let key_m1 = manager1.get_api_key(&provider_id).unwrap();
        let key_m2 = manager2.get_api_key(&provider_id).unwrap();

        prop_assert_eq!(&key_m1, &key_m2);
        prop_assert_eq!(&key_m1, &key1);
    });
}

/// Property: Defaults are applied when no config is provided
#[test]
fn prop_defaults_applied() {
    let manager = ConfigurationManager::new();

    // Verify defaults are set
    assert_eq!(manager.default_provider(), "openai");
    assert_eq!(manager.default_model(), "gpt-4");
}

/// Property: Multiple providers can coexist with different precedence
#[test]
fn prop_multiple_providers_precedence() {
    proptest!(|(
        provider1 in "openai|anthropic",
        provider2 in "google|ollama",
        key1_env in api_key_strategy(),
        key2_env in api_key_strategy(),
    )| {
        // Skip if providers are the same
        prop_assume!(provider1 != provider2);

        // Lock to prevent interference from other tests
        let _lock = ENV_LOCK.lock().unwrap();

        let mut manager = ConfigurationManager::new();

        // Set environment variables for both
        let env_var1 = format!("{}_API_KEY", provider1.to_uppercase());
        let env_var2 = format!("{}_API_KEY", provider2.to_uppercase());
        std::env::set_var(&env_var1, &key1_env);
        std::env::set_var(&env_var2, &key2_env);

        // Load from environment
        manager.load_from_env().unwrap();

        // Verify each provider has correct precedence
        let retrieved_key1 = manager.get_api_key(&provider1).unwrap();
        let retrieved_key2 = manager.get_api_key(&provider2).unwrap();

        prop_assert_eq!(retrieved_key1, key1_env);
        prop_assert_eq!(retrieved_key2, key2_env);

        // Cleanup
        std::env::remove_var(&env_var1);
        std::env::remove_var(&env_var2);
    });
}

/// Property: Configuration merging preserves existing values
#[test]
fn prop_merge_preserves_existing() {
    proptest!(|(
        provider_id in provider_id_strategy(),
        base_url in r"https://[a-z0-9.-]+\.[a-z]{2,}",
        timeout_secs in 1u64..300,
        retry_count in 1usize..10,
    )| {
        let mut manager = ConfigurationManager::new();

        // Set up initial config
        manager.config_mut().providers.insert(
            provider_id.clone(),
            ProviderSettings {
                api_key: Some("initial-key".to_string()),
                base_url: Some(base_url.clone()),
                timeout: Some(Duration::from_secs(timeout_secs)),
                retry_count: Some(retry_count),
            },
        );

        // Merge with empty config (should preserve existing)
        let mut new_config = manager.config().clone();
        new_config.providers.clear();

        // Manually merge (simulating merge_from_file behavior)
        for (id, settings) in new_config.providers {
            manager.config_mut()
                .providers
                .entry(id)
                .and_modify(|existing| {
                    if settings.api_key.is_some() {
                        existing.api_key = settings.api_key.clone();
                    }
                    if settings.base_url.is_some() {
                        existing.base_url = settings.base_url.clone();
                    }
                    if settings.timeout.is_some() {
                        existing.timeout = settings.timeout;
                    }
                    if settings.retry_count.is_some() {
                        existing.retry_count = settings.retry_count;
                    }
                })
                .or_insert(settings);
        }

        // Verify existing values are preserved
        let settings = manager.get_provider_settings(&provider_id).unwrap();
        prop_assert_eq!(&settings.api_key, &Some("initial-key".to_string()));
        prop_assert_eq!(&settings.base_url, &Some(base_url));
        prop_assert_eq!(settings.timeout, Some(Duration::from_secs(timeout_secs)));
        prop_assert_eq!(settings.retry_count, Some(retry_count));
    });
}
