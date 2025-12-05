//! Permission prompt module

pub mod decision;

pub use decision::{PromptResult, UserDecision};

use std::io::{self, Write};

/// Permission prompt for user interaction
#[derive(Debug, Clone)]
pub struct PermissionPrompt {
    /// Tool name
    pub tool_name: String,
    /// Tool description
    pub tool_description: Option<String>,
    /// What the tool will do
    pub action_description: Option<String>,
    /// Timeout duration in seconds (default 30)
    pub timeout_seconds: u64,
}

impl PermissionPrompt {
    /// Create a new permission prompt
    pub fn new(tool_name: String) -> Self {
        Self {
            tool_name,
            tool_description: None,
            action_description: None,
            timeout_seconds: 30,
        }
    }

    /// Set the tool description
    pub fn with_description(mut self, description: String) -> Self {
        self.tool_description = Some(description);
        self
    }

    /// Set the action description
    pub fn with_action(mut self, action: String) -> Self {
        self.action_description = Some(action);
        self
    }

    /// Set the timeout duration
    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = seconds;
        self
    }

    /// Display the permission prompt to the user
    pub fn display_prompt(&self) -> io::Result<()> {
        println!("\n{}", "=".repeat(60));
        println!("🔒 Permission Required");
        println!("{}", "=".repeat(60));

        // Display tool name
        println!("\n📋 Tool: {}", self.tool_name);

        // Display tool description if available
        if let Some(desc) = &self.tool_description {
            println!("   Description: {}", desc);
        }

        // Display action description if available
        if let Some(action) = &self.action_description {
            println!("\n⚙️  Action: {}", action);
        }

        println!("\n{}", "-".repeat(60));
        println!("Allow this action? (yes/y/approve, no/n/deny, cancel/c)");
        println!("Timeout: {} seconds", self.timeout_seconds);
        println!("{}", "-".repeat(60));

        Ok(())
    }

    /// Display what the tool will do
    pub fn display_action(&self) -> io::Result<()> {
        if let Some(action) = &self.action_description {
            println!("\n📝 This tool will:");
            println!("   {}", action);
        }
        Ok(())
    }

    /// Collect user decision from stdin
    #[allow(clippy::only_used_in_recursion)]
    pub fn collect_decision(&self) -> io::Result<UserDecision> {
        print!("\nYour decision: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let decision = match input.trim().to_lowercase().as_str() {
            "yes" | "y" | "approve" => UserDecision::Approved,
            "no" | "n" | "deny" => UserDecision::Denied,
            "cancel" | "c" => UserDecision::Cancelled,
            _ => {
                println!("Invalid input. Please enter: yes/y/approve, no/n/deny, or cancel/c");
                return self.collect_decision();
            }
        };

        Ok(decision)
    }

    /// Execute the full prompt flow and return the result
    pub fn execute(&self) -> io::Result<PromptResult> {
        self.display_prompt()?;
        self.display_action()?;

        let decision = self.collect_decision()?;

        let result = decision.to_prompt_result();

        Ok(result)
    }
}

impl Default for PermissionPrompt {
    fn default() -> Self {
        Self::new("unknown".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_prompt_creation() {
        let prompt = PermissionPrompt::new("test_tool".to_string());
        assert_eq!(prompt.tool_name, "test_tool");
        assert_eq!(prompt.tool_description, None);
        assert_eq!(prompt.action_description, None);
        assert_eq!(prompt.timeout_seconds, 30);
    }

    #[test]
    fn test_permission_prompt_with_description() {
        let prompt = PermissionPrompt::new("test_tool".to_string())
            .with_description("A test tool".to_string());
        assert_eq!(prompt.tool_description, Some("A test tool".to_string()));
    }

    #[test]
    fn test_permission_prompt_with_action() {
        let prompt = PermissionPrompt::new("test_tool".to_string())
            .with_action("Will execute test".to_string());
        assert_eq!(prompt.action_description, Some("Will execute test".to_string()));
    }

    #[test]
    fn test_permission_prompt_with_timeout() {
        let prompt = PermissionPrompt::new("test_tool".to_string()).with_timeout(60);
        assert_eq!(prompt.timeout_seconds, 60);
    }

    #[test]
    fn test_permission_prompt_default() {
        let prompt = PermissionPrompt::default();
        assert_eq!(prompt.tool_name, "unknown");
    }

    #[test]
    fn test_permission_prompt_builder_chain() {
        let prompt = PermissionPrompt::new("my_tool".to_string())
            .with_description("My tool description".to_string())
            .with_action("Will do something".to_string())
            .with_timeout(45);

        assert_eq!(prompt.tool_name, "my_tool");
        assert_eq!(prompt.tool_description, Some("My tool description".to_string()));
        assert_eq!(prompt.action_description, Some("Will do something".to_string()));
        assert_eq!(prompt.timeout_seconds, 45);
    }
}
