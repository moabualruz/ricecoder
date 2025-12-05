//! Property-based tests for hook registry
//!
//! **Feature: ricecoder-hooks, Property 6: Hook enable/disable**
//! **Validates: Requirements Hooks-1.1**

use proptest::prelude::*;
use ricecoder_hooks::*;

/// Strategy for generating valid hook IDs
fn hook_id_strategy() -> impl Strategy<Value = String> {
    "[a-z0-9_-]{5,20}".prop_map(|s| s.to_string())
}

/// Strategy for generating valid event names
fn event_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("file_saved".to_string()),
        Just("file_modified".to_string()),
        Just("test_passed".to_string()),
        Just("test_failed".to_string()),
        Just("generation_complete".to_string()),
    ]
}

/// Strategy for generating valid hooks
fn hook_strategy() -> impl Strategy<Value = Hook> {
    (hook_id_strategy(), event_strategy()).prop_map(|(id, event)| Hook {
        id,
        name: "Test Hook".to_string(),
        description: None,
        event,
        action: Action::Command(CommandAction {
            command: "echo".to_string(),
            args: vec!["test".to_string()],
            timeout_ms: None,
            capture_output: false,
        }),
        enabled: true,
        tags: vec![],
        metadata: serde_json::json!({}),
        condition: None,
    })
}

