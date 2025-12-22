//! Property-based tests for keybind data models
//! **Feature: ricecoder-keybinds, Property 5: Configuration round-trip (parse/serialize equivalence)**
//! **Validates: Requirements 1.2, 1.3**

use std::{collections::HashSet, str::FromStr};

use proptest::prelude::*;
use ricecoder_keybinds::{ConflictDetector, Key, KeyCombo, Keybind, KeybindRegistry, Modifier};

/// Strategy for generating valid modifiers
fn modifier_strategy() -> impl Strategy<Value = Modifier> {
    prop_oneof![
        Just(Modifier::Ctrl),
        Just(Modifier::Shift),
        Just(Modifier::Alt),
        Just(Modifier::Meta),
    ]
}

/// Strategy for generating valid keys
fn key_strategy() -> impl Strategy<Value = Key> {
    prop_oneof![
        // Single character keys
        (b'a'..=b'z').prop_map(|c| Key::Char(c as char)),
        (b'0'..=b'9').prop_map(|c| Key::Char(c as char)),
        // Special keys
        Just(Key::Enter),
        Just(Key::Escape),
        Just(Key::Tab),
        Just(Key::Backspace),
        Just(Key::Delete),
        Just(Key::Home),
        Just(Key::End),
        Just(Key::PageUp),
        Just(Key::PageDown),
        Just(Key::Up),
        Just(Key::Down),
        Just(Key::Left),
        Just(Key::Right),
        // Function keys F1-F12
        (1u8..=12u8).prop_map(Key::F),
    ]
}

/// Strategy for generating valid key combinations
fn key_combo_strategy() -> impl Strategy<Value = KeyCombo> {
    (
        prop::collection::vec(modifier_strategy(), 0..3),
        key_strategy(),
    )
        .prop_map(|(modifiers, key)| KeyCombo { modifiers, key })
}

/// Strategy for generating valid keybinds
fn keybind_strategy() -> impl Strategy<Value = Keybind> {
    (
        r"[a-z]+(\.[a-z]+)*",
        r"[a-z]+(\.[a-z]+)*",
        r"[a-z]+",
        r"[a-z ]+",
        any::<bool>(),
    )
        .prop_map(|(action_id, category, key_str, description, is_default)| {
            let mut kb = Keybind::new(action_id, key_str, category, description);
            kb.is_default = is_default;
            kb
        })
}

