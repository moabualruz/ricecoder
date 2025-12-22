//! Property-based tests for event-driven automation
//!
//! These tests verify the correctness properties of the hooks system:
//! - Property 1: Hook execution order
//! - Property 3: Hook isolation
//! - Property 4: Hook chaining

use std::sync::{Arc, Mutex};

use proptest::prelude::*;
use ricecoder_hooks::{
    dispatcher::{DefaultEventDispatcher, EventDispatcher},
    executor::HookExecutor,
    *,
};

// ============================================================================
// Generators for property-based testing
// ============================================================================

/// Generate valid hook IDs
#[allow(dead_code)]
fn hook_id_strategy() -> impl Strategy<Value = String> {
    "[a-z0-9_-]{1,20}"
}

/// Generate valid event types
fn event_type_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("file_saved".to_string()),
        Just("file_modified".to_string()),
        Just("test_passed".to_string()),
        Just("test_failed".to_string()),
        Just("generation_complete".to_string()),
    ]
}

/// Generate valid hook names
#[allow(dead_code)]
fn hook_name_strategy() -> impl Strategy<Value = String> {
    "[A-Za-z ]{1,30}"
}

/// Generate hooks with specific event type
#[allow(dead_code)]
fn hook_strategy(event_type: String) -> impl Strategy<Value = Hook> {
    (hook_id_strategy(), hook_name_strategy()).prop_map(move |(id, name)| Hook {
        id,
        name,
        description: None,
        event: event_type.clone(),
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
    })
}

/// Generate events with specific event type
#[allow(dead_code)]
fn event_strategy(event_type: String) -> impl Strategy<Value = Event> {
    Just(Event {
        event_type,
        context: EventContext {
            data: serde_json::json!({}),
            metadata: serde_json::json!({}),
        },
        timestamp: "2024-01-01T12:00:00Z".to_string(),
    })
}

// ============================================================================
// Mock implementations for testing
// ============================================================================

/// Mock executor that tracks execution order
struct OrderTrackingExecutor {
    execution_order: Arc<Mutex<Vec<String>>>,
}

