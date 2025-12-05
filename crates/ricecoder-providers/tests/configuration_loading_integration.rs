//! Integration tests for configuration loading from multiple sources
//! Tests loading from all configuration sources and precedence

use ricecoder_providers::config::ConfigurationManager;
use ricecoder_providers::models::{DefaultsConfig, ProviderConfig, ProviderSettings};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

/// Test: Loading configuration with proper precedence
#[test]
fn test_configuration_precedence_env_overrides_project() {
    let mut manager = ConfigurationManager::new();

    // Set up initial config with project settings
    manager.config_mut().providers.insert(
        "openai".to_string(),
        ProviderSettings {
            api_key: Some("project-key".to_string()),
            base_url: None,
            timeout: None,
            retry_count: None,
        },
    );

    // Set environment variable (should override)
    std::env::set_var("OPENAI_API_KEY", "env-key");

    manager.load_from_env().unwrap();

    // Environment variable should override project config
    let settings = manager.get_provider_settings("openai").unwrap();
    assert_eq!(settings.api_key, Some("env-key".to_string()));

    // Cleanup
    std::env::remove_var("OPENAI_API_KEY");
}

/// Test: Loading configuration from environment variables
#[test]
#[serial_test::serial]
fn test_load_from_environment_variables() {
    let mut manager = ConfigurationManager::new();

    // Set multiple environment variables
    std::env::set_var("OPENAI_API_KEY", "openai-key-123");
    std::env::set_var("ANTHROPIC_API_KEY", "anthropic-key-456");
    std::env::set_var("GOOGLE_API_KEY", "google-key-789");

    manager.load_from_env().unwrap();

    // Verify all keys were loaded
    let openai_settings = manager.get_provider_settings("openai");
    assert!(
        openai_settings.is_some(),
        "OpenAI provider settings should exist"
    );
    assert_eq!(
        openai_settings.unwrap().api_key,
        Some("openai-key-123".to_string())
    );

    let anthropic_settings = manager.get_provider_settings("anthropic");
    assert!(
        anthropic_settings.is_some(),
        "Anthropic provider settings should exist"
    );
    assert_eq!(
        anthropic_settings.unwrap().api_key,
        Some("anthropic-key-456".to_string())
    );

    let google_settings = manager.get_provider_settings("google");
    assert!(
        google_settings.is_some(),
        "Google provider settings should exist"
    );
    assert_eq!(
        google_settings.unwrap().api_key,
        Some("google-key-789".to_string())
    );

    // Cleanup
    std::env::remove_var("OPENAI_API_KEY");
    std::env::remove_var("ANTHROPIC_API_KEY");
    std::env::remove_var("GOOGLE_API_KEY");
}

/// Test: Loading RICECODER_PROVIDER_* environment variables
#[test]
fn test_load_ricecoder_provider_env_variables() {
    let mut manager = ConfigurationManager::new();

    // Set RICECODER_PROVIDER_* environment variables
    std::env::set_var("RICECODER_PROVIDER_CUSTOM", "custom-key-123");
    std::env::set_var("RICECODER_PROVIDER_ANOTHER", "another-key-456");

    manager.load_from_env().unwrap();

    // Verify custom providers were loaded
    assert_eq!(
        manager.get_provider_settings("custom").unwrap().api_key,
        Some("custom-key-123".to_string())
    );
    assert_eq!(
        manager.get_provider_settings("another").unwrap().api_key,
        Some("another-key-456".to_string())
    );

    // Cleanup
    std::env::remove_var("RICECODER_PROVIDER_CUSTOM");
    std::env::remove_var("RICECODER_PROVIDER_ANOTHER");
}

/// Test: Configuration paths are correct
#[test]
fn test_configuration_paths() {
    let global_path = ConfigurationManager::get_global_config_path();
    let project_path = ConfigurationManager::get_project_config_path();

    // Global path should contain .ricecoder
    assert!(global_path.to_string_lossy().contains(".ricecoder"));
    assert!(global_path.to_string_lossy().contains("config.yaml"));

    // Project path should be ./.agent/config.yaml
    assert_eq!(project_path, PathBuf::from("./.agent/config.yaml"));
}

/// Test: Validation of loaded configuration
#[test]
fn test_validate_loaded_configuration() {
    let mut manager = ConfigurationManager::new();

    // Add a valid provider
    manager.config_mut().providers.insert(
        "openai".to_string(),
        ProviderSettings {
            api_key: Some("test-key".to_string()),
            base_url: None,
            timeout: None,
            retry_count: None,
        },
    );

    // Should validate successfully
    assert!(manager.validate().is_ok());
}