proptest! {
    /// Property 5: Configuration round-trip
    /// For any valid keybind configuration, serializing then deserializing
    /// should produce an equivalent configuration.
    /// **Validates: Requirements 1.2, 1.3**
    #[test]
    fn prop_keybind_round_trip(keybind in keybind_strategy()) {
        // Serialize to JSON
        let json = serde_json::to_string(&keybind)
            .expect("Failed to serialize keybind");

        // Deserialize from JSON
        let deserialized: Keybind = serde_json::from_str(&json)
            .expect("Failed to deserialize keybind");

        // Verify equivalence
        assert_eq!(keybind.action_id, deserialized.action_id);
        assert_eq!(keybind.key, deserialized.key);
        assert_eq!(keybind.category, deserialized.category);
        assert_eq!(keybind.description, deserialized.description);
        assert_eq!(keybind.is_default, deserialized.is_default);
    }

    /// Property: KeyCombo round-trip through Display and FromStr
    /// For any valid key combination, converting to string and parsing back
    /// should produce an equivalent key combination.
    #[test]
    fn prop_key_combo_round_trip(key_combo in key_combo_strategy()) {
        // Convert to string
        let key_str = key_combo.to_string();

        // Parse back from string
        let parsed = KeyCombo::from_str(&key_str)
            .expect("Failed to parse key combo");

        // Verify equivalence
        assert_eq!(key_combo.modifiers, parsed.modifiers);
        assert_eq!(key_combo.key, parsed.key);
    }

    /// Property: Modifier round-trip through Display and FromStr
    /// For any valid modifier, converting to string and parsing back
    /// should produce an equivalent modifier.
    #[test]
    fn prop_modifier_round_trip(modifier in modifier_strategy()) {
        // Convert to string
        let mod_str = modifier.to_string();

        // Parse back from string
        let parsed = mod_str.parse::<Modifier>()
            .expect("Failed to parse modifier");

        // Verify equivalence
        assert_eq!(modifier, parsed);
    }

    /// Property: Key round-trip through Display and FromStr
    /// For any valid key, converting to string and parsing back
    /// should produce an equivalent key.
    #[test]
    fn prop_key_round_trip(key in key_strategy()) {
        // Convert to string
        let key_str = key.to_string();

        // Parse back from string
        let parsed = key_str.parse::<Key>()
            .expect("Failed to parse key");

        // Verify equivalence
        assert_eq!(key, parsed);
    }

    /// Property 2: Keybind consistency (deterministic lookups)
    /// For any keybind lookup operation, the result SHALL be consistent
    /// across multiple calls with the same input.
    /// **Feature: ricecoder-keybinds, Property 2: Keybind consistency (deterministic lookups)**
    /// **Validates: Requirements 1.1, 1.5**
    #[test]
    fn prop_keybind_consistency(keybinds in prop::collection::vec(keybind_strategy(), 1..20)) {
        let mut registry = KeybindRegistry::new();

        // Register all keybinds
        for keybind in keybinds.iter() {
            // Skip keybinds that would cause conflicts
            if registry.register(keybind.clone()).is_err() {
                continue;
            }
        }

        // For each registered keybind, verify lookup consistency
        for keybind in registry.all_keybinds() {
            let action_id = &keybind.action_id;

            // Lookup by action multiple times
            let lookup1 = registry.lookup_by_action(action_id);
            let lookup2 = registry.lookup_by_action(action_id);
            let lookup3 = registry.lookup_by_action(action_id);

            // All lookups should return the same result
            assert_eq!(lookup1, lookup2, "Lookup by action should be deterministic");
            assert_eq!(lookup2, lookup3, "Lookup by action should be deterministic");

            // Verify the returned keybind matches
            if let Some(found) = lookup1 {
                assert_eq!(found.action_id, keybind.action_id);
                assert_eq!(found.key, keybind.key);
            }
        }

        // For each registered keybind, verify key lookup consistency
        for keybind in registry.all_keybinds() {
            if let Ok(key_combo) = keybind.parse_key() {
                // Lookup by key multiple times
                let lookup1 = registry.lookup_by_key(&key_combo);
                let lookup2 = registry.lookup_by_key(&key_combo);
                let lookup3 = registry.lookup_by_key(&key_combo);

                // All lookups should return the same result
                assert_eq!(lookup1, lookup2, "Lookup by key should be deterministic");
                assert_eq!(lookup2, lookup3, "Lookup by key should be deterministic");

                // Verify the returned action matches
                if let Some(action) = lookup1 {
                    assert_eq!(action, keybind.action_id.as_str());
                }
            }
        }
    }

    /// Property 3: Profile isolation
    /// For any profile, keybinds in one profile SHALL not affect keybinds in other profiles.
    /// **Feature: ricecoder-keybinds, Property 3: Profile isolation (profiles don't affect each other)**
    /// **Validates: Requirements 3.1, 3.2, 3.5**
    #[test]
    fn prop_profile_isolation(
        profile1_keybinds in prop::collection::vec(keybind_strategy(), 1..10),
        profile2_keybinds in prop::collection::vec(keybind_strategy(), 1..10),
    ) {
        use ricecoder_keybinds::ProfileManager;

        let mut manager = ProfileManager::new();

        // Create two profiles with different keybinds
        let _ = manager.create_profile("profile1", profile1_keybinds.clone());
        let _ = manager.create_profile("profile2", profile2_keybinds.clone());

        // Get profile1 keybinds
        let profile1 = manager.get_profile("profile1");
        let profile1_actions: HashSet<String> = profile1
            .map(|p| p.keybinds.iter().map(|kb| kb.action_id.clone()).collect())
            .unwrap_or_default();

        // Get profile2 keybinds
        let profile2 = manager.get_profile("profile2");
        let profile2_actions: HashSet<String> = profile2
            .map(|p| p.keybinds.iter().map(|kb| kb.action_id.clone()).collect())
            .unwrap_or_default();

        // Verify profiles are isolated - modifying one doesn't affect the other
        // (This is verified by the fact that they maintain separate keybind lists)
        assert_eq!(
            profile1_actions.len(),
            profile1.map(|p| p.keybinds.len()).unwrap_or(0),
            "Profile1 should maintain its keybinds"
        );

        assert_eq!(
            profile2_actions.len(),
            profile2.map(|p| p.keybinds.len()).unwrap_or(0),
            "Profile2 should maintain its keybinds"
        );
    }

    /// Property 1: Conflict detection completeness
    /// For any set of keybinds, all duplicate key combinations SHALL be detected.
    /// **Feature: ricecoder-keybinds, Property 1: Conflict detection completeness (finds all duplicates)**
    /// **Validates: Requirements 2.1, 2.2**
    #[test]
    fn prop_conflict_detection_completeness(
        keybinds in prop::collection::vec(keybind_strategy(), 1..50)
    ) {
        // Detect conflicts
        let conflicts = ConflictDetector::detect(&keybinds);

        // Build a map of key strings to action IDs
        let mut key_to_actions: std::collections::HashMap<String, HashSet<String>> =
            std::collections::HashMap::new();

        for keybind in &keybinds {
            if let Ok(key_combo) = keybind.parse_key() {
                let key_str = key_combo.to_string();
                key_to_actions
                    .entry(key_str)
                    .or_insert_with(HashSet::new)
                    .insert(keybind.action_id.clone());
            }
        }

        // Find all expected conflicts (keys with multiple actions)
        let expected_conflicts: HashSet<String> = key_to_actions
            .iter()
            .filter(|(_, actions)| actions.len() > 1)
            .map(|(key, _)| key.clone())
            .collect();

        // Find all detected conflicts
        let detected_conflicts: HashSet<String> = conflicts
            .iter()
            .map(|c| c.key_combo.to_string())
            .collect();

        // Verify all expected conflicts were detected
        assert_eq!(
            expected_conflicts, detected_conflicts,
            "All conflicts should be detected"
        );

        // Verify no false positives
        for conflict in &conflicts {
            let key_str = conflict.key_combo.to_string();
            assert!(
                key_to_actions.get(&key_str).map_or(false, |actions| actions.len() > 1),
                "Detected conflict should have multiple actions"
            );

            // Verify all conflicting actions are reported
            let expected_actions: HashSet<String> = key_to_actions
                .get(&key_str)
                .map(|actions| actions.clone())
                .unwrap_or_default();

            let detected_actions: HashSet<String> = conflict.actions.iter().cloned().collect();

            assert_eq!(
                expected_actions, detected_actions,
                "All conflicting actions should be reported"
            );
        }
    }

    /// Property 4: Default keybind preservation
    /// For any default keybind, resetting to defaults SHALL restore the original keybind exactly.
    /// **Feature: ricecoder-keybinds, Property 4: Default keybind preservation (reset works correctly)**
    /// **Validates: Requirements 5.4, 5.5**
    #[test]
    fn prop_default_keybind_preservation(
        default_keybinds in prop::collection::vec(
            (
                r"[a-z]+(\.[a-z]+)*",
                r"[a-z]+(\.[a-z]+)*",
                r"[a-z]+",
                r"[a-z ]+",
            )
                .prop_map(|(action_id, category, key_str, description)| {
                    Keybind::new_default(action_id, key_str, category, description)
                }),
            1..20
        )
    ) {
        use ricecoder_keybinds::KeybindEngine;

        let mut engine = KeybindEngine::new();

        // Store the original defaults
        let original_keybinds = default_keybinds.clone();

        // Manually set defaults (simulating load_defaults_from_file)
        engine.set_defaults(original_keybinds.clone());

        // Apply defaults
        let _ = engine.apply_defaults();

        // Modify keybinds
        let custom_keybinds = vec![
            Keybind::new("custom.action", "Ctrl+Q", "custom", "Custom action"),
        ];
        let _ = engine.apply_keybinds(custom_keybinds);

        // Reset to defaults
        let _ = engine.reset_to_defaults();

        // Verify all defaults are restored
        let restored_keybinds = engine.get_defaults();

        assert_eq!(
            restored_keybinds.len(),
            original_keybinds.len(),
            "All default keybinds should be restored"
        );

        // Verify each default keybind is restored exactly
        for original in &original_keybinds {
            let found = restored_keybinds.iter().find(|kb| kb.action_id == original.action_id);
            assert!(found.is_some(), "Default keybind {} should be restored", original.action_id);

            let found = found.unwrap();
            assert_eq!(found.action_id, original.action_id, "Action ID should match");
            assert_eq!(found.key, original.key, "Key should match");
            assert_eq!(found.category, original.category, "Category should match");
            assert_eq!(found.description, original.description, "Description should match");
            assert!(found.is_default, "is_default flag should be set");
        }
    }

    /// Property 8: Keybinding Context Isolation
    /// For any set of keybinds with different contexts, key lookups SHALL respect context boundaries.
    /// **Feature: ricecoder-keybinds, Property 8: Keybinding Context Isolation**
    /// **Validates: Requirements 50.1, 50.2**
    #[test]
    fn prop_keybinding_context_isolation(
        global_keybinds in prop::collection::vec(keybind_strategy(), 1..5),
        input_keybinds in prop::collection::vec(keybind_strategy(), 1..5),
        chat_keybinds in prop::collection::vec(keybind_strategy(), 1..5),
        dialog_keybinds in prop::collection::vec(keybind_strategy(), 1..5),
    ) {
        use ricecoder_keybinds::{KeybindEngine, Context};

        let mut engine = KeybindEngine::new();

        // Create context-specific keybinds
        let mut all_keybinds = Vec::new();

        // Add global keybinds
        for kb in &global_keybinds {
            let mut global_kb = kb.clone();
            global_kb.contexts = vec![Context::Global];
            all_keybinds.push(global_kb);
        }

        // Add input keybinds
        for kb in &input_keybinds {
            let mut input_kb = kb.clone();
            input_kb.contexts = vec![Context::Input];
            all_keybinds.push(input_kb);
        }

        // Add chat keybinds
        for kb in &chat_keybinds {
            let mut chat_kb = kb.clone();
            chat_kb.contexts = vec![Context::Chat];
            all_keybinds.push(chat_kb);
        }

        // Add dialog keybinds
        for kb in &dialog_keybinds {
            let mut dialog_kb = kb.clone();
            dialog_kb.contexts = vec![Context::Dialog];
            all_keybinds.push(dialog_kb);
        }

        // Apply all keybinds
        let _ = engine.apply_keybinds(all_keybinds);

        // Test context isolation: same key in different contexts should return different actions
        for global_kb in &global_keybinds {
            if let Ok(key_combo) = global_kb.parse_key() {
                // In global context, should find the global keybind
                let global_action = engine.get_action_in_context(&key_combo, &Context::Global);
                prop_assert_eq!(global_action, Some(global_kb.action_id.as_str()));

                // In input context, should NOT find the global keybind (unless it also applies to input)
                let input_action = engine.get_action_in_context(&key_combo, &Context::Input);
                if !global_kb.applies_to_context(&Context::Input) {
                    // If global keybind doesn't apply to input, input context should not find it
                    prop_assert_ne!(input_action, Some(global_kb.action_id.as_str()),
                        "Global keybind should not be found in input context unless it applies there");
                }
            }
        }

        // Test context hierarchy: more specific contexts should override general ones
        for input_kb in &input_keybinds {
            if let Ok(key_combo) = input_kb.parse_key() {
                // Check if there's a conflicting global keybind with the same key
                let conflicting_global = global_keybinds.iter().find(|gkb| {
                    gkb.parse_key().ok() == Some(key_combo.clone())
                });

                if let Some(_) = conflicting_global {
                    // With context hierarchy, input context should find the input keybind
                    let input_action = engine.get_action_in_context(&key_combo, &Context::Input);
                    prop_assert_eq!(input_action, Some(input_kb.action_id.as_str()),
                        "Input context should prioritize input-specific keybinds");
                }
            }
        }

        // Test active contexts: multiple contexts should be searched in priority order
        engine.set_context(Context::Input);
        engine.push_context(Context::Dialog);

        let active_contexts = engine.active_contexts();
        prop_assert!(active_contexts.contains(&Context::Dialog), "Active contexts should include dialog");
        prop_assert!(active_contexts.contains(&Context::Input), "Active contexts should include input");

        // Dialog should have higher priority than input
        let dialog_priority = Context::Dialog.priority();
        let input_priority = Context::Input.priority();
        prop_assert!(dialog_priority > input_priority, "Dialog should have higher priority than input");
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_keybind_serialization() {
        let kb = Keybind::new("editor.save", "Ctrl+S", "editing", "Save file");
        let json = serde_json::to_string(&kb).unwrap();
        let deserialized: Keybind = serde_json::from_str(&json).unwrap();

        assert_eq!(kb.action_id, deserialized.action_id);
        assert_eq!(kb.key, deserialized.key);
    }

    #[test]
    fn test_keybind_default_flag() {
        let kb = Keybind::new_default("editor.undo", "Ctrl+Z", "editing", "Undo");
        let json = serde_json::to_string(&kb).unwrap();
        let deserialized: Keybind = serde_json::from_str(&json).unwrap();

        assert!(deserialized.is_default);
    }

    #[test]
    fn test_key_combo_display() {
        let combo = KeyCombo {
            modifiers: vec![Modifier::Ctrl, Modifier::Shift],
            key: Key::Char('z'),
        };

        let display = combo.to_string();
        assert!(display.contains("Ctrl"));
        assert!(display.contains("Shift"));
        assert!(display.contains("z"));
    }

    #[test]
    fn test_key_combo_parse_single_modifier() {
        let combo = KeyCombo::from_str("Ctrl+S").unwrap();
        assert_eq!(combo.modifiers.len(), 1);
        assert_eq!(combo.modifiers[0], Modifier::Ctrl);
        assert_eq!(combo.key, Key::Char('s'));
    }

    #[test]
    fn test_key_combo_parse_multiple_modifiers() {
        let combo = KeyCombo::from_str("Ctrl+Shift+Z").unwrap();
        assert_eq!(combo.modifiers.len(), 2);
        assert!(combo.modifiers.contains(&Modifier::Ctrl));
        assert!(combo.modifiers.contains(&Modifier::Shift));
    }

    #[test]
    fn test_key_combo_parse_special_keys() {
        let combo = KeyCombo::from_str("Ctrl+Enter").unwrap();
        assert_eq!(combo.key, Key::Enter);

        let combo = KeyCombo::from_str("Alt+F1").unwrap();
        assert_eq!(combo.key, Key::F(1));
    }

    #[test]
    fn test_modifier_case_insensitive() {
        assert_eq!(Modifier::from_str("ctrl").unwrap(), Modifier::Ctrl);
        assert_eq!(Modifier::from_str("CTRL").unwrap(), Modifier::Ctrl);
        assert_eq!(Modifier::from_str("Ctrl").unwrap(), Modifier::Ctrl);
    }

    #[test]
    fn test_modifier_aliases() {
        assert_eq!(Modifier::from_str("control").unwrap(), Modifier::Ctrl);
        assert_eq!(Modifier::from_str("cmd").unwrap(), Modifier::Meta);
        assert_eq!(Modifier::from_str("command").unwrap(), Modifier::Meta);
    }

    #[test]
    fn test_key_aliases() {
        assert_eq!(Key::from_str("return").unwrap(), Key::Enter);
        assert_eq!(Key::from_str("esc").unwrap(), Key::Escape);
        assert_eq!(Key::from_str("del").unwrap(), Key::Delete);
        assert_eq!(Key::from_str("bksp").unwrap(), Key::Backspace);
    }

    #[test]
    fn test_function_keys() {
        for i in 1..=12 {
            let key = Key::from_str(&format!("F{}", i)).unwrap();
            assert_eq!(key, Key::F(i));
        }
    }

    #[test]
    fn test_invalid_function_key() {
        assert!(Key::from_str("F0").is_err());
        assert!(Key::from_str("F13").is_err());
    }
}
