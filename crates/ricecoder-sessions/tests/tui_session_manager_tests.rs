use ricecoder_sessions::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_session() {
        let mut manager = TuiSessionManager::new();

        let id = manager.create_session("Test Session".to_string()).unwrap();

        assert!(id.starts_with("session-"));
        assert_eq!(manager.session_count(), 1);
    }

    #[test]
    fn test_create_multiple_sessions() {
        let mut manager = TuiSessionManager::new();

        let id1 = manager.create_session("Session 1".to_string()).unwrap();
        let id2 = manager.create_session("Session 2".to_string()).unwrap();
        let id3 = manager.create_session("Session 3".to_string()).unwrap();

        assert_eq!(manager.session_count(), 3);
        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
    }

    #[test]
    fn test_delete_session() {
        let mut manager = TuiSessionManager::new();

        let id = manager.create_session("Test Session".to_string()).unwrap();
        assert_eq!(manager.session_count(), 1);

        manager.delete_session(&id).unwrap();
        assert_eq!(manager.session_count(), 0);
    }

    #[test]
    fn test_delete_nonexistent_session() {
        let mut manager = TuiSessionManager::new();

        let result = manager.delete_session("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_rename_session() {
        let mut manager = TuiSessionManager::new();

        let id = manager.create_session("Old Name".to_string()).unwrap();
        manager.rename_session(&id, "New Name".to_string()).unwrap();

        let data = manager.get_session_data(&id).unwrap();
        assert_eq!(data.name, "New Name");
    }

    #[test]
    fn test_rename_nonexistent_session() {
        let mut manager = TuiSessionManager::new();

        let result = manager.rename_session("nonexistent", "New Name".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_add_content_to_session() {
        let mut manager = TuiSessionManager::new();

        let id = manager.create_session("Test Session".to_string()).unwrap();
        manager.add_content_to_session(&id, "Hello").unwrap();
        manager.add_content_to_session(&id, "World").unwrap();

        let content = manager.get_session_content(&id).unwrap();
        assert!(content.contains("Hello"));
        assert!(content.contains("World"));
    }

    #[test]
    fn test_get_all_session_ids() {
        let mut manager = TuiSessionManager::new();

        let id1 = manager.create_session("Session 1".to_string()).unwrap();
        let id2 = manager.create_session("Session 2".to_string()).unwrap();

        let ids = manager.all_session_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&id1));
        assert!(ids.contains(&id2));
    }

    #[test]
    fn test_get_all_session_names() {
        let mut manager = TuiSessionManager::new();

        manager.create_session("Session 1".to_string()).unwrap();
        manager.create_session("Session 2".to_string()).unwrap();

        let names = manager.all_session_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"Session 1".to_string()));
        assert!(names.contains(&"Session 2".to_string()));
    }

    #[test]
    fn test_clear_all_sessions() {
        let mut manager = TuiSessionManager::new();

        manager.create_session("Session 1".to_string()).unwrap();
        manager.create_session("Session 2".to_string()).unwrap();

        assert_eq!(manager.session_count(), 2);

        manager.clear_all_sessions();
        assert_eq!(manager.session_count(), 0);
    }

    #[test]
    fn test_session_data_persistence() {
        let mut manager = TuiSessionManager::new();

        let id = manager.create_session("Test Session".to_string()).unwrap();
        manager.add_content_to_session(&id, "Message 1").unwrap();
        manager.add_content_to_session(&id, "Message 2").unwrap();

        let data = manager.get_session_data(&id).unwrap();
        assert_eq!(data.name, "Test Session");
        assert!(data.content.contains("Message 1"));
        assert!(data.content.contains("Message 2"));
    }
}