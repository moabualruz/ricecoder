use crate::error::{CommandError, Result};
use crate::types::CommandDefinition;
use std::collections::HashMap;

/// Registry for managing custom commands
#[derive(Debug, Clone)]
pub struct CommandRegistry {
    commands: HashMap<String, CommandDefinition>,
}

impl CommandRegistry {
    /// Create a new empty command registry
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    /// Register a command
    pub fn register(&mut self, command: CommandDefinition) -> Result<()> {
        if command.id.is_empty() {
            return Err(CommandError::InvalidCommandName(
                "Command ID cannot be empty".to_string(),
            ));
        }

        if self.commands.contains_key(&command.id) {
            return Err(CommandError::InvalidCommandName(format!(
                "Command already registered: {}",
                command.id
            )));
        }

        self.commands.insert(command.id.clone(), command);
        Ok(())
    }

    /// Unregister a command
    pub fn unregister(&mut self, command_id: &str) -> Result<()> {
        self.commands
            .remove(command_id)
            .ok_or_else(|| CommandError::CommandNotFound(command_id.to_string()))?;
        Ok(())
    }

    /// Get a command by ID
    pub fn get(&self, command_id: &str) -> Result<CommandDefinition> {
        self.commands
            .get(command_id)
            .cloned()
            .ok_or_else(|| CommandError::CommandNotFound(command_id.to_string()))
    }

    /// Get all commands
    pub fn list_all(&self) -> Vec<CommandDefinition> {
        self.commands.values().cloned().collect()
    }

    /// Get all enabled commands
    pub fn list_enabled(&self) -> Vec<CommandDefinition> {
        self.commands
            .values()
            .filter(|cmd| cmd.enabled)
            .cloned()
            .collect()
    }

    /// Get commands by tag
    pub fn find_by_tag(&self, tag: &str) -> Vec<CommandDefinition> {
        self.commands
            .values()
            .filter(|cmd| cmd.tags.contains(&tag.to_string()))
            .cloned()
            .collect()
    }

    /// Search commands by name or description
    pub fn search(&self, query: &str) -> Vec<CommandDefinition> {
        let query_lower = query.to_lowercase();
        self.commands
            .values()
            .filter(|cmd| {
                cmd.name.to_lowercase().contains(&query_lower)
                    || cmd.description.to_lowercase().contains(&query_lower)
            })
            .cloned()
            .collect()
    }

    /// Enable a command
    pub fn enable(&mut self, command_id: &str) -> Result<()> {
        let command = self
            .commands
            .get_mut(command_id)
            .ok_or_else(|| CommandError::CommandNotFound(command_id.to_string()))?;
        command.enabled = true;
        Ok(())
    }

    /// Disable a command
    pub fn disable(&mut self, command_id: &str) -> Result<()> {
        let command = self
            .commands
            .get_mut(command_id)
            .ok_or_else(|| CommandError::CommandNotFound(command_id.to_string()))?;
        command.enabled = false;
        Ok(())
    }

    /// Check if a command exists
    pub fn exists(&self, command_id: &str) -> bool {
        self.commands.contains_key(command_id)
    }

    /// Get the number of registered commands
    pub fn count(&self) -> usize {
        self.commands.len()
    }

    /// Clear all commands
    pub fn clear(&mut self) {
        self.commands.clear();
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

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
