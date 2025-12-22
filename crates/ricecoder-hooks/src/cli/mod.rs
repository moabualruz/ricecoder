//! CLI commands for hook management
//!
//! This module provides command-line interface commands for managing hooks,
//! including listing, inspecting, enabling, disabling, and deleting hooks.

pub mod commands;
pub mod formatter;

pub use commands::{delete_hook, disable_hook, enable_hook, inspect_hook, list_hooks, HookCommand};
pub use formatter::{format_hook_json, format_hook_table, format_hooks_json, format_hooks_table};

use crate::{error::Result, registry::HookRegistry};

/// Hook management CLI interface
pub struct HookCli<R: HookRegistry> {
    registry: R,
}

impl<R: HookRegistry> HookCli<R> {
    /// Create a new hook CLI instance
    pub fn new(registry: R) -> Self {
        Self { registry }
    }

    /// Execute a hook command
    pub fn execute(&mut self, command: HookCommand) -> Result<String> {
        match command {
            HookCommand::List { format } => {
                let hooks = self.registry.list_hooks()?;
                Ok(match format.as_deref() {
                    Some("json") => format_hooks_json(&hooks)?,
                    _ => format_hooks_table(&hooks),
                })
            }
            HookCommand::Inspect { id, format } => {
                let hook = self.registry.get_hook(&id)?;
                Ok(match format.as_deref() {
                    Some("json") => format_hook_json(&hook)?,
                    _ => format_hook_table(&hook),
                })
            }
            HookCommand::Enable { id } => {
                self.registry.enable_hook(&id)?;
                Ok(format!("Hook '{}' enabled", id))
            }
            HookCommand::Disable { id } => {
                self.registry.disable_hook(&id)?;
                Ok(format!("Hook '{}' disabled", id))
            }
            HookCommand::Delete { id } => {
                self.registry.unregister_hook(&id)?;
                Ok(format!("Hook '{}' deleted", id))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        registry::InMemoryHookRegistry,
        types::{Action, CommandAction, Hook},
    };

    fn create_test_hook(id: &str, name: &str) -> Hook {
        Hook {
            id: id.to_string(),
            name: name.to_string(),
            description: Some("Test hook".to_string()),
            event: "test_event".to_string(),
            action: Action::Command(CommandAction {
                command: "echo".to_string(),
                args: vec!["test".to_string()],
                timeout_ms: Some(5000),
                capture_output: true,
            }),
            enabled: true,
            tags: vec!["test".to_string()],
            metadata: serde_json::json!({}),
            condition: None,
        }
    }

    #[test]
    fn test_list_hooks() {
        let mut registry = InMemoryHookRegistry::new();
        let hook1 = create_test_hook("hook1", "Hook 1");
        let hook2 = create_test_hook("hook2", "Hook 2");

        registry.register_hook(hook1).unwrap();
        registry.register_hook(hook2).unwrap();

        let mut cli = HookCli::new(registry);
        let result = cli.execute(HookCommand::List { format: None }).unwrap();

        assert!(result.contains("Hook 1"));
        assert!(result.contains("Hook 2"));
    }

    #[test]
    fn test_inspect_hook() {
        let mut registry = InMemoryHookRegistry::new();
        let hook = create_test_hook("hook1", "Hook 1");
        registry.register_hook(hook).unwrap();

        let mut cli = HookCli::new(registry);
        let result = cli
            .execute(HookCommand::Inspect {
                id: "hook1".to_string(),
                format: None,
            })
            .unwrap();

        assert!(result.contains("Hook 1"));
    }

    #[test]
    fn test_enable_hook() {
        let mut registry = InMemoryHookRegistry::new();
        let mut hook = create_test_hook("hook1", "Hook 1");
        hook.enabled = false;
        registry.register_hook(hook).unwrap();

        let mut cli = HookCli::new(registry);
        let result = cli
            .execute(HookCommand::Enable {
                id: "hook1".to_string(),
            })
            .unwrap();

        assert!(result.contains("enabled"));
    }

    #[test]
    fn test_disable_hook() {
        let mut registry = InMemoryHookRegistry::new();
        let hook = create_test_hook("hook1", "Hook 1");
        registry.register_hook(hook).unwrap();

        let mut cli = HookCli::new(registry);
        let result = cli
            .execute(HookCommand::Disable {
                id: "hook1".to_string(),
            })
            .unwrap();

        assert!(result.contains("disabled"));
    }

    #[test]
    fn test_delete_hook() {
        let mut registry = InMemoryHookRegistry::new();
        let hook = create_test_hook("hook1", "Hook 1");
        registry.register_hook(hook).unwrap();

        let mut cli = HookCli::new(registry);
        let result = cli
            .execute(HookCommand::Delete {
                id: "hook1".to_string(),
            })
            .unwrap();

        assert!(result.contains("deleted"));
    }
}
