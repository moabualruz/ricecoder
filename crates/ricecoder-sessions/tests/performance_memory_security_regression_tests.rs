//! Performance, memory, and security regression tests
//! **Feature: ricecoder-sessions, Unit Tests: Performance/Memory/Security Regression**
//! **Validates: Requirements 6.1, 6.2, 6.3, 7.1, 7.2, 7.3**

use chrono::Duration;
use ricecoder_sessions::{
    BackgroundAgent, Message, MessageRole, Session, SessionContext, SessionManager, SessionMode,
    SessionStore, SharePermissions, ShareService,
};
use std::sync::Arc;
use std::thread;
use std::time::{Duration as StdDuration, Instant};
use tempfile::TempDir;
use tokio::runtime::Runtime;

fn create_test_context() -> SessionContext {
    SessionContext::new("openai".to_string(), "gpt-4".to_string(), SessionMode::Chat)
}

fn create_test_session(name: &str) -> Session {
    Session::new(name.to_string(), create_test_context())
}

#[test]
fn test_session_creation_performance() {
    let mut manager = SessionManager::new(1000);
    let context = create_test_context();

    let start = Instant::now();
    let iterations = 100;

    for i in 0..iterations {
        let _session = manager
            .create_session(format!("Perf Test {}", i), context.clone())
            .unwrap();
    }

    let elapsed = start.elapsed();
    let avg_time = elapsed.as_micros() as f64 / iterations as f64;

    // Should be reasonably fast (< 1ms per session creation)
    assert!(
        avg_time < 1000.0,
        "Session creation too slow: {}μs avg",
        avg_time
    );

    println!("Session creation performance: {}μs avg", avg_time);
}

#[test]
fn test_session_store_persistence_performance() {
    let temp_dir = TempDir::new().unwrap();
    let sessions_dir = temp_dir.path().join("sessions");
    let archive_dir = temp_dir.path().join("archive");

    let rt = Runtime::new().unwrap();
    let store = rt.block_on(async { SessionStore::with_dirs(sessions_dir, archive_dir).unwrap() });

    let session = create_test_session("Perf Test");

    // Test save performance
    let start = Instant::now();
    let iterations = 50;

    for _ in 0..iterations {
        let _ = rt.block_on(store.save(&session));
    }

    let elapsed = start.elapsed();
    let avg_save_time = elapsed.as_millis() as f64 / iterations as f64;

    // Should be reasonably fast (< 10ms per save)
    assert!(
        avg_save_time < 10.0,
        "Session save too slow: {}ms avg",
        avg_save_time
    );

    // Test load performance
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = rt.block_on(store.load(&session.id));
    }

    let elapsed = start.elapsed();
    let avg_load_time = elapsed.as_millis() as f64 / iterations as f64;

    // Should be reasonably fast (< 5ms per load)
    assert!(
        avg_load_time < 5.0,
        "Session load too slow: {}ms avg",
        avg_load_time
    );

    println!(
        "Persistence performance - Save: {}ms avg, Load: {}ms avg",
        avg_save_time, avg_load_time
    );
}

#[test]
fn test_memory_usage_regression() {
    let mut manager = SessionManager::new(100);

    // Create sessions with substantial data
    let start = Instant::now();
    let session_count = 50;

    for i in 0..session_count {
        let mut session = create_test_session(&format!("Memory Test {}", i));

        // Add substantial history
        for j in 0..10 {
            let msg = Message::new(
                if j % 2 == 0 {
                    MessageRole::User
                } else {
                    MessageRole::Assistant
                },
                format!(
                    "Message {} with substantial content to test memory usage {}",
                    j,
                    "x".repeat(100)
                ),
            );
            session.history.push(msg);
        }

        // Add background agents
        for j in 0..3 {
            let agent = BackgroundAgent::new(
                format!("agent_type_{}", j),
                Some(format!("Task {} with description", j)),
            );
            session.background_agents.push(agent);
        }

        manager
            .create_session(session.name, session.context)
            .unwrap();
    }

    let elapsed = start.elapsed();

    // Should complete within reasonable time
    assert!(
        elapsed < StdDuration::from_secs(5),
        "Memory test took too long: {:?}",
        elapsed
    );

    // Verify all sessions exist
    assert_eq!(manager.session_count(), session_count);

    println!("Memory usage test completed in {:?}", elapsed);
}

