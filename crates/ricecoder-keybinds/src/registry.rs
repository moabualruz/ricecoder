//! Keybind registry with fast lookup capabilities

use std::collections::HashMap;

use crate::error::RegistryError;
use crate::models::{Keybind, KeyCombo};

/// Registry for storing and looking up keybinds
pub struct KeybindRegistry {
    /// Map from action_id to keybind
    by_action: HashMap<String, Keybind>,
    /// Map from key_combo to action_id
    by_key: HashMap<String, String>,
}

impl KeybindRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        KeybindRegistry {
            by_action: HashMap::new(),
            by_key: HashMap::new(),
        }
    }

    /// Register a keybind
    pub fn register(&mut self, keybind: Keybind) -> Result<(), RegistryError> {
        // Validate action_id format
        if keybind.action_id.is_empty() {
            return Err(RegistryError::InvalidActionIdFormat(
                "Action ID cannot be empty".to_string(),
            ));
        }

        // Parse the key combination
        let key_combo = keybind.parse_key().map_err(|e| {
            RegistryError::InvalidActionIdFormat(format!("Invalid key: {}", e))
        })?;

        let key_str = key_combo.to_string();

        // Check for duplicate key combinations
        if let Some(existing_action) = self.by_key.get(&key_str) {
            if existing_action != &keybind.action_id {
                return Err(RegistryError::DuplicateActionId(format!(
                    "Key {} already bound to {}",
                    key_str, existing_action
                )));
            }
        }

        // Register the keybind
        let action_id = keybind.action_id.clone();
        self.by_action.insert(action_id.clone(), keybind);
        self.by_key.insert(key_str, action_id);

        Ok(())
    }

    /// Lookup keybind by action ID
    pub fn lookup_by_action(&self, action_id: &str) -> Option<&Keybind> {
        self.by_action.get(action_id)
    }

    /// Lookup action ID by key combination
    pub fn lookup_by_key(&self, key: &KeyCombo) -> Option<&str> {
        let key_str = key.to_string();
        self.by_key.get(&key_str).map(|s| s.as_str())
    }

    /// Get all keybinds
    pub fn all_keybinds(&self) -> Vec<&Keybind> {
        self.by_action.values().collect()
    }

    /// Get number of registered keybinds
    pub fn len(&self) -> usize {
        self.by_action.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.by_action.is_empty()
    }

    /// Clear all keybinds
    pub fn clear(&mut self) {
        self.by_action.clear();
        self.by_key.clear();
    }

    /// Get all keybinds for a category
    pub fn keybinds_by_category(&self, category: &str) -> Vec<&Keybind> {
        self.by_action
            .values()
            .filter(|kb| kb.category == category)
            .collect()
    }

    /// Get all unique categories
    pub fn categories(&self) -> Vec<String> {
        let mut categories: Vec<String> = self
            .by_action
            .values()
            .map(|kb| kb.category.clone())
            .collect();
        categories.sort();
        categories.dedup();
        categories
    }
}

impl Default for KeybindRegistry {
    fn default() -> Self {
        Self::new()
    }
}

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

    use std::str::FromStr;
}
