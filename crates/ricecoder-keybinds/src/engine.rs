//! Keybind engine that combines registry and profile management

use crate::conflict::ConflictDetector;
use crate::error::EngineError;
use crate::help::KeybindHelp;
use crate::merge::KeybindMerger;
use crate::models::{Context, Keybind, KeyCombo};
use crate::parser::ParserRegistry;
use crate::persistence::KeybindPersistence;
use crate::profile::ProfileManager;
use crate::registry::KeybindRegistry;
use std::path::Path;

/// Main keybind engine combining registry and profile management
pub struct KeybindEngine {
    registry: KeybindRegistry,
    profile_manager: ProfileManager,
    default_keybinds: Vec<Keybind>,
    current_context: Context,
    context_stack: Vec<Context>,
}

impl KeybindEngine {
    /// Create a new keybind engine
    pub fn new() -> Self {
        KeybindEngine {
            registry: KeybindRegistry::new(),
            profile_manager: ProfileManager::new(),
            default_keybinds: Vec::new(),
            current_context: Context::Global,
            context_stack: Vec::new(),
        }
    }

    /// Load default keybinds from a JSON file
    pub fn load_defaults_from_file(&mut self, path: impl AsRef<Path>) -> Result<(), EngineError> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            EngineError::DefaultsLoadError(format!("Failed to read defaults file: {}", e))
        })?;

        let parser = ParserRegistry::new();
        let keybinds = parser.parse(&content, "json").map_err(|e| {
            EngineError::DefaultsLoadError(format!("Failed to parse defaults: {}", e))
        })?;

        self.default_keybinds = keybinds.clone();
        Ok(())
    }

    /// Apply default keybinds and create default profile if needed
    pub fn apply_defaults(&mut self) -> Result<(), EngineError> {
        if self.default_keybinds.is_empty() {
            return Err(EngineError::DefaultsLoadError(
                "No defaults loaded".to_string(),
            ));
        }

        // Create default profile if it doesn't exist
        if self.profile_manager.active_profile_name().is_none() {
            self.create_profile("default", self.default_keybinds.clone())?;
            self.select_profile("default")?;
        } else {
            // Apply defaults to current registry (no merging needed for defaults-only)
            self.apply_keybinds(self.default_keybinds.clone())?;
        }

        Ok(())
    }

    /// Reset keybinds to defaults
    pub fn reset_to_defaults(&mut self) -> Result<(), EngineError> {
        if self.default_keybinds.is_empty() {
            return Err(EngineError::DefaultsLoadError(
                "No defaults available for reset".to_string(),
            ));
        }

        // Verify all defaults have is_default flag set
        for keybind in &self.default_keybinds {
            if !keybind.is_default {
                return Err(EngineError::DefaultsLoadError(
                    "Invalid default keybind: is_default flag not set".to_string(),
                ));
            }
        }

        // Apply defaults to registry
        self.apply_keybinds(self.default_keybinds.clone())?;

        Ok(())
    }

    /// Get the default keybinds
    pub fn get_defaults(&self) -> &[Keybind] {
        &self.default_keybinds
    }

    /// Set the default keybinds (for testing purposes)
    pub fn set_defaults(&mut self, keybinds: Vec<Keybind>) {
        self.default_keybinds = keybinds;
    }

    /// Apply keybinds to the engine
    pub fn apply_keybinds(&mut self, keybinds: Vec<Keybind>) -> Result<(), EngineError> {
        self.registry.clear();

        for keybind in keybinds {
            self.registry.register(keybind)?;
        }

        Ok(())
    }

    /// Apply profile keybinds with merging against defaults
    pub fn apply_profile_with_merge(&mut self, profile_keybinds: &[Keybind]) -> Result<(), EngineError> {
        // Merge profile keybinds with defaults
        let merge_result = KeybindMerger::merge_with_contexts(&self.default_keybinds, profile_keybinds)
            .map_err(|e| EngineError::DefaultsLoadError(format!("Failed to merge keybinds: {}", e)))?;

        // Log resolved conflicts
        for conflict in &merge_result.resolved_conflicts {
            tracing::info!(
                "Resolved keybind conflict: {} in context {} - kept user binding",
                conflict.key, conflict.context
            );
        }

        // Log unresolved conflicts
        for conflict in &merge_result.unresolved_conflicts {
            tracing::warn!(
                "Unresolved keybind conflict: {} in context {} - {}",
                conflict.key, conflict.context, conflict.user.action_id
            );
        }

        // Apply merged keybinds
        self.apply_keybinds(merge_result.merged)?;

        Ok(())
    }

    /// Get action for a key combination (legacy - uses current context)
    pub fn get_action(&self, key: &KeyCombo) -> Option<&str> {
        self.get_action_in_context(key, &self.current_context)
    }

    /// Get action for a key combination in a specific context
    pub fn get_action_in_context(&self, key: &KeyCombo, context: &Context) -> Option<&str> {
        self.registry.lookup_by_key_in_context(key, context)
    }

    /// Get action for a key combination with context hierarchy
    pub fn get_action_with_contexts(&self, key: &KeyCombo, contexts: &[Context]) -> Option<&str> {
        self.registry.lookup_by_key_with_contexts(key, contexts)
    }

    /// Get keybind for an action
    pub fn get_keybind(&self, action_id: &str) -> Option<&Keybind> {
        self.registry.lookup_by_action(action_id)
    }

    /// Get all keybinds
    pub fn all_keybinds(&self) -> Vec<&Keybind> {
        self.registry.all_keybinds()
    }

    /// Get keybinds by category
    pub fn keybinds_by_category(&self, category: &str) -> Vec<&Keybind> {
        self.registry.keybinds_by_category(category)
    }

    /// Get all categories
    pub fn categories(&self) -> Vec<String> {
        self.registry.categories()
    }

    /// Create a new profile
    pub fn create_profile(
        &mut self,
        name: impl Into<String>,
        keybinds: Vec<Keybind>,
    ) -> Result<(), EngineError> {
        self.profile_manager.create_profile(name, keybinds)?;
        Ok(())
    }

    /// Select a profile and apply its keybinds immediately
    pub fn select_profile(&mut self, name: &str) -> Result<(), EngineError> {
        self.profile_manager.select_profile(name)?;

        // Get the keybinds before applying (to avoid borrow checker issues)
        let keybinds = {
            let profile = self.profile_manager.get_active_profile()?;
            profile.keybinds.clone()
        };

        // Apply the selected profile's keybinds with merging
        self.apply_profile_with_merge(&keybinds)?;

        Ok(())
    }

    /// Delete a profile
    pub fn delete_profile(&mut self, name: &str) -> Result<(), EngineError> {
        self.profile_manager.delete_profile(name)?;
        Ok(())
    }

    /// Get active profile name
    pub fn active_profile_name(&self) -> Option<&str> {
        self.profile_manager.active_profile_name()
    }

    /// Get number of keybinds
    pub fn keybind_count(&self) -> usize {
        self.registry.len()
    }

    /// Check if engine has keybinds
    pub fn has_keybinds(&self) -> bool {
        !self.registry.is_empty()
    }

    /// Get the current context
    pub fn current_context(&self) -> &Context {
        &self.current_context
    }

    /// Set the current context
    pub fn set_context(&mut self, context: Context) {
        self.current_context = context;
    }

    /// Push a context onto the stack (for modal contexts)
    pub fn push_context(&mut self, context: Context) {
        self.context_stack.push(self.current_context);
        self.current_context = context;
    }

    /// Pop the top context from the stack
    pub fn pop_context(&mut self) -> Option<Context> {
        if let Some(previous) = self.context_stack.pop() {
            self.current_context = previous;
            Some(previous)
        } else {
            None
        }
    }

    /// Get the context stack (for debugging)
    pub fn context_stack(&self) -> &[Context] {
        &self.context_stack
    }

    /// Get all contexts that should be searched (current + stack)
    pub fn active_contexts(&self) -> Vec<Context> {
        let mut contexts = vec![self.current_context];
        contexts.extend(self.context_stack.iter().rev().copied());
        contexts
    }

    /// Get action for a key combination using active contexts
    pub fn get_action_with_active_contexts(&self, key: &KeyCombo) -> Option<&str> {
        let contexts = self.active_contexts();
        self.get_action_with_contexts(key, &contexts)
    }

    /// Validate keybinds for conflicts
    pub fn validate_keybinds(&self, keybinds: &[Keybind]) -> ValidationResult {
        // Detect conflicts
        let conflicts = ConflictDetector::detect(keybinds);
        let is_valid = conflicts.is_empty();

        // Suggest resolutions for each conflict
        let mut resolutions = Vec::new();
        for conflict in &conflicts {
            let suggestions = ConflictDetector::suggest_resolution(conflict, keybinds);
            resolutions.extend(suggestions);
        }

        ValidationResult {
            is_valid,
            conflicts,
            resolutions,
            applied_keybinds: keybinds.len(),
        }
    }

    /// Full validation pipeline: parse → validate → apply
    pub fn validate_and_apply_from_string(
        &mut self,
        content: &str,
        format: &str,
    ) -> Result<ValidationResult, EngineError> {
        // Step 1: Parse configuration
        let parser = ParserRegistry::new();
        let keybinds = parser.parse(content, format)?;

        // Step 2: Validate for conflicts
        let validation = self.validate_keybinds(&keybinds);

        // Step 3: Apply keybinds if valid
        if validation.is_valid {
            self.apply_keybinds(keybinds)?;
        }

        Ok(validation)
    }

    /// Full validation pipeline with persistence: parse → validate → apply → persist
    pub fn validate_apply_and_persist_from_string(
        &mut self,
        content: &str,
        format: &str,
        profile_name: &str,
        persistence: &dyn KeybindPersistence,
    ) -> Result<ValidationResult, EngineError> {
        // Step 1: Parse configuration
        let parser = ParserRegistry::new();
        let keybinds = parser.parse(content, format)?;

        // Step 2: Validate for conflicts
        let validation = self.validate_keybinds(&keybinds);

        // Step 3: Apply keybinds if valid
        if validation.is_valid {
            self.apply_keybinds(keybinds.clone())?;

            // Step 4: Create profile and persist
            self.create_profile(profile_name, keybinds)?;
            let profile = self.profile_manager.get_active_profile()?;
            persistence.save_profile(profile)?;
        }

        Ok(validation)
    }

    /// Get help for all keybinds
    pub fn get_help_all(&self) -> String {
        let keybinds = self.all_keybinds();
        KeybindHelp::display_all(&keybinds)
    }

    /// Get help for keybinds by category
    pub fn get_help_by_category(&self, category: &str) -> String {
        let keybinds = self.keybinds_by_category(category);
        KeybindHelp::display_by_category(&keybinds, category)
    }

    /// Search keybinds
    pub fn search_keybinds(&self, query: &str) -> Vec<&Keybind> {
        let keybinds = self.all_keybinds();
        KeybindHelp::search(&keybinds, query)
    }

    /// Get paginated keybinds
    pub fn get_keybinds_paginated(&self, page: usize, page_size: usize) -> crate::help::Page<&Keybind> {
        let keybinds = self.all_keybinds();
        KeybindHelp::paginate(&keybinds, page, page_size)
    }

    /// Get a profile by name
    pub fn get_profile(&self, name: &str) -> Option<&crate::profile::Profile> {
        self.profile_manager.get_profile(name)
    }
}

impl Default for KeybindEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Validation result containing conflicts and resolutions
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub conflicts: Vec<crate::conflict::Conflict>,
    pub resolutions: Vec<crate::conflict::Resolution>,
    pub applied_keybinds: usize,
}

impl ValidationResult {
    /// Check if validation passed (no conflicts)
    pub fn passed(&self) -> bool {
        self.is_valid && self.conflicts.is_empty()
    }

    /// Get conflict count
    pub fn conflict_count(&self) -> usize {
        self.conflicts.len()
    }
}

/// Helper function to initialize engine with defaults from a file
pub fn initialize_engine_with_defaults(
    defaults_path: impl AsRef<Path>,
) -> Result<KeybindEngine, EngineError> {
    let mut engine = KeybindEngine::new();
    engine.load_defaults_from_file(defaults_path)?;
    engine.apply_defaults()?;
    Ok(engine)
}

/// Helper function to get the default storage location for keybind profiles
/// 
/// Returns a FileSystemPersistence configured to use the default storage location:
/// `projects/ricecoder/config/keybinds/`
pub fn get_default_persistence() -> Result<crate::persistence::FileSystemPersistence, EngineError> {
    crate::persistence::FileSystemPersistence::with_default_location()
        .map_err(EngineError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

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
