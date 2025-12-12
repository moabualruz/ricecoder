//! Context-aware keybind merging
//!
//! This module provides merging logic for keybinds that respects context hierarchies
//! and allows user customizations to override defaults appropriately.

use crate::error::EngineError;
use crate::models::{Context, Keybind};
use std::collections::HashMap;

/// Result of merging keybinds
#[derive(Debug, Clone)]
pub struct MergeResult {
    /// Successfully merged keybinds
    pub merged: Vec<Keybind>,
    /// Conflicts that were resolved
    pub resolved_conflicts: Vec<MergeConflict>,
    /// Conflicts that could not be resolved
    pub unresolved_conflicts: Vec<MergeConflict>,
}

/// A conflict between keybinds
#[derive(Debug, Clone)]
pub struct MergeConflict {
    /// The key combination that conflicts
    pub key: String,
    /// The context where the conflict occurs
    pub context: Context,
    /// The default keybind
    pub default: Keybind,
    /// The user keybind that conflicts
    pub user: Keybind,
    /// Suggested resolution
    pub resolution: ConflictResolution,
}

/// How to resolve a conflict
#[derive(Debug, Clone)]
pub enum ConflictResolution {
    /// Keep the user keybind
    KeepUser,
    /// Keep the default keybind
    KeepDefault,
    /// Change the user keybind to a different key
    RemapUser(String),
    /// Change the default keybind to a different key
    RemapDefault(String),
}

/// Keybind merger for context-aware merging
pub struct KeybindMerger;

impl KeybindMerger {
    /// Merge user keybinds with defaults, respecting contexts
    pub fn merge_with_contexts(
        defaults: &[Keybind],
        user_keybinds: &[Keybind],
    ) -> Result<MergeResult, EngineError> {
        let mut merged = Vec::new();
        let mut resolved_conflicts = Vec::new();
        let mut unresolved_conflicts = Vec::new();

        // Group keybinds by (key, context) for conflict detection
        let mut key_context_map: HashMap<(String, Context), Vec<&Keybind>> = HashMap::new();

        // Add all defaults
        for keybind in defaults {
            if keybind.contexts.is_empty() {
                // Global keybind
                key_context_map
                    .entry((keybind.key.clone(), Context::Global))
                    .or_insert_with(Vec::new)
                    .push(keybind);
            } else {
                // Context-specific keybinds
                for &context in &keybind.contexts {
                    key_context_map
                        .entry((keybind.key.clone(), context))
                        .or_insert_with(Vec::new)
                        .push(keybind);
                }
            }
        }

        // Add user keybinds, checking for conflicts
        for user_keybind in user_keybinds {
            if user_keybind.contexts.is_empty() {
                // Global user keybind
                Self::check_and_add_keybind(
                    user_keybind,
                    Context::Global,
                    &mut key_context_map,
                    &mut resolved_conflicts,
                    &mut unresolved_conflicts,
                );
            } else {
                // Context-specific user keybinds
                for &context in &user_keybind.contexts {
                    Self::check_and_add_keybind(
                        user_keybind,
                        context,
                        &mut key_context_map,
                        &mut resolved_conflicts,
                        &mut unresolved_conflicts,
                    );
                }
            }
        }

        // Build final merged list
        for keybinds in key_context_map.values() {
            // For each (key, context), we should have at most one keybind after conflict resolution
            if keybinds.len() == 1 {
                merged.push((*keybinds[0]).clone());
            } else if keybinds.len() > 1 {
                // This shouldn't happen after conflict resolution, but handle it
                // Prioritize user keybinds over defaults
                let user_keybind = keybinds
                    .iter()
                    .find(|kb| !kb.is_default)
                    .or_else(|| keybinds.first());

                if let Some(keybind) = user_keybind {
                    merged.push((*keybind).clone());
                }
            }
        }

        Ok(MergeResult {
            merged,
            resolved_conflicts,
            unresolved_conflicts,
        })
    }

