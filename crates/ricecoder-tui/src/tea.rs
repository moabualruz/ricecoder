//! Elm Architecture (TEA) implementation for RiceCoder TUI
//!
//! This module implements the Model-Update-View pattern for predictable,
//! immutable state management with structural sharing and reactive updates.
//!
//! The core TEA components are now split into separate modules:
//! - `model.rs`: Contains AppModel, AppMessage, and related state types
//! - `update.rs`: Contains the pure update function and Command enum
//! - `tea.rs`: Contains ReactiveState manager and TEA orchestration

use ricecoder_storage::TuiConfig;

use crate::{model::*, style::Theme, terminal_state::TerminalCapabilities, update::Command};

// StateDiff and StateChange are now in model.rs

/// Reactive state manager with change tracking, message batching, and debugging
#[derive(Debug)]
pub struct ReactiveState {
    current: AppModel,
    history: Vec<AppModel>,
    max_history: usize,
    message_processor: MessageBatchProcessor,
    debugger: StateDebugger,
}

/// State debugger for inspection and time-travel debugging
#[derive(Debug)]
pub struct StateDebugger {
    enabled: bool,
    state_snapshots: Vec<StateSnapshot>,
    max_snapshots: usize,
    change_log: Vec<StateChangeLog>,
    breakpoints: std::collections::HashSet<String>,
}

#[derive(Debug, Clone)]
pub struct StateSnapshot {
    pub model: AppModel,
    pub timestamp: std::time::Instant,
    pub message: Option<AppMessage>,
    pub diff: Option<StateDiff>,
}

#[derive(Debug, Clone)]
pub struct StateChangeLog {
    pub timestamp: std::time::Instant,
    pub message: AppMessage,
    pub previous_state: serde_json::Value,
    pub new_state: serde_json::Value,
    pub diff: StateDiff,
}

impl StateDebugger {
    /// Create a new state debugger
    pub fn new() -> Self {
        Self {
            enabled: false,
            state_snapshots: Vec::new(),
            max_snapshots: 100,
            change_log: Vec::new(),
            breakpoints: std::collections::HashSet::new(),
        }
    }

    /// Enable debugging
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable debugging
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Check if debugging is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Take a state snapshot
    pub fn snapshot(
        &mut self,
        model: &AppModel,
        message: Option<&AppMessage>,
        diff: Option<&StateDiff>,
    ) {
        if !self.enabled {
            return;
        }

        let snapshot = StateSnapshot {
            model: model.clone(),
            timestamp: std::time::Instant::now(),
            message: message.cloned(),
            diff: diff.cloned(),
        };

        self.state_snapshots.push(snapshot);
        if self.state_snapshots.len() > self.max_snapshots {
            self.state_snapshots.remove(0);
        }
    }

    /// Log a state change
    pub fn log_change(
        &mut self,
        message: &AppMessage,
        previous_state: &AppModel,
        new_state: &AppModel,
        diff: &StateDiff,
    ) {
        if !self.enabled {
            return;
        }

        // Convert states to JSON for logging (simplified - in practice you'd implement proper serialization)
        let previous_json = serde_json::json!({
            "mode": format!("{:?}", previous_state.mode),
            "session_count": previous_state.sessions.session_count,
            "command_count": previous_state.commands.command_history.len(),
        });

        let new_json = serde_json::json!({
            "mode": format!("{:?}", new_state.mode),
            "session_count": new_state.sessions.session_count,
            "command_count": new_state.commands.command_history.len(),
        });

        let log_entry = StateChangeLog {
            timestamp: std::time::Instant::now(),
            message: message.clone(),
            previous_state: previous_json,
            new_state: new_json,
            diff: diff.clone(),
        };

        self.change_log.push(log_entry);

        // Check breakpoints
        if self.breakpoints.contains(&format!("{:?}", message)) {
            tracing::warn!("State debugger breakpoint hit for message: {:?}", message);
            // In a real implementation, this could pause execution or trigger other debugging actions
        }
    }

    /// Add a breakpoint for a specific message type
    pub fn add_breakpoint(&mut self, message_pattern: &str) {
        self.breakpoints.insert(message_pattern.to_string());
    }

    /// Remove a breakpoint
    pub fn remove_breakpoint(&mut self, message_pattern: &str) {
        self.breakpoints.remove(message_pattern);
    }

    /// Get all snapshots
    pub fn snapshots(&self) -> &[StateSnapshot] {
        &self.state_snapshots
    }

    /// Get snapshot at specific index
    pub fn snapshot_at(&self, index: usize) -> Option<&StateSnapshot> {
        self.state_snapshots.get(index)
    }

    /// Get change log
    pub fn change_log(&self) -> &[StateChangeLog] {
        &self.change_log
    }

    /// Get current snapshot count
    pub fn snapshot_count(&self) -> usize {
        self.state_snapshots.len()
    }

    /// Clear all debugging data
    pub fn clear(&mut self) {
        self.state_snapshots.clear();
        self.change_log.clear();
    }

    /// Get debugging statistics
    pub fn stats(&self) -> StateDebugStats {
        StateDebugStats {
            enabled: self.enabled,
            snapshot_count: self.state_snapshots.len(),
            max_snapshots: self.max_snapshots,
            change_log_count: self.change_log.len(),
            breakpoint_count: self.breakpoints.len(),
        }
    }
}

