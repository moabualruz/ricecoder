//! Mode switcher for handling mode transitions with context preservation

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::{ModeError, Result};
use crate::mode::Mode;
use crate::models::ModeContext;

/// Handles mode transitions with context preservation
///
/// The ModeSwitcher is responsible for:
/// - Validating mode transitions
/// - Preserving context across switches
/// - Restoring context after switches
/// - Managing mode-specific data
pub struct ModeSwitcher {
    /// Available modes
    modes: HashMap<String, Arc<dyn Mode>>,
    /// Current active mode
    current_mode: Arc<RwLock<Option<String>>>,
    /// Saved contexts for each mode
    saved_contexts: Arc<RwLock<HashMap<String, ModeContext>>>,
    /// Current execution context
    context: Arc<RwLock<ModeContext>>,
}

impl ModeSwitcher {
    /// Create a new mode switcher with the given context
    pub fn new(context: ModeContext) -> Self {
        Self {
            modes: HashMap::new(),
            current_mode: Arc::new(RwLock::new(None)),
            saved_contexts: Arc::new(RwLock::new(HashMap::new())),
            context: Arc::new(RwLock::new(context)),
        }
    }

    /// Register a mode
    pub fn register_mode(&mut self, mode: Arc<dyn Mode>) {
        self.modes.insert(mode.id().to_string(), mode);
    }

    /// Get a registered mode by ID
    pub fn get_mode(&self, id: &str) -> Result<Arc<dyn Mode>> {
        self.modes
            .get(id)
            .cloned()
            .ok_or_else(|| ModeError::NotFound(id.to_string()))
    }

    /// Get the current active mode
    pub async fn current_mode(&self) -> Result<Option<Arc<dyn Mode>>> {
        let mode_id = self.current_mode.read().await;
        match mode_id.as_ref() {
            Some(id) => Ok(Some(self.get_mode(id)?)),
            None => Ok(None),
        }
    }

    /// Get the current mode ID
    pub async fn current_mode_id(&self) -> Option<String> {
        self.current_mode.read().await.clone()
    }

    /// Switch to a different mode with context preservation
    ///
    /// This method:
    /// 1. Validates the mode exists
    /// 2. Saves the current context if switching from a mode
    /// 3. Restores the saved context for the new mode if available
    /// 4. Creates a fresh context if the mode hasn't been visited before
    /// 5. Updates the current mode
    pub async fn switch_mode(&self, mode_id: &str) -> Result<Arc<dyn Mode>> {
        // Validate the target mode exists
        let target_mode = self.get_mode(mode_id)?;

        // Save current context if we're switching from a mode
        if let Some(current_id) = self.current_mode.read().await.as_ref() {
            let ctx = self.context.read().await.clone();
            let mut saved = self.saved_contexts.write().await;
            saved.insert(current_id.clone(), ctx);
        }

        // Restore context for the new mode if available, otherwise create fresh context
        let mut saved = self.saved_contexts.write().await;
        if let Some(saved_ctx) = saved.remove(mode_id) {
            let mut ctx = self.context.write().await;
            *ctx = saved_ctx;
        } else {
            // Create a fresh context for this mode (preserving session_id)
            let mut ctx = self.context.write().await;
            let session_id = ctx.session_id.clone();
            let project_path = ctx.project_path.clone();
            *ctx = ModeContext::new(session_id);
            ctx.project_path = project_path;
        }

        // Update current mode
        let mut current = self.current_mode.write().await;
        *current = Some(mode_id.to_string());

        Ok(target_mode)
    }

    /// Get the current context
    pub async fn context(&self) -> ModeContext {
        self.context.read().await.clone()
    }

    /// Update the context with a closure
    pub async fn update_context<F>(&self, f: F) -> Result<()>
    where
        F: FnOnce(&mut ModeContext),
    {
        let mut ctx = self.context.write().await;
        f(&mut ctx);
        Ok(())
    }

    /// Save the current context for a specific mode
    pub async fn save_context_for_mode(&self, mode_id: &str) -> Result<()> {
        let ctx = self.context.read().await.clone();
        let mut saved = self.saved_contexts.write().await;
        saved.insert(mode_id.to_string(), ctx);
        Ok(())
    }

    /// Restore context for a specific mode
    pub async fn restore_context_for_mode(&self, mode_id: &str) -> Result<()> {
        let mut saved = self.saved_contexts.write().await;
        if let Some(saved_ctx) = saved.remove(mode_id) {
            let mut ctx = self.context.write().await;
            *ctx = saved_ctx;
            Ok(())
        } else {
            Err(ModeError::ContextError(format!(
                "No saved context for mode: {}",
                mode_id
            )))
        }
    }

    /// Check if a mode is registered
    pub fn has_mode(&self, id: &str) -> bool {
        self.modes.contains_key(id)
    }

    /// Get the number of registered modes
    pub fn mode_count(&self) -> usize {
        self.modes.len()
    }