proptest! {
    /// Property 6: Disabled hooks don't appear in list_hooks_for_event
    ///
    /// For any hook that is disabled, it should not appear in the list of hooks
    /// for its event type.
    ///
    /// **Validates: Requirements Hooks-1.1**
    #[test]
    fn prop_disabled_hooks_not_in_event_list(hook in hook_strategy()) {
        let mut registry = InMemoryHookRegistry::new();

        // Register the hook (it starts enabled)
        let hook_id = registry.register_hook(hook.clone()).unwrap();
        let event = hook.event.clone();

        // Verify it appears in the event list
        let hooks_before = registry.list_hooks_for_event(&event).unwrap();
        prop_assert_eq!(hooks_before.len(), 1);
        prop_assert_eq!(&hooks_before[0].id, &hook_id);

        // Disable the hook
        registry.disable_hook(&hook_id).unwrap();

        // Verify it no longer appears in the event list
        let hooks_after = registry.list_hooks_for_event(&event).unwrap();
        prop_assert_eq!(hooks_after.len(), 0);
    }

    /// Property 6: Enabled hooks appear in list_hooks_for_event
    ///
    /// For any hook that is enabled, it should appear in the list of hooks
    /// for its event type.
    ///
    /// **Validates: Requirements Hooks-1.1**
    #[test]
    fn prop_enabled_hooks_in_event_list(hook in hook_strategy()) {
        let mut registry = InMemoryHookRegistry::new();

        // Register the hook (it starts enabled)
        let hook_id = registry.register_hook(hook.clone()).unwrap();
        let event = hook.event.clone();

        // Verify it appears in the event list
        let hooks = registry.list_hooks_for_event(&event).unwrap();
        prop_assert_eq!(hooks.len(), 1);
        prop_assert_eq!(&hooks[0].id, &hook_id);
        prop_assert!(hooks[0].enabled);
    }

    /// Property 6: Hook state transitions are valid (enable/disable/enable)
    ///
    /// For any hook, we should be able to transition between enabled and disabled
    /// states multiple times, and the hook should always reflect the correct state.
    ///
    /// **Validates: Requirements Hooks-1.1**
    #[test]
    fn prop_hook_state_transitions(hook in hook_strategy()) {
        let mut registry = InMemoryHookRegistry::new();

        // Register the hook (starts enabled)
        let hook_id = registry.register_hook(hook.clone()).unwrap();

        // Verify initial state is enabled
        let h = registry.get_hook(&hook_id).unwrap();
        prop_assert!(h.enabled);

        // Disable the hook
        registry.disable_hook(&hook_id).unwrap();
        let h = registry.get_hook(&hook_id).unwrap();
        prop_assert!(!h.enabled);

        // Enable the hook again
        registry.enable_hook(&hook_id).unwrap();
        let h = registry.get_hook(&hook_id).unwrap();
        prop_assert!(h.enabled);

        // Disable again
        registry.disable_hook(&hook_id).unwrap();
        let h = registry.get_hook(&hook_id).unwrap();
        prop_assert!(!h.enabled);

        // Enable again
        registry.enable_hook(&hook_id).unwrap();
        let h = registry.get_hook(&hook_id).unwrap();
        prop_assert!(h.enabled);
    }

    /// Property 6: Multiple hooks can have different enabled states
    ///
    /// For any set of hooks, we should be able to enable/disable them independently,
    /// and each hook should maintain its own state.
    ///
    /// **Validates: Requirements Hooks-1.1**
    #[test]
    fn prop_multiple_hooks_independent_state(
        hook1 in hook_strategy(),
        hook2 in hook_strategy(),
        hook3 in hook_strategy(),
    ) {
        let mut registry = InMemoryHookRegistry::new();

        // Register three hooks
        let id1 = registry.register_hook(hook1.clone()).unwrap();
        let id2 = registry.register_hook(hook2.clone()).unwrap();
        let id3 = registry.register_hook(hook3.clone()).unwrap();

        // Disable hook 2
        registry.disable_hook(&id2).unwrap();

        // Verify states
        let h1 = registry.get_hook(&id1).unwrap();
        let h2 = registry.get_hook(&id2).unwrap();
        let h3 = registry.get_hook(&id3).unwrap();

        prop_assert!(h1.enabled);
        prop_assert!(!h2.enabled);
        prop_assert!(h3.enabled);

        // Enable hook 2, disable hook 1
        registry.enable_hook(&id2).unwrap();
        registry.disable_hook(&id1).unwrap();

        // Verify new states
        let h1 = registry.get_hook(&id1).unwrap();
        let h2 = registry.get_hook(&id2).unwrap();
        let h3 = registry.get_hook(&id3).unwrap();

        prop_assert!(!h1.enabled);
        prop_assert!(h2.enabled);
        prop_assert!(h3.enabled);
    }

    /// Property 6: Disabled hooks don't affect enabled hooks in event list
    ///
    /// For any set of hooks for the same event, disabling some should not affect
    /// the enabled ones in the event list.
    ///
    /// **Validates: Requirements Hooks-1.1**
    #[test]
    fn prop_disabled_hooks_dont_affect_enabled(
        event in event_strategy(),
        num_hooks in 1usize..10,
    ) {
        let mut registry = InMemoryHookRegistry::new();

        // Register multiple hooks for the same event
        let mut hook_ids = Vec::new();
        for i in 0..num_hooks {
            let hook = Hook {
                id: format!("hook_{}", i),
                name: format!("Hook {}", i),
                description: None,
                event: event.clone(),
                action: Action::Command(CommandAction {
                    command: "echo".to_string(),
                    args: vec!["test".to_string()],
                    timeout_ms: None,
                    capture_output: false,
                }),
                enabled: true,
                tags: vec![],
                metadata: serde_json::json!({}),
                condition: None,
            };
            let id = registry.register_hook(hook).unwrap();
            hook_ids.push(id);
        }

        // Verify all hooks are in the event list
        let hooks = registry.list_hooks_for_event(&event).unwrap();
        prop_assert_eq!(hooks.len(), num_hooks);

        // Disable half of the hooks
        for hook_id in hook_ids.iter().take(num_hooks / 2) {
            registry.disable_hook(hook_id).unwrap();
        }

        // Verify only the enabled hooks are in the event list
        let hooks = registry.list_hooks_for_event(&event).unwrap();
        let expected_count = num_hooks - (num_hooks / 2);
        prop_assert_eq!(hooks.len(), expected_count);

        // Verify all remaining hooks are enabled
        for hook in hooks {
            prop_assert!(hook.enabled);
        }
    }
}
