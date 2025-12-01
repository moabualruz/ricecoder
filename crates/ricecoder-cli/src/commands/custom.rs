// Custom commands module
// Allows users to define and execute custom commands with template substitution

use crate::error::{CliError, CliResult};
use super::Command;
use std::path::PathBuf;
use std::time::Duration;

/// Represents a custom command definition
#[derive(Debug, Clone)]
pub struct CommandDef {
    /// Command name (unique identifier)
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Command template with placeholders for substitution
    pub template: String,
    /// Optional agent to use for execution
    pub agent: Option<String>,
    /// Optional model to use for execution
    pub model: Option<String>,
    /// Whether this command should be treated as a subtask
    pub subtask: Option<bool>,
}

impl CommandDef {
    /// Create a new command definition
    pub fn new(name: String, description: String, template: String) -> Self {
        Self {
            name,
            description,
            template,
            agent: None,
            model: None,
            subtask: None,
        }
    }

    /// Validate the command definition
    pub fn validate(&self) -> CliResult<()> {
        // Check required fields
        if self.name.is_empty() {
            return Err(CliError::InvalidArgument {
                message: "Command name cannot be empty".to_string(),
            });
        }

        if self.description.is_empty() {
            return Err(CliError::InvalidArgument {
                message: "Command description cannot be empty".to_string(),
            });
        }

        if self.template.is_empty() {
            return Err(CliError::InvalidArgument {
                message: "Command template cannot be empty".to_string(),
            });
        }

        Ok(())
    }
}

/// Represents the context for command execution
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// The command definition to execute
    pub command: CommandDef,
    /// Arguments provided to the command
    pub arguments: Vec<String>,
    /// Working directory for execution
    pub working_dir: PathBuf,
    /// Timeout in seconds for command execution
    pub timeout_secs: u64,
}

impl ExecutionContext {
    /// Create a new execution context
    pub fn new(
        command: CommandDef,
        arguments: Vec<String>,
        working_dir: PathBuf,
    ) -> Self {
        Self {
            command,
            arguments,
            working_dir,
            timeout_secs: 30, // Default 30 second timeout
        }
    }

    /// Set a custom timeout
    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    /// Get the timeout as a Duration
    pub fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_secs)
    }
}

/// Status of command execution
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionStatus {
    /// Command executed successfully
    Success,
    /// Error in argument substitution or validation
    ArgumentError(String),
    /// Referenced file not found
    FileNotFound(String),
    /// Shell command execution failed
    ShellCommandFailed(String),
    /// Command execution timed out
    Timeout,
    /// Invalid template or placeholder
    InvalidTemplate(String),
}

impl ExecutionStatus {
    /// Check if the status represents success
    pub fn is_success(&self) -> bool {
        matches!(self, ExecutionStatus::Success)
    }

    /// Check if the status represents an error
    pub fn is_error(&self) -> bool {
        !self.is_success()
    }

    /// Get a human-readable message for the status
    pub fn message(&self) -> String {
        match self {
            ExecutionStatus::Success => "Command executed successfully".to_string(),
            ExecutionStatus::ArgumentError(msg) => format!("Argument error: {}", msg),
            ExecutionStatus::FileNotFound(path) => format!("File not found: {}", path),
            ExecutionStatus::ShellCommandFailed(msg) => format!("Shell command failed: {}", msg),
            ExecutionStatus::Timeout => "Command execution timed out".to_string(),
            ExecutionStatus::InvalidTemplate(msg) => format!("Invalid template: {}", msg),
        }
    }
}

/// Result of command execution
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Status of the execution
    pub status: ExecutionStatus,
    /// Output from the command execution
    pub output: String,
    /// Optional error message
    pub error: Option<String>,
    /// Duration of execution in milliseconds
    pub duration_ms: u64,
}

impl ExecutionResult {
    /// Create a successful execution result
    pub fn success(output: String, duration_ms: u64) -> Self {
        Self {
            status: ExecutionStatus::Success,
            output,
            error: None,
            duration_ms,
        }
    }

    /// Create a failed execution result
    pub fn error(status: ExecutionStatus, error_msg: String, duration_ms: u64) -> Self {
        Self {
            status,
            output: String::new(),
            error: Some(error_msg),
            duration_ms,
        }
    }

    /// Check if execution was successful
    pub fn is_success(&self) -> bool {
        self.status.is_success()
    }
}

/// Custom command handler
pub struct CustomCommand {
    pub command_def: CommandDef,
}

impl CustomCommand {
    /// Create a new custom command handler
    pub fn new(command_def: CommandDef) -> Self {
        Self { command_def }
    }
}

impl Command for CustomCommand {
    fn execute(&self) -> CliResult<()> {
        // Validate the command definition
        self.command_def.validate()?;

        // TODO: Implement actual command execution
        // This will be implemented in subsequent tasks

        Ok(())
    }
}

/// Command executor for executing registered commands with template substitution
pub mod executor {
    use super::*;
    use std::time::Instant;

    /// Execute a command with the given context
    ///
    /// This function:
    /// 1. Validates the command definition
    /// 2. Performs argument substitution
    /// 3. Executes shell commands if present
    /// 4. Includes file references if present
    /// 5. Returns the final result
    ///
    /// # Arguments
    /// - `context`: The execution context with command and arguments
    ///
    /// # Returns
    /// - `Ok(ExecutionResult)`: The result of command execution
    /// - `Err(CliError)`: If execution fails
    pub fn execute(context: &ExecutionContext) -> CliResult<ExecutionResult> {
        let start = Instant::now();

        // Validate the command definition
        context.command.validate()?;

        // Start with the template
        let mut template = context.command.template.clone();

        // Step 1: Substitute file references first (before shell commands)
        if file_reference::has_references(&template) {
            match file_reference::substitute(
                &template,
                &context.working_dir,
                truncation::DEFAULT_MAX_FILE_SIZE,
            ) {
                Ok(substituted) => template = substituted,
                Err(e) => {
                    let duration_ms = start.elapsed().as_millis() as u64;
                    return Ok(ExecutionResult::error(
                        ExecutionStatus::FileNotFound(e.to_string()),
                        e.to_string(),
                        duration_ms,
                    ));
                }
            }
        }

        // Step 2: Execute shell commands (if present)
        // Shell commands are in the format: !command
        if template.contains('!') {
            template = match execute_shell_injections(&template, context.timeout_secs) {
                Ok(result) => result,
                Err(e) => {
                    let duration_ms = start.elapsed().as_millis() as u64;
                    return Ok(ExecutionResult::error(
                        ExecutionStatus::ShellCommandFailed(e.to_string()),
                        e.to_string(),
                        duration_ms,
                    ));
                }
            };
        }

        // Step 3: Substitute arguments
        match template::substitute(&template, &context.arguments, true) {
            Ok(final_template) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                Ok(ExecutionResult::success(final_template, duration_ms))
            }
            Err(e) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                Ok(ExecutionResult::error(
                    ExecutionStatus::ArgumentError(e.to_string()),
                    e.to_string(),
                    duration_ms,
                ))
            }
        }
    }

    /// Execute shell command injections in a template
    ///
    /// Shell commands are in the format: !command
    /// They are executed and their output is injected into the template
    fn execute_shell_injections(template: &str, timeout_secs: u64) -> Result<String, String> {
        let mut result = template.to_string();

        // Find all shell command injections: !command
        let regex = regex::Regex::new(r"!([a-zA-Z0-9_\-./\s]+)").map_err(|e| e.to_string())?;

        // Collect all matches first to avoid borrowing issues
        let matches: Vec<_> = regex
            .captures_iter(template)
            .map(|cap| {
                let full_match = cap.get(0).unwrap().as_str().to_string();
                let command = cap.get(1).unwrap().as_str().to_string();
                (full_match, command)
            })
            .collect();

        // Execute each shell command
        for (full_match, command) in matches {
            // Parse the command and arguments
            let parts: Vec<&str> = command.split_whitespace().collect();
            if parts.is_empty() {
                return Err("Empty shell command".to_string());
            }

            // Execute the command
            match shell::execute(parts[0], &parts[1..], timeout_secs) {
                Ok(shell_result) => {
                    // Truncate output if needed
                    let (output, _) =
                        truncation::truncate_output(&shell_result.stdout, truncation::DEFAULT_MAX_OUTPUT_SIZE);
                    result = result.replace(&full_match, &output);
                }
                Err(e) => {
                    return Err(format!("Shell command failed: {}", e));
                }
            }
        }

        Ok(result)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_execute_simple_command() {
            let cmd = CommandDef::new(
                "test".to_string(),
                "Test command".to_string(),
                "echo $ARGUMENTS".to_string(),
            );

            let context = ExecutionContext::new(
                cmd,
                vec!["hello".to_string(), "world".to_string()],
                PathBuf::from("."),
            );

            let result = execute(&context);
            assert!(result.is_ok());

            let exec_result = result.unwrap();
            assert!(exec_result.is_success());
            assert!(exec_result.output.contains("hello"));
            assert!(exec_result.output.contains("world"));
        }

        #[test]
        fn test_execute_with_positional_args() {
            let cmd = CommandDef::new(
                "test".to_string(),
                "Test command".to_string(),
                "echo $1 and $2".to_string(),
            );

            let context = ExecutionContext::new(
                cmd,
                vec!["first".to_string(), "second".to_string()],
                PathBuf::from("."),
            );

            let result = execute(&context);
            assert!(result.is_ok());

            let exec_result = result.unwrap();
            assert!(exec_result.is_success());
            assert!(exec_result.output.contains("first"));
            assert!(exec_result.output.contains("second"));
        }

        #[test]
        fn test_execute_missing_argument() {
            let cmd = CommandDef::new(
                "test".to_string(),
                "Test command".to_string(),
                "echo $1 and $2".to_string(),
            );

            let context = ExecutionContext::new(
                cmd,
                vec!["only_one".to_string()],
                PathBuf::from("."),
            );

            let result = execute(&context);
            assert!(result.is_ok());

            let exec_result = result.unwrap();
            assert!(!exec_result.is_success());
            match exec_result.status {
                ExecutionStatus::ArgumentError(_) => {}
                _ => panic!("Expected ArgumentError"),
            }
        }

        #[test]
        fn test_execute_no_arguments() {
            let cmd = CommandDef::new(
                "test".to_string(),
                "Test command".to_string(),
                "echo hello".to_string(),
            );

            let context = ExecutionContext::new(cmd, vec![], PathBuf::from("."));

            let result = execute(&context);
            assert!(result.is_ok());

            let exec_result = result.unwrap();
            assert!(exec_result.is_success());
            assert_eq!(exec_result.output, "echo hello");
        }

        #[test]
        fn test_execute_with_special_characters() {
            let cmd = CommandDef::new(
                "test".to_string(),
                "Test command".to_string(),
                "echo $ARGUMENTS".to_string(),
            );

            let context = ExecutionContext::new(
                cmd,
                vec!["hello world".to_string(), "foo$bar".to_string()],
                PathBuf::from("."),
            );

            let result = execute(&context);
            assert!(result.is_ok());

            let exec_result = result.unwrap();
            assert!(exec_result.is_success());
            // Arguments should be escaped
            assert!(exec_result.output.contains("'hello world'"));
        }
    }
}

/// Command discovery and help system
pub mod discovery {
    use super::*;

    /// Information about a command for display purposes
    #[derive(Debug, Clone)]
    pub struct CommandInfo {
        /// Command name
        pub name: String,
        /// Command description
        pub description: String,
        /// Command template
        pub template: String,
        /// Maximum arguments required
        pub max_args: usize,
        /// Whether command uses $ARGUMENTS
        pub uses_all_args: bool,
    }

