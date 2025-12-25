//! Keybind engine that combines registry and profile management

use std::{path::Path, time::{Duration, Instant}};

use crate::{
    conflict::ConflictDetector,
    error::EngineError,
    help::KeybindHelp,
    merge::KeybindMerger,
    models::{Context, Key, KeyCombo, Keybind, Modifier},
    parser::ParserRegistry,
    persistence::KeybindPersistence,
    profile::ProfileManager,
    registry::KeybindRegistry,
};

/// Leader key state for OpenCode compatibility (G02)
#[derive(Debug, Clone)]
pub struct LeaderState {
    /// Whether leader key is currently active
    pub active: bool,
    /// Time when leader was pressed
    pub pressed_at: Option<Instant>,
    /// Leader timeout (default 2 seconds)
    pub timeout: Duration,
}

impl Default for LeaderState {
    fn default() -> Self {
        Self {
            active: false,
            pressed_at: None,
            timeout: Duration::from_secs(2),
        }
    }
}

impl LeaderState {
    /// Activate leader state
    pub fn activate(&mut self) {
        self.active = true;
        self.pressed_at = Some(Instant::now());
    }
    
    /// Deactivate leader state
    pub fn deactivate(&mut self) {
        self.active = false;
        self.pressed_at = None;
    }
    
    /// Check if leader state has timed out
    pub fn is_timed_out(&self) -> bool {
        if let Some(pressed_at) = self.pressed_at {
            pressed_at.elapsed() > self.timeout
        } else {
            false
        }
    }
    
    /// Check if leader is active and not timed out
    pub fn is_valid(&self) -> bool {
        self.active && !self.is_timed_out()
    }
}

/// Pending key state for multi-key chords (Helix G08)
#[derive(Debug, Clone)]
pub struct PendingKeyState {
    /// Keys pressed so far in the sequence
    pub keys: Vec<KeyCombo>,
    /// Time when first key was pressed
    pub started_at: Option<Instant>,
    /// Timeout for chord sequences (default 1 second)
    pub timeout: Duration,
}

impl Default for PendingKeyState {
    fn default() -> Self {
        Self {
            keys: Vec::new(),
            started_at: None,
            timeout: Duration::from_secs(1),
        }
    }
}

impl PendingKeyState {
    /// Add a key to the pending sequence
    pub fn push(&mut self, key: KeyCombo) {
        if self.keys.is_empty() {
            self.started_at = Some(Instant::now());
        }
        self.keys.push(key);
    }
    
    /// Clear pending state
    pub fn clear(&mut self) {
        self.keys.clear();
        self.started_at = None;
    }
    
    /// Check if pending state has timed out
    pub fn is_timed_out(&self) -> bool {
        if let Some(started_at) = self.started_at {
            started_at.elapsed() > self.timeout
        } else {
            false
        }
    }
    
    /// Check if sequence is active
    pub fn is_active(&self) -> bool {
        !self.keys.is_empty() && !self.is_timed_out()
    }
    
    /// Get the pending key sequence as a string (e.g., "gg")
    pub fn as_string(&self) -> String {
        self.keys.iter()
            .map(|k| k.to_string())
            .collect::<Vec<_>>()
            .join("")
    }
}

/// Main keybind engine combining registry and profile management
pub struct KeybindEngine {
    registry: KeybindRegistry,
    profile_manager: ProfileManager,
    default_keybinds: Vec<Keybind>,
    current_context: Context,
    context_stack: Vec<Context>,
    /// Leader key state (OpenCode G02)
    leader_state: LeaderState,
    /// Pending key state for multi-key chords (Helix G08)
    pending_keys: PendingKeyState,
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
            leader_state: LeaderState::default(),
            pending_keys: PendingKeyState::default(),
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
    
    // ========================================================================
    // Leader Key Support (OpenCode G02)
    // ========================================================================
    
    /// Set leader timeout
    pub fn set_leader_timeout(&mut self, timeout: Duration) {
        self.leader_state.timeout = timeout;
    }
    
    /// Get leader state
    pub fn leader_state(&self) -> &LeaderState {
        &self.leader_state
    }
    
    /// Check if leader key is currently active
    pub fn is_leader_active(&self) -> bool {
        self.leader_state.is_valid()
    }
    
    /// Activate leader key
    pub fn activate_leader(&mut self) {
        self.leader_state.activate();
    }
    
    /// Deactivate leader key
    pub fn deactivate_leader(&mut self) {
        self.leader_state.deactivate();
    }
    
    /// Process leader timeout (call periodically)
    pub fn process_leader_timeout(&mut self) {
        if self.leader_state.is_timed_out() {
            self.deactivate_leader();
        }
    }
    
    // ========================================================================
    // Multi-Key Chord Support (Helix G08)
    // ========================================================================
    
    /// Set chord timeout
    pub fn set_chord_timeout(&mut self, timeout: Duration) {
        self.pending_keys.timeout = timeout;
    }
    
