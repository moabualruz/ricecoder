use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crate::{
    error::{ModeError, Result},
    models::{ComplexityLevel, ThinkMoreConfig, ThinkingDepth},
};

/// Manages extended thinking capabilities for complex tasks
#[derive(Debug, Clone)]
pub struct ThinkMoreController {
    /// Global Think More configuration
    global_config: Arc<Mutex<ThinkMoreConfig>>,
    /// Per-task configuration overrides
    task_configs: Arc<Mutex<std::collections::HashMap<String, ThinkMoreConfig>>>,
    /// Current thinking state
    thinking_state: Arc<Mutex<ThinkingState>>,
}

/// Represents the current state of thinking
#[derive(Debug, Clone)]
struct ThinkingState {
    /// Whether thinking is currently active
    active: bool,
    /// When thinking started
    start_time: Option<Instant>,
    /// Accumulated thinking content
    thinking_content: String,
    /// Current thinking depth
    depth: ThinkingDepth,
}

impl Default for ThinkingState {
    fn default() -> Self {
        Self {
            active: false,
            start_time: None,
            thinking_content: String::new(),
            depth: ThinkingDepth::Medium,
        }
    }
}

impl ThinkMoreController {
    /// Create a new Think More controller with default configuration
    pub fn new() -> Self {
        Self {
            global_config: Arc::new(Mutex::new(ThinkMoreConfig::default())),
            task_configs: Arc::new(Mutex::new(std::collections::HashMap::new())),
            thinking_state: Arc::new(Mutex::new(ThinkingState::default())),
        }
    }

    /// Create a new Think More controller with custom configuration
    pub fn with_config(config: ThinkMoreConfig) -> Self {
        Self {
            global_config: Arc::new(Mutex::new(config)),
            task_configs: Arc::new(Mutex::new(std::collections::HashMap::new())),
            thinking_state: Arc::new(Mutex::new(ThinkingState::default())),
        }
    }

    /// Enable extended thinking globally
    pub fn enable(&self) -> Result<()> {
        let mut config = self.global_config.lock().map_err(|_| {
            ModeError::ConfigError("Failed to acquire lock on global config".to_string())
        })?;
        config.enabled = true;
        Ok(())
    }

    /// Disable extended thinking globally
    pub fn disable(&self) -> Result<()> {
        let mut config = self.global_config.lock().map_err(|_| {
            ModeError::ConfigError("Failed to acquire lock on global config".to_string())
        })?;
        config.enabled = false;
        Ok(())
    }

    /// Check if extended thinking is enabled
    pub fn is_enabled(&self) -> Result<bool> {
        let config = self.global_config.lock().map_err(|_| {
            ModeError::ConfigError("Failed to acquire lock on global config".to_string())
        })?;
        Ok(config.enabled)
    }

    /// Set the thinking depth
    pub fn set_depth(&self, depth: ThinkingDepth) -> Result<()> {
        let mut config = self.global_config.lock().map_err(|_| {
            ModeError::ConfigError("Failed to acquire lock on global config".to_string())
        })?;
        config.depth = depth;
        Ok(())
    }

    /// Get the current thinking depth
    pub fn get_depth(&self) -> Result<ThinkingDepth> {
        let config = self.global_config.lock().map_err(|_| {
            ModeError::ConfigError("Failed to acquire lock on global config".to_string())
        })?;
        Ok(config.depth)
    }

    /// Set the thinking timeout
    pub fn set_timeout(&self, timeout: Duration) -> Result<()> {
        let mut config = self.global_config.lock().map_err(|_| {
            ModeError::ConfigError("Failed to acquire lock on global config".to_string())
        })?;
        config.timeout = timeout;
        Ok(())
    }

    /// Get the current thinking timeout
    pub fn get_timeout(&self) -> Result<Duration> {
        let config = self.global_config.lock().map_err(|_| {
            ModeError::ConfigError("Failed to acquire lock on global config".to_string())
        })?;
        Ok(config.timeout)
    }

    /// Enable auto-enable based on task complexity
    pub fn enable_auto_enable(&self) -> Result<()> {
        let mut config = self.global_config.lock().map_err(|_| {
            ModeError::ConfigError("Failed to acquire lock on global config".to_string())
        })?;
        config.auto_enable = true;
        Ok(())
    }

    /// Disable auto-enable
    pub fn disable_auto_enable(&self) -> Result<()> {
        let mut config = self.global_config.lock().map_err(|_| {
            ModeError::ConfigError("Failed to acquire lock on global config".to_string())
        })?;
        config.auto_enable = false;
        Ok(())
    }

