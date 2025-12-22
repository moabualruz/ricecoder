use ricecoder_sessions::*;

use crate::models::SessionMode;

fn create_test_context() -> SessionContext {
    SessionContext::new("openai".to_string(), "gpt-4".to_string(), SessionMode::Chat)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_session() {
        let mut router = SessionRouter::new();
        let context = create_test_context();

        let session = router
            .create_session("Test Session".to_string(), context)
            .unwrap();

        assert_eq!(session.name, "Test Session");
        assert_eq!(router.session_count(), 1);
    }

    #[test]
    fn test_route_to_active_session() {
        let mut router = SessionRouter::new();
        let context = create_test_context();

        router
            .create_session("Test Session".to_string(), context)
            .unwrap();

        let session_id = router.route_to_active_session("Hello").unwrap();

        let session = router.get_session(&session_id).unwrap();
        assert_eq!(session.history.len(), 1);
        assert_eq!(session.history[0].content(), "Hello");
    }

    #[test]
    fn test_route_to_specific_session() {
        let mut router = SessionRouter::new();
        let context = create_test_context();

        let session1 = router
            .create_session("Session 1".to_string(), context.clone())
            .unwrap();
        let session2 = router
            .create_session("Session 2".to_string(), context)
            .unwrap();

        // Route message to session 2
        let routed_session_id = router
            .route_to_session(&session2.id, "Message to session 2")
            .unwrap();

        assert_eq!(routed_session_id, session2.id);

        // Verify message is in session 2, not session 1
        let s1 = router.get_session(&session1.id).unwrap();
        let s2 = router.get_session(&session2.id).unwrap();

        assert_eq!(s1.history.len(), 0);
        assert_eq!(s2.history.len(), 1);
    }

    #[test]
    fn test_switch_session() {
        let mut router = SessionRouter::new();
        let context = create_test_context();

        let session1 = router
            .create_session("Session 1".to_string(), context.clone())
            .unwrap();
        let session2 = router
            .create_session("Session 2".to_string(), context)
            .unwrap();

        // Initially session1 is active
        assert_eq!(router.active_session_id(), Some(session1.id.as_str()));

        // Switch to session 2
        router.switch_session(&session2.id).unwrap();

        assert_eq!(router.active_session_id(), Some(session2.id.as_str()));
    }

    #[test]
    fn test_message_isolation() {
        let mut router = SessionRouter::new();
        let context = create_test_context();

        let session1 = router
            .create_session("Session 1".to_string(), context.clone())
            .unwrap();
        let session2 = router
            .create_session("Session 2".to_string(), context)
            .unwrap();

        // Route message to session 1
        router.route_to_session(&session1.id, "Message 1").unwrap();

        // Switch to session 2 and route message
        router.switch_session(&session2.id).unwrap();
        router.route_to_active_session("Message 2").unwrap();

        // Verify messages are isolated
        let s1 = router.get_session(&session1.id).unwrap();
        let s2 = router.get_session(&session2.id).unwrap();

        assert_eq!(s1.history.len(), 1);
        assert_eq!(s2.history.len(), 1);
        assert_eq!(s1.history[0].content(), "Message 1");
        assert_eq!(s2.history[0].content(), "Message 2");
    }

    #[test]
    fn test_delete_session() {
        let mut router = SessionRouter::new();
        let context = create_test_context();

        let session = router
            .create_session("Test Session".to_string(), context)
            .unwrap();

        router.delete_session(&session.id).unwrap();

        assert_eq!(router.session_count(), 0);
        assert!(router.get_session(&session.id).is_err());
    }

    #[test]
    fn test_get_message_session() {
        let mut router = SessionRouter::new();
        let context = create_test_context();

        let session = router
            .create_session("Test Session".to_string(), context)
            .unwrap();

        let session_id = router.route_to_active_session("Hello").unwrap();
        let message_id = router.get_session(&session_id).unwrap().history[0]
            .id
            .clone();

        assert_eq!(router.get_message_session(&message_id), Some(session.id));
    }

    #[test]
    fn test_verify_message_in_session() {
        let mut router = SessionRouter::new();
        let context = create_test_context();

        let session1 = router
            .create_session("Session 1".to_string(), context.clone())
            .unwrap();
        let session2 = router
            .create_session("Session 2".to_string(), context)
            .unwrap();

        router.route_to_session(&session1.id, "Message").unwrap();
        let message_id = router.get_session(&session1.id).unwrap().history[0]
            .id
            .clone();

        assert!(router.verify_message_in_session(&message_id, &session1.id));
        assert!(!router.verify_message_in_session(&message_id, &session2.id));
    }
}
