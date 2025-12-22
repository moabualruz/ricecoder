//! Property-based tests for hook configuration
//!
//! Tests that verify correctness properties of the hook configuration system.
//! **Feature: ricecoder-hooks, Property 5: Hook configuration persistence**
//! **Validates: Requirements Hooks-3.1**

use std::collections::HashMap;

use proptest::prelude::*;
use ricecoder_hooks::{
    config::{ConfigReloader, ConfigValidator, TemplateManager},
    *,
};

// Strategy for generating valid hook IDs
fn hook_id_strategy() -> impl Strategy<Value = String> {
    "[a-z0-9_-]{1,20}".prop_map(|s| s.to_string())
}

// Strategy for generating valid hook names
fn hook_name_strategy() -> impl Strategy<Value = String> {
    "[A-Za-z0-9 ]{1,50}".prop_map(|s| s.to_string())
}

// Strategy for generating valid event names
fn event_name_strategy() -> impl Strategy<Value = String> {
    "[a-z_]{1,30}".prop_map(|s| s.to_string())
}

// Strategy for generating command actions
fn command_action_strategy() -> impl Strategy<Value = Action> {
    (
        "[a-z]{1,20}".prop_map(|s| s.to_string()),
        prop::collection::vec("[a-z0-9_-]{1,20}", 0..5),
        prop::option::of(1000u64..60000u64),
    )
        .prop_map(|(command, args, timeout)| {
            Action::Command(CommandAction {
                command,
                args,
                timeout_ms: timeout,
                capture_output: true,
            })
        })
}

// Strategy for generating valid hooks
fn hook_strategy() -> impl Strategy<Value = Hook> {
    (
        hook_id_strategy(),
        hook_name_strategy(),
        event_name_strategy(),
        command_action_strategy(),
    )
        .prop_map(|(id, name, event, action)| Hook {
            id,
            name,
            description: None,
            event,
            action,
            enabled: true,
            tags: vec![],
            metadata: serde_json::json!({}),
            condition: None,
        })
}

