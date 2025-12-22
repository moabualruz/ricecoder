use std::collections::HashMap;

use ricecoder_commands::*;

#[cfg(test)]
mod tests {
    use super::*;

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
