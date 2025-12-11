//! Property-based tests for message routing correctness
//! **Feature: ricecoder-sessions, Property 5: Message Routing Correctness**
//! **Validates: Requirements 1.2**

use proptest::prelude::*;
use ricecoder_sessions::{SessionContext, SessionMode, SessionRouter};

// Strategy for generating session names
fn session_name_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_-]{1,50}".prop_map(|s| format!("Session-{}", s))
}

// Strategy for generating message content
fn message_content_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 .,!?]{1,200}".prop_map(|s| s)
}

// Strategy for generating session contexts
fn session_context_strategy() -> impl Strategy<Value = SessionContext> {
    (
        "[a-z]{3,10}".prop_map(|s| s),
        "[a-z0-9-]{3,20}".prop_map(|s| s),
    )
        .prop_map(|(provider, model)| SessionContext::new(provider, model, SessionMode::Chat))
}

// Helper function for creating test context
fn create_test_context() -> SessionContext {
    SessionContext::new("openai".to_string(), "gpt-4".to_string(), SessionMode::Chat)
}

proptest! {
    /// Property 5: Message Routing Correctness
    /// For any active session, messages sent to that session SHALL be routed to the correct session
    /// and SHALL NOT be routed to other sessions.
    #[test]
    fn prop_message_routed_to_active_session(
        session_name in session_name_strategy(),
        message in message_content_strategy(),
        context in session_context_strategy(),
    ) {
        let mut router = SessionRouter::new();

        // Create a session
        let session = router
            .create_session(session_name.clone(), context)
            .expect("Failed to create session");
        let session_id = session.id.clone();

        // Route a message to the active session
        let routed_session_id = router
            .route_to_active_session(&message)
            .expect("Failed to route message");

        // Verify the message was routed to the correct session
        prop_assert_eq!(&routed_session_id, &session_id);

        // Verify the message is in the session history
        let updated_session = router
            .get_session(&session_id)
            .expect("Failed to get session");
        prop_assert_eq!(updated_session.history.len(), 1);
        prop_assert_eq!(&updated_session.history[0].content(), &message);
    }

    /// Property 5: Message Routing Correctness - Multiple Sessions
    /// For any set of sessions, messages routed to a specific session SHALL NOT appear in other sessions.
    #[test]
    fn prop_message_isolation_between_sessions(
        session1_name in session_name_strategy(),
        session2_name in session_name_strategy(),
        message1 in message_content_strategy(),
        message2 in message_content_strategy()
    ) {
        prop_assume!(session1_name != session2_name);

        let mut router = SessionRouter::new();
        let context = create_test_context();

        let s1 = router.create_session(session1_name, context.clone()).unwrap();
        let s2 = router.create_session(session2_name, context).unwrap();

        router.route_to_session(&s1.id, &message1).unwrap();
        router.route_to_session(&s2.id, &message2).unwrap();

        let sessions = router.list_sessions();

        // Session 1 should only have message1
        let s1_updated = router.get_session(&s1.id).unwrap();
        prop_assert_eq!(s1_updated.history.len(), 1);
        prop_assert_eq!(&s1_updated.history[0].content(), &message1);

        // Session 2 should only have message2
        let s2_updated = router.get_session(&s2.id).unwrap();
        prop_assert_eq!(s2_updated.history.len(), 1);
        prop_assert_eq!(&s2_updated.history[0].content(), &message2);
    }

    /// Property 5: Message Routing Correctness - Session Switching
    /// When switching sessions, subsequent messages SHALL be routed to the new active session.
    #[test]
    fn prop_message_routing_after_session_switch(
        session1_name in session_name_strategy(),
        session2_name in session_name_strategy(),
        message1 in message_content_strategy(),
        message2 in message_content_strategy()
    ) {
        prop_assume!(session1_name != session2_name);

        let mut router = SessionRouter::new();
        let context = create_test_context();

        let s1 = router.create_session(session1_name, context.clone()).unwrap();
        let s2 = router.create_session(session2_name, context).unwrap();

        // Route message1 to session1
        router.route_to_session(&s1.id, &message1).unwrap();

        // Switch to session2 and route message2
        router.switch_session(&s2.id).unwrap();
        router.route_to_active_session(&message2).unwrap();

        let sessions = router.list_sessions();

        // Session 1 should only have message1
        let s1_updated = router.get_session(&s1.id).unwrap();
        prop_assert_eq!(s1_updated.history.len(), 1);
        prop_assert_eq!(&s1_updated.history[0].content(), &message1);

        // Session 2 should only have message2
        let s2_updated = router.get_session(&s2.id).unwrap();
        prop_assert_eq!(s2_updated.history.len(), 1);
        prop_assert_eq!(&s2_updated.history[0].content(), &message2);
    }

    /// Property 5: Message Routing Correctness - Message Tracking
    /// For any message routed to a session, the router SHALL correctly track which session the message belongs to.
    #[test]
    fn prop_message_session_tracking(
        session_name in session_name_strategy(),
        message in message_content_strategy(),
        context in session_context_strategy(),
    ) {
        let mut router = SessionRouter::new();

        // Create a session
        let session = router
            .create_session(session_name, context)
            .expect("Failed to create session");
        let session_id = session.id.clone();

        // Route a message
        router
            .route_to_active_session(&message)
            .expect("Failed to route message");

        // Get the message ID
        let updated_session = router
            .get_session(&session_id)
            .expect("Failed to get session");
        let message_id = updated_session.history[0].id.clone();

        // Verify the router tracks the message correctly
        let tracked_session_id = router
            .get_message_session(&message_id)
            .expect("Failed to get message session");
        prop_assert_eq!(&tracked_session_id, &session_id);

        // Verify message verification works
        prop_assert!(router.verify_message_in_session(&message_id, &session_id));
    }

    /// Property 5: Message Routing Correctness - Multiple Messages
    /// For any sequence of messages routed to a session, all messages SHALL be correctly routed
    /// and SHALL maintain their order in the session history.
    #[test]
    fn prop_multiple_messages_routing(
        session_name in session_name_strategy(),
        messages in prop::collection::vec(message_content_strategy(), 1..10),
        context in session_context_strategy(),
    ) {
        let mut router = SessionRouter::new();

        // Create a session
        let session = router
            .create_session(session_name, context)
            .expect("Failed to create session");
        let session_id = session.id.clone();

        // Route multiple messages
        for message in &messages {
            router
                .route_to_active_session(message)
                .expect("Failed to route message");
        }

        // Verify all messages are in the session
        let updated_session = router
            .get_session(&session_id)
            .expect("Failed to get session");

        prop_assert_eq!(updated_session.history.len(), messages.len());

        // Verify messages are in the correct order
        for (i, expected_message) in messages.iter().enumerate() {
            prop_assert_eq!(&updated_session.history[i].content(), expected_message);
        }
    }
}
