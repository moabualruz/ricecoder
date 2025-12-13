use crate::config::ConfigManager;
use crate::error::{CommandError, Result};
use crate::output_injection::{OutputInjectionConfig, OutputInjector};
use crate::registry::CommandRegistry;
use crate::types::{ArgumentType, CommandArgument, CommandContext, CommandDefinition, CommandExecutionResult};
use std::collections::HashMap;
use std::path::Path;

/// High-level command manager for executing and managing commands
pub struct CommandManager {
    registry: CommandRegistry,
    output_config: OutputInjectionConfig,
}

impl CommandManager {
    /// Create a new command manager
    pub fn new(registry: CommandRegistry) -> Self {
        Self {
            registry,
            output_config: OutputInjectionConfig::default(),
        }
    }

    /// Create a command manager from a config file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let registry = ConfigManager::load_from_file(path)?;
        Ok(Self::new(registry))
    }

    /// Set the output injection configuration
    pub fn set_output_config(&mut self, config: OutputInjectionConfig) {
        self.output_config = config;
    }

    /// Get the output injection configuration
    pub fn output_config(&self) -> &OutputInjectionConfig {
        &self.output_config
    }

    /// Get the registry
    pub fn registry(&self) -> &CommandRegistry {
        &self.registry
    }

    /// Get a mutable reference to the registry
    pub fn registry_mut(&mut self) -> &mut CommandRegistry {
        &mut self.registry
    }

    /// Execute a command by ID
    pub fn execute(
        &self,
        command_id: &str,
        arguments: HashMap<String, String>,
        cwd: String,
    ) -> Result<CommandExecutionResult> {
        let command = self.registry.get(command_id)?;

        // Validate arguments
        self.validate_arguments(&command, &arguments)?;

        // Build context with defaults
        let context = self.build_context_with_defaults(&command, arguments, cwd)?;

        // Execute the command
        self.execute_command(&command, &context)
    }

    /// Execute a command and get injected output
    pub fn execute_and_inject(
        &self,
        command_id: &str,
        arguments: HashMap<String, String>,
        cwd: String,
    ) -> Result<String> {
        let result = self.execute(command_id, arguments, cwd)?;
        OutputInjector::inject(&result, &self.output_config)
    }

    /// List all commands
    pub fn list_commands(&self) -> Vec<CommandDefinition> {
        self.registry.list_all()
    }

    /// List enabled commands
    pub fn list_enabled_commands(&self) -> Vec<CommandDefinition> {
        self.registry.list_enabled()
    }

    /// Get command details
    pub fn get_command(&self, command_id: &str) -> Result<CommandDefinition> {
        self.registry.get(command_id)
    }

    /// Search for commands
    pub fn search_commands(&self, query: &str) -> Vec<CommandDefinition> {
        self.registry.search(query)
    }

    /// Find commands by tag
    pub fn find_commands_by_tag(&self, tag: &str) -> Vec<CommandDefinition> {
        self.registry.find_by_tag(tag)
    }

    /// Register a new command
    pub fn register_command(&mut self, command: CommandDefinition) -> Result<()> {
        self.registry.register(command)
    }

    /// Unregister a command
    pub fn unregister_command(&mut self, command_id: &str) -> Result<()> {
        self.registry.unregister(command_id)
    }

    /// Enable a command
    pub fn enable_command(&mut self, command_id: &str) -> Result<()> {
        self.registry.enable(command_id)
    }

    /// Disable a command
    pub fn disable_command(&mut self, command_id: &str) -> Result<()> {
        self.registry.disable(command_id)
    }

    /// Save commands to a file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        ConfigManager::save_to_file(&self.registry, path)
    }

    /// Reload commands from a file
    pub fn reload_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        self.registry = ConfigManager::load_from_file(path)?;
        Ok(())
    }

    /// Validate command arguments
    fn validate_arguments(
        &self,
        command: &CommandDefinition,
        arguments: &HashMap<String, String>,
    ) -> Result<()> {
        for arg in &command.arguments {
            let value = arguments.get(&arg.name);

            // Check required arguments
            if arg.required && (value.is_none() || value.unwrap().trim().is_empty()) {
                return Err(CommandError::ValidationError(format!(
                    "Required argument '{}' is missing or empty",
                    arg.name
                )));
            }

            // Validate argument type and pattern if value is provided
            if let Some(val) = value {
                self.validate_argument_value(arg, val)?;
            }
        }

        Ok(())
    }

    /// Validate a single argument value
    fn validate_argument_value(&self, arg: &CommandArgument, value: &str) -> Result<()> {
        // Type validation
        match arg.arg_type {
            ArgumentType::String => {
                // String validation - just check length if pattern exists
                if let Some(pattern) = &arg.validation_pattern {
                    let regex = regex::Regex::new(pattern).map_err(|_| {
                        CommandError::ValidationError(format!(
                            "Invalid regex pattern for argument '{}'",
                            arg.name
                        ))
                    })?;
                    if !regex.is_match(value) {
                        return Err(CommandError::ValidationError(format!(
                            "Argument '{}' does not match required pattern",
                            arg.name
                        )));
                    }
                }
            }
            ArgumentType::Number => {
                if value.parse::<f64>().is_err() {
                    return Err(CommandError::ValidationError(format!(
                        "Argument '{}' must be a valid number",
                        arg.name
                    )));
                }
            }
            ArgumentType::Boolean => {
                let lower = value.to_lowercase();
                if !matches!(lower.as_str(), "true" | "false" | "1" | "0" | "yes" | "no") {
                    return Err(CommandError::ValidationError(format!(
                        "Argument '{}' must be a boolean value",
                        arg.name
                    )));
                }
            }
            ArgumentType::Path => {
                // For now, just check it's not empty - could add path validation later
                if value.trim().is_empty() {
                    return Err(CommandError::ValidationError(format!(
                        "Argument '{}' cannot be empty",
                        arg.name
                    )));
                }
            }
            ArgumentType::Choice(ref options) => {
                if !options.contains(&value.to_string()) {
                    return Err(CommandError::ValidationError(format!(
                        "Argument '{}' must be one of: {:?}",
                        arg.name, options
                    )));
                }
            }
        }

        Ok(())
    }

    /// Build execution context with default values
    fn build_context_with_defaults(
        &self,
        command: &CommandDefinition,
        mut arguments: HashMap<String, String>,
        cwd: String,
    ) -> Result<CommandContext> {
        // Fill in default values for missing arguments
        for arg in &command.arguments {
            if !arguments.contains_key(&arg.name) {
                if let Some(default) = &arg.default {
                    arguments.insert(arg.name.clone(), default.clone());
                }
            }
        }

        Ok(CommandContext {
            cwd,
            env: std::env::vars().collect(),
            arguments,
        })
    }

    /// Execute a command with the given context
    fn execute_command(
        &self,
        command: &CommandDefinition,
        context: &CommandContext,
    ) -> Result<CommandExecutionResult> {
        use std::process::{Command, Stdio};
        use std::time::Instant;

        let start_time = Instant::now();

        // Substitute arguments in the command template
        let mut command_str = command.command.clone();
        for (key, value) in &context.arguments {
            let placeholder = format!("{{{{{}}}}}", key);
            command_str = command_str.replace(&placeholder, value);
        }

        // Execute the command
        let mut cmd = if cfg!(target_os = "windows") {
            let mut c = Command::new("cmd");
            c.args(["/C", &command_str]);
            c
        } else {
            let mut c = Command::new("sh");
            c.args(["-c", &command_str]);
            c
        };

        cmd.current_dir(&context.cwd)
            .env_clear()
            .envs(&context.env)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let output = cmd.output()
            .map_err(|e| CommandError::ExecutionError(format!("Failed to execute command: {}", e)))?;

        let duration = start_time.elapsed().as_millis() as u64;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);
        let success = output.status.success();

        Ok(CommandExecutionResult {
            command_id: command.id.clone(),
            exit_code,
            stdout,
            stderr,
            success,
            duration_ms: duration,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ArgumentType, CommandArgument};

    fn create_test_manager() -> CommandManager {
        let mut registry = CommandRegistry::new();
        let cmd = CommandDefinition::new("test", "Test Command", "echo {{message}}")
            .with_description("A test command")
            .with_argument(
                CommandArgument::new("message", ArgumentType::String)
                    .with_description("Message to echo")
                    .with_required(true),
            );
        registry.register(cmd).ok();
        CommandManager::new(registry)
    }

    #[test]
    fn test_list_commands() {
        let manager = create_test_manager();
        let commands = manager.list_commands();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].id, "test");
    }

    #[test]
    fn test_get_command() {
        let manager = create_test_manager();
        let cmd = manager.get_command("test").unwrap();
        assert_eq!(cmd.id, "test");
        assert_eq!(cmd.name, "Test Command");
    }

    #[test]
    fn test_search_commands() {
        let manager = create_test_manager();
        let results = manager.search_commands("test");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_register_command() {
        let mut manager = create_test_manager();
        let cmd = CommandDefinition::new("new-cmd", "New Command", "echo new");
        assert!(manager.register_command(cmd).is_ok());
        assert_eq!(manager.list_commands().len(), 2);
    }

    #[test]
    fn test_unregister_command() {
        let mut manager = create_test_manager();
        assert!(manager.unregister_command("test").is_ok());
        assert_eq!(manager.list_commands().len(), 0);
    }

    #[test]
    fn test_enable_disable() {
        let mut manager = create_test_manager();
        assert!(manager.disable_command("test").is_ok());
        assert_eq!(manager.list_enabled_commands().len(), 0);
        assert!(manager.enable_command("test").is_ok());
        assert_eq!(manager.list_enabled_commands().len(), 1);
    }

    #[test]
    fn test_execute_command() {
        let manager = create_test_manager();
        let mut args = HashMap::new();
        args.insert("message".to_string(), "Hello".to_string());

        let result = manager.execute("test", args, ".".to_string()).unwrap();
        assert!(result.success);
        assert!(result.stdout.contains("Hello"));
    }

    #[test]
    fn test_execute_missing_required_argument() {
        let manager = create_test_manager();
        let args = HashMap::new();
        let result = manager.execute("test", args, ".".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_nonexistent_command() {
        let manager = create_test_manager();
        let args = HashMap::new();
        let result = manager.execute("nonexistent", args, ".".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_output_config() {
        let mut manager = create_test_manager();
        let config = OutputInjectionConfig {
            inject_stdout: false,
            ..Default::default()
        };
        manager.set_output_config(config);
        assert!(!manager.output_config().inject_stdout);
    }
}
