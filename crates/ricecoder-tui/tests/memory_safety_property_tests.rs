//! Memory Safety Property Tests
//!
//! This module contains property-based tests focused on memory safety validation,
//! ensuring that all operations maintain memory safety guarantees and prevent
//! common memory-related vulnerabilities.

use proptest::prelude::*;
use ricecoder_tui::{
    get_tui_lifecycle_manager, initialize_tui_lifecycle_manager, PerformanceProfiler,
    TuiLifecycleManager,
};
use std::sync::{Arc, Mutex};

/// Test that the global lifecycle manager is safely initialized and accessed
proptest! {
    #[test]
    fn test_lifecycle_manager_thread_safety(initializations in 1..10usize) {
        // Test multiple initializations (should only succeed once due to OnceLock)
        for _ in 0..initializations {
            let _manager = initialize_tui_lifecycle_manager();
        }

        // Verify we can safely get the manager
        let manager = get_tui_lifecycle_manager();
        prop_assert!(manager.is_some());

        // Test concurrent access
        let manager_arc = manager.unwrap();
        let handles: Vec<_> = (0..initializations).map(|_| {
            let manager_clone = Arc::clone(&manager_arc);
            std::thread::spawn(move || {
                // Simulate concurrent operations
                let _state = manager_clone.get_all_component_states();
                true
            })
        }).collect();

        for handle in handles {
            prop_assert!(handle.join().unwrap());
        }
    }
}

/// Test that performance profiler handles are memory safe
proptest! {
    #[test]
    fn test_performance_profiler_memory_safety(span_names in prop::collection::vec("[a-zA-Z0-9_]{1,32}", 1..20)) {
        let profiler = Arc::new(Mutex::new(PerformanceProfiler::new()));

        // Create multiple spans
        let mut handles = Vec::new();
        for name in span_names {
            if let Some(handle) = PerformanceProfiler::start_span(&profiler, &name) {
                handles.push(handle);
            }
        }

        // Spans should be automatically ended when handles are dropped
        drop(handles);

        // Verify profiler state is consistent
        let profiler_guard = profiler.lock().unwrap();
        let stats = profiler_guard.stats();

        // All spans should be completed
        prop_assert_eq!(stats.active_spans, 0);
        prop_assert!(stats.total_spans >= span_names.len() as usize);
    }
}

/// Test memory safety invariants for component registration
proptest! {
    #[test]
    fn test_component_registration_memory_safety(
        component_names in prop::collection::vec("[a-zA-Z0-9_]{1,32}", 1..10),
        dependency_counts in prop::collection::vec(0..5usize, 1..10)
    ) {
        let manager = TuiLifecycleManager::new();

        // Register components with dependencies
        for (i, name) in component_names.iter().enumerate() {
            let deps: Vec<String> = (0..dependency_counts[i]).map(|j| format!("dep_{}", j)).collect();

            // Create a mock component
            struct MockComponent(String);
            impl ricecoder_tui::lifecycle::TuiLifecycleComponent for MockComponent {
                fn name(&self) -> &'static str { &self.0 }
            }

            let component = MockComponent(name.clone());
            manager.register_component(component, deps).unwrap();
        }

        // Verify all components are registered
        let states = manager.get_all_component_states();
        prop_assert_eq!(states.len(), component_names.len());

        // Test that component names are preserved
        for name in &component_names {
            prop_assert!(states.iter().any(|(n, _)| n == name));
        }
    }
}

/// Test that concurrent profiler operations don't cause memory corruption
proptest! {
    #[test]
    fn test_concurrent_profiler_operations(thread_count in 1..8usize, operations_per_thread in 1..50usize) {
        let profiler = Arc::new(Mutex::new(PerformanceProfiler::new()));

        let handles: Vec<_> = (0..thread_count).map(|thread_id| {
            let profiler_clone = Arc::clone(&profiler);
            std::thread::spawn(move || {
                for op in 0..operations_per_thread {
                    let span_name = format!("thread_{}_op_{}", thread_id, op);

                    if let Some(_handle) = PerformanceProfiler::start_span(&profiler_clone, &span_name) {
                        // Handle is automatically dropped, ending the span
                    }

                    // Also test stats access
                    {
                        let profiler_guard = profiler_clone.lock().unwrap();
                        let _stats = profiler_guard.stats();
                    }
                }
                true
            })
        }).collect();

        // Wait for all threads
        for handle in handles {
            prop_assert!(handle.join().unwrap());
        }

        // Verify final state
        let profiler_guard = profiler.lock().unwrap();
        let stats = profiler_guard.stats();
        prop_assert_eq!(stats.active_spans, 0);
        prop_assert!(stats.total_spans >= (thread_count * operations_per_thread) as usize);
    }
}

/// Test memory safety of lifecycle manager with invalid operations
proptest! {
    #[test]
    fn test_lifecycle_manager_invalid_operations(invalid_names in prop::collection::vec("[^a-zA-Z0-9_]{1,32}", 1..10)) {
        let manager = TuiLifecycleManager::new();

        // Try to get states for non-existent components
        for name in invalid_names {
            let state = manager.get_component_state(&name);
            prop_assert!(state.is_none());
        }

        // Verify manager remains in valid state
        let all_states = manager.get_all_component_states();
        prop_assert!(all_states.is_empty());
    }
}

/// Test that profiler handles prevent use-after-free
proptest! {
    #[test]
    fn test_profiler_handle_lifetime_safety(span_count in 1..100usize) {
        let profiler = Arc::new(Mutex::new(PerformanceProfiler::new()));

        // Create a scope where handles exist
        {
            let mut handles = Vec::new();
            for i in 0..span_count {
                let span_name = format!("span_{}", i);
                if let Some(handle) = PerformanceProfiler::start_span(&profiler, &span_name) {
                    handles.push(handle);
                }
            }

            // Verify spans are active
            {
                let profiler_guard = profiler.lock().unwrap();
                let stats = profiler_guard.stats();
                prop_assert!(stats.active_spans > 0);
            }

            // Handles go out of scope here, spans should be ended
        }

        // Verify all spans are now ended
        let profiler_guard = profiler.lock().unwrap();
        let stats = profiler_guard.stats();
        prop_assert_eq!(stats.active_spans, 0);
    }
}
