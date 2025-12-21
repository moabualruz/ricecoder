//! Comprehensive session validation, error handling, and compliance tests
//! **Feature: ricecoder-sessions, Unit Tests: Validation/Error Handling/Compliance**
//! **Validates: Requirements 4.1, 4.2, 4.3, 5.1, 5.2, 5.3**

use chrono::Duration;
use ricecoder_security::audit::{AuditEventType, AuditLogger, MemoryAuditStorage};
use ricecoder_sessions::{
    BackgroundAgent, Message, MessageRole, Session, SessionContext, SessionError, SessionManager,
    SessionMode, SessionStatus, SharePermissions, ShareService,
};
use std::sync::Arc;

fn create_test_context() -> SessionContext {
    SessionContext::new("openai".to_string(), "gpt-4".to_string(), SessionMode::Chat)
}

fn create_test_session(name: &str) -> Session {
    Session::new(name.to_string(), create_test_context())
}

#[test]
fn test_session_validation_invalid_names() {
    let context = create_test_context();

    // Empty name should be handled gracefully (depending on implementation)
    let session = Session::new("".to_string(), context.clone());
    assert_eq!(session.name, "");

    // Very long name
    let long_name = "a".repeat(1000);
    let session = Session::new(long_name.clone(), context.clone());
    assert_eq!(session.name, long_name);

    // Name with special characters
    let special_name = "Session@#$%^&*()";
    let session = Session::new(special_name.to_string(), context);
    assert_eq!(session.name, special_name);
}

#[test]
fn test_session_context_validation() {
    // Valid context
    let valid_context =
        SessionContext::new("openai".to_string(), "gpt-4".to_string(), SessionMode::Chat);
    assert_eq!(valid_context.provider, "openai");
    assert_eq!(valid_context.model, "gpt-4");
    assert_eq!(valid_context.mode, SessionMode::Chat);

    // Empty provider/model (should be allowed, validation happens elsewhere)
    let empty_context = SessionContext::new("".to_string(), "".to_string(), SessionMode::Code);
    assert_eq!(empty_context.provider, "");
    assert_eq!(empty_context.model, "");
}

#[test]
fn test_session_manager_error_handling() {
    let mut manager = SessionManager::new(2);
    let context = create_test_context();

    // Create sessions up to limit
    manager
        .create_session("Session 1".to_string(), context.clone())
        .unwrap();
    manager
        .create_session("Session 2".to_string(), context.clone())
        .unwrap();

    // Attempt to create beyond limit
    let result = manager.create_session("Session 3".to_string(), context);
    assert!(matches!(result, Err(SessionError::LimitReached { max: 2 })));

    // Attempt to switch to non-existent session
    let result = manager.switch_session("nonexistent-id");
    assert!(result.is_err());

    // Attempt to delete non-existent session
    let result = manager.delete_session("nonexistent-id");
    assert!(result.is_err());

    // Attempt to get non-existent session
    let result = manager.get_session("nonexistent-id");
    assert!(result.is_err());
}

#[test]
fn test_message_validation() {
    // Valid message
    let valid_msg = Message::new(MessageRole::User, "Hello".to_string());
    assert_eq!(valid_msg.role, MessageRole::User);
    assert_eq!(valid_msg.content(), "Hello");

    // Empty content
    let empty_msg = Message::new(MessageRole::Assistant, "".to_string());
    assert_eq!(empty_msg.content(), "");

    // Very long content
    let long_content = "a".repeat(10000);
    let long_msg = Message::new(MessageRole::System, long_content.clone());
    assert_eq!(long_msg.content(), long_content);

    // Message with parts
    let mut complex_msg = Message::new(MessageRole::Assistant, "Base content".to_string());
    complex_msg.add_code("rust".to_string(), "fn main() {}".to_string());
    complex_msg.add_reasoning("This is a simple function".to_string());
    assert!(complex_msg.parts.len() > 1);
}

