use crate::error::{CommandError, Result};
use crate::template::TemplateProcessor;
use crate::types::{CommandContext, CommandDefinition, CommandExecutionResult};
use std::collections::HashMap;
use std::process::Command;
use std::time::Instant;

/// Command executor for running shell commands
pub struct CommandExecutor;

impl CommandExecutor {
    /// Execute a command with the given context
    pub fn execute(
        command_def: &CommandDefinition,
        context: &CommandContext,
    ) -> Result<CommandExecutionResult> {
        // Validate that the command is enabled
        if !command_def.enabled {
            return Err(CommandError::ExecutionFailed(format!(
                "Command is disabled: {}",
                command_def.id
            )));
        }

        // Process the command template with arguments
        let processed_command =
            TemplateProcessor::process(&command_def.command, &context.arguments)?;

        // Start timing
        let start = Instant::now();

        // Execute the command
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", &processed_command])
                .current_dir(&context.cwd)
                .envs(&context.env)
                .output()
                .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?
        } else {
            Command::new("sh")
                .args(["-c", &processed_command])
                .current_dir(&context.cwd)
                .envs(&context.env)
                .output()
                .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?
        };

        let duration = start.elapsed();

        // Extract output
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);

        // Check for timeout
        if command_def.timeout_seconds > 0 && duration.as_secs() > command_def.timeout_seconds {
            return Err(CommandError::ExecutionFailed(format!(
                "Command execution timed out after {} seconds",
                command_def.timeout_seconds
            )));
        }

        Ok(CommandExecutionResult::new(&command_def.id, exit_code)
            .with_stdout(stdout)
            .with_stderr(stderr)
            .with_duration(duration.as_millis() as u64))
    }

    /// Execute a command and return only the output
    pub fn execute_and_get_output(
        command_def: &CommandDefinition,
        context: &CommandContext,
    ) -> Result<String> {
        let result = Self::execute(command_def, context)?;

        if result.success {
            Ok(result.stdout)
        } else {
            Err(CommandError::ExecutionFailed(format!(
                "Command failed with exit code {}: {}",
                result.exit_code, result.stderr
            )))
        }
    }

    /// Execute a command and return both stdout and stderr
    pub fn execute_and_get_all_output(
        command_def: &CommandDefinition,
        context: &CommandContext,
    ) -> Result<(String, String)> {
        let result = Self::execute(command_def, context)?;
        Ok((result.stdout, result.stderr))
    }

    /// Validate command arguments against the command definition
    pub fn validate_arguments(
        command_def: &CommandDefinition,
        arguments: &HashMap<String, String>,
    ) -> Result<()> {
        for arg_def in &command_def.arguments {
            if arg_def.required && !arguments.contains_key(&arg_def.name) {
                return Err(CommandError::InvalidArgument(format!(
                    "Missing required argument: {}",
                    arg_def.name
                )));
            }

            if let Some(value) = arguments.get(&arg_def.name) {
                // Validate against pattern if provided
                if let Some(pattern) = &arg_def.validation_pattern {
                    let regex = regex::Regex::new(pattern)?;
                    if !regex.is_match(value) {
                        return Err(CommandError::InvalidArgument(format!(
                            "Argument '{}' does not match pattern: {}",
                            arg_def.name, pattern
                        )));
                    }
                }
            }
        }

        Ok(())
    }

    /// Build a context with default values for missing arguments
    pub fn build_context_with_defaults(
        command_def: &CommandDefinition,
        mut arguments: HashMap<String, String>,
        cwd: String,
    ) -> Result<CommandContext> {
        // Fill in default values for missing arguments
        for arg_def in &command_def.arguments {
            if !arguments.contains_key(&arg_def.name) {
                if let Some(default) = &arg_def.default {
                    arguments.insert(arg_def.name.clone(), default.clone());
                }
            }
        }

        Ok(CommandContext {
            cwd,
            env: std::env::vars().collect(),
            arguments,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ArgumentType, CommandArgument};

    #[test]
    fn test_validate_arguments_success() {
        let mut cmd = CommandDefinition::new("test", "Test", "echo {{name}}");
        cmd.arguments.push(
            CommandArgument::new("name", ArgumentType::String)
                .with_required(true)
                .with_description("User name"),
        );

        let mut args = HashMap::new();
        args.insert("name".to_string(), "Alice".to_string());

        assert!(CommandExecutor::validate_arguments(&cmd, &args).is_ok());
    }

    #[test]
    fn test_validate_arguments_missing_required() {
        let mut cmd = CommandDefinition::new("test", "Test", "echo {{name}}");
        cmd.arguments.push(
            CommandArgument::new("name", ArgumentType::String)
                .with_required(true)
                .with_description("User name"),
        );

        let args = HashMap::new();
        assert!(CommandExecutor::validate_arguments(&cmd, &args).is_err());
    }

    #[test]
    fn test_validate_arguments_with_pattern() {
        let mut cmd = CommandDefinition::new("test", "Test", "echo {{email}}");
        cmd.arguments.push(
            CommandArgument::new("email", ArgumentType::String)
                .with_required(true)
                .with_validation_pattern(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"),
        );

        let mut args = HashMap::new();
        args.insert("email".to_string(), "invalid-email".to_string());
        assert!(CommandExecutor::validate_arguments(&cmd, &args).is_err());

        args.insert("email".to_string(), "test@example.com".to_string());
        assert!(CommandExecutor::validate_arguments(&cmd, &args).is_ok());
    }

    #[test]
    fn test_build_context_with_defaults() {
        let mut cmd = CommandDefinition::new("test", "Test", "echo {{name}}");
        cmd.arguments.push(
            CommandArgument::new("name", ArgumentType::String)
                .with_default("Guest")
                .with_description("User name"),
        );

        let args = HashMap::new();
        let context =
            CommandExecutor::build_context_with_defaults(&cmd, args, ".".to_string()).unwrap();

        assert_eq!(context.arguments.get("name").unwrap(), "Guest");
    }

    #[test]
    fn test_build_context_override_defaults() {
        let mut cmd = CommandDefinition::new("test", "Test", "echo {{name}}");
        cmd.arguments.push(
            CommandArgument::new("name", ArgumentType::String)
                .with_default("Guest")
                .with_description("User name"),
        );

        let mut args = HashMap::new();
        args.insert("name".to_string(), "Alice".to_string());
        let context =
            CommandExecutor::build_context_with_defaults(&cmd, args, ".".to_string()).unwrap();

        assert_eq!(context.arguments.get("name").unwrap(), "Alice");
    }

    #[test]
    fn test_execute_simple_command() {
        let cmd = CommandDefinition::new("test", "Test", "echo hello");
        let context = CommandContext {
            cwd: ".".to_string(),
            env: std::env::vars().collect(),
            arguments: HashMap::new(),
        };

        let result = CommandExecutor::execute(&cmd, &context).unwrap();
        assert!(result.success);
        assert!(result.stdout.contains("hello"));
    }

    #[test]
    fn test_execute_disabled_command() {
        let mut cmd = CommandDefinition::new("test", "Test", "echo hello");
        cmd.enabled = false;

        let context = CommandContext {
            cwd: ".".to_string(),
            env: std::env::vars().collect(),
            arguments: HashMap::new(),
        };

        assert!(CommandExecutor::execute(&cmd, &context).is_err());
    }

    #[test]
    fn test_execute_with_template() {
        let cmd = CommandDefinition::new("test", "Test", "echo {{message}}");
        let mut args = HashMap::new();
        args.insert("message".to_string(), "Hello World".to_string());

        let context = CommandContext {
            cwd: ".".to_string(),
            env: std::env::vars().collect(),
            arguments: args,
        };

        let result = CommandExecutor::execute(&cmd, &context).unwrap();
        assert!(result.success);
        assert!(result.stdout.contains("Hello World"));
    }
}
