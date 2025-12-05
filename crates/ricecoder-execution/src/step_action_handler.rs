//! Step action handlers for different action types
//!
//! Implements handlers for each step action type:
//! - CreateFile: Create new files with content
//! - ModifyFile: Apply diffs to existing files
//! - DeleteFile: Delete files using PathResolver
//! - RunCommand: Execute shell commands
//! - RunTests: Run tests with framework detection
//!
//! **CRITICAL**: All file operations use FileOperations wrapper which ensures
//! all paths are validated through ricecoder_storage::PathResolver.

use crate::error::{ExecutionError, ExecutionResult};
use crate::file_operations::FileOperations;
use std::process::Command;
use tracing::{debug, error, info};

/// Handles file creation actions
pub struct CreateFileHandler;

impl CreateFileHandler {
    /// Create a file with the specified content
    ///
    /// # Arguments
    /// * `path` - File path (validated with PathResolver via FileOperations)
    /// * `content` - Content to write to the file
    ///
    /// # Errors
    /// Returns error if path is invalid or file creation fails
    pub fn handle(path: &str, content: &str) -> ExecutionResult<()> {
        debug!(path = %path, content_len = content.len(), "Creating file");

        // Use FileOperations wrapper which validates path with PathResolver
        FileOperations::create_file(path, content)?;

        info!(path = %path, "File created successfully");
        Ok(())
    }
}

/// Handles file modification actions
pub struct ModifyFileHandler;

impl ModifyFileHandler {
    /// Modify a file by applying a diff
    ///
    /// # Arguments
    /// * `path` - File path (validated with PathResolver via FileOperations)
    /// * `diff` - Diff to apply to the file
    ///
    /// # Errors
    /// Returns error if path is invalid, file doesn't exist, or diff application fails
    pub fn handle(path: &str, diff: &str) -> ExecutionResult<()> {
        debug!(path = %path, diff_len = diff.len(), "Modifying file");

        // Use FileOperations wrapper which validates path with PathResolver
        FileOperations::modify_file(path, diff)?;

        info!(path = %path, "File modified successfully");
        Ok(())
    }
}

/// Handles file deletion actions
pub struct DeleteFileHandler;

impl DeleteFileHandler {
    /// Delete a file
    ///
    /// # Arguments
    /// * `path` - File path (validated with PathResolver via FileOperations)
    ///
    /// # Errors
    /// Returns error if path is invalid or file deletion fails
    pub fn handle(path: &str) -> ExecutionResult<()> {
        debug!(path = %path, "Deleting file");

        // Use FileOperations wrapper which validates path with PathResolver
        FileOperations::delete_file(path)?;

        info!(path = %path, "File deleted successfully");
        Ok(())
    }
}

/// Handles command execution actions
pub struct CommandHandler;

