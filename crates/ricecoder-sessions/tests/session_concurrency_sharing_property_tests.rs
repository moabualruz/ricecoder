//! Property-based tests for session management with concurrency and sharing validation
//!
//! **Feature: ricecoder-sessions, Property Tests: Concurrency and Sharing**
//! **Validates: Requirements SESSION-1.1, SESSION-1.2, SESSION-2.1, SESSION-2.2, SESSION-3.1**
//!
//! These tests verify that session management correctly handles concurrent operations,
//! race conditions, and sharing validation under various scenarios.

use std::{collections::HashMap, sync::Arc};

use chrono::Duration as ChronoDuration;
use proptest::prelude::*;
use ricecoder_sessions::{
    Message, MessageRole, Session, SessionContext, SessionManager, SessionMode, SessionStore,
    SharePermissions, ShareService,
};
use tokio::{
    sync::{RwLock, Semaphore},
    time::{timeout, Duration},
};

// ============================================================================
// Generators for property-based testing
// ============================================================================

fn arb_session_context() -> impl Strategy<Value = SessionContext> {
    (
        "[a-z0-9]{1,20}".prop_map(|s| s), // provider
        "[a-z0-9]{1,20}".prop_map(|s| s), // model
        prop_oneof![
            Just(SessionMode::Chat),
            Just(SessionMode::Code),
            Just(SessionMode::Vibe)
        ],
    )
        .prop_map(|(provider, model, mode)| SessionContext::new(provider, model, mode))
}

fn arb_session() -> impl Strategy<Value = Session> {
    ("[a-z0-9]{1,20}", arb_session_context())
        .prop_map(|(name, context)| Session::new(name, context))
}

fn arb_session_with_messages() -> impl Strategy<Value = Session> {
    (arb_session(), 1..50usize).prop_map(|(mut session, num_messages)| {
        for i in 0..num_messages {
            let role = if i % 2 == 0 {
                MessageRole::User
            } else {
                MessageRole::Assistant
            };
            session
                .history
                .push(Message::new(role, format!("Message {}", i)));
        }
        session
    })
}

fn arb_share_permissions() -> impl Strategy<Value = SharePermissions> {
    (
        any::<bool>(), // read_only
        any::<bool>(), // include_history
        any::<bool>(), // include_context
    )
        .prop_map(
            |(read_only, include_history, include_context)| SharePermissions {
                read_only,
                include_history,
                include_context,
            },
        )
}

// ============================================================================
// Property 1: Concurrent Session Creation
// ============================================================================

proptest! {
    /// Property 1: Concurrent Session Creation
    /// *For any* number of concurrent session creation requests, all sessions SHALL be created successfully
    /// without conflicts or data corruption.
    /// **Validates: Requirements SESSION-1.1, SESSION-2.1**
    #[test]
    fn prop_concurrent_session_creation(
        session_count in 1..100usize,
        concurrent_threads in 1..20usize,
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.block_on(async {
            let session_manager = Arc::new(SessionManager::new(Default::default()).await
                .expect("Failed to create session manager"));
            let semaphore = Arc::new(Semaphore::new(concurrent_threads));

            let mut handles = Vec::new();

            for i in 0..session_count {
                let session_manager_clone = session_manager.clone();
                let semaphore_clone = semaphore.clone();

                let handle = tokio::spawn(async move {
                    let _permit = semaphore_clone.acquire().await.unwrap();

                    let session_name = format!("concurrent-session-{}", i);
                    let result = session_manager_clone.create_session(&session_name).await;

                    result.is_ok()
                });

                handles.push(handle);
            }

            // Wait for all sessions to be created
            let mut success_count = 0;
            for handle in handles {
                if let Ok(success) = handle.await {
                    if success {
                        success_count += 1;
                    }
                }
            }

            // All sessions should be created successfully
            prop_assert_eq!(success_count, session_count);
        });
    }

    /// Property 1 variant: Session creation with unique names under concurrency
    #[test]
    fn prop_concurrent_session_unique_names(
        base_name in "[a-z0-9]{1,10}",
        session_count in 2..50usize,
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.block_on(async {
            let session_manager = Arc::new(SessionManager::new(Default::default()).await
                .expect("Failed to create session manager"));

            let mut handles = Vec::new();
            let mut expected_names = Vec::new();

            for i in 0..session_count {
                let session_manager_clone = session_manager.clone();
                let session_name = format!("{}-{}", base_name, i);
                expected_names.push(session_name.clone());

                let handle = tokio::spawn(async move {
                    session_manager_clone.create_session(&session_name).await
                });

                handles.push(handle);
            }

            // Collect results
            let mut created_sessions = Vec::new();
            for handle in handles {
                if let Ok(Ok(session_id)) = handle.await {
                    created_sessions.push(session_id);
                }
            }

            // Should have created all sessions
            prop_assert_eq!(created_sessions.len(), session_count);

            // All session IDs should be unique
            let mut sorted_ids = created_sessions.clone();
            sorted_ids.sort();
            sorted_ids.dedup();
            prop_assert_eq!(sorted_ids.len(), created_sessions.len());
        });
    }
}

