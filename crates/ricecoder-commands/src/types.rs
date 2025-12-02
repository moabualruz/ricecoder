use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A custom command definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandDefinition {
    /// Unique identifier for the command
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Description of what the command does
    pub description: String,

    /// The shell command to execute (supports templates)
    pub command: String,

    /// Command arguments with templates
    pub arguments: Vec<CommandArgument>,

    /// Whether the command is enabled
    pub enabled: bool,

    /// Whether to inject output into chat
    pub inject_output: bool,

    /// Timeout in seconds (0 = no timeout)
    pub timeout_seconds: u64,

    /// Tags for categorization
    pub tags: Vec<String>,

    /// Custom metadata
    pub metadata: HashMap<String, String>,
}

/// A command argument definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandArgument {
    /// Argument name
    pub name: String,

    /// Argument description
    pub description: String,

    /// Whether the argument is required
    pub required: bool,

    /// Default value if not provided
    pub default: Option<String>,

    /// Validation pattern (regex)
    pub validation_pattern: Option<String>,

    /// Argument type (string, number, boolean, etc.)
    pub arg_type: ArgumentType,
}

/// Argument type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ArgumentType {
    String,
    Number,
    Boolean,
    Path,
    Choice(Vec<String>),
}

/// Command execution context
#[derive(Debug, Clone)]
pub struct CommandContext {
    /// Current working directory
    pub cwd: String,

    /// Environment variables
    pub env: HashMap<String, String>,

    /// User-provided arguments
    pub arguments: HashMap<String, String>,
}

/// Result of command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandExecutionResult {
    /// Command ID that was executed
    pub command_id: String,

    /// Exit code (0 = success)
    pub exit_code: i32,

    /// Standard output
    pub stdout: String,

    /// Standard error
    pub stderr: String,

    /// Whether execution was successful
    pub success: bool,

    /// Execution duration in milliseconds
    pub duration_ms: u64,
}

impl CommandDefinition {
    /// Create a new command definition
    pub fn new(id: impl Into<String>, name: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            command: command.into(),
            arguments: Vec::new(),
            enabled: true,
            inject_output: false,
            timeout_seconds: 0,
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Set the description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Add an argument
    pub fn with_argument(mut self, argument: CommandArgument) -> Self {
        self.arguments.push(argument);
        self
    }

    /// Set whether to inject output
    pub fn with_inject_output(mut self, inject: bool) -> Self {
        self.inject_output = inject;
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = seconds;
        self
    }

    /// Add a tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Set enabled state
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

impl CommandArgument {
    /// Create a new command argument
    pub fn new(name: impl Into<String>, arg_type: ArgumentType) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            required: false,
            default: None,
            validation_pattern: None,
            arg_type,
        }
    }

    /// Set the description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Set whether the argument is required
    pub fn with_required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Set the default value
    pub fn with_default(mut self, default: impl Into<String>) -> Self {
        self.default = Some(default.into());
        self
    }

    /// Set the validation pattern
    pub fn with_validation_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.validation_pattern = Some(pattern.into());
        self
    }
}

impl CommandExecutionResult {
    /// Create a new execution result
    pub fn new(command_id: impl Into<String>, exit_code: i32) -> Self {
        Self {
            command_id: command_id.into(),
            exit_code,
            stdout: String::new(),
            stderr: String::new(),
            success: exit_code == 0,
            duration_ms: 0,
        }
    }

    /// Set stdout
    pub fn with_stdout(mut self, stdout: impl Into<String>) -> Self {
        self.stdout = stdout.into();
        self
    }

    /// Set stderr
    pub fn with_stderr(mut self, stderr: impl Into<String>) -> Self {
        self.stderr = stderr.into();
        self
    }

    /// Set duration
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = duration_ms;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_definition_creation() {
        let cmd = CommandDefinition::new("test-cmd", "Test Command", "echo hello");
        assert_eq!(cmd.id, "test-cmd");
        assert_eq!(cmd.name, "Test Command");
        assert_eq!(cmd.command, "echo hello");
        assert!(cmd.enabled);
        assert!(!cmd.inject_output);
    }

    #[test]
    fn test_command_definition_builder() {
        let cmd = CommandDefinition::new("test-cmd", "Test Command", "echo hello")
            .with_description("A test command")
            .with_inject_output(true)
            .with_timeout(30)
            .with_tag("test")
            .with_enabled(false);

        assert_eq!(cmd.description, "A test command");
        assert!(cmd.inject_output);
        assert_eq!(cmd.timeout_seconds, 30);
        assert_eq!(cmd.tags, vec!["test"]);
        assert!(!cmd.enabled);
    }

    #[test]
    fn test_command_argument_creation() {
        let arg = CommandArgument::new("name", ArgumentType::String)
            .with_description("User name")
            .with_required(true);

        assert_eq!(arg.name, "name");
        assert_eq!(arg.description, "User name");
        assert!(arg.required);
        assert_eq!(arg.arg_type, ArgumentType::String);
    }

    #[test]
    fn test_execution_result_creation() {
        let result = CommandExecutionResult::new("test-cmd", 0)
            .with_stdout("output")
            .with_duration(100);

        assert_eq!(result.command_id, "test-cmd");
        assert_eq!(result.exit_code, 0);
        assert!(result.success);
        assert_eq!(result.stdout, "output");
        assert_eq!(result.duration_ms, 100);
    }
}
