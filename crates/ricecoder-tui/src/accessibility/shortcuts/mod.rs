//! Keyboard shortcut customization and help system

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use std::collections::HashMap;

/// Keyboard shortcut customizer
#[derive(Debug, Clone)]
pub struct KeyboardShortcutCustomizer {
    /// Default shortcuts
    defaults: HashMap<String, Vec<KeyEvent>>,
    /// User customizations
    customizations: HashMap<String, Vec<KeyEvent>>,
    /// Conflicts checker
    conflicts: HashMap<Vec<KeyEvent>, Vec<String>>,
}

impl KeyboardShortcutCustomizer {
    pub fn new() -> Self {
        let mut defaults = HashMap::new();

        defaults.insert(
            "mode.chat".to_string(),
            vec![KeyEvent {
                code: KeyCode::Char('1'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: KeyEventState::empty(),
            }],
        );

        defaults.insert(
            "mode.command".to_string(),
            vec![KeyEvent {
                code: KeyCode::Char('2'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: KeyEventState::empty(),
            }],
        );

        defaults.insert(
            "focus.next".to_string(),
            vec![KeyEvent {
                code: KeyCode::Tab,
                modifiers: KeyModifiers::empty(),
                kind: KeyEventKind::Press,
                state: KeyEventState::empty(),
            }],
        );

        defaults.insert(
            "focus.previous".to_string(),
            vec![KeyEvent {
                code: KeyCode::Tab,
                modifiers: KeyModifiers::SHIFT,
                kind: KeyEventKind::Press,
                state: KeyEventState::empty(),
            }],
        );

        Self {
            defaults,
            customizations: HashMap::new(),
            conflicts: HashMap::new(),
        }
    }

    /// Get shortcut for action
    pub fn get_shortcut(&self, action: &str) -> Option<&Vec<KeyEvent>> {
        self.customizations
            .get(action)
            .or_else(|| self.defaults.get(action))
    }

    /// Set custom shortcut for action
    pub fn set_shortcut(&mut self, action: String, keys: Vec<KeyEvent>) -> Result<(), String> {
        for (existing_keys, actions) in &self.conflicts {
            if existing_keys == &keys {
                return Err(format!("Shortcut conflicts with: {}", actions.join(", ")));
            }
        }

        if let Some(old_keys) = self.customizations.get(&action) {
            if let Some(actions) = self.conflicts.get_mut(old_keys) {
                actions.retain(|a| a != &action);
                if actions.is_empty() {
                    self.conflicts.remove(old_keys);
                }
            }
        }

        self.customizations.insert(action.clone(), keys.clone());
        self.conflicts
            .entry(keys)
            .or_insert_with(Vec::new)
            .push(action);

        Ok(())
    }

    /// Reset shortcut to default
    pub fn reset_shortcut(&mut self, action: &str) {
        if let Some(keys) = self.customizations.remove(action) {
            if let Some(actions) = self.conflicts.get_mut(&keys) {
                actions.retain(|a| a != action);
                if actions.is_empty() {
                    self.conflicts.remove(&keys);
                }
            }
        }
    }

    /// Get all available actions
    pub fn available_actions(&self) -> Vec<String> {
        let mut actions: Vec<String> = self.defaults.keys().cloned().collect();
        actions.sort();
        actions
    }
}

impl Default for KeyboardShortcutCustomizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Keyboard shortcut help system
#[derive(Debug, Default)]
pub struct KeyboardShortcutHelp {
    /// Available shortcuts organized by category
    shortcuts: HashMap<String, Vec<KeyboardShortcut>>,
    /// Current search filter
    search_filter: String,
}

/// Keyboard shortcut definition
#[derive(Debug, Clone)]
pub struct KeyboardShortcut {
    pub keys: String,
    pub description: String,
    pub category: String,
    pub context: Option<String>,
}

impl KeyboardShortcutHelp {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a keyboard shortcut
    pub fn add_shortcut(&mut self, shortcut: KeyboardShortcut) {
        self.shortcuts
            .entry(shortcut.category.clone())
            .or_insert_with(Vec::new)
            .push(shortcut);
    }

    /// Set search filter
    pub fn set_search_filter(&mut self, filter: String) {
        self.search_filter = filter;
    }

    /// Search shortcuts by description or keys
    pub fn search(&self, query: &str) -> Vec<&KeyboardShortcut> {
        let query_lower = query.to_lowercase();
        self.shortcuts
            .values()
            .flatten()
            .filter(|shortcut| {
                shortcut.description.to_lowercase().contains(&query_lower)
                    || shortcut.keys.to_lowercase().contains(&query_lower)
                    || shortcut.category.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    /// Clear all shortcuts
    pub fn clear(&mut self) {
        self.shortcuts.clear();
        self.search_filter.clear();
    }
}

/// Initialize default keyboard shortcuts for RiceCoder
pub fn initialize_default_shortcuts() -> Vec<KeyboardShortcut> {
    vec![
        KeyboardShortcut {
            keys: "Ctrl+C".to_string(),
            description: "Exit application".to_string(),
            category: "General".to_string(),
            context: None,
        },
        KeyboardShortcut {
            keys: "Ctrl+Z".to_string(),
            description: "Undo last action".to_string(),
            category: "General".to_string(),
            context: None,
        },
        KeyboardShortcut {
            keys: "Ctrl+Y".to_string(),
            description: "Redo last action".to_string(),
            category: "General".to_string(),
            context: None,
        },
        KeyboardShortcut {
            keys: "Ctrl+S".to_string(),
            description: "Save current file".to_string(),
            category: "File".to_string(),
            context: Some("when file is open".to_string()),
        },
        KeyboardShortcut {
            keys: "Ctrl+O".to_string(),
            description: "Open file".to_string(),
            category: "File".to_string(),
            context: None,
        },
        KeyboardShortcut {
            keys: "F1".to_string(),
            description: "Show help".to_string(),
            category: "Help".to_string(),
            context: None,
        },
        KeyboardShortcut {
            keys: "Ctrl+F".to_string(),
            description: "Find in file".to_string(),
            category: "Search".to_string(),
            context: Some("when file is open".to_string()),
        },
        KeyboardShortcut {
            keys: "Ctrl+P".to_string(),
            description: "Command palette".to_string(),
            category: "Navigation".to_string(),
            context: None,
        },
    ]
}