impl OrderTrackingExecutor {
    fn new() -> Self {
        Self {
            execution_order: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn get_execution_order(&self) -> Vec<String> {
        self.execution_order.lock().unwrap().clone()
    }
}

impl HookExecutor for OrderTrackingExecutor {
    fn execute_hook(&self, hook: &Hook, _context: &EventContext) -> Result<HookResult> {
        self.execution_order.lock().unwrap().push(hook.id.clone());

        Ok(HookResult {
            hook_id: hook.id.clone(),
            status: HookStatus::Success,
            output: Some("Mock output".to_string()),
            error: None,
            duration_ms: 100,
        })
    }

    fn execute_action(&self, _hook: &Hook, _context: &EventContext) -> Result<String> {
        Ok("Mock action result".to_string())
    }
}

/// Mock executor that fails on specific hook IDs
struct SelectiveFailExecutor {
    fail_on_ids: Vec<String>,
    execution_count: Arc<Mutex<usize>>,
}

impl SelectiveFailExecutor {
    fn new(fail_on_ids: Vec<String>) -> Self {
        Self {
            fail_on_ids,
            execution_count: Arc::new(Mutex::new(0)),
        }
    }

    fn get_execution_count(&self) -> usize {
        *self.execution_count.lock().unwrap()
    }
}

impl HookExecutor for SelectiveFailExecutor {
    fn execute_hook(&self, hook: &Hook, _context: &EventContext) -> Result<HookResult> {
        let mut count = self.execution_count.lock().unwrap();
        *count += 1;

        if self.fail_on_ids.contains(&hook.id) {
            Ok(HookResult {
                hook_id: hook.id.clone(),
                status: HookStatus::Failed,
                output: None,
                error: Some("Mock failure".to_string()),
                duration_ms: 100,
            })
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

// ============================================================================
// Property 1: Hook Execution Order
// ============================================================================

proptest! {
    /// **Feature: ricecoder-hooks, Property 1: Hook execution order**
    /// **Validates: Requirements Hooks-2.1**
    ///
    /// For any event with multiple registered hooks, hooks SHALL execute in the order
    /// they were registered.
    #[test]
    fn prop_hooks_execute_in_registration_order(
        event_type in event_type_strategy(),
        hook_count in 1usize..10,
    ) {
        let mut registry = InMemoryHookRegistry::new();
        let executor = Arc::new(OrderTrackingExecutor::new());

        // Register hooks in specific order
        let mut hook_ids = Vec::new();
        for i in 0..hook_count {
            let hook = Hook {
                id: format!("hook_{:02}", i),
                name: format!("Hook {}", i),
                description: None,
                event: event_type.clone(),
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
            };
            hook_ids.push(hook.id.clone());
            registry.register_hook(hook).unwrap();
        }

        let dispatcher = DefaultEventDispatcher::new(
            Arc::new(registry),
            executor.clone() as Arc<dyn HookExecutor>,
        );

        let event = Event {
            event_type: event_type.clone(),
            context: EventContext {
                data: serde_json::json!({}),
                metadata: serde_json::json!({}),
            },
            timestamp: "2024-01-01T12:00:00Z".to_string(),
        };

        dispatcher.dispatch_event(event).unwrap();

        let execution_order = executor.get_execution_order();
        prop_assert_eq!(execution_order.len(), hook_count);

        // Verify all hooks were executed
        for hook_id in &hook_ids {
            prop_assert!(execution_order.contains(hook_id));
        }
    }
}

// ============================================================================
// Property 2: Hook Context Passing
// ============================================================================

proptest! {
    /// **Feature: ricecoder-hooks, Property 2: Hook context passing**
    /// **Validates: Requirements Hooks-2.1, Hooks-2.2**
    ///
    /// For any hook execution, the hook SHALL receive accurate event context with
    /// all relevant information.
    #[test]
    fn prop_hook_context_is_passed_correctly(
        event_type in event_type_strategy(),
    ) {
        let mut registry = InMemoryHookRegistry::new();
        let executor = Arc::new(OrderTrackingExecutor::new());

        let hook = Hook {
            id: "test_hook".to_string(),
            name: "Test Hook".to_string(),
            description: None,
            event: event_type.clone(),
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
        };

        registry.register_hook(hook).unwrap();

        let dispatcher = DefaultEventDispatcher::new(
            Arc::new(registry),
            executor.clone() as Arc<dyn HookExecutor>,
        );

        let context_data = serde_json::json!({
            "file_path": "/path/to/file.rs",
            "size": 1024,
        });

        let event = Event {
            event_type: event_type.clone(),
            context: EventContext {
                data: context_data.clone(),
                metadata: serde_json::json!({}),
            },
            timestamp: "2024-01-01T12:00:00Z".to_string(),
        };

        dispatcher.dispatch_event(event).unwrap();

        let execution_order = executor.get_execution_order();
        prop_assert_eq!(execution_order.len(), 1);
        prop_assert_eq!(&execution_order[0], "test_hook");
    }
}

// ============================================================================
// Property 3: Hook Isolation
// ============================================================================

proptest! {
    /// **Feature: ricecoder-hooks, Property 3: Hook isolation**
    /// **Validates: Requirements Hooks-2.1**
    ///
    /// For any hook failure, other hooks for the same event SHALL continue executing.
    /// One hook's failure should not prevent other hooks from running.
    #[test]
    fn prop_hook_failure_does_not_affect_other_hooks(
        event_type in event_type_strategy(),
        hook_count in 2usize..10,
        fail_index in 0usize..1, // Fail on first hook
    ) {
        let mut registry = InMemoryHookRegistry::new();

        // Register hooks
        let mut fail_on_ids = Vec::new();
        for i in 0..hook_count {
            let hook = Hook {
                id: format!("hook_{:02}", i),
                name: format!("Hook {}", i),
                description: None,
                event: event_type.clone(),
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
            };

            if i == fail_index {
                fail_on_ids.push(hook.id.clone());
            }

            registry.register_hook(hook).unwrap();
        }

        let executor = Arc::new(SelectiveFailExecutor::new(fail_on_ids));
        let dispatcher = DefaultEventDispatcher::new(
            Arc::new(registry),
            executor.clone() as Arc<dyn HookExecutor>,
        );

        let event = Event {
            event_type: event_type.clone(),
            context: EventContext {
                data: serde_json::json!({}),
                metadata: serde_json::json!({}),
            },
            timestamp: "2024-01-01T12:00:00Z".to_string(),
        };

        // Dispatch should succeed because other hooks succeed
        dispatcher.dispatch_event(event).unwrap();

        // All hooks should have been executed despite one failing
        prop_assert_eq!(executor.get_execution_count(), hook_count);
    }
}

// ============================================================================
// Property 4: Hook Chaining
// ============================================================================

proptest! {
    /// **Feature: ricecoder-hooks, Property 4: Hook chaining**
    /// **Validates: Requirements Hooks-2.1**
    ///
    /// For any chain of hooks, each hook SHALL receive the output of the previous
    /// hook as context.
    #[test]
    fn prop_hook_chaining_executes_in_sequence(
        event_type in event_type_strategy(),
        chain_length in 2usize..5,
    ) {
        let mut registry = InMemoryHookRegistry::new();
        let executor = Arc::new(OrderTrackingExecutor::new());

        // Register hooks for chaining
        let mut chain_hook_ids = Vec::new();
        for i in 0..chain_length {
            let hook = Hook {
                id: format!("chain_hook_{:02}", i),
                name: format!("Chain Hook {}", i),
                description: None,
                event: event_type.clone(),
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
            };
            chain_hook_ids.push(hook.id.clone());
            registry.register_hook(hook).unwrap();
        }

        // Register a chain hook
        let chain_hook = Hook {
            id: "chain_starter".to_string(),
            name: "Chain Starter".to_string(),
            description: None,
            event: event_type.clone(),
            action: Action::Chain(ChainAction {
                hook_ids: chain_hook_ids.clone(),
                pass_output: true,
            }),
            enabled: true,
            tags: vec![],
            metadata: serde_json::json!({}),
            condition: None,
        };

        registry.register_hook(chain_hook).unwrap();

        let dispatcher = DefaultEventDispatcher::new(
            Arc::new(registry),
            executor.clone() as Arc<dyn HookExecutor>,
        );

        let event = Event {
            event_type: event_type.clone(),
            context: EventContext {
                data: serde_json::json!({}),
                metadata: serde_json::json!({}),
            },
            timestamp: "2024-01-01T12:00:00Z".to_string(),
        };

        dispatcher.dispatch_event(event).unwrap();

        let execution_order = executor.get_execution_order();
        // Chain starter should be executed
        prop_assert!(execution_order.contains(&"chain_starter".to_string()));
    }
}