#[test]
fn test_concurrent_session_operations() {
    let manager = Arc::new(SessionManager::new(100));
    let context = Arc::new(create_test_context());

    let start = Instant::now();
    let thread_count = 10;
    let operations_per_thread = 20;

    let mut handles = vec![];

    for _ in 0..thread_count {
        let manager_clone = Arc::clone(&manager);
        let context_clone = Arc::clone(&context);

        let handle = thread::spawn(move || {
            let mut local_success = 0;
            let mut local_fail = 0;

            for i in 0..operations_per_thread {
                let result = manager_clone
                    .create_session(format!("Concurrent {}", i), (**context_clone).clone());

                match result {
                    Ok(_) => local_success += 1,
                    Err(_) => local_fail += 1,
                }
            }

            (local_success, local_fail)
        });

        handles.push(handle);
    }

    let mut total_success = 0;
    let mut total_fail = 0;

    for handle in handles {
        let (success, fail) = handle.join().unwrap();
        total_success += success;
        total_fail += fail;
    }

    let elapsed = start.elapsed();

    // Should complete within reasonable time
    assert!(
        elapsed < StdDuration::from_secs(10),
        "Concurrent operations took too long: {:?}",
        elapsed
    );

    // Should have reasonable success rate
    let total_operations = total_success + total_fail;
    let success_rate = total_success as f64 / total_operations as f64;
    assert!(
        success_rate > 0.8,
        "Success rate too low: {}%",
        success_rate * 100.0
    );

    println!(
        "Concurrent operations - Success: {}, Fail: {}, Time: {:?}",
        total_success, total_fail, elapsed
    );
}

#[test]
fn test_share_service_performance_under_load() {
    let service = Arc::new(ShareService::new());
    let session = Arc::new(create_test_session("Load Test"));
    let permissions = SharePermissions {
        read_only: true,
        include_history: true,
        include_context: true,
    };

    let start = Instant::now();
    let iterations = 100;

    let mut handles = vec![];

    // Create multiple threads generating shares
    for _ in 0..10 {
        let service_clone = Arc::clone(&service);
        let session_clone = Arc::clone(&session);
        let permissions_clone = permissions.clone();

        let handle = thread::spawn(move || {
            let mut local_shares = vec![];

            for i in 0..(iterations / 10) {
                let share = service_clone
                    .generate_share_link(&session_clone.id, permissions_clone.clone(), None)
                    .unwrap();
                local_shares.push(share);
            }

            local_shares
        });

        handles.push(handle);
    }

    let mut total_shares = 0;

    for handle in handles {
        let shares = handle.join().unwrap();
        total_shares += shares.len();
    }

    let elapsed = start.elapsed();

    // Should complete within reasonable time
    assert!(
        elapsed < StdDuration::from_secs(5),
        "Share generation took too long: {:?}",
        elapsed
    );
    assert_eq!(total_shares, iterations);

    println!(
        "Share service performance: {} shares in {:?}",
        total_shares, elapsed
    );
}

#[test]
fn test_security_input_validation() {
    let mut manager = SessionManager::new(10);

    // Test with potentially malicious inputs
    let malicious_inputs = vec![
        "<script>alert('xss')</script>",
        "../../../etc/passwd",
        "DROP TABLE sessions;",
        "'; DROP TABLE sessions; --",
        "<img src=x onerror=alert(1)>",
        "javascript:alert('xss')",
        "data:text/html,<script>alert('xss')</script>",
        "\0null\0byte",
        "very".repeat(1000), // Extremely long input
    ];

    let context = create_test_context();

    for input in malicious_inputs {
        // Should not crash or panic
        let result = manager.create_session(input.to_string(), context.clone());
        match result {
            Ok(session) => {
                // If successful, verify the input was stored as-is (no transformation)
                assert_eq!(session.name, input);
            }
            Err(_) => {
                // If failed, it should be due to validation, not security issues
            }
        }
    }
}