    /// Check for conflicts and add keybind to the map
    fn check_and_add_keybind<'a>(
        keybind: &'a Keybind,
        context: Context,
        key_context_map: &mut HashMap<(String, Context), Vec<&'a Keybind>>,
        resolved_conflicts: &mut Vec<MergeConflict>,
        unresolved_conflicts: &mut Vec<MergeConflict>,
    ) {
        let key = (keybind.key.clone(), context);

        if let Some(existing) = key_context_map.get(&key) {
            // There's already a keybind for this (key, context)
            if let Some(existing_kb) = existing.first() {
                if existing_kb.is_default && !keybind.is_default {
                    // User keybind overrides default - this is expected
                    resolved_conflicts.push(MergeConflict {
                        key: keybind.key.clone(),
                        context,
                        default: (*existing_kb).clone(),
                        user: keybind.clone(),
                        resolution: ConflictResolution::KeepUser,
                    });
                    // Replace the default with user keybind
                    key_context_map.insert(key, vec![keybind]);
                } else if !existing_kb.is_default && keybind.is_default {
                    // Default trying to override user - keep user
                    resolved_conflicts.push(MergeConflict {
                        key: keybind.key.clone(),
                        context,
                        default: keybind.clone(),
                        user: (*existing_kb).clone(),
                        resolution: ConflictResolution::KeepUser,
                    });
                    // Keep existing user keybind
                } else if existing_kb.is_default && keybind.is_default {
                    // Two defaults - this shouldn't happen, but keep the new one
                    key_context_map.insert(key, vec![keybind]);
                } else {
                    // Two user keybinds - conflict
                    unresolved_conflicts.push(MergeConflict {
                        key: keybind.key.clone(),
                        context,
                        default: (*existing_kb).clone(),
                        user: keybind.clone(),
                        resolution: ConflictResolution::RemapUser("Suggest alternative key".to_string()),
                    });
                }
            }
        } else {
            // No conflict, add the keybind
            key_context_map.insert(key, vec![keybind]);
        }
    }

    /// Merge keybinds without context awareness (legacy)
    pub fn merge_simple(defaults: &[Keybind], user_keybinds: &[Keybind]) -> Vec<Keybind> {
        let mut merged = Vec::new();
        let mut user_keys = std::collections::HashSet::new();

        // Collect user key combinations
        for user_kb in user_keybinds {
            user_keys.insert(&user_kb.key);
        }

        // Add defaults that don't conflict with user keybinds
        for default_kb in defaults {
            if !user_keys.contains(&default_kb.key) {
                merged.push(default_kb.clone());
            }
        }

        // Add all user keybinds
        merged.extend(user_keybinds.iter().cloned());

        merged
    }

    /// Suggest alternative keys for resolving conflicts
    pub fn suggest_alternative_keys(
        conflict_key: &str,
        existing_keys: &[String],
    ) -> Vec<String> {
        // Simple suggestion logic - in practice, this could be more sophisticated
        let alternatives = vec![
            format!("Ctrl+{}", conflict_key.to_uppercase()),
            format!("Alt+{}", conflict_key.to_uppercase()),
            format!("Shift+{}", conflict_key.to_uppercase()),
            format!("{}+F1", conflict_key),
        ];

        alternatives
            .into_iter()
            .filter(|alt| !existing_keys.contains(alt))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Context;

    #[test]
    fn test_merge_no_conflicts() {
        let defaults = vec![
            Keybind::new_default("action1", "Ctrl+A", "global", "Action 1"),
            Keybind::new_default("action2", "Ctrl+B", "global", "Action 2"),
        ];

        let user = vec![
            Keybind::new("action3", "Ctrl+C", "global", "Action 3"),
        ];

        let result = KeybindMerger::merge_with_contexts(&defaults, &user).unwrap();

        assert_eq!(result.merged.len(), 3);
        assert!(result.unresolved_conflicts.is_empty());
    }

    #[test]
    fn test_merge_user_overrides_default() {
        let defaults = vec![
            Keybind::new_default("action1", "Ctrl+A", "global", "Default Action 1"),
        ];

        let user = vec![
            Keybind::new("action1", "Ctrl+A", "global", "User Action 1"),
        ];

        let result = KeybindMerger::merge_with_contexts(&defaults, &user).unwrap();

        assert_eq!(result.merged.len(), 1);
        assert_eq!(result.merged[0].description, "User Action 1");
        assert_eq!(result.resolved_conflicts.len(), 1);
    }

    #[test]
    fn test_merge_context_specific() {
        let defaults = vec![
            Keybind::new_default_with_contexts(
                "action1",
                "Ctrl+A",
                "global",
                "Global Action",
                vec![Context::Global],
            ),
        ];

        let user = vec![
            Keybind::new_with_contexts(
                "action1",
                "Ctrl+A",
                "input",
                "Input Action",
                vec![Context::Input],
            ),
        ];

        let result = KeybindMerger::merge_with_contexts(&defaults, &user).unwrap();

        // Should have both - they apply to different contexts
        assert_eq!(result.merged.len(), 2);
        assert!(result.unresolved_conflicts.is_empty());
    }

    #[test]
    fn test_merge_context_conflict() {
        let defaults = vec![
            Keybind::new_default_with_contexts(
                "action1",
                "Ctrl+A",
                "global",
                "Default Action",
                vec![Context::Input],
            ),
        ];

        let user = vec![
            Keybind::new_with_contexts(
                "action1",
                "Ctrl+A",
                "input",
                "User Action",
                vec![Context::Input],
            ),
        ];

        let result = KeybindMerger::merge_with_contexts(&defaults, &user).unwrap();

        // User should override default in same context
        assert_eq!(result.merged.len(), 1);
        assert_eq!(result.merged[0].description, "User Action");
        assert_eq!(result.resolved_conflicts.len(), 1);
    }
}