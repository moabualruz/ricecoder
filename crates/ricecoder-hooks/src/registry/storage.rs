//! In-memory hook storage implementation

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use uuid::Uuid;

use crate::{
    error::{HooksError, Result},
    types::Hook,
};

/// In-memory hook registry implementation
#[derive(Debug, Clone)]
pub struct InMemoryHookRegistry {
    hooks: Arc<RwLock<HashMap<String, Hook>>>,
}

impl InMemoryHookRegistry {
    /// Create a new in-memory hook registry
    pub fn new() -> Self {
        Self {
            hooks: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryHookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl super::HookRegistry for InMemoryHookRegistry {
    fn register_hook(&mut self, mut hook: Hook) -> Result<String> {
        // Generate unique ID if not provided
        if hook.id.is_empty() {
            hook.id = Uuid::new_v4().to_string();
        }

        let hook_id = hook.id.clone();
        let mut hooks = self.hooks.write().map_err(|e| {
            HooksError::StorageError(format!("Failed to acquire write lock: {}", e))
        })?;

        hooks.insert(hook_id.clone(), hook);
        Ok(hook_id)
    }

    fn unregister_hook(&self, hook_id: &str) -> Result<()> {
        let mut hooks = self.hooks.write().map_err(|e| {
            HooksError::StorageError(format!("Failed to acquire write lock: {}", e))
        })?;

        hooks
            .remove(hook_id)
            .ok_or_else(|| HooksError::HookNotFound(hook_id.to_string()))?;

        Ok(())
    }

    fn get_hook(&self, hook_id: &str) -> Result<Hook> {
        let hooks = self
            .hooks
            .read()
            .map_err(|e| HooksError::StorageError(format!("Failed to acquire read lock: {}", e)))?;

        hooks
            .get(hook_id)
            .cloned()
            .ok_or_else(|| HooksError::HookNotFound(hook_id.to_string()))
    }

    fn list_hooks(&self) -> Result<Vec<Hook>> {
        let hooks = self
            .hooks
            .read()
            .map_err(|e| HooksError::StorageError(format!("Failed to acquire read lock: {}", e)))?;

        Ok(hooks.values().cloned().collect())
    }

    fn list_hooks_for_event(&self, event: &str) -> Result<Vec<Hook>> {
        let hooks = self
            .hooks
            .read()
            .map_err(|e| HooksError::StorageError(format!("Failed to acquire read lock: {}", e)))?;

        Ok(hooks
            .values()
            .filter(|h| h.event == event && h.enabled)
            .cloned()
            .collect())
    }

    fn enable_hook(&mut self, hook_id: &str) -> Result<()> {
        let mut hooks = self.hooks.write().map_err(|e| {
            HooksError::StorageError(format!("Failed to acquire write lock: {}", e))
        })?;

        let hook = hooks
            .get_mut(hook_id)
            .ok_or_else(|| HooksError::HookNotFound(hook_id.to_string()))?;

        hook.enabled = true;
        Ok(())
    }

    fn disable_hook(&mut self, hook_id: &str) -> Result<()> {
        let mut hooks = self.hooks.write().map_err(|e| {
            HooksError::StorageError(format!("Failed to acquire write lock: {}", e))
        })?;

        let hook = hooks
            .get_mut(hook_id)
            .ok_or_else(|| HooksError::HookNotFound(hook_id.to_string()))?;

        hook.enabled = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        registry::HookRegistry,
        types::{Action, CommandAction},
    };

    fn create_test_hook(id: &str, event: &str, enabled: bool) -> Hook {
        Hook {
            id: id.to_string(),
            name: format!("Test Hook {}", id),
            description: None,
            event: event.to_string(),
            action: Action::Command(CommandAction {
                command: "echo".to_string(),
                args: vec!["test".to_string()],
                timeout_ms: None,
                capture_output: false,
            }),
            enabled,
            tags: vec![],
            metadata: serde_json::json!({}),
            condition: None,
        }
    }

    #[test]
    fn test_register_hook() {
        let mut registry = InMemoryHookRegistry::new();
        let hook = create_test_hook("hook1", "file_saved", true);

        let id = registry.register_hook(hook).unwrap();
        assert!(!id.is_empty());
    }

    #[test]
    fn test_register_hook_generates_id() {
        let mut registry = InMemoryHookRegistry::new();
        let mut hook = create_test_hook("", "file_saved", true);
        hook.id = String::new();

        let id = registry.register_hook(hook).unwrap();
        assert!(!id.is_empty());
    }

    #[test]
    fn test_get_hook() {
        let mut registry = InMemoryHookRegistry::new();
        let hook = create_test_hook("hook1", "file_saved", true);

        registry.register_hook(hook.clone()).unwrap();
        let retrieved = registry.get_hook("hook1").unwrap();

        assert_eq!(retrieved.id, "hook1");
        assert_eq!(retrieved.name, hook.name);
    }

    #[test]
    fn test_get_hook_not_found() {
        let registry = InMemoryHookRegistry::new();
        let result = registry.get_hook("nonexistent");

        assert!(result.is_err());
    }

    #[test]
    fn test_list_hooks() {
        let mut registry = InMemoryHookRegistry::new();
        let hook1 = create_test_hook("hook1", "file_saved", true);
        let hook2 = create_test_hook("hook2", "test_passed", true);

        registry.register_hook(hook1).unwrap();
        registry.register_hook(hook2).unwrap();

        let hooks = registry.list_hooks().unwrap();
        assert_eq!(hooks.len(), 2);
    }

    #[test]
    fn test_list_hooks_for_event() {
        let mut registry = InMemoryHookRegistry::new();
        let hook1 = create_test_hook("hook1", "file_saved", true);
        let hook2 = create_test_hook("hook2", "file_saved", true);
        let hook3 = create_test_hook("hook3", "test_passed", true);

        registry.register_hook(hook1).unwrap();
        registry.register_hook(hook2).unwrap();
        registry.register_hook(hook3).unwrap();

        let hooks = registry.list_hooks_for_event("file_saved").unwrap();
        assert_eq!(hooks.len(), 2);
    }

    #[test]
    fn test_list_hooks_for_event_excludes_disabled() {
        let mut registry = InMemoryHookRegistry::new();
        let hook1 = create_test_hook("hook1", "file_saved", true);
        let hook2 = create_test_hook("hook2", "file_saved", false);

        registry.register_hook(hook1).unwrap();
        registry.register_hook(hook2).unwrap();

        let hooks = registry.list_hooks_for_event("file_saved").unwrap();
        assert_eq!(hooks.len(), 1);
        assert_eq!(hooks[0].id, "hook1");
    }

    #[test]
    fn test_enable_hook() {
        let mut registry = InMemoryHookRegistry::new();
        let hook = create_test_hook("hook1", "file_saved", false);

        registry.register_hook(hook).unwrap();
        registry.enable_hook("hook1").unwrap();

        let retrieved = registry.get_hook("hook1").unwrap();
        assert!(retrieved.enabled);
    }

    #[test]
    fn test_disable_hook() {
        let mut registry = InMemoryHookRegistry::new();
        let hook = create_test_hook("hook1", "file_saved", true);

        registry.register_hook(hook).unwrap();
        registry.disable_hook("hook1").unwrap();

        let retrieved = registry.get_hook("hook1").unwrap();
        assert!(!retrieved.enabled);
    }

    #[test]
    fn test_unregister_hook() {
        let mut registry = InMemoryHookRegistry::new();
        let hook = create_test_hook("hook1", "file_saved", true);

        registry.register_hook(hook).unwrap();
        registry.unregister_hook("hook1").unwrap();

        let result = registry.get_hook("hook1");
        assert!(result.is_err());
    }

    #[test]
    fn test_unregister_nonexistent_hook() {
        let registry = InMemoryHookRegistry::new();
        let result = registry.unregister_hook("nonexistent");

        assert!(result.is_err());
    }
}