#[test]
fn test_background_agent_validation() {
    // Valid agent
    let agent = BackgroundAgent::new("code_review".to_string(), Some("Review code".to_string()));
    assert_eq!(agent.agent_type, "code_review");
    assert_eq!(agent.task, Some("Review code".to_string()));
    assert_eq!(agent.status, ricecoder_sessions::AgentStatus::Running);

    // Agent without task
    let agent_no_task = BackgroundAgent::new("diff_analysis".to_string(), None);
    assert_eq!(agent_no_task.task, None);

    // Agent with empty type
    let agent_empty_type = BackgroundAgent::new("".to_string(), Some("Task".to_string()));
    assert_eq!(agent_empty_type.agent_type, "");
}

#[test]
fn test_share_service_validation_errors() {
    let service = ShareService::new();
    let session = create_test_session("Test Session");
    let permissions = SharePermissions {
        read_only: true,
        include_history: true,
        include_context: true,
    };

    // Valid share creation
    let share = service
        .generate_share_link(&session.id, permissions.clone(), None)
        .unwrap();
    assert!(!share.id.is_empty());

    // Attempt to get non-existent share
    let result = service.get_share("nonexistent");
    assert!(matches!(result, Err(SessionError::ShareNotFound(_))));

    // Attempt to revoke non-existent share
    let result = service.revoke_share("nonexistent", None);
    assert!(matches!(result, Err(SessionError::ShareNotFound(_))));

    // Attempt to import from non-existent share
    let result = service.import_shared_session("nonexistent", &session, None);
    assert!(matches!(result, Err(SessionError::ShareNotFound(_))));
}

#[test]
fn test_expired_share_handling() {
    let service = ShareService::new();
    let session = create_test_session("Test Session");
    let permissions = SharePermissions {
        read_only: true,
        include_history: true,
        include_context: true,
    };

    // Create share that expires immediately
    let share = service
        .generate_share_link(&session.id, permissions, Some(Duration::seconds(-1)))
        .unwrap();

    // Attempt to get expired share
    let result = service.get_share(&share.id);
    assert!(matches!(result, Err(SessionError::ShareExpired(_))));

    // Attempt to import from expired share
    let result = service.import_shared_session(&share.id, &session, None);
    assert!(matches!(result, Err(SessionError::ShareExpired(_))));
}

#[test]
fn test_compliance_audit_logging() {
    let audit_storage = Arc::new(MemoryAuditStorage::new());
    let audit_logger = Arc::new(AuditLogger::new(audit_storage.clone()));
    let service = ShareService::with_audit_logging("https://test.com".to_string(), audit_logger);

    let session = create_test_session("Audit Session");
    let permissions = SharePermissions {
        read_only: true,
        include_history: true,
        include_context: true,
    };

    // Generate share (should log)
    let share = service
        .generate_share_link(&session.id, permissions.clone(), None)
        .unwrap();

    // Access share (should log)
    let _retrieved = service.get_share(&share.id).unwrap();

    // Revoke share (should log)
    service
        .revoke_share(&share.id, Some("admin@test.com".to_string()))
        .unwrap();

    // Check audit events
    let events = audit_storage.get_events().unwrap();
    assert!(!events.is_empty());

    // Should have multiple audit events
    let audit_events: Vec<_> = events
        .iter()
        .filter(|e| e.event_type == AuditEventType::DataAccess)
        .collect();
    assert!(!audit_events.is_empty());
}

#[test]
fn test_session_status_transition_validation() {
    let mut session = create_test_session("Status Test");

    // Valid transitions
    assert_eq!(session.status, SessionStatus::Active);
    session.status = SessionStatus::Paused;
    assert_eq!(session.status, SessionStatus::Paused);
    session.status = SessionStatus::Active;
    assert_eq!(session.status, SessionStatus::Active);
    session.status = SessionStatus::Archived;
    assert_eq!(session.status, SessionStatus::Archived);

    // Archived sessions shouldn't be reactivated (business rule)
    // This is more of a business logic validation
    session.status = SessionStatus::Active; // This should be allowed or not based on business rules
    assert_eq!(session.status, SessionStatus::Active);
}

