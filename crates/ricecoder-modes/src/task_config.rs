use crate::error::{ModeError, Result};
use crate::models::ThinkMoreConfig;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Manages per-task Think More configuration with context storage
#[derive(Debug, Clone)]
pub struct TaskConfigManager {
    /// Per-task configurations
    configs: Arc<Mutex<HashMap<String, TaskConfig>>>,
}

/// Configuration for a specific task
#[derive(Debug, Clone)]
pub struct TaskConfig {
    /// Task identifier
    pub task_id: String,
    /// Think More configuration for this task
    pub think_more_config: ThinkMoreConfig,
    /// Custom context data for this task
    pub context: HashMap<String, serde_json::Value>,
}

impl TaskConfig {
    /// Create a new task configuration
    pub fn new(task_id: String, think_more_config: ThinkMoreConfig) -> Self {
        Self {
            task_id,
            think_more_config,
            context: HashMap::new(),
        }
    }

    /// Add context data to the task
    pub fn add_context(&mut self, key: String, value: serde_json::Value) {
        self.context.insert(key, value);
    }

    /// Get context data from the task
    pub fn get_context(&self, key: &str) -> Option<&serde_json::Value> {
        self.context.get(key)
    }

    /// Remove context data from the task
    pub fn remove_context(&mut self, key: &str) -> Option<serde_json::Value> {
        self.context.remove(key)
    }

    /// Clear all context data
    pub fn clear_context(&mut self) {
        self.context.clear();
    }
}

