//! Bash/Shell Command Execution Tool
//!
//! Provides shell command execution with security checks, timeout handling,
//! and proper output formatting matching OpenCode patterns.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

use crate::context::ToolContext;
use crate::descriptions::get_description;
use crate::error::ToolError;
use crate::tool::{ParameterSchema, Tool, ToolDefinition, ToolExecutionResult, ToolParameters};

/// Default timeout in milliseconds (2 minutes)
const DEFAULT_TIMEOUT_MS: u64 = 120_000;

/// Maximum output size before truncation (30K chars)
const MAX_OUTPUT_SIZE: usize = 30_000;

/// Bash tool input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BashInput {
    /// Command to execute
    pub command: String,

    /// Optional working directory (overrides context)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workdir: Option<String>,

    /// Optional timeout in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,

    /// Optional description of what this command does
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Bash tool output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BashOutput {
    /// Combined stdout and stderr
    pub output: String,

    /// Exit code
    pub exit_code: i32,

    /// Whether command succeeded (exit code 0)
    pub success: bool,

    /// Execution duration in milliseconds
    pub duration_ms: u64,

    /// Whether output was truncated
    pub truncated: bool,

    /// Working directory used
    pub workdir: String,
}

/// Bash execution tool
pub struct BashTool {
    /// Default workspace root
    workspace_root: PathBuf,
}

impl BashTool {
    /// Create a new BashTool with a workspace root
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }

    /// Create a new BashTool using current directory as workspace
    pub fn with_current_dir() -> Result<Self, ToolError> {
        let workspace_root = std::env::current_dir()
            .map_err(|e| ToolError::new("INIT_ERROR", format!("Failed to get current directory: {}", e)))?;
        Ok(Self { workspace_root })
    }

    /// Execute bash command with given input
    pub async fn execute_command(&self, input: &BashInput, _ctx: &ToolContext) -> Result<BashOutput, ToolError> {
        let start = Instant::now();

        // Validate command
        if input.command.trim().is_empty() {
            return Err(ToolError::new(
                "INVALID_INPUT",
                "Command cannot be empty",
            ));
        }

        // Determine working directory
        let workdir = if let Some(ref dir) = input.workdir {
            PathBuf::from(dir)
        } else {
            self.workspace_root.clone()
        };

        // Verify working directory exists
        if !workdir.exists() {
            return Err(ToolError::new(
                "INVALID_WORKDIR",
                format!("Working directory does not exist: {}", workdir.display()),
            ));
        }

        // Get timeout
        let timeout_ms = input.timeout.unwrap_or(DEFAULT_TIMEOUT_MS);

        // Prepare shell command
        let (shell, shell_arg) = if cfg!(target_os = "windows") {
            // On Windows, prefer Git Bash if available, else cmd
            if let Ok(git_bash) = std::env::var("RICECODER_GIT_BASH_PATH") {
                (git_bash, "-c".to_string())
            } else {
                ("cmd".to_string(), "/C".to_string())
            }
        } else {
            // On Unix, use sh -c
            ("sh".to_string(), "-c".to_string())
        };

        // Execute command with timeout
        let command_future = Command::new(&shell)
            .arg(&shell_arg)
            .arg(&input.command)
            .current_dir(&workdir)
            .env("TERM", "dumb") // Disable terminal formatting
            .output();

        let output = match timeout(Duration::from_millis(timeout_ms), command_future).await {
            Ok(Ok(output)) => output,
            Ok(Err(e)) => {
                return Err(ToolError::new(
                    "EXECUTION_ERROR",
                    format!("Command execution failed: {}", e),
                ));
            }
            Err(_) => {
                return Err(ToolError::new(
                    "TIMEOUT",
                    format!("Command timed out after {}ms", timeout_ms),
                ));
            }
        };

        let duration_ms = start.elapsed().as_millis() as u64;

        // Combine stdout and stderr
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let mut combined = String::new();
        if !stdout.is_empty() {
            combined.push_str(&stdout);
        }
        if !stderr.is_empty() {
            if !combined.is_empty() {
                combined.push('\n');
            }
            combined.push_str(&stderr);
        }

        // Truncate if necessary
        let truncated = combined.len() > MAX_OUTPUT_SIZE;
        let output_text = if truncated {
            format!(
                "{}...\n\n[Output truncated: {} chars total, showing first {}]",
                &combined[..MAX_OUTPUT_SIZE],
                combined.len(),
                MAX_OUTPUT_SIZE
            )
        } else if combined.is_empty() {
            "(no output)".to_string()
        } else {
            combined
        };

        let exit_code = output.status.code().unwrap_or(-1);

        Ok(BashOutput {
            output: output_text,
            exit_code,
            success: output.status.success(),
            duration_ms,
            truncated,
            workdir: workdir.display().to_string(),
        })
    }
}

