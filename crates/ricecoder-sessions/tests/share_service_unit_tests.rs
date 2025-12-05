//! Unit tests for ShareService operations
//! **Feature: ricecoder-sessions, Unit Tests: ShareService**
//! **Validates: Requirements 3.1, 3.2, 3.3, 3.4, 3.5**

use chrono::Duration;
use ricecoder_sessions::{
    Message, MessageRole, Session, SessionContext, SessionMode, SharePermissions, ShareService,
};

fn create_test_session(name: &str) -> Session {
    let context = SessionContext::new("openai".to_string(), "gpt-4".to_string(), SessionMode::Chat);
    Session::new(name.to_string(), context)
}

fn create_test_permissions() -> SharePermissions {
    SharePermissions {
        read_only: true,
        include_history: true,
        include_context: true,
    }
}

#[test]
fn test_generate_share_link() {
    let service = ShareService::new();
    let session = create_test_session("test_session");
    let permissions = create_test_permissions();

    let share = service
        .generate_share_link(&session.id, permissions, None)
        .unwrap();

    assert!(!share.id.is_empty());
    assert_eq!(share.session_id, session.id);
    assert_eq!(share.permissions.read_only, true);
    assert_eq!(share.permissions.include_history, true);
    assert_eq!(share.permissions.include_context, true);
    assert!(share.expires_at.is_none());
}

#[test]
fn test_generate_share_link_with_expiration() {
    let service = ShareService::new();
    let session = create_test_session("test_session");
    let permissions = create_test_permissions();
    let expires_in = Duration::hours(24);

    let share = service
        .generate_share_link(&session.id, permissions, Some(expires_in))
        .unwrap();

    assert!(share.expires_at.is_some());
}

#[test]
fn test_get_share() {
    let service = ShareService::new();
    let session = create_test_session("test_session");
    let permissions = create_test_permissions();

    let share = service
        .generate_share_link(&session.id, permissions, None)
        .unwrap();

    let retrieved = service.get_share(&share.id).unwrap();
    assert_eq!(retrieved.id, share.id);
    assert_eq!(retrieved.session_id, share.session_id);
}

