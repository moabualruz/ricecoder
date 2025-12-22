//! Step creation utilities for building execution steps

use std::path::Path;

use ricecoder_storage::PathResolver;
use serde_json::json;

use crate::{
    error::{ExecutionError, ExecutionResult},
    models::{ExecutionStep, RollbackAction, RollbackType, StepAction},
};

/// Helper for creating execution steps with validation
pub struct StepCreator;

impl StepCreator {
    /// Create a file creation step
    ///
    /// # Arguments
    /// * `path` - File path (validated with PathResolver)
    /// * `content` - File content
    ///
    /// # Returns
    /// An ExecutionStep configured to create the file
    ///
    /// # Errors
    /// Returns error if path is invalid
    pub fn create_file(path: String, content: String) -> ExecutionResult<ExecutionStep> {
        // Validate path using PathResolver
        let _resolved = PathResolver::expand_home(Path::new(&path))
            .map_err(|e| ExecutionError::ValidationError(format!("Invalid path: {}", e)))?;

        let mut step = ExecutionStep::new(
            format!("Create file: {}", path),
            StepAction::CreateFile {
                path: path.clone(),
                content,
            },
        );

        // Set rollback action to delete the created file
        step.rollback_action = Some(RollbackAction {
            action_type: RollbackType::DeleteFile,
            data: json!({ "path": path }),
        });

        Ok(step)
    }

    /// Create a file modification step
    ///
    /// # Arguments
    /// * `path` - File path (validated with PathResolver)
    /// * `diff` - Diff to apply
    ///
    /// # Returns
    /// An ExecutionStep configured to modify the file
    ///
    /// # Errors
    /// Returns error if path is invalid
    pub fn modify_file(path: String, diff: String) -> ExecutionResult<ExecutionStep> {
        // Validate path using PathResolver
        let _resolved = PathResolver::expand_home(Path::new(&path))
            .map_err(|e| ExecutionError::ValidationError(format!("Invalid path: {}", e)))?;

        let mut step = ExecutionStep::new(
            format!("Modify file: {}", path),
            StepAction::ModifyFile {
                path: path.clone(),
                diff,
            },
        );

        // Set rollback action to restore the file from backup
        step.rollback_action = Some(RollbackAction {
            action_type: RollbackType::RestoreFile,
            data: json!({ "path": path }),
        });

        Ok(step)
    }

    /// Create a file deletion step
    ///
    /// # Arguments
    /// * `path` - File path (validated with PathResolver)
    ///
    /// # Returns
    /// An ExecutionStep configured to delete the file
    ///
    /// # Errors
    /// Returns error if path is invalid
    pub fn delete_file(path: String) -> ExecutionResult<ExecutionStep> {
        // Validate path using PathResolver
        let _resolved = PathResolver::expand_home(Path::new(&path))
            .map_err(|e| ExecutionError::ValidationError(format!("Invalid path: {}", e)))?;

        let mut step = ExecutionStep::new(
            format!("Delete file: {}", path),
            StepAction::DeleteFile { path: path.clone() },
        );

        // Set rollback action to restore the file from backup
        step.rollback_action = Some(RollbackAction {
            action_type: RollbackType::RestoreFile,
            data: json!({ "path": path }),
        });

        Ok(step)
    }

    /// Create a command execution step
    ///
    /// # Arguments
    /// * `command` - Command to execute
    /// * `args` - Command arguments
    ///
    /// # Returns
    /// An ExecutionStep configured to run the command
    pub fn run_command(command: String, args: Vec<String>) -> ExecutionStep {
        let mut step = ExecutionStep::new(
            format!("Run command: {} {}", command, args.join(" ")),
            StepAction::RunCommand {
                command: command.clone(),
                args: args.clone(),
            },
        );

        // Set rollback action to run reverse command
        step.rollback_action = Some(RollbackAction {
            action_type: RollbackType::RunCommand,
            data: json!({
                "command": command,
                "args": args,
                "reverse": true
            }),
        });

        step
    }

    /// Create a test execution step
    ///
    /// # Arguments
    /// * `pattern` - Optional test pattern to filter tests
    ///
    /// # Returns
    /// An ExecutionStep configured to run tests
    pub fn run_tests(pattern: Option<String>) -> ExecutionStep {
        let description = if let Some(ref p) = pattern {
            format!("Run tests matching: {}", p)
        } else {
            "Run all tests".to_string()
        };

        ExecutionStep::new(description, StepAction::RunTests { pattern })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_file_step() {
        let result = StepCreator::create_file("test.txt".to_string(), "content".to_string());
        assert!(result.is_ok());
        let step = result.unwrap();
        assert!(matches!(step.action, StepAction::CreateFile { .. }));
        assert!(step.rollback_action.is_some());
    }

    #[test]
    fn test_modify_file_step() {
        let result = StepCreator::modify_file("test.txt".to_string(), "diff".to_string());
        assert!(result.is_ok());
        let step = result.unwrap();
        assert!(matches!(step.action, StepAction::ModifyFile { .. }));
        assert!(step.rollback_action.is_some());
    }

    #[test]
    fn test_delete_file_step() {
        let result = StepCreator::delete_file("test.txt".to_string());
        assert!(result.is_ok());
        let step = result.unwrap();
        assert!(matches!(step.action, StepAction::DeleteFile { .. }));
        assert!(step.rollback_action.is_some());
    }

    #[test]
    fn test_run_command_step() {
        let step = StepCreator::run_command("echo".to_string(), vec!["hello".to_string()]);
        assert!(matches!(step.action, StepAction::RunCommand { .. }));
        assert!(step.rollback_action.is_some());
    }

    #[test]
    fn test_run_tests_step() {
        let step = StepCreator::run_tests(Some("*.rs".to_string()));
        assert!(matches!(step.action, StepAction::RunTests { .. }));
    }

    #[test]
    fn test_rollback_action_for_create() {
        let step = StepCreator::create_file("test.txt".to_string(), "content".to_string()).unwrap();
        let rollback = step.rollback_action.unwrap();
        assert_eq!(rollback.action_type, RollbackType::DeleteFile);
    }

    #[test]
    fn test_rollback_action_for_modify() {
        let step = StepCreator::modify_file("test.txt".to_string(), "diff".to_string()).unwrap();
        let rollback = step.rollback_action.unwrap();
        assert_eq!(rollback.action_type, RollbackType::RestoreFile);
    }

    #[test]
    fn test_rollback_action_for_delete() {
        let step = StepCreator::delete_file("test.txt".to_string()).unwrap();
        let rollback = step.rollback_action.unwrap();
        assert_eq!(rollback.action_type, RollbackType::RestoreFile);
    }

    #[test]
    fn test_rollback_action_for_command() {
        let step = StepCreator::run_command("echo".to_string(), vec!["hello".to_string()]);
        let rollback = step.rollback_action.unwrap();
        assert_eq!(rollback.action_type, RollbackType::RunCommand);
    }
}