    /// Check if auto-enable is enabled
    pub fn is_auto_enable_enabled(&self) -> Result<bool> {
        let config = self.global_config.lock().map_err(|_| {
            ModeError::ConfigError("Failed to acquire lock on global config".to_string())
        })?;
        Ok(config.auto_enable)
    }

    /// Set per-task configuration
    pub fn set_task_config(&self, task_id: String, config: ThinkMoreConfig) -> Result<()> {
        let mut task_configs = self.task_configs.lock().map_err(|_| {
            ModeError::ConfigError("Failed to acquire lock on task configs".to_string())
        })?;
        task_configs.insert(task_id, config);
        Ok(())
    }

    /// Get per-task configuration
    pub fn get_task_config(&self, task_id: &str) -> Result<Option<ThinkMoreConfig>> {
        let task_configs = self.task_configs.lock().map_err(|_| {
            ModeError::ConfigError("Failed to acquire lock on task configs".to_string())
        })?;
        Ok(task_configs.get(task_id).cloned())
    }

    /// Remove per-task configuration
    pub fn remove_task_config(&self, task_id: &str) -> Result<()> {
        let mut task_configs = self.task_configs.lock().map_err(|_| {
            ModeError::ConfigError("Failed to acquire lock on task configs".to_string())
        })?;
        task_configs.remove(task_id);
        Ok(())
    }

    /// Get effective configuration for a task (task-specific or global)
    pub fn get_effective_config(&self, task_id: Option<&str>) -> Result<ThinkMoreConfig> {
        // Check for task-specific config first
        if let Some(id) = task_id {
            if let Some(config) = self.get_task_config(id)? {
                return Ok(config);
            }
        }
        // Fall back to global config
        let config = self.global_config.lock().map_err(|_| {
            ModeError::ConfigError("Failed to acquire lock on global config".to_string())
        })?;
        Ok(config.clone())
    }

    /// Start thinking for a task
    pub fn start_thinking(&self, depth: ThinkingDepth) -> Result<()> {
        let mut state = self.thinking_state.lock().map_err(|_| {
            ModeError::ConfigError("Failed to acquire lock on thinking state".to_string())
        })?;
        state.active = true;
        state.start_time = Some(Instant::now());
        state.thinking_content.clear();
        state.depth = depth;
        Ok(())
    }

    /// Stop thinking and return the accumulated content
    pub fn stop_thinking(&self) -> Result<String> {
        let mut state = self.thinking_state.lock().map_err(|_| {
            ModeError::ConfigError("Failed to acquire lock on thinking state".to_string())
        })?;
        state.active = false;
        state.start_time = None;
        Ok(state.thinking_content.clone())
    }

    /// Add content to the thinking process
    pub fn add_thinking_content(&self, content: &str) -> Result<()> {
        let mut state = self.thinking_state.lock().map_err(|_| {
            ModeError::ConfigError("Failed to acquire lock on thinking state".to_string())
        })?;
        if state.active {
            state.thinking_content.push_str(content);
            state.thinking_content.push('\n');
        }
        Ok(())
    }

    /// Check if thinking is currently active
    pub fn is_thinking(&self) -> Result<bool> {
        let state = self.thinking_state.lock().map_err(|_| {
            ModeError::ConfigError("Failed to acquire lock on thinking state".to_string())
        })?;
        Ok(state.active)
    }

    /// Get the current thinking content
    pub fn get_thinking_content(&self) -> Result<String> {
        let state = self.thinking_state.lock().map_err(|_| {
            ModeError::ConfigError("Failed to acquire lock on thinking state".to_string())
        })?;
        Ok(state.thinking_content.clone())
    }

    /// Get the elapsed time since thinking started
    pub fn get_elapsed_time(&self) -> Result<Option<Duration>> {
        let state = self.thinking_state.lock().map_err(|_| {
            ModeError::ConfigError("Failed to acquire lock on thinking state".to_string())
        })?;
        Ok(state.start_time.map(|start| start.elapsed()))
    }

    /// Check if thinking has exceeded the timeout
    pub fn has_exceeded_timeout(&self) -> Result<bool> {
        let config = self.global_config.lock().map_err(|_| {
            ModeError::ConfigError("Failed to acquire lock on global config".to_string())
        })?;
        let state = self.thinking_state.lock().map_err(|_| {
            ModeError::ConfigError("Failed to acquire lock on thinking state".to_string())
        })?;

        if let Some(start) = state.start_time {
            Ok(start.elapsed() > config.timeout)
        } else {
            Ok(false)
        }
    }