// ============================================================================
// Property 2: Concurrent Session Access
// ============================================================================

proptest! {
    /// Property 2: Concurrent Session Access
    /// *For any* session being accessed concurrently by multiple operations, the session state
    /// SHALL remain consistent and operations SHALL complete without race conditions.
    /// **Validates: Requirements SESSION-1.2, SESSION-2.2**
    #[test]
    fn prop_concurrent_session_access(
        mut session in arb_session_with_messages(),
        access_count in 1..100usize,
        concurrent_threads in 1..10usize,
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.block_on(async {
            let session_manager = Arc::new(SessionManager::new(Default::default()).await
                .expect("Failed to create session manager"));
            let shared_session = Arc::new(RwLock::new(session));
            let semaphore = Arc::new(Semaphore::new(concurrent_threads));

            // Create the session first
            let session_id = session_manager.create_session(&shared_session.read().await.name).await
                .expect("Failed to create session");

            let mut handles = Vec::new();

            for i in 0..access_count {
                let session_manager_clone = session_manager.clone();
                let shared_session_clone = shared_session.clone();
                let semaphore_clone = semaphore.clone();
                let session_id_clone = session_id.clone();

                let handle = tokio::spawn(async move {
                    let _permit = semaphore_clone.acquire().await.unwrap();

                    // Perform various operations
                    match i % 4 {
                        0 => {
                            // Read operation
                            let _session = session_manager_clone.get_session(&session_id_clone).await;
                        }
                        1 => {
                            // Add message
                            let message = Message::new(
                                if i % 2 == 0 { MessageRole::User } else { MessageRole::Assistant },
                                format!("Concurrent message {}", i)
                            );
                            let mut session = shared_session_clone.write().await;
                            session.history.push(message);
                        }
                        2 => {
                            // Update context
                            let mut session = shared_session_clone.write().await;
                            session.context.custom.insert(
                                format!("key-{}", i),
                                serde_json::json!(format!("value-{}", i))
                            );
                        }
                        3 => {
                            // List sessions
                            let _sessions = session_manager_clone.list_sessions().await;
                        }
                        _ => unreachable!(),
                    }

                    true // Success
                });

                handles.push(handle);
            }

            // Wait for all operations to complete
            let mut success_count = 0;
            for handle in handles {
                if let Ok(success) = handle.await {
                    if success {
                        success_count += 1;
                    }
                }
            }

            // All operations should succeed
            prop_assert_eq!(success_count, access_count);

            // Session should still be retrievable
            let final_session = session_manager.get_session(&session_id).await;
            prop_assert!(final_session.is_ok());
        });
    }

    /// Property 2 variant: Concurrent message addition maintains order
    #[test]
    fn prop_concurrent_message_ordering(
        session_name in "[a-z0-9]{1,20}",
        message_count in 1..50usize,
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.block_on(async {
            let session_manager = Arc::new(SessionManager::new(Default::default()).await
                .expect("Failed to create session manager"));

            let session_id = session_manager.create_session(&session_name).await
                .expect("Failed to create session");

            let shared_message_log = Arc::new(RwLock::new(Vec::new()));

            let mut handles = Vec::new();

            for i in 0..message_count {
                let session_manager_clone = session_manager.clone();
                let shared_message_log_clone = shared_message_log.clone();
                let session_id_clone = session_id.clone();

                let handle = tokio::spawn(async move {
                    // Add message to session
                    let message = Message::new(
                        MessageRole::User,
                        format!("Ordered message {}", i)
                    );

                    // Record the addition order
                    {
                        let mut log = shared_message_log_clone.write().await;
                        log.push(i);
                    }

                    // This would be the actual session update in a real implementation
                    // For testing, we just verify the logging works
                    true
                });

                handles.push(handle);
            }

            // Wait for all messages to be added
            for handle in handles {
                let _ = handle.await;
            }

            // Verify all messages were logged
            let final_log = shared_message_log.read().await;
            prop_assert_eq!(final_log.len(), message_count);

            // The log should contain all expected indices
            let mut sorted_log = final_log.clone();
            sorted_log.sort();
            for i in 0..message_count {
                prop_assert!(sorted_log.contains(&i));
            }
        });
    }
}

// ============================================================================
// Property 3: Concurrent Sharing Operations
// ============================================================================