/// Test: Validation fails with missing API key
#[test]
fn test_validate_fails_with_missing_api_key() {
    // Clean up any existing environment variables
    std::env::remove_var("OPENAI_API_KEY");
    std::env::remove_var("MISSING_PROVIDER_API_KEY");

    let mut manager = ConfigurationManager::new();

    // Use a unique provider name that won't have env vars set
    manager.config_mut().defaults.provider = "missing_provider".to_string();

    // Add provider without API key
    manager.config_mut().providers.insert(
        "missing_provider".to_string(),
        ProviderSettings {
            api_key: None,
            base_url: None,
            timeout: None,
            retry_count: None,
        },
    );

    // Should fail validation
    assert!(manager.validate().is_err());
}

/// Test: Validation fails with invalid default provider
#[test]
fn test_validate_fails_with_invalid_default_provider() {
    let mut manager = ConfigurationManager::new();

    // Set default provider that doesn't exist
    manager.config_mut().defaults.provider = "non-existent".to_string();

    // Add a different provider
    manager.config_mut().providers.insert(
        "openai".to_string(),
        ProviderSettings {
            api_key: Some("test-key".to_string()),
            base_url: None,
            timeout: None,
            retry_count: None,
        },
    );

    // Should fail validation
    assert!(manager.validate().is_err());
}

/// Test: Validation with multiple providers
#[test]
fn test_validate_with_multiple_providers() {
    let mut manager = ConfigurationManager::new();

    // Add multiple providers
    manager.config_mut().providers.insert(
        "openai".to_string(),
        ProviderSettings {
            api_key: Some("openai-key".to_string()),
            base_url: None,
            timeout: None,
            retry_count: None,
        },
    );

    manager.config_mut().providers.insert(
        "anthropic".to_string(),
        ProviderSettings {
            api_key: Some("anthropic-key".to_string()),
            base_url: None,
            timeout: None,
            retry_count: None,
        },
    );

    manager.config_mut().providers.insert(
        "google".to_string(),
        ProviderSettings {
            api_key: Some("google-key".to_string()),
            base_url: None,
            timeout: None,
            retry_count: None,
        },
    );

    // Should validate successfully
    assert!(manager.validate().is_ok());
}

/// Test: Validation with per-command defaults
#[test]
fn test_validate_with_per_command_defaults() {
    let mut manager = ConfigurationManager::new();

    manager.config_mut().providers.insert(
        "openai".to_string(),
        ProviderSettings {
            api_key: Some("test-key".to_string()),
            base_url: None,
            timeout: None,
            retry_count: None,
        },
    );

    // Add valid per-command defaults
    manager
        .config_mut()
        .defaults
        .per_command
        .insert("gen".to_string(), "gpt-4".to_string());
    manager
        .config_mut()
        .defaults
        .per_command
        .insert("refactor".to_string(), "gpt-4".to_string());
    manager
        .config_mut()
        .defaults
        .per_command
        .insert("review".to_string(), "gpt-4".to_string());

    // Should validate successfully
    assert!(manager.validate().is_ok());
}

/// Test: Validation fails with invalid per-command defaults
#[test]
fn test_validate_fails_with_invalid_per_command_defaults() {
    let mut manager = ConfigurationManager::new();

    manager.config_mut().providers.insert(
        "openai".to_string(),
        ProviderSettings {
            api_key: Some("test-key".to_string()),
            base_url: None,
            timeout: None,
            retry_count: None,
        },
    );

    // Add invalid per-command default
    manager
        .config_mut()
        .defaults
        .per_command
        .insert("invalid".to_string(), "gpt-4".to_string());

    // Should fail validation
    assert!(manager.validate().is_err());
}

/// Test: Validation with per-action defaults
#[test]
fn test_validate_with_per_action_defaults() {
    let mut manager = ConfigurationManager::new();

    manager.config_mut().providers.insert(
        "openai".to_string(),
        ProviderSettings {
            api_key: Some("test-key".to_string()),
            base_url: None,
            timeout: None,
            retry_count: None,
        },
    );

    // Add valid per-action defaults
    manager
        .config_mut()
        .defaults
        .per_action
        .insert("analysis".to_string(), "gpt-4".to_string());
    manager
        .config_mut()
        .defaults
        .per_action
        .insert("generation".to_string(), "gpt-4".to_string());

    // Should validate successfully
    assert!(manager.validate().is_ok());
}

/// Test: Validation fails with invalid per-action defaults
#[test]
fn test_validate_fails_with_invalid_per_action_defaults() {
    let mut manager = ConfigurationManager::new();

    manager.config_mut().providers.insert(
        "openai".to_string(),
        ProviderSettings {
            api_key: Some("test-key".to_string()),
            base_url: None,
            timeout: None,
            retry_count: None,
        },
    );

    // Add invalid per-action default
    manager
        .config_mut()
        .defaults
        .per_action
        .insert("invalid".to_string(), "gpt-4".to_string());

    // Should fail validation
    assert!(manager.validate().is_err());
}

