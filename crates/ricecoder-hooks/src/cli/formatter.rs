//! Output formatting for hook commands

use crate::error::{HooksError, Result};
use crate::types::{Action, Hook};

/// Format a single hook as a table
pub fn format_hook_table(hook: &Hook) -> String {
    let status = if hook.enabled {
        "✓ Enabled"
    } else {
        "✗ Disabled"
    };
    let action_type = match &hook.action {
        Action::Command(_) => "Command",
        Action::ToolCall(_) => "Tool Call",
        Action::AiPrompt(_) => "AI Prompt",
        Action::Chain(_) => "Chain",
    };

    let mut output = String::new();
    output.push_str(&format!("ID:          {}\n", hook.id));
    output.push_str(&format!("Name:        {}\n", hook.name));
    if let Some(desc) = &hook.description {
        output.push_str(&format!("Description: {}\n", desc));
    }
    output.push_str(&format!("Event:       {}\n", hook.event));
    output.push_str(&format!("Action:      {}\n", action_type));
    output.push_str(&format!("Status:      {}\n", status));
    if !hook.tags.is_empty() {
        output.push_str(&format!("Tags:        {}\n", hook.tags.join(", ")));
    }

    output
}

/// Format multiple hooks as a table
pub fn format_hooks_table(hooks: &[Hook]) -> String {
    if hooks.is_empty() {
        return "No hooks found".to_string();
    }

    let mut output = String::new();
    output.push_str("ID                                   | Name                     | Event              | Status   | Action\n");
    output.push_str("-------------------------------------|--------------------------|--------------------|---------|-----------\n");

    for hook in hooks {
        let status = if hook.enabled { "Enabled" } else { "Disabled" };
        let action_type = match &hook.action {
            Action::Command(_) => "Command",
            Action::ToolCall(_) => "Tool Call",
            Action::AiPrompt(_) => "AI Prompt",
            Action::Chain(_) => "Chain",
        };

        let id = if hook.id.len() > 36 {
            format!("{}...", &hook.id[..33])
        } else {
            hook.id.clone()
        };

        let name = if hook.name.len() > 24 {
            format!("{}...", &hook.name[..21])
        } else {
            hook.name.clone()
        };

        let event = if hook.event.len() > 18 {
            format!("{}...", &hook.event[..15])
        } else {
            hook.event.clone()
        };

        output.push_str(&format!(
            "{:<36} | {:<24} | {:<18} | {:<8} | {}\n",
            id, name, event, status, action_type
        ));
    }

    output
}

/// Format a single hook as JSON
pub fn format_hook_json(hook: &Hook) -> Result<String> {
    serde_json::to_string_pretty(hook)
        .map_err(|e| HooksError::InvalidConfiguration(format!("Failed to serialize hook: {}", e)))
}

/// Format multiple hooks as JSON
pub fn format_hooks_json(hooks: &[Hook]) -> Result<String> {
    serde_json::to_string_pretty(hooks)
        .map_err(|e| HooksError::InvalidConfiguration(format!("Failed to serialize hooks: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::CommandAction;

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
    fn test_format_hook_table() {
        let hook = create_test_hook("hook1", "Test Hook");
        let output = format_hook_table(&hook);

        assert!(output.contains("hook1"));
        assert!(output.contains("Test Hook"));
        assert!(output.contains("test_event"));
        assert!(output.contains("Enabled"));
        assert!(output.contains("Command"));
    }

    #[test]
    fn test_format_hooks_table_empty() {
        let hooks: Vec<Hook> = vec![];
        let output = format_hooks_table(&hooks);

        assert_eq!(output, "No hooks found");
    }

    #[test]
    fn test_format_hooks_table() {
        let hook1 = create_test_hook("hook1", "Hook 1");
        let hook2 = create_test_hook("hook2", "Hook 2");
        let hooks = vec![hook1, hook2];

        let output = format_hooks_table(&hooks);

        assert!(output.contains("hook1"));
        assert!(output.contains("hook2"));
        assert!(output.contains("Hook 1"));
        assert!(output.contains("Hook 2"));
    }

    #[test]
    fn test_format_hook_json() {
        let hook = create_test_hook("hook1", "Test Hook");
        let json = format_hook_json(&hook).unwrap();

        assert!(json.contains("\"id\":\"hook1\"") || json.contains("\"id\": \"hook1\""));
        assert!(json.contains("Test Hook"));
    }

    #[test]
    fn test_format_hooks_json() {
        let hook1 = create_test_hook("hook1", "Hook 1");
        let hook2 = create_test_hook("hook2", "Hook 2");
        let hooks = vec![hook1, hook2];

        let json = format_hooks_json(&hooks).unwrap();

        assert!(json.contains("hook1"));
        assert!(json.contains("hook2"));
    }

    #[test]
    fn test_format_hook_table_disabled() {
        let mut hook = create_test_hook("hook1", "Test Hook");
        hook.enabled = false;
        let output = format_hook_table(&hook);

        assert!(output.contains("Disabled"));
    }

    #[test]
    fn test_format_hooks_table_truncation() {
        let mut hook = create_test_hook("a".repeat(50).as_str(), "b".repeat(50).as_str());
        hook.event = "c".repeat(50);
        let output = format_hooks_table(&[hook]);

        // Should contain truncated versions
        assert!(output.contains("..."));
    }
}
