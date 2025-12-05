/// Property-based tests for Think More configuration
/// **Feature: ricecoder-modes, Property 8: Think More Configuration**
/// **Validates: Requirements 4.3**
#[cfg(test)]
mod tests {
    use crate::{ThinkMoreController, ThinkingDepth, ThinkMoreConfig, TaskConfigManager};
    use proptest::prelude::*;
    use std::time::Duration;

    proptest! {
        /// Property: Per-task configuration overrides global configuration
        /// For any task-specific configuration, it should override the global settings
        #[test]
        fn prop_task_config_overrides_global(
            global_depth in prop_oneof![
                Just(ThinkingDepth::Light),
                Just(ThinkingDepth::Medium),
                Just(ThinkingDepth::Deep),
            ],
            task_depth in prop_oneof![
                Just(ThinkingDepth::Light),
                Just(ThinkingDepth::Medium),
                Just(ThinkingDepth::Deep),
            ],
        ) {
            let controller = ThinkMoreController::new();
            
            // Set global configuration
            controller.set_depth(global_depth).unwrap();
            
            // Create task-specific configuration
            let task_config = ThinkMoreConfig {
                enabled: true,
                depth: task_depth,
                timeout: Duration::from_secs(60),
                auto_enable: false,
            };
            
            // Set task configuration
            controller.set_task_config("task1".to_string(), task_config).unwrap();
            
            // Get effective configuration for the task
            let effective = controller.get_effective_config(Some("task1")).unwrap();
            
            // Task configuration should override global
            prop_assert_eq!(effective.depth, task_depth);
        }

        /// Property: Global configuration is used when no task-specific config exists
        /// For any global configuration, it should be used when no task config is set
        #[test]
        fn prop_global_config_fallback(
            depth in prop_oneof![
                Just(ThinkingDepth::Light),
                Just(ThinkingDepth::Medium),
                Just(ThinkingDepth::Deep),
            ],
        ) {
            let controller = ThinkMoreController::new();
            
            // Set global configuration
            controller.set_depth(depth).unwrap();
            
            // Get effective configuration for non-existent task
            let effective = controller.get_effective_config(Some("nonexistent")).unwrap();
            
            // Should use global configuration
            prop_assert_eq!(effective.depth, depth);
        }

        /// Property: Task configuration can be updated
        /// For any task configuration, it should be updatable to a new configuration
        #[test]
        fn prop_task_config_updatable(
            initial_depth in prop_oneof![
                Just(ThinkingDepth::Light),
                Just(ThinkingDepth::Medium),
                Just(ThinkingDepth::Deep),
            ],
            updated_depth in prop_oneof![
                Just(ThinkingDepth::Light),
                Just(ThinkingDepth::Medium),
                Just(ThinkingDepth::Deep),
            ],
        ) {
            let controller = ThinkMoreController::new();
            
            // Set initial task configuration
            let initial_config = ThinkMoreConfig {
                enabled: true,
                depth: initial_depth,
                timeout: Duration::from_secs(30),
                auto_enable: false,
            };
            controller.set_task_config("task1".to_string(), initial_config).unwrap();
            
            // Update task configuration
            let updated_config = ThinkMoreConfig {
                enabled: true,
                depth: updated_depth,
                timeout: Duration::from_secs(60),
                auto_enable: true,
            };
            controller.set_task_config("task1".to_string(), updated_config).unwrap();
            
            // Verify update
            let retrieved = controller.get_task_config("task1").unwrap();
            prop_assert!(retrieved.is_some());
            prop_assert_eq!(retrieved.unwrap().depth, updated_depth);
        }

        /// Property: Task configuration can be removed
        /// For any task configuration, it should be removable
        #[test]
        fn prop_task_config_removable(
            depth in prop_oneof![
                Just(ThinkingDepth::Light),
                Just(ThinkingDepth::Medium),
                Just(ThinkingDepth::Deep),
            ],
        ) {
            let controller = ThinkMoreController::new();
            
            // Set task configuration
            let config = ThinkMoreConfig {
                enabled: true,
                depth,
                timeout: Duration::from_secs(30),
                auto_enable: false,
            };
            controller.set_task_config("task1".to_string(), config).unwrap();
            
            // Verify it exists
            prop_assert!(controller.get_task_config("task1").unwrap().is_some());
            
            // Remove it
            controller.remove_task_config("task1").unwrap();
            
            // Verify it's gone
            prop_assert!(controller.get_task_config("task1").unwrap().is_none());
        }

        /// Property: Multiple tasks can have different configurations
        /// For any set of task configurations, each task should maintain its own config
        #[test]
        fn prop_multiple_task_configs(
            depths in prop::collection::vec(
                prop_oneof![
                    Just(ThinkingDepth::Light),
                    Just(ThinkingDepth::Medium),
                    Just(ThinkingDepth::Deep),
                ],
                1..10
            ),
        ) {
            let _num_tasks = depths.len();
            let controller = ThinkMoreController::new();
            
            // Set configurations for multiple tasks
            for (i, depth) in depths.iter().enumerate() {
                let config = ThinkMoreConfig {
                    enabled: true,
                    depth: *depth,
                    timeout: Duration::from_secs(30),
                    auto_enable: false,
                };
                let task_id = format!("task{}", i);
                controller.set_task_config(task_id, config).unwrap();
            }
            
            // Verify each task has its configuration
            for (i, expected_depth) in depths.iter().enumerate() {
                let task_id = format!("task{}", i);
                let config = controller.get_task_config(&task_id).unwrap();
                prop_assert!(config.is_some());
                prop_assert_eq!(config.unwrap().depth, *expected_depth);
            }
        }

        /// Property: Configuration settings are independent
        /// For any configuration, changing one setting should not affect others
        #[test]
        fn prop_config_settings_independent(
            _enabled in any::<bool>(),
            depth in prop_oneof![
                Just(ThinkingDepth::Light),
                Just(ThinkingDepth::Medium),
                Just(ThinkingDepth::Deep),
            ],
            timeout_secs in 1u64..300,
            auto_enable in any::<bool>(),
        ) {
            let controller = ThinkMoreController::new();
            
            // Set various configurations
            controller.enable().unwrap();
            controller.set_depth(depth).unwrap();
            controller.set_timeout(Duration::from_secs(timeout_secs)).unwrap();
            if auto_enable {
                controller.enable_auto_enable().unwrap();
            } else {
                controller.disable_auto_enable().unwrap();
            }
            
            // Verify each setting
            prop_assert!(controller.is_enabled().unwrap());
            prop_assert_eq!(controller.get_depth().unwrap(), depth);
            prop_assert_eq!(controller.get_timeout().unwrap(), Duration::from_secs(timeout_secs));
            prop_assert_eq!(controller.is_auto_enable_enabled().unwrap(), auto_enable);
        }
    }

