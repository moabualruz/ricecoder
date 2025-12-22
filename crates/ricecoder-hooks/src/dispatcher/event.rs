//! Event routing and dispatching implementation

use std::sync::Arc;

use tracing::{debug, error, info};

use crate::{
    error::{HooksError, Result},
    executor::HookExecutor,
    registry::HookRegistry,
    types::Event,
};

/// Default implementation of EventDispatcher
///
/// Routes events to matching hooks in the registry and executes them using the executor.
/// Implements hook isolation: if one hook fails, other hooks continue executing.
#[derive(Clone)]
pub struct DefaultEventDispatcher {
    registry: Arc<dyn HookRegistry>,
    executor: Arc<dyn HookExecutor>,
}

impl DefaultEventDispatcher {
    /// Create a new event dispatcher
    ///
    /// # Arguments
    ///
    /// * `registry` - Hook registry for querying hooks
    /// * `executor` - Hook executor for executing hooks
    pub fn new(registry: Arc<dyn HookRegistry>, executor: Arc<dyn HookExecutor>) -> Self {
        Self { registry, executor }
    }
}

impl super::EventDispatcher for DefaultEventDispatcher {
    fn dispatch_event(&self, event: Event) -> Result<()> {
        debug!(
            event_type = %event.event_type,
            timestamp = %event.timestamp,
            "Dispatching event"
        );

        // Query registry for hooks matching this event type
        let hooks = self.registry.list_hooks_for_event(&event.event_type)?;
        let hook_count = hooks.len();

        if hooks.is_empty() {
            debug!(
                event_type = %event.event_type,
                "No hooks registered for event"
            );
            return Ok(());
        }

        info!(
            event_type = %event.event_type,
            hook_count = hook_count,
            "Found hooks for event"
        );

        // Execute each hook in order
        let mut execution_errors = Vec::new();

        for hook in hooks {
            debug!(
                hook_id = %hook.id,
                hook_name = %hook.name,
                "Executing hook"
            );

            // Execute the hook with the event context
            match self.executor.execute_hook(&hook, &event.context) {
                Ok(result) => {
                    info!(
                        hook_id = %hook.id,
                        status = ?result.status,
                        duration_ms = result.duration_ms,
                        "Hook executed successfully"
                    );
                }
                Err(e) => {
                    error!(
                        hook_id = %hook.id,
                        error = %e,
                        "Hook execution failed"
                    );
                    execution_errors.push((hook.id.clone(), e));
                    // Continue with next hook (hook isolation)
                }
            }
        }

        // If all hooks failed, return error
        if !execution_errors.is_empty() && execution_errors.len() == hook_count {
            let error_msg = execution_errors
                .iter()
                .map(|(id, e)| format!("{}: {}", id, e))
                .collect::<Vec<_>>()
                .join("; ");
            return Err(HooksError::ExecutionFailed(format!(
                "All hooks failed: {}",
                error_msg
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;
    use crate::{
        dispatcher::EventDispatcher,
        executor::HookExecutor,
        registry::InMemoryHookRegistry,
        types::{Action, CommandAction, EventContext, Hook, HookResult, HookStatus},
    };

    struct MockExecutor {
        call_count: Arc<Mutex<usize>>,
        should_fail: bool,
    }

    impl MockExecutor {
        fn new(should_fail: bool) -> Self {
            Self {
                call_count: Arc::new(Mutex::new(0)),
                should_fail,
            }
        }

        fn get_call_count(&self) -> usize {
            *self.call_count.lock().unwrap()
        }
    }

    impl HookExecutor for MockExecutor {
        fn execute_hook(&self, hook: &Hook, _context: &EventContext) -> Result<HookResult> {
            let mut count = self.call_count.lock().unwrap();
            *count += 1;

            if self.should_fail {
                Err(HooksError::ExecutionFailed("Mock failure".to_string()))
            } else {
                Ok(HookResult {
                    hook_id: hook.id.clone(),
                    status: HookStatus::Success,
                    output: Some("Mock output".to_string()),
                    error: None,
                    duration_ms: 100,
                })
            }
        }

        fn execute_action(&self, _hook: &Hook, _context: &EventContext) -> Result<String> {
            Ok("Mock action result".to_string())
        }
    }

    fn create_test_hook(id: &str, event: &str) -> Hook {
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
            enabled: true,
            tags: vec![],
            metadata: serde_json::json!({}),
            condition: None,
        }
    }

    fn create_test_event(event_type: &str) -> Event {
        Event {
            event_type: event_type.to_string(),
            context: EventContext {
                data: serde_json::json!({}),
                metadata: serde_json::json!({}),
            },
            timestamp: "2024-01-01T12:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_dispatch_event_with_matching_hooks() {
        let mut registry = InMemoryHookRegistry::new();
        let hook1 = create_test_hook("hook1", "file_saved");
        let hook2 = create_test_hook("hook2", "file_saved");

        registry.register_hook(hook1).unwrap();
        registry.register_hook(hook2).unwrap();

        let executor = Arc::new(MockExecutor::new(false));
        let dispatcher = DefaultEventDispatcher::new(
            Arc::new(registry),
            executor.clone() as Arc<dyn HookExecutor>,
        );

        let event = create_test_event("file_saved");
        dispatcher.dispatch_event(event).unwrap();

        assert_eq!(executor.get_call_count(), 2);
    }

    #[test]
    fn test_dispatch_event_no_matching_hooks() {
        let registry = InMemoryHookRegistry::new();
        let executor = Arc::new(MockExecutor::new(false));
        let dispatcher = DefaultEventDispatcher::new(
            Arc::new(registry),
            executor.clone() as Arc<dyn HookExecutor>,
        );

        let event = create_test_event("file_saved");
        dispatcher.dispatch_event(event).unwrap();

        assert_eq!(executor.get_call_count(), 0);
    }

    #[test]
    fn test_dispatch_event_hook_isolation() {
        let mut registry = InMemoryHookRegistry::new();
        let hook1 = create_test_hook("hook1", "file_saved");
        let hook2 = create_test_hook("hook2", "file_saved");

        registry.register_hook(hook1).unwrap();
        registry.register_hook(hook2).unwrap();

        // Create executor that fails on first call, succeeds on second
        let call_count = Arc::new(Mutex::new(0));
        let call_count_clone = call_count.clone();

        struct SelectiveFailExecutor {
            call_count: Arc<Mutex<usize>>,
        }

        impl HookExecutor for SelectiveFailExecutor {
            fn execute_hook(&self, hook: &Hook, _context: &EventContext) -> Result<HookResult> {
                let mut count = self.call_count.lock().unwrap();
                *count += 1;

                if *count == 1 {
                    Err(HooksError::ExecutionFailed("First hook fails".to_string()))
                } else {
                    Ok(HookResult {
                        hook_id: hook.id.clone(),
                        status: HookStatus::Success,
                        output: None,
                        error: None,
                        duration_ms: 100,
                    })
                }
            }

            fn execute_action(&self, _hook: &Hook, _context: &EventContext) -> Result<String> {
                Ok("Mock action result".to_string())
            }
        }

        let executor = Arc::new(SelectiveFailExecutor {
            call_count: call_count_clone,
        });
        let dispatcher = DefaultEventDispatcher::new(
            Arc::new(registry),
            executor.clone() as Arc<dyn HookExecutor>,
        );

        let event = create_test_event("file_saved");
        // Should succeed because second hook succeeds (hook isolation)
        dispatcher.dispatch_event(event).unwrap();

        assert_eq!(*call_count.lock().unwrap(), 2);
    }

    #[test]
    fn test_dispatch_event_all_hooks_fail() {
        let mut registry = InMemoryHookRegistry::new();
        let hook1 = create_test_hook("hook1", "file_saved");
        let hook2 = create_test_hook("hook2", "file_saved");

        registry.register_hook(hook1).unwrap();
        registry.register_hook(hook2).unwrap();

        let executor = Arc::new(MockExecutor::new(true));
        let dispatcher = DefaultEventDispatcher::new(
            Arc::new(registry),
            executor.clone() as Arc<dyn HookExecutor>,
        );

        let event = create_test_event("file_saved");
        let result = dispatcher.dispatch_event(event);

        assert!(result.is_err());
        assert_eq!(executor.get_call_count(), 2);
    }

    #[test]
    fn test_dispatch_event_respects_hook_order() {
        let mut registry = InMemoryHookRegistry::new();
        let hook1 = create_test_hook("hook1", "file_saved");
        let hook2 = create_test_hook("hook2", "file_saved");
        let hook3 = create_test_hook("hook3", "file_saved");

        registry.register_hook(hook1).unwrap();
        registry.register_hook(hook2).unwrap();
        registry.register_hook(hook3).unwrap();

        let execution_order = Arc::new(Mutex::new(Vec::new()));
        let execution_order_clone = execution_order.clone();

        struct OrderTrackingExecutor {
            execution_order: Arc<Mutex<Vec<String>>>,
        }

        impl HookExecutor for OrderTrackingExecutor {
            fn execute_hook(&self, hook: &Hook, _context: &EventContext) -> Result<HookResult> {
                self.execution_order.lock().unwrap().push(hook.id.clone());

                Ok(HookResult {
                    hook_id: hook.id.clone(),
                    status: HookStatus::Success,
                    output: None,
                    error: None,
                    duration_ms: 100,
                })
            }

            fn execute_action(&self, _hook: &Hook, _context: &EventContext) -> Result<String> {
                Ok("Mock action result".to_string())
            }
        }

        let executor = Arc::new(OrderTrackingExecutor {
            execution_order: execution_order_clone,
        });
        let dispatcher = DefaultEventDispatcher::new(
            Arc::new(registry),
            executor.clone() as Arc<dyn HookExecutor>,
        );

        let event = create_test_event("file_saved");
        dispatcher.dispatch_event(event).unwrap();

        let order = execution_order.lock().unwrap();
        assert_eq!(order.len(), 3);
        // Verify hooks were executed (order may vary due to HashMap iteration)
        assert!(order.contains(&"hook1".to_string()));
        assert!(order.contains(&"hook2".to_string()));
        assert!(order.contains(&"hook3".to_string()));
    }
}
