//! Property-based tests for share history filtering
//! **Feature: ricecoder-sharing, Property 5: History Filtering**
//! **Validates: Requirements 3.4, 4.2**

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

fn arb_session_with_history() -> impl Strategy<Value = Session> {
    ("[a-z0-9]{1,20}", arb_session_context(), 1..10usize).prop_map(
        |(name, context, num_messages)| {
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

            session
        },
    )
}

proptest! {
    /// Property 5: History Filtering
    /// *For any* shared session with include_history=false, the returned session SHALL have an empty message history.
    /// **Validates: Requirements 3.4, 4.2**
    #[test]
    fn prop_history_filtering(
        session in arb_session_with_history(),
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
            prop_assert_eq!(
                shared_view.history.len(),
                original_history_len,
                "History should be included when include_history=true"
            );
        } else {
            prop_assert_eq!(
                shared_view.history.len(),
                0,
                "History should be empty when include_history=false"
            );
        }
    }
}

proptest! {
    /// Property 5 variant: History content is preserved when included
    #[test]
    fn prop_history_content_preserved(
        session in arb_session_with_history(),
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
        prop_assert_eq!(
            shared_view.history.len(),
            session.history.len(),
            "History length should match"
        );

        for (original, shared) in session.history.iter().zip(shared_view.history.iter()) {
            prop_assert_eq!(
                &original.id, &shared.id,
                "Message ID should be preserved"
            );
            prop_assert_eq!(
                &original.content(), &shared.content(),
                "Message content should be preserved"
            );
            prop_assert_eq!(
                original.role, shared.role,
                "Message role should be preserved"
            );
            prop_assert_eq!(
                original.timestamp, shared.timestamp,
                "Message timestamp should be preserved"
            );
        }
    }
}

proptest! {
    /// Property 5 variant: History is completely empty when excluded
    #[test]
    fn prop_history_completely_empty_when_excluded(
        session in arb_session_with_history(),
    ) {
        let service = ShareService::new();

        let permissions = SharePermissions {
            read_only: true,
            include_history: false,
            include_context: true,
        };

        // Create shared view
        let shared_view = service.create_shared_session_view(&session, &permissions);

        // Verify history is completely empty
        prop_assert_eq!(
            shared_view.history.len(),
            0,
            "History should be completely empty"
        );
        prop_assert!(
            shared_view.history.is_empty(),
            "History should be empty"
        );
    }
}

proptest! {
    /// Property 5 variant: History filtering doesn't affect other session data
    #[test]
    fn prop_history_filtering_isolation(
        session in arb_session_with_history(),
    ) {
        let service = ShareService::new();

        let permissions = SharePermissions {
            read_only: true,
            include_history: false,
            include_context: true,
        };

        let original_name = session.name.clone();
        let original_provider = session.context.provider.clone();
        let original_model = session.context.model.clone();

        // Create shared view
        let shared_view = service.create_shared_session_view(&session, &permissions);

        // Verify other data is preserved
        prop_assert_eq!(
            &shared_view.name, &original_name,
            "Session name should be preserved"
        );
        prop_assert_eq!(
            &shared_view.context.provider, &original_provider,
            "Provider should be preserved"
        );
        prop_assert_eq!(
            &shared_view.context.model, &original_model,
            "Model should be preserved"
        );
    }
}

proptest! {
    /// Property 5 variant: History filtering works with empty history
    #[test]
    fn prop_history_filtering_empty_history(
        session_name in "[a-z0-9]{1,20}",
        context in arb_session_context(),
    ) {
        let service = ShareService::new();
        let session = Session::new(session_name, context);

        // Session has no history
        prop_assert_eq!(session.history.len(), 0);

        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Create shared view
        let shared_view = service.create_shared_session_view(&session, &permissions);

        // Verify history is still empty
        prop_assert_eq!(
            shared_view.history.len(),
            0,
            "History should remain empty"
        );
    }
}

proptest! {
    /// Property 5 variant: History filtering is consistent across multiple calls
    #[test]
    fn prop_history_filtering_consistency(
        session in arb_session_with_history(),
    ) {
        let service = ShareService::new();

        let permissions = SharePermissions {
            read_only: true,
            include_history: false,
            include_context: true,
        };

        // Create shared view multiple times
        for _ in 0..3 {
            let shared_view = service.create_shared_session_view(&session, &permissions);

            // Verify history is always empty
            prop_assert_eq!(
                shared_view.history.len(),
                0,
                "History should be consistently empty"
            );
        }
    }
}

proptest! {
    /// Property 5 variant: History filtering with context filtering
    #[test]
    fn prop_history_filtering_with_context_filtering(
        session in arb_session_with_history(),
        include_context in any::<bool>(),
    ) {
        let service = ShareService::new();

        let permissions = SharePermissions {
            read_only: true,
            include_history: false,
            include_context,
        };

        // Create shared view
        let shared_view = service.create_shared_session_view(&session, &permissions);

        // Verify history is always empty regardless of context filtering
        prop_assert_eq!(
            shared_view.history.len(),
            0,
            "History should be empty when include_history=false"
        );

        // Verify context filtering is independent
        if include_context {
            prop_assert!(
                !shared_view.context.files.is_empty() || session.context.files.is_empty(),
                "Context should be included when include_context=true"
            );
        } else {
            prop_assert_eq!(
                shared_view.context.files.len(),
                0,
                "Context should be empty when include_context=false"
            );
        }
    }
}
