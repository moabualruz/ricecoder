//! Arguments context for TUI runtime state
//!
//! This module provides context for runtime arguments passed to the TUI,
//! managing command-line arguments and session configuration.
//!
//! **Note**: This is NOT for CLI argument parsing (which uses clap in ricecoder-cli).
//! This is for runtime state management of arguments within the TUI session.

use std::time::SystemTime;

/// Arguments context containing runtime configuration
///
/// Manages runtime arguments for the TUI session including:
/// - Model selection
/// - Agent configuration
/// - Initial prompt
/// - Session continuation
/// - Session identifier
///
/// This context is populated from CLI arguments parsed by ricecoder-cli
/// and passed to the TUI at runtime.
#[derive(Debug, Clone, Default)]
pub struct ArgsContext {
    /// AI model to use for the session
    pub model: Option<String>,
    /// Agent type to use for the session
    pub agent: Option<String>,
    /// Initial prompt to send
    pub prompt: Option<String>,
    /// Whether to continue an existing session
    pub continue_session: bool,
    /// Session ID to continue or create
    pub session_id: Option<String>,
    /// Timestamp when context was created
    pub created_at: SystemTime,
    /// Whether the context is ready for use
    pub ready: bool,
}

impl ArgsContext {
    /// Create a new empty arguments context
    ///
    /// # Examples
    ///
    /// ```
    /// use ricecoder_tui::ArgsContext;
    ///
    /// let ctx = ArgsContext::new();
    /// assert!(!ctx.is_ready());
    /// ```
    pub fn new() -> Self {
        Self {
            model: None,
            agent: None,
            prompt: None,
            continue_session: false,
            session_id: None,
            created_at: SystemTime::now(),
            ready: false,
        }
    }

    /// Create a new arguments context with model selection
    ///
    /// # Arguments
    ///
    /// * `model` - The AI model to use
    ///
    /// # Examples
    ///
    /// ```
    /// use ricecoder_tui::ArgsContext;
    ///
    /// let ctx = ArgsContext::with_model("gpt-4");
    /// assert_eq!(ctx.get_model(), Some("gpt-4"));
    /// ```
    pub fn with_model(model: impl Into<String>) -> Self {
        Self {
            model: Some(model.into()),
            agent: None,
            prompt: None,
            continue_session: false,
            session_id: None,
            created_at: SystemTime::now(),
            ready: false,
        }
    }

    /// Create a new arguments context with agent selection
    ///
    /// # Arguments
    ///
    /// * `agent` - The agent type to use
    ///
    /// # Examples
    ///
    /// ```
    /// use ricecoder_tui::ArgsContext;
    ///
    /// let ctx = ArgsContext::with_agent("build");
    /// assert_eq!(ctx.get_agent(), Some("build"));
    /// ```
    pub fn with_agent(agent: impl Into<String>) -> Self {
        Self {
            model: None,
            agent: Some(agent.into()),
            prompt: None,
            continue_session: false,
            session_id: None,
            created_at: SystemTime::now(),
            ready: false,
        }
    }

    /// Create a new arguments context with session ID
    ///
    /// # Arguments
    ///
    /// * `session_id` - The session ID to continue or create
    ///
    /// # Examples
    ///
    /// ```
    /// use ricecoder_tui::ArgsContext;
    ///
    /// let ctx = ArgsContext::with_session_id("session-123");
    /// assert_eq!(ctx.get_session_id(), Some("session-123"));
    /// ```
    pub fn with_session_id(session_id: impl Into<String>) -> Self {
        Self {
            model: None,
            agent: None,
            prompt: None,
            continue_session: false,
            session_id: Some(session_id.into()),
            created_at: SystemTime::now(),
            ready: false,
        }
    }

    /// Set the model
    ///
    /// # Arguments
    ///
    /// * `model` - The AI model to use
    pub fn set_model(&mut self, model: impl Into<String>) {
        self.model = Some(model.into());
    }

    /// Get the model
    ///
    /// # Returns
    ///
    /// The configured model, or None if not set
    pub fn get_model(&self) -> Option<&str> {
        self.model.as_deref()
    }

    /// Set the agent
    ///
    /// # Arguments
    ///
    /// * `agent` - The agent type to use
    pub fn set_agent(&mut self, agent: impl Into<String>) {
        self.agent = Some(agent.into());
    }

