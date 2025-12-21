// Property-based tests for provider fallback chain and registry consistency
// **Feature: ricecoder-cli, Property 5: Provider Fallback Chain**
// **Validates: Requirements 7.4**
// **Feature: ricecoder-cli, Property 9: Provider Registry Consistency**
// **Validates: Requirements 7.1, 7.2, 7.3**

use proptest::prelude::*;
use ricecoder_cli::commands::ChatCommand;

// ============================================================================
// Property 5: Provider Fallback Chain
// ============================================================================
// For any provider specified via CLI or configuration, the system should
// successfully resolve to a valid provider without panicking.

proptest! {
    #[test]
    fn prop_provider_fallback_chain_cli_override(provider in "zen|openai|anthropic|local") {
        // When a provider is specified via CLI, it should be used
        let cmd = ChatCommand::new(None, Some(provider.clone()), None);

        // The provider should be stored in the command
        assert_eq!(cmd.provider, Some(provider.clone()));

        // Getting the provider should return the CLI-specified value
        let result = cmd.get_provider();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), provider);
    }

    #[test]
    fn prop_provider_fallback_chain_no_cli_uses_config(
        _dummy in prop::bool::ANY
    ) {
        // When no provider is specified via CLI, it should fall back to config
        let cmd = ChatCommand::new(None, None, None);

        // The provider should be None in the command
        assert!(cmd.provider.is_none());

        // Getting the provider should return a valid provider (from config or default)
        let result = cmd.get_provider();
        assert!(result.is_ok());

        let provider = result.unwrap();
        assert!(!provider.is_empty());
    }

    #[test]
    fn prop_provider_fallback_chain_validation(provider in "zen|openai|anthropic|local|custom") {
        // Any non-empty provider name should pass validation
        let cmd = ChatCommand::new(None, Some(provider.clone()), None);

        let result = cmd.validate_provider(&provider);
        assert!(result.is_ok());
    }

    #[test]
    fn prop_provider_fallback_chain_empty_provider_fails(
        _dummy in prop::bool::ANY
    ) {
        // Empty provider name should fail validation
        let cmd = ChatCommand::new(None, None, None);

        let result = cmd.validate_provider("");
        assert!(result.is_err());
    }
}

// ============================================================================
// Property 9: Provider Registry Consistency
// ============================================================================
// For any configuration state, the provider registry should maintain
// consistency: providers registered should be retrievable, and the
// registry should not lose or corrupt provider information.