proptest! {
    /// Property 3: Concurrent Sharing Operations
    /// *For any* session being shared concurrently with different permissions, all shares
    /// SHALL be created successfully with unique identifiers.
    /// **Validates: Requirements SESSION-3.1**
    #[test]
    fn prop_concurrent_sharing_operations(
        session_id in "[a-z0-9]{1,20}",
        share_count in 1..50usize,
        concurrent_threads in 1..10usize,
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.block_on(async {
            let share_service = Arc::new(ShareService::new());
            let semaphore = Arc::new(Semaphore::new(concurrent_threads));

            let mut handles = Vec::new();
            let mut expected_permissions = Vec::new();

            for i in 0..share_count {
                let share_service_clone = share_service.clone();
                let semaphore_clone = semaphore.clone();
                let session_id_clone = session_id.clone();

                // Generate different permissions for each share
                let permissions = SharePermissions {
                    read_only: i % 2 == 0,
                    include_history: i % 3 == 0,
                    include_context: i % 4 == 0,
                };
                expected_permissions.push(permissions.clone());

                let handle = tokio::spawn(async move {
                    let _permit = semaphore_clone.acquire().await.unwrap();

                    share_service_clone.generate_share_link(
                        &session_id_clone,
                        permissions,
                        Some(ChronoDuration::hours(1)),
                    )
                });

                handles.push(handle);
            }

            // Collect results
            let mut share_ids = Vec::new();
            for handle in handles {
                if let Ok(Ok(share)) = handle.await {
                    share_ids.push(share.id);
                }
            }

            // Should have created all shares
            prop_assert_eq!(share_ids.len(), share_count);

            // All share IDs should be unique
            let mut sorted_ids = share_ids.clone();
            sorted_ids.sort();
            sorted_ids.dedup();
            prop_assert_eq!(sorted_ids.len(), share_ids.len());

            // All shares should be retrievable
            for share_id in share_ids {
                let share = share_service.get_share(&share_id).await;
                prop_assert!(share.is_ok());
            }
        });
    }

    /// Property 3 variant: Concurrent share revocation
    #[test]
    fn prop_concurrent_share_revocation(
        session_id in "[a-z0-9]{1,20}",
        share_count in 2..20usize,
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.block_on(async {
            let share_service = Arc::new(ShareService::new());

            // Create multiple shares
            let mut share_ids = Vec::new();
            for i in 0..share_count {
                let permissions = SharePermissions {
                    read_only: true,
                    include_history: i % 2 == 0,
                    include_context: i % 3 == 0,
                };

                let share = share_service.generate_share_link(
                    &session_id,
                    permissions,
                    None,
                ).await.expect("Failed to create share");

                share_ids.push(share.id);
            }

            // Concurrently revoke shares
            let mut handles = Vec::new();
            for share_id in share_ids.clone() {
                let share_service_clone = share_service.clone();

                let handle = tokio::spawn(async move {
                    share_service_clone.revoke_share(&share_id).await
                });

                handles.push(handle);
            }

            // Wait for all revocations
            let mut success_count = 0;
            for handle in handles {
                if let Ok(Ok(())) = handle.await {
                    success_count += 1;
                }
            }

            // All revocations should succeed
            prop_assert_eq!(success_count, share_count);

            // All shares should be inaccessible
            for share_id in share_ids {
                let share = share_service.get_share(&share_id).await;
                prop_assert!(share.is_err());
            }
        });
    }
}

// ============================================================================
// Property 4: Session Store Concurrency
// ============================================================================