/// Statistics for state debugging
#[derive(Debug, Clone)]
pub struct StateDebugStats {
    pub enabled: bool,
    pub snapshot_count: usize,
    pub max_snapshots: usize,
    pub change_log_count: usize,
    pub breakpoint_count: usize,
}

impl ReactiveState {
    /// Create a new reactive state manager
    pub fn new(initial_model: AppModel) -> Self {
        Self {
            current: initial_model,
            history: Vec::new(),
            max_history: 50,
            message_processor: MessageBatchProcessor::new(),
            debugger: StateDebugger::new(),
        }
    }

    /// Apply a message and return the state diff
    pub fn update(&mut self, message: AppMessage) -> Result<StateDiff, String> {
        if !self.current.can_transition(&message) {
            return Err(format!("Invalid state transition: {:?}", message));
        }

        let previous = self.current.clone();
        let (new_state, _commands) = self.current.clone().update(message.clone());

        // Validate the new state
        new_state.validate()?;

        // Update current state
        self.current = new_state.clone();

        // Calculate diff
        let diff = new_state.diff(&self.current);

        // Log change in debugger
        self.debugger
            .log_change(&message, &previous, &new_state, &diff);

        // Take snapshot
        self.debugger
            .snapshot(&new_state, Some(&message), Some(&diff));

        // Store previous state in history
        self.history.push(previous);
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }

        // Update current state
        self.current = new_state;

        Ok(diff)
    }

    /// Add a message to the batch processor
    pub fn batch_message(&mut self, message: AppMessage, priority: MessagePriority) {
        self.message_processor.add_message(message, priority);
    }

    /// Process all pending message batches
    pub fn process_batches(&mut self) -> Result<Vec<StateDiff>, String> {
        let mut diffs = Vec::new();

        let results = self.message_processor.process_batches(|message, model| {
            // Apply the message to create a new model
            let (new_model, _) = model.clone().update(message.clone());
            new_model
        });

        for (message, new_model) in results {
            // Calculate diff
            let diff = self.current.diff(&new_model);

            // Update state
            let previous = self.current.clone();
            self.history.push(previous);
            if self.history.len() > self.max_history {
                self.history.remove(0);
            }
            self.current = new_model;

            diffs.push(diff);
        }

        Ok(diffs)
    }

    /// Check if there are pending message batches
    pub fn has_pending_batches(&self) -> bool {
        self.message_processor.has_pending_batches()
    }

    /// Get message batch statistics
    pub fn batch_stats(&self) -> MessageBatchStats {
        self.message_processor.stats()
    }

    /// Force process all pending batches immediately
    pub fn flush_batches(&mut self) -> Result<Vec<StateDiff>, String> {
        // Temporarily reduce batch timeout to force processing
        let original_timeout = self.message_processor.batch_timeout();
        self.message_processor
            .set_batch_timeout(std::time::Duration::from_nanos(1));

        let result = self.process_batches();

        // Restore original timeout
        self.message_processor.set_batch_timeout(original_timeout);

        result
    }

    /// Enable state debugging
    pub fn enable_debugging(&mut self) {
        self.debugger.enable();
    }

    /// Disable state debugging
    pub fn disable_debugging(&mut self) {
        self.debugger.disable();
    }

    /// Check if debugging is enabled
    pub fn is_debugging_enabled(&self) -> bool {
        self.debugger.is_enabled()
    }

    /// Add a debugging breakpoint
    pub fn add_debug_breakpoint(&mut self, message_pattern: &str) {
        self.debugger.add_breakpoint(message_pattern);
    }

    /// Remove a debugging breakpoint
    pub fn remove_debug_breakpoint(&mut self, message_pattern: &str) {
        self.debugger.remove_breakpoint(message_pattern);
    }

    /// Get state snapshots for time-travel debugging
    pub fn debug_snapshots(&self) -> &[StateSnapshot] {
        self.debugger.snapshots()
    }

    /// Get snapshot at specific index
    pub fn debug_snapshot_at(&self, index: usize) -> Option<&StateSnapshot> {
        self.debugger.snapshot_at(index)
    }

    /// Get state change log
    pub fn debug_change_log(&self) -> &[StateChangeLog] {
        self.debugger.change_log()
    }

    /// Get debugging statistics
    pub fn debug_stats(&self) -> StateDebugStats {
        self.debugger.stats()
    }

    /// Clear debugging data
    pub fn clear_debug_data(&mut self) {
        self.debugger.clear();
    }

    /// Time-travel to a specific snapshot (for debugging)
    pub fn time_travel_to_snapshot(&mut self, index: usize) -> Result<(), String> {
        if let Some(snapshot) = self.debugger.snapshot_at(index) {
            self.current = snapshot.model.clone();
            Ok(())
        } else {
            Err(format!("Snapshot at index {} not found", index))
        }
    }

    /// Undo last change
    pub fn undo(&mut self) -> Result<StateDiff, String> {
        if let Some(previous) = self.history.pop() {
            let diff = self.current.diff(&previous);
            self.current = previous;
            Ok(diff)
        } else {
            Err("No more states to undo".to_string())
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.history.is_empty()
    }

    /// Get the current application model
    pub fn current(&self) -> &AppModel {
        &self.current
    }

    /// Get state at specific history index
    pub fn state_at(&self, index: usize) -> Option<&AppModel> {
        if index == 0 {
            Some(&self.current)
        } else {
            self.history.get(self.history.len().saturating_sub(index))
        }
    }
}