impl CommandHandler {
    /// Execute a shell command
    ///
    /// # Arguments
    /// * `command` - Command to execute
    /// * `args` - Command arguments
    ///
    /// # Errors
    /// Returns error if command execution fails or returns non-zero exit code
    pub fn handle(command: &str, args: &[String]) -> ExecutionResult<()> {
        debug!(command = %command, args_count = args.len(), "Running command");

        // Create and execute the command
        let mut cmd = Command::new(command);
        cmd.args(args);

        let output = cmd.output().map_err(|e| {
            ExecutionError::StepFailed(format!("Failed to execute command {}: {}", command, e))
        })?;

        // Check exit code
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let exit_code = output.status.code().unwrap_or(-1);
            error!(
                command = %command,
                exit_code = exit_code,
                stderr = %stderr,
                "Command failed"
            );
            return Err(ExecutionError::StepFailed(format!(
                "Command {} failed with exit code {}: {}",
                command, exit_code, stderr
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        info!(
            command = %command,
            output_len = stdout.len(),
            "Command executed successfully"
        );

        Ok(())
    }
}

/// Handles test execution actions
pub struct TestHandler;

impl TestHandler {
    /// Run tests with optional pattern filtering
    ///
    /// # Arguments
    /// * `pattern` - Optional test pattern to filter tests
    ///
    /// # Errors
    /// Returns error if test framework detection fails or tests fail
    pub fn handle(pattern: &Option<String>) -> ExecutionResult<()> {
        debug!(pattern = ?pattern, "Running tests");

        // Detect test framework
        let framework = Self::detect_test_framework()?;

        // Build test command based on framework
        let (command, args) = Self::build_test_command(&framework, pattern)?;

        // Execute tests
        CommandHandler::handle(&command, &args)?;

        info!("Tests executed successfully");
        Ok(())
    }

    /// Detect the test framework based on project structure
    fn detect_test_framework() -> ExecutionResult<TestFramework> {
        let current_dir = std::env::current_dir()
            .map_err(|e| ExecutionError::ValidationError(format!("Failed to get current dir: {}", e)))?;

        // Check for Rust (Cargo.toml)
        if current_dir.join("Cargo.toml").exists() {
            debug!("Detected Rust project");
            return Ok(TestFramework::Rust);
        }

        // Check for TypeScript/Node.js (package.json)
        if current_dir.join("package.json").exists() {
            debug!("Detected TypeScript/Node.js project");
            return Ok(TestFramework::TypeScript);
        }

        // Check for Python (pytest.ini or setup.py)
        if current_dir.join("pytest.ini").exists() || current_dir.join("setup.py").exists() {
            debug!("Detected Python project");
            return Ok(TestFramework::Python);
        }

        Err(ExecutionError::ValidationError(
            "Could not detect test framework".to_string(),
        ))
    }

    /// Build test command for the detected framework
    fn build_test_command(
        framework: &TestFramework,
        pattern: &Option<String>,
    ) -> ExecutionResult<(String, Vec<String>)> {
        match framework {
            TestFramework::Rust => {
                let mut args = vec!["test".to_string()];
                if let Some(p) = pattern {
                    args.push(p.clone());
                }
                Ok(("cargo".to_string(), args))
            }
            TestFramework::TypeScript => {
                let mut args = vec![];
                if let Some(p) = pattern {
                    args.push(p.clone());
                }
                Ok(("npm".to_string(), args))
            }
            TestFramework::Python => {
                let mut args = vec![];
                if let Some(p) = pattern {
                    args.push(p.clone());
                }
                Ok(("pytest".to_string(), args))
            }
        }
    }
}

/// Test framework type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestFramework {
    /// Rust (cargo test)
    Rust,
    /// TypeScript (npm test)
    TypeScript,
    /// Python (pytest)
    Python,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_file_handler() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let path_str = file_path.to_string_lossy().to_string();

        let result = CreateFileHandler::handle(&path_str, "test content");
        assert!(result.is_ok());
        assert!(file_path.exists());

        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "test content");
    }

    #[test]
    fn test_create_file_with_parent_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("subdir/nested/test.txt");
        let path_str = file_path.to_string_lossy().to_string();

        let result = CreateFileHandler::handle(&path_str, "nested content");
        assert!(result.is_ok());
        assert!(file_path.exists());
    }

    #[test]
    fn test_delete_file_handler() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let path_str = file_path.to_string_lossy().to_string();

        // Create the file first
        std::fs::write(&file_path, "content").unwrap();
        assert!(file_path.exists());

        // Delete it
        let result = DeleteFileHandler::handle(&path_str);
        assert!(result.is_ok());
        assert!(!file_path.exists());
    }

    #[test]
    fn test_delete_nonexistent_file() {
        let result = DeleteFileHandler::handle("/nonexistent/path/file.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_modify_file_handler() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let path_str = file_path.to_string_lossy().to_string();

        // Create the file first
        std::fs::write(&file_path, "original content").unwrap();

        // Modify it (with a non-empty diff)
        let result = ModifyFileHandler::handle(&path_str, "some diff");
        assert!(result.is_ok());
    }

    #[test]
    fn test_modify_nonexistent_file() {
        let result = ModifyFileHandler::handle("/nonexistent/path/file.txt", "diff");
        assert!(result.is_err());
    }

    #[test]
    fn test_modify_with_empty_diff() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let path_str = file_path.to_string_lossy().to_string();

        std::fs::write(&file_path, "content").unwrap();

        let result = ModifyFileHandler::handle(&path_str, "");
        assert!(result.is_err());
    }

    #[test]
    fn test_command_handler_success() {
        let result = CommandHandler::handle("echo", &["hello".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_command_handler_failure() {
        let result = CommandHandler::handle("false", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_command_handler_nonexistent() {
        let result = CommandHandler::handle("nonexistent_command_xyz", &[]);
        assert!(result.is_err());
    }
}
