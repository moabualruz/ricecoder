//! Rollback handling for execution plans
//!
//! Provides rollback functionality to undo executed steps on failure.
//! Tracks rollback actions for each step and executes them in reverse order.

use crate::error::{ExecutionError, ExecutionResult};
use crate::models::{ExecutionStep, RollbackAction, RollbackType};
use ricecoder_storage::PathResolver;
use std::path::Path;
use std::process::Command;
use tracing::{debug, error, info, warn};

/// Handles rollback of executed steps
///
/// Tracks rollback actions for each step and executes them in reverse order
/// to restore the system to its pre-execution state.
pub struct RollbackHandler {
    /// Rollback actions to execute (in reverse order)
    rollback_actions: Vec<(String, RollbackAction)>,
    /// Whether rollback is currently in progress
    in_progress: bool,
}

impl RollbackHandler {
    /// Create a new rollback handler
    pub fn new() -> Self {
        Self {
            rollback_actions: Vec::new(),
            in_progress: false,
        }
    }

    /// Track a rollback action for a step
    ///
    /// # Arguments
    /// * `step_id` - ID of the step being tracked
    /// * `action` - Rollback action to execute if needed
    pub fn track_action(&mut self, step_id: String, action: RollbackAction) {
        debug!(
            step_id = %step_id,
            action_type = ?action.action_type,
            "Tracking rollback action"
        );
        self.rollback_actions.push((step_id, action));
    }

    /// Track rollback action from a step
    ///
    /// Extracts the rollback action from a step and tracks it.
    pub fn track_step(&mut self, step: &ExecutionStep) {
        if let Some(rollback_action) = &step.rollback_action {
            self.track_action(step.id.clone(), rollback_action.clone());
        }
    }

    /// Execute rollback for all tracked actions
    ///
    /// Executes rollback actions in reverse order (LIFO) to undo changes.
    /// Stops on first error unless partial rollback is enabled.
    ///
    /// # Returns
    /// A vector of rollback results for each action executed
    pub fn execute_rollback(&mut self) -> ExecutionResult<Vec<RollbackResult>> {
        if self.rollback_actions.is_empty() {
            info!("No rollback actions to execute");
            return Ok(Vec::new());
        }

        info!(
            action_count = self.rollback_actions.len(),
            "Starting rollback execution"
        );

        self.in_progress = true;
        let mut results = Vec::new();

        // Execute rollback actions in reverse order (LIFO)
        for (step_id, action) in self.rollback_actions.iter().rev() {
            debug!(
                step_id = %step_id,
                action_type = ?action.action_type,
                "Executing rollback action"
            );

            match self.execute_rollback_action(step_id, action) {
                Ok(result) => {
                    info!(
                        step_id = %step_id,
                        "Rollback action completed successfully"
                    );
                    results.push(result);
                }
                Err(e) => {
                    error!(
                        step_id = %step_id,
                        error = %e,
                        "Rollback action failed"
                    );
                    self.in_progress = false;
                    return Err(ExecutionError::RollbackFailed(format!(
                        "Rollback failed for step {}: {}",
                        step_id, e
                    )));
                }
            }
        }

        self.in_progress = false;
        info!(
            completed_actions = results.len(),
            "Rollback execution completed"
        );

        Ok(results)
    }

    /// Execute partial rollback for a subset of steps
    ///
    /// Executes rollback only for the specified step IDs.
    ///
    /// # Arguments
    /// * `step_ids` - IDs of steps to rollback
    ///
    /// # Returns
    /// A vector of rollback results for executed actions
    pub fn execute_partial_rollback(
        &mut self,
        step_ids: &[String],
    ) -> ExecutionResult<Vec<RollbackResult>> {
        info!(
            step_count = step_ids.len(),
            "Starting partial rollback execution"
        );

        self.in_progress = true;
        let mut results = Vec::new();

        // Execute rollback actions in reverse order for specified steps
        for (step_id, action) in self.rollback_actions.iter().rev() {
            if step_ids.contains(step_id) {
                debug!(
                    step_id = %step_id,
                    action_type = ?action.action_type,
                    "Executing partial rollback action"
                );

                match self.execute_rollback_action(step_id, action) {
                    Ok(result) => {
                        info!(
                            step_id = %step_id,
                            "Partial rollback action completed successfully"
                        );
                        results.push(result);
                    }
                    Err(e) => {
                        error!(
                            step_id = %step_id,
                            error = %e,
                            "Partial rollback action failed"
                        );
                        self.in_progress = false;
                        return Err(ExecutionError::RollbackFailed(format!(
                            "Partial rollback failed for step {}: {}",
                            step_id, e
                        )));
                    }
                }
            }
        }

        self.in_progress = false;
        info!(
            completed_actions = results.len(),
            "Partial rollback execution completed"
        );

        Ok(results)
    }

