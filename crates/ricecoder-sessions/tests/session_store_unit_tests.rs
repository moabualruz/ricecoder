//! Unit tests for SessionStore operations
//! **Feature: ricecoder-sessions, Unit Tests: SessionStore**
//! **Validates: Requirements 2.1, 2.2**

use ricecoder_sessions::{
    Session, SessionContext, SessionMode, SessionStore,
};
use std::fs;
use tempfile::TempDir;

fn create_test_session(name: &str) -> Session {
    let context = SessionContext::new("openai".to_string(), "gpt-4".to_string(), SessionMode::Chat);
    Session::new(name.to_string(), context)
}

#[tokio::test]
async fn test_session_store_save_and_load() {
    let temp_dir = TempDir::new().unwrap();
    let sessions_dir = temp_dir.path().join("sessions");
    let archive_dir = temp_dir.path().join("archive");

    let store = SessionStore::with_dirs(sessions_dir.clone(), archive_dir).unwrap();

    let session = create_test_session("test_session");
    let session_id = session.id.clone();

    // Save the session
    store.save(&session).await.unwrap();

    // Verify file was created
    assert!(store.exists(&session_id));

    // Load the session
    let loaded = store.load(&session_id).await.unwrap();

    // Verify loaded session matches original
    assert_eq!(loaded.id, session.id);
    assert_eq!(loaded.name, session.name);
    assert_eq!(loaded.status, session.status);
    assert_eq!(loaded.context.provider, session.context.provider);
    assert_eq!(loaded.context.model, session.context.model);
}

#[tokio::test]
async fn test_session_store_list_sessions() {
    let temp_dir = TempDir::new().unwrap();
    let sessions_dir = temp_dir.path().join("sessions");
    let archive_dir = temp_dir.path().join("archive");

    let store = SessionStore::with_dirs(sessions_dir, archive_dir).unwrap();

    // Create and save multiple sessions
    let session1 = create_test_session("session1");
    let session2 = create_test_session("session2");
    let session3 = create_test_session("session3");

    store.save(&session1).await.unwrap();
    store.save(&session2).await.unwrap();
    store.save(&session3).await.unwrap();

    // List sessions
    let sessions = store.list().await.unwrap();

    // Verify all sessions are listed
    assert_eq!(sessions.len(), 3);
    let session_ids: Vec<String> = sessions.iter().map(|s| s.id.clone()).collect();
    assert!(session_ids.contains(&session1.id));
    assert!(session_ids.contains(&session2.id));
    assert!(session_ids.contains(&session3.id));
}

#[tokio::test]
async fn test_session_store_delete_and_archive() {
    let temp_dir = TempDir::new().unwrap();
    let sessions_dir = temp_dir.path().join("sessions");
    let archive_dir = temp_dir.path().join("archive");

    let store = SessionStore::with_dirs(sessions_dir, archive_dir).unwrap();

    let session = create_test_session("test_session");
    let session_id = session.id.clone();

    // Save the session
    store.save(&session).await.unwrap();
    assert!(store.exists(&session_id));

    // Delete the session
    store.delete(&session_id).await.unwrap();

    // Verify session file is deleted
    assert!(!store.exists(&session_id));

    // Verify session is archived
    let archived_path = store.archive_dir().join(format!("{}.json", session_id));
    assert!(archived_path.exists());
}

#[tokio::test]
async fn test_session_store_export() {
    let temp_dir = TempDir::new().unwrap();
    let sessions_dir = temp_dir.path().join("sessions");
    let archive_dir = temp_dir.path().join("archive");
    let export_path = temp_dir.path().join("export.json");

    let store = SessionStore::with_dirs(sessions_dir, archive_dir).unwrap();

    let session = create_test_session("test_session");
    let session_id = session.id.clone();

    // Save the session
    store.save(&session).await.unwrap();

    // Export the session
    store.export(&session_id, &export_path).await.unwrap();

    // Verify export file exists
    assert!(export_path.exists());

    // Verify export content is valid JSON
    let content = fs::read_to_string(&export_path).unwrap();
    let exported_session: Session = serde_json::from_str(&content).unwrap();
    assert_eq!(exported_session.id, session_id);
    assert_eq!(exported_session.name, session.name);
}

