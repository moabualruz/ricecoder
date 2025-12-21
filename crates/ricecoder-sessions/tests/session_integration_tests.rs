use crate::models::SessionMode;
use ricecoder_sessions::*;

fn create_test_context() -> crate::models::SessionContext {
    crate::models::SessionContext::new("openai".to_string(), "gpt-4".to_string(), SessionMode::Chat)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_session_integration() {
        let mut integration = SessionIntegration::new(5);
        let context = create_test_context();

        let session = integration
            .create_session("Test Session".to_string(), context)
            .unwrap();

        assert_eq!(integration.session_count(), 1);
        assert_eq!(integration.active_session_id(), Some(session.id.as_str()));
    }

    #[test]
    fn test_message_routing() {
        let mut integration = SessionIntegration::new(5);
        let context = create_test_context();

        let session = integration
            .create_session("Test Session".to_string(), context)
            .unwrap();

        let routed_id = integration.add_message_to_active("Hello").unwrap();

        assert_eq!(routed_id, session.id);

        let session = integration.get_session(&session.id).unwrap();
        assert_eq!(session.history.len(), 1);
        assert_eq!(session.history[0].content(), "Hello");
    }

    #[test]
    fn test_session_limit_enforcement() {
        let mut integration = SessionIntegration::new(2);
        let context = create_test_context();

        integration
            .create_session("Session 1".to_string(), context.clone())
            .unwrap();
        integration
            .create_session("Session 2".to_string(), context.clone())
            .unwrap();

        assert!(integration.is_limit_reached());

        let result = integration.create_session("Session 3".to_string(), context);
        assert!(result.is_err());
    }
}