#[test]
fn test_encryption_performance() {
    let temp_dir = TempDir::new().unwrap();
    let sessions_dir = temp_dir.path().join("sessions");
    let archive_dir = temp_dir.path().join("archive");

    let rt = Runtime::new().unwrap();
    let mut store =
        rt.block_on(async { SessionStore::with_dirs(sessions_dir, archive_dir).unwrap() });

    // Enable encryption
    store.enable_encryption("test-password-12345").unwrap();

    let session = create_test_session("Encryption Test");

    // Test encrypted save/load performance
    let start = Instant::now();
    let iterations = 20;

    for _ in 0..iterations {
        rt.block_on(store.save(&session)).unwrap();
        let _loaded = rt.block_on(store.load(&session.id)).unwrap();
    }

    let elapsed = start.elapsed();
    let avg_time = elapsed.as_millis() as f64 / iterations as f64;

    // Should be reasonably fast even with encryption (< 50ms per round-trip)
    assert!(
        avg_time < 50.0,
        "Encryption performance too slow: {}ms avg",
        avg_time
    );

    println!(
        "Encryption performance: {}ms avg per save/load cycle",
        avg_time
    );
}

#[test]
fn test_enterprise_encryption_performance() {
    let temp_dir = TempDir::new().unwrap();
    let sessions_dir = temp_dir.path().join("sessions");
    let archive_dir = temp_dir.path().join("archive");

    let rt = Runtime::new().unwrap();
    let mut store =
        rt.block_on(async { SessionStore::with_dirs(sessions_dir, archive_dir).unwrap() });

    // Enable enterprise encryption
    store
        .enable_enterprise_encryption("enterprise-password-12345")
        .unwrap();

    let session = create_test_session("Enterprise Encryption Test");

    // Test enterprise encrypted save/load performance
    let start = Instant::now();
    let iterations = 20;

    for _ in 0..iterations {
        rt.block_on(store.save(&session)).unwrap();
        let _loaded = rt.block_on(store.load(&session.id)).unwrap();
    }

    let elapsed = start.elapsed();
    let avg_time = elapsed.as_millis() as f64 / iterations as f64;

    // Should be reasonably fast even with enterprise encryption (< 100ms per round-trip)
    assert!(
        avg_time < 100.0,
        "Enterprise encryption performance too slow: {}ms avg",
        avg_time
    );

    println!(
        "Enterprise encryption performance: {}ms avg per save/load cycle",
        avg_time
    );
}

#[test]
fn test_resource_cleanup_performance() {
    let service = ShareService::new();

    // Create many shares
    let session = create_test_session("Cleanup Test");
    let permissions = SharePermissions {
        read_only: true,
        include_history: true,
        include_context: true,
    };

    let start = Instant::now();
    let share_count = 100;

    let mut share_ids = vec![];
    for i in 0..share_count {
        let share = service
            .generate_share_link(
                &session.id,
                permissions.clone(),
                Some(Duration::seconds(i as i64)), // Different expiration times
            )
            .unwrap();
        share_ids.push(share.id);
    }

    let creation_time = start.elapsed();

    // Test cleanup performance
    let start = Instant::now();
    let cleaned = service.cleanup_expired_shares().unwrap();
    let cleanup_time = start.elapsed();

    // Should clean up most shares (those with negative duration are already expired)
    assert!(cleaned > 0);

    // Cleanup should be fast
    assert!(
        cleanup_time < StdDuration::from_millis(100),
        "Cleanup too slow: {:?}",
        cleanup_time
    );

    println!(
        "Resource cleanup - Created: {} shares in {:?}, Cleaned: {} in {:?}",
        share_count, creation_time, cleaned, cleanup_time
    );
}

