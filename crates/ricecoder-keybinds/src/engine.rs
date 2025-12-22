//! Keybind engine that combines registry and profile management

use std::path::Path;

use crate::{
    conflict::ConflictDetector,
    error::EngineError,
    help::KeybindHelp,
    merge::KeybindMerger,
    models::{Context, KeyCombo, Keybind},
    parser::ParserRegistry,
    persistence::KeybindPersistence,
    profile::ProfileManager,
    registry::KeybindRegistry,
};

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
    pub fn apply_profile_with_merge(
        &mut self,
        profile_keybinds: &[Keybind],
    ) -> Result<(), EngineError> {
        // Merge profile keybinds with defaults
        let merge_result =
            KeybindMerger::merge_with_contexts(&self.default_keybinds, profile_keybinds).map_err(
                |e| EngineError::DefaultsLoadError(format!("Failed to merge keybinds: {}", e)),
            )?;

        // Log resolved conflicts
        for conflict in &merge_result.resolved_conflicts {
            tracing::info!(
                "Resolved keybind conflict: {} in context {} - kept user binding",
                conflict.key,
                conflict.context
            );
        }

        // Log unresolved conflicts
        for conflict in &merge_result.unresolved_conflicts {
            tracing::warn!(
                "Unresolved keybind conflict: {} in context {} - {}",
                conflict.key,
                conflict.context,
                conflict.user.action_id
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
    pub fn get_keybinds_paginated(
        &self,
        page: usize,
        page_size: usize,
    ) -> crate::help::Page<&Keybind> {
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
    crate::persistence::FileSystemPersistence::with_default_location().map_err(EngineError::from)
}
