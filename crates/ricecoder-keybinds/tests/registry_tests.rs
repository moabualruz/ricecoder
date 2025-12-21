use ricecoder_keybinds::*;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_keybind() {
        let mut registry = KeybindRegistry::new();
        let kb = Keybind::new("editor.save", "Ctrl+S", "editing", "Save file");
        assert!(registry.register(kb).is_ok());
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_lookup_by_action() {
        let mut registry = KeybindRegistry::new();
        let kb = Keybind::new("editor.save", "Ctrl+S", "editing", "Save file");
        registry.register(kb).unwrap();

        let found = registry.lookup_by_action("editor.save");
        assert!(found.is_some());
        assert_eq!(found.unwrap().key, "Ctrl+S");
    }

    #[test]
    fn test_lookup_by_key() {
        let mut registry = KeybindRegistry::new();
        let kb = Keybind::new("editor.save", "Ctrl+S", "editing", "Save file");
        registry.register(kb).unwrap();

        let key_combo = KeyCombo::from_str("Ctrl+S").unwrap();
        let action = registry.lookup_by_key(&key_combo);
        assert_eq!(action, Some("editor.save"));
    }

    #[test]
    fn test_duplicate_key_detection() {
        let mut registry = KeybindRegistry::new();
        let kb1 = Keybind::new("editor.save", "Ctrl+S", "editing", "Save file");
        let kb2 = Keybind::new("editor.save_all", "Ctrl+S", "editing", "Save all");

        registry.register(kb1).unwrap();
        assert!(registry.register(kb2).is_err());
    }

    #[test]
    fn test_all_keybinds() {
        let mut registry = KeybindRegistry::new();
        registry
            .register(Keybind::new("editor.save", "Ctrl+S", "editing", "Save"))
            .unwrap();
        registry
            .register(Keybind::new("editor.undo", "Ctrl+Z", "editing", "Undo"))
            .unwrap();

        let all = registry.all_keybinds();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_keybinds_by_category() {
        let mut registry = KeybindRegistry::new();
        registry
            .register(Keybind::new("editor.save", "Ctrl+S", "editing", "Save"))
            .unwrap();
        registry
            .register(Keybind::new("nav.next", "Tab", "navigation", "Next"))
            .unwrap();

        let editing = registry.keybinds_by_category("editing");
        assert_eq!(editing.len(), 1);
        assert_eq!(editing[0].action_id, "editor.save");
    }

    #[test]
    fn test_categories() {
        let mut registry = KeybindRegistry::new();
        registry
            .register(Keybind::new("editor.save", "Ctrl+S", "editing", "Save"))
            .unwrap();
        registry
            .register(Keybind::new("nav.next", "Tab", "navigation", "Next"))
            .unwrap();
        registry
            .register(Keybind::new("editor.undo", "Ctrl+Z", "editing", "Undo"))
            .unwrap();

        let categories = registry.categories();
        assert_eq!(categories.len(), 2);
        assert!(categories.contains(&"editing".to_string()));
        assert!(categories.contains(&"navigation".to_string()));
    }
}