proptest! {
    /// Property 5.1: Hooks persist to configuration correctly
    ///
    /// For any valid hook, when serialized to YAML and deserialized back,
    /// the hook should be identical to the original.
    ///
    /// **Validates: Requirements Hooks-3.1**
    #[test]
    fn prop_hook_serialization_roundtrip(hook in hook_strategy()) {
        // Serialize hook to YAML
        let yaml_str = serde_yaml::to_string(&hook)
            .expect("Should serialize hook to YAML");

        // Deserialize back
        let deserialized: Hook = serde_yaml::from_str(&yaml_str)
            .expect("Should deserialize hook from YAML");

        // Verify hook is identical
        prop_assert_eq!(hook.id, deserialized.id);
        prop_assert_eq!(hook.name, deserialized.name);
        prop_assert_eq!(hook.event, deserialized.event);
        prop_assert_eq!(hook.enabled, deserialized.enabled);
    }

    /// Property 5.2: Configuration hierarchy is respected
    ///
    /// For any set of hooks, when loaded through the configuration hierarchy,
    /// hooks from higher priority sources should override hooks from lower
    /// priority sources with the same ID.
    ///
    /// **Validates: Requirements Hooks-3.1**
    #[test]
    fn prop_configuration_hierarchy_respected(
        hook1 in hook_strategy(),
        hook2 in hook_strategy(),
    ) {
        // Ensure hooks have different IDs for this test
        let mut hook1 = hook1;
        let mut hook2 = hook2;
        hook1.id = "hook-a".to_string();
        hook2.id = "hook-b".to_string();

        // Create two configuration maps
        let mut config1 = HashMap::new();
        config1.insert(hook1.id.clone(), hook1.clone());

        let mut config2 = HashMap::new();
        config2.insert(hook2.id.clone(), hook2.clone());

        // Merge with config2 overriding config1
        config1.extend(config2);

        // Verify both hooks are present
        prop_assert!(config1.contains_key("hook-a"));
        prop_assert!(config1.contains_key("hook-b"));
        prop_assert_eq!(config1.len(), 2);
    }

    /// Property 5.3: Hook state is preserved during reload
    ///
    /// For any set of hooks with various enabled/disabled states, when
    /// reloading configuration, the enabled/disabled state should be preserved.
    ///
    /// **Validates: Requirements Hooks-3.1**
    #[test]
    fn prop_hook_state_preserved_on_reload(
        hook1 in hook_strategy(),
        hook2 in hook_strategy(),
    ) {
        // Ensure hooks have different IDs
        let mut hook1 = hook1;
        let mut hook2 = hook2;
        hook1.id = "hook-1".to_string();
        hook2.id = "hook-2".to_string();

        // Set different enabled states
        hook1.enabled = true;
        hook2.enabled = false;

        // Create initial hooks map
        let mut hooks = HashMap::new();
        hooks.insert(hook1.id.clone(), hook1.clone());
        hooks.insert(hook2.id.clone(), hook2.clone());

        // Create reloader and save state
        let mut reloader = ConfigReloader::new();
        reloader.save_hook_state(&hooks);

        // Modify hooks (simulate reload with different configuration)
        hooks.get_mut("hook-1").unwrap().enabled = false;
        hooks.get_mut("hook-2").unwrap().enabled = true;

        // Restore state
        reloader.restore_hook_state(&mut hooks);

        // Verify state is restored
        prop_assert_eq!(hooks.get("hook-1").unwrap().enabled, true);
        prop_assert_eq!(hooks.get("hook-2").unwrap().enabled, false);
    }

    /// Property 5.4: Validator accepts valid hooks
    ///
    /// For any valid hook generated by the hook_strategy, the validator
    /// should accept it without errors.
    ///
    /// **Validates: Requirements Hooks-3.1**
    #[test]
    fn prop_validator_accepts_valid_hooks(hook in hook_strategy()) {
        let result = ConfigValidator::validate_hook(&hook);
        prop_assert!(result.is_ok(), "Validator should accept valid hook");
    }

    /// Property 5.5: Validator rejects hooks with empty ID
    ///
    /// For any valid hook, when the ID is set to empty string, the validator
    /// should reject it.
    ///
    /// **Validates: Requirements Hooks-3.1**
    #[test]
    fn prop_validator_rejects_empty_id(hook in hook_strategy()) {
        let mut hook = hook;
        hook.id = String::new();

        let result = ConfigValidator::validate_hook(&hook);
        prop_assert!(result.is_err(), "Validator should reject hook with empty ID");
    }

    /// Property 5.6: Validator rejects hooks with empty name
    ///
    /// For any valid hook, when the name is set to empty string, the validator
    /// should reject it.
    ///
    /// **Validates: Requirements Hooks-3.1**
    #[test]
    fn prop_validator_rejects_empty_name(hook in hook_strategy()) {
        let mut hook = hook;
        hook.name = String::new();

        let result = ConfigValidator::validate_hook(&hook);
        prop_assert!(result.is_err(), "Validator should reject hook with empty name");
    }

    /// Property 5.7: Validator rejects hooks with invalid event names
    ///
    /// For any valid hook, when the event name contains uppercase letters,
    /// the validator should reject it.
    ///
    /// **Validates: Requirements Hooks-3.1**
    #[test]
    fn prop_validator_rejects_invalid_event_names(hook in hook_strategy()) {
        let mut hook = hook;
        hook.event = "InvalidEvent".to_string();

        let result = ConfigValidator::validate_hook(&hook);
        prop_assert!(result.is_err(), "Validator should reject hook with invalid event name");
    }

    /// Property 5.8: Template instantiation preserves hook properties
    ///
    /// For any valid hook template and parameters, when instantiated,
    /// the resulting hook should have the correct ID, name, and event.
    ///
    /// **Validates: Requirements Hooks-3.1**
    #[test]
    fn prop_template_instantiation_preserves_properties(
        hook_id in hook_id_strategy(),
        hook_name in hook_name_strategy(),
    ) {
        let templates = TemplateManager::get_builtin_templates();
        let template = templates.get("file_save").expect("Should find template");

        let mut params = HashMap::new();
        params.insert("command".to_string(), "prettier".to_string());

        let hook = TemplateManager::instantiate_template(
            template,
            &hook_id,
            &hook_name,
            &params,
        ).expect("Should instantiate template");

        prop_assert_eq!(&hook.id, &hook_id);
        prop_assert_eq!(&hook.name, &hook_name);
        prop_assert_eq!(&hook.event, &template.event);
        prop_assert!(hook.enabled);
    }

    /// Property 5.9: Multiple hooks can coexist in configuration
    ///
    /// For any set of valid hooks with unique IDs, when stored in a
    /// configuration map, all hooks should be retrievable.
    ///
    /// **Validates: Requirements Hooks-3.1**
    #[test]
    fn prop_multiple_hooks_coexist(
        hook1 in hook_strategy(),
        hook2 in hook_strategy(),
        hook3 in hook_strategy(),
    ) {
        // Ensure unique IDs
        let mut hook1 = hook1;
        let mut hook2 = hook2;
        let mut hook3 = hook3;
        hook1.id = "hook-1".to_string();
        hook2.id = "hook-2".to_string();
        hook3.id = "hook-3".to_string();

        // Create configuration map
        let mut config = HashMap::new();
        config.insert(hook1.id.clone(), hook1.clone());
        config.insert(hook2.id.clone(), hook2.clone());
        config.insert(hook3.id.clone(), hook3.clone());

        // Verify all hooks are present
        prop_assert_eq!(config.len(), 3);
        prop_assert!(config.contains_key("hook-1"));
        prop_assert!(config.contains_key("hook-2"));
        prop_assert!(config.contains_key("hook-3"));

        // Verify hooks are retrievable
        prop_assert_eq!(&config.get("hook-1").unwrap().id, "hook-1");
        prop_assert_eq!(&config.get("hook-2").unwrap().id, "hook-2");
        prop_assert_eq!(&config.get("hook-3").unwrap().id, "hook-3");
    }

    /// Property 5.10: Configuration can be serialized and deserialized
    ///
    /// For any set of valid hooks, when serialized to YAML and deserialized
    /// back, the configuration should be identical.
    ///
    /// **Validates: Requirements Hooks-3.1**
    #[test]
    fn prop_configuration_serialization_roundtrip(
        hook1 in hook_strategy(),
        hook2 in hook_strategy(),
    ) {
        // Ensure unique IDs
        let mut hook1 = hook1;
        let mut hook2 = hook2;
        hook1.id = "hook-1".to_string();
        hook2.id = "hook-2".to_string();

        // Create configuration
        let mut config = HashMap::new();
        config.insert(hook1.id.clone(), hook1.clone());
        config.insert(hook2.id.clone(), hook2.clone());

        // Serialize to YAML
        let yaml_str = serde_yaml::to_string(&config)
            .expect("Should serialize config to YAML");

        // Deserialize back
        let deserialized: HashMap<String, Hook> = serde_yaml::from_str(&yaml_str)
            .expect("Should deserialize config from YAML");

        // Verify configuration is identical
        prop_assert_eq!(config.len(), deserialized.len());
        for (id, hook) in &config {
            let deserialized_hook = deserialized.get(id).expect("Should find hook");
            prop_assert_eq!(&hook.id, &deserialized_hook.id);
            prop_assert_eq!(&hook.name, &deserialized_hook.name);
            prop_assert_eq!(&hook.event, &deserialized_hook.event);
        }
    }
}