    impl CommandInfo {
        /// Create command info from a command definition
        pub fn from_def(cmd: &CommandDef) -> Self {
            let max_args = template::max_argument_index(&cmd.template);
            let uses_all_args = template::uses_all_arguments(&cmd.template);

            Self {
                name: cmd.name.clone(),
                description: cmd.description.clone(),
                template: cmd.template.clone(),
                max_args,
                uses_all_args,
            }
        }

        /// Get a formatted help string for this command
        pub fn help_text(&self) -> String {
            let mut help = format!("Command: {}\n", self.name);
            help.push_str(&format!("Description: {}\n", self.description));
            help.push_str(&format!("Template: {}\n", self.template));

            if self.uses_all_args {
                help.push_str("Arguments: Accepts any number of arguments (via $ARGUMENTS)\n");
            } else if self.max_args > 0 {
                help.push_str(&format!("Arguments: Requires up to {} arguments\n", self.max_args));
            } else {
                help.push_str("Arguments: No arguments required\n");
            }

            help
        }

        /// Get a one-line summary of the command
        pub fn summary(&self) -> String {
            format!("{:<20} {}", self.name, self.description)
        }
    }

    /// List all available commands with their descriptions
    pub fn list_commands(registry: &CommandRegistry) -> Vec<CommandInfo> {
        registry
            .list_all()
            .iter()
            .map(CommandInfo::from_def)
            .collect()
    }

    /// Get detailed help for a specific command
    pub fn get_command_help(registry: &CommandRegistry, name: &str) -> Option<String> {
        registry.get(name).map(|cmd| {
            let info = CommandInfo::from_def(&cmd);
            info.help_text()
        })
    }

    /// Format a list of commands for display
    pub fn format_command_list(commands: &[CommandInfo]) -> String {
        if commands.is_empty() {
            return "No commands available".to_string();
        }

        let mut output = String::from("Available Commands:\n");
        output.push_str("===================\n\n");

        for cmd in commands {
            output.push_str(&cmd.summary());
            output.push('\n');
        }

        output
    }

    /// Search for commands by name or description
    pub fn search_commands(registry: &CommandRegistry, query: &str) -> Vec<CommandInfo> {
        let query_lower = query.to_lowercase();

        registry
            .list_all()
            .iter()
            .filter(|cmd| {
                cmd.name.to_lowercase().contains(&query_lower)
                    || cmd.description.to_lowercase().contains(&query_lower)
            })
            .map(CommandInfo::from_def)
            .collect()
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_command_info_from_def() {
            let cmd = CommandDef::new(
                "test".to_string(),
                "Test command".to_string(),
                "echo $1 and $2".to_string(),
            );

            let info = CommandInfo::from_def(&cmd);
            assert_eq!(info.name, "test");
            assert_eq!(info.description, "Test command");
            assert_eq!(info.max_args, 2);
            assert!(!info.uses_all_args);
        }

        #[test]
        fn test_command_info_with_all_args() {
            let cmd = CommandDef::new(
                "test".to_string(),
                "Test command".to_string(),
                "echo $ARGUMENTS".to_string(),
            );

            let info = CommandInfo::from_def(&cmd);
            assert!(info.uses_all_args);
        }

        #[test]
        fn test_command_info_help_text() {
            let cmd = CommandDef::new(
                "test".to_string(),
                "Test command".to_string(),
                "echo $1 and $2".to_string(),
            );

            let info = CommandInfo::from_def(&cmd);
            let help = info.help_text();

            assert!(help.contains("Command: test"));
            assert!(help.contains("Description: Test command"));
            assert!(help.contains("Template: echo $1 and $2"));
            assert!(help.contains("Arguments: Requires up to 2 arguments"));
        }

        #[test]
        fn test_command_info_summary() {
            let cmd = CommandDef::new(
                "test".to_string(),
                "Test command".to_string(),
                "echo $ARGUMENTS".to_string(),
            );

            let info = CommandInfo::from_def(&cmd);
            let summary = info.summary();

            assert!(summary.contains("test"));
            assert!(summary.contains("Test command"));
        }

        #[test]
        fn test_list_commands() {
            let mut registry = CommandRegistry::new();

            let cmd1 = CommandDef::new(
                "cmd1".to_string(),
                "Command 1".to_string(),
                "echo 1".to_string(),
            );

            let cmd2 = CommandDef::new(
                "cmd2".to_string(),
                "Command 2".to_string(),
                "echo 2".to_string(),
            );

            registry.register(cmd1).unwrap();
            registry.register(cmd2).unwrap();

            let commands = list_commands(&registry);
            assert_eq!(commands.len(), 2);
        }

        #[test]
        fn test_get_command_help() {
            let mut registry = CommandRegistry::new();

            let cmd = CommandDef::new(
                "test".to_string(),
                "Test command".to_string(),
                "echo $ARGUMENTS".to_string(),
            );

            registry.register(cmd).unwrap();

            let help = get_command_help(&registry, "test");
            assert!(help.is_some());

            let help_text = help.unwrap();
            assert!(help_text.contains("Command: test"));
        }

        #[test]
        fn test_get_command_help_not_found() {
            let registry = CommandRegistry::new();
            let help = get_command_help(&registry, "nonexistent");
            assert!(help.is_none());
        }

        #[test]
        fn test_format_command_list() {
            let mut registry = CommandRegistry::new();

            let cmd1 = CommandDef::new(
                "cmd1".to_string(),
                "Command 1".to_string(),
                "echo 1".to_string(),
            );

            let cmd2 = CommandDef::new(
                "cmd2".to_string(),
                "Command 2".to_string(),
                "echo 2".to_string(),
            );

            registry.register(cmd1).unwrap();
            registry.register(cmd2).unwrap();

            let commands = list_commands(&registry);
            let formatted = format_command_list(&commands);

            assert!(formatted.contains("Available Commands"));
            assert!(formatted.contains("cmd1"));
            assert!(formatted.contains("cmd2"));
        }

        #[test]
        fn test_format_command_list_empty() {
            let commands = vec![];
            let formatted = format_command_list(&commands);
            assert_eq!(formatted, "No commands available");
        }

        #[test]
        fn test_search_commands_by_name() {
            let mut registry = CommandRegistry::new();

            let cmd1 = CommandDef::new(
                "test-cmd".to_string(),
                "Test command".to_string(),
                "echo 1".to_string(),
            );

            let cmd2 = CommandDef::new(
                "other-cmd".to_string(),
                "Other command".to_string(),
                "echo 2".to_string(),
            );

            registry.register(cmd1).unwrap();
            registry.register(cmd2).unwrap();

            let results = search_commands(&registry, "test");
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].name, "test-cmd");
        }

        #[test]
        fn test_search_commands_by_description() {
            let mut registry = CommandRegistry::new();

            let cmd1 = CommandDef::new(
                "cmd1".to_string(),
                "Test command".to_string(),
                "echo 1".to_string(),
            );

            let cmd2 = CommandDef::new(
                "cmd2".to_string(),
                "Other command".to_string(),
                "echo 2".to_string(),
            );

            registry.register(cmd1).unwrap();
            registry.register(cmd2).unwrap();

            let results = search_commands(&registry, "test");
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].name, "cmd1");
        }

        #[test]
        fn test_search_commands_case_insensitive() {
            let mut registry = CommandRegistry::new();

            let cmd = CommandDef::new(
                "TestCmd".to_string(),
                "Test command".to_string(),
                "echo 1".to_string(),
            );

            registry.register(cmd).unwrap();

            let results = search_commands(&registry, "testcmd");
            assert_eq!(results.len(), 1);
        }

        #[test]
        fn test_search_commands_no_results() {
            let registry = CommandRegistry::new();
            let results = search_commands(&registry, "nonexistent");
            assert_eq!(results.len(), 0);
        }
    }
}

/// Registry for managing custom command definitions
#[derive(Debug, Clone)]
pub struct CommandRegistry {
    /// Map of command name to command definition
    commands: std::collections::HashMap<String, CommandDef>,
}

impl CommandRegistry {
    /// Create a new empty command registry
    pub fn new() -> Self {
        Self {
            commands: std::collections::HashMap::new(),
        }
    }

    /// Register a command definition
    pub fn register(&mut self, command: CommandDef) -> CliResult<()> {
        // Validate the command before registering
        command.validate()?;

        // Check if command already exists
        if self.commands.contains_key(&command.name) {
            return Err(CliError::InvalidArgument {
                message: format!("Command '{}' is already registered", command.name),
            });
        }

        self.commands.insert(command.name.clone(), command);
        Ok(())
    }

    /// Unregister a command by name
    pub fn unregister(&mut self, name: &str) -> CliResult<()> {
        if self.commands.remove(name).is_none() {
            return Err(CliError::InvalidArgument {
                message: format!("Command '{}' not found", name),
            });
        }
        Ok(())
    }

    /// Update an existing command definition
    pub fn update(&mut self, command: CommandDef) -> CliResult<()> {
        // Validate the command before updating
        command.validate()?;

        // Check if command exists
        if !self.commands.contains_key(&command.name) {
            return Err(CliError::InvalidArgument {
                message: format!("Command '{}' not found", command.name),
            });
        }

        self.commands.insert(command.name.clone(), command);
        Ok(())
    }

    /// Look up a command by name
    pub fn get(&self, name: &str) -> Option<CommandDef> {
        self.commands.get(name).cloned()
    }

    /// Check if a command exists
    pub fn contains(&self, name: &str) -> bool {
        self.commands.contains_key(name)
    }

    /// Get all registered commands
    pub fn list_all(&self) -> Vec<CommandDef> {
        self.commands.values().cloned().collect()
    }

    /// Get all command names
    pub fn list_names(&self) -> Vec<String> {
        self.commands.keys().cloned().collect()
    }

    /// Get the number of registered commands
    pub fn count(&self) -> usize {
        self.commands.len()
    }

    /// Clear all commands from the registry
    pub fn clear(&mut self) {
        self.commands.clear();
    }