    /// Get all registered mode IDs
    pub fn list_mode_ids(&self) -> Vec<String> {
        self.modes.keys().cloned().collect()
    }

    /// Check if context is saved for a mode
    pub async fn has_saved_context(&self, mode_id: &str) -> bool {
        self.saved_contexts.read().await.contains_key(mode_id)
    }

    /// Get the number of saved contexts
    pub async fn saved_context_count(&self) -> usize {
        self.saved_contexts.read().await.len()
    }

    /// Clear all saved contexts
    pub async fn clear_saved_contexts(&self) {
        self.saved_contexts.write().await.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Capability, ModeConfig, ModeConstraints, Operation};

    struct TestMode {
        id: String,
        config: ModeConfig,
    }

    #[async_trait::async_trait]
    impl Mode for TestMode {
        fn id(&self) -> &str {
            &self.id
        }

        fn name(&self) -> &str {
            "Test Mode"
        }

        fn description(&self) -> &str {
            "A test mode"
        }

        fn system_prompt(&self) -> &str {
            "You are a test mode"
        }

        async fn process(
            &self,
            _input: &str,
            _context: &ModeContext,
        ) -> Result<crate::models::ModeResponse> {
            Ok(crate::models::ModeResponse::new(
                "Test response".to_string(),
                self.id.clone(),
            ))
        }

        fn capabilities(&self) -> Vec<Capability> {
            vec![Capability::QuestionAnswering]
        }

        fn config(&self) -> &ModeConfig {
            &self.config
        }

        fn can_execute(&self, _operation: &Operation) -> bool {
            true
        }

        fn constraints(&self) -> ModeConstraints {
            ModeConstraints {
                allow_file_operations: false,
                allow_command_execution: false,
                allow_code_generation: false,
                require_specs: false,
                auto_think_more_threshold: None,
            }
        }
    }

    #[test]
    fn test_mode_switcher_creation() {
        let context = ModeContext::new("test-session".to_string());
        let switcher = ModeSwitcher::new(context);
        assert_eq!(switcher.mode_count(), 0);
    }

    #[test]
    fn test_register_mode() {
        let context = ModeContext::new("test-session".to_string());
        let mut switcher = ModeSwitcher::new(context);

        let mode = Arc::new(TestMode {
            id: "test".to_string(),
            config: ModeConfig {
                temperature: 0.7,
                max_tokens: 1000,
                system_prompt: "Test".to_string(),
                capabilities: vec![Capability::QuestionAnswering],
                constraints: ModeConstraints {
                    allow_file_operations: false,
                    allow_command_execution: false,
                    allow_code_generation: false,
                    require_specs: false,
                    auto_think_more_threshold: None,
                },
            },
        });

        switcher.register_mode(mode);
        assert_eq!(switcher.mode_count(), 1);
        assert!(switcher.has_mode("test"));
    }

