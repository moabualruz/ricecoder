//! Keybind registry with fast lookup capabilities

use std::collections::HashMap;

use crate::{
    error::RegistryError,
    models::{Context, KeyCombo, Keybind},
};

/// Registry for storing and looking up keybinds
pub struct KeybindRegistry {
    /// Map from action_id to keybind
    by_action: HashMap<String, Keybind>,
    /// Map from (context, key_combo) to action_id
    by_key_context: HashMap<(Context, String), String>,
    /// Map from key_combo to action_id (for global keybinds)
    by_key_global: HashMap<String, String>,
}

impl KeybindRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        KeybindRegistry {
            by_action: HashMap::new(),
            by_key_context: HashMap::new(),
            by_key_global: HashMap::new(),
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

        // Parse all key combinations (primary + alternatives)
        let key_combos = keybind
            .parse_all_keys()
            .map_err(|e| RegistryError::InvalidActionIdFormat(format!("Invalid key: {}", e)))?;

        // Register the keybind by action
        let action_id = keybind.action_id.clone();
        self.by_action.insert(action_id.clone(), keybind.clone());

        // Register each key combination
        for key_combo in key_combos {
            let key_str = key_combo.to_string();

            // Register by key and context
            if keybind.contexts.is_empty() {
                // Global keybind
                if let Some(existing_action) = self.by_key_global.get(&key_str) {
                    if existing_action != &keybind.action_id {
                        return Err(RegistryError::DuplicateActionId(format!(
                            "Global key {} already bound to {}",
                            key_str, existing_action
                        )));
                    }
                }
                self.by_key_global.insert(key_str, action_id.clone());
            } else {
                // Context-specific keybinds
                for &context in &keybind.contexts {
                    let key = (context, key_str.clone());
                    if let Some(existing_action) = self.by_key_context.get(&key) {
                        if existing_action != &keybind.action_id {
                            return Err(RegistryError::DuplicateActionId(format!(
                                "Key {} in context {} already bound to {}",
                                key_str, context, existing_action
                            )));
                        }
                    }
                    self.by_key_context.insert(key, action_id.clone());
                }
            }
        }

        Ok(())
    }

    /// Lookup keybind by action ID
    pub fn lookup_by_action(&self, action_id: &str) -> Option<&Keybind> {
        self.by_action.get(action_id)
    }

    /// Lookup action ID by key combination (legacy - uses global context)
    pub fn lookup_by_key(&self, key: &KeyCombo) -> Option<&str> {
        self.lookup_by_key_in_context(key, &Context::Global)
    }

    /// Lookup action ID by key combination in a specific context
    pub fn lookup_by_key_in_context(&self, key: &KeyCombo, context: &Context) -> Option<&str> {
        let key_str = key.to_string();

        // First try context-specific lookup
        if let Some(action_id) = self.by_key_context.get(&(*context, key_str.clone())) {
            return Some(action_id.as_str());
        }

        // Fall back to global lookup
        self.by_key_global.get(&key_str).map(|s| s.as_str())
    }

    /// Lookup action ID by key combination with context hierarchy
    /// Searches from most specific to least specific context
    pub fn lookup_by_key_with_contexts(
        &self,
        key: &KeyCombo,
        contexts: &[Context],
    ) -> Option<&str> {
        let key_str = key.to_string();

        // Sort contexts by priority (highest first)
        let mut sorted_contexts = contexts.to_vec();
        sorted_contexts.sort_by_key(|b| std::cmp::Reverse(b.priority()));

        // Try context-specific lookups in priority order
        for context in &sorted_contexts {
            if let Some(action_id) = self.by_key_context.get(&(*context, key_str.clone())) {
                return Some(action_id.as_str());
            }
        }

        // Fall back to global lookup
        self.by_key_global.get(&key_str).map(|s| s.as_str())
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

    /// Unregister a keybind by action ID
    pub fn unregister(&mut self, action_id: &str) -> Result<(), RegistryError> {
        if let Some(keybind) = self.by_action.remove(action_id) {
            // Remove all key combinations from mappings
            let key_combos = keybind
                .parse_all_keys()
                .map_err(|e| RegistryError::InvalidActionIdFormat(format!("Invalid key: {}", e)))?;

            for key_combo in key_combos {
                let key_str = key_combo.to_string();

                if keybind.contexts.is_empty() {
                    self.by_key_global.remove(&key_str);
                } else {
                    for context in &keybind.contexts {
                        self.by_key_context.remove(&(*context, key_str.clone()));
                    }
                }
            }
        }
        Ok(())
    }

    /// Update an existing keybind
    pub fn update(&mut self, action_id: &str, new_keybind: Keybind) -> Result<(), RegistryError> {
        // First unregister the old keybind
        self.unregister(action_id)?;

        // Then register the new one
        self.register(new_keybind)
    }

    /// Clear all keybinds
    pub fn clear(&mut self) {
        self.by_action.clear();
        self.by_key_context.clear();
        self.by_key_global.clear();
    }

    /// Get all keybinds for a category
    pub fn keybinds_by_category(&self, category: &str) -> Vec<&Keybind> {
        self.by_action
            .values()
            .filter(|kb| kb.category == category)
            .collect()
    }

    /// Get all keybinds for a specific context
    pub fn keybinds_by_context(&self, context: &Context) -> Vec<&Keybind> {
        self.by_action
            .values()
            .filter(|kb| kb.applies_to_context(context))
            .collect()
    }

    /// Get all keybinds that apply to any of the given contexts
    pub fn keybinds_for_contexts(&self, contexts: &[Context]) -> Vec<&Keybind> {
        self.by_action
            .values()
            .filter(|kb| kb.applies_to_any_context(contexts))
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