    /// Get commands filtered by a predicate
    pub fn filter<F>(&self, predicate: F) -> Vec<CommandDef>
    where
        F: Fn(&CommandDef) -> bool,
    {
        self.commands
            .values()
            .filter(|cmd| predicate(cmd))
            .cloned()
            .collect()
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Template engine for argument substitution
pub mod template {
    use regex::Regex;

    /// Represents an error during template substitution
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum SubstitutionError {
        /// Missing required argument for a placeholder
        MissingArgument { placeholder: String, index: usize },
        /// Invalid placeholder syntax
        InvalidPlaceholder { placeholder: String },
        /// Unsubstituted placeholder found in final output
        UnsubstitutedPlaceholder { placeholder: String },
    }

    impl std::fmt::Display for SubstitutionError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                SubstitutionError::MissingArgument { placeholder, index } => {
                    write!(f, "Missing argument for placeholder '{}' (index {})", placeholder, index)
                }
                SubstitutionError::InvalidPlaceholder { placeholder } => {
                    write!(f, "Invalid placeholder syntax: '{}'", placeholder)
                }
                SubstitutionError::UnsubstitutedPlaceholder { placeholder } => {
                    write!(f, "Unsubstituted placeholder found: '{}'", placeholder)
                }
            }
        }
    }

    /// Escape special characters in a string for shell safety
    pub fn escape_shell_arg(arg: &str) -> String {
        // If the argument contains special characters, wrap it in single quotes
        // and escape any single quotes within it
        if arg.contains(|c: char| matches!(c, ' ' | '\t' | '\n' | '"' | '\'' | '\\' | '$' | '`' | '!' | '*' | '?' | '[' | ']' | '(' | ')' | '{' | '}' | ';' | '&' | '|' | '<' | '>' | '#')) {
            // Use single quotes and escape any single quotes in the string
            format!("'{}'", arg.replace('\'', "'\\''"))
        } else {
            arg.to_string()
        }
    }

    /// Substitute placeholders in a template with provided arguments
    ///
    /// Supports:
    /// - `$ARGUMENTS`: replaced with all arguments joined by spaces
    /// - `$1`, `$2`, etc.: replaced with specific positional arguments
    ///
    /// # Arguments
    /// - `template`: The template string with placeholders
    /// - `arguments`: The arguments to substitute
    /// - `escape`: Whether to escape arguments for shell safety
    ///
    /// # Returns
    /// - `Ok(String)`: The substituted template
    /// - `Err(SubstitutionError)`: If substitution fails
    pub fn substitute(
        template: &str,
        arguments: &[String],
        escape: bool,
    ) -> Result<String, SubstitutionError> {
        let mut result = template.to_string();

        // First, substitute $ARGUMENTS with all arguments joined
        if result.contains("$ARGUMENTS") {
            let args_str = if escape {
                arguments
                    .iter()
                    .map(|arg| escape_shell_arg(arg))
                    .collect::<Vec<_>>()
                    .join(" ")
            } else {
                arguments.join(" ")
            };
            result = result.replace("$ARGUMENTS", &args_str);
        }

        // Then, substitute positional placeholders ($1, $2, etc.)
        // Use a regex to find all positional placeholders
        let regex = Regex::new(r"\$(\d+)").expect("Invalid regex");

        for cap in regex.captures_iter(template) {
            let full_match = cap.get(0).unwrap().as_str();
            let index_str = cap.get(1).unwrap().as_str();

            let index: usize = index_str
                .parse()
                .map_err(|_| SubstitutionError::InvalidPlaceholder {
                    placeholder: full_match.to_string(),
                })?;

            // Check if the index is valid (1-based indexing)
            if index == 0 {
                return Err(SubstitutionError::InvalidPlaceholder {
                    placeholder: full_match.to_string(),
                });
            }

            // Get the argument (convert from 1-based to 0-based indexing)
            let arg_index = index - 1;
            if arg_index >= arguments.len() {
                return Err(SubstitutionError::MissingArgument {
                    placeholder: full_match.to_string(),
                    index,
                });
            }

            let arg_value = if escape {
                escape_shell_arg(&arguments[arg_index])
            } else {
                arguments[arg_index].clone()
            };

            result = result.replace(full_match, &arg_value);
        }

        // Verify that all placeholders have been substituted
        // Check for any remaining $-prefixed placeholders
        let remaining_regex = Regex::new(r"\$[A-Za-z_]\w*|\$\d+").expect("Invalid regex");
        if let Some(remaining) = remaining_regex.find(&result) {
            return Err(SubstitutionError::UnsubstitutedPlaceholder {
                placeholder: remaining.as_str().to_string(),
            });
        }

        Ok(result)
    }

    /// Validate that a template can be substituted with the given arguments
    ///
    /// This checks:
    /// - All positional placeholders have corresponding arguments
    /// - No invalid placeholder syntax
    pub fn validate(template: &str, arguments: &[String]) -> Result<(), SubstitutionError> {
        // Try to substitute with a dry run
        substitute(template, arguments, false)?;
        Ok(())
    }

    /// Get the maximum argument index required by a template
    ///
    /// Returns the highest index used in positional placeholders (e.g., $5 returns 5)
    /// Returns 0 if no positional placeholders are found
    pub fn max_argument_index(template: &str) -> usize {
        let regex = Regex::new(r"\$(\d+)").expect("Invalid regex");
        regex
            .captures_iter(template)
            .filter_map(|cap| {
                cap.get(1)
                    .and_then(|m| m.as_str().parse::<usize>().ok())
            })
            .max()
            .unwrap_or(0)
    }

    /// Check if a template uses $ARGUMENTS placeholder
    pub fn uses_all_arguments(template: &str) -> bool {
        template.contains("$ARGUMENTS")
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_escape_shell_arg_simple() {
            let arg = "hello";
            assert_eq!(escape_shell_arg(arg), "hello");
        }

        #[test]
        fn test_escape_shell_arg_with_spaces() {
            let arg = "hello world";
            assert_eq!(escape_shell_arg(arg), "'hello world'");
        }

        #[test]
        fn test_escape_shell_arg_with_single_quote() {
            let arg = "it's";
            assert_eq!(escape_shell_arg(arg), "'it'\\''s'");
        }

        #[test]
        fn test_escape_shell_arg_with_dollar() {
            let arg = "$HOME";
            assert_eq!(escape_shell_arg(arg), "'$HOME'");
        }

        #[test]
        fn test_substitute_all_arguments() {
            let template = "echo $ARGUMENTS";
            let args = vec!["hello".to_string(), "world".to_string()];
            let result = substitute(template, &args, false).unwrap();
            assert_eq!(result, "echo hello world");
        }

        #[test]
        fn test_substitute_all_arguments_escaped() {
            let template = "echo $ARGUMENTS";
            let args = vec!["hello world".to_string(), "foo".to_string()];
            let result = substitute(template, &args, true).unwrap();
            assert_eq!(result, "echo 'hello world' foo");
        }

        #[test]
        fn test_substitute_positional() {
            let template = "echo $1 and $2";
            let args = vec!["hello".to_string(), "world".to_string()];
            let result = substitute(template, &args, false).unwrap();
            assert_eq!(result, "echo hello and world");
        }

        #[test]
        fn test_substitute_positional_escaped() {
            let template = "echo $1 and $2";
            let args = vec!["hello world".to_string(), "foo bar".to_string()];
            let result = substitute(template, &args, true).unwrap();
            assert_eq!(result, "echo 'hello world' and 'foo bar'");
        }

        #[test]
        fn test_substitute_mixed() {
            let template = "echo $1 and $ARGUMENTS";
            let args = vec!["hello".to_string(), "world".to_string()];
            let result = substitute(template, &args, false).unwrap();
            assert_eq!(result, "echo hello and hello world");
        }

        #[test]
        fn test_substitute_missing_argument() {
            let template = "echo $1 and $2";
            let args = vec!["hello".to_string()];
            let result = substitute(template, &args, false);
            assert!(result.is_err());
            match result.unwrap_err() {
                SubstitutionError::MissingArgument { index, .. } => assert_eq!(index, 2),
                _ => panic!("Expected MissingArgument error"),
            }
        }

        #[test]
        fn test_substitute_invalid_placeholder_zero() {
            let template = "echo $0";
            let args = vec!["hello".to_string()];
            let result = substitute(template, &args, false);
            assert!(result.is_err());
        }

        #[test]
        fn test_substitute_no_arguments() {
            let template = "echo hello";
            let args = vec![];
            let result = substitute(template, &args, false).unwrap();
            assert_eq!(result, "echo hello");
        }

        #[test]
        fn test_substitute_empty_arguments() {
            let template = "echo $ARGUMENTS";
            let args = vec![];
            let result = substitute(template, &args, false).unwrap();
            assert_eq!(result, "echo ");
        }

        #[test]
        fn test_validate_success() {
            let template = "echo $1 and $2";
            let args = vec!["hello".to_string(), "world".to_string()];
            assert!(validate(template, &args).is_ok());
        }

        #[test]
        fn test_validate_missing_argument() {
            let template = "echo $1 and $2";
            let args = vec!["hello".to_string()];
            assert!(validate(template, &args).is_err());
        }

        #[test]
        fn test_max_argument_index() {
            let template = "echo $1 and $3 and $2";
            assert_eq!(max_argument_index(template), 3);
        }

        #[test]
        fn test_max_argument_index_no_positional() {
            let template = "echo $ARGUMENTS";
            assert_eq!(max_argument_index(template), 0);
        }

        #[test]
        fn test_uses_all_arguments() {
            let template = "echo $ARGUMENTS";
            assert!(uses_all_arguments(template));
        }

        #[test]
        fn test_uses_all_arguments_false() {
            let template = "echo $1 and $2";
            assert!(!uses_all_arguments(template));
        }
    }
}

/// Shell command executor with timeout protection and output capture
pub mod shell {
    use std::process::{Command, Stdio};
    use std::time::Instant;
    use std::io::Read;

    /// Represents an error during shell command execution
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum ShellError {
        /// Command execution failed with exit code
        CommandFailed { exit_code: i32, stderr: String },
        /// Command timed out
        Timeout,
        /// Failed to execute command
        ExecutionFailed(String),
        /// Failed to read output
        OutputReadFailed(String),
    }

