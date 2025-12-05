//! Rollback action handlers for different action types
//!
//! Provides specialized handlers for:
//! - RestoreFile: Restore files from backup
//! - DeleteFile: Delete created files
//! - RunCommand: Execute undo commands

use crate::error::{ExecutionError, ExecutionResult};
use crate::models::RollbackAction;
use ricecoder_storage::PathResolver;
use std::path::Path;
use std::process::Command;
use tracing::{debug, info, warn};

/// Handles file restoration from backup
pub struct RestoreFileHandler;

impl RestoreFileHandler {
    /// Restore a file from backup
    ///
    /// # Arguments
    /// * `action` - Rollback action containing file and backup paths
    ///
    /// # Returns
    /// Success message if restoration succeeds
    pub fn handle(action: &RollbackAction) -> ExecutionResult<String> {
        debug!("Restoring file from backup");

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

        // Create parent directories if needed
        if let Some(parent) = resolved_file_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                ExecutionError::RollbackFailed(format!(
                    "Failed to create parent directories for {}: {}",
                    file_path, e
                ))
            })?;
        }

        // Restore the file from backup
        std::fs::copy(&resolved_backup_path, &resolved_file_path).map_err(|e| {
            ExecutionError::RollbackFailed(format!(
                "Failed to restore file {} from backup: {}",
                file_path, e
            ))
        })?;

        let message = format!("Restored {} from backup", file_path);
        info!("{}", message);
        Ok(message)
    }
}

/// Handles deletion of created files
pub struct DeleteFileHandler;

impl DeleteFileHandler {
    /// Delete a created file
    ///
    /// # Arguments
    /// * `action` - Rollback action containing file path
    ///
    /// # Returns
    /// Success message if deletion succeeds
    pub fn handle(action: &RollbackAction) -> ExecutionResult<String> {
        debug!("Deleting created file");

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
            return Ok(format!("File {} already deleted", file_path));
        }

        // Delete the file
        std::fs::remove_file(&resolved_path).map_err(|e| {
            ExecutionError::RollbackFailed(format!("Failed to delete file {}: {}", file_path, e))
        })?;

        let message = format!("Deleted {}", file_path);
        info!("{}", message);
        Ok(message)
    }
}

/// Handles execution of undo commands
pub struct UndoCommandHandler;

impl UndoCommandHandler {
    /// Execute an undo command
    ///
    /// # Arguments
    /// * `action` - Rollback action containing command and arguments
    ///
    /// # Returns
    /// Success message if command succeeds
    pub fn handle(action: &RollbackAction) -> ExecutionResult<String> {
        debug!("Running undo command");

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

        let message = format!("Executed undo command: {}", command);
        info!("{}", message);
        Ok(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_restore_file_handler_missing_file_path() {
        let action = RollbackAction {
            action_type: crate::models::RollbackType::RestoreFile,
            data: json!({ "backup_path": "/tmp/backup.txt" }),
        };

        let result = RestoreFileHandler::handle(&action);
        assert!(result.is_err());
    }

    #[test]
    fn test_restore_file_handler_missing_backup_path() {
        let action = RollbackAction {
            action_type: crate::models::RollbackType::RestoreFile,
            data: json!({ "file_path": "/tmp/test.txt" }),
        };

        let result = RestoreFileHandler::handle(&action);
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_file_handler_missing_file_path() {
        let action = RollbackAction {
            action_type: crate::models::RollbackType::DeleteFile,
            data: json!({}),
        };

        let result = DeleteFileHandler::handle(&action);
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_file_handler_nonexistent_file() {
        let action = RollbackAction {
            action_type: crate::models::RollbackType::DeleteFile,
            data: json!({ "file_path": "/tmp/nonexistent_file_12345.txt" }),
        };

        let result = DeleteFileHandler::handle(&action);
        assert!(result.is_ok());
        let message = result.unwrap();
        assert!(message.contains("already deleted"));
    }

    #[test]
    fn test_undo_command_handler_missing_command() {
        let action = RollbackAction {
            action_type: crate::models::RollbackType::RunCommand,
            data: json!({ "args": [] }),
        };

        let result = UndoCommandHandler::handle(&action);
        assert!(result.is_err());
    }

    #[test]
    fn test_undo_command_handler_with_args() {
        let action = RollbackAction {
            action_type: crate::models::RollbackType::RunCommand,
            data: json!({
                "command": "echo",
                "args": ["test"]
            }),
        };

        let result = UndoCommandHandler::handle(&action);
        assert!(result.is_ok());
    }
}
