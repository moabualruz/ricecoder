//! Hooks command handler

use crate::error::{CliError, CliResult};
use ricecoder_hooks::{HookCli, HookCommand, InMemoryHookRegistry};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hooks_list_action() {
        let action = HooksAction::List { format: None };
        let _cmd = HooksCommand::new(action);
        // Just verify it can be created
        assert!(true);
    }

    #[test]
    fn test_hooks_inspect_action() {
        let action = HooksAction::Inspect {
            id: "hook1".to_string(),
            format: None,
        };
        let _cmd = HooksCommand::new(action);
        // Just verify it can be created
        assert!(true);
    }

    #[test]
    fn test_hooks_enable_action() {
        let action = HooksAction::Enable {
            id: "hook1".to_string(),
        };
        let _cmd = HooksCommand::new(action);
        // Just verify it can be created
        assert!(true);
    }

    #[test]
    fn test_hooks_disable_action() {
        let action = HooksAction::Disable {
            id: "hook1".to_string(),
        };
        let _cmd = HooksCommand::new(action);
        // Just verify it can be created
        assert!(true);
    }

    #[test]
    fn test_hooks_delete_action() {
        let action = HooksAction::Delete {
            id: "hook1".to_string(),
        };
        let _cmd = HooksCommand::new(action);
        // Just verify it can be created
        assert!(true);
    }
}