    impl std::fmt::Display for ShellError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                ShellError::CommandFailed { exit_code, stderr } => {
                    write!(f, "Command failed with exit code {}: {}", exit_code, stderr)
                }
                ShellError::Timeout => write!(f, "Command execution timed out"),
                ShellError::ExecutionFailed(msg) => write!(f, "Failed to execute command: {}", msg),
                ShellError::OutputReadFailed(msg) => write!(f, "Failed to read output: {}", msg),
            }
        }
    }

    /// Result of shell command execution
    #[derive(Debug, Clone)]
    pub struct ShellResult {
        /// Standard output from the command
        pub stdout: String,
        /// Standard error from the command
        pub stderr: String,
        /// Exit code of the command
        pub exit_code: i32,
        /// Whether the command succeeded (exit code 0)
        pub success: bool,
        /// Duration of execution in milliseconds
        pub duration_ms: u64,
    }

    impl ShellResult {
        /// Get the combined output (stdout + stderr)
        pub fn combined_output(&self) -> String {
            let mut output = self.stdout.clone();
            if !self.stderr.is_empty() {
                if !output.is_empty() {
                    output.push('\n');
                }
                output.push_str(&self.stderr);
            }
            output
        }
    }

    /// Execute a shell command with timeout protection
    ///
    /// # Arguments
    /// - `command`: The command to execute (e.g., "echo", "ls")
    /// - `args`: Arguments to pass to the command
    /// - `timeout_secs`: Timeout in seconds (0 = no timeout)
    ///
    /// # Returns
    /// - `Ok(ShellResult)`: The result of command execution
    /// - `Err(ShellError)`: If execution fails
    pub fn execute(
        command: &str,
        args: &[&str],
        timeout_secs: u64,
    ) -> Result<ShellResult, ShellError> {
        let start = Instant::now();

        // Build the command
        let mut cmd = Command::new(command);
        cmd.args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Spawn the process
        let mut child = cmd.spawn()
            .map_err(|e| ShellError::ExecutionFailed(e.to_string()))?;

        // Get stdout and stderr handles
        let stdout_handle = child.stdout.take()
            .ok_or_else(|| ShellError::OutputReadFailed("Failed to get stdout".to_string()))?;

        let stderr_handle = child.stderr.take()
            .ok_or_else(|| ShellError::OutputReadFailed("Failed to get stderr".to_string()))?;

        // Read stdout in a separate thread to avoid deadlock
        let stdout_thread = std::thread::spawn(move || {
            let mut output = String::new();
            let mut reader = std::io::BufReader::new(stdout_handle);
            let _ = reader.read_to_string(&mut output);
            output
        });

        // Read stderr in a separate thread to avoid deadlock
        let stderr_thread = std::thread::spawn(move || {
            let mut output = String::new();
            let mut reader = std::io::BufReader::new(stderr_handle);
            let _ = reader.read_to_string(&mut output);
            output
        });

        // Wait for the process to complete with timeout
        let result = if timeout_secs > 0 {
            let timeout_duration = std::time::Duration::from_secs(timeout_secs);
            match std::thread::spawn(move || child.wait()).join() {
                Ok(Ok(status)) => {
                    let elapsed = start.elapsed();
                    if elapsed > timeout_duration {
                        return Err(ShellError::Timeout);
                    }
                    Ok(status)
                }
                Ok(Err(e)) => Err(ShellError::ExecutionFailed(e.to_string())),
                Err(_) => Err(ShellError::ExecutionFailed("Thread panicked".to_string())),
            }
        } else {
            child.wait()
                .map_err(|e| ShellError::ExecutionFailed(e.to_string()))
        };

        // Get the exit status
        let status = result?;
        let exit_code = status.code().unwrap_or(-1);

        // Get the output from threads
        let stdout = stdout_thread.join().unwrap_or_default();
        let stderr = stderr_thread.join().unwrap_or_default();

        let duration_ms = start.elapsed().as_millis() as u64;

        // Check for timeout
        if timeout_secs > 0 && duration_ms > (timeout_secs * 1000) {
            return Err(ShellError::Timeout);
        }

        // If command failed, return error
        if !status.success() {
            return Err(ShellError::CommandFailed {
                exit_code,
                stderr: stderr.clone(),
            });
        }

        Ok(ShellResult {
            stdout,
            stderr,
            exit_code,
            success: status.success(),
            duration_ms,
        })
    }

    /// Execute a shell command without timeout
    pub fn execute_no_timeout(command: &str, args: &[&str]) -> Result<ShellResult, ShellError> {
        execute(command, args, 0)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_execute_simple_command() {
            #[cfg(target_os = "windows")]
            let result = execute("cmd", &["/c", "echo hello"], 5);

            #[cfg(not(target_os = "windows"))]
            let result = execute("echo", &["hello"], 5);

            assert!(result.is_ok());
            let shell_result = result.unwrap();
            assert!(shell_result.success);
            assert!(shell_result.stdout.contains("hello"));
        }

        #[test]
        fn test_execute_command_with_args() {
            #[cfg(target_os = "windows")]
            let result = execute("cmd", &["/c", "echo hello world"], 5);

            #[cfg(not(target_os = "windows"))]
            let result = execute("echo", &["hello", "world"], 5);

            assert!(result.is_ok());
            let shell_result = result.unwrap();
            assert!(shell_result.success);
            assert!(shell_result.stdout.contains("hello"));
            assert!(shell_result.stdout.contains("world"));
        }

        #[test]
        fn test_execute_failing_command() {
            #[cfg(target_os = "windows")]
            let result = execute("cmd", &["/c", "exit 1"], 5);

            #[cfg(not(target_os = "windows"))]
            let result = execute("sh", &["-c", "exit 1"], 5);

            assert!(result.is_err());
            match result.unwrap_err() {
                ShellError::CommandFailed { exit_code, .. } => {
                    assert_eq!(exit_code, 1);
                }
                _ => panic!("Expected CommandFailed error"),
            }
        }

        #[test]
        fn test_execute_nonexistent_command() {
            let result = execute("nonexistent_command_xyz", &[], 5);
            assert!(result.is_err());
        }

        #[test]
        fn test_shell_result_combined_output() {
            let result = ShellResult {
                stdout: "output".to_string(),
                stderr: "error".to_string(),
                exit_code: 0,
                success: true,
                duration_ms: 100,
            };

            let combined = result.combined_output();
            assert!(combined.contains("output"));
            assert!(combined.contains("error"));
        }

        #[test]
        fn test_shell_result_combined_output_no_stderr() {
            let result = ShellResult {
                stdout: "output".to_string(),
                stderr: String::new(),
                exit_code: 0,
                success: true,
                duration_ms: 100,
            };

            let combined = result.combined_output();
            assert_eq!(combined, "output");
        }
    }
}

/// Output truncation utilities for large shell outputs and file content
pub mod truncation {
    /// Default maximum output size in bytes (10KB)
    pub const DEFAULT_MAX_OUTPUT_SIZE: usize = 10 * 1024;

    /// Default maximum file size in bytes (50KB)
    pub const DEFAULT_MAX_FILE_SIZE: usize = 50 * 1024;

    /// Truncation indicator suffix
    const TRUNCATION_INDICATOR: &str = "\n... (output truncated)";

    /// Truncate output if it exceeds the maximum size
    ///
    /// # Arguments
    /// - `output`: The output to truncate
    /// - `max_size`: Maximum size in bytes
    ///
    /// # Returns
    /// A tuple of (truncated_output, was_truncated)
    pub fn truncate_output(output: &str, max_size: usize) -> (String, bool) {
        if output.len() <= max_size {
            return (output.to_string(), false);
        }

        // Truncate to max_size, accounting for the truncation indicator
        let indicator_len = TRUNCATION_INDICATOR.len();
        let target_size = if max_size > indicator_len {
            max_size - indicator_len
        } else {
            max_size
        };

        // Find a safe truncation point that doesn't split UTF-8 characters
        let mut truncate_at = target_size.min(output.len());

        // Back up to the nearest character boundary
        while truncate_at > 0 && !output.is_char_boundary(truncate_at) {
            truncate_at -= 1;
        }

        // If we couldn't find a valid boundary, just use the whole string
        if truncate_at == 0 {
            truncate_at = output.len();
        }

        let mut truncated = output[..truncate_at].to_string();
        truncated.push_str(TRUNCATION_INDICATOR);
        (truncated, true)
    }

    /// Check if output was truncated (contains truncation indicator)
    pub fn is_truncated(output: &str) -> bool {
        output.ends_with(TRUNCATION_INDICATOR)
    }

    /// Get the original size before truncation (if available)
    /// This is a heuristic - we can't know the exact original size from truncated output
    pub fn estimate_original_size(output: &str) -> Option<usize> {
        if is_truncated(output) {
            // We know it was at least the current size + the truncation indicator
            Some(output.len())
        } else {
            None
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_truncate_output_no_truncation_needed() {
            let output = "hello world";
            let (truncated, was_truncated) = truncate_output(output, 100);
            assert_eq!(truncated, "hello world");
            assert!(!was_truncated);
        }

        #[test]
        fn test_truncate_output_truncation_needed() {
            let output = "hello world this is a long output";
            let (truncated, was_truncated) = truncate_output(output, 10);
            assert!(was_truncated);
            assert!(truncated.ends_with(TRUNCATION_INDICATOR));
            assert!(truncated.len() <= 10 + TRUNCATION_INDICATOR.len() + 10); // Some buffer for UTF-8
        }

        #[test]
        fn test_truncate_output_exact_size() {
            let output = "hello";
            let (truncated, was_truncated) = truncate_output(output, 5);
            assert!(!was_truncated);
            assert_eq!(truncated, "hello");
        }

        #[test]
        fn test_truncate_output_one_byte_over() {
            let output = "hello";
            let (truncated, was_truncated) = truncate_output(output, 4);
            assert!(was_truncated);
            assert!(truncated.ends_with(TRUNCATION_INDICATOR));
        }

        #[test]
        fn test_truncate_output_utf8_boundary() {
            let output = "hello 世界"; // Contains multi-byte UTF-8 characters
            let (truncated, was_truncated) = truncate_output(output, 8);
            assert!(was_truncated);
            // Should not panic and should be valid UTF-8
            assert!(truncated.is_char_boundary(truncated.len()));
        }

        #[test]
        fn test_is_truncated() {
            let truncated_output = "hello\n... (output truncated)";
            assert!(is_truncated(truncated_output));

            let normal_output = "hello";
            assert!(!is_truncated(normal_output));
        }

        #[test]
        fn test_estimate_original_size() {
            let truncated_output = "hello\n... (output truncated)";
            let size = estimate_original_size(truncated_output);
            assert!(size.is_some());

            let normal_output = "hello";
            let size = estimate_original_size(normal_output);
            assert!(size.is_none());
        }

        #[test]
        fn test_truncate_large_output() {
            let large_output = "x".repeat(100_000);
            let (truncated, was_truncated) = truncate_output(&large_output, DEFAULT_MAX_OUTPUT_SIZE);
            assert!(was_truncated);
            assert!(truncated.len() <= DEFAULT_MAX_OUTPUT_SIZE + TRUNCATION_INDICATOR.len() + 10);
        }
    }
}

/// File reference handler for including file content in templates
pub mod file_reference {
    use std::path::Path;
    use regex::Regex;

    /// Represents an error during file reference processing
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum FileReferenceError {
        /// File not found at the specified path
        FileNotFound { path: String },
        /// Failed to read file
        ReadFailed { path: String, reason: String },
        /// Path traversal attempt detected (security violation)
        PathTraversal { path: String },
        /// Invalid file reference syntax
        InvalidReference { reference: String },
    }