proptest! {
    #[test]
    fn prop_provider_registry_consistency_cli_provider(
        provider in "zen|openai|anthropic|local"
    ) {
        // When a provider is specified via CLI, it should be consistently
        // available through the command
        let cmd = ChatCommand::new(None, Some(provider.clone()), None);

        // First access
        let result1 = cmd.get_provider();
        assert!(result1.is_ok());
        let provider1 = result1.unwrap();

        // Second access should return the same value (consistency)
        let result2 = cmd.get_provider();
        assert!(result2.is_ok());
        let provider2 = result2.unwrap();

        assert_eq!(provider1, provider2);
        assert_eq!(provider1, provider);
    }

    #[test]
    fn prop_provider_registry_consistency_config_provider(
        _dummy in prop::bool::ANY
    ) {
        // When no CLI provider is specified, config should provide consistent results
        let cmd = ChatCommand::new(None, None, None);

        // First access
        let result1 = cmd.get_provider();
        assert!(result1.is_ok());
        let provider1 = result1.unwrap();

        // Second access should return the same value (consistency)
        let result2 = cmd.get_provider();
        assert!(result2.is_ok());
        let provider2 = result2.unwrap();

        assert_eq!(provider1, provider2);
    }

    #[test]
    fn prop_provider_registry_consistency_model_selection(
        model in "gpt-4|gpt-3.5-turbo|claude-3|llama-2"
    ) {
        // When a model is specified via CLI, it should be consistently available
        let cmd = ChatCommand::new(None, None, Some(model.clone()));

        // First access
        let result1 = cmd.get_model();
        assert!(result1.is_ok());
        let model1 = result1.unwrap();

        // Second access should return the same value (consistency)
        let result2 = cmd.get_model();
        assert!(result2.is_ok());
        let model2 = result2.unwrap();

        assert_eq!(model1, model2);
        assert_eq!(model1, model);
    }

    #[test]
    fn prop_provider_registry_consistency_config_model(
        _dummy in prop::bool::ANY
    ) {
        // When no CLI model is specified, config should provide consistent results
        let cmd = ChatCommand::new(None, None, None);

        // First access
        let result1 = cmd.get_model();
        assert!(result1.is_ok());
        let model1 = result1.unwrap();

        // Second access should return the same value (consistency)
        let result2 = cmd.get_model();
        assert!(result2.is_ok());
        let model2 = result2.unwrap();

        assert_eq!(model1, model2);
    }

    #[test]
    fn prop_provider_registry_consistency_cli_overrides_config(
        provider in "zen|openai|anthropic|local",
        model in "gpt-4|gpt-3.5-turbo|claude-3|llama-2"
    ) {
        // CLI arguments should always override configuration
        let cmd = ChatCommand::new(None, Some(provider.clone()), Some(model.clone()));

        // Provider from CLI should be used
        let result_provider = cmd.get_provider();
        assert!(result_provider.is_ok());
        assert_eq!(result_provider.unwrap(), provider);

        // Model from CLI should be used
        let result_model = cmd.get_model();
        assert!(result_model.is_ok());
        assert_eq!(result_model.unwrap(), model);
    }

    #[test]
    fn prop_provider_registry_consistency_no_panic_on_any_input(
        provider in ".*",
        model in ".*"
    ) {
        // The system should never panic, even with arbitrary input
        let cmd = ChatCommand::new(None, Some(provider), Some(model));

        // These operations should not panic
        let _ = cmd.get_provider();
        let _ = cmd.get_model();
    }
}

// ============================================================================
// Integration Tests for Provider Fallback and Registry
// ============================================================================

#[test]
fn test_provider_fallback_chain_integration() {
    // Test the complete fallback chain:
    // 1. CLI provider (if specified)
    // 2. Config provider (if available)
    // 3. Default provider

    // Case 1: CLI provider specified
    let cmd1 = ChatCommand::new(None, Some("openai".to_string()), None);
    assert_eq!(cmd1.get_provider().unwrap(), "openai");

    // Case 2: No CLI provider, should use config or default
    let cmd2 = ChatCommand::new(None, None, None);
    let provider = cmd2.get_provider().unwrap();
    assert!(!provider.is_empty());
}

#[test]
fn test_provider_registry_consistency_integration() {
    // Test that provider registry maintains consistency across multiple operations

    let cmd = ChatCommand::new(
        None,
        Some("zen".to_string()),
        Some("big-pickle".to_string()),
    );

    // Multiple accesses should return consistent results
    for _ in 0..10 {
        assert_eq!(cmd.get_provider().unwrap(), "zen");
        assert_eq!(cmd.get_model().unwrap(), "big-pickle");
    }
}

#[test]
fn test_provider_fallback_with_empty_cli_args() {
    // When CLI args are None, should fall back to config
    let cmd = ChatCommand::new(None, None, None);

    let provider = cmd.get_provider();
    assert!(provider.is_ok());

    let model = cmd.get_model();
    assert!(model.is_ok());
}

#[test]
fn test_provider_validation_accepts_any_non_empty() {
    // Provider validation should accept any non-empty provider name
    let cmd = ChatCommand::new(None, None, None);

    let valid_providers = vec!["zen", "openai", "anthropic", "local", "custom-provider"];

    for provider in valid_providers {
        assert!(cmd.validate_provider(provider).is_ok());
    }
}

#[test]
fn test_provider_validation_rejects_empty() {
    // Provider validation should reject empty provider name
    let cmd = ChatCommand::new(None, None, None);

    assert!(cmd.validate_provider("").is_err());
}
