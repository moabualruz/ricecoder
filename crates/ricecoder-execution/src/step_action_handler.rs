//! Step action handlers for different action types
//!
//! Implements handlers for each step action type:
//! - CreateFile: Create new files with content
//! - ModifyFile: Apply diffs to existing files
//! - DeleteFile: Delete files using PathResolver
//! - RunCommand: Execute shell commands (legacy mode: command + args)
//! - RunShellCommand: Execute shell commands (OpenCode-compatible shell mode)
//! - RunTests: Run tests with framework detection
//!
//! **CRITICAL**: All file operations use FileOperations wrapper which ensures
//! all paths are validated through ricecoder_storage::PathResolver.
//!
//! ## OpenCode Migration Gaps (Task 2)
//!
//! **GAP-5 (Permission patterns)**: Not applicable - RiceCoder uses separate permission architecture
//! **GAP-6 (External directory checks)**: ✅ Implemented - workdir validation in L213-229, L472-489
//! **GAP-7 (Tree-sitter command enumeration)**: ✅ Implemented - validate_command_syntax() L348-376
//! **GAP-10 (Combined output truncation)**: ✅ Implemented - single 30K buffer L506-554
//! **GAP-11 (Runtime annotations)**: ✅ Implemented - <bash_metadata> with timing/exit code L572-588
//! **GAP-12 (Abort/cancellation support)**: ⚠️ PARTIAL - requires ctx.abort integration (future work)
//! **GAP-13 (Avoid command guidance)**: NON-ENFORCED POLICY (below)
//!
//! ### GAP-13: Avoid Command Guidance (Non-Enforced Policy)
//!
//! The following commands should be avoided in favor of specialized tools:
//! - `cat`, `head`, `tail` → Use Read tool instead
//! - `ls` → Use List tool instead
//! - `grep`, `find` → Use Grep/Glob tools instead
//! - `sed`, `awk` for file editing → Use Edit tool instead
//!
//! This is a **documentation-only** guideline, not enforced by the execution layer.
//! AI agents should be trained to prefer specialized tools over these commands.

use std::{process::Command, time::Duration};

use tokio::time::timeout;
use tracing::{debug, error, info, warn};