    /// Cancel thinking
    pub fn cancel_thinking(&self) -> Result<()> {
        let mut state = self.thinking_state.lock().map_err(|_| {
            ModeError::ConfigError("Failed to acquire lock on thinking state".to_string())
        })?;
        state.active = false;
        state.start_time = None;
        Ok(())
    }

    /// Determine if Think More should be auto-enabled for a given complexity level
    pub fn should_auto_enable(&self, complexity: ComplexityLevel) -> Result<bool> {
        let config = self.global_config.lock().map_err(|_| {
            ModeError::ConfigError("Failed to acquire lock on global config".to_string())
        })?;

        if !config.auto_enable {
            return Ok(false);
        }

        // Auto-enable for Complex tasks
        Ok(complexity == ComplexityLevel::Complex)
    }

    /// Get metadata about the current thinking session
    pub fn get_thinking_metadata(&self) -> Result<ThinkingMetadata> {
        let config = self.global_config.lock().map_err(|_| {
            ModeError::ConfigError("Failed to acquire lock on global config".to_string())
        })?;
        let state = self.thinking_state.lock().map_err(|_| {
            ModeError::ConfigError("Failed to acquire lock on thinking state".to_string())
        })?;

        Ok(ThinkingMetadata {
            enabled: config.enabled,
            active: state.active,
            depth: state.depth,
            elapsed_time: state.start_time.map(|start| start.elapsed()),
            timeout: config.timeout,
            content_length: state.thinking_content.len(),
            auto_enable: config.auto_enable,
        })
    }
}

impl Default for ThinkMoreController {
    fn default() -> Self {
        Self::new()
    }
}

/// Metadata about a thinking session
#[derive(Debug, Clone)]
pub struct ThinkingMetadata {
    /// Whether Think More is enabled
    pub enabled: bool,
    /// Whether thinking is currently active
    pub active: bool,
    /// Current thinking depth
    pub depth: ThinkingDepth,
    /// Elapsed time since thinking started
    pub elapsed_time: Option<Duration>,
    /// Timeout for thinking
    pub timeout: Duration,
    /// Length of accumulated thinking content
    pub content_length: usize,
    /// Whether auto-enable is enabled
    pub auto_enable: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_think_more_controller_creation() {
        let controller = ThinkMoreController::new();
        assert!(!controller.is_enabled().unwrap());
    }

    #[test]
    fn test_enable_disable() {
        let controller = ThinkMoreController::new();
        controller.enable().unwrap();
        assert!(controller.is_enabled().unwrap());
        controller.disable().unwrap();
        assert!(!controller.is_enabled().unwrap());
    }

    #[test]
    fn test_set_get_depth() {
        let controller = ThinkMoreController::new();
        controller.set_depth(ThinkingDepth::Deep).unwrap();
        assert_eq!(controller.get_depth().unwrap(), ThinkingDepth::Deep);
    }

    #[test]
    fn test_set_get_timeout() {
        let controller = ThinkMoreController::new();
        let timeout = Duration::from_secs(60);
        controller.set_timeout(timeout).unwrap();
        assert_eq!(controller.get_timeout().unwrap(), timeout);
    }

    #[test]
    fn test_auto_enable() {
        let controller = ThinkMoreController::new();
        controller.enable_auto_enable().unwrap();
        assert!(controller.is_auto_enable_enabled().unwrap());
        controller.disable_auto_enable().unwrap();
        assert!(!controller.is_auto_enable_enabled().unwrap());
    }

