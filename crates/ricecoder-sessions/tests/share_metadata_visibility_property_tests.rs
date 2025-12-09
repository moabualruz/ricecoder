//! Property-based tests for share metadata visibility
//! **Feature: ricecoder-sharing, Property 9: Metadata Visibility**
//! **Validates: Requirements 4.3**

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
        prop_assert_eq!(
            &shared_view.name, &original_name,
            "Session name should be visible"
        );
        prop_assert_eq!(
            shared_view.created_at, original_created_at,
            "Creation timestamp should be visible"
        );
    }
}

proptest! {
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
                prop_assert_eq!(
                    &shared_view.name, &session.name,
                    "Session name should always be visible"
                );
                prop_assert_eq!(
                    shared_view.created_at, session.created_at,
                    "Creation timestamp should always be visible"
                );
                prop_assert_eq!(
                    &shared_view.context.provider, &session.context.provider,
                    "Provider should always be visible"
                );
                prop_assert_eq!(
                    &shared_view.context.model, &session.context.model,
                    "Model should always be visible"
                );
            }
        }
    }
}

proptest! {
    /// Property 9 variant: Session metadata matches original
    #[test]
    fn prop_metadata_matches_original(
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

        // Verify all metadata matches
        prop_assert_eq!(
            &shared_view.id, &session.id,
            "Session ID should match"
        );
        prop_assert_eq!(
            &shared_view.name, &session.name,
            "Session name should match"
        );
        prop_assert_eq!(
            shared_view.created_at, session.created_at,
            "Creation timestamp should match"
        );
        prop_assert_eq!(
            shared_view.updated_at, session.updated_at,
            "Update timestamp should match"
        );
        prop_assert_eq!(
            shared_view.status, session.status,
            "Session status should match"
        );
    }
}

proptest! {
    /// Property 9 variant: Context metadata is visible
    #[test]
    fn prop_context_metadata_visible(
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

        // Verify context metadata is visible
        prop_assert_eq!(
            &shared_view.context.provider, &session.context.provider,
            "Provider should be visible"
        );
        prop_assert_eq!(
            &shared_view.context.model, &session.context.model,
            "Model should be visible"
        );
        prop_assert_eq!(
            shared_view.context.mode, session.context.mode,
            "Session mode should be visible"
        );
    }
}

proptest! {
    /// Property 9 variant: Metadata is visible without history
    #[test]
    fn prop_metadata_visible_without_history(
        session in arb_session_with_content(),
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: false,
            include_context: true,
        };

        let original_name = session.name.clone();
        let original_created_at = session.created_at;

        // Create shared view
        let shared_view = service.create_shared_session_view(&session, &permissions);

        // Verify metadata is still visible
        prop_assert_eq!(
            &shared_view.name, &original_name,
            "Session name should be visible without history"
        );
        prop_assert_eq!(
            shared_view.created_at, original_created_at,
            "Creation timestamp should be visible without history"
        );
    }
}

proptest! {
    /// Property 9 variant: Metadata is visible without context
    #[test]
    fn prop_metadata_visible_without_context(
        session in arb_session_with_content(),
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: false,
        };

        let original_name = session.name.clone();
        let original_created_at = session.created_at;

        // Create shared view
        let shared_view = service.create_shared_session_view(&session, &permissions);

        // Verify metadata is still visible
        prop_assert_eq!(
            &shared_view.name, &original_name,
            "Session name should be visible without context"
        );
        prop_assert_eq!(
            shared_view.created_at, original_created_at,
            "Creation timestamp should be visible without context"
        );
    }
}

proptest! {
    /// Property 9 variant: Metadata is visible without history and context
    #[test]
    fn prop_metadata_visible_without_history_and_context(
        session in arb_session_with_content(),
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: false,
            include_context: false,
        };

        let original_name = session.name.clone();
        let original_created_at = session.created_at;
        let original_provider = session.context.provider.clone();
        let original_model = session.context.model.clone();

        // Create shared view
        let shared_view = service.create_shared_session_view(&session, &permissions);

        // Verify metadata is still visible
        prop_assert_eq!(
            &shared_view.name, &original_name,
            "Session name should be visible"
        );
        prop_assert_eq!(
            shared_view.created_at, original_created_at,
            "Creation timestamp should be visible"
        );
        prop_assert_eq!(
            &shared_view.context.provider, &original_provider,
            "Provider should be visible"
        );
        prop_assert_eq!(
            &shared_view.context.model, &original_model,
            "Model should be visible"
        );
    }
}

proptest! {
    /// Property 9 variant: Metadata is consistent across multiple views
    #[test]
    fn prop_metadata_consistency(
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

        // Create shared view multiple times
        for _ in 0..3 {
            let shared_view = service.create_shared_session_view(&session, &permissions);

            // Verify metadata is consistent
            prop_assert_eq!(
                &shared_view.name, &original_name,
                "Session name should be consistent"
            );
            prop_assert_eq!(
                shared_view.created_at, original_created_at,
                "Creation timestamp should be consistent"
            );
        }
    }
}