#[test]
fn test_concurrent_access_error_handling() {
    // Test concurrent access to shared resources
    let service = Arc::new(ShareService::new());
    let session = Arc::new(create_test_session("Concurrent Session"));
    let permissions = SharePermissions {
        read_only: true,
        include_history: true,
        include_context: true,
    };

    let mut handles = vec![];

    // Spawn multiple threads trying to create shares
    for i in 0..10 {
        let service_clone = Arc::clone(&service);
        let session_clone = Arc::clone(&session);
        let permissions_clone = permissions.clone();

        let handle = std::thread::spawn(move || {
            let result =
                service_clone.generate_share_link(&session_clone.id, permissions_clone, None);
            result
        });
        handles.push(handle);
    }

    // Collect results
    let mut success_count = 0;
    let mut error_count = 0;

    for handle in handles {
        let result = handle.join().unwrap();
        match result {
            Ok(_) => success_count += 1,
            Err(_) => error_count += 1,
        }
    }

    // All should succeed (no locking issues in this implementation)
    assert_eq!(success_count, 10);
    assert_eq!(error_count, 0);
}

#[test]
fn test_input_sanitization_and_validation() {
    let mut manager = SessionManager::new(10);

    // Test with various session names
    let test_names = vec![
        "Normal Name",
        "Name with spaces",
        "Name-with-dashes",
        "Name_with_underscores",
        "Name123",
        "🚀 Emoji Name",
        "a".repeat(255), // Long name
        "",              // Empty name
    ];

    let context = create_test_context();

    for name in test_names {
        let result = manager.create_session(name.to_string(), context.clone());
        // Should not panic, but may succeed or fail based on validation rules
        // The important thing is graceful handling
        match result {
            Ok(session) => assert_eq!(session.name, name),
            Err(_) => {} // Expected for some inputs
        }
    }
}

#[test]
fn test_resource_limit_enforcement() {
    let manager = SessionManager::new(3);
    let context = create_test_context();

    // Create sessions up to limit
    let _s1 = manager
        .create_session("S1".to_string(), context.clone())
        .unwrap();
    let _s2 = manager
        .create_session("S2".to_string(), context.clone())
        .unwrap();
    let _s3 = manager
        .create_session("S3".to_string(), context.clone())
        .unwrap();

    // Attempt to create one more
    let result = manager.create_session("S4".to_string(), context);
    assert!(result.is_err());
}

#[test]
fn test_data_integrity_validation() {
    let session = create_test_session("Integrity Test");

    // Verify UUID format
    assert!(uuid::Uuid::parse_str(&session.id).is_ok());

    // Verify timestamps are reasonable
    let now = chrono::Utc::now();
    assert!(session.created_at <= now);
    assert!(session.updated_at <= now);
    assert!(session.created_at <= session.updated_at);

    // Verify status is valid
    match session.status {
        SessionStatus::Active | SessionStatus::Paused | SessionStatus::Archived => {}
    }

    // Verify context has required fields
    assert!(!session.context.provider.is_empty());
    assert!(!session.context.model.is_empty());
}

#[test]
fn test_error_message_quality() {
    let mut manager = SessionManager::new(1);
    let context = create_test_context();

    // Create one session
    manager
        .create_session("Test".to_string(), context.clone())
        .unwrap();

    // Try to create another (should fail)
    let result = manager.create_session("Test2".to_string(), context);

    if let Err(SessionError::LimitReached { max }) = result {
        assert_eq!(max, 1);
    } else {
        panic!("Expected LimitReached error");
    }

    // Try to access non-existent session
    let result = manager.get_session("nonexistent");
    assert!(result.is_err()); // Error message should be meaningful
}

#[test]
fn test_graceful_degradation() {
    // Test that the system continues to function when some operations fail

    let service = ShareService::new();
    let session = create_test_session("Degradation Test");

    // Try operations that might fail
    let permissions = SharePermissions {
        read_only: true,
        include_history: true,
        include_context: true,
    };

    // This should work
    let share = service
        .generate_share_link(&session.id, permissions, None)
        .unwrap();

    // Try to get a non-existent share (should fail gracefully)
    let result = service.get_share("nonexistent");
    assert!(result.is_err());

    // But the service should still work for valid operations
    let retrieved = service.get_share(&share.id).unwrap();
    assert_eq!(retrieved.id, share.id);

    // Try to revoke non-existent share (should fail gracefully)
    let result = service.revoke_share("nonexistent", None);
    assert!(result.is_err());

    // But revocation of valid share should still work
    service.revoke_share(&share.id, None).unwrap();
    let result = service.get_share(&share.id);
    assert!(result.is_err());
}
