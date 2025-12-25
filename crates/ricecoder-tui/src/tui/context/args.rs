//! CLI Arguments Context Provider
//!
//! This module provides access to command-line arguments passed to the TUI.
//! Integrates with clap for argument parsing and provides structured access
//! to TUI-related flags.

use std::sync::Arc;
use tokio::sync::RwLock;

/// CLI arguments for TUI session
#[derive(Debug, Clone, Default)]
pub struct Args {
    /// Model to use for the session
    pub model: Option<String>,
    /// Agent to use for the session
    pub agent: Option<String>,
    /// Initial prompt to send
    pub prompt: Option<String>,
    /// Continue previous session
    pub continue_session: bool,
    /// Session ID to continue
    pub session_id: Option<String>,
}

/// Context provider for CLI arguments
#[derive(Debug, Clone)]
pub struct ArgsProvider {
    args: Arc<RwLock<Args>>,
}

impl ArgsProvider {
    /// Create new args provider
    pub fn new(args: Args) -> Self {
        Self {
            args: Arc::new(RwLock::new(args)),
        }
    }

    /// Get current args (read-only)
    pub async fn get(&self) -> Args {
        self.args.read().await.clone()
    }

    /// Get model if specified
    pub async fn model(&self) -> Option<String> {
        self.args.read().await.model.clone()
    }

    /// Get agent if specified
    pub async fn agent(&self) -> Option<String> {
        self.args.read().await.agent.clone()
    }

    /// Get prompt if specified
    pub async fn prompt(&self) -> Option<String> {
        self.args.read().await.prompt.clone()
    }

    /// Check if should continue session
    pub async fn should_continue(&self) -> bool {
        self.args.read().await.continue_session
    }

    /// Get session ID if specified
    pub async fn session_id(&self) -> Option<String> {
        self.args.read().await.session_id.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_args_provider_creation() {
        let args = Args {
            model: Some("gpt-4".to_string()),
            agent: Some("build".to_string()),
            prompt: Some("Hello".to_string()),
            continue_session: true,
            session_id: Some("123".to_string()),
        };

        let provider = ArgsProvider::new(args);
        assert_eq!(provider.model().await, Some("gpt-4".to_string()));
        assert_eq!(provider.agent().await, Some("build".to_string()));
        assert_eq!(provider.prompt().await, Some("Hello".to_string()));
        assert!(provider.should_continue().await);
        assert_eq!(provider.session_id().await, Some("123".to_string()));
    }

    #[tokio::test]
    async fn test_args_provider_defaults() {
        let provider = ArgsProvider::new(Args::default());
        assert_eq!(provider.model().await, None);
        assert_eq!(provider.agent().await, None);
        assert_eq!(provider.prompt().await, None);
        assert!(!provider.should_continue().await);
        assert_eq!(provider.session_id().await, None);
    }

    #[tokio::test]
    async fn test_args_clone() {
        let args = Args {
            model: Some("claude-3".to_string()),
            agent: None,
            prompt: None,
            continue_session: false,
            session_id: None,
        };

        let provider = ArgsProvider::new(args);
        let cloned = provider.get().await;
        assert_eq!(cloned.model, Some("claude-3".to_string()));
    }
}
