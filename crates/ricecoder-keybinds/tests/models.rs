use ricecoder_keybinds::*;
use std::str::FromStr;

#[test]
fn test_modifier_from_str() {
    assert_eq!(Modifier::from_str("ctrl").unwrap(), Modifier::Ctrl);
    assert_eq!(Modifier::from_str("Shift").unwrap(), Modifier::Shift);
    assert_eq!(Modifier::from_str("alt").unwrap(), Modifier::Alt);
    assert_eq!(Modifier::from_str("meta").unwrap(), Modifier::Meta);
    assert_eq!(Modifier::from_str("cmd").unwrap(), Modifier::Meta);
}

#[test]
fn test_key_from_str() {
    assert_eq!(Key::from_str("enter").unwrap(), Key::Enter);
    assert_eq!(Key::from_str("F1").unwrap(), Key::F(1));
    assert_eq!(Key::from_str("a").unwrap(), Key::Char('a'));
    assert!(Key::from_str("F13").is_err());
}

#[test]
fn test_key_combo_from_str() {
    let combo = KeyCombo::from_str("Ctrl+S").unwrap();
    assert_eq!(combo.modifiers.len(), 1);
    assert_eq!(combo.modifiers[0], Modifier::Ctrl);
    assert_eq!(combo.key, Key::Char('s'));

    let combo = KeyCombo::from_str("Ctrl+Shift+Z").unwrap();
    assert_eq!(combo.modifiers.len(), 2);
    assert_eq!(combo.key, Key::Char('z'));
}

#[test]
fn test_keybind_creation() {
    let kb = Keybind::new("editor.save", "Ctrl+S", "editing", "Save file");
    assert_eq!(kb.action_id, "editor.save");
    assert_eq!(kb.key, "Ctrl+S");
    assert!(!kb.is_default);

    let kb = Keybind::new_default("editor.undo", "Ctrl+Z", "editing", "Undo");
    assert!(kb.is_default);
}

#[test]
fn test_keybind_manager_trait() {
    struct MockKeybindManager {
        bindings: std::collections::HashMap<String, Keybind>,
    }

    impl MockKeybindManager {
        fn new() -> Self {
            Self {
                bindings: std::collections::HashMap::new(),
            }
        }
    }

    impl KeybindManager for MockKeybindManager {
        fn bind(
            &mut self,
            action: String,
            key_combo: KeyCombo,
        ) -> Result<(), crate::error::RegistryError> {
            let keybind =
                Keybind::new(action.clone(), key_combo.to_string(), "test", "Test action");
            self.bindings.insert(action, keybind);
            Ok(())
        }

        fn get_binding(&self, action: &str) -> Option<&Keybind> {
            self.bindings.get(action)
        }

        fn resolve_action(&self, key_combo: &KeyCombo, _context: &Context) -> Option<&str> {
            for (action, keybind) in &self.bindings {
                if keybind.parse_key().ok().as_ref() == Some(key_combo) {
                    return Some(action);
                }
            }
            None
        }
    }

    let mut manager = MockKeybindManager::new();
    let combo = KeyCombo::from_str("Ctrl+S").unwrap();

    // Test binding
    manager.bind("save".to_string(), combo.clone()).unwrap();

    // Test getting binding
    let binding = manager.get_binding("save");
    assert!(binding.is_some());
    assert_eq!(binding.unwrap().action_id, "save");

    // Test resolving action
    let action = manager.resolve_action(&combo, &Context::Global);
    assert_eq!(action, Some("save"));

    // Test non-existent action
    let action = manager.resolve_action(&KeyCombo::from_str("Ctrl+X").unwrap(), &Context::Global);
    assert!(action.is_none());
}