#[test]
fn test_get_nonexistent_share() {
    let service = ShareService::new();

    let result = service.get_share("nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_share_link_uniqueness() {
    let service = ShareService::new();
    let session = create_test_session("test_session");
    let permissions = create_test_permissions();

    let share1 = service
        .generate_share_link(&session.id, permissions.clone(), None)
        .unwrap();

    let share2 = service
        .generate_share_link(&session.id, permissions, None)
        .unwrap();

    // Share IDs should be unique
    assert_ne!(share1.id, share2.id);
}

#[test]
fn test_create_shared_session_view_with_history() {
    let service = ShareService::new();
    let mut session = create_test_session("test_session");

    // Add messages to history
    session
        .history
        .push(Message::new(MessageRole::User, "Hello".to_string()));
    session
        .history
        .push(Message::new(MessageRole::Assistant, "Hi".to_string()));

    let permissions = SharePermissions {
        read_only: true,
        include_history: true,
        include_context: true,
    };

    let shared_view = service.create_shared_session_view(&session, &permissions);

    // History should be included
    assert_eq!(shared_view.history.len(), 2);
}

#[test]
fn test_create_shared_session_view_without_history() {
    let service = ShareService::new();
    let mut session = create_test_session("test_session");

    // Add messages to history
    session
        .history
        .push(Message::new(MessageRole::User, "Hello".to_string()));
    session
        .history
        .push(Message::new(MessageRole::Assistant, "Hi".to_string()));

    let permissions = SharePermissions {
        read_only: true,
        include_history: false,
        include_context: true,
    };

    let shared_view = service.create_shared_session_view(&session, &permissions);

    // History should be cleared
    assert_eq!(shared_view.history.len(), 0);
}

#[test]
fn test_create_shared_session_view_without_context() {
    let service = ShareService::new();
    let mut session = create_test_session("test_session");

    session.context.project_path = Some("/path/to/project".to_string());
    session.context.files.push("file1.rs".to_string());

    let permissions = SharePermissions {
        read_only: true,
        include_history: true,
        include_context: false,
    };

    let shared_view = service.create_shared_session_view(&session, &permissions);

    // Context should be cleared
    assert_eq!(shared_view.context.files.len(), 0);
    assert_eq!(shared_view.context.custom.len(), 0);
}

#[test]
fn test_import_shared_session() {
    let service = ShareService::new();
    let mut session = create_test_session("original_session");
    session.context.project_path = Some("/path/to/project".to_string());
    session
        .history
        .push(Message::new(MessageRole::User, "Hello".to_string()));

    let permissions = create_test_permissions();
    let share = service
        .generate_share_link(&session.id, permissions, None)
        .unwrap();

    let imported = service.import_shared_session(&share.id, &session).unwrap();

    // Imported session should have different ID
    assert_ne!(imported.id, session.id);

    // But same content
    assert_eq!(imported.context.project_path, session.context.project_path);
    assert_eq!(imported.history.len(), session.history.len());
}

#[test]
fn test_import_shared_session_with_expired_share() {
    let service = ShareService::new();
    let session = create_test_session("test_session");
    let permissions = create_test_permissions();

    // Create a share that expires immediately
    let share = service
        .generate_share_link(&session.id, permissions, Some(Duration::seconds(-1)))
        .unwrap();

    // Try to import from expired share
    let result = service.import_shared_session(&share.id, &session);
    assert!(result.is_err());
}

#[test]
fn test_revoke_share() {
    let service = ShareService::new();
    let session = create_test_session("test_session");
    let permissions = create_test_permissions();

    let share = service
        .generate_share_link(&session.id, permissions, None)
        .unwrap();

    // Share should exist
    assert!(service.get_share(&share.id).is_ok());

    // Revoke the share
    service.revoke_share(&share.id).unwrap();

    // Share should no longer exist
    assert!(service.get_share(&share.id).is_err());
}

#[test]
fn test_revoke_nonexistent_share() {
    let service = ShareService::new();

    let result = service.revoke_share("nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_list_shares() {
    let service = ShareService::new();
    let session1 = create_test_session("session1");
    let session2 = create_test_session("session2");
    let permissions = create_test_permissions();

    service
        .generate_share_link(&session1.id, permissions.clone(), None)
        .unwrap();
    service
        .generate_share_link(&session2.id, permissions, None)
        .unwrap();

    let shares = service.list_shares().unwrap();
    assert_eq!(shares.len(), 2);
}

#[test]
fn test_list_shares_excludes_expired() {
    let service = ShareService::new();
    let session1 = create_test_session("session1");
    let session2 = create_test_session("session2");
    let permissions = create_test_permissions();

    // Create a non-expiring share
    service
        .generate_share_link(&session1.id, permissions.clone(), None)
        .unwrap();

    // Create an expiring share
    service
        .generate_share_link(&session2.id, permissions, Some(Duration::seconds(-1)))
        .unwrap();

    let shares = service.list_shares().unwrap();
    // Should only include the non-expired share
    assert_eq!(shares.len(), 1);
}

#[test]
fn test_cleanup_expired_shares() {
    let service = ShareService::new();
    let session1 = create_test_session("session1");
    let session2 = create_test_session("session2");
    let permissions = create_test_permissions();

    // Create a non-expiring share
    service
        .generate_share_link(&session1.id, permissions.clone(), None)
        .unwrap();

    // Create an expiring share
    service
        .generate_share_link(&session2.id, permissions, Some(Duration::seconds(-1)))
        .unwrap();

    // Cleanup expired shares
    let removed = service.cleanup_expired_shares().unwrap();
    assert_eq!(removed, 1);

    // Only non-expired share should remain
    let shares = service.list_shares().unwrap();
    assert_eq!(shares.len(), 1);
}

#[test]
fn test_share_permissions_read_only() {
    let permissions = SharePermissions {
        read_only: true,
        include_history: true,
        include_context: true,
    };

    assert!(permissions.read_only);
}

#[test]
fn test_share_permissions_include_history() {
    let permissions = SharePermissions {
        read_only: true,
        include_history: true,
        include_context: false,
    };

    assert!(permissions.include_history);
    assert!(!permissions.include_context);
}

#[test]
fn test_multiple_shares_same_session() {
    let service = ShareService::new();
    let session = create_test_session("test_session");
    let permissions = create_test_permissions();

    let share1 = service
        .generate_share_link(&session.id, permissions.clone(), None)
        .unwrap();

    let share2 = service
        .generate_share_link(&session.id, permissions, None)
        .unwrap();

    // Both shares should exist
    assert!(service.get_share(&share1.id).is_ok());
    assert!(service.get_share(&share2.id).is_ok());

    // But have different IDs
    assert_ne!(share1.id, share2.id);
}

#[test]
fn test_share_preserves_session_metadata() {
    let service = ShareService::new();
    let mut session = create_test_session("test_session");
    session.context.provider = "anthropic".to_string();
    session.context.model = "claude-3".to_string();

    let permissions = create_test_permissions();
    let share = service
        .generate_share_link(&session.id, permissions, None)
        .unwrap();

    let imported = service.import_shared_session(&share.id, &session).unwrap();

    // Metadata should be preserved
    assert_eq!(imported.context.provider, "anthropic");
    assert_eq!(imported.context.model, "claude-3");
}

#[test]
fn test_default_share_service() {
    let service = ShareService::default();
    let shares = service.list_shares().unwrap();
    assert_eq!(shares.len(), 0);
}