    /// Get the agent
    ///
    /// # Returns
    ///
    /// The configured agent, or None if not set
    pub fn get_agent(&self) -> Option<&str> {
        self.agent.as_deref()
    }

    /// Set the initial prompt
    ///
    /// # Arguments
    ///
    /// * `prompt` - The initial prompt to send
    pub fn set_prompt(&mut self, prompt: impl Into<String>) {
        self.prompt = Some(prompt.into());
    }

    /// Get the initial prompt
    ///
    /// # Returns
    ///
    /// The configured prompt, or None if not set
    pub fn get_prompt(&self) -> Option<&str> {
        self.prompt.as_deref()
    }

    /// Set whether to continue an existing session
    ///
    /// # Arguments
    ///
    /// * `continue_session` - Whether to continue a session
    pub fn set_continue_session(&mut self, continue_session: bool) {
        self.continue_session = continue_session;
    }

    /// Check if continuing an existing session
    ///
    /// # Returns
    ///
    /// True if continuing a session, false otherwise
    pub fn is_continue_session(&self) -> bool {
        self.continue_session
    }

    /// Set the session ID
    ///
    /// # Arguments
    ///
    /// * `session_id` - The session ID to continue or create
    pub fn set_session_id(&mut self, session_id: impl Into<String>) {
        self.session_id = Some(session_id.into());
    }

    /// Get the session ID
    ///
    /// # Returns
    ///
    /// The configured session ID, or None if not set
    pub fn get_session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    /// Clear all arguments
    pub fn clear(&mut self) {
        self.model = None;
        self.agent = None;
        self.prompt = None;
        self.continue_session = false;
        self.session_id = None;
        self.created_at = SystemTime::now();
        self.ready = false;
    }

    /// Check if the context has any arguments configured
    ///
    /// # Returns
    ///
    /// True if any argument is set, false otherwise
    pub fn has_arguments(&self) -> bool {
        self.model.is_some()
            || self.agent.is_some()
            || self.prompt.is_some()
            || self.session_id.is_some()
            || self.continue_session
    }

    /// Check if the context is complete (ready for use)
    ///
    /// # Returns
    ///
    /// True if context has arguments and is ready, false otherwise
    pub fn is_complete(&self) -> bool {
        self.has_arguments()
    }

    /// Mark the context as ready
    pub fn mark_ready(&mut self) {
        self.ready = true;
    }

    /// Mark the context as not ready
    pub fn mark_not_ready(&mut self) {
        self.ready = false;
    }

    /// Check if the context is ready to use
    ///
    /// # Returns
    ///
    /// True if ready, false otherwise
    pub fn is_ready(&self) -> bool {
        self.ready
    }

    /// Get a summary of the context
    ///
    /// # Returns
    ///
    /// A human-readable summary string
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();

        if let Some(model) = &self.model {
            parts.push(format!("Model: {}", model));
        }

        if let Some(agent) = &self.agent {
            parts.push(format!("Agent: {}", agent));
        }

        if let Some(prompt) = &self.prompt {
            let preview = if prompt.len() > 30 {
                format!("{}...", &prompt[..30])
            } else {
                prompt.clone()
            };
            parts.push(format!("Prompt: {}", preview));
        }

        if self.continue_session {
            parts.push("Continue: yes".to_string());
        }

        if let Some(session_id) = &self.session_id {
            parts.push(format!("Session: {}", session_id));
        }