impl TaskConfigManager {
    /// Create a new task configuration manager
    pub fn new() -> Self {
        Self {
            configs: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register a task with its configuration
    pub fn register_task(&self, task_id: String, config: ThinkMoreConfig) -> Result<()> {
        let mut configs = self.configs.lock().map_err(|_| {
            ModeError::ConfigError("Failed to acquire lock on task configs".to_string())
        })?;
        let task_config = TaskConfig::new(task_id.clone(), config);
        configs.insert(task_id, task_config);
        Ok(())
    }

    /// Get the configuration for a task
    pub fn get_task_config(&self, task_id: &str) -> Result<Option<ThinkMoreConfig>> {
        let configs = self.configs.lock().map_err(|_| {
            ModeError::ConfigError("Failed to acquire lock on task configs".to_string())
        })?;
        Ok(configs.get(task_id).map(|tc| tc.think_more_config.clone()))
    }

    /// Update the configuration for a task
    pub fn update_task_config(&self, task_id: &str, config: ThinkMoreConfig) -> Result<()> {
        let mut configs = self.configs.lock().map_err(|_| {
            ModeError::ConfigError("Failed to acquire lock on task configs".to_string())
        })?;

        if let Some(task_config) = configs.get_mut(task_id) {
            task_config.think_more_config = config;
            Ok(())
        } else {
            Err(ModeError::NotFound(format!("Task {} not found", task_id)))
        }
    }

    /// Unregister a task
    pub fn unregister_task(&self, task_id: &str) -> Result<()> {
        let mut configs = self.configs.lock().map_err(|_| {
            ModeError::ConfigError("Failed to acquire lock on task configs".to_string())
        })?;
        configs.remove(task_id);
        Ok(())
    }

    /// Check if a task is registered
    pub fn has_task(&self, task_id: &str) -> Result<bool> {
        let configs = self.configs.lock().map_err(|_| {
            ModeError::ConfigError("Failed to acquire lock on task configs".to_string())
        })?;
        Ok(configs.contains_key(task_id))
    }

    /// Add context data to a task
    pub fn add_task_context(
        &self,
        task_id: &str,
        key: String,
        value: serde_json::Value,
    ) -> Result<()> {
        let mut configs = self.configs.lock().map_err(|_| {
            ModeError::ConfigError("Failed to acquire lock on task configs".to_string())
        })?;

        if let Some(task_config) = configs.get_mut(task_id) {
            task_config.add_context(key, value);
            Ok(())
        } else {
            Err(ModeError::NotFound(format!("Task {} not found", task_id)))
        }
    }

    /// Get context data from a task
    pub fn get_task_context(&self, task_id: &str, key: &str) -> Result<Option<serde_json::Value>> {
        let configs = self.configs.lock().map_err(|_| {
            ModeError::ConfigError("Failed to acquire lock on task configs".to_string())
        })?;

        Ok(configs
            .get(task_id)
            .and_then(|tc| tc.get_context(key).cloned()))
    }

    /// Remove context data from a task
    pub fn remove_task_context(
        &self,
        task_id: &str,
        key: &str,
    ) -> Result<Option<serde_json::Value>> {
        let mut configs = self.configs.lock().map_err(|_| {
            ModeError::ConfigError("Failed to acquire lock on task configs".to_string())
        })?;

        if let Some(task_config) = configs.get_mut(task_id) {
            Ok(task_config.remove_context(key))
        } else {
            Err(ModeError::NotFound(format!("Task {} not found", task_id)))
        }
    }

    /// Clear all context data for a task
    pub fn clear_task_context(&self, task_id: &str) -> Result<()> {
        let mut configs = self.configs.lock().map_err(|_| {
            ModeError::ConfigError("Failed to acquire lock on task configs".to_string())
        })?;

        if let Some(task_config) = configs.get_mut(task_id) {
            task_config.clear_context();
            Ok(())
        } else {
            Err(ModeError::NotFound(format!("Task {} not found", task_id)))
        }
    }

    /// Get all registered task IDs
    pub fn get_all_task_ids(&self) -> Result<Vec<String>> {
        let configs = self.configs.lock().map_err(|_| {
            ModeError::ConfigError("Failed to acquire lock on task configs".to_string())
        })?;
        Ok(configs.keys().cloned().collect())
    }

    /// Get the number of registered tasks
    pub fn task_count(&self) -> Result<usize> {
        let configs = self.configs.lock().map_err(|_| {
            ModeError::ConfigError("Failed to acquire lock on task configs".to_string())
        })?;
        Ok(configs.len())
    }

    /// Clear all tasks
    pub fn clear_all_tasks(&self) -> Result<()> {
        let mut configs = self.configs.lock().map_err(|_| {
            ModeError::ConfigError("Failed to acquire lock on task configs".to_string())
        })?;
        configs.clear();
        Ok(())
    }

    /// Get all task configurations
    pub fn get_all_configs(&self) -> Result<Vec<TaskConfig>> {
        let configs = self.configs.lock().map_err(|_| {
            ModeError::ConfigError("Failed to acquire lock on task configs".to_string())
        })?;
        Ok(configs.values().cloned().collect())
    }
}

impl Default for TaskConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ThinkingDepth;
    use std::time::Duration;

    fn create_test_config() -> ThinkMoreConfig {
        ThinkMoreConfig {
            enabled: true,
            depth: crate::models::ThinkingDepth::Medium,
            timeout: Duration::from_secs(30),
            auto_enable: false,
        }
    }

    #[test]
    fn test_task_config_creation() {
        let config = create_test_config();
        let task_config = TaskConfig::new("task1".to_string(), config.clone());
        assert_eq!(task_config.task_id, "task1");
        assert_eq!(task_config.think_more_config.depth, ThinkingDepth::Medium);
    }

    #[test]
    fn test_task_config_context() {
        let config = create_test_config();
        let mut task_config = TaskConfig::new("task1".to_string(), config);

        task_config.add_context("key1".to_string(), serde_json::json!("value1"));
        assert!(task_config.get_context("key1").is_some());

        task_config.remove_context("key1");
        assert!(task_config.get_context("key1").is_none());
    }

    #[test]
    fn test_manager_register_task() {
        let manager = TaskConfigManager::new();
        let config = create_test_config();

        manager.register_task("task1".to_string(), config).unwrap();
        assert!(manager.has_task("task1").unwrap());
    }

    #[test]
    fn test_manager_get_task_config() {
        let manager = TaskConfigManager::new();
        let config = create_test_config();

        manager
            .register_task("task1".to_string(), config.clone())
            .unwrap();
        let retrieved = manager.get_task_config("task1").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().depth, ThinkingDepth::Medium);
    }

