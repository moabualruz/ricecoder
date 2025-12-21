use ricecoder_sessions::*;

fn create_test_context() -> SessionContext {
    SessionContext::new("openai".to_string(), "gpt-4".to_string(), SessionMode::Chat)
}

#[test]
fn test_create_session() {
    let mut manager = SessionManager::new(5);
    let context = create_test_context();

    let session = manager
        .create_session("Test Session".to_string(), context)
        .unwrap();

    assert_eq!(session.name, "Test Session");
    assert_eq!(manager.session_count(), 1);
}

#[test]
fn test_session_limit_enforcement() {
    let mut manager = SessionManager::new(2);
    let context = create_test_context();

    // Create first session
    manager
        .create_session("Session 1".to_string(), context.clone())
        .unwrap();

    // Create second session
    manager
        .create_session("Session 2".to_string(), context.clone())
        .unwrap();

    // Third session should fail
    let result = manager.create_session("Session 3".to_string(), context);
    assert!(matches!(result, Err(SessionError::LimitReached { max: 2 })));
}

#[test]
fn test_switch_session() {
    let mut manager = SessionManager::new(5);
    let context = create_test_context();

    let _session1 = manager
        .create_session("Session 1".to_string(), context.clone())
        .unwrap();
    let session2 = manager
        .create_session("Session 2".to_string(), context)
        .unwrap();

    // Switch to session 2
    manager.switch_session(&session2.id).unwrap();

    let active = manager.get_active_session().unwrap();
    assert_eq!(active.id, session2.id);
}

#[test]
fn test_delete_session() {
    let mut manager = SessionManager::new(5);
    let context = create_test_context();

    let session = manager
        .create_session("Test Session".to_string(), context)
        .unwrap();

    manager.delete_session(&session.id).unwrap();

    assert_eq!(manager.session_count(), 0);
    assert!(manager.get_session(&session.id).is_err());
}

#[test]
fn test_list_sessions() {
    let mut manager = SessionManager::new(5);
    let context = create_test_context();

    manager
        .create_session("Session 1".to_string(), context.clone())
        .unwrap();
    manager
        .create_session("Session 2".to_string(), context)
        .unwrap();

    let sessions = manager.list_sessions();
    assert_eq!(sessions.len(), 2);
}

#[test]
fn test_close_session() {
    let mut manager = SessionManager::new(5);
    let context = create_test_context();

    let session = manager
        .create_session("Test Session".to_string(), context)
        .unwrap();

    // Close session (should work same as delete)
    manager.close_session(&session.id).unwrap();

    assert_eq!(manager.session_count(), 0);
    assert!(manager.get_session(&session.id).is_err());
}
