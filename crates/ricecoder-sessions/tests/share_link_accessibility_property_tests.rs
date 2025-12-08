//! Property-based tests for share link accessibility
//! **Feature: ricecoder-sharing, Property 2: Link Accessibility**
//! **Validates: Requirements 1.3, 4.1**

use proptest::prelude::*;
use ricecoder_sessions::{
    Message, MessageRole, Session, SessionContext, SessionMode, SharePermissions, ShareService,
};

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

fn arb_session_with_content() -> impl Strategy<Value = Session> {
    (
        "[a-z0-9]{1,20}",
        arb_session_context(),
        1..5usize,
        1..3usize,
    )
        .prop_map(|(name, context, num_messages, num_files)| {
            let mut session = Session::new(name, context);

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

proptest! {
    /// Property 2: Link Accessibility
    /// *For any* generated share link, accessing the session with that link SHALL return the session data.
    /// **Validates: Requirements 1.3, 4.1**
    #[test]
    fn prop_link_accessibility_basic(
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
        prop_assert!(share.is_ok(), "Failed to generate share link");
        let share = share.unwrap();

        // Should be able to retrieve the share
        let retrieved = service.get_share(&share.id);
        prop_assert!(retrieved.is_ok(), "Failed to retrieve share");

        let retrieved = retrieved.unwrap();
        prop_assert_eq!(retrieved.id, share.id, "Share ID mismatch");
        prop_assert_eq!(retrieved.session_id, session_id, "Session ID mismatch");
        prop_assert_eq!(retrieved.permissions.read_only, true, "Read-only flag not set");
    }

    /// Property 2 variant: Accessing non-existent share fails
    #[test]
    fn prop_link_accessibility_nonexistent(
        fake_share_id in "[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}",
    ) {
        let service = ShareService::new();

        // Try to retrieve a non-existent share
        let retrieved = service.get_share(&fake_share_id);
        prop_assert!(retrieved.is_err(), "Should fail for non-existent share");
    }

    /// Property 2 variant: Returned session matches original with filtering applied
    #[test]
    fn prop_link_accessibility_session_matching(
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

                // Generate a share link
                let share = service.generate_share_link(&session.id, permissions.clone(), None);
                prop_assert!(share.is_ok());
                let share = share.unwrap();

                // Retrieve the share
                let retrieved = service.get_share(&share.id);
                prop_assert!(retrieved.is_ok());
                let retrieved = retrieved.unwrap();

                // Verify permissions are preserved
                prop_assert_eq!(
                    retrieved.permissions.include_history, *include_history,
                    "History permission mismatch"
                );
                prop_assert_eq!(
                    retrieved.permissions.include_context, *include_context,
                    "Context permission mismatch"
                );

                // Create shared view and verify filtering
                let shared_view = service.create_shared_session_view(&session, &permissions);

                // Verify history filtering
                if *include_history {
                    prop_assert_eq!(
                        shared_view.history.len(),
                        session.history.len(),
                        "History should be included"
                    );
                } else {
                    prop_assert_eq!(
                        shared_view.history.len(),
                        0,
                        "History should be empty"
                    );
                }

                // Verify context filtering
                if *include_context {
                    prop_assert_eq!(
                        shared_view.context.files.len(),
                        session.context.files.len(),
                        "Context files should be included"
                    );
                } else {
                    prop_assert_eq!(
                        shared_view.context.files.len(),
                        0,
                        "Context files should be empty"
                    );
                }

                // Metadata should always be present
                prop_assert_eq!(
                    &shared_view.name, &session.name,
                    "Session name should be preserved"
                );
                prop_assert_eq!(
                    shared_view.created_at, session.created_at,
                    "Creation timestamp should be preserved"
                );
            }
        }
    }

    /// Property 2 variant: Multiple accesses to same share return consistent data
    #[test]
    fn prop_link_accessibility_consistency(
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
        let share_id = share.id.clone();
        let session_id_ref = session_id.clone();
        let created_at = share.created_at;

        // Access the share multiple times
        for _ in 0..5 {
            let retrieved = service.get_share(&share_id);
            prop_assert!(retrieved.is_ok(), "Failed to retrieve share on repeated access");

            let retrieved = retrieved.unwrap();
            prop_assert_eq!(
                &retrieved.id, &share_id,
                "Share ID changed on repeated access"
            );
            prop_assert_eq!(
                &retrieved.session_id, &session_id_ref,
                "Session ID changed on repeated access"
            );
            prop_assert_eq!(
                retrieved.created_at, created_at,
                "Creation timestamp changed on repeated access"
            );
        }
    }

    /// Property 2 variant: Share metadata is complete and accurate
    #[test]
    fn prop_link_accessibility_metadata_complete(
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

        // Retrieve the share
        let retrieved = service.get_share(&share.id);
        prop_assert!(retrieved.is_ok());
        let retrieved = retrieved.unwrap();

        // Verify all metadata fields are present and correct
        prop_assert!(!retrieved.id.is_empty(), "Share ID should not be empty");
        prop_assert_eq!(&retrieved.session_id, &session_id, "Session ID mismatch");
        prop_assert!(
            retrieved.created_at <= chrono::Utc::now(),
            "Creation timestamp should be in the past"
        );
        prop_assert_eq!(
            retrieved.permissions.read_only, true,
            "Read-only flag should be true"
        );
        prop_assert_eq!(
            retrieved.permissions.include_history, true,
            "Include history should match"
        );
        prop_assert_eq!(
            retrieved.permissions.include_context, true,
            "Include context should match"
        );
    }
}