/// Test: Validation with timeout settings
#[test]
fn test_validate_with_timeout_settings() {
    let mut manager = ConfigurationManager::new();

    manager.config_mut().providers.insert(
        "openai".to_string(),
        ProviderSettings {
            api_key: Some("test-key".to_string()),
            base_url: None,
            timeout: Some(Duration::from_secs(30)),
            retry_count: None,
        },
    );

    // Should validate successfully
    assert!(manager.validate().is_ok());
}

/// Test: Validation fails with zero timeout
#[test]
fn test_validate_fails_with_zero_timeout() {
    let mut manager = ConfigurationManager::new();

    manager.config_mut().providers.insert(
        "openai".to_string(),
        ProviderSettings {
            api_key: Some("test-key".to_string()),
            base_url: None,
            timeout: Some(Duration::from_secs(0)),
            retry_count: None,
        },
    );

    // Should fail validation
    assert!(manager.validate().is_err());
}

/// Test: Validation with retry count settings
#[test]
fn test_validate_with_retry_count_settings() {
    let mut manager = ConfigurationManager::new();

    manager.config_mut().providers.insert(
        "openai".to_string(),
        ProviderSettings {
            api_key: Some("test-key".to_string()),
            base_url: None,
            timeout: None,
            retry_count: Some(3),
        },
    );

    // Should validate successfully
    assert!(manager.validate().is_ok());
}

/// Test: Validation fails with excessive retry count
#[test]
fn test_validate_fails_with_excessive_retry_count() {
    let mut manager = ConfigurationManager::new();

    manager.config_mut().providers.insert(
        "openai".to_string(),
        ProviderSettings {
            api_key: Some("test-key".to_string()),
            base_url: None,
            timeout: None,
            retry_count: Some(15),
        },
    );

    // Should fail validation
    assert!(manager.validate().is_err());
}

/// Test: Get API key from configuration
#[test]
fn test_get_api_key_from_configuration() {
    let mut manager = ConfigurationManager::new();

    manager.config_mut().providers.insert(
        "openai".to_string(),
        ProviderSettings {
            api_key: Some("config-key".to_string()),
            base_url: None,
            timeout: None,
            retry_count: None,
        },
    );

    let key = manager.get_api_key("openai");
    assert!(key.is_ok());
    assert_eq!(key.unwrap(), "config-key");
}

/// Test: Get API key from environment when not in config
#[test]
fn test_get_api_key_from_environment() {
    // Ensure clean environment state
    std::env::remove_var("ANTHROPIC_API_KEY");

    let mut manager = ConfigurationManager::new();

    manager.config_mut().providers.insert(
        "anthropic".to_string(),
        ProviderSettings {
            api_key: None,
            base_url: None,
            timeout: None,
            retry_count: None,
        },
    );

    std::env::set_var("ANTHROPIC_API_KEY", "env-key");

    let key = manager.get_api_key("anthropic");
    assert!(key.is_ok());
    assert_eq!(key.unwrap(), "env-key");

    std::env::remove_var("ANTHROPIC_API_KEY");
}

/// Test: Get API key fails when not found
#[test]
fn test_get_api_key_fails_when_not_found() {
    let mut manager = ConfigurationManager::new();

    manager.config_mut().providers.insert(
        "missing".to_string(),
        ProviderSettings {
            api_key: None,
            base_url: None,
            timeout: None,
            retry_count: None,
        },
    );

    let key = manager.get_api_key("missing");
    assert!(key.is_err());
}

/// Test: Configuration manager default values
#[test]
fn test_configuration_manager_defaults() {
    let manager = ConfigurationManager::new();

    assert_eq!(manager.default_provider(), "openai");
    assert_eq!(manager.default_model(), "gpt-4");
    assert_eq!(manager.config().providers.len(), 0);
}

/// Test: Merge configuration preserves existing settings
#[test]
fn test_merge_configuration_preserves_existing() {
    let mut manager = ConfigurationManager::new();

    // Add initial provider
    manager.config_mut().providers.insert(
        "openai".to_string(),
        ProviderSettings {
            api_key: Some("initial-key".to_string()),
            base_url: None,
            timeout: None,
            retry_count: None,
        },
    );

    // Create new config to merge
    let new_config = ProviderConfig {
        defaults: DefaultsConfig {
            provider: "anthropic".to_string(),
            model: "claude-3".to_string(),
            per_command: HashMap::new(),
            per_action: HashMap::new(),
        },
        providers: {
            let mut map = HashMap::new();
            map.insert(
                "anthropic".to_string(),
                ProviderSettings {
                    api_key: Some("anthropic-key".to_string()),
                    base_url: None,
                    timeout: None,
                    retry_count: None,
                },
            );
            map
        },
    };

    // Simulate merge
    manager.config_mut().defaults = new_config.defaults;
    manager.config_mut().providers.extend(new_config.providers);

    // Verify both providers exist
    assert!(manager.config().providers.contains_key("openai"));
    assert!(manager.config().providers.contains_key("anthropic"));
    assert_eq!(manager.default_provider(), "anthropic");
}