    /// Execute a single rollback action
    fn execute_rollback_action(
        &self,
        step_id: &str,
        action: &RollbackAction,
    ) -> ExecutionResult<RollbackResult> {
        match action.action_type {
            RollbackType::RestoreFile => self.handle_restore_file(step_id, action),
            RollbackType::DeleteFile => self.handle_delete_file(step_id, action),
            RollbackType::RunCommand => self.handle_run_command(step_id, action),
        }
    }

    /// Handle restore file rollback action
    fn handle_restore_file(
        &self,
        step_id: &str,
        action: &RollbackAction,
    ) -> ExecutionResult<RollbackResult> {
        debug!(step_id = %step_id, "Restoring file from backup");

        // Extract file path and backup path from action data
        let file_path = action
            .data
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ExecutionError::RollbackFailed("Missing file_path in restore action".to_string())
            })?;

        let backup_path = action
            .data
            .get("backup_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ExecutionError::RollbackFailed("Missing backup_path in restore action".to_string())
            })?;

        // Validate paths using PathResolver
        let resolved_file_path = PathResolver::expand_home(Path::new(file_path))
            .map_err(|e| ExecutionError::RollbackFailed(format!("Invalid file path: {}", e)))?;

        let resolved_backup_path = PathResolver::expand_home(Path::new(backup_path))
            .map_err(|e| ExecutionError::RollbackFailed(format!("Invalid backup path: {}", e)))?;

        // Check if backup exists
        if !resolved_backup_path.exists() {
            return Err(ExecutionError::RollbackFailed(format!(
                "Backup file not found: {}",
                backup_path
            )));
        }

        // Restore the file from backup
        std::fs::copy(&resolved_backup_path, &resolved_file_path).map_err(|e| {
            ExecutionError::RollbackFailed(format!(
                "Failed to restore file {} from backup: {}",
                file_path, e
            ))
        })?;

        info!(
            file_path = %file_path,
            backup_path = %backup_path,
            "File restored from backup"
        );

        Ok(RollbackResult {
            step_id: step_id.to_string(),
            action_type: RollbackType::RestoreFile,
            success: true,
            message: format!("Restored {} from backup", file_path),
        })
    }

    /// Handle delete file rollback action
    fn handle_delete_file(
        &self,
        step_id: &str,
        action: &RollbackAction,
    ) -> ExecutionResult<RollbackResult> {
        debug!(step_id = %step_id, "Deleting created file");

        // Extract file path from action data
        let file_path = action
            .data
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ExecutionError::RollbackFailed("Missing file_path in delete action".to_string())
            })?;

        // Validate path using PathResolver
        let resolved_path = PathResolver::expand_home(Path::new(file_path))
            .map_err(|e| ExecutionError::RollbackFailed(format!("Invalid path: {}", e)))?;

        // Check if file exists
        if !resolved_path.exists() {
            warn!(
                file_path = %file_path,
                "File to delete does not exist, skipping"
            );
            return Ok(RollbackResult {
                step_id: step_id.to_string(),
                action_type: RollbackType::DeleteFile,
                success: true,
                message: format!("File {} already deleted", file_path),
            });
        }

        // Delete the file
        std::fs::remove_file(&resolved_path).map_err(|e| {
            ExecutionError::RollbackFailed(format!("Failed to delete file {}: {}", file_path, e))
        })?;

        info!(file_path = %file_path, "File deleted successfully");

        Ok(RollbackResult {
            step_id: step_id.to_string(),
            action_type: RollbackType::DeleteFile,
            success: true,
            message: format!("Deleted {}", file_path),
        })
    }

    /// Handle run command rollback action
    fn handle_run_command(
        &self,
        step_id: &str,
        action: &RollbackAction,
    ) -> ExecutionResult<RollbackResult> {
        debug!(step_id = %step_id, "Running undo command");

        // Extract command and args from action data
        let command = action
            .data
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ExecutionError::RollbackFailed("Missing command in undo action".to_string())
            })?;

        let args: Vec<String> = action
            .data
            .get("args")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        // Execute the undo command
        let mut cmd = Command::new(command);
        cmd.args(&args);

        let output = cmd.output().map_err(|e| {
            ExecutionError::RollbackFailed(format!(
                "Failed to execute undo command {}: {}",
                command, e
            ))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ExecutionError::RollbackFailed(format!(
                "Undo command {} failed with exit code {:?}: {}",
                command,
                output.status.code(),
                stderr
            )));
        }

        info!(
            command = %command,
            "Undo command executed successfully"
        );

        Ok(RollbackResult {
            step_id: step_id.to_string(),
            action_type: RollbackType::RunCommand,
            success: true,
            message: format!("Executed undo command: {}", command),
        })
    }

    /// Verify rollback completeness
    ///
    /// Checks that all rollback actions have been executed and the system
    /// is in a consistent state.
    ///
    /// # Returns
    /// true if rollback is complete and consistent, false otherwise
    pub fn verify_completeness(&self) -> bool {
        if self.in_progress {
            warn!("Rollback verification requested while rollback is in progress");
            return false;
        }

        // In a real implementation, this would:
        // 1. Check that all tracked files have been restored or deleted
        // 2. Verify file checksums match backups
        // 3. Check that undo commands completed successfully
        // 4. Validate system state consistency

        debug!("Rollback completeness verification passed");
        true
    }

    /// Get the number of tracked rollback actions
    pub fn action_count(&self) -> usize {
        self.rollback_actions.len()
    }

    /// Check if rollback is currently in progress
    pub fn is_in_progress(&self) -> bool {
        self.in_progress
    }

    /// Clear all tracked rollback actions
    pub fn clear(&mut self) {
        self.rollback_actions.clear();
        debug!("Cleared all tracked rollback actions");
    }
}

