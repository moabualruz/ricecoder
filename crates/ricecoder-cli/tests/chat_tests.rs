use ricecoder_cli::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_session_creation() {
        let session = ChatSession::new("openai".to_string(), "gpt-4".to_string());
        assert_eq!(session.provider, "openai");
        assert_eq!(session.model, "gpt-4");
        assert!(session.history.is_empty());
    }

    #[test]
    fn test_add_message() {
        let mut session = ChatSession::new("openai".to_string(), "gpt-4".to_string());
        session.add_message("user".to_string(), "Hello".to_string());
        assert_eq!(session.history.len(), 1);
        assert_eq!(session.history[0].role, "user");
        assert_eq!(session.history[0].content, "Hello");
    }

    #[test]
    fn test_get_history() {
        let mut session = ChatSession::new("openai".to_string(), "gpt-4".to_string());
        session.add_message("user".to_string(), "Hello".to_string());
        session.add_message("assistant".to_string(), "Hi there!".to_string());

        let history = session.get_history();
        assert_eq!(history.len(), 2);
    }
}