//! Comprehensive property-based tests for conversation sharing
//! **Feature: ricecoder-sharing, Properties 1-11**
//! **Validates: Requirements 1.1, 1.3, 1.4, 1.5, 2.1, 2.2, 2.3, 2.4, 3.2, 3.4, 3.5, 4.1, 4.2, 4.3, 5.1, 5.3, 5.4, 5.5**

use chrono::Duration;
use proptest::prelude::*;
use ricecoder_sessions::{
    Message, MessageRole, Session, SessionContext, SessionMode, SharePermissions, ShareService,
};
use std::thread;
use std::time::Duration as StdDuration;

// ============================================================================
// Generators for property-based testing
// ============================================================================

fn arb_session_context() -> impl Strategy<Value = SessionContext> {
    (
        "[a-z0-9]{1,20}",
        "[a-z0-9]{1,20}",
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

fn arb_session_with_content() -> impl Strategy<Value = Session> {
    (arb_session(), 1..5usize, 1..3usize).prop_map(|(mut session, num_messages, num_files)| {
        // Add messages
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

        // Add context files
        for i in 0..num_files {
            session.context.files.push(format!("file{}.rs", i));
        }

        // Add custom context
        session
            .context
            .custom
            .insert("key".to_string(), serde_json::json!("value"));

        session
    })
}

// ============================================================================
// Property 1: Link Uniqueness
// ============================================================================

proptest! {
    /// Property 1: Link Uniqueness
    /// *For any* two shares generated, the share IDs SHALL be unique and follow UUID format.
    /// **Validates: Requirements 1.1**
    #[test]
    fn prop_link_uniqueness_basic(
        session_id_1 in "[a-z0-9]{1,20}",
        session_id_2 in "[a-z0-9]{1,20}",
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Generate two share links
        let share_1 = service.generate_share_link(&session_id_1, permissions.clone(), None);
        let share_2 = service.generate_share_link(&session_id_2, permissions.clone(), None);

        // Both should succeed
        prop_assert!(share_1.is_ok());
        prop_assert!(share_2.is_ok());

        let share_1 = share_1.unwrap();
        let share_2 = share_2.unwrap();

        // Share IDs should be unique
        prop_assert_ne!(&share_1.id, &share_2.id);

        // Share IDs should be valid UUIDs (36 chars with hyphens)
        prop_assert_eq!(share_1.id.len(), 36);
        prop_assert_eq!(share_2.id.len(), 36);
    }

    /// Property 1 variant: Multiple shares for same session have unique IDs
    #[test]
    fn prop_link_uniqueness_multiple_shares(
        session_id in "[a-z0-9]{1,20}",
        num_shares in 2..20usize,
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        let mut share_ids = Vec::new();

        // Generate multiple shares for the same session
        for _ in 0..num_shares {
            let share = service.generate_share_link(&session_id, permissions.clone(), None);
            prop_assert!(share.is_ok());
            share_ids.push(share.unwrap().id);
        }

        // All share IDs should be unique
        let mut sorted_ids = share_ids.clone();
        sorted_ids.sort();
        sorted_ids.dedup();
        prop_assert_eq!(sorted_ids.len(), share_ids.len());
    }
}

// ============================================================================
// Property 2: Link Accessibility
// ============================================================================

proptest! {
    /// Property 2: Link Accessibility
    /// *For any* generated share link, accessing the session with that link SHALL return the session data.
    /// **Validates: Requirements 1.3, 4.1**
    #[test]
    fn prop_link_accessibility(
        session_id in "[a-z0-9]{1,20}",
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Generate a share link
        let share = service.generate_share_link(&session_id, permissions.clone(), None);
        prop_assert!(share.is_ok());
        let share = share.unwrap();

        // Should be able to retrieve the share
        let retrieved = service.get_share(&share.id);
        prop_assert!(retrieved.is_ok());

        let retrieved = retrieved.unwrap();
        prop_assert_eq!(retrieved.id, share.id);
        prop_assert_eq!(retrieved.session_id, session_id);
        prop_assert_eq!(retrieved.permissions.read_only, true);
    }

    /// Property 2 variant: Accessing non-existent share fails
    #[test]
    fn prop_link_accessibility_nonexistent(
        fake_share_id in "[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}",
    ) {
        let service = ShareService::new();

        // Try to retrieve a non-existent share
        let retrieved = service.get_share(&fake_share_id);
        prop_assert!(retrieved.is_err());
    }
}

// ============================================================================
// Property 3: Share List Completeness
// ============================================================================

proptest! {
    /// Property 3: Share List Completeness
    /// *For any* set of active shares, the list command SHALL return all active shares with creation date and expiration.
    /// **Validates: Requirements 1.4, 5.1**
    #[test]
    fn prop_share_list_completeness(
        session_id in "[a-z0-9]{1,20}",
        num_shares in 1..10usize,
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Create multiple shares
        let mut created_ids = Vec::new();
        for _ in 0..num_shares {
            let share = service.generate_share_link(&session_id, permissions.clone(), None);
            prop_assert!(share.is_ok());
            created_ids.push(share.unwrap().id);
        }

        // List all shares
        let shares = service.list_shares();
        prop_assert!(shares.is_ok());
        let shares = shares.unwrap();

        // Should contain all created shares
        prop_assert_eq!(shares.len(), num_shares);

        // All created IDs should be in the list
        for created_id in &created_ids {
            let found = shares.iter().any(|s| &s.id == created_id);
            prop_assert!(found, "Share {} not found in list", created_id);
        }

        // All shares should have creation date
        for share in &shares {
            prop_assert!(share.created_at <= chrono::Utc::now());
        }
    }

    /// Property 3 variant: List includes expiration info
    #[test]
    fn prop_share_list_includes_expiration(
        session_id in "[a-z0-9]{1,20}",
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Create share with expiration
        let share = service.generate_share_link(
            &session_id,
            permissions.clone(),
            Some(Duration::hours(1)),
        );
        prop_assert!(share.is_ok());

        // List shares
        let shares = service.list_shares();
        prop_assert!(shares.is_ok());
        let shares = shares.unwrap();

        // Should have at least one share with expiration
        let has_expiration = shares.iter().any(|s| s.expires_at.is_some());
        prop_assert!(has_expiration);
    }
}

// ============================================================================
// Property 4: Revocation Completeness
// ============================================================================

proptest! {
    /// Property 4: Revocation Completeness
    /// *For any* revoked share, subsequent access attempts SHALL return an error (ShareRevoked).
    /// **Validates: Requirements 1.5, 5.3**
    #[test]
    fn prop_revocation_completeness(
        session_id in "[a-z0-9]{1,20}",
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Generate a share link
        let share = service.generate_share_link(&session_id, permissions.clone(), None);
        prop_assert!(share.is_ok());
        let share = share.unwrap();

        // Should be accessible before revocation
        let retrieved = service.get_share(&share.id);
        prop_assert!(retrieved.is_ok());

        // Revoke the share
        let revoked = service.revoke_share(&share.id);
        prop_assert!(revoked.is_ok());

        // Should not be accessible after revocation
        let retrieved = service.get_share(&share.id);
        prop_assert!(retrieved.is_err());
    }

    /// Property 4 variant: Revoked share no longer appears in list
    #[test]
    fn prop_revocation_removes_from_list(
        session_id in "[a-z0-9]{1,20}",
        num_shares in 2..5usize,
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Create multiple shares
        let mut share_ids = Vec::new();
        for _ in 0..num_shares {
            let share = service.generate_share_link(&session_id, permissions.clone(), None);
            prop_assert!(share.is_ok());
            share_ids.push(share.unwrap().id);
        }

        // Revoke the first share
        let revoked = service.revoke_share(&share_ids[0]);
        prop_assert!(revoked.is_ok());

        // List shares
        let shares = service.list_shares();
        prop_assert!(shares.is_ok());
        let shares = shares.unwrap();

        // Revoked share should not be in list
        let found = shares.iter().any(|s| s.id == share_ids[0]);
        prop_assert!(!found);

        // Other shares should still be in list
        for i in 1..share_ids.len() {
            let found = shares.iter().any(|s| s.id == share_ids[i]);
            prop_assert!(found);
        }
    }
}

// ============================================================================
// Property 5: History Filtering
// ============================================================================

proptest! {
    /// Property 5: History Filtering
    /// *For any* shared session with include_history=false, the returned session SHALL have an empty message history.
    /// **Validates: Requirements 3.4, 4.2**
    #[test]
    fn prop_history_filtering(
        mut session in arb_session_with_content(),
        include_history in any::<bool>(),
    ) {
        let service = ShareService::new();

        let permissions = SharePermissions {
            read_only: true,
            include_history,
            include_context: true,
        };

        let original_history_len = session.history.len();

        // Create shared view
        let shared_view = service.create_shared_session_view(&session, &permissions);

        // Verify history filtering
        if include_history {
            prop_assert_eq!(shared_view.history.len(), original_history_len);
        } else {
            prop_assert_eq!(shared_view.history.len(), 0);
        }
    }

    /// Property 5 variant: History content is preserved when included
    #[test]
    fn prop_history_content_preserved(
        session in arb_session_with_content(),
    ) {
        let service = ShareService::new();

        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Create shared view
        let shared_view = service.create_shared_session_view(&session, &permissions);

        // Verify history content is preserved
        prop_assert_eq!(shared_view.history.len(), session.history.len());
        for (original, shared) in session.history.iter().zip(shared_view.history.iter()) {
            prop_assert_eq!(&original.id, &shared.id);
            prop_assert_eq!(&original.content(), &shared.content());
            prop_assert_eq!(original.role, shared.role);
        }
    }
}

// ============================================================================
// Property 6: Context Filtering
// ============================================================================

proptest! {
    /// Property 6: Context Filtering
    /// *For any* shared session with include_context=false, the returned session SHALL have empty context files and custom data.
    /// **Validates: Requirements 3.5, 4.3**
    #[test]
    fn prop_context_filtering(
        session in arb_session_with_content(),
        include_context in any::<bool>(),
    ) {
        let service = ShareService::new();

        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context,
        };

        let original_files_len = session.context.files.len();
        let original_custom_len = session.context.custom.len();

        // Create shared view
        let shared_view = service.create_shared_session_view(&session, &permissions);

        // Verify context filtering
        if include_context {
            prop_assert_eq!(shared_view.context.files.len(), original_files_len);
            prop_assert_eq!(shared_view.context.custom.len(), original_custom_len);
        } else {
            prop_assert_eq!(shared_view.context.files.len(), 0);
            prop_assert_eq!(shared_view.context.custom.len(), 0);
        }

        // Provider and model should always be included
        prop_assert_eq!(&shared_view.context.provider, &session.context.provider);
        prop_assert_eq!(&shared_view.context.model, &session.context.model);
    }

    /// Property 6 variant: Context content is preserved when included
    #[test]
    fn prop_context_content_preserved(
        session in arb_session_with_content(),
    ) {
        let service = ShareService::new();

        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Create shared view
        let shared_view = service.create_shared_session_view(&session, &permissions);

        // Verify context content is preserved
        prop_assert_eq!(shared_view.context.files, session.context.files);
        prop_assert_eq!(shared_view.context.custom, session.context.custom);
    }
}

// ============================================================================
// Property 7: Expiration Enforcement
// ============================================================================

proptest! {
    /// Property 7: Expiration Enforcement
    /// *For any* share with expiration time set, accessing the share after expiration
    /// SHALL return an error (ShareExpired).
    /// **Validates: Requirements 3.2**
    #[test]
    fn prop_expiration_enforcement(
        session_id in "[a-z0-9]{1,20}",
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Generate a share link that expires in 100ms
        let share = service.generate_share_link(
            &session_id,
            permissions.clone(),
            Some(Duration::milliseconds(100)),
        );
        prop_assert!(share.is_ok());
        let share = share.unwrap();

        // Should be accessible immediately
        let retrieved = service.get_share(&share.id);
        prop_assert!(retrieved.is_ok());

        // Wait for expiration
        thread::sleep(StdDuration::from_millis(150));

        // Should be expired now
        let retrieved = service.get_share(&share.id);
        prop_assert!(retrieved.is_err());
    }

    /// Property 7 variant: Non-expiring shares remain accessible
    #[test]
    fn prop_no_expiration_always_accessible(
        session_id in "[a-z0-9]{1,20}",
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Generate a share link with no expiration
        let share = service.generate_share_link(&session_id, permissions.clone(), None);
        prop_assert!(share.is_ok());
        let share = share.unwrap();

        // Should be accessible
        let retrieved = service.get_share(&share.id);
        prop_assert!(retrieved.is_ok());

        // Wait a bit
        thread::sleep(StdDuration::from_millis(100));

        // Should still be accessible
        let retrieved = service.get_share(&share.id);
        prop_assert!(retrieved.is_ok());
    }
}

// ============================================================================
// Property 8: Read-Only Enforcement
// ============================================================================

proptest! {
    /// Property 8: Read-Only Enforcement
    /// *For any* shared session, the read_only permission flag SHALL be set to true,
    /// preventing modifications.
    /// **Validates: Requirements 2.1, 2.2, 2.3, 2.4**
    #[test]
    fn prop_readonly_enforcement(
        session_id in "[a-z0-9]{1,20}",
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Generate a share link
        let share = service.generate_share_link(&session_id, permissions.clone(), None);
        prop_assert!(share.is_ok());
        let share = share.unwrap();

        // Verify read_only is true
        prop_assert_eq!(share.permissions.read_only, true);

        // Retrieve the share and verify read_only is still true
        let retrieved = service.get_share(&share.id);
        prop_assert!(retrieved.is_ok());
        let retrieved = retrieved.unwrap();
        prop_assert_eq!(retrieved.permissions.read_only, true);
    }

    /// Property 8 variant: All permission combinations preserve read_only
    #[test]
    fn prop_readonly_with_all_permission_combinations(
        session_id in "[a-z0-9]{1,20}",
    ) {
        let service = ShareService::new();

        // Test all combinations of include_history and include_context
        for include_history in &[true, false] {
            for include_context in &[true, false] {
                let permissions = SharePermissions {
                    read_only: true,
                    include_history: *include_history,
                    include_context: *include_context,
                };

                let share = service.generate_share_link(&session_id, permissions.clone(), None);
                prop_assert!(share.is_ok());
                let share = share.unwrap();

                // read_only should always be true
                prop_assert_eq!(share.permissions.read_only, true);
            }
        }
    }
}

// ============================================================================
// Property 9: Metadata Visibility
// ============================================================================

proptest! {
    /// Property 9: Metadata Visibility
    /// *For any* shared session, session metadata (name, created_at) SHALL be visible to viewers.
    /// **Validates: Requirements 4.3**
    #[test]
    fn prop_metadata_visibility(
        session in arb_session_with_content(),
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        let original_name = session.name.clone();
        let original_created_at = session.created_at;

        // Create shared view
        let shared_view = service.create_shared_session_view(&session, &permissions);

        // Verify metadata is visible
        prop_assert_eq!(shared_view.name, original_name);
        prop_assert_eq!(shared_view.created_at, original_created_at);
    }

    /// Property 9 variant: Metadata is visible even with filtering
    #[test]
    fn prop_metadata_visible_with_filtering(
        session in arb_session_with_content(),
    ) {
        let service = ShareService::new();

        // Test with all permission combinations
        for include_history in &[true, false] {
            for include_context in &[true, false] {
                let permissions = SharePermissions {
                    read_only: true,
                    include_history: *include_history,
                    include_context: *include_context,
                };

                let shared_view = service.create_shared_session_view(&session, &permissions);

                // Metadata should always be visible
                prop_assert_eq!(&shared_view.name, &session.name);
                prop_assert_eq!(shared_view.created_at, session.created_at);
                prop_assert_eq!(&shared_view.context.provider, &session.context.provider);
                prop_assert_eq!(&shared_view.context.model, &session.context.model);
            }
        }
    }
}

// ============================================================================
// Property 10: Session Deletion Invalidation
// ============================================================================

proptest! {
    /// Property 10: Session Deletion Invalidation
    /// *For any* session that is deleted, all associated shares SHALL become invalid
    /// (return error on access).
    /// **Validates: Requirements 5.5**
    #[test]
    fn prop_session_deletion_invalidation(
        session_id in "[a-z0-9]{1,20}",
        num_shares in 1..5usize,
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Create multiple shares for a session
        let mut share_ids = Vec::new();
        for _ in 0..num_shares {
            let share = service.generate_share_link(&session_id, permissions.clone(), None);
            prop_assert!(share.is_ok());
            share_ids.push(share.unwrap().id);
        }

        // All shares should be accessible
        for share_id in &share_ids {
            let retrieved = service.get_share(share_id);
            prop_assert!(retrieved.is_ok());
        }

        // Invalidate all shares for the session (simulating session deletion)
        let invalidated = service.invalidate_session_shares(&session_id);
        prop_assert!(invalidated.is_ok());
        let invalidated = invalidated.unwrap();
        prop_assert_eq!(invalidated, num_shares);

        // All shares should now be inaccessible
        for share_id in &share_ids {
            let retrieved = service.get_share(share_id);
            prop_assert!(retrieved.is_err());
        }
    }

    /// Property 10 variant: Invalidation only affects specified session
    #[test]
    fn prop_session_deletion_isolation(
        session_id_1 in "[a-z0-9]{1,20}",
        session_id_2 in "[a-z0-9]{1,20}",
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Create shares for two different sessions
        let share_1 = service.generate_share_link(&session_id_1, permissions.clone(), None);
        let share_2 = service.generate_share_link(&session_id_2, permissions.clone(), None);

        prop_assert!(share_1.is_ok());
        prop_assert!(share_2.is_ok());

        let share_1_id = share_1.unwrap().id;
        let share_2_id = share_2.unwrap().id;

        // Invalidate shares for session 1
        let invalidated = service.invalidate_session_shares(&session_id_1);
        prop_assert!(invalidated.is_ok());

        // Share 1 should be inaccessible
        let retrieved = service.get_share(&share_1_id);
        prop_assert!(retrieved.is_err());

        // Share 2 should still be accessible
        let retrieved = service.get_share(&share_2_id);
        prop_assert!(retrieved.is_ok());
    }
}

// ============================================================================
// Property 11: Automatic Expiration Cleanup
// ============================================================================

proptest! {
    /// Property 11: Automatic Expiration Cleanup
    /// *For any* share that has expired, it SHALL be automatically cleaned up
    /// and no longer accessible.
    /// **Validates: Requirements 5.4**
    #[test]
    fn prop_automatic_expiration_cleanup(
        session_id in "[a-z0-9]{1,20}",
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Generate an expiring share
        let share = service.generate_share_link(
            &session_id,
            permissions.clone(),
            Some(Duration::milliseconds(100)),
        );
        prop_assert!(share.is_ok());
        let share_id = share.unwrap().id;

        // Wait for expiration
        thread::sleep(StdDuration::from_millis(150));

        // Run cleanup
        let cleaned = service.cleanup_expired_shares();
        prop_assert!(cleaned.is_ok());
        let cleaned = cleaned.unwrap();

        // Should have cleaned up at least one share
        prop_assert!(cleaned >= 1);

        // Expired share should no longer be accessible
        let retrieved = service.get_share(&share_id);
        prop_assert!(retrieved.is_err());
    }

    /// Property 11 variant: Cleanup preserves non-expired shares
    #[test]
    fn prop_cleanup_preserves_active_shares(
        session_id in "[a-z0-9]{1,20}",
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Create an expiring share
        let expiring = service.generate_share_link(
            &session_id,
            permissions.clone(),
            Some(Duration::milliseconds(100)),
        );
        prop_assert!(expiring.is_ok());

        // Create a non-expiring share
        let permanent = service.generate_share_link(&session_id, permissions.clone(), None);
        prop_assert!(permanent.is_ok());
        let permanent_id = permanent.unwrap().id;

        // Wait for expiration
        thread::sleep(StdDuration::from_millis(150));

        // Run cleanup
        let cleaned = service.cleanup_expired_shares();
        prop_assert!(cleaned.is_ok());

        // Non-expiring share should still be accessible
        let retrieved = service.get_share(&permanent_id);
        prop_assert!(retrieved.is_ok());
    }
}
