// Custom commands module - CLI integration for ricecoder-commands
// Integrates the ricecoder-commands crate with the CLI router and storage

use super::custom_storage::CustomCommandsStorage;
use super::Command;
use crate::error::{CliError, CliResult};
use ricecoder_commands::{CommandManager, CommandRegistry, ConfigManager};
use std::collections::HashMap;
use std::path::PathBuf;

/// Action for custom command handler
#[derive(Debug, Clone)]
pub enum CustomAction {
    /// List all available custom commands
    List,
    /// Show info for a specific command
    Info(String),
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
    manager: CommandManager,
    storage: CustomCommandsStorage,
}

impl CustomCommandHandler {
    /// Create a new custom command handler
    pub fn new(action: CustomAction) -> Self {
        // Load commands from storage
        let storage = CustomCommandsStorage::default();
        let registry = storage
            .load_all()
            .unwrap_or_else(|_| CommandRegistry::new());
        let manager = CommandManager::new(registry);

        Self {
            action,
            manager,
            storage,
        }
    }

    /// Create a handler with a pre-populated manager
    pub fn with_manager(action: CustomAction, manager: CommandManager) -> Self {
        let storage = CustomCommandsStorage::default();
        Self {
            action,
            manager,
            storage,
        }
    }
}

impl Command for CustomCommandHandler {
    fn execute(&self) -> CliResult<()> {
        match &self.action {
            CustomAction::List => self.handle_list(),
            CustomAction::Info(name) => self.handle_info(name),
            CustomAction::Run(name, args) => self.handle_run(name, args),
            CustomAction::Load(file) => self.handle_load(file),
            CustomAction::Search(query) => self.handle_search(query),
        }
    }
}

impl CustomCommandHandler {
    /// Handle list action - display all available commands
    fn handle_list(&self) -> CliResult<()> {
        let commands = self.manager.list_commands();

        if commands.is_empty() {
            println!("No custom commands available.");
            println!("Use 'rice custom load <file>' to load commands from a file.");
            return Ok(());
        }

        println!("Available Commands:");
        println!("==================\n");

        for cmd in commands {
            println!("{:<20} {}", cmd.name, cmd.description);
        }

        Ok(())
    }

    /// Handle info action - show info for a specific command
    fn handle_info(&self, name: &str) -> CliResult<()> {
        let cmd = self
            .manager
            .get_command(name)
            .map_err(|e| CliError::Internal(e.to_string()))?;

        println!("Command: {}", cmd.name);
        println!("Description: {}", cmd.description);
        println!("Command: {}", cmd.command);

        if !cmd.arguments.is_empty() {
            println!("\nArguments:");
            for arg in &cmd.arguments {
                let required = if arg.required { "required" } else { "optional" };
                println!("  {} ({}): {}", arg.name, required, arg.description);
                if let Some(default) = &arg.default {
                    println!("    Default: {}", default);
                }
            }
        }

        Ok(())
    }

    /// Handle run action - execute a custom command
    fn handle_run(&self, name: &str, args: &[String]) -> CliResult<()> {
        // Get the command from registry
        let cmd = self
            .manager
            .get_command(name)
            .map_err(|e| CliError::Internal(e.to_string()))?;

        // Build arguments map from positional arguments
        let mut arguments = HashMap::new();
        for (i, arg) in args.iter().enumerate() {
            if i < cmd.arguments.len() {
                arguments.insert(cmd.arguments[i].name.clone(), arg.clone());
            }
        }

        // Execute the command
        let cwd = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .to_string_lossy()
            .to_string();

        let result = self
            .manager
            .execute(name, arguments, cwd)
            .map_err(|e| CliError::Internal(e.to_string()))?;

        // Display the result
        if result.success {
            println!("{}", result.stdout);
        } else {
            eprintln!("{}", result.stderr);
            return Err(CliError::Internal(format!(
                "Command failed with exit code {}",
                result.exit_code
            )));
        }

        Ok(())
    }

    /// Handle load action - load commands from a file
    fn handle_load(&self, file: &str) -> CliResult<()> {
        let registry =
            ConfigManager::load_from_file(file).map_err(|e| CliError::Internal(e.to_string()))?;

        let commands = registry.list_all();

        if commands.is_empty() {
            println!("No commands found in file: {}", file);
            return Ok(());
        }

        println!("Loaded {} command(s) from {}", commands.len(), file);

        // Save each command to storage
        for cmd in commands {
            println!("  - {}: {}", cmd.name, cmd.description);

            match self.storage.save_command(&cmd) {
                Ok(path) => {
                    println!("    Saved to: {}", path.display());
                }
                Err(e) => {
                    eprintln!("    Warning: Failed to save command: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Handle search action - search for commands
    fn handle_search(&self, query: &str) -> CliResult<()> {
        let results = self.manager.search_commands(query);

        if results.is_empty() {
            println!("No commands found matching '{}'", query);
            return Ok(());
        }

        println!("Search results for '{}':", query);
        println!("========================\n");

        for cmd in results {
            println!("{:<20} {}", cmd.name, cmd.description);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_handler_list_empty() {
        let handler = CustomCommandHandler::new(CustomAction::List);
        // Should not panic and should handle empty registry gracefully
        let result = handler.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_custom_handler_info_not_found() {
        let handler = CustomCommandHandler::new(CustomAction::Info("nonexistent".to_string()));
        let result = handler.execute();
        assert!(result.is_err());
    }

    #[test]
    fn test_custom_handler_run_not_found() {
        let handler =
            CustomCommandHandler::new(CustomAction::Run("nonexistent".to_string(), vec![]));
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