    #[test]
    fn test_manager_update_task_config() {
        let manager = TaskConfigManager::new();
        let config = create_test_config();

        manager.register_task("task1".to_string(), config).unwrap();

        let mut new_config = create_test_config();
        new_config.depth = ThinkingDepth::Deep;
        manager.update_task_config("task1", new_config).unwrap();

        let retrieved = manager.get_task_config("task1").unwrap();
        assert_eq!(retrieved.unwrap().depth, ThinkingDepth::Deep);
    }

    #[test]
    fn test_manager_unregister_task() {
        let manager = TaskConfigManager::new();
        let config = create_test_config();

        manager.register_task("task1".to_string(), config).unwrap();
        assert!(manager.has_task("task1").unwrap());

        manager.unregister_task("task1").unwrap();
        assert!(!manager.has_task("task1").unwrap());
    }

    #[test]
    fn test_manager_add_task_context() {
        let manager = TaskConfigManager::new();
        let config = create_test_config();

        manager.register_task("task1".to_string(), config).unwrap();
        manager
            .add_task_context("task1", "key1".to_string(), serde_json::json!("value1"))
            .unwrap();

        let value = manager.get_task_context("task1", "key1").unwrap();
        assert!(value.is_some());
    }

    #[test]
    fn test_manager_remove_task_context() {
        let manager = TaskConfigManager::new();
        let config = create_test_config();

        manager.register_task("task1".to_string(), config).unwrap();
        manager
            .add_task_context("task1", "key1".to_string(), serde_json::json!("value1"))
            .unwrap();
        manager.remove_task_context("task1", "key1").unwrap();

        let value = manager.get_task_context("task1", "key1").unwrap();
        assert!(value.is_none());
    }

    #[test]
    fn test_manager_clear_task_context() {
        let manager = TaskConfigManager::new();
        let config = create_test_config();

        manager.register_task("task1".to_string(), config).unwrap();
        manager
            .add_task_context("task1", "key1".to_string(), serde_json::json!("value1"))
            .unwrap();
        manager
            .add_task_context("task1", "key2".to_string(), serde_json::json!("value2"))
            .unwrap();

        manager.clear_task_context("task1").unwrap();

        assert!(manager.get_task_context("task1", "key1").unwrap().is_none());
        assert!(manager.get_task_context("task1", "key2").unwrap().is_none());
    }

    #[test]
    fn test_manager_get_all_task_ids() {
        let manager = TaskConfigManager::new();
        let config = create_test_config();

        manager
            .register_task("task1".to_string(), config.clone())
            .unwrap();
        manager
            .register_task("task2".to_string(), config.clone())
            .unwrap();
        manager.register_task("task3".to_string(), config).unwrap();

        let ids = manager.get_all_task_ids().unwrap();
        assert_eq!(ids.len(), 3);
    }

    #[test]
    fn test_manager_task_count() {
        let manager = TaskConfigManager::new();
        let config = create_test_config();

        manager
            .register_task("task1".to_string(), config.clone())
            .unwrap();
        manager.register_task("task2".to_string(), config).unwrap();

        assert_eq!(manager.task_count().unwrap(), 2);
    }

    #[test]
    fn test_manager_clear_all_tasks() {
        let manager = TaskConfigManager::new();
        let config = create_test_config();

        manager
            .register_task("task1".to_string(), config.clone())
            .unwrap();
        manager.register_task("task2".to_string(), config).unwrap();

        manager.clear_all_tasks().unwrap();
        assert_eq!(manager.task_count().unwrap(), 0);
    }

    #[test]
    fn test_manager_get_all_configs() {
        let manager = TaskConfigManager::new();
        let config = create_test_config();

        manager
            .register_task("task1".to_string(), config.clone())
            .unwrap();
        manager.register_task("task2".to_string(), config).unwrap();

        let configs = manager.get_all_configs().unwrap();
        assert_eq!(configs.len(), 2);
    }

    #[test]
    fn test_manager_default() {
        let manager = TaskConfigManager::default();
        assert_eq!(manager.task_count().unwrap(), 0);
    }

    #[test]
    fn test_manager_error_on_nonexistent_task() {
        let manager = TaskConfigManager::new();
        let result = manager.get_task_config("nonexistent");
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_manager_error_on_update_nonexistent_task() {
        let manager = TaskConfigManager::new();
        let config = create_test_config();
        let result = manager.update_task_config("nonexistent", config);
        assert!(result.is_err());
    }
}
