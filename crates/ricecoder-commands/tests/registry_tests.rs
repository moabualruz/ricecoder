use ricecoder_commands::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_command() {
        let mut registry = CommandRegistry::new();
        let cmd = CommandDefinition::new("test", "Test", "echo test");
        assert!(registry.register(cmd).is_ok());
        assert_eq!(registry.count(), 1);
    }

    #[test]
    fn test_builtin_slash_commands() {
        let registry = CommandRegistry::default();
        // Should have all the built-in slash commands
        assert!(registry.exists("/help"));
        assert!(registry.exists("/new"));
        assert!(registry.exists("/exit"));
        assert!(registry.exists("/undo"));
        assert!(registry.exists("/sessions"));

        // Check that they have the correct tags
        let help_cmd = registry.get("/help").unwrap();
        assert!(help_cmd.tags.contains(&"slash-command".to_string()));
        assert!(help_cmd.tags.contains(&"utility".to_string()));

        let new_cmd = registry.get("/new").unwrap();
        assert!(new_cmd.tags.contains(&"slash-command".to_string()));
        assert!(new_cmd.tags.contains(&"session".to_string()));
    }

    #[test]
    fn test_register_duplicate_command() {
        let mut registry = CommandRegistry::new();
        let cmd1 = CommandDefinition::new("test", "Test", "echo test");
        let cmd2 = CommandDefinition::new("test", "Test", "echo test");
        assert!(registry.register(cmd1).is_ok());
        assert!(registry.register(cmd2).is_err());
    }

    #[test]
    fn test_get_command() {
        let mut registry = CommandRegistry::new();
        let cmd = CommandDefinition::new("test", "Test", "echo test");
        registry.register(cmd).unwrap();
        let retrieved = registry.get("test").unwrap();
        assert_eq!(retrieved.id, "test");
    }

    #[test]
    fn test_get_nonexistent_command() {
        let registry = CommandRegistry::new();
        assert!(registry.get("nonexistent").is_err());
    }

    #[test]
    fn test_unregister_command() {
        let mut registry = CommandRegistry::new();
        let cmd = CommandDefinition::new("test", "Test", "echo test");
        registry.register(cmd).unwrap();
        assert!(registry.unregister("test").is_ok());
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_list_all_commands() {
        let mut registry = CommandRegistry::new();
        registry
            .register(CommandDefinition::new("cmd1", "Cmd1", "echo 1"))
            .ok();
        registry
            .register(CommandDefinition::new("cmd2", "Cmd2", "echo 2"))
            .ok();
        assert_eq!(registry.list_all().len(), 2);
    }

    #[test]
    fn test_list_enabled_commands() {
        let mut registry = CommandRegistry::new();
        let cmd1 = CommandDefinition::new("cmd1", "Cmd1", "echo 1");
        let mut cmd2 = CommandDefinition::new("cmd2", "Cmd2", "echo 2");
        cmd2.enabled = false;
        registry.register(cmd1).ok();
        registry.register(cmd2).ok();
        assert_eq!(registry.list_enabled().len(), 1);
    }

    #[test]
    fn test_find_by_tag() {
        let mut registry = CommandRegistry::new();
        let cmd1 = CommandDefinition::new("cmd1", "Cmd1", "echo 1").with_tag("test");
        let cmd2 = CommandDefinition::new("cmd2", "Cmd2", "echo 2").with_tag("prod");
        registry.register(cmd1).ok();
        registry.register(cmd2).ok();
        assert_eq!(registry.find_by_tag("test").len(), 1);
    }

    #[test]
    fn test_search_commands() {
        let mut registry = CommandRegistry::new();
        let cmd1 = CommandDefinition::new("cmd1", "Test Command", "echo 1");
        let cmd2 = CommandDefinition::new("cmd2", "Other", "echo 2");
        registry.register(cmd1).ok();
        registry.register(cmd2).ok();
        assert_eq!(registry.search("test").len(), 1);
    }

    #[test]
    fn test_enable_disable() {
        let mut registry = CommandRegistry::new();
        let cmd = CommandDefinition::new("test", "Test", "echo test");
        registry.register(cmd).ok();
        registry.disable("test").ok();
        assert!(!registry.get("test").unwrap().enabled);
        registry.enable("test").ok();
        assert!(registry.get("test").unwrap().enabled);
    }
}