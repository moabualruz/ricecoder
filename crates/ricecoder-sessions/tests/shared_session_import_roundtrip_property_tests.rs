//! Property-based tests for shared session import round-trip
//! **Feature: ricecoder-sessions, Property 8: Shared Session Import Round-Trip**
//! **Validates: Requirements 3.3**

use proptest::prelude::*;
use ricecoder_sessions::{
    ShareService, SharePermissions, Session, SessionContext, SessionMode, Message, MessageRole,
};

fn arb_session_context() -> impl Strategy<Value = SessionContext> {
    (
        ".*",
        ".*",
        prop_oneof![Just(SessionMode::Chat), Just(SessionMode::Code), Just(SessionMode::Vibe)],
    )
        .prop_map(|(provider, model, mode)| {
            SessionContext::new(provider, model, mode)
        })
}

fn arb_session() -> impl Strategy<Value = Session> {
    (
        ".*",
        arb_session_context(),
    )
        .prop_map(|(name, context)| {
            Session::new(name, context)
        })
}

proptest! {
    /// Property 8: Shared Session Import Round-Trip
    /// *For any* shared session, importing the shared session SHALL create a new local session
    /// with identical context and history as the original.
    /// **Validates: Requirements 3.3**
    #[test]
    fn prop_shared_session_import_roundtrip(
        mut session in arb_session(),
    ) {
        let service = ShareService::new();
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        // Add some messages to the session
        session.history.push(Message::new(MessageRole::User, "Hello".to_string()));
        session.history.push(Message::new(MessageRole::Assistant, "Hi there!".to_string()));

        // Generate a share link
        let share = service.generate_share_link(&session.id, permissions.clone(), None);
        prop_assert!(share.is_ok());
        let share = share.unwrap();

        // Create a shared view
        let shared_view = service.create_shared_session_view(&session, &permissions);

        // Import the shared session
        let imported = service.import_shared_session(&share.id, &shared_view);
        prop_assert!(imported.is_ok());
        let imported = imported.unwrap();

        // Verify the imported session has the same context
        prop_assert_eq!(imported.context.provider, session.context.provider);
        prop_assert_eq!(imported.context.model, session.context.model);
        prop_assert_eq!(imported.context.mode, session.context.mode);

        // Verify the imported session has the same history
        prop_assert_eq!(imported.history.len(), session.history.len());
        for (original_msg, imported_msg) in session.history.iter().zip(imported.history.iter()) {
            prop_assert_eq!(original_msg.role, imported_msg.role);
            prop_assert_eq!(&original_msg.content, &imported_msg.content);
        }

        // Verify the imported session has a different ID
        prop_assert_ne!(imported.id, session.id);
    }

    /// Property 8 variant: Import respects permission settings
    #[test]
    fn prop_shared_session_import_respects_permissions(
        mut session in arb_session(),
    ) {
        let service = ShareService::new();

        // Add messages and context
        session.history.push(Message::new(MessageRole::User, "Test".to_string()));
        session.context.files.push("test.rs".to_string());

        // Test with history excluded
        let permissions_no_history = SharePermissions {
            read_only: true,
            include_history: false,
            include_context: true,
        };

        let share = service.generate_share_link(&session.id, permissions_no_history.clone(), None);
        prop_assert!(share.is_ok());
        let share = share.unwrap();

        let shared_view = service.create_shared_session_view(&session, &permissions_no_history);
        let imported = service.import_shared_session(&share.id, &shared_view);
        prop_assert!(imported.is_ok());
        let imported = imported.unwrap();

        // History should be empty
        prop_assert_eq!(imported.history.len(), 0);

        // Context should be preserved
        prop_assert_eq!(imported.context.provider, session.context.provider);
    }
}
