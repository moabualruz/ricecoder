//! Conflict detection and resolution for keybinds

use std::collections::HashMap;

use crate::models::{Keybind, KeyCombo};

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
                    conflicts.push(Conflict {
                        key_combo,
                        actions,
                    });
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
                reason: format!(
                    "Suggested alternative for {} action",
                    category
                ),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_no_conflicts() {
        let keybinds = vec![
            Keybind::new("editor.save", "Ctrl+S", "editing", "Save"),
            Keybind::new("editor.undo", "Ctrl+Z", "editing", "Undo"),
        ];

        let conflicts = ConflictDetector::detect(&keybinds);
        assert_eq!(conflicts.len(), 0);
    }

    #[test]
    fn test_detect_conflicts() {
        let keybinds = vec![
            Keybind::new("editor.save", "Ctrl+S", "editing", "Save"),
            Keybind::new("editor.save_all", "Ctrl+S", "editing", "Save all"),
        ];

        let conflicts = ConflictDetector::detect(&keybinds);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].actions.len(), 2);
        assert!(conflicts[0].actions.contains(&"editor.save".to_string()));
        assert!(conflicts[0].actions.contains(&"editor.save_all".to_string()));
    }

    #[test]
    fn test_suggest_resolution() {
        let keybinds = vec![
            Keybind::new("editor.save", "Ctrl+S", "editing", "Save"),
            Keybind::new("editor.save_all", "Ctrl+S", "editing", "Save all"),
        ];

        let conflicts = ConflictDetector::detect(&keybinds);
        assert_eq!(conflicts.len(), 1);

        let resolutions = ConflictDetector::suggest_resolution(&conflicts[0], &keybinds);
        assert_eq!(resolutions.len(), 2);
        assert!(resolutions.iter().any(|r| r.action_id == "editor.save"));
        assert!(resolutions.iter().any(|r| r.action_id == "editor.save_all"));
    }

    #[test]
    fn test_multiple_conflicts() {
        let keybinds = vec![
            Keybind::new("editor.save", "Ctrl+S", "editing", "Save"),
            Keybind::new("editor.save_all", "Ctrl+S", "editing", "Save all"),
            Keybind::new("nav.next", "Tab", "navigation", "Next"),
            Keybind::new("nav.prev", "Tab", "navigation", "Previous"),
        ];

        let conflicts = ConflictDetector::detect(&keybinds);
        assert_eq!(conflicts.len(), 2);
    }

    #[test]
    fn test_empty_keybind_set() {
        let keybinds: Vec<Keybind> = vec![];
        let conflicts = ConflictDetector::detect(&keybinds);
        assert_eq!(conflicts.len(), 0);
    }

    #[test]
    fn test_single_keybind() {
        let keybinds = vec![Keybind::new("editor.save", "Ctrl+S", "editing", "Save")];
        let conflicts = ConflictDetector::detect(&keybinds);
        assert_eq!(conflicts.len(), 0);
    }

    #[test]
    fn test_three_way_conflict() {
        let keybinds = vec![
            Keybind::new("action1", "Ctrl+S", "editing", "Action 1"),
            Keybind::new("action2", "Ctrl+S", "editing", "Action 2"),
            Keybind::new("action3", "Ctrl+S", "editing", "Action 3"),
        ];

        let conflicts = ConflictDetector::detect(&keybinds);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].actions.len(), 3);
        assert!(conflicts[0].actions.contains(&"action1".to_string()));
        assert!(conflicts[0].actions.contains(&"action2".to_string()));
        assert!(conflicts[0].actions.contains(&"action3".to_string()));
    }

    #[test]
    fn test_resolution_suggestions_editing() {
        let keybinds = vec![
            Keybind::new("editor.save", "Ctrl+S", "editing", "Save"),
            Keybind::new("editor.save_all", "Ctrl+S", "editing", "Save all"),
        ];

        let conflicts = ConflictDetector::detect(&keybinds);
        assert_eq!(conflicts.len(), 1);

        let resolutions = ConflictDetector::suggest_resolution(&conflicts[0], &keybinds);
        assert_eq!(resolutions.len(), 2);

        // Both should suggest editing category alternatives
        for resolution in &resolutions {
            assert!(resolution.reason.contains("editing"));
            assert!(resolution.suggested_key.contains("Ctrl+Alt"));
        }
    }

    #[test]
    fn test_resolution_suggestions_navigation() {
        let keybinds = vec![
            Keybind::new("nav.next", "Tab", "navigation", "Next"),
            Keybind::new("nav.prev", "Tab", "navigation", "Previous"),
        ];

        let conflicts = ConflictDetector::detect(&keybinds);
        assert_eq!(conflicts.len(), 1);

        let resolutions = ConflictDetector::suggest_resolution(&conflicts[0], &keybinds);
        assert_eq!(resolutions.len(), 2);

        // Both should suggest navigation category alternatives
        for resolution in &resolutions {
            assert!(resolution.reason.contains("navigation"));
        }
    }

    #[test]
    fn test_resolution_suggestions_mixed_categories() {
        let keybinds = vec![
            Keybind::new("editor.save", "Ctrl+S", "editing", "Save"),
            Keybind::new("search.find", "Ctrl+S", "search", "Find"),
        ];

        let conflicts = ConflictDetector::detect(&keybinds);
        assert_eq!(conflicts.len(), 1);

        let resolutions = ConflictDetector::suggest_resolution(&conflicts[0], &keybinds);
        assert_eq!(resolutions.len(), 2);

        // Each should have a different category in the reason
        let reasons: Vec<String> = resolutions.iter().map(|r| r.reason.clone()).collect();
        assert!(reasons.iter().any(|r| r.contains("editing")));
        assert!(reasons.iter().any(|r| r.contains("search")));
    }

    #[test]
    fn test_invalid_key_syntax_ignored() {
        let mut keybinds = vec![
            Keybind::new("editor.save", "Ctrl+S", "editing", "Save"),
            Keybind::new("editor.undo", "Ctrl+Z", "editing", "Undo"),
        ];

        // Add a keybind with invalid key syntax
        keybinds.push(Keybind::new("invalid", "InvalidKey", "editing", "Invalid"));

        // Should only detect valid keybinds
        let conflicts = ConflictDetector::detect(&keybinds);
        assert_eq!(conflicts.len(), 0);
    }

    #[test]
    fn test_conflict_with_many_keybinds() {
        let mut keybinds = vec![
            Keybind::new("action1", "Ctrl+A", "editing", "Action 1"),
            Keybind::new("action2", "Ctrl+B", "editing", "Action 2"),
            Keybind::new("action3", "Ctrl+C", "editing", "Action 3"),
            Keybind::new("action4", "Ctrl+D", "editing", "Action 4"),
            Keybind::new("action5", "Ctrl+E", "editing", "Action 5"),
        ];

        // Add conflicting keybinds
        keybinds.push(Keybind::new("conflict1", "Ctrl+A", "editing", "Conflict 1"));
        keybinds.push(Keybind::new("conflict2", "Ctrl+B", "editing", "Conflict 2"));

        let conflicts = ConflictDetector::detect(&keybinds);
        assert_eq!(conflicts.len(), 2);

        // Verify each conflict has exactly 2 actions
        for conflict in &conflicts {
            assert_eq!(conflict.actions.len(), 2);
        }
    }
}