impl Default for BashTool {
    fn default() -> Self {
        Self::with_current_dir().unwrap_or_else(|_| Self {
            workspace_root: PathBuf::from("."),
        })
    }
}

#[async_trait]
impl Tool for BashTool {
    fn id(&self) -> &str {
        "bash"
    }

    async fn init(&self, _ctx: Option<&ToolContext>) -> Result<ToolDefinition, ToolError> {
        let mut parameters = ToolParameters::new();

        parameters.insert(
            "command".to_string(),
            ParameterSchema {
                type_: "string".to_string(),
                description: "The bash command to execute. Can include pipes, redirects, and multiple commands separated by && or ;".to_string(),
                required: true,
                default: None,
                properties: None,
                items: None,
            },
        );

        parameters.insert(
            "workdir".to_string(),
            ParameterSchema {
                type_: "string".to_string(),
                description: "Working directory for command execution. Defaults to current workspace.".to_string(),
                required: false,
                default: None,
                properties: None,
                items: None,
            },
        );

        parameters.insert(
            "timeout".to_string(),
            ParameterSchema {
                type_: "number".to_string(),
                description: "Optional timeout in milliseconds. Default: 120000 (2 minutes)".to_string(),
                required: false,
                default: Some(Value::Number(DEFAULT_TIMEOUT_MS.into())),
                properties: None,
                items: None,
            },
        );

        parameters.insert(
            "description".to_string(),
            ParameterSchema {
                type_: "string".to_string(),
                description: "Brief description of what this command does (5-10 words)".to_string(),
                required: false,
                default: None,
                properties: None,
                items: None,
            },
        );

        let description = get_description(
            "bash",
            "Execute a bash command in the shell. Returns stdout, stderr, and exit code. Use this for running tests, builds, git commands, and other shell operations.",
        );

        Ok(ToolDefinition {
            description,
            parameters,
            format_validation_error: None,
        })
    }

    async fn execute(
        &self,
        args: HashMap<String, Value>,
        ctx: &ToolContext,
    ) -> Result<ToolExecutionResult, ToolError> {
        // Parse input
        let command = args
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::new("MISSING_PARAM", "Missing required parameter: command"))?
            .to_string();

        let workdir = args.get("workdir").and_then(|v| v.as_str()).map(String::from);
        let timeout_val = args.get("timeout").and_then(|v| v.as_u64());
        let description = args.get("description").and_then(|v| v.as_str()).map(String::from);

        let input = BashInput {
            command: command.clone(),
            workdir,
            timeout: timeout_val,
            description: description.clone(),
        };

        // Execute command
        let result = self.execute_command(&input, ctx).await?;

        // Build title
        let title = description.unwrap_or_else(|| {
            let cmd_preview: String = command.chars().take(50).collect();
            if command.len() > 50 {
                format!("{}...", cmd_preview)
            } else {
                cmd_preview
            }
        });

        // Build metadata
        let mut metadata = HashMap::new();
        metadata.insert("exit_code".to_string(), Value::Number(result.exit_code.into()));
        metadata.insert("success".to_string(), Value::Bool(result.success));
        metadata.insert("duration_ms".to_string(), Value::Number(result.duration_ms.into()));
        metadata.insert("truncated".to_string(), Value::Bool(result.truncated));
        metadata.insert("workdir".to_string(), Value::String(result.workdir));