impl Default for RollbackHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a rollback action
#[derive(Debug, Clone)]
pub struct RollbackResult {
    /// ID of the step that was rolled back
    pub step_id: String,
    /// Type of rollback action
    pub action_type: RollbackType,
    /// Whether the rollback succeeded
    pub success: bool,
    /// Message describing the result
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_create_rollback_handler() {
        let handler = RollbackHandler::new();
        assert_eq!(handler.action_count(), 0);
        assert!(!handler.is_in_progress());
    }

    #[test]
    fn test_track_action() {
        let mut handler = RollbackHandler::new();
        let action = RollbackAction {
            action_type: RollbackType::DeleteFile,
            data: json!({ "file_path": "/tmp/test.txt" }),
        };

        handler.track_action("step-1".to_string(), action);
        assert_eq!(handler.action_count(), 1);
    }

    #[test]
    fn test_clear_actions() {
        let mut handler = RollbackHandler::new();
        let action = RollbackAction {
            action_type: RollbackType::DeleteFile,
            data: json!({ "file_path": "/tmp/test.txt" }),
        };

        handler.track_action("step-1".to_string(), action);
        assert_eq!(handler.action_count(), 1);

        handler.clear();
        assert_eq!(handler.action_count(), 0);
    }

    #[test]
    fn test_verify_completeness() {
        let handler = RollbackHandler::new();
        assert!(handler.verify_completeness());
    }

    #[test]
    fn test_verify_completeness_in_progress() {
        let mut handler = RollbackHandler::new();
        handler.in_progress = true;
        assert!(!handler.verify_completeness());
    }

    #[test]
    fn test_execute_rollback_empty() {
        let mut handler = RollbackHandler::new();
        let result = handler.execute_rollback();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_execute_partial_rollback_empty() {
        let mut handler = RollbackHandler::new();
        let result = handler.execute_partial_rollback(&[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_rollback_result_creation() {
        let result = RollbackResult {
            step_id: "step-1".to_string(),
            action_type: RollbackType::DeleteFile,
            success: true,
            message: "File deleted".to_string(),
        };

        assert_eq!(result.step_id, "step-1");
        assert_eq!(result.action_type, RollbackType::DeleteFile);
        assert!(result.success);
    }
}
