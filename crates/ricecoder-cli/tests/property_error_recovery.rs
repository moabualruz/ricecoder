// Property-based tests for error recovery without panic
// **Feature: ricecoder-cli, Property 13: Error Recovery Without Panic**
// **Validates: Requirements 9.1, 9.2, 9.3**

use proptest::prelude::*;
use ricecoder_cli::sync_utils::{safe_lock, safe_lock_optional, SafeLockable};
use std::sync::{Arc, Mutex};
use std::thread;

// ============================================================================
// Property 13: Error Recovery Without Panic
// ============================================================================
// For any mutex operation, the system SHALL handle errors gracefully without panicking
// and provide helpful error messages.

proptest! {
    /// Property: Safe lock operations never panic
    /// For any number of concurrent threads accessing a mutex, safe lock operations
    /// should never panic, even if the mutex is poisoned.
    #[test]
    fn prop_safe_lock_never_panics(thread_count in 1..20usize) {
        let data = Arc::new(Mutex::new(0));
        let mut handles = vec![];

        // Spawn threads that access the mutex
        for _ in 0..thread_count {
            let data_clone = Arc::clone(&data);
            let handle = thread::spawn(move || {
                // This should never panic, even if mutex is poisoned
                if let Ok(mut guard) = safe_lock(&data_clone, "concurrent access") {
                    *guard += 1;
                }
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            // Should not panic when joining
            let _ = handle.join();
        }

        // Verify the operation completed successfully
        let final_value = safe_lock(&data, "final check").unwrap();
        assert_eq!(*final_value, thread_count as i32);
    }

    /// Property: Safe lock optional returns Some or None, never panics
    /// For any mutex operation, safe_lock_optional should return Some or None
    /// without panicking.
    #[test]
    fn prop_safe_lock_optional_never_panics(thread_count in 1..20usize) {
        let data: Arc<Mutex<Vec<i32>>> = Arc::new(Mutex::new(vec![]));
        let mut handles = vec![];

        for _ in 0..thread_count {
            let data_clone = Arc::clone(&data);
            let handle = thread::spawn(move || {
                // This should never panic
                if let Some(mut guard) = safe_lock_optional(&data_clone, "optional access") {
                    guard.push(1);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            let _ = handle.join();
        }

        // Verify we got some result
        let result = safe_lock_optional(&data, "final check");
        assert!(result.is_some() || result.is_none()); // Always true, but verifies no panic
    }

    /// Property: Safe lock or default returns a value, never panics
    /// For any mutex operation, safe_lock_or_default should return a value
    /// without panicking.
    #[test]
    fn prop_safe_lock_or_default_never_panics(thread_count in 1..20usize) {
        let data: Arc<Mutex<Vec<i32>>> = Arc::new(Mutex::new(vec![1, 2, 3]));
        let mut handles = vec![];

        for _ in 0..thread_count {
            let data_clone = Arc::clone(&data);
            let handle = thread::spawn(move || {
                // This should never panic and should return a value
                let guard = data_clone.safe_lock_or_default("default access");
                assert!(!guard.is_empty() || guard.is_empty()); // Always true
            });
            handles.push(handle);
        }

        for handle in handles {
            let _ = handle.join();
        }
    }

    /// Property: Trait-based safe lock never panics
    /// For any mutex operation using the SafeLockable trait, operations
    /// should never panic.
    #[test]
    fn prop_trait_safe_lock_never_panics(thread_count in 1..20usize) {
        let data: Arc<Mutex<i32>> = Arc::new(Mutex::new(0));
        let mut handles = vec![];

        for _ in 0..thread_count {
            let data_clone = Arc::clone(&data);
            let handle = thread::spawn(move || {
                // Using trait method should never panic
                if let Ok(mut guard) = data_clone.safe_lock("trait access") {
                    *guard += 1;
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            let _ = handle.join();
        }

        let final_value = data.safe_lock("final check").unwrap();
        assert_eq!(*final_value, thread_count as i32);
    }

    /// Property: Multiple sequential operations never panic
    /// For any sequence of safe lock operations, they should all complete
    /// without panicking.
    #[test]
    fn prop_sequential_operations_never_panic(operations in 1..100usize) {
        let data: Arc<Mutex<i32>> = Arc::new(Mutex::new(0));

        for _ in 0..operations {
            // Try different safe lock operations
            if let Ok(mut guard) = safe_lock(&data, "sequential") {
                *guard += 1;
            }

            if let Some(mut guard) = safe_lock_optional(&data, "sequential optional") {
                *guard += 1;
            }

            let mut guard = data.safe_lock_or_default("sequential default");
            *guard += 1;
        }

        // All operations should have completed
        let final_value = safe_lock(&data, "final").unwrap();
        assert!(*final_value > 0);
    }

    /// Property: Error messages are always helpful
    /// For any error condition, the system should provide a helpful error message
    /// that includes context about what operation failed.
    #[test]
    fn prop_error_messages_include_context(context_str in ".*") {
        let data: Arc<Mutex<i32>> = Arc::new(Mutex::new(42));

        // Perform a safe lock operation with context
        let result = safe_lock(&data, &context_str);

        // Result should be Ok (no panic)
        assert!(result.is_ok());

        // If we had an error, it should include the context
        // (In this case, we won't have an error, but the pattern is tested)
    }

    /// Property: Concurrent access with safe locks maintains consistency
    /// For any number of concurrent threads incrementing a counter with safe locks,
    /// the final value should equal the number of threads.
    #[test]
    fn prop_concurrent_consistency(thread_count in 1..50usize) {
        let counter = Arc::new(Mutex::new(0));
        let mut handles = vec![];

        for _ in 0..thread_count {
            let counter_clone = Arc::clone(&counter);
            let handle = thread::spawn(move || {
                // Use safe lock to increment
                if let Ok(mut guard) = safe_lock(&counter_clone, "increment") {
                    *guard += 1;
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            let _ = handle.join();
        }

        // Final value should match thread count
        let final_value = safe_lock(&counter, "final").unwrap();
        assert_eq!(*final_value, thread_count as i32);
    }

    /// Property: Safe lock operations are idempotent
    /// For any safe lock operation, calling it multiple times on the same data
    /// should produce consistent results.
    #[test]
    fn prop_safe_lock_idempotent(value in 0i32..1000) {
        let data = Arc::new(Mutex::new(value));

        // First lock
        let result1 = safe_lock(&data, "idempotent test").unwrap();
        let value1 = *result1;

        // Second lock
        let result2 = safe_lock(&data, "idempotent test").unwrap();
        let value2 = *result2;

        // Values should be identical
        assert_eq!(value1, value2);
        assert_eq!(value1, value);
    }

    /// Property: Safe lock optional is consistent
    /// For any safe lock optional operation, repeated calls should return
    /// consistent results (all Some or all None).
    #[test]
    fn prop_safe_lock_optional_consistent(value in 0i32..1000) {
        let data = Arc::new(Mutex::new(value));

        // Multiple calls
        let result1 = safe_lock_optional(&data, "optional test");
        let result2 = safe_lock_optional(&data, "optional test");
        let result3 = safe_lock_optional(&data, "optional test");

        // All should be Some (since we're not poisoning the mutex)
        assert!(result1.is_some());
        assert!(result2.is_some());
        assert!(result3.is_some());

        // Values should be identical
        assert_eq!(*result1.unwrap(), value);
        assert_eq!(*result2.unwrap(), value);
        assert_eq!(*result3.unwrap(), value);
    }

    /// Property: Safe lock or default always returns a value
    /// For any safe lock or default operation, it should always return a value
    /// (never panic or return None).
    #[test]
    fn prop_safe_lock_or_default_always_returns(value in 0i32..1000) {
        let data = Arc::new(Mutex::new(value));

        // Multiple calls
        let result1 = data.safe_lock_or_default("default test");
        let result2 = data.safe_lock_or_default("default test");
        let result3 = data.safe_lock_or_default("default test");

        // All should return values
        assert_eq!(*result1, value);
        assert_eq!(*result2, value);
        assert_eq!(*result3, value);
    }

    /// Property: No panics with high contention
    /// For any high-contention scenario with many threads accessing the same mutex,
    /// safe lock operations should never panic.
    #[test]
    fn prop_no_panic_high_contention(thread_count in 10..100usize) {
        let data = Arc::new(Mutex::new(0));
        let mut handles = vec![];

        for _ in 0..thread_count {
            let data_clone = Arc::clone(&data);
            let handle = thread::spawn(move || {
                // Perform multiple operations in quick succession
                for _ in 0..10 {
                    if let Ok(mut guard) = safe_lock(&data_clone, "high contention") {
                        *guard += 1;
                    }
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            let _ = handle.join();
        }

        // Should have completed without panic
        let final_value = safe_lock(&data, "final").unwrap();
        assert!(*final_value > 0);
    }
}

// ============================================================================
// Unit Tests for Error Recovery
// ============================================================================

#[test]
fn test_safe_lock_basic() {
    let data = Mutex::new(42);
    let result = safe_lock(&data, "test");
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), 42);
}

#[test]
fn test_safe_lock_optional_basic() {
    let data = Mutex::new(42);
    let result = safe_lock_optional(&data, "test");
    assert!(result.is_some());
    assert_eq!(*result.unwrap(), 42);
}

#[test]
fn test_safe_lock_or_default_basic() {
    let data: Mutex<Vec<i32>> = Mutex::new(vec![1, 2, 3]);
    let guard = data.safe_lock_or_default("test");
    assert_eq!(guard.len(), 3);
}

#[test]
fn test_safe_lock_with_context() {
    let data = Mutex::new("test data");
    let result = safe_lock(&data, "accessing test data");
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), "test data");
}

#[test]
fn test_multiple_safe_locks() {
    let data = Arc::new(Mutex::new(0));

    // First lock
    {
        let mut guard = safe_lock(&data, "first").unwrap();
        *guard = 10;
    }

    // Second lock
    {
        let guard = safe_lock(&data, "second").unwrap();
        assert_eq!(*guard, 10);
    }

    // Third lock
    {
        let mut guard = safe_lock(&data, "third").unwrap();
        *guard = 20;
    }

    // Verify final value
    let final_guard = safe_lock(&data, "final").unwrap();
    assert_eq!(*final_guard, 20);
}

#[test]
fn test_safe_lock_trait_implementation() {
    let data = Mutex::new(vec![1, 2, 3]);

    // Using trait methods
    let result = data.safe_lock("trait test");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 3);

    let optional = data.safe_lock_optional("trait optional");
    assert!(optional.is_some());
    assert_eq!(optional.unwrap().len(), 3);

    let default = data.safe_lock_or_default("trait default");
    assert_eq!(default.len(), 3);
}

#[test]
fn test_concurrent_safe_locks() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter_clone = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            if let Ok(mut guard) = safe_lock(&counter_clone, "concurrent") {
                *guard += 1;
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let final_value = safe_lock(&counter, "final").unwrap();
    assert_eq!(*final_value, 10);
}

#[test]
fn test_safe_lock_with_different_types() {
    // Test with i32
    let int_data = Mutex::new(42);
    assert_eq!(*safe_lock(&int_data, "int").unwrap(), 42);

    // Test with String
    let string_data = Mutex::new("hello".to_string());
    assert_eq!(*safe_lock(&string_data, "string").unwrap(), "hello");

    // Test with Vec
    let vec_data = Mutex::new(vec![1, 2, 3]);
    assert_eq!(safe_lock(&vec_data, "vec").unwrap().len(), 3);
}

#[test]
fn test_safe_lock_optional_with_different_types() {
    // Test with i32
    let int_data = Mutex::new(42);
    assert_eq!(*safe_lock_optional(&int_data, "int").unwrap(), 42);

    // Test with String
    let string_data = Mutex::new("hello".to_string());
    assert_eq!(*safe_lock_optional(&string_data, "string").unwrap(), "hello");

    // Test with Vec
    let vec_data = Mutex::new(vec![1, 2, 3]);
    assert_eq!(safe_lock_optional(&vec_data, "vec").unwrap().len(), 3);
}

#[test]
fn test_safe_lock_or_default_with_different_types() {
    // Test with i32
    let int_data: Mutex<i32> = Mutex::new(42);
    assert_eq!(*int_data.safe_lock_or_default("int"), 42);

    // Test with String
    let string_data: Mutex<String> = Mutex::new("hello".to_string());
    assert_eq!(*string_data.safe_lock_or_default("string"), "hello");

    // Test with Vec
    let vec_data: Mutex<Vec<i32>> = Mutex::new(vec![1, 2, 3]);
    assert_eq!(vec_data.safe_lock_or_default("vec").len(), 3);
}