    impl std::fmt::Display for FileReferenceError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                FileReferenceError::FileNotFound { path } => {
                    write!(f, "File not found: {}", path)
                }
                FileReferenceError::ReadFailed { path, reason } => {
                    write!(f, "Failed to read file {}: {}", path, reason)
                }
                FileReferenceError::PathTraversal { path } => {
                    write!(f, "Path traversal attempt detected: {}", path)
                }
                FileReferenceError::InvalidReference { reference } => {
                    write!(f, "Invalid file reference syntax: {}", reference)
                }
            }
        }
    }

    /// Result of file reading
    #[derive(Debug, Clone)]
    pub struct FileContent {
        /// Path to the file
        pub path: String,
        /// Content of the file
        pub content: String,
        /// Whether the content was truncated
        pub truncated: bool,
        /// Original size in bytes (if truncated)
        pub original_size: Option<usize>,
    }

    /// Read a file from the project directory with security checks
    ///
    /// # Arguments
    /// - `file_path`: Path to the file (relative to project directory)
    /// - `project_dir`: Project directory (base path for security)
    /// - `max_size`: Maximum file size in bytes (0 = no limit)
    ///
    /// # Returns
    /// - `Ok(FileContent)`: The file content
    /// - `Err(FileReferenceError)`: If reading fails
    pub fn read_file(
        file_path: &str,
        project_dir: &Path,
        max_size: usize,
    ) -> Result<FileContent, FileReferenceError> {
        // Validate file path (no path traversal)
        validate_path(file_path)?;

        // Construct the full path
        let full_path = project_dir.join(file_path);

        // Ensure the resolved path is within the project directory
        let canonical_project = project_dir.canonicalize()
            .map_err(|e| FileReferenceError::ReadFailed {
                path: file_path.to_string(),
                reason: format!("Failed to canonicalize project directory: {}", e),
            })?;

        let canonical_file = full_path.canonicalize()
            .map_err(|_| FileReferenceError::FileNotFound {
                path: file_path.to_string(),
            })?;

        // Check if the file is within the project directory
        if !canonical_file.starts_with(&canonical_project) {
            return Err(FileReferenceError::PathTraversal {
                path: file_path.to_string(),
            });
        }

        // Read the file
        let content = std::fs::read_to_string(&canonical_file)
            .map_err(|e| FileReferenceError::ReadFailed {
                path: file_path.to_string(),
                reason: e.to_string(),
            })?;

        // Check if truncation is needed
        let (truncated_content, was_truncated, original_size) = if max_size > 0 && content.len() > max_size {
            let (truncated, _) = super::truncation::truncate_output(&content, max_size);
            (truncated, true, Some(content.len()))
        } else {
            (content, false, None)
        };

        Ok(FileContent {
            path: file_path.to_string(),
            content: truncated_content,
            truncated: was_truncated,
            original_size,
        })
    }

    /// Validate a file path for security (no path traversal)
    fn validate_path(file_path: &str) -> Result<(), FileReferenceError> {
        // Check for absolute paths
        if Path::new(file_path).is_absolute() {
            return Err(FileReferenceError::PathTraversal {
                path: file_path.to_string(),
            });
        }

        // Check for path traversal attempts
        if file_path.contains("..") {
            return Err(FileReferenceError::PathTraversal {
                path: file_path.to_string(),
            });
        }

        // Check for null bytes
        if file_path.contains('\0') {
            return Err(FileReferenceError::InvalidReference {
                reference: file_path.to_string(),
            });
        }

        Ok(())
    }

    /// Extract all file references from a template
    ///
    /// File references are in the format: @filename
    /// Returns a vector of file paths in the order they appear
    pub fn extract_references(template: &str) -> Vec<String> {
        let regex = Regex::new(r"@([a-zA-Z0-9_\-./]+)").expect("Invalid regex");
        regex
            .captures_iter(template)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .collect()
    }

    /// Substitute file references in a template
    ///
    /// # Arguments
    /// - `template`: The template string with file references
    /// - `project_dir`: Project directory (base path for security)
    /// - `max_file_size`: Maximum file size in bytes
    ///
    /// # Returns
    /// - `Ok(String)`: The template with file references substituted
    /// - `Err(FileReferenceError)`: If any file reference fails
    pub fn substitute(
        template: &str,
        project_dir: &Path,
        max_file_size: usize,
    ) -> Result<String, FileReferenceError> {
        let mut result = template.to_string();

        // Find all file references
        let regex = Regex::new(r"@([a-zA-Z0-9_\-./]+)").expect("Invalid regex");

        // Collect all matches first to avoid borrowing issues
        let matches: Vec<_> = regex
            .captures_iter(template)
            .map(|cap| {
                let full_match = cap.get(0).unwrap().as_str().to_string();
                let file_path = cap.get(1).unwrap().as_str().to_string();
                (full_match, file_path)
            })
            .collect();

        // Process each file reference
        for (full_match, file_path) in matches {
            let file_content = read_file(&file_path, project_dir, max_file_size)?;
            result = result.replace(&full_match, &file_content.content);
        }

        Ok(result)
    }

    /// Check if a template contains any file references
    pub fn has_references(template: &str) -> bool {
        template.contains('@')
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::fs;
        use tempfile::TempDir;

        #[test]
        fn test_validate_path_simple() {
            assert!(validate_path("file.txt").is_ok());
            assert!(validate_path("dir/file.txt").is_ok());
            assert!(validate_path("dir/subdir/file.txt").is_ok());
        }

        #[test]
        fn test_validate_path_absolute() {
            #[cfg(target_os = "windows")]
            assert!(validate_path("C:\\Windows\\System32").is_err());
            #[cfg(not(target_os = "windows"))]
            assert!(validate_path("/etc/passwd").is_err());
        }

        #[test]
        fn test_validate_path_traversal() {
            assert!(validate_path("../etc/passwd").is_err());
            assert!(validate_path("dir/../../etc/passwd").is_err());
            assert!(validate_path("..").is_err());
        }

        #[test]
        fn test_validate_path_null_byte() {
            assert!(validate_path("file\0.txt").is_err());
        }

        #[test]
        fn test_extract_references() {
            let template = "Include @file1.txt and @file2.txt";
            let refs = extract_references(template);
            assert_eq!(refs.len(), 2);
            assert_eq!(refs[0], "file1.txt");
            assert_eq!(refs[1], "file2.txt");
        }

        #[test]
        fn test_extract_references_with_paths() {
            let template = "Include @src/main.rs and @docs/README.md";
            let refs = extract_references(template);
            assert_eq!(refs.len(), 2);
            assert_eq!(refs[0], "src/main.rs");
            assert_eq!(refs[1], "docs/README.md");
        }

        #[test]
        fn test_extract_references_no_references() {
            let template = "No file references here";
            let refs = extract_references(template);
            assert_eq!(refs.len(), 0);
        }

        #[test]
        fn test_extract_references_duplicate() {
            let template = "Include @file.txt and @file.txt again";
            let refs = extract_references(template);
            assert_eq!(refs.len(), 2);
            assert_eq!(refs[0], "file.txt");
            assert_eq!(refs[1], "file.txt");
        }

        #[test]
        fn test_read_file_success() {
            let temp_dir = TempDir::new().unwrap();
            let file_path = temp_dir.path().join("test.txt");
            fs::write(&file_path, "Hello, World!").unwrap();

            let result = read_file("test.txt", temp_dir.path(), 0);
            assert!(result.is_ok());

            let content = result.unwrap();
            assert_eq!(content.content, "Hello, World!");
            assert!(!content.truncated);
            assert_eq!(content.original_size, None);
        }

        #[test]
        fn test_read_file_not_found() {
            let temp_dir = TempDir::new().unwrap();
            let result = read_file("nonexistent.txt", temp_dir.path(), 0);
            assert!(result.is_err());
            match result.unwrap_err() {
                FileReferenceError::FileNotFound { .. } => {}
                _ => panic!("Expected FileNotFound error"),
            }
        }

        #[test]
        fn test_read_file_path_traversal() {
            let temp_dir = TempDir::new().unwrap();
            let result = read_file("../etc/passwd", temp_dir.path(), 0);
            assert!(result.is_err());
            match result.unwrap_err() {
                FileReferenceError::PathTraversal { .. } => {}
                _ => panic!("Expected PathTraversal error"),
            }
        }

        #[test]
        fn test_read_file_truncation() {
            let temp_dir = TempDir::new().unwrap();
            let file_path = temp_dir.path().join("large.txt");
            let large_content = "x".repeat(1000);
            fs::write(&file_path, &large_content).unwrap();

            let result = read_file("large.txt", temp_dir.path(), 100);
            assert!(result.is_ok());

            let content = result.unwrap();
            assert!(content.truncated);
            assert!(content.original_size.is_some());
            assert_eq!(content.original_size.unwrap(), 1000);
            assert!(content.content.len() < 1000);
        }

        #[test]
        fn test_read_file_no_truncation_needed() {
            let temp_dir = TempDir::new().unwrap();
            let file_path = temp_dir.path().join("small.txt");
            fs::write(&file_path, "small").unwrap();

            let result = read_file("small.txt", temp_dir.path(), 100);
            assert!(result.is_ok());

            let content = result.unwrap();
            assert!(!content.truncated);
            assert_eq!(content.original_size, None);
            assert_eq!(content.content, "small");
        }

        #[test]
        fn test_read_file_subdirectory() {
            let temp_dir = TempDir::new().unwrap();
            let subdir = temp_dir.path().join("subdir");
            fs::create_dir(&subdir).unwrap();
            let file_path = subdir.join("test.txt");
            fs::write(&file_path, "nested content").unwrap();

            let result = read_file("subdir/test.txt", temp_dir.path(), 0);
            assert!(result.is_ok());

            let content = result.unwrap();
            assert_eq!(content.content, "nested content");
        }

        #[test]
        fn test_substitute_single_file() {
            let temp_dir = TempDir::new().unwrap();
            let file_path = temp_dir.path().join("test.txt");
            fs::write(&file_path, "file content").unwrap();

            let template = "Here is the file: @test.txt";
            let result = substitute(template, temp_dir.path(), 0);
            assert!(result.is_ok());

            let substituted = result.unwrap();
            assert_eq!(substituted, "Here is the file: file content");
        }

        #[test]
        fn test_substitute_multiple_files() {
            let temp_dir = TempDir::new().unwrap();
            let file1 = temp_dir.path().join("file1.txt");
            let file2 = temp_dir.path().join("file2.txt");
            fs::write(&file1, "content1").unwrap();
            fs::write(&file2, "content2").unwrap();

            let template = "First: @file1.txt, Second: @file2.txt";
            let result = substitute(template, temp_dir.path(), 0);
            assert!(result.is_ok());

            let substituted = result.unwrap();
            assert_eq!(substituted, "First: content1, Second: content2");
        }

        #[test]
        fn test_substitute_file_not_found() {
            let temp_dir = TempDir::new().unwrap();
            let template = "Here is the file: @nonexistent.txt";
            let result = substitute(template, temp_dir.path(), 0);
            assert!(result.is_err());
        }

        #[test]
        fn test_substitute_preserves_order() {
            let temp_dir = TempDir::new().unwrap();
            let file1 = temp_dir.path().join("a.txt");
            let file2 = temp_dir.path().join("b.txt");
            let file3 = temp_dir.path().join("c.txt");
            fs::write(&file1, "A").unwrap();
            fs::write(&file2, "B").unwrap();
            fs::write(&file3, "C").unwrap();

            let template = "@c.txt @a.txt @b.txt";
            let result = substitute(template, temp_dir.path(), 0);
            assert!(result.is_ok());

            let substituted = result.unwrap();
            assert_eq!(substituted, "C A B");
        }

        #[test]
        fn test_has_references() {
            assert!(has_references("Include @file.txt"));
            assert!(!has_references("No file references"));
        }
    }
}

/// Parser for command definitions from JSON and Markdown formats
pub mod parser {
    use crate::error::{CliError, CliResult};
    use super::CommandDef;
    use serde::{Deserialize, Serialize};
    use std::fs;

    /// Represents a command definition in JSON format
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CommandDefJson {
        pub name: String,
        pub description: String,
        pub template: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub agent: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub model: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub subtask: Option<bool>,
    }

    impl From<CommandDefJson> for CommandDef {
        fn from(json: CommandDefJson) -> Self {
            CommandDef {
                name: json.name,
                description: json.description,
                template: json.template,
                agent: json.agent,
                model: json.model,
                subtask: json.subtask,
            }
        }
    }

    impl From<CommandDef> for CommandDefJson {
        fn from(cmd: CommandDef) -> Self {
            CommandDefJson {
                name: cmd.name,
                description: cmd.description,
                template: cmd.template,
                agent: cmd.agent,
                model: cmd.model,
                subtask: cmd.subtask,
            }
        }
    }

    /// Parse a command definition from JSON string
    pub fn parse_json(json_str: &str) -> CliResult<CommandDef> {
        let json_def: CommandDefJson = serde_json::from_str(json_str)
            .map_err(|e| CliError::Config(format!("Failed to parse JSON: {}", e)))?;

        let cmd_def = CommandDef::from(json_def);
        cmd_def.validate()?;

        Ok(cmd_def)
    }

    /// Parse a command definition from a JSON file
    pub fn parse_json_file(path: &str) -> CliResult<CommandDef> {
        let content = fs::read_to_string(path)
            .map_err(|e| CliError::Io(e))?;

        parse_json(&content)
    }

