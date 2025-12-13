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

impl CommandRegistry {
    /// Create a registry with built-in slash commands
    pub fn with_builtin_commands() -> Self {
        let mut registry = Self::new();

        // Session management commands
        let _ = registry.register(
            CommandDefinition::new("/new", "Create New Session", "")
                .with_description("Create a new chat session")
                .with_tag("slash-command")
                .with_tag("session")
                .with_enabled(true),
        );

        let _ = registry.register(
            CommandDefinition::new("/sessions", "List Sessions", "")
                .with_description("List and switch between chat sessions")
                .with_tag("slash-command")
                .with_tag("session")
                .with_enabled(true),
        );

        let _ = registry.register(
            CommandDefinition::new("/rename", "Rename Session", "")
                .with_description("Rename the current session")
                .with_tag("slash-command")
                .with_tag("session")
                .with_enabled(true),
        );

        let _ = registry.register(
            CommandDefinition::new("/delete", "Delete Session", "")
                .with_description("Delete the current session")
                .with_tag("slash-command")
                .with_tag("session")
                .with_enabled(true),
        );

        let _ = registry.register(
            CommandDefinition::new("/clear", "Clear Session", "")
                .with_description("Clear all messages from current session")
                .with_tag("slash-command")
                .with_tag("session")
                .with_enabled(true),
        );

        // Navigation and utility commands
        let _ = registry.register(
            CommandDefinition::new("/help", "Show Help", "")
                .with_description("Show help dialog with commands and shortcuts")
                .with_tag("slash-command")
                .with_tag("utility")
                .with_enabled(true),
        );

        let _ = registry.register(
            CommandDefinition::new("/models", "List Models", "")
                .with_description("List and select available AI models")
                .with_tag("slash-command")
                .with_tag("utility")
                .with_enabled(true),
        );

        let _ = registry.register(
            CommandDefinition::new("/themes", "List Themes", "")
                .with_description("List and select available themes")
                .with_tag("slash-command")
                .with_tag("utility")
                .with_enabled(true),
        );

        let _ = registry.register(
            CommandDefinition::new("/settings", "Open Settings", "")
                .with_description("Open settings interface")
                .with_tag("slash-command")
                .with_tag("utility")
                .with_enabled(true),
        );

        let _ = registry.register(
            CommandDefinition::new("/exit", "Exit Application", "")
                .with_description("Exit RiceCoder")
                .with_tag("slash-command")
                .with_tag("utility")
                .with_enabled(true),
        );

        let _ = registry.register(
            CommandDefinition::new("/quit", "Quit Application", "")
                .with_description("Exit RiceCoder (alias for /exit)")
                .with_tag("slash-command")
                .with_tag("utility")
                .with_enabled(true),
        );

        let _ = registry.register(
            CommandDefinition::new("/undo", "Undo Last Action", "")
                .with_description("Undo the last message or action")
                .with_tag("slash-command")
                .with_tag("utility")
                .with_enabled(true),
        );

        let _ = registry.register(
            CommandDefinition::new("/redo", "Redo Last Action", "")
                .with_description("Redo the last undone action")
                .with_tag("slash-command")
                .with_tag("utility")
                .with_enabled(true),
        );

        let _ = registry.register(
            CommandDefinition::new("/compact", "Compact Session", "")
                .with_description("Compact current session to reduce token usage")
                .with_tag("slash-command")
                .with_tag("utility")
                .with_enabled(true),
        );

        let _ = registry.register(
            CommandDefinition::new("/export", "Export Session", "")
                .with_description("Export session to Markdown file")
                .with_tag("slash-command")
                .with_tag("utility")
                .with_enabled(true),
        );

        let _ = registry.register(
            CommandDefinition::new("/copy", "Copy Last Message", "")
                .with_description("Copy the last message to clipboard")
                .with_tag("slash-command")
                .with_tag("utility")
                .with_enabled(true),
        );

        let _ = registry.register(
            CommandDefinition::new("/details", "Toggle Details", "")
                .with_description("Toggle tool execution details visibility")
                .with_tag("slash-command")
                .with_tag("utility")
                .with_enabled(true),
        );

        let _ = registry.register(
            CommandDefinition::new("/debug", "Toggle Debug Mode", "")
                .with_description("Toggle debug mode")
                .with_tag("slash-command")
                .with_tag("utility")
                .with_enabled(true),
        );

        registry
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::with_builtin_commands()
    }
}
