use ricecoder_commands::*;
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;

    struct MockCommandExecutor {
        history: Vec<Command>,
    }

    impl MockCommandExecutor {
        fn new() -> Self {
            Self { history: Vec::new() }
        }
    }

    impl CommandExecutor for MockCommandExecutor {
        fn execute(&mut self, command: Command) -> Result<CommandResult, CommandError> {
            self.history.push(command);
            Ok(CommandResult {
                success: true,
                output: "Mock output".to_string(),
                error: None,
            })
        }

        fn get_command_history(&self) -> Vec<&Command> {
            self.history.iter().collect()
        }
    }

    #[test]
    fn test_command_execution() {
        let mut executor = MockCommandExecutor::new();
        let command = Command::Save;

        let result = executor.execute(command).unwrap();
        assert!(result.success);
        assert_eq!(result.output, "Mock output");
    }

    #[test]
    fn test_command_history() {
        let mut executor = MockCommandExecutor::new();

        executor.execute(Command::Save).unwrap();
        executor.execute(Command::Custom("test".to_string())).unwrap();

        let history = executor.get_command_history();
        assert_eq!(history.len(), 2);
        assert!(matches!(history[0], Command::Save));
        assert!(matches!(history[1], Command::Custom(_)));
    }
}

#[cfg(test)]
mod registry_tests {
    use super::*;

    struct MockExecutor;

    #[async_trait::async_trait]
    impl CommandExecutor for MockExecutor {
        async fn execute(
            &self,
            command: &str,
            parameters: HashMap<String, String>,
            _context: &CommandContext,
        ) -> CommandResult<CommandExecutionResult> {
            Ok(CommandExecutionResult {
                command: command.to_string(),
                parameters,
                status: ExecutionStatus::Success,
                output: Some("Command executed successfully".to_string()),
                error: None,
                execution_time_ms: 100,
                executed_at: chrono::Utc::now(),
            })
        }

        fn validate_parameters(
            &self,
            _command: &str,
            _parameters: &HashMap<String, String>,
        ) -> CommandResult<()> {
            Ok(())
        }

        async fn get_autocomplete(
            &self,
            _command: &str,
            _parameter: &str,
            _partial_value: &str,
            _context: &CommandContext,
        ) -> CommandResult<Vec<String>> {
            Ok(vec!["option1".to_string(), "option2".to_string()])
        }
    }

    #[test]
    fn test_parameter_validation() {
        let param = CommandParameter {
            name: "test".to_string(),
            param_type: ParameterType::Integer,
            description: "Test parameter".to_string(),
            required: true,
            default_value: None,
            validation: None,
            autocomplete: None,
        };

        // Valid integer
        assert!(validate_parameter(&param, "42").is_ok());

        // Invalid integer
        assert!(validate_parameter(&param, "not_a_number").is_err());
    }

    #[test]
    fn test_command_registry() {
        let mut registry = CommandRegistry::new();

        let cmd_def = CommandDefinition {
            name: "test".to_string(),
            display_name: "Test Command".to_string(),
            description: "A test command".to_string(),
            category: "Test".to_string(),
            parameters: vec![],
            requires_confirmation: false,
            confirmation_message: None,
            timeout_seconds: None,
            permissions: vec![],
        };

        registry.register_command(cmd_def.clone(), Arc::new(MockExecutor));

        assert!(registry.get_command("test").is_some());
        assert_eq!(registry.list_commands().len(), 1);
    }
}