    /// Parse a command definition from Markdown code block
    /// Expected format:
    /// ```command
    /// name: my-command
    /// description: My command description
    /// template: echo $ARGUMENTS
    /// agent: optional-agent
    /// model: optional-model
    /// ```
    pub fn parse_markdown(markdown_str: &str) -> CliResult<CommandDef> {
        // Find the command code block
        let start_marker = "```command";
        let end_marker = "```";

        let start_idx = markdown_str.find(start_marker)
            .ok_or_else(|| CliError::Config(
                "No command code block found (expected ```command)".to_string()
            ))?;

        let content_start = start_idx + start_marker.len();
        let remaining = &markdown_str[content_start..];

        let end_idx = remaining.find(end_marker)
            .ok_or_else(|| CliError::Config(
                "Unclosed command code block".to_string()
            ))?;

        let block_content = &remaining[..end_idx].trim();

        // Parse YAML-like format
        let mut name = None;
        let mut description = None;
        let mut template = None;
        let mut agent = None;
        let mut model = None;
        let mut subtask = None;

        for line in block_content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if let Some(colon_idx) = line.find(':') {
                let key = line[..colon_idx].trim();
                let value = line[colon_idx + 1..].trim();

                match key {
                    "name" => name = Some(value.to_string()),
                    "description" => description = Some(value.to_string()),
                    "template" => template = Some(value.to_string()),
                    "agent" => agent = Some(value.to_string()),
                    "model" => model = Some(value.to_string()),
                    "subtask" => subtask = Some(value.parse::<bool>().unwrap_or(false)),
                    _ => {} // Ignore unknown fields
                }
            }
        }

        // Validate required fields
        let name = name.ok_or_else(|| CliError::Config(
            "Missing required field: name".to_string()
        ))?;

        let description = description.ok_or_else(|| CliError::Config(
            "Missing required field: description".to_string()
        ))?;

        let template = template.ok_or_else(|| CliError::Config(
            "Missing required field: template".to_string()
        ))?;

        let cmd_def = CommandDef {
            name,
            description,
            template,
            agent,
            model,
            subtask,
        };

        cmd_def.validate()?;

