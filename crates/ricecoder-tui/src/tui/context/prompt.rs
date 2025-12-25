//! Prompt Component Reference Context
//!
//! This module provides a reference holder for the prompt component,
//! allowing other parts of the TUI to access and manipulate the prompt
//! input (e.g., focus, clear, set text).

use std::sync::Arc;
use tokio::sync::RwLock;

/// Prompt operations interface
pub trait PromptOps: Send + Sync {
    /// Focus the prompt input
    fn focus(&mut self);

    /// Clear the prompt input
    fn clear(&mut self);

    /// Set prompt text
    fn set_text(&mut self, text: String);

    /// Get current prompt text
    fn get_text(&self) -> String;

    /// Check if prompt is focused
    fn is_focused(&self) -> bool;
}

/// Prompt reference holder
#[derive(Clone)]
pub struct PromptRef {
    inner: Arc<RwLock<Option<Box<dyn PromptOps>>>>,
}

impl PromptRef {
    /// Create new prompt ref (initially empty)
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(None)),
        }
    }

    /// Set the prompt component
    pub async fn set(&self, prompt: Box<dyn PromptOps>) {
        let mut write = self.inner.write().await;
        *write = Some(prompt);
    }

    /// Clear the prompt component reference
    pub async fn clear_ref(&self) {
        let mut write = self.inner.write().await;
        *write = None;
    }

    /// Focus the prompt
    pub async fn focus(&self) {
        if let Some(ref mut p) = *self.inner.write().await {
            p.focus();
        }
    }

    /// Clear prompt text
    pub async fn clear_text(&self) {
        if let Some(ref mut p) = *self.inner.write().await {
            p.clear();
        }
    }

    /// Set prompt text
    pub async fn set_text(&self, text: String) {
        if let Some(ref mut p) = *self.inner.write().await {
            p.set_text(text);
        }
    }

    /// Get current prompt text
    pub async fn get_text(&self) -> Option<String> {
        if let Some(ref p) = *self.inner.read().await {
            Some(p.get_text())
        } else {
            None
        }
    }

    /// Check if prompt is focused
    pub async fn is_focused(&self) -> bool {
        self.inner
            .read()
            .await
            .as_ref()
            .map(|p| p.is_focused())
            .unwrap_or(false)
    }
}

impl Default for PromptRef {
    fn default() -> Self {
        Self::new()
    }
}

/// Prompt reference provider
#[derive(Clone)]
pub struct PromptRefProvider {
    prompt_ref: PromptRef,
}

impl PromptRefProvider {
    /// Create new prompt ref provider
    pub fn new() -> Self {
        Self {
            prompt_ref: PromptRef::new(),
        }
    }

    /// Get the prompt reference
    pub fn get_ref(&self) -> PromptRef {
        self.prompt_ref.clone()
    }
}

impl Default for PromptRefProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockPrompt {
        text: String,
        focused: bool,
    }

    impl PromptOps for MockPrompt {
        fn focus(&mut self) {
            self.focused = true;
        }

        fn clear(&mut self) {
            self.text.clear();
        }

        fn set_text(&mut self, text: String) {
            self.text = text;
        }

        fn get_text(&self) -> String {
            self.text.clone()
        }

        fn is_focused(&self) -> bool {
            self.focused
        }
    }

    #[tokio::test]
    async fn test_prompt_ref_creation() {
        let prompt_ref = PromptRef::new();
        assert_eq!(prompt_ref.get_text().await, None);
        assert!(!prompt_ref.is_focused().await);
    }

    #[tokio::test]
    async fn test_prompt_ref_set() {
        let prompt_ref = PromptRef::new();
        let mock = Box::new(MockPrompt {
            text: "initial".to_string(),
            focused: false,
        });

        prompt_ref.set(mock).await;
        assert_eq!(prompt_ref.get_text().await, Some("initial".to_string()));
    }

    #[tokio::test]
    async fn test_prompt_ref_operations() {
        let prompt_ref = PromptRef::new();
        let mock = Box::new(MockPrompt {
            text: String::new(),
            focused: false,
        });

        prompt_ref.set(mock).await;

        prompt_ref.set_text("Hello".to_string()).await;
        assert_eq!(prompt_ref.get_text().await, Some("Hello".to_string()));

        prompt_ref.focus().await;
        assert!(prompt_ref.is_focused().await);

        prompt_ref.clear_text().await;
        assert_eq!(prompt_ref.get_text().await, Some(String::new()));
    }

    #[tokio::test]
    async fn test_prompt_ref_clear() {
        let prompt_ref = PromptRef::new();
        let mock = Box::new(MockPrompt {
            text: "test".to_string(),
            focused: true,
        });

        prompt_ref.set(mock).await;
        assert_eq!(prompt_ref.get_text().await, Some("test".to_string()));

        prompt_ref.clear_ref().await;
        assert_eq!(prompt_ref.get_text().await, None);
    }

    #[tokio::test]
    async fn test_prompt_ref_provider() {
        let provider = PromptRefProvider::new();
        let prompt_ref = provider.get_ref();

        let mock = Box::new(MockPrompt {
            text: "provider test".to_string(),
            focused: false,
        });

        prompt_ref.set(mock).await;
        assert_eq!(
            prompt_ref.get_text().await,
            Some("provider test".to_string())
        );
    }
}