    /// Get pending key state
    pub fn pending_keys(&self) -> &PendingKeyState {
        &self.pending_keys
    }
    
    /// Check if a chord sequence is pending
    pub fn is_chord_pending(&self) -> bool {
        self.pending_keys.is_active()
    }
    
    /// Add a key to pending chord sequence
    pub fn push_pending_key(&mut self, key: KeyCombo) {
        self.pending_keys.push(key);
    }
    
    /// Clear pending chord sequence
    pub fn clear_pending_keys(&mut self) {
        self.pending_keys.clear();
    }
    
    /// Process chord timeout (call periodically)
    pub fn process_chord_timeout(&mut self) {
        if self.pending_keys.is_timed_out() {
            self.clear_pending_keys();
        }
    }
    
    /// Handle Escape key to cancel pending state (Helix G15)
    pub fn handle_escape(&mut self) {
        self.clear_pending_keys();
        self.deactivate_leader();
    }
    
    /// Try to match pending chord sequence to an action
    pub fn match_pending_chord(&self) -> Option<&str> {
        if !self.is_chord_pending() {
            return None;
        }
        
        // Build chord string (e.g., "gg", "dd")
        let chord_str = self.pending_keys.as_string();
        
        // Try to find a keybind that matches this chord sequence
        // For now, we treat it as a single key combo
        // TODO: Implement proper trie-based chord matching for Helix G08
        let chord_combo = KeyCombo {
            modifiers: Vec::new(),
            key: Key::Char(chord_str.chars().next().unwrap_or(' ')),
            leader: false,
        };
        
        self.get_action_with_active_contexts(&chord_combo)
    }
    
    /// Process a key event with leader and chord support
    /// Returns (action_id, consumed_key) tuple
    /// - action_id: matched action if found (owned String to avoid borrow issues)
    /// - consumed_key: whether the key was consumed by leader/chord logic
    pub fn process_key_with_state(&mut self, key: KeyCombo) -> (Option<String>, bool) {
        // Process timeouts first
        self.process_leader_timeout();
        self.process_chord_timeout();
        
        // Handle Escape (G15)
        if matches!(key.key, Key::Escape) {
            self.handle_escape();
            return (Some("cancel_pending".to_string()), true);
        }
        
        // Handle leader key activation
        if key.leader {
            if self.is_leader_active() {
                // Leader already active, deactivate
                self.deactivate_leader();
                return (None, true);
            } else {
                // Activate leader
                self.activate_leader();
                return (None, true);
            }
        }
        
        // Try direct match first (clone to avoid borrow issues)
        let action_opt = self.get_action_with_active_contexts(&key).map(|s| s.to_string());
        if let Some(action) = action_opt {
            // Clear pending state on match
            self.clear_pending_keys();
            return (Some(action), true);
        }
        
        // If no match, check if this could be part of a chord
        self.push_pending_key(key.clone());
        
        // Try to match pending chord (clone to avoid borrow issues)
        let chord_action_opt = self.match_pending_chord().map(|s| s.to_string());
        if let Some(action) = chord_action_opt {
            self.clear_pending_keys();
            return (Some(action), true);
        }
        
        // Key is pending (not consumed)
        (None, false)
    }
    
    // ========================================================================
    // Runtime Config Loading (G01)
    // ========================================================================
    
    /// Load keybinds from runtime config file
    pub fn load_from_config_file(&mut self, path: impl AsRef<Path>) -> Result<(), EngineError> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| EngineError::DefaultsLoadError(format!("Failed to read config: {}", e)))?;
        
        // Determine format from extension
        let ext = path.as_ref()
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("json");
        
        let parser = ParserRegistry::new();
        let keybinds = parser.parse(&content, ext)
            .map_err(|e| EngineError::DefaultsLoadError(format!("Failed to parse config: {}", e)))?;
        
        // Apply keybinds with merge if defaults exist
        if !self.default_keybinds.is_empty() {
            self.apply_profile_with_merge(&keybinds)?;
        } else {
            self.apply_keybinds(keybinds)?;
        }
        
        Ok(())
    }
    
    /// Load keybinds from config directory (G01)
    /// Looks for:
    /// - config/keybinds/default.json
    /// - config/keybinds/custom.json (merged with defaults)
    pub fn load_from_config_dir(&mut self, config_dir: impl AsRef<Path>) -> Result<(), EngineError> {
        let config_dir = config_dir.as_ref();
        let keybinds_dir = config_dir.join("keybinds");
        
        // Load defaults first
        let default_path = keybinds_dir.join("default.json");
        if default_path.exists() {
            self.load_defaults_from_file(&default_path)?;
            self.apply_defaults()?;
        }
        
        // Load and merge custom keybinds
        let custom_path = keybinds_dir.join("custom.json");
        if custom_path.exists() {
            self.load_from_config_file(&custom_path)?;
        }
        
        Ok(())
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