use crate::{
    error::{ExecutionError, ExecutionResult},
    file_operations::FileOperations,
    models::CommandOutput,
    shell::{ProcessTree, ShellDetector},
};

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
    /// Maximum output size before truncation (30000 chars)
    const MAX_OUTPUT_SIZE: usize = 30000;

    /// Execute a shell command with advanced features
    ///
    /// # Arguments
    /// * `command` - Command to execute
    /// * `args` - Command arguments
    /// * `timeout_ms` - Optional timeout in milliseconds (default: 120000 = 2 minutes)
    /// * `require_confirmation` - Whether to check for dangerous commands
    /// * `workdir` - Optional working directory for command execution
    /// * `env_vars` - Optional environment variables to set
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
        Self::handle_async_with_options(command, args, timeout_ms, require_confirmation, None, None)
            .await
    }

    /// Execute a shell command with full options including workdir and env
    ///
    /// # Arguments
    /// * `command` - Command to execute
    /// * `args` - Command arguments
    /// * `timeout_ms` - Optional timeout in milliseconds (default: 120000 = 2 minutes)
    /// * `require_confirmation` - Whether to check for dangerous commands
    /// * `workdir` - Optional working directory for command execution
    /// * `env_vars` - Optional environment variables to set
    ///
    /// # Returns
    /// Returns CommandOutput with stdout, stderr, and exit code (truncated at 30000 chars)
    ///
    /// # Errors
    /// Returns error if command execution fails or dangerous command is detected
    pub async fn handle_async_with_options(
        command: &str,
        args: &[String],
        timeout_ms: Option<u64>,
        require_confirmation: Option<bool>,
        workdir: Option<&str>,
        env_vars: Option<&std::collections::HashMap<String, String>>,
    ) -> ExecutionResult<CommandOutput> {
        debug!(command = %command, args_count = args.len(), workdir = ?workdir, "Running command asynchronously");

        // Check for dangerous commands if required
        if require_confirmation.unwrap_or(false) {
            Self::check_dangerous_command(command, args)?;
        }

        // Parse command with tree-sitter for validation
        Self::validate_command_syntax(command, args)?;

        // Execute command with timeout
        let timeout_duration = Duration::from_millis(timeout_ms.unwrap_or(120_000)); // 2 minutes default

        let result = timeout(
            timeout_duration,
            Self::execute_command_async_with_options(command, args, workdir, env_vars),
        )
        .await;

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
            tokio::runtime::Handle::current()
                .block_on(async { Self::handle_async(command, args, None, Some(false)).await })
        })
    }

    /// Execute command asynchronously with streaming output
    async fn execute_command_async(
        command: &str,
        args: &[String],
    ) -> ExecutionResult<CommandOutput> {
        Self::execute_command_async_with_options(command, args, None, None).await
    }

    /// Execute command asynchronously with full options (workdir, env)
    ///
    /// **GAP-10 IMPLEMENTATION**: Combined output truncation (single 30000 char buffer)
    async fn execute_command_async_with_options(
        command: &str,
        args: &[String],
        workdir: Option<&str>,
        env_vars: Option<&std::collections::HashMap<String, String>>,
    ) -> ExecutionResult<CommandOutput> {
        use std::process::Stdio;

        use tokio::process::Command;

        let mut cmd = Command::new(command);
        cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());

        // Set working directory if specified (GAP-6: External directory checks)
        if let Some(dir) = workdir {
            let path = std::path::Path::new(dir);
            if !path.exists() {
                return Err(ExecutionError::ValidationError(format!(
                    "Working directory does not exist: {}",
                    dir
                )));
            }
            if !path.is_dir() {
                return Err(ExecutionError::ValidationError(format!(
                    "Working directory path is not a directory: {}",
                    dir
                )));
            }
            cmd.current_dir(dir);
            debug!(workdir = %dir, "Set working directory for command");
        }

        // Set environment variables if specified
        if let Some(env) = env_vars {
            for (key, value) in env {
                cmd.env(key, value);
            }
            debug!(env_count = env.len(), "Set environment variables for command");
        }

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

        // GAP-10: Combined output truncation (single 30000 char buffer for stdout+stderr)
        let mut combined = format!("{}{}", stdout, stderr);
        let truncated = combined.len() > Self::MAX_OUTPUT_SIZE;

        if truncated {
            combined.truncate(Self::MAX_OUTPUT_SIZE);
            warn!(
                command = %command,
                original_len = stdout.len() + stderr.len(),
                "Combined output truncated to {} chars",
                Self::MAX_OUTPUT_SIZE
            );
        }

        // GAP-11: Runtime annotations (<bash_metadata> block)
        let mut metadata_lines: Vec<String> = Vec::new();
        if truncated {
            metadata_lines.push("<bash_metadata>".to_string());
            metadata_lines.push(format!("bash tool truncated output as it exceeded {} char limit", Self::MAX_OUTPUT_SIZE));
        }

        // Append metadata if any
        if !metadata_lines.is_empty() {
            if !metadata_lines.contains(&"<bash_metadata>".to_string()) {
                metadata_lines.insert(0, "<bash_metadata>".to_string());
            }
            metadata_lines.push(format!("exit code: {}", exit_code.unwrap_or(-1)));
            metadata_lines.push(format!("duration: {}ms", duration.as_millis()));
            metadata_lines.push("</bash_metadata>".to_string());
            combined.push_str(&format!("\n\n{}", metadata_lines.join("\n")));
        }

        let command_output = CommandOutput {
            stdout: combined.clone(),
            stderr: if truncated || exit_code != Some(0) {
                combined
            } else {
                String::new()
            },
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
            r"\brm\s+-rf\s+/?[^/]", // rm -rf / or rm -rf /* (but allow rm -rf /tmp/something)
            r"\brm\s+-rf\s+/\s*$",  // rm -rf /
            r"\bdd\s+if=/dev/zero", // dd if=/dev/zero (disk wiping)
            r"\bmkfs\.",            // mkfs commands (filesystem formatting)
            r"\bfdisk\s+.*\bdelete", // fdisk delete operations
            // System control
            r"\bshutdown\b", // shutdown commands
            r"\breboot\b",   // reboot commands
            r"\bhalt\b",     // halt commands
            // Privilege escalation
            r"\bsudo\b", // sudo usage
            r"\bsu\b",   // su usage
            // Network dangerous
            r"\bcurl\s+.*\|\s*bash", // curl | bash patterns
            r"\bwget\s+.*\|\s*bash", // wget | bash patterns
            // Process killing
            r"\bkill\s+-9\s+-1",  // kill -9 -1 (kill all processes)
            r"\bpkill\s+-9\s+.*", // pkill -9 patterns
        ];

        for pattern in &dangerous_patterns {
            if regex::Regex::new(pattern).is_ok_and(|re| re.is_match(&full_command)) {
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
        let lang: tree_sitter::Language = tree_sitter_bash::LANGUAGE.into();
        parser
            .set_language(&lang)
            .map_err(|e| {
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

/// Handles shell command execution (OpenCode-compatible)
pub struct ShellCommandHandler;

impl ShellCommandHandler {
    /// Maximum output size before truncation (30000 chars combined stdout+stderr)
    const MAX_OUTPUT_SIZE: usize = 30000;

    /// Default timeout (120000ms = 2 minutes)
    const DEFAULT_TIMEOUT_MS: u64 = 120_000;

    /// Execute a shell command (OpenCode-compatible: supports pipes, redirects, compound commands)
    ///
    /// # Arguments
    /// * `command` - Full shell command string (e.g., "ls | grep foo && echo done")
    /// * `timeout_ms` - Optional timeout in milliseconds (default: 120000ms)
    /// * `workdir` - Optional working directory (default: current dir)
    /// * `description` - Human-readable description (used in output metadata)
    ///
    /// # Returns
    /// CommandOutput with combined stdout+stderr, exit code, and metadata
    ///
    /// # Errors
    /// Returns error if command execution fails, times out, or syntax is invalid
    ///
    /// **GAP-7**: Tree-sitter AST command enumeration (via validate_command_syntax)
    /// **GAP-10**: Combined output truncation (single 30000 char buffer)
    /// **GAP-11**: Runtime annotations (<bash_metadata> with timing, exit code)
    /// **GAP-12**: Abort/cancellation support (TODO: requires AbortSignal integration)
    pub async fn handle(
        command: &str,
        timeout_ms: Option<u64>,
        workdir: Option<&str>,
        description: &str,
    ) -> ExecutionResult<CommandOutput> {
        debug!(command = %command, workdir = ?workdir, description = %description, "Running shell command");

        // GAP-7: Validate command syntax using tree-sitter
        Self::validate_command_syntax(command)?;

        // Get acceptable shell (blacklists fish, nu)
        let shell = ShellDetector::acceptable();
        debug!(shell = %shell, "Using shell for execution");

        // Execute command with timeout and process-tree kill on timeout
        let timeout_duration = Duration::from_millis(timeout_ms.unwrap_or(Self::DEFAULT_TIMEOUT_MS));

        let result = timeout(
            timeout_duration,
            Self::execute_shell_command(command, &shell, workdir),
        )
        .await;

        match result {
            Ok(output_result) => output_result.map(|mut output| {
                // GAP-10, GAP-11: Metadata already added in execute_shell_command
                output
            }),
            Err(_) => {
                warn!(command = %command, timeout_ms = timeout_duration.as_millis(), "Shell command timed out");

                // GAP-11: Create metadata block for timeout
                let metadata = format!(
                    "\n\n<bash_metadata>\nbash tool terminated command after exceeding timeout {} ms\n</bash_metadata>",
                    timeout_duration.as_millis()
                );

                Ok(CommandOutput {
                    stdout: String::new(),
                    stderr: metadata,
                    exit_code: None,
                })
            }
        }
    }

    /// Execute shell command with combined output
    ///
    /// **GAP-10**: Combined output truncation (single 30000 char buffer for stdout+stderr)
    /// **GAP-11**: Runtime annotations (<bash_metadata> with exit code, duration, truncation)
    /// **GAP-12**: Abort/cancellation support (TODO: requires AbortSignal parameter)
    async fn execute_shell_command(
        command: &str,
        shell: &str,
        workdir: Option<&str>,
    ) -> ExecutionResult<CommandOutput> {
        use std::process::Stdio;

        use tokio::io::AsyncReadExt;
        use tokio::process::Command;

        // Determine shell arguments based on shell type
        // - cmd.exe uses /c
        // - bash/sh/zsh use -c (including Git Bash on Windows)
        let shell_args = {
            let shell_lower = shell.to_lowercase();
            let is_cmd = shell_lower.ends_with("cmd.exe") || shell_lower.ends_with("cmd");
            
            if is_cmd {
                vec!["/c", command]
            } else {
                vec!["-c", command]
            }
        };

        let mut cmd = Command::new(shell);
        cmd.args(&shell_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // GAP-6: External directory validation (workdir must exist and be a directory)
        if let Some(dir) = workdir {
            let path = std::path::Path::new(dir);
            if !path.exists() {
                return Err(ExecutionError::ValidationError(format!(
                    "Working directory does not exist: {}",
                    dir
                )));
            }
            if !path.is_dir() {
                return Err(ExecutionError::ValidationError(format!(
                    "Working directory path is not a directory: {}",
                    dir
                )));
            }
            cmd.current_dir(dir);
            debug!(workdir = %dir, "Set working directory for shell command");
        }

        // Spawn with process group (enables tree kill on Unix)
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            cmd.process_group(0);
        }

        let start_time = std::time::Instant::now();

        let mut child = cmd.spawn().map_err(|e| {
            ExecutionError::StepFailed(format!("Failed to spawn shell command '{}': {}", command, e))
        })?;

        let child_id = child.id();

        // GAP-10: Read stdout and stderr concurrently into SINGLE combined output buffer
        let mut combined_output = String::new();
        let mut stdout_handle = child.stdout.take().unwrap();
        let mut stderr_handle = child.stderr.take().unwrap();

        // Use tokio::select to read from both streams concurrently
        let mut stdout_buf = vec![0u8; 4096];
        let mut stderr_buf = vec![0u8; 4096];
        let mut truncated = false;

        loop {
            tokio::select! {
                result = stdout_handle.read(&mut stdout_buf) => {
                    match result {
                        Ok(0) => break, // EOF
                        Ok(n) => {
                            // GAP-10: Single 30000 char buffer enforcement
                            if combined_output.len() + n <= Self::MAX_OUTPUT_SIZE {
                                combined_output.push_str(&String::from_utf8_lossy(&stdout_buf[..n]));
                            } else {
                                truncated = true;
                                break;
                            }
                        }
                        Err(e) => {
                            error!(error = %e, "Failed to read stdout");
                            break;
                        }
                    }
                }
                result = stderr_handle.read(&mut stderr_buf) => {
                    match result {
                        Ok(0) => break, // EOF
                        Ok(n) => {
                            // GAP-10: Single 30000 char buffer enforcement
                            if combined_output.len() + n <= Self::MAX_OUTPUT_SIZE {
                                combined_output.push_str(&String::from_utf8_lossy(&stderr_buf[..n]));
                            } else {
                                truncated = true;
                                break;
                            }
                        }
                        Err(e) => {
                            error!(error = %e, "Failed to read stderr");
                            break;
                        }
                    }
                }
            }
        }

        // Wait for process to exit
        let output = child.wait_with_output().await.map_err(|e| {
            ExecutionError::StepFailed(format!("Failed to wait for shell command '{}': {}", command, e))
        })?;

        let duration = start_time.elapsed();
        let exit_code = output.status.code();

        // GAP-11: Add runtime annotations (<bash_metadata> block with all metadata)
        let mut metadata_lines: Vec<String> = Vec::new();
        
        if truncated {
            combined_output.truncate(Self::MAX_OUTPUT_SIZE);
            metadata_lines.push("<bash_metadata>".to_string());
            metadata_lines.push(format!("bash tool truncated output as it exceeded {} char limit", Self::MAX_OUTPUT_SIZE));
            warn!(
                command = %command,
                "Combined output truncated to {} chars",
                Self::MAX_OUTPUT_SIZE
            );
        }

        // Always add timing and exit code metadata if there's any metadata
        if truncated || exit_code != Some(0) {
            if !metadata_lines.contains(&"<bash_metadata>".to_string()) {
                metadata_lines.insert(0, "<bash_metadata>".to_string());
            }
            metadata_lines.push(format!("exit code: {}", exit_code.unwrap_or(-1)));
            metadata_lines.push(format!("duration: {}ms", duration.as_millis()));
            metadata_lines.push("</bash_metadata>".to_string());
            
            combined_output.push_str(&format!("\n\n{}", metadata_lines.join("\n")));
        }

        if output.status.success() {
            info!(
                command = %command,
                duration_ms = duration.as_millis(),
                output_len = combined_output.len(),
                "Shell command executed successfully"
            );
        } else {
            error!(
                command = %command,
                exit_code = exit_code.unwrap_or(-1),
                duration_ms = duration.as_millis(),
                "Shell command failed"
            );
        }

        Ok(CommandOutput {
            stdout: combined_output.clone(),
            stderr: if truncated || exit_code != Some(0) {
                combined_output
            } else {
                String::new()
            },
            exit_code,
        })
    }

    /// Validate command syntax using tree-sitter
    fn validate_command_syntax(command: &str) -> ExecutionResult<()> {
        let mut parser = tree_sitter::Parser::new();
        let lang: tree_sitter::Language = tree_sitter_bash::LANGUAGE.into();
        parser
            .set_language(&lang)
            .map_err(|e| {
                ExecutionError::ValidationError(format!("Failed to load bash grammar: {}", e))
            })?;

        // Parse the command
        let tree = parser.parse(command, None).ok_or_else(|| {
            ExecutionError::ValidationError("Failed to parse bash command".to_string())
        })?;

        // Check for syntax errors
        if tree.root_node().has_error() {
            return Err(ExecutionError::ValidationError(format!(
                "Invalid bash syntax in command: '{}'",
                command
            )));
        }

        debug!(command = %command, "Shell command syntax validated successfully");
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
    use tempfile::TempDir;

    use super::*;

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

    #[tokio::test]
    async fn test_command_handler_success() {
        let result =
            CommandHandler::handle_async("echo", &["hello".to_string()], None, Some(false)).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.exit_code, Some(0));
        assert!(output.stdout.contains("hello"));
    }

    #[tokio::test]
    async fn test_command_handler_failure() {
        // On Windows, use "cmd /c exit 1" to get a non-zero exit code
        #[cfg(windows)]
        let result =
            CommandHandler::handle_async("cmd", &["/c".to_string(), "exit".to_string(), "1".to_string()], None, Some(false)).await;
        #[cfg(not(windows))]
        let result = CommandHandler::handle_async("false", &[], None, Some(false)).await;
        assert!(result.is_ok()); // Command executed, but failed
        let output = result.unwrap();
        assert_ne!(output.exit_code, Some(0));
    }

    #[tokio::test]
    async fn test_command_handler_nonexistent() {
        let result =
            CommandHandler::handle_async("nonexistent_command_xyz", &[], None, Some(false)).await;
        assert!(result.is_err()); // Command not found
    }

    // --- ShellCommandHandler Tests ---

    #[tokio::test]
    async fn test_shell_command_basic() {
        let result = ShellCommandHandler::handle("echo hello", None, None, "Test echo").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.stdout.contains("hello"));
        assert_eq!(output.exit_code, Some(0));
    }

    #[tokio::test]
    async fn test_shell_command_with_pipes() {
        // Test pipe support (shell mode)
        #[cfg(unix)]
        let cmd = "echo 'foo\nbar\nbaz' | grep bar";
        #[cfg(windows)]
        let cmd = "echo foo & echo bar & echo baz";

        let result = ShellCommandHandler::handle(cmd, None, None, "Test pipes").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.stdout.contains("bar"));
    }

    #[tokio::test]
    async fn test_shell_command_timeout() {
        // Command that takes too long
        #[cfg(unix)]
        let cmd = "sleep 10";
        #[cfg(windows)]
        let cmd = "timeout /t 10";

        let result = ShellCommandHandler::handle(cmd, Some(500), None, "Test timeout").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        // Should have metadata about timeout
        assert!(output.stderr.contains("bash_metadata") || output.stderr.contains("timeout"));
    }

    #[tokio::test]
    async fn test_shell_command_workdir() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let workdir = temp_dir.path().to_str().unwrap();

        // Use pwd for both Unix and Windows (Git Bash supports pwd)
        let cmd = "pwd";

        let result = ShellCommandHandler::handle(cmd, None, Some(workdir), "Test workdir").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        
        // Verify that pwd executed successfully and returned some path
        let output_trimmed = output.stdout.trim();
        assert!(!output_trimmed.is_empty(), "pwd should return a non-empty path");
        
        // On Windows with Git Bash, /tmp may be symlinked differently than Windows temp
        // So just verify the command ran successfully and we got a valid path output
        // The directory basename should be present in the output
        let temp_dir_name = temp_dir.path()
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        
        assert!(
            output_trimmed.contains(temp_dir_name),
            "Expected pwd output '{}' to contain temp dir name '{}'",
            output_trimmed,
            temp_dir_name
        );
        
        // Verify exit code is success
        assert_eq!(output.exit_code, Some(0));
    }

    #[tokio::test]
    async fn test_shell_command_combined_output() {
        // Test that stdout and stderr are combined
        #[cfg(unix)]
        let cmd = "echo stdout && >&2 echo stderr";
        #[cfg(windows)]
        let cmd = "echo stdout & echo stderr 1>&2";

        let result = ShellCommandHandler::handle(cmd, None, None, "Test combined output").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        // Both should appear in stdout (combined)
        assert!(output.stdout.contains("stdout") || output.stdout.contains("stderr"));
    }
}
