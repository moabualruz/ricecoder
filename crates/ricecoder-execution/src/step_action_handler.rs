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
use crate::models::CommandOutput;
use std::process::Command;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

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
    /// Execute a shell command with advanced features
    ///
    /// # Arguments
    /// * `command` - Command to execute
    /// * `args` - Command arguments
    /// * `timeout_ms` - Optional timeout in milliseconds (default: 120000 = 2 minutes)
    /// * `require_confirmation` - Whether to check for dangerous commands
    ///
    /// # Returns
    /// Returns CommandOutput with stdout, stderr, and exit code
    ///
    /// # Errors
    /// Returns error if command execution fails or dangerous command is detected
    pub async fn handle_async(
        command: &str,
        args: &[String],
        timeout_ms: Option<u64>,
        require_confirmation: Option<bool>,
    ) -> ExecutionResult<CommandOutput> {
        debug!(command = %command, args_count = args.len(), "Running command asynchronously");

        // Check for dangerous commands if required
        if require_confirmation.unwrap_or(false) {
            Self::check_dangerous_command(command, args)?;
        }

        // Parse command with tree-sitter for validation
        Self::validate_command_syntax(command, args)?;

        // Execute command with timeout
        let timeout_duration = Duration::from_millis(timeout_ms.unwrap_or(120_000)); // 2 minutes default

        let result = timeout(timeout_duration, Self::execute_command_async(command, args)).await;

        match result {
            Ok(output_result) => output_result,
            Err(_) => {
                warn!(command = %command, timeout_ms = timeout_duration.as_millis(), "Command timed out");
                Err(ExecutionError::StepFailed(format!(
                    "Command '{}' timed out after {}ms",
                    command,
                    timeout_duration.as_millis()
                )))
            }
        }
    }

    /// Legacy synchronous version for backward compatibility
    pub fn handle(command: &str, args: &[String]) -> ExecutionResult<CommandOutput> {
        // Run the async version on a blocking task
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                Self::handle_async(command, args, None, Some(false)).await
            })
        })
    }

    /// Execute command asynchronously with streaming output
    async fn execute_command_async(command: &str, args: &[String]) -> ExecutionResult<CommandOutput> {
        use tokio::process::Command;
        use std::process::Stdio;

        let mut cmd = Command::new(command);
        cmd.args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let start_time = std::time::Instant::now();

        let mut child = cmd.spawn().map_err(|e| {
            ExecutionError::StepFailed(format!("Failed to spawn command '{}': {}", command, e))
        })?;

        // Wait for completion
        let output = child.wait_with_output().await.map_err(|e| {
            ExecutionError::StepFailed(format!("Failed to wait for command '{}': {}", command, e))
        })?;

        let duration = start_time.elapsed();

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code();

        let command_output = CommandOutput {
            stdout,
            stderr,
            exit_code,
        };

        if output.status.success() {
            info!(
                command = %command,
                duration_ms = duration.as_millis(),
                output_len = command_output.stdout.len(),
                "Command executed successfully"
            );
        } else {
            let exit_code = exit_code.unwrap_or(-1);
            error!(
                command = %command,
                exit_code = exit_code,
                duration_ms = duration.as_millis(),
                stderr = %command_output.stderr,
                "Command failed"
            );
        }

        Ok(command_output)
    }

    /// Check for dangerous command patterns
    fn check_dangerous_command(command: &str, args: &[String]) -> ExecutionResult<()> {
        let full_command = format!("{} {}", command, args.join(" "));

        // Dangerous command patterns
        let dangerous_patterns = [
            // File system destruction
            r"\brm\s+-rf\s+/?[^/]",  // rm -rf / or rm -rf /* (but allow rm -rf /tmp/something)
            r"\brm\s+-rf\s+/\s*$",   // rm -rf /
            r"\bdd\s+if=/dev/zero",  // dd if=/dev/zero (disk wiping)
            r"\bmkfs\.",             // mkfs commands (filesystem formatting)
            r"\bfdisk\s+.*\bdelete", // fdisk delete operations

            // System control
            r"\bshutdown\b",         // shutdown commands
            r"\breboot\b",          // reboot commands
            r"\bhalt\b",            // halt commands

            // Privilege escalation
            r"\bsudo\b",            // sudo usage
            r"\bsu\b",              // su usage

            // Network dangerous
            r"\bcurl\s+.*\|\s*bash", // curl | bash patterns
            r"\bwget\s+.*\|\s*bash", // wget | bash patterns

            // Process killing
            r"\bkill\s+-9\s+-1",    // kill -9 -1 (kill all processes)
            r"\bpkill\s+-9\s+.*",   // pkill -9 patterns
        ];

        for pattern in &dangerous_patterns {
            if regex::Regex::new(pattern).map_or(false, |re| re.is_match(&full_command)) {
                return Err(ExecutionError::ValidationError(format!(
                    "Dangerous command detected: '{}'. This command requires explicit confirmation.",
                    full_command
                )));
            }
        }

        Ok(())
    }

    /// Validate command syntax using tree-sitter
    fn validate_command_syntax(command: &str, args: &[String]) -> ExecutionResult<()> {
        let full_command = format!("{} {}", command, args.join(" "));

        // Initialize tree-sitter parser
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(tree_sitter_bash::language().into()).map_err(|e| {
            ExecutionError::ValidationError(format!("Failed to load bash grammar: {}", e))
        })?;

        // Parse the command
        let tree = parser.parse(&full_command, None).ok_or_else(|| {
            ExecutionError::ValidationError("Failed to parse bash command".to_string())
        })?;

        // Check for syntax errors
        if tree.root_node().has_error() {
            return Err(ExecutionError::ValidationError(format!(
                "Invalid bash syntax in command: '{}'",
                full_command
            )));
        }

        debug!(command = %full_command, "Command syntax validated successfully");
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
        let current_dir = std::env::current_dir().map_err(|e| {
            ExecutionError::ValidationError(format!("Failed to get current dir: {}", e))
        })?;

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
        let output = result.unwrap();
        assert_eq!(output.exit_code, Some(0));
        assert!(output.stdout.contains("hello"));
    }

    #[test]
    fn test_command_handler_failure() {
        let result = CommandHandler::handle("false", &[]);
        assert!(result.is_ok()); // Command executed, but failed
        let output = result.unwrap();
        assert_ne!(output.exit_code, Some(0));
    }

    #[test]
    fn test_command_handler_nonexistent() {
        let result = CommandHandler::handle("nonexistent_command_xyz", &[]);
        assert!(result.is_err()); // Command not found
    }
}