    proptest! {
        /// Property: TaskConfigManager maintains per-task configurations
        /// For any set of task configurations, the manager should maintain them correctly
        #[test]
        fn prop_task_config_manager_maintains_configs(
            depths in prop::collection::vec(
                prop_oneof![
                    Just(ThinkingDepth::Light),
                    Just(ThinkingDepth::Medium),
                    Just(ThinkingDepth::Deep),
                ],
                1..10
            ),
        ) {
            let num_tasks = depths.len();
            let manager = TaskConfigManager::new();
            
            // Register tasks
            for (i, depth) in depths.iter().enumerate() {
                let config = ThinkMoreConfig {
                    enabled: true,
                    depth: *depth,
                    timeout: Duration::from_secs(30),
                    auto_enable: false,
                };
                let task_id = format!("task{}", i);
                manager.register_task(task_id, config).unwrap();
            }
            
            // Verify count
            prop_assert_eq!(manager.task_count().unwrap(), num_tasks);
            
            // Verify each task
            for (i, expected_depth) in depths.iter().enumerate() {
                let task_id = format!("task{}", i);
                let config = manager.get_task_config(&task_id).unwrap();
                prop_assert!(config.is_some());
                prop_assert_eq!(config.unwrap().depth, *expected_depth);
            }
        }

        /// Property: TaskConfigManager can store and retrieve context
        /// For any context data, it should be storable and retrievable per task
        #[test]
        fn prop_task_config_manager_context(
            num_tasks in 1..5usize,
            context_values in prop::collection::vec(".*", 1..5),
        ) {
            let manager = TaskConfigManager::new();
            
            // Register tasks with context
            for i in 0..num_tasks {
                let config = ThinkMoreConfig::default();
                let task_id = format!("task{}", i);
                manager.register_task(task_id.clone(), config).unwrap();
                
                // Add context
                for (j, value) in context_values.iter().enumerate() {
                    let key = format!("key{}", j);
                    manager.add_task_context(
                        &task_id,
                        key,
                        serde_json::json!(value),
                    ).unwrap();
                }
            }
            
            // Verify context
            for i in 0..num_tasks {
                let task_id = format!("task{}", i);
                for (j, _expected_value) in context_values.iter().enumerate() {
                    let key = format!("key{}", j);
                    let value = manager.get_task_context(&task_id, &key).unwrap();
                    prop_assert!(value.is_some());
                }
            }
        }
    }
}
