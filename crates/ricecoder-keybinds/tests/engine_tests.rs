use ricecoder_keybinds::{
    engine::{get_default_persistence, initialize_engine_with_defaults},
    *,
};

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_apply_keybinds() {
        let mut engine = KeybindEngine::new();
        let keybinds = vec![
            Keybind::new("editor.save", "Ctrl+S", "editing", "Save"),
            Keybind::new("editor.undo", "Ctrl+Z", "editing", "Undo"),
        ];

        assert!(engine.apply_keybinds(keybinds).is_ok());
        assert_eq!(engine.keybind_count(), 2);
    }

    #[test]
    fn test_get_action() {
        let mut engine = KeybindEngine::new();
        let keybinds = vec![Keybind::new("editor.save", "Ctrl+S", "editing", "Save")];

        engine.apply_keybinds(keybinds).unwrap();

        let key_combo = KeyCombo::from_str("Ctrl+S").unwrap();
        let action = engine.get_action(&key_combo);
        assert_eq!(action, Some("editor.save"));
    }

    #[test]
    fn test_get_keybind() {
        let mut engine = KeybindEngine::new();
        let keybinds = vec![Keybind::new("editor.save", "Ctrl+S", "editing", "Save")];

        engine.apply_keybinds(keybinds).unwrap();

        let keybind = engine.get_keybind("editor.save");
        assert!(keybind.is_some());
        assert_eq!(keybind.unwrap().key, "Ctrl+S");
    }

    #[test]
    fn test_all_keybinds() {
        let mut engine = KeybindEngine::new();
        let keybinds = vec![
            Keybind::new("editor.save", "Ctrl+S", "editing", "Save"),
            Keybind::new("editor.undo", "Ctrl+Z", "editing", "Undo"),
        ];

        engine.apply_keybinds(keybinds).unwrap();

        let all = engine.all_keybinds();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_categories() {
        let mut engine = KeybindEngine::new();
        let keybinds = vec![
            Keybind::new("editor.save", "Ctrl+S", "editing", "Save"),
            Keybind::new("nav.next", "Tab", "navigation", "Next"),
        ];

        engine.apply_keybinds(keybinds).unwrap();

        let categories = engine.categories();
        assert_eq!(categories.len(), 2);
    }

    #[test]
    fn test_create_profile() {
        let mut engine = KeybindEngine::new();
        let keybinds = vec![Keybind::new("editor.save", "Ctrl+S", "editing", "Save")];

        assert!(engine.create_profile("default", keybinds).is_ok());
    }

    #[test]
    fn test_select_profile_applies_keybinds() {
        let mut engine = KeybindEngine::new();
        let keybinds1 = vec![Keybind::new("editor.save", "Ctrl+S", "editing", "Save")];
        let keybinds2 = vec![Keybind::new("editor.undo", "Ctrl+Z", "editing", "Undo")];

        engine.create_profile("profile1", keybinds1).unwrap();
        engine.create_profile("profile2", keybinds2).unwrap();

        // Select profile2 and verify keybinds are applied
        engine.select_profile("profile2").unwrap();
        assert_eq!(engine.active_profile_name(), Some("profile2"));

        // Verify profile2's keybinds are active
        let key_combo = KeyCombo::from_str("Ctrl+Z").unwrap();
        assert_eq!(engine.get_action(&key_combo), Some("editor.undo"));
    }

    #[test]
    fn test_delete_profile() {
        let mut engine = KeybindEngine::new();
        let keybinds = vec![Keybind::new("editor.save", "Ctrl+S", "editing", "Save")];

        engine.create_profile("profile1", keybinds.clone()).unwrap();
        engine.create_profile("profile2", keybinds).unwrap();

        // Switch to profile2 before deleting profile1
        engine.select_profile("profile2").unwrap();
        assert!(engine.delete_profile("profile1").is_ok());
    }

    #[test]
    fn test_keybind_count() {
        let mut engine = KeybindEngine::new();
        let keybinds = vec![
            Keybind::new("editor.save", "Ctrl+S", "editing", "Save"),
            Keybind::new("editor.undo", "Ctrl+Z", "editing", "Undo"),
            Keybind::new("nav.next", "Tab", "navigation", "Next"),
        ];

        engine.apply_keybinds(keybinds).unwrap();
        assert_eq!(engine.keybind_count(), 3);
    }

    #[test]
    fn test_has_keybinds() {
        let mut engine = KeybindEngine::new();
        assert!(!engine.has_keybinds());

        let keybinds = vec![Keybind::new("editor.save", "Ctrl+S", "editing", "Save")];
        engine.apply_keybinds(keybinds).unwrap();
        assert!(engine.has_keybinds());
    }

    #[test]
    fn test_keybinds_by_category() {
        let mut engine = KeybindEngine::new();
        let keybinds = vec![
            Keybind::new("editor.save", "Ctrl+S", "editing", "Save"),
            Keybind::new("editor.undo", "Ctrl+Z", "editing", "Undo"),
            Keybind::new("nav.next", "Tab", "navigation", "Next"),
        ];

        engine.apply_keybinds(keybinds).unwrap();

        let editing = engine.keybinds_by_category("editing");
        assert_eq!(editing.len(), 2);

        let navigation = engine.keybinds_by_category("navigation");
        assert_eq!(navigation.len(), 1);
    }

    #[test]
    fn test_apply_keybinds_clears_previous() {
        let mut engine = KeybindEngine::new();
        let keybinds1 = vec![Keybind::new("editor.save", "Ctrl+S", "editing", "Save")];
        let keybinds2 = vec![Keybind::new("editor.undo", "Ctrl+Z", "editing", "Undo")];

        engine.apply_keybinds(keybinds1).unwrap();
        assert_eq!(engine.keybind_count(), 1);

        engine.apply_keybinds(keybinds2).unwrap();
        assert_eq!(engine.keybind_count(), 1);

        // Verify old keybind is gone
        let key_combo = KeyCombo::from_str("Ctrl+S").unwrap();
        assert_eq!(engine.get_action(&key_combo), None);

        // Verify new keybind is present
        let key_combo = KeyCombo::from_str("Ctrl+Z").unwrap();
        assert_eq!(engine.get_action(&key_combo), Some("editor.undo"));
    }

    #[test]
    fn test_get_keybind_returns_none_for_missing_action() {
        let engine = KeybindEngine::new();
        assert_eq!(engine.get_keybind("nonexistent.action"), None);
    }

    #[test]
    fn test_get_action_returns_none_for_missing_key() {
        let engine = KeybindEngine::new();
        let key_combo = KeyCombo::from_str("Ctrl+X").unwrap();
        assert_eq!(engine.get_action(&key_combo), None);
    }

    #[test]
    fn test_active_profile_name() {
        let mut engine = KeybindEngine::new();
        let keybinds = vec![Keybind::new("editor.save", "Ctrl+S", "editing", "Save")];

        engine.create_profile("default", keybinds).unwrap();
        assert_eq!(engine.active_profile_name(), Some("default"));
    }

    #[test]
    fn test_load_defaults_from_file() {
        let mut engine = KeybindEngine::new();

        // Try multiple possible paths
        let possible_paths = vec![
            "../../../../config/keybinds/defaults.json",
            "projects/ricecoder/config/keybinds/defaults.json",
            "config/keybinds/defaults.json",
        ];

        let mut loaded = false;
        for path in possible_paths {
            if engine.load_defaults_from_file(path).is_ok() {
                loaded = true;
                break;
            }
        }

        if loaded {
            assert!(!engine.get_defaults().is_empty());

            // Verify all defaults have is_default flag
            for keybind in engine.get_defaults() {
                assert!(keybind.is_default);
            }
        }
    }

    #[test]
    fn test_apply_defaults() {
        let mut engine = KeybindEngine::new();

        // Try multiple possible paths
        let possible_paths = vec![
            "../../../../config/keybinds/defaults.json",
            "projects/ricecoder/config/keybinds/defaults.json",
            "config/keybinds/defaults.json",
        ];

        let mut loaded = false;
        for path in possible_paths {
            if engine.load_defaults_from_file(path).is_ok() {
                loaded = true;
                break;
            }
        }

        if loaded {
            assert!(engine.apply_defaults().is_ok());

            // Verify default profile was created
            assert_eq!(engine.active_profile_name(), Some("default"));

            // Verify keybinds were applied
            assert!(engine.keybind_count() > 0);
        }
    }

    #[test]
    fn test_reset_to_defaults() {
        let mut engine = KeybindEngine::new();

        // Try multiple possible paths
        let possible_paths = vec![
            "../../../../config/keybinds/defaults.json",
            "projects/ricecoder/config/keybinds/defaults.json",
            "config/keybinds/defaults.json",
        ];

        let mut loaded = false;
        for path in possible_paths {
            if engine.load_defaults_from_file(path).is_ok() {
                loaded = true;
                break;
            }
        }

        if loaded {
            engine.apply_defaults().unwrap();

            let initial_count = engine.keybind_count();

            // Modify keybinds
            let custom_keybinds = vec![Keybind::new("custom.action", "Ctrl+Q", "custom", "Custom")];
            engine.apply_keybinds(custom_keybinds).unwrap();
            assert_eq!(engine.keybind_count(), 1);

            // Reset to defaults
            assert!(engine.reset_to_defaults().is_ok());
            assert_eq!(engine.keybind_count(), initial_count);

            // Verify defaults are restored
            let key_combo = KeyCombo::from_str("Ctrl+S").unwrap();
            assert_eq!(engine.get_action(&key_combo), Some("editor.save"));
        }
    }

    #[test]
    fn test_reset_to_defaults_without_loading() {
        let mut engine = KeybindEngine::new();
        assert!(engine.reset_to_defaults().is_err());
    }

    #[test]
    fn test_apply_defaults_without_loading() {
        let mut engine = KeybindEngine::new();
        assert!(engine.apply_defaults().is_err());
    }

    #[test]
    fn test_get_defaults() {
        let mut engine = KeybindEngine::new();

        // Try multiple possible paths
        let possible_paths = vec![
            "../../../../config/keybinds/defaults.json",
            "projects/ricecoder/config/keybinds/defaults.json",
            "config/keybinds/defaults.json",
        ];

        let mut loaded = false;
        for path in possible_paths {
            if engine.load_defaults_from_file(path).is_ok() {
                loaded = true;
                break;
            }
        }

        if loaded {
            let defaults = engine.get_defaults();

            assert!(!defaults.is_empty());
            for keybind in defaults {
                assert!(keybind.is_default);
            }
        }
    }

    #[test]
    fn test_initialize_engine_with_defaults() {
        // Try multiple possible paths
        let possible_paths = vec![
            "../../../../config/keybinds/defaults.json",
            "projects/ricecoder/config/keybinds/defaults.json",
            "config/keybinds/defaults.json",
        ];

        let mut engine_result = Err(EngineError::DefaultsLoadError("No path found".to_string()));
        for path in possible_paths {
            if let Ok(engine) = initialize_engine_with_defaults(path) {
                engine_result = Ok(engine);
                break;
            }
        }

        if let Ok(engine) = engine_result {
            assert!(engine.keybind_count() > 0);
            assert_eq!(engine.active_profile_name(), Some("default"));
        }
    }

    #[test]
    fn test_get_default_persistence() {
        use crate::Profile;

        let result = get_default_persistence();
        assert!(result.is_ok());

        let persistence = result.unwrap();

        // Verify the persistence is configured with the correct directory
        assert!(persistence.config_dir().exists());

        // Verify we can use it to save and load profiles
        let keybinds = vec![Keybind::new("editor.save", "Ctrl+S", "editing", "Save")];
        let profile = Profile::new("test_default_persistence", keybinds);

        assert!(persistence.save_profile(&profile).is_ok());
        assert!(persistence.load_profile("test_default_persistence").is_ok());

        // Clean up
        let _ = persistence.delete_profile("test_default_persistence");
    }
}
