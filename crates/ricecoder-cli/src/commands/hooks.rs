//! Hooks command handler

use ricecoder_hooks::{HookCli, HookCommand, InMemoryHookRegistry};

use crate::error::{CliError, CliResult};

/// Hooks command action
#[derive(Debug, Clone)]
pub enum HooksAction {
    /// List all hooks
    List {
        /// Output format (table or json)
        format: Option<String>,
    },

    /// Inspect a specific hook
    Inspect {
        /// Hook ID
        id: String,

        /// Output format (table or json)
        format: Option<String>,
    },

    /// Enable a hook
    Enable {
        /// Hook ID
        id: String,
    },

    /// Disable a hook
    Disable {
        /// Hook ID
        id: String,
    },

    /// Delete a hook
    Delete {
        /// Hook ID
        id: String,
    },
}

/// Hooks command handler
pub struct HooksCommand {
    action: HooksAction,
}

impl HooksCommand {
    /// Create a new hooks command
    pub fn new(action: HooksAction) -> Self {
        Self { action }
    }

    /// Execute the hooks command
    pub fn execute(&self) -> CliResult<()> {
        // Create an in-memory registry for now
        // In the future, this should load from configuration
        let registry = InMemoryHookRegistry::new();
        let mut cli = HookCli::new(registry);

        let command = match &self.action {
            HooksAction::List { format } => HookCommand::List {
                format: format.clone(),
            },
            HooksAction::Inspect { id, format } => HookCommand::Inspect {
                id: id.clone(),
                format: format.clone(),
            },
            HooksAction::Enable { id } => HookCommand::Enable { id: id.clone() },
            HooksAction::Disable { id } => HookCommand::Disable { id: id.clone() },
            HooksAction::Delete { id } => HookCommand::Delete { id: id.clone() },
        };

        let result = cli
            .execute(command)
            .map_err(|e| CliError::Internal(format!("Hook command failed: {}", e)))?;

        println!("{}", result);
        Ok(())
    }
}