#[tokio::test]
async fn test_session_store_load_nonexistent() {
    let temp_dir = TempDir::new().unwrap();
    let sessions_dir = temp_dir.path().join("sessions");
    let archive_dir = temp_dir.path().join("archive");

    let store = SessionStore::with_dirs(sessions_dir, archive_dir).unwrap();

    // Try to load a non-existent session
    let result = store.load("nonexistent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_session_store_delete_nonexistent() {
    let temp_dir = TempDir::new().unwrap();
    let sessions_dir = temp_dir.path().join("sessions");
    let archive_dir = temp_dir.path().join("archive");

    let store = SessionStore::with_dirs(sessions_dir, archive_dir).unwrap();

    // Try to delete a non-existent session
    let result = store.delete("nonexistent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_session_store_exists() {
    let temp_dir = TempDir::new().unwrap();
    let sessions_dir = temp_dir.path().join("sessions");
    let archive_dir = temp_dir.path().join("archive");

    let store = SessionStore::with_dirs(sessions_dir, archive_dir).unwrap();

    let session = create_test_session("test_session");
    let session_id = session.id.clone();

    // Session should not exist initially
    assert!(!store.exists(&session_id));

    // Save the session
    store.save(&session).await.unwrap();

    // Session should exist now
    assert!(store.exists(&session_id));

    // Delete the session
    store.delete(&session_id).await.unwrap();

    // Session should not exist after deletion
    assert!(!store.exists(&session_id));
}

#[tokio::test]
async fn test_session_store_preserves_session_data() {
    let temp_dir = TempDir::new().unwrap();
    let sessions_dir = temp_dir.path().join("sessions");
    let archive_dir = temp_dir.path().join("archive");

    let store = SessionStore::with_dirs(sessions_dir, archive_dir).unwrap();

    let mut session = create_test_session("test_session");
    session.context.project_path = Some("/path/to/project".to_string());
    session.context.files.push("file1.rs".to_string());
    session.context.files.push("file2.rs".to_string());

    let session_id = session.id.clone();

    // Save the session
    store.save(&session).await.unwrap();

    // Load the session
    let loaded = store.load(&session_id).await.unwrap();

    // Verify all data is preserved
    assert_eq!(loaded.context.project_path, Some("/path/to/project".to_string()));
    assert_eq!(loaded.context.files.len(), 2);
    assert!(loaded.context.files.contains(&"file1.rs".to_string()));
    assert!(loaded.context.files.contains(&"file2.rs".to_string()));
}

#[tokio::test]
async fn test_session_store_list_empty_directory() {
    let temp_dir = TempDir::new().unwrap();
    let sessions_dir = temp_dir.path().join("sessions");
    let archive_dir = temp_dir.path().join("archive");

    let store = SessionStore::with_dirs(sessions_dir, archive_dir).unwrap();

    // List sessions from empty directory
    let sessions = store.list().await.unwrap();

    // Should return empty list
    assert_eq!(sessions.len(), 0);
}

#[tokio::test]
async fn test_session_store_multiple_saves() {
    let temp_dir = TempDir::new().unwrap();
    let sessions_dir = temp_dir.path().join("sessions");
    let archive_dir = temp_dir.path().join("archive");

    let store = SessionStore::with_dirs(sessions_dir, archive_dir).unwrap();

    let mut session = create_test_session("test_session");
    let session_id = session.id.clone();

    // Save the session
    store.save(&session).await.unwrap();

    // Modify and save again
    session.name = "updated_name".to_string();
    store.save(&session).await.unwrap();

    // Load and verify the update
    let loaded = store.load(&session_id).await.unwrap();
    assert_eq!(loaded.name, "updated_name");
}

#[tokio::test]
async fn test_session_store_export_nonexistent() {
    let temp_dir = TempDir::new().unwrap();
    let sessions_dir = temp_dir.path().join("sessions");
    let archive_dir = temp_dir.path().join("archive");
    let export_path = temp_dir.path().join("export.json");

    let store = SessionStore::with_dirs(sessions_dir, archive_dir).unwrap();

    // Try to export a non-existent session
    let result = store.export("nonexistent", &export_path).await;
    assert!(result.is_err());
}