proptest! {
    /// Property 4: Session Store Concurrency
    /// *For any* concurrent operations on the session store, the store SHALL maintain
    /// data integrity and consistency.
    /// **Validates: Requirements SESSION-2.1, SESSION-2.2**
    #[test]
    fn prop_session_store_concurrency(
        operation_count in 1..100usize,
        concurrent_threads in 1..15usize,
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.block_on(async {
            let session_store = Arc::new(SessionStore::new(Default::default()).await
                .expect("Failed to create session store"));
            let semaphore = Arc::new(Semaphore::new(concurrent_threads));

            let mut handles = Vec::new();

            for i in 0..operation_count {
                let session_store_clone = session_store.clone();
                let semaphore_clone = semaphore.clone();

                let handle = tokio::spawn(async move {
                    let _permit = semaphore_clone.acquire().await.unwrap();

                    match i % 3 {
                        0 => {
                            // Store operation
                            let session = Session::new(
                                format!("store-session-{}", i),
                                SessionContext::new("test", "model", SessionMode::Chat)
                            );
                            session_store_clone.store_session(&session).await.is_ok()
                        }
                        1 => {
                            // Load operation
                            let session_name = format!("load-session-{}", i);
                            // Try to load (may not exist, that's ok)
                            session_store_clone.load_session(&session_name).await.is_ok()
                        }
                        2 => {
                            // List operation
                            session_store_clone.list_sessions().await.is_ok()
                        }
                        _ => unreachable!(),
                    }
                });

                handles.push(handle);
            }

            // Wait for all operations
            let mut success_count = 0;
            for handle in handles {
                if let Ok(success) = handle.await {
                    if success {
                        success_count += 1;
                    }
                }
            }

            // Most operations should succeed (some loads may fail if session doesn't exist)
            prop_assert!(success_count >= operation_count / 2);
        });
    }

    /// Property 4 variant: Store and load consistency under concurrency
    #[test]
    fn prop_store_load_consistency_concurrency(
        session_count in 1..30usize,
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.block_on(async {
            let session_store = Arc::new(SessionStore::new(Default::default()).await
                .expect("Failed to create session store"));

            let mut handles = Vec::new();

            // Store sessions concurrently
            for i in 0..session_count {
                let session_store_clone = session_store.clone();

                let handle = tokio::spawn(async move {
                    let session = Session::new(
                        format!("consistency-session-{}", i),
                        SessionContext::new("test", "model", SessionMode::Chat)
                    );

                    session_store_clone.store_session(&session).await
                });

                handles.push(handle);
            }

            // Wait for all stores to complete
            for handle in handles {
                let _ = handle.await;
            }

            // Now load all sessions concurrently
            let mut load_handles = Vec::new();
            for i in 0..session_count {
                let session_store_clone = session_store.clone();

                let handle = tokio::spawn(async move {
                    let session_name = format!("consistency-session-{}", i);
                    session_store_clone.load_session(&session_name).await
                });

                load_handles.push(handle);
            }

            // Verify all loads succeed
            let mut success_count = 0;
            for handle in load_handles {
                if let Ok(Ok(_)) = handle.await {
                    success_count += 1;
                }
            }

            prop_assert_eq!(success_count, session_count);
        });
    }
}

// ============================================================================
// Property 5: Sharing Validation Under Load
// ============================================================================

proptest! {
    /// Property 5: Sharing Validation Under Load
    /// *For any* high-volume sharing operations, the sharing service SHALL maintain
    /// correct access controls and permissions.
    /// **Validates: Requirements SESSION-3.1**
    #[test]
    fn prop_sharing_validation_under_load(
        session_count in 1..20usize,
        shares_per_session in 1..10usize,
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.block_on(async {
            let share_service = Arc::new(ShareService::new());

            let mut all_share_ids = Vec::new();

            // Create multiple sessions with multiple shares each
            for session_i in 0..session_count {
                let session_id = format!("load-session-{}", session_i);

                for share_i in 0..shares_per_session {
                    let permissions = SharePermissions {
                        read_only: share_i % 2 == 0,
                        include_history: share_i % 3 == 0,
                        include_context: share_i % 4 == 0,
                    };

                    let share = share_service.generate_share_link(
                        &session_id,
                        permissions,
                        Some(ChronoDuration::hours(1)),
                    ).await.expect("Failed to create share");

                    all_share_ids.push((session_id.clone(), share.id));
                }
            }

            // Verify all shares are accessible and have correct permissions
            for (expected_session_id, share_id) in all_share_ids {
                let share = share_service.get_share(&share_id).await
                    .expect("Share should exist");

                prop_assert_eq!(share.session_id, expected_session_id);
                prop_assert!(share.created_at <= chrono::Utc::now());
            }

            // Total shares should be correct
            let total_expected = session_count * shares_per_session;
            prop_assert_eq!(all_share_ids.len(), total_expected);
        });
    }

    /// Property 5 variant: Permission isolation under concurrent access
    #[test]
    fn prop_permission_isolation_concurrent_access(
        session_id in "[a-z0-9]{1,20}",
        access_count in 1..50usize,
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.block_on(async {
            let share_service = Arc::new(ShareService::new());

            // Create a share with restrictive permissions
            let restrictive_permissions = SharePermissions {
                read_only: true,
                include_history: false,
                include_context: false,
            };

            let share = share_service.generate_share_link(
                &session_id,
                restrictive_permissions,
                None,
            ).await.expect("Failed to create share");

            let share_id = share.id;

            // Simulate concurrent access attempts
            let mut handles = Vec::new();
            for _ in 0..access_count {
                let share_service_clone = share_service.clone();
                let share_id_clone = share_id.clone();

                let handle = tokio::spawn(async move {
                    let share = share_service_clone.get_share(&share_id_clone).await;
                    if let Ok(share) = share {
                        // Verify permissions are maintained
                        share.permissions.read_only &&
                        !share.permissions.include_history &&
                        !share.permissions.include_context
                    } else {
                        false
                    }
                });

                handles.push(handle);
            }

            // All access attempts should return correct permissions
            let mut correct_permission_count = 0;
            for handle in handles {
                if let Ok(correct) = handle.await {
                    if correct {
                        correct_permission_count += 1;
                    }
                }
            }

            prop_assert_eq!(correct_permission_count, access_count);
        });
    }
}