    #[test]
    fn test_task_config() {
        let controller = ThinkMoreController::new();
        let config = ThinkMoreConfig {
            enabled: true,
            depth: ThinkingDepth::Deep,
            timeout: Duration::from_secs(60),
            auto_enable: false,
        };
        controller
            .set_task_config("task1".to_string(), config.clone())
            .unwrap();
        let retrieved = controller.get_task_config("task1").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().depth, ThinkingDepth::Deep);
    }

    #[test]
    fn test_remove_task_config() {
        let controller = ThinkMoreController::new();
        let config = ThinkMoreConfig::default();
        controller
            .set_task_config("task1".to_string(), config)
            .unwrap();
        controller.remove_task_config("task1").unwrap();
        assert!(controller.get_task_config("task1").unwrap().is_none());
    }

    #[test]
    fn test_effective_config_task_override() {
        let controller = ThinkMoreController::new();
        let global_config = ThinkMoreConfig {
            enabled: false,
            depth: ThinkingDepth::Light,
            timeout: Duration::from_secs(30),
            auto_enable: false,
        };
        let task_config = ThinkMoreConfig {
            enabled: true,
            depth: ThinkingDepth::Deep,
            timeout: Duration::from_secs(60),
            auto_enable: true,
        };

        *controller.global_config.lock().unwrap() = global_config;
        controller
            .set_task_config("task1".to_string(), task_config)
            .unwrap();

        let effective = controller.get_effective_config(Some("task1")).unwrap();
        assert!(effective.enabled);
        assert_eq!(effective.depth, ThinkingDepth::Deep);
    }

    #[test]
    fn test_effective_config_global_fallback() {
        let controller = ThinkMoreController::new();
        let global_config = ThinkMoreConfig {
            enabled: true,
            depth: ThinkingDepth::Medium,
            timeout: Duration::from_secs(30),
            auto_enable: false,
        };
        *controller.global_config.lock().unwrap() = global_config;

        let effective = controller
            .get_effective_config(Some("nonexistent"))
            .unwrap();
        assert!(effective.enabled);
        assert_eq!(effective.depth, ThinkingDepth::Medium);
    }

    #[test]
    fn test_start_stop_thinking() {
        let controller = ThinkMoreController::new();
        controller.start_thinking(ThinkingDepth::Deep).unwrap();
        assert!(controller.is_thinking().unwrap());

        let content = controller.stop_thinking().unwrap();
        assert!(!controller.is_thinking().unwrap());
        assert_eq!(content, "");
    }

    #[test]
    fn test_add_thinking_content() {
        let controller = ThinkMoreController::new();
        controller.start_thinking(ThinkingDepth::Deep).unwrap();
        controller.add_thinking_content("First thought").unwrap();
        controller.add_thinking_content("Second thought").unwrap();

        let content = controller.get_thinking_content().unwrap();
        assert!(content.contains("First thought"));
        assert!(content.contains("Second thought"));
    }

    #[test]
    fn test_cancel_thinking() {
        let controller = ThinkMoreController::new();
        controller.start_thinking(ThinkingDepth::Deep).unwrap();
        assert!(controller.is_thinking().unwrap());

        controller.cancel_thinking().unwrap();
        assert!(!controller.is_thinking().unwrap());
    }

    #[test]
    fn test_should_auto_enable() {
        let controller = ThinkMoreController::new();
        controller.enable_auto_enable().unwrap();

        assert!(controller
            .should_auto_enable(ComplexityLevel::Complex)
            .unwrap());
        assert!(!controller
            .should_auto_enable(ComplexityLevel::Moderate)
            .unwrap());
        assert!(!controller
            .should_auto_enable(ComplexityLevel::Simple)
            .unwrap());
    }

    #[test]
    fn test_should_not_auto_enable_when_disabled() {
        let controller = ThinkMoreController::new();
        controller.disable_auto_enable().unwrap();

        assert!(!controller
            .should_auto_enable(ComplexityLevel::Complex)
            .unwrap());
    }

    #[test]
    fn test_thinking_metadata() {
        let controller = ThinkMoreController::new();
        controller.enable().unwrap();
        controller.start_thinking(ThinkingDepth::Deep).unwrap();

        let metadata = controller.get_thinking_metadata().unwrap();
        assert!(metadata.enabled);
        assert!(metadata.active);
        assert_eq!(metadata.depth, ThinkingDepth::Deep);
        assert!(metadata.elapsed_time.is_some());
    }

    #[test]
    fn test_has_exceeded_timeout() {
        let controller = ThinkMoreController::new();
        controller.set_timeout(Duration::from_millis(1)).unwrap();
        controller.start_thinking(ThinkingDepth::Deep).unwrap();

        // Sleep to exceed timeout
        std::thread::sleep(Duration::from_millis(10));

        assert!(controller.has_exceeded_timeout().unwrap());
    }

    #[test]
    fn test_default_implementation() {
        let controller = ThinkMoreController::default();
        assert!(!controller.is_enabled().unwrap());
    }

    #[test]
    fn test_with_config() {
        let config = ThinkMoreConfig {
            enabled: true,
            depth: ThinkingDepth::Deep,
            timeout: Duration::from_secs(60),
            auto_enable: true,
        };
        let controller = ThinkMoreController::with_config(config);
        assert!(controller.is_enabled().unwrap());
        assert_eq!(controller.get_depth().unwrap(), ThinkingDepth::Deep);
    }
}