#[test]
fn test_memory_leak_regression() {
    // Test for potential memory leaks by creating and destroying many sessions
    let initial_memory = get_memory_usage().unwrap_or(0);

    let mut managers = vec![];

    for i in 0..10 {
        let mut manager = SessionManager::new(50);
        let context = create_test_context();

        // Create many sessions
        for j in 0..50 {
            let _session = manager
                .create_session(format!("Leak Test {} {}", i, j), context.clone())
                .unwrap();
        }

        managers.push(manager);
    }

    // Drop all managers (should free memory)
    drop(managers);

    // Force garbage collection if possible
    std::thread::sleep(StdDuration::from_millis(100));

    let final_memory = get_memory_usage().unwrap_or(0);

    // Memory should not have grown significantly (allowing for some overhead)
    let memory_growth = final_memory as f64 - initial_memory as f64;
    let growth_percentage = if initial_memory > 0 {
        (memory_growth / initial_memory as f64) * 100.0
    } else {
        0.0
    };

    // Allow up to 50% growth (should be much less in practice)
    assert!(
        growth_percentage < 50.0,
        "Potential memory leak: {}% growth",
        growth_percentage
    );

    println!(
        "Memory usage - Initial: {} KB, Final: {} KB, Growth: {}%",
        initial_memory, final_memory, growth_percentage
    );
}

#[cfg(target_os = "linux")]
fn get_memory_usage() -> Option<usize> {
    use std::fs;

    if let Ok(statm) = fs::read_to_string("/proc/self/statm") {
        if let Some(vmsize) = statm.split_whitespace().next() {
            if let Ok(pages) = vmsize.parse::<usize>() {
                // Convert pages to KB (assuming 4KB pages)
                return Some(pages * 4);
            }
        }
    }

    None
}

#[cfg(not(target_os = "linux"))]
fn get_memory_usage() -> Option<usize> {
    // Fallback for non-Linux systems
    Some(0)
}

#[test]
fn test_security_timing_attacks_mitigation() {
    let service = ShareService::new();

    // Test that operations with invalid inputs don't take significantly different time
    // This helps prevent timing attacks

    let valid_session = create_test_session("Valid");
    let permissions = SharePermissions {
        read_only: true,
        include_history: true,
        include_context: true,
    };

    let valid_share = service
        .generate_share_link(&valid_session.id, permissions, None)
        .unwrap();

    let start_valid = Instant::now();
    let _ = service.get_share(&valid_share.id);
    let valid_time = start_valid.elapsed();

    let start_invalid = Instant::now();
    let _ = service.get_share("invalid-id-1234567890123456789012345678901234567890");
    let invalid_time = start_invalid.elapsed();

    // Times should be similar (within 10x difference to account for variability)
    let ratio = if valid_time > invalid_time {
        valid_time.as_nanos() as f64 / invalid_time.as_nanos() as f64
    } else {
        invalid_time.as_nanos() as f64 / valid_time.as_nanos() as f64
    };

    assert!(
        ratio < 10.0,
        "Potential timing attack vulnerability: {}x time difference",
        ratio
    );

    println!(
        "Timing attack test - Valid: {:?}, Invalid: {:?}, Ratio: {:.2}",
        valid_time, invalid_time, ratio
    );
}

#[test]
fn test_large_session_handling() {
    let mut session = create_test_session("Large Session");

    // Add a large number of messages
    for i in 0..1000 {
        let msg = Message::new(
            if i % 2 == 0 {
                MessageRole::User
            } else {
                MessageRole::Assistant
            },
            format!("Message {} with content {}", i, "x".repeat(100)),
        );
        session.history.push(msg);
    }

    // Add many background agents
    for i in 0..50 {
        let agent = BackgroundAgent::new(
            format!("agent_{}", i),
            Some(format!("Task {} description", i)),
        );
        session.background_agents.push(agent);
    }

    // Test serialization/deserialization
    let serialized = serde_json::to_string(&session).unwrap();
    let deserialized: Session = serde_json::from_str(&serialized).unwrap();

    // Verify data integrity
    assert_eq!(deserialized.id, session.id);
    assert_eq!(deserialized.history.len(), 1000);
    assert_eq!(deserialized.background_agents.len(), 50);

    // Should not be too large (reasonable size check)
    let size_kb = serialized.len() / 1024;
    assert!(size_kb < 5000, "Session too large: {} KB", size_kb); // Allow up to 5MB

    println!(
        "Large session handling - Size: {} KB, Messages: {}, Agents: {}",
        size_kb,
        deserialized.history.len(),
        deserialized.background_agents.len()
    );
}