        // Format output with bash_metadata annotation (OpenCode pattern)
        let output = format!(
            "{}\n\n<bash_metadata>\nexit_code: {}\nduration_ms: {}\n</bash_metadata>",
            result.output,
            result.exit_code,
            result.duration_ms
        );

        Ok(ToolExecutionResult {
            title,
            metadata,
            output,
            attachments: Vec::new(),
        })
    }
}

/// Check if a command is safe for read-only mode
pub fn is_read_only_command(command: &str) -> bool {
    let cmd_lower = command.trim().to_lowercase();

    // Check for dangerous patterns
    if cmd_lower.contains('>')
        || cmd_lower.contains(">>")
        || cmd_lower.contains("| tee")
        || cmd_lower.contains("|tee")
        || cmd_lower.contains("$(")
        || cmd_lower.contains('`')
    {
        return false;
    }

    // Get first command
    let first_cmd = cmd_lower
        .split('|')
        .next()
        .unwrap_or(&cmd_lower)
        .split("&&")
        .next()
        .unwrap_or(&cmd_lower)
        .split(';')
        .next()
        .unwrap_or(&cmd_lower)
        .trim();

    let cmd_name = first_cmd.split_whitespace().next().unwrap_or("");

    // Safe read-only commands
    const SAFE_COMMANDS: &[&str] = &[
        "ls", "cat", "head", "tail", "less", "more", "grep", "find", "tree", "file", "pwd",
        "whoami", "hostname", "date", "echo", "which", "type", "env", "printenv", "df", "du",
        "wc", "curl", "wget", "rg", "fd", "bat", "exa", "eza",
    ];

    const SAFE_GIT_SUBCOMMANDS: &[&str] = &[
        "status", "log", "diff", "branch", "show", "remote", "tag", "describe", "rev-parse",
        "config", "ls-files", "ls-tree", "shortlog", "blame", "reflog",
    ];

    const SAFE_CARGO_SUBCOMMANDS: &[&str] = &[
        "version", "check", "clippy", "fmt", "test", "build", "doc", "tree", "metadata",
    ];

    if SAFE_COMMANDS.contains(&cmd_name) {
        return true;
    }

    if cmd_name == "git" {
        let parts: Vec<&str> = first_cmd.split_whitespace().collect();
        if parts.len() >= 2 {
            return SAFE_GIT_SUBCOMMANDS.contains(&parts[1]);
        }
        return true;
    }

    if cmd_name == "cargo" {
        let parts: Vec<&str> = first_cmd.split_whitespace().collect();
        if parts.len() >= 2 {
            return SAFE_CARGO_SUBCOMMANDS.contains(&parts[1]);
        }
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_read_only_command() {
        // Safe commands
        assert!(is_read_only_command("ls -la"));
        assert!(is_read_only_command("cat file.txt"));
        assert!(is_read_only_command("git status"));
        assert!(is_read_only_command("git log --oneline"));
        assert!(is_read_only_command("cargo check"));

        // Unsafe commands
        assert!(!is_read_only_command("echo test > file.txt"));
        assert!(!is_read_only_command("rm -rf /"));
        assert!(!is_read_only_command("git push"));
        assert!(!is_read_only_command("$(dangerous)"));
    }

    #[tokio::test]
    async fn test_bash_tool_init() {
        let tool = BashTool::default();
        let def = tool.init(None).await.unwrap();

        assert!(def.description.contains("bash"));
        assert!(def.parameters.contains_key("command"));
        assert!(def.parameters.get("command").unwrap().required);
    }

    #[tokio::test]
    async fn test_bash_tool_simple_command() {
        let tool = BashTool::default();
        let ctx = ToolContext::default();

        let mut args = HashMap::new();
        args.insert("command".to_string(), Value::String("echo hello".to_string()));

        let result = tool.execute(args, &ctx).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.output.contains("hello"));
        assert_eq!(output.metadata.get("exit_code"), Some(&Value::Number(0.into())));
    }

    #[tokio::test]
    async fn test_bash_tool_missing_command() {
        let tool = BashTool::default();
        let ctx = ToolContext::default();

        let args = HashMap::new(); // No command provided

        let result = tool.execute(args, &ctx).await;
        assert!(result.is_err());
    }
}
