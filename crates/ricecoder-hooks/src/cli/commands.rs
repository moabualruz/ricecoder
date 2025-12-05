//! Hook management commands

/// Hook management commands
#[derive(Debug, Clone)]
pub enum HookCommand {
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

/// List all hooks
pub fn list_hooks() -> HookCommand {
    HookCommand::List { format: None }
}

/// List all hooks with JSON format
pub fn list_hooks_json() -> HookCommand {
    HookCommand::List {
        format: Some("json".to_string()),
    }
}

/// Inspect a hook
pub fn inspect_hook(id: impl Into<String>) -> HookCommand {
    HookCommand::Inspect {
        id: id.into(),
        format: None,
    }
}

/// Inspect a hook with JSON format
pub fn inspect_hook_json(id: impl Into<String>) -> HookCommand {
    HookCommand::Inspect {
        id: id.into(),
        format: Some("json".to_string()),
    }
}

/// Enable a hook
pub fn enable_hook(id: impl Into<String>) -> HookCommand {
    HookCommand::Enable { id: id.into() }
}

/// Disable a hook
pub fn disable_hook(id: impl Into<String>) -> HookCommand {
    HookCommand::Disable { id: id.into() }
}

/// Delete a hook
pub fn delete_hook(id: impl Into<String>) -> HookCommand {
    HookCommand::Delete { id: id.into() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_hooks_command() {
        let cmd = list_hooks();
        match cmd {
            HookCommand::List { format } => {
                assert!(format.is_none());
            }
            _ => panic!("Expected List command"),
        }
    }

    #[test]
    fn test_list_hooks_json_command() {
        let cmd = list_hooks_json();
        match cmd {
            HookCommand::List { format } => {
                assert_eq!(format, Some("json".to_string()));
            }
            _ => panic!("Expected List command"),
        }
    }

    #[test]
    fn test_inspect_hook_command() {
        let cmd = inspect_hook("hook1");
        match cmd {
            HookCommand::Inspect { id, format } => {
                assert_eq!(id, "hook1");
                assert!(format.is_none());
            }
            _ => panic!("Expected Inspect command"),
        }
    }

    #[test]
    fn test_enable_hook_command() {
        let cmd = enable_hook("hook1");
        match cmd {
            HookCommand::Enable { id } => {
                assert_eq!(id, "hook1");
            }
            _ => panic!("Expected Enable command"),
        }
    }

    #[test]
    fn test_disable_hook_command() {
        let cmd = disable_hook("hook1");
        match cmd {
            HookCommand::Disable { id } => {
                assert_eq!(id, "hook1");
            }
            _ => panic!("Expected Disable command"),
        }
    }

    #[test]
    fn test_delete_hook_command() {
        let cmd = delete_hook("hook1");
        match cmd {
            HookCommand::Delete { id } => {
                assert_eq!(id, "hook1");
            }
            _ => panic!("Expected Delete command"),
        }
    }
}