        Ok(cmd_def)
    }

    /// Parse a command definition from a Markdown file
    pub fn parse_markdown_file(path: &str) -> CliResult<CommandDef> {
        let content = fs::read_to_string(path)
            .map_err(|e| CliError::Io(e))?;

        parse_markdown(&content)
    }

    /// Serialize a command definition to JSON string
    pub fn to_json(cmd: &CommandDef) -> CliResult<String> {
        let json_def = CommandDefJson::from(cmd.clone());
        serde_json::to_string_pretty(&json_def)
            .map_err(|e| CliError::Config(format!("Failed to serialize to JSON: {}", e)))
    }

    /// Serialize a command definition to Markdown format
    pub fn to_markdown(cmd: &CommandDef) -> String {
        let mut markdown = String::from("```command\n");
        markdown.push_str(&format!("name: {}\n", cmd.name));
        markdown.push_str(&format!("description: {}\n", cmd.description));
        markdown.push_str(&format!("template: {}\n", cmd.template));

        if let Some(agent) = &cmd.agent {
            markdown.push_str(&format!("agent: {}\n", agent));
        }

        if let Some(model) = &cmd.model {
            markdown.push_str(&format!("model: {}\n", model));
        }

        if let Some(subtask) = cmd.subtask {
            markdown.push_str(&format!("subtask: {}\n", subtask));
        }

        markdown.push_str("```\n");
        markdown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_def_creation() {
        let cmd = CommandDef::new(
            "test".to_string(),
            "Test command".to_string(),
            "echo $ARGUMENTS".to_string(),
        );

        assert_eq!(cmd.name, "test");
        assert_eq!(cmd.description, "Test command");
        assert_eq!(cmd.template, "echo $ARGUMENTS");
        assert_eq!(cmd.agent, None);
        assert_eq!(cmd.model, None);
        assert_eq!(cmd.subtask, None);
    }

    #[test]
    fn test_command_def_validation_success() {
        let cmd = CommandDef::new(
            "test".to_string(),
            "Test command".to_string(),
            "echo $ARGUMENTS".to_string(),
        );

        assert!(cmd.validate().is_ok());
    }

    #[test]
    fn test_command_def_validation_empty_name() {
        let cmd = CommandDef::new(
            String::new(),
            "Test command".to_string(),
            "echo $ARGUMENTS".to_string(),
        );

        assert!(cmd.validate().is_err());
    }

    #[test]
    fn test_command_def_validation_empty_description() {
        let cmd = CommandDef::new(
            "test".to_string(),
            String::new(),
            "echo $ARGUMENTS".to_string(),
        );

        assert!(cmd.validate().is_err());
    }

    #[test]
    fn test_command_def_validation_empty_template() {
        let cmd = CommandDef::new(
            "test".to_string(),
            "Test command".to_string(),
            String::new(),
        );

        assert!(cmd.validate().is_err());
    }

    #[test]
    fn test_execution_context_creation() {
        let cmd = CommandDef::new(
            "test".to_string(),
            "Test command".to_string(),
            "echo $ARGUMENTS".to_string(),
        );

        let ctx = ExecutionContext::new(
            cmd.clone(),
            vec!["arg1".to_string(), "arg2".to_string()],
            PathBuf::from("/tmp"),
        );

        assert_eq!(ctx.command.name, "test");
        assert_eq!(ctx.arguments.len(), 2);
        assert_eq!(ctx.timeout_secs, 30);
    }

    #[test]
    fn test_execution_context_custom_timeout() {
        let cmd = CommandDef::new(
            "test".to_string(),
            "Test command".to_string(),
            "echo $ARGUMENTS".to_string(),
        );

        let ctx = ExecutionContext::new(
            cmd,
            vec![],
            PathBuf::from("/tmp"),
        ).with_timeout(60);

        assert_eq!(ctx.timeout_secs, 60);
    }

    #[test]
    fn test_execution_status_success() {
        let status = ExecutionStatus::Success;
        assert!(status.is_success());
        assert!(!status.is_error());
    }

    #[test]
    fn test_execution_status_error() {
        let status = ExecutionStatus::ArgumentError("test error".to_string());
        assert!(!status.is_success());
        assert!(status.is_error());
    }

    #[test]
    fn test_execution_result_success() {
        let result = ExecutionResult::success("output".to_string(), 100);
        assert!(result.is_success());
        assert_eq!(result.output, "output");
        assert_eq!(result.error, None);
    }

    #[test]
    fn test_execution_result_error() {
        let result = ExecutionResult::error(
            ExecutionStatus::ArgumentError("test".to_string()),
            "error message".to_string(),
            100,
        );
        assert!(!result.is_success());
        assert_eq!(result.error, Some("error message".to_string()));
    }

    // Parser tests
    use crate::commands::custom::parser;

    #[test]
    fn test_parse_json_valid() {
        let json = r#"{
            "name": "test-cmd",
            "description": "Test command",
            "template": "echo $ARGUMENTS"
        }"#;

        let result = parser::parse_json(json);
        assert!(result.is_ok());

        let cmd = result.unwrap();
        assert_eq!(cmd.name, "test-cmd");
        assert_eq!(cmd.description, "Test command");
        assert_eq!(cmd.template, "echo $ARGUMENTS");
    }

    #[test]
    fn test_parse_json_with_optional_fields() {
        let json = r#"{
            "name": "test-cmd",
            "description": "Test command",
            "template": "echo $ARGUMENTS",
            "agent": "my-agent",
            "model": "my-model",
            "subtask": true
        }"#;

        let result = parser::parse_json(json);
        assert!(result.is_ok());

        let cmd = result.unwrap();
        assert_eq!(cmd.agent, Some("my-agent".to_string()));
        assert_eq!(cmd.model, Some("my-model".to_string()));
        assert_eq!(cmd.subtask, Some(true));
    }

    #[test]
    fn test_parse_json_missing_required_field() {
        let json = r#"{
            "name": "test-cmd",
            "description": "Test command"
        }"#;

        let result = parser::parse_json(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_json_invalid_json() {
        let json = r#"{ invalid json }"#;

        let result = parser::parse_json(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_markdown_valid() {
        let markdown = r#"
# My Command

```command
name: test-cmd
description: Test command
template: echo $ARGUMENTS
```
"#;

        let result = parser::parse_markdown(markdown);
        assert!(result.is_ok());

        let cmd = result.unwrap();
        assert_eq!(cmd.name, "test-cmd");
        assert_eq!(cmd.description, "Test command");
        assert_eq!(cmd.template, "echo $ARGUMENTS");
    }

    #[test]
    fn test_parse_markdown_with_optional_fields() {
        let markdown = r#"
```command
name: test-cmd
description: Test command
template: echo $ARGUMENTS
agent: my-agent
model: my-model
subtask: true
```
"#;

        let result = parser::parse_markdown(markdown);
        assert!(result.is_ok());

        let cmd = result.unwrap();
        assert_eq!(cmd.agent, Some("my-agent".to_string()));
        assert_eq!(cmd.model, Some("my-model".to_string()));
        assert_eq!(cmd.subtask, Some(true));
    }

    #[test]
    fn test_parse_markdown_missing_code_block() {
        let markdown = r#"
# My Command

This is just text without a code block.
"#;

        let result = parser::parse_markdown(markdown);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_markdown_missing_required_field() {
        let markdown = r#"
```command
name: test-cmd
description: Test command
```
"#;

        let result = parser::parse_markdown(markdown);
        assert!(result.is_err());
    }

    #[test]
    fn test_to_json() {
        let cmd = CommandDef::new(
            "test-cmd".to_string(),
            "Test command".to_string(),
            "echo $ARGUMENTS".to_string(),
        );

        let result = parser::to_json(&cmd);
        assert!(result.is_ok());

        let json_str = result.unwrap();
        assert!(json_str.contains("test-cmd"));
        assert!(json_str.contains("Test command"));
        assert!(json_str.contains("echo $ARGUMENTS"));
    }

    #[test]
    fn test_to_markdown() {
        let cmd = CommandDef::new(
            "test-cmd".to_string(),
            "Test command".to_string(),
            "echo $ARGUMENTS".to_string(),
        );

        let markdown = parser::to_markdown(&cmd);
        assert!(markdown.contains("```command"));
        assert!(markdown.contains("name: test-cmd"));
        assert!(markdown.contains("description: Test command"));
        assert!(markdown.contains("template: echo $ARGUMENTS"));
        assert!(markdown.contains("```"));
    }

    #[test]
    fn test_json_roundtrip() {
        let original = CommandDef {
            name: "test-cmd".to_string(),
            description: "Test command".to_string(),
            template: "echo $ARGUMENTS".to_string(),
            agent: Some("my-agent".to_string()),
            model: Some("my-model".to_string()),
            subtask: Some(true),
        };

        let json_str = parser::to_json(&original).unwrap();
        let parsed = parser::parse_json(&json_str).unwrap();

        assert_eq!(original.name, parsed.name);
        assert_eq!(original.description, parsed.description);
        assert_eq!(original.template, parsed.template);
        assert_eq!(original.agent, parsed.agent);
        assert_eq!(original.model, parsed.model);
        assert_eq!(original.subtask, parsed.subtask);
    }

    #[test]
    fn test_markdown_roundtrip() {
        let original = CommandDef {
            name: "test-cmd".to_string(),
            description: "Test command".to_string(),
            template: "echo $ARGUMENTS".to_string(),
            agent: Some("my-agent".to_string()),
            model: Some("my-model".to_string()),
            subtask: Some(true),
        };

        let markdown = parser::to_markdown(&original);
        let parsed = parser::parse_markdown(&markdown).unwrap();

        assert_eq!(original.name, parsed.name);
        assert_eq!(original.description, parsed.description);
        assert_eq!(original.template, parsed.template);
        assert_eq!(original.agent, parsed.agent);
        assert_eq!(original.model, parsed.model);
        assert_eq!(original.subtask, parsed.subtask);
    }

    // Registry tests
    #[test]
    fn test_registry_creation() {
        let registry = CommandRegistry::new();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_registry_register_command() {
        let mut registry = CommandRegistry::new();
        let cmd = CommandDef::new(
            "test".to_string(),
            "Test command".to_string(),
            "echo $ARGUMENTS".to_string(),
        );

        let result = registry.register(cmd.clone());
        assert!(result.is_ok());
        assert_eq!(registry.count(), 1);
        assert!(registry.contains("test"));
    }

    #[test]
    fn test_registry_register_duplicate() {
        let mut registry = CommandRegistry::new();
        let cmd = CommandDef::new(
            "test".to_string(),
            "Test command".to_string(),
            "echo $ARGUMENTS".to_string(),
        );

        registry.register(cmd.clone()).unwrap();
        let result = registry.register(cmd);
        assert!(result.is_err());
    }

    #[test]
    fn test_registry_get_command() {
        let mut registry = CommandRegistry::new();
        let cmd = CommandDef::new(
            "test".to_string(),
            "Test command".to_string(),
            "echo $ARGUMENTS".to_string(),
        );

        registry.register(cmd.clone()).unwrap();
        let retrieved = registry.get("test");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "test");
    }

    #[test]
    fn test_registry_get_nonexistent() {
        let registry = CommandRegistry::new();
        let retrieved = registry.get("nonexistent");
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_registry_unregister() {
        let mut registry = CommandRegistry::new();
        let cmd = CommandDef::new(
            "test".to_string(),
            "Test command".to_string(),
            "echo $ARGUMENTS".to_string(),
        );

        registry.register(cmd).unwrap();
        assert_eq!(registry.count(), 1);

        let result = registry.unregister("test");
        assert!(result.is_ok());
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_registry_unregister_nonexistent() {
        let mut registry = CommandRegistry::new();
        let result = registry.unregister("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_registry_update() {
        let mut registry = CommandRegistry::new();
        let cmd = CommandDef::new(
            "test".to_string(),
            "Test command".to_string(),
            "echo $ARGUMENTS".to_string(),
        );

        registry.register(cmd).unwrap();

        let updated = CommandDef {
            name: "test".to_string(),
            description: "Updated description".to_string(),
            template: "echo updated".to_string(),
            agent: None,
            model: None,
            subtask: None,
        };

        let result = registry.update(updated);
        assert!(result.is_ok());

        let retrieved = registry.get("test").unwrap();
        assert_eq!(retrieved.description, "Updated description");
        assert_eq!(retrieved.template, "echo updated");
    }

    #[test]
    fn test_registry_update_nonexistent() {
        let mut registry = CommandRegistry::new();
        let cmd = CommandDef::new(
            "test".to_string(),
            "Test command".to_string(),
            "echo $ARGUMENTS".to_string(),
        );

        let result = registry.update(cmd);
        assert!(result.is_err());
    }

    #[test]
    fn test_registry_list_all() {
        let mut registry = CommandRegistry::new();

        let cmd1 = CommandDef::new(
            "cmd1".to_string(),
            "Command 1".to_string(),
            "echo 1".to_string(),
        );

        let cmd2 = CommandDef::new(
            "cmd2".to_string(),
            "Command 2".to_string(),
            "echo 2".to_string(),
        );

        registry.register(cmd1).unwrap();
        registry.register(cmd2).unwrap();

        let all = registry.list_all();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_registry_list_names() {
        let mut registry = CommandRegistry::new();

        let cmd1 = CommandDef::new(
            "cmd1".to_string(),
            "Command 1".to_string(),
            "echo 1".to_string(),
        );

        let cmd2 = CommandDef::new(
            "cmd2".to_string(),
            "Command 2".to_string(),
            "echo 2".to_string(),
        );

        registry.register(cmd1).unwrap();
        registry.register(cmd2).unwrap();

        let names = registry.list_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"cmd1".to_string()));
        assert!(names.contains(&"cmd2".to_string()));
    }

    #[test]
    fn test_registry_clear() {
        let mut registry = CommandRegistry::new();

        let cmd = CommandDef::new(
            "test".to_string(),
            "Test command".to_string(),
            "echo $ARGUMENTS".to_string(),
        );

        registry.register(cmd).unwrap();
        assert_eq!(registry.count(), 1);

        registry.clear();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_registry_filter() {
        let mut registry = CommandRegistry::new();

        let cmd1 = CommandDef {
            name: "cmd1".to_string(),
            description: "Command 1".to_string(),
            template: "echo 1".to_string(),
            agent: Some("agent1".to_string()),
            model: None,
            subtask: None,
        };

        let cmd2 = CommandDef {
            name: "cmd2".to_string(),
            description: "Command 2".to_string(),
            template: "echo 2".to_string(),
            agent: None,
            model: None,
            subtask: None,
        };

        registry.register(cmd1).unwrap();
        registry.register(cmd2).unwrap();

        let with_agent = registry.filter(|cmd| cmd.agent.is_some());
        assert_eq!(with_agent.len(), 1);
        assert_eq!(with_agent[0].name, "cmd1");
    }

    #[test]
    fn test_registry_default() {
        let registry = CommandRegistry::default();
        assert_eq!(registry.count(), 0);
    }
}


/// Action for custom command handler
#[derive(Debug, Clone)]
pub enum CustomAction {
    /// List all available custom commands
    List,
    /// Show help for a specific command
    Help(String),
    /// Execute a custom command
    Run(String, Vec<String>),
    /// Load custom commands from a file
    Load(String),
    /// Search for custom commands
    Search(String),
}

/// Handler for custom command CLI operations
pub struct CustomCommandHandler {
    action: CustomAction,
    registry: CommandRegistry,
}

impl CustomCommandHandler {
    /// Create a new custom command handler
    pub fn new(action: CustomAction) -> Self {
        Self {
            action,
            registry: CommandRegistry::new(),
        }
    }

    /// Create a handler with a pre-populated registry
    pub fn with_registry(action: CustomAction, registry: CommandRegistry) -> Self {
        Self { action, registry }
    }
}

impl Command for CustomCommandHandler {
    fn execute(&self) -> CliResult<()> {
        match &self.action {
            CustomAction::List => self.handle_list(),
            CustomAction::Help(name) => self.handle_help(name),
            CustomAction::Run(name, args) => self.handle_run(name, args),
            CustomAction::Load(file) => self.handle_load(file),
            CustomAction::Search(query) => self.handle_search(query),
        }
    }
}

impl CustomCommandHandler {
    /// Handle list action - display all available commands
    fn handle_list(&self) -> CliResult<()> {
        let commands = discovery::list_commands(&self.registry);

        if commands.is_empty() {
            println!("No custom commands available.");
            println!("Use 'rice custom load <file>' to load commands from a file.");
            return Ok(());
        }

        let formatted = discovery::format_command_list(&commands);
        println!("{}", formatted);

        Ok(())
    }

    /// Handle help action - show help for a specific command
    fn handle_help(&self, name: &str) -> CliResult<()> {
        match discovery::get_command_help(&self.registry, name) {
            Some(help_text) => {
                println!("{}", help_text);
                Ok(())
            }
            None => Err(CliError::InvalidArgument {
                message: format!("Command '{}' not found", name),
            }),
        }
    }

    /// Handle run action - execute a custom command
    fn handle_run(&self, name: &str, args: &[String]) -> CliResult<()> {
        // Get the command from registry
        let cmd_def = self.registry.get(name).ok_or_else(|| CliError::InvalidArgument {
            message: format!("Command '{}' not found", name),
        })?;

        // Create execution context
        let context = ExecutionContext::new(
            cmd_def,
            args.to_vec(),
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        );

        // Execute the command
        match executor::execute(&context) {
            Ok(result) => {
                if result.is_success() {
                    println!("{}", result.output);
                    Ok(())
                } else {
                    Err(CliError::Generation(
                        result.error.unwrap_or_else(|| result.status.message()),
                    ))
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Handle load action - load commands from a file
    fn handle_load(&self, file: &str) -> CliResult<()> {
        // Determine file format based on extension
        let cmd_def = if file.ends_with(".json") {
            parser::parse_json_file(file)?
        } else if file.ends_with(".md") {
            parser::parse_markdown_file(file)?
        } else {
            // Try JSON first, then Markdown
            parser::parse_json_file(file).or_else(|_| parser::parse_markdown_file(file))?
        };

        println!("Loaded command: {}", cmd_def.name);
        println!("Description: {}", cmd_def.description);
        println!("Template: {}", cmd_def.template);

        Ok(())
    }

    /// Handle search action - search for commands
    fn handle_search(&self, query: &str) -> CliResult<()> {
        let results = discovery::search_commands(&self.registry, query);

        if results.is_empty() {
            println!("No commands found matching '{}'", query);
            return Ok(());
        }

        println!("Search results for '{}':", query);
        println!("========================\n");

        for cmd in results {
            println!("{}", cmd.summary());
        }

        Ok(())
    }
}

#[cfg(test)]
mod custom_handler_tests {
    use super::*;

    #[test]
    fn test_custom_handler_list_empty() {
        let handler = CustomCommandHandler::new(CustomAction::List);
        // Should not panic and should handle empty registry gracefully
        let result = handler.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_custom_handler_help_not_found() {
        let handler = CustomCommandHandler::new(CustomAction::Help("nonexistent".to_string()));
        let result = handler.execute();
        assert!(result.is_err());
    }

    #[test]
    fn test_custom_handler_run_not_found() {
        let handler = CustomCommandHandler::new(CustomAction::Run(
            "nonexistent".to_string(),
            vec![],
        ));
        let result = handler.execute();
        assert!(result.is_err());
    }

    #[test]
    fn test_custom_handler_search_empty() {
        let handler = CustomCommandHandler::new(CustomAction::Search("test".to_string()));
        // Should not panic and should handle empty results gracefully
        let result = handler.execute();
        assert!(result.is_ok());
    }
}


/// Storage integration for custom commands
pub mod storage {
    use super::*;
    use std::fs;
    use std::path::Path;

    /// Load custom commands from storage
    ///
    /// Loads commands from:
    /// 1. Global storage: ~/.ricecoder/commands/
    /// 2. Project storage: ./.agent/commands/
    ///
    /// # Arguments
    /// - `global_path`: Path to global storage directory
    /// - `project_path`: Optional path to project storage directory
    ///
    /// # Returns
    /// A CommandRegistry with all loaded commands
    pub fn load_from_storage(
        global_path: &Path,
        project_path: Option<&Path>,
    ) -> CliResult<CommandRegistry> {
        let mut registry = CommandRegistry::new();

        // Load from global storage
        let global_commands_dir = global_path.join("commands");
        if global_commands_dir.exists() {
            load_commands_from_directory(&global_commands_dir, &mut registry)?;
        }

        // Load from project storage (overrides global)
        if let Some(project_path) = project_path {
            let project_commands_dir = project_path.join("commands");
            if project_commands_dir.exists() {
                load_commands_from_directory(&project_commands_dir, &mut registry)?;
            }
        }

        Ok(registry)
    }

    /// Load commands from a directory
    fn load_commands_from_directory(
        dir: &Path,
        registry: &mut CommandRegistry,
    ) -> CliResult<()> {
        if !dir.is_dir() {
            return Ok(());
        }

        for entry in fs::read_dir(dir).map_err(|e| CliError::Io(e))? {
            let entry = entry.map_err(|e| CliError::Io(e))?;
            let path = entry.path();

            if path.is_file() {
                let file_name = path.file_name().unwrap().to_string_lossy();

                // Try to load as JSON or Markdown
                let cmd_def = if file_name.ends_with(".json") {
                    parser::parse_json_file(path.to_str().unwrap())?
                } else if file_name.ends_with(".md") {
                    parser::parse_markdown_file(path.to_str().unwrap())?
                } else {
                    // Skip files that don't match expected formats
                    continue;
                };

                // Register the command (ignore duplicates from project overriding global)
                let _ = registry.register(cmd_def);
            }
        }

        Ok(())
    }

    /// Save a command to storage
    ///
    /// Saves to project storage if available, otherwise to global storage
    ///
    /// # Arguments
    /// - `cmd`: The command definition to save
    /// - `global_path`: Path to global storage directory
    /// - `project_path`: Optional path to project storage directory
    /// - `format`: Format to save in (JSON or Markdown)
    ///
    /// # Returns
    /// The path where the command was saved
    pub fn save_command(
        cmd: &CommandDef,
        global_path: &Path,
        project_path: Option<&Path>,
        format: CommandFormat,
    ) -> CliResult<PathBuf> {
        // Determine target directory
        let target_dir = if let Some(project_path) = project_path {
            project_path.join("commands")
        } else {
            global_path.join("commands")
        };

        // Create directory if it doesn't exist
        fs::create_dir_all(&target_dir).map_err(|e| CliError::Io(e))?;

        // Determine file name and content
        let (file_name, content) = match format {
            CommandFormat::Json => {
                let json_str = parser::to_json(cmd)?;
                (format!("{}.json", cmd.name), json_str.into_bytes())
            }
            CommandFormat::Markdown => {
                let markdown = parser::to_markdown(cmd);
                (format!("{}.md", cmd.name), markdown.into_bytes())
            }
        };

        let file_path = target_dir.join(&file_name);

        // Write file
        fs::write(&file_path, content).map_err(|e| CliError::Io(e))?;

        Ok(file_path)
    }

    /// Delete a command from storage
    ///
    /// Deletes from project storage if available, otherwise from global storage
    pub fn delete_command(
        name: &str,
        global_path: &Path,
        project_path: Option<&Path>,
    ) -> CliResult<()> {
        // Try project storage first
        if let Some(project_path) = project_path {
            let project_commands_dir = project_path.join("commands");
            for format in &["json", "md"] {
                let file_path = project_commands_dir.join(format!("{}.{}", name, format));
                if file_path.exists() {
                    fs::remove_file(&file_path).map_err(|e| CliError::Io(e))?;
                    return Ok(());
                }
            }
        }

        // Try global storage
        let global_commands_dir = global_path.join("commands");
        for format in &["json", "md"] {
            let file_path = global_commands_dir.join(format!("{}.{}", name, format));
            if file_path.exists() {
                fs::remove_file(&file_path).map_err(|e| CliError::Io(e))?;
                return Ok(());
            }
        }

        Err(CliError::InvalidArgument {
            message: format!("Command '{}' not found in storage", name),
        })
    }

    /// Format for saving commands
    #[derive(Debug, Clone, Copy)]
    pub enum CommandFormat {
        /// Save as JSON
        Json,
        /// Save as Markdown
        Markdown,
    }
}

#[cfg(test)]
mod storage_tests {
    use super::storage::*;
    use super::CommandDef;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_load_from_empty_storage() {
        let temp_dir = TempDir::new().unwrap();
        let registry = load_from_storage(temp_dir.path(), None).unwrap();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_save_and_load_json_command() {
        let temp_dir = TempDir::new().unwrap();
        let cmd = CommandDef::new(
            "test-cmd".to_string(),
            "Test command".to_string(),
            "echo $ARGUMENTS".to_string(),
        );

        // Save command
        let saved_path = save_command(&cmd, temp_dir.path(), None, CommandFormat::Json).unwrap();
        assert!(saved_path.exists());

        // Load and verify
        let registry = load_from_storage(temp_dir.path(), None).unwrap();
        assert_eq!(registry.count(), 1);

        let loaded = registry.get("test-cmd").unwrap();
        assert_eq!(loaded.name, cmd.name);
        assert_eq!(loaded.description, cmd.description);
        assert_eq!(loaded.template, cmd.template);
    }

    #[test]
    fn test_save_and_load_markdown_command() {
        let temp_dir = TempDir::new().unwrap();
        let cmd = CommandDef::new(
            "test-cmd".to_string(),
            "Test command".to_string(),
            "echo $ARGUMENTS".to_string(),
        );

        // Save command
        let saved_path =
            save_command(&cmd, temp_dir.path(), None, CommandFormat::Markdown).unwrap();
        assert!(saved_path.exists());

        // Load and verify
        let registry = load_from_storage(temp_dir.path(), None).unwrap();
        assert_eq!(registry.count(), 1);

        let loaded = registry.get("test-cmd").unwrap();
        assert_eq!(loaded.name, cmd.name);
    }

    #[test]
    fn test_delete_command() {
        let temp_dir = TempDir::new().unwrap();
        let cmd = CommandDef::new(
            "test-cmd".to_string(),
            "Test command".to_string(),
            "echo $ARGUMENTS".to_string(),
        );

        // Save command
        save_command(&cmd, temp_dir.path(), None, CommandFormat::Json).unwrap();

        // Verify it exists
        let registry = load_from_storage(temp_dir.path(), None).unwrap();
        assert_eq!(registry.count(), 1);

        // Delete command
        delete_command("test-cmd", temp_dir.path(), None).unwrap();

        // Verify it's gone
        let registry = load_from_storage(temp_dir.path(), None).unwrap();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_delete_nonexistent_command() {
        let temp_dir = TempDir::new().unwrap();
        let result = delete_command("nonexistent", temp_dir.path(), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_project_storage_overrides_global() {
        let global_dir = TempDir::new().unwrap();
        let project_dir = TempDir::new().unwrap();

        let global_cmd = CommandDef::new(
            "test-cmd".to_string(),
            "Global command".to_string(),
            "echo global".to_string(),
        );

        let project_cmd = CommandDef::new(
            "test-cmd".to_string(),
            "Project command".to_string(),
            "echo project".to_string(),
        );

        // Save to both
        save_command(&global_cmd, global_dir.path(), None, CommandFormat::Json).unwrap();
        save_command(&project_cmd, project_dir.path(), None, CommandFormat::Json).unwrap();

        // Load with project override
        let registry = load_from_storage(global_dir.path(), Some(project_dir.path())).unwrap();
        assert_eq!(registry.count(), 1);

        // Project version should override
        let loaded = registry.get("test-cmd").unwrap();
        assert_eq!(loaded.description, "Project command");
    }
}

// Property-based tests using proptest
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    /// Strategy for generating valid command names
    fn command_name_strategy() -> impl Strategy<Value = String> {
        "[a-z0-9_-]{1,50}".prop_map(|s| s.to_string())
    }

    /// Strategy for generating valid command descriptions
    fn command_description_strategy() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9.,!?-]{1,200}".prop_map(|s| {
            // Ensure at least one non-whitespace character
            if s.trim().is_empty() {
                "Description".to_string()
            } else {
                s
            }
        })
    }

    /// Strategy for generating valid command templates
    fn command_template_strategy() -> impl Strategy<Value = String> {
        r"[a-zA-Z0-9${}()_\-./,;:!?@#%&*+\[\]]{1,500}".prop_map(|s| {
            // Ensure at least one non-whitespace character
            if s.trim().is_empty() {
                "echo".to_string()
            } else {
                s
            }
        })
    }

    /// Strategy for generating valid CommandDef instances
    fn command_def_strategy() -> impl Strategy<Value = CommandDef> {
        (
            command_name_strategy(),
            command_description_strategy(),
            command_template_strategy(),
            prop::option::of("[a-z0-9_-]{1,20}".prop_map(|s| s.to_string())),
            prop::option::of("[a-z0-9_-]{1,20}".prop_map(|s| s.to_string())),
            prop::option::of(any::<bool>()),
        )
            .prop_map(|(name, description, template, agent, model, subtask)| {
                CommandDef {
                    name,
                    description,
                    template,
                    agent,
                    model,
                    subtask,
                }
            })
    }

    // Property 10: Configuration Parsing Round-Trip
    // *For any* valid command configuration in JSON or Markdown format, parsing and then serializing
    // SHALL produce an equivalent configuration.
    // **Validates: Requirements 1.1, 1.2**
    #[test]
    fn prop_json_roundtrip() {
        proptest!(|(cmd in command_def_strategy().prop_filter("Valid commands only", |c| c.validate().is_ok()))| {
            // Serialize to JSON
            let json_str = parser::to_json(&cmd).expect("Failed to serialize to JSON");

            // Parse back from JSON
            let parsed = parser::parse_json(&json_str).expect("Failed to parse JSON");

            // Verify all fields match
            prop_assert_eq!(cmd.name, parsed.name, "Name mismatch after round-trip");
            prop_assert_eq!(cmd.description, parsed.description, "Description mismatch after round-trip");
            prop_assert_eq!(cmd.template, parsed.template, "Template mismatch after round-trip");
            prop_assert_eq!(cmd.agent, parsed.agent, "Agent mismatch after round-trip");
            prop_assert_eq!(cmd.model, parsed.model, "Model mismatch after round-trip");
            prop_assert_eq!(cmd.subtask, parsed.subtask, "Subtask mismatch after round-trip");
        });
    }

    #[test]
    fn prop_markdown_roundtrip() {
        proptest!(|(cmd in command_def_strategy().prop_filter("Valid commands only", |c| c.validate().is_ok()))| {
            // Serialize to Markdown
            let markdown = parser::to_markdown(&cmd);

            // Parse back from Markdown
            let parsed = parser::parse_markdown(&markdown).expect("Failed to parse Markdown");

            // Verify all fields match
            prop_assert_eq!(cmd.name, parsed.name, "Name mismatch after round-trip");
            prop_assert_eq!(cmd.description, parsed.description, "Description mismatch after round-trip");
            prop_assert_eq!(cmd.template, parsed.template, "Template mismatch after round-trip");
            prop_assert_eq!(cmd.agent, parsed.agent, "Agent mismatch after round-trip");
            prop_assert_eq!(cmd.model, parsed.model, "Model mismatch after round-trip");
            prop_assert_eq!(cmd.subtask, parsed.subtask, "Subtask mismatch after round-trip");
        });
    }

    // Property 11: Invalid Configuration Detection
    // *For any* invalid command configuration, the system SHALL detect the error and report it with clear context about what is invalid.
    // **Validates: Requirements 1.5**
    #[test]
    fn prop_invalid_config_detection() {
        // Test empty name
        proptest!(|(description in command_description_strategy(), template in command_template_strategy())| {
            let cmd = CommandDef {
                name: String::new(),
                description,
                template,
                agent: None,
                model: None,
                subtask: None,
            };

            // Should fail validation
            prop_assert!(cmd.validate().is_err(), "Empty name should fail validation");
        });

        // Test empty description
        proptest!(|(name in command_name_strategy(), template in command_template_strategy())| {
            let cmd = CommandDef {
                name,
                description: String::new(),
                template,
                agent: None,
                model: None,
                subtask: None,
            };

            // Should fail validation
            prop_assert!(cmd.validate().is_err(), "Empty description should fail validation");
        });

        // Test empty template
        proptest!(|(name in command_name_strategy(), description in command_description_strategy())| {
            let cmd = CommandDef {
                name,
                description,
                template: String::new(),
                agent: None,
                model: None,
                subtask: None,
            };

            // Should fail validation
            prop_assert!(cmd.validate().is_err(), "Empty template should fail validation");
        });
    }
}