    #[tokio::test]
    async fn test_switch_mode() {
        let context = ModeContext::new("test-session".to_string());
        let mut switcher = ModeSwitcher::new(context);

        let mode = Arc::new(TestMode {
            id: "test".to_string(),
            config: ModeConfig {
                temperature: 0.7,
                max_tokens: 1000,
                system_prompt: "Test".to_string(),
                capabilities: vec![Capability::QuestionAnswering],
                constraints: ModeConstraints {
                    allow_file_operations: false,
                    allow_command_execution: false,
                    allow_code_generation: false,
                    require_specs: false,
                    auto_think_more_threshold: None,
                },
            },
        });

        switcher.register_mode(mode);
        let result = switcher.switch_mode("test").await;
        assert!(result.is_ok());

        let current = switcher.current_mode().await;
        assert!(current.is_ok());
        assert!(current.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_switch_nonexistent_mode() {
        let context = ModeContext::new("test-session".to_string());
        let switcher = ModeSwitcher::new(context);
        let result = switcher.switch_mode("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_context_preservation_on_switch() {
        let context = ModeContext::new("test-session".to_string());
        let mut switcher = ModeSwitcher::new(context);

        let mode1 = Arc::new(TestMode {
            id: "mode1".to_string(),
            config: ModeConfig {
                temperature: 0.7,
                max_tokens: 1000,
                system_prompt: "Mode 1".to_string(),
                capabilities: vec![Capability::QuestionAnswering],
                constraints: ModeConstraints {
                    allow_file_operations: false,
                    allow_command_execution: false,
                    allow_code_generation: false,
                    require_specs: false,
                    auto_think_more_threshold: None,
                },
            },
        });

        let mode2 = Arc::new(TestMode {
            id: "mode2".to_string(),
            config: ModeConfig {
                temperature: 0.7,
                max_tokens: 1000,
                system_prompt: "Mode 2".to_string(),
                capabilities: vec![Capability::QuestionAnswering],
                constraints: ModeConstraints {
                    allow_file_operations: false,
                    allow_command_execution: false,
                    allow_code_generation: false,
                    require_specs: false,
                    auto_think_more_threshold: None,
                },
            },
        });

        switcher.register_mode(mode1);
        switcher.register_mode(mode2);

        // Switch to mode1
        switcher.switch_mode("mode1").await.unwrap();

        // Add a message to the context
        switcher
            .update_context(|ctx| {
                ctx.add_message(
                    crate::models::MessageRole::User,
                    "Hello from mode1".to_string(),
                );
            })
            .await
            .unwrap();

        let ctx1 = switcher.context().await;
        assert_eq!(ctx1.conversation_history.len(), 1);

        // Switch to mode2
        switcher.switch_mode("mode2").await.unwrap();

        // Context should be empty for mode2
        let ctx2 = switcher.context().await;
        assert_eq!(ctx2.conversation_history.len(), 0);

        // Switch back to mode1
        switcher.switch_mode("mode1").await.unwrap();

        // Context should be restored
        let ctx1_restored = switcher.context().await;
        assert_eq!(ctx1_restored.conversation_history.len(), 1);
        assert_eq!(
            ctx1_restored.conversation_history[0].content,
            "Hello from mode1"
        );
    }

    #[tokio::test]
    async fn test_save_and_restore_context() {
        let context = ModeContext::new("test-session".to_string());
        let switcher = ModeSwitcher::new(context);

        // Add a message to the context
        switcher
            .update_context(|ctx| {
                ctx.add_message(
                    crate::models::MessageRole::User,
                    "Test message".to_string(),
                );
            })
            .await
            .unwrap();

        // Save context for a mode
        switcher.save_context_for_mode("test-mode").await.unwrap();
        assert!(switcher.has_saved_context("test-mode").await);

        // Clear the current context
        switcher
            .update_context(|ctx| {
                ctx.conversation_history.clear();
            })
            .await
            .unwrap();

        let ctx = switcher.context().await;
        assert_eq!(ctx.conversation_history.len(), 0);

        // Restore context
        switcher.restore_context_for_mode("test-mode").await.unwrap();

        let restored_ctx = switcher.context().await;
        assert_eq!(restored_ctx.conversation_history.len(), 1);
        assert_eq!(restored_ctx.conversation_history[0].content, "Test message");
    }

    #[tokio::test]
    async fn test_current_mode_id() {
        let context = ModeContext::new("test-session".to_string());
        let mut switcher = ModeSwitcher::new(context);

        let mode = Arc::new(TestMode {
            id: "test".to_string(),
            config: ModeConfig {
                temperature: 0.7,
                max_tokens: 1000,
                system_prompt: "Test".to_string(),
                capabilities: vec![Capability::QuestionAnswering],
                constraints: ModeConstraints {
                    allow_file_operations: false,
                    allow_command_execution: false,
                    allow_code_generation: false,
                    require_specs: false,
                    auto_think_more_threshold: None,
                },
            },
        });

        switcher.register_mode(mode);

        assert!(switcher.current_mode_id().await.is_none());

        switcher.switch_mode("test").await.unwrap();

        let mode_id = switcher.current_mode_id().await;
        assert_eq!(mode_id, Some("test".to_string()));
    }

    #[tokio::test]
    async fn test_list_mode_ids() {
        let context = ModeContext::new("test-session".to_string());
        let mut switcher = ModeSwitcher::new(context);

        let mode1 = Arc::new(TestMode {
            id: "mode1".to_string(),
            config: ModeConfig {
                temperature: 0.7,
                max_tokens: 1000,
                system_prompt: "Mode 1".to_string(),
                capabilities: vec![Capability::QuestionAnswering],
                constraints: ModeConstraints {
                    allow_file_operations: false,
                    allow_command_execution: false,
                    allow_code_generation: false,
                    require_specs: false,
                    auto_think_more_threshold: None,
                },
            },
        });

        let mode2 = Arc::new(TestMode {
            id: "mode2".to_string(),
            config: ModeConfig {
                temperature: 0.7,
                max_tokens: 1000,
                system_prompt: "Mode 2".to_string(),
                capabilities: vec![Capability::QuestionAnswering],
                constraints: ModeConstraints {
                    allow_file_operations: false,
                    allow_command_execution: false,
                    allow_code_generation: false,
                    require_specs: false,
                    auto_think_more_threshold: None,
                },
            },
        });

        switcher.register_mode(mode1);
        switcher.register_mode(mode2);

        let ids = switcher.list_mode_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"mode1".to_string()));
        assert!(ids.contains(&"mode2".to_string()));
    }

    #[tokio::test]
    async fn test_clear_saved_contexts() {
        let context = ModeContext::new("test-session".to_string());
        let switcher = ModeSwitcher::new(context);

        switcher.save_context_for_mode("mode1").await.unwrap();
        switcher.save_context_for_mode("mode2").await.unwrap();

        assert_eq!(switcher.saved_context_count().await, 2);

        switcher.clear_saved_contexts().await;

        assert_eq!(switcher.saved_context_count().await, 0);
    }
}
