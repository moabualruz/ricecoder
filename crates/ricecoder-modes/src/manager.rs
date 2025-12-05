//! Mode manager for lifecycle and transitions

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::{ModeError, Result};
use crate::mode::Mode;
use crate::models::ModeContext;

/// Central coordinator for mode lifecycle and transitions
///
/// The ModeManager is responsible for:
/// - Registering and managing available modes
/// - Switching between modes
/// - Maintaining the current mode state
/// - Managing the execution context
pub struct ModeManager {
    modes: HashMap<String, Arc<dyn Mode>>,
    current_mode: Arc<RwLock<Option<String>>>,
    context: Arc<RwLock<ModeContext>>,
}

impl ModeManager {
    /// Create a new mode manager with the given context
    pub fn new(context: ModeContext) -> Self {
        Self {
            modes: HashMap::new(),
            current_mode: Arc::new(RwLock::new(None)),
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

    /// Get all registered modes
    pub fn list_modes(&self) -> Vec<Arc<dyn Mode>> {
        self.modes.values().cloned().collect()
    }

    /// Get the current active mode
    pub async fn current_mode(&self) -> Result<Option<Arc<dyn Mode>>> {
        let mode_id = self.current_mode.read().await;
        match mode_id.as_ref() {
            Some(id) => Ok(Some(self.get_mode(id)?)),
            None => Ok(None),
        }
    }

    /// Switch to a different mode
    pub async fn switch_mode(&self, mode_id: &str) -> Result<Arc<dyn Mode>> {
        let mode = self.get_mode(mode_id)?;
        let mut current = self.current_mode.write().await;
        *current = Some(mode_id.to_string());
        Ok(mode)
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

    /// Check if a mode is registered
    pub fn has_mode(&self, id: &str) -> bool {
        self.modes.contains_key(id)
    }

    /// Get the number of registered modes
    pub fn mode_count(&self) -> usize {
        self.modes.len()
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
    fn test_mode_manager_creation() {
        let context = ModeContext::new("test-session".to_string());
        let manager = ModeManager::new(context);
        assert_eq!(manager.mode_count(), 0);
    }

    #[test]
    fn test_register_mode() {
        let context = ModeContext::new("test-session".to_string());
        let mut manager = ModeManager::new(context);

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

        manager.register_mode(mode);
        assert_eq!(manager.mode_count(), 1);
        assert!(manager.has_mode("test"));
    }

    #[test]
    fn test_get_mode() {
        let context = ModeContext::new("test-session".to_string());
        let mut manager = ModeManager::new(context);

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

        manager.register_mode(mode);
        let retrieved = manager.get_mode("test");
        assert!(retrieved.is_ok());
    }

    #[test]
    fn test_get_nonexistent_mode() {
        let context = ModeContext::new("test-session".to_string());
        let manager = ModeManager::new(context);
        let result = manager.get_mode("nonexistent");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_switch_mode() {
        let context = ModeContext::new("test-session".to_string());
        let mut manager = ModeManager::new(context);

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

        manager.register_mode(mode);
        let result = manager.switch_mode("test").await;
        assert!(result.is_ok());

        let current = manager.current_mode().await;
        assert!(current.is_ok());
        assert!(current.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_context_operations() {
        let context = ModeContext::new("test-session".to_string());
        let manager = ModeManager::new(context);

        let ctx = manager.context().await;
        assert_eq!(ctx.session_id, "test-session");

        manager
            .update_context(|ctx| {
                ctx.think_more_enabled = true;
            })
            .await
            .unwrap();

        let updated_ctx = manager.context().await;
        assert!(updated_ctx.think_more_enabled);
    }
}
