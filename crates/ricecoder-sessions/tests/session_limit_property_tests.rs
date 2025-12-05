//! Property-based tests for session limit enforcement
//! **Feature: ricecoder-sessions, Property 4: Session Limit Enforcement**
//! **Validates: Requirements 1.1, 1.5**

use proptest::prelude::*;
use ricecoder_sessions::{SessionManager, SessionContext, SessionMode};

/// Property: For any configured session limit, the SessionManager SHALL reject creation
/// of new sessions when the limit is reached.
///
/// This property tests that:
/// 1. Sessions can be created up to the limit
/// 2. Creating a session beyond the limit fails
/// 3. The error indicates the limit was reached
#[test]
fn prop_session_limit_enforcement() {
    proptest!(|(limit in 1usize..=10, num_sessions in 0usize..=15)| {
        let mut manager = SessionManager::new(limit);
        let context = SessionContext::new(
            "openai".to_string(),
            "gpt-4".to_string(),
            SessionMode::Chat,
        );

        // Try to create num_sessions sessions
        let mut created_count = 0;
        for i in 0..num_sessions {
            let result = manager.create_session(
                format!("Session {}", i),
                context.clone(),
            );

            match result {
                Ok(_) => {
                    created_count += 1;
                    // Should not exceed limit
                    prop_assert!(created_count <= limit, "Created more sessions than limit");
                }
                Err(e) => {
                    // Error should only occur when limit is reached
                    prop_assert!(
                        created_count >= limit,
                        "Got error before reaching limit: {:?}",
                        e
                    );
                }
            }
        }

        // Verify that we created exactly min(num_sessions, limit) sessions
        let expected_count = std::cmp::min(num_sessions, limit);
        prop_assert_eq!(
            manager.session_count(),
            expected_count,
            "Session count mismatch"
        );

        // Verify that is_limit_reached is accurate
        if manager.session_count() >= limit {
            prop_assert!(manager.is_limit_reached(), "Limit should be reached");
        } else {
            prop_assert!(!manager.is_limit_reached(), "Limit should not be reached");
        }
    });
}

/// Property: When the session limit is reached, attempting to create a new session
/// SHALL return a LimitReached error with the correct limit value.
#[test]
fn prop_session_limit_error_message() {
    proptest!(|(limit in 1usize..=10)| {
        let mut manager = SessionManager::new(limit);
        let context = SessionContext::new(
            "openai".to_string(),
            "gpt-4".to_string(),
            SessionMode::Chat,
        );

        // Fill up to the limit
        for i in 0..limit {
            manager
                .create_session(format!("Session {}", i), context.clone())
                .expect("Should create session within limit");
        }

        // Try to create one more
        let result = manager.create_session("Extra Session".to_string(), context);

        // Should fail with LimitReached error
        prop_assert!(result.is_err(), "Should fail when limit is reached");

        // Check the error message contains the limit
        let error_msg = format!("{:?}", result.unwrap_err());
        prop_assert!(
            error_msg.contains(&limit.to_string()),
            "Error message should contain the limit value"
        );
    });
}

/// Property: After deleting a session, a new session can be created even if
/// the limit was previously reached.
#[test]
fn prop_session_limit_after_deletion() {
    proptest!(|(limit in 1usize..=5)| {
        let mut manager = SessionManager::new(limit);
        let context = SessionContext::new(
            "openai".to_string(),
            "gpt-4".to_string(),
            SessionMode::Chat,
        );

        // Fill up to the limit
        let mut session_ids = Vec::new();
        for i in 0..limit {
            let session = manager
                .create_session(format!("Session {}", i), context.clone())
                .expect("Should create session within limit");
            session_ids.push(session.id);
        }

        // Verify limit is reached
        prop_assert!(manager.is_limit_reached(), "Limit should be reached");

        // Delete the first session
        if !session_ids.is_empty() {
            manager
                .delete_session(&session_ids[0])
                .expect("Should delete session");

            // Verify limit is no longer reached
            prop_assert!(!manager.is_limit_reached(), "Limit should not be reached after deletion");

            // Should be able to create a new session
            let result = manager.create_session("New Session".to_string(), context);
            prop_assert!(result.is_ok(), "Should create session after deletion");
        }
    });
}