        if parts.is_empty() {
            "No arguments configured".to_string()
        } else {
            parts.join(", ")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_context() {
        let ctx = ArgsContext::new();
        assert!(ctx.model.is_none());
        assert!(ctx.agent.is_none());
        assert!(ctx.prompt.is_none());
        assert!(!ctx.continue_session);
        assert!(ctx.session_id.is_none());
        assert!(!ctx.ready);
        assert!(!ctx.has_arguments());
    }

    #[test]
    fn test_with_model() {
        let ctx = ArgsContext::with_model("gpt-4");
        assert_eq!(ctx.get_model(), Some("gpt-4"));
        assert!(ctx.has_arguments());
    }

    #[test]
    fn test_with_agent() {
        let ctx = ArgsContext::with_agent("build");
        assert_eq!(ctx.get_agent(), Some("build"));
        assert!(ctx.has_arguments());
    }

    #[test]
    fn test_with_session_id() {
        let ctx = ArgsContext::with_session_id("session-123");
        assert_eq!(ctx.get_session_id(), Some("session-123"));
        assert!(ctx.has_arguments());
    }

    #[test]
    fn test_set_and_get_model() {
        let mut ctx = ArgsContext::new();
        ctx.set_model("claude-3");
        assert_eq!(ctx.get_model(), Some("claude-3"));
    }

    #[test]
    fn test_set_and_get_agent() {
        let mut ctx = ArgsContext::new();
        ctx.set_agent("plan");
        assert_eq!(ctx.get_agent(), Some("plan"));
    }

    #[test]
    fn test_set_and_get_prompt() {
        let mut ctx = ArgsContext::new();
        ctx.set_prompt("Help me refactor this code");
        assert_eq!(ctx.get_prompt(), Some("Help me refactor this code"));
    }

    #[test]
    fn test_continue_session() {
        let mut ctx = ArgsContext::new();
        assert!(!ctx.is_continue_session());
        ctx.set_continue_session(true);
        assert!(ctx.is_continue_session());
        assert!(ctx.has_arguments());
    }

    #[test]
    fn test_set_and_get_session_id() {
        let mut ctx = ArgsContext::new();
        ctx.set_session_id("abc-123");
        assert_eq!(ctx.get_session_id(), Some("abc-123"));
    }

    #[test]
    fn test_clear() {
        let mut ctx = ArgsContext::new();
        ctx.set_model("gpt-4");
        ctx.set_agent("build");
        ctx.set_prompt("test");
        ctx.set_continue_session(true);
        ctx.set_session_id("session-1");
        ctx.mark_ready();

        ctx.clear();

        assert!(ctx.model.is_none());
        assert!(ctx.agent.is_none());
        assert!(ctx.prompt.is_none());
        assert!(!ctx.continue_session);
        assert!(ctx.session_id.is_none());
        assert!(!ctx.ready);
        assert!(!ctx.has_arguments());
    }

    #[test]
    fn test_has_arguments() {
        let mut ctx = ArgsContext::new();
        assert!(!ctx.has_arguments());

        ctx.set_model("gpt-4");
        assert!(ctx.has_arguments());

        ctx.clear();
        ctx.set_continue_session(true);
        assert!(ctx.has_arguments());
    }

    #[test]
    fn test_is_complete() {
        let mut ctx = ArgsContext::new();
        assert!(!ctx.is_complete());

        ctx.set_model("gpt-4");
        assert!(ctx.is_complete());
    }

    #[test]
    fn test_mark_ready() {
        let mut ctx = ArgsContext::new();
        assert!(!ctx.is_ready());

        ctx.mark_ready();
        assert!(ctx.is_ready());

        ctx.mark_not_ready();
        assert!(!ctx.is_ready());
    }

    #[test]
    fn test_summary_empty() {
        let ctx = ArgsContext::new();
        assert_eq!(ctx.summary(), "No arguments configured");
    }

    #[test]
    fn test_summary_with_model() {
        let ctx = ArgsContext::with_model("gpt-4");
        assert!(ctx.summary().contains("Model: gpt-4"));
    }

    #[test]
    fn test_summary_with_multiple_args() {
        let mut ctx = ArgsContext::new();
        ctx.set_model("gpt-4");
        ctx.set_agent("build");
        ctx.set_continue_session(true);

        let summary = ctx.summary();
        assert!(summary.contains("Model: gpt-4"));
        assert!(summary.contains("Agent: build"));
        assert!(summary.contains("Continue: yes"));
    }

    #[test]
    fn test_summary_long_prompt_truncation() {
        let mut ctx = ArgsContext::new();
        ctx.set_prompt("This is a very long prompt that should be truncated in the summary");

        let summary = ctx.summary();
        assert!(summary.contains("Prompt: This is a very long prompt th..."));
    }

    #[test]
    fn test_default_trait() {
        let ctx = ArgsContext::default();
        assert!(!ctx.has_arguments());
        assert!(!ctx.is_ready());
    }
}
