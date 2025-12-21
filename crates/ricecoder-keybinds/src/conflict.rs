//! Conflict detection and resolution for keybinds

use std::collections::HashMap;

use crate::models::{KeyCombo, Keybind};

/// Represents a conflict between multiple keybinds
#[derive(Debug, Clone)]
pub struct Conflict {
    pub key_combo: KeyCombo,
    pub actions: Vec<String>,
}

/// Represents a suggested resolution for a conflict
#[derive(Debug, Clone)]
pub struct Resolution {
    pub action_id: String,
    pub suggested_key: String,
    pub reason: String,
}

/// Detects and suggests resolutions for keybind conflicts
pub struct ConflictDetector;

impl ConflictDetector {
    /// Detect all conflicts in a set of keybinds
    pub fn detect(keybinds: &[Keybind]) -> Vec<Conflict> {
        let mut key_to_actions: HashMap<String, Vec<String>> = HashMap::new();

        // Build reverse index
        for keybind in keybinds {
            if let Ok(key_combo) = keybind.parse_key() {
                let key_str = key_combo.to_string();
                key_to_actions
                    .entry(key_str)
                    .or_default()
                    .push(keybind.action_id.clone());
            }
        }

        // Find conflicts (keys with multiple actions)
        let mut conflicts = Vec::new();
        for (key_str, actions) in key_to_actions {
            if actions.len() > 1 {
                if let Ok(key_combo) = key_str.parse() {
                    conflicts.push(Conflict { key_combo, actions });
                }
            }
        }

        conflicts
    }

    /// Suggest resolutions for a conflict
    pub fn suggest_resolution(conflict: &Conflict, keybinds: &[Keybind]) -> Vec<Resolution> {
        let mut suggestions = Vec::new();

        // Get category information for conflicting actions
        let action_categories: HashMap<String, String> = keybinds
            .iter()
            .filter(|kb| conflict.actions.contains(&kb.action_id))
            .map(|kb| (kb.action_id.clone(), kb.category.clone()))
            .collect();

        // Suggest alternatives based on category
        for action_id in &conflict.actions {
            let category = action_categories
                .get(action_id)
                .map(|s| s.as_str())
                .unwrap_or("general");

            let suggested_key = Self::suggest_alternative_key(category);
            suggestions.push(Resolution {
                action_id: action_id.clone(),
                suggested_key,
                reason: format!("Suggested alternative for {} action", category),
            });
        }

        suggestions
    }

    /// Suggest an alternative key based on category
    fn suggest_alternative_key(category: &str) -> String {
        match category {
            "editing" => "Ctrl+Alt+S".to_string(),
            "navigation" => "Ctrl+Alt+N".to_string(),
            "search" => "Ctrl+Alt+F".to_string(),
            "view" => "Ctrl+Alt+V".to_string(),
            _ => "Ctrl+Alt+X".to_string(),
        }
    }
}
