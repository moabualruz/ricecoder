//! Unit tests for session management

use ricecoder_storage::*;
use tempfile::TempDir;

#[test]
fn test_session_creation() {
    let temp_dir = TempDir::new().unwrap();
    let manager = SessionManager::new(temp_dir.path().to_path_buf());

    let session = manager.create_session("Test Session".to_string(), "test_user".to_string()).unwrap();

    assert_eq!(session.name, "Test Session");
    assert_eq!(session.owner, "test_user");
    assert!(!session.is_shared);
    assert!(session.state.command_history.is_empty());
}

#[test]
fn test_session_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let manager = SessionManager::new(temp_dir.path().to_path_buf());

    // Create and save session
    let mut session = manager.create_session("Persistent Session".to_string(), "test_user".to_string()).unwrap();
    session.state.command_history.push("ls -la".to_string());
    manager.save_session(&session).unwrap();

    // Load session
    let loaded_session = manager.load_session(&session.id).unwrap();
    assert_eq!(loaded_session.name, "Persistent Session");
    assert_eq!(loaded_session.state.command_history.len(), 1);
    assert_eq!(loaded_session.state.command_history[0], "ls -la");
}

#[test]
fn test_session_listing() {
    let temp_dir = TempDir::new().unwrap();
    let manager = SessionManager::new(temp_dir.path().to_path_buf());

    // Create multiple sessions
    let session1 = manager.create_session("Session 1".to_string(), "user1".to_string()).unwrap();
    let session2 = manager.create_session("Session 2".to_string(), "user2".to_string()).unwrap();

    let sessions = manager.list_sessions().unwrap();
    assert_eq!(sessions.len(), 2);

    let session_ids: Vec<_> = sessions.iter().map(|s| &s.id).collect();
    assert!(session_ids.contains(&session1.id));
    assert!(session_ids.contains(&session2.id));
}

#[test]
fn test_session_deletion() {
    let temp_dir = TempDir::new().unwrap();
    let manager = SessionManager::new(temp_dir.path().to_path_buf());

    let session = manager.create_session("To Delete".to_string(), "test_user".to_string()).unwrap();
    manager.save_session(&session).unwrap();

    // Verify session exists
    assert!(manager.load_session(&session.id).is_ok());

    // Delete session
    manager.delete_session(&session.id).unwrap();

    // Verify session is gone
    assert!(manager.load_session(&session.id).is_err());
}

#[test]
fn test_session_update() {
    let temp_dir = TempDir::new().unwrap();
    let manager = SessionManager::new(temp_dir.path().to_path_buf());

    let mut session = manager.create_session("Original Name".to_string(), "test_user".to_string()).unwrap();
    session.name = "Updated Name".to_string();
    session.state.command_history.push("pwd".to_string());

    manager.save_session(&session).unwrap();

    let loaded = manager.load_session(&session.id).unwrap();
    assert_eq!(loaded.name, "Updated Name");
    assert_eq!(loaded.state.command_history.len(), 1);
    assert_eq!(loaded.state.command_history[0], "pwd");
}

#[test]
fn test_session_sharing() {
    let temp_dir = TempDir::new().unwrap();
    let manager = SessionManager::new(temp_dir.path().to_path_buf());

    let mut session = manager.create_session("Shared Session".to_string(), "owner".to_string()).unwrap();
    session.is_shared = true;
    session.shared_with = vec!["user1".to_string(), "user2".to_string()];

    manager.save_session(&session).unwrap();

    let loaded = manager.load_session(&session.id).unwrap();
    assert!(loaded.is_shared);
    assert_eq!(loaded.shared_with.len(), 2);
    assert!(loaded.shared_with.contains(&"user1".to_string()));
    assert!(loaded.shared_with.contains(&"user2".to_string()));
}

#[test]
fn test_session_metadata() {
    let temp_dir = TempDir::new().unwrap();
    let manager = SessionManager::new(temp_dir.path().to_path_buf());

    let mut session = manager.create_session("Meta Session".to_string(), "test_user".to_string()).unwrap();
    session.metadata.insert("project".to_string(), "ricecoder".to_string());
    session.metadata.insert("version".to_string(), "0.1.0".to_string());

    manager.save_session(&session).unwrap();

    let loaded = manager.load_session(&session.id).unwrap();
    assert_eq!(loaded.metadata.get("project"), Some(&"ricecoder".to_string()));
    assert_eq!(loaded.metadata.get("version"), Some(&"0.1.0".to_string()));
}

#[test]
fn test_session_search() {
    let temp_dir = TempDir::new().unwrap();
    let manager = SessionManager::new(temp_dir.path().to_path_buf());

    manager.create_session("Alpha Project".to_string(), "user1".to_string()).unwrap();
    manager.create_session("Beta Project".to_string(), "user1".to_string()).unwrap();
    manager.create_session("Gamma Tool".to_string(), "user2".to_string()).unwrap();

    let results = manager.search_sessions("Project").unwrap();
    assert_eq!(results.len(), 2);

    let names: Vec<_> = results.iter().map(|s| &s.name).collect();
    assert!(names.contains(&"Alpha Project".to_string()));
    assert!(names.contains(&"Beta Project".to_string()));
}

#[test]
fn test_session_validation() {
    let temp_dir = TempDir::new().unwrap();
    let manager = SessionManager::new(temp_dir.path().to_path_buf());

    // Test empty name validation
    let result = manager.create_session("".to_string(), "test_user".to_string());
    assert!(result.is_err());

    // Test empty owner validation
    let result = manager.create_session("Valid Name".to_string(), "".to_string());
    assert!(result.is_err());

    // Test valid session
    let result = manager.create_session("Valid Session".to_string(), "valid_user".to_string());
    assert!(result.is_ok());
}

#[test]
fn test_session_backup_and_restore() {
    let temp_dir = TempDir::new().unwrap();
    let backup_dir = TempDir::new().unwrap();

    let manager = SessionManager::new(temp_dir.path().to_path_buf());

    // Create test session
    let session = manager.create_session("Backup Test".to_string(), "test_user".to_string()).unwrap();
    manager.save_session(&session).unwrap();

    // Backup sessions
    manager.backup_sessions(backup_dir.path()).unwrap();

    // Verify backup exists
    let backup_files = std::fs::read_dir(backup_dir.path()).unwrap();
    let count = backup_files.count();
    assert!(count > 0);
}

#[test]
fn test_concurrent_session_access() {
    let temp_dir = TempDir::new().unwrap();
    let manager = SessionManager::new(temp_dir.path().to_path_buf());

    let session = manager.create_session("Concurrent Test".to_string(), "test_user".to_string()).unwrap();
    manager.save_session(&session).unwrap();

    // Test concurrent reads (should work)
    let manager_clone = manager.clone();
    let session_id = session.id.clone();

    let handle = std::thread::spawn(move || {
        manager_clone.load_session(&session_id).unwrap()
    });

    let loaded_session = handle.join().unwrap();
    assert_eq!(loaded_session.name, "Concurrent Test");
}

#[test]
fn test_session_size_limits() {
    let temp_dir = TempDir::new().unwrap();
    let manager = SessionManager::new(temp_dir.path().to_path_buf());

    let mut session = manager.create_session("Size Test".to_string(), "test_user".to_string()).unwrap();

    // Add many commands to test size limits
    for i in 0..1000 {
        session.state.command_history.push(format!("command {}", i));
    }

    manager.save_session(&session).unwrap();

    let loaded = manager.load_session(&session.id).unwrap();
    assert_eq!(loaded.state.command_history.len(), 1000);
}

#[test]
fn test_session_cleanup() {
    let temp_dir = TempDir::new().unwrap();
    let manager = SessionManager::new(temp_dir.path().to_path_buf());

    // Create sessions with old timestamps
    let mut session1 = manager.create_session("Old Session 1".to_string(), "user1".to_string()).unwrap();
    let mut session2 = manager.create_session("Old Session 2".to_string(), "user2".to_string()).unwrap();

    // Simulate old timestamps (30 days ago)
    let old_time = chrono::Utc::now() - chrono::Duration::days(30);
    session1.created_at = old_time;
    session2.created_at = old_time;

    manager.save_session(&session1).unwrap();
    manager.save_session(&session2).unwrap();

    // Cleanup old sessions (older than 7 days)
    let deleted_count = manager.cleanup_old_sessions(7).unwrap();
    assert_eq!(deleted_count, 2);

    // Verify sessions are gone
    assert!(manager.load_session(&session1.id).is_err());
    assert!(manager.load_session(&session2.id).is_err());
}

#[test]
fn test_session_statistics() {
    let temp_dir = TempDir::new().unwrap();
    let manager = SessionManager::new(temp_dir.path().to_path_buf());

    manager.create_session("Stats Session 1".to_string(), "user1".to_string()).unwrap();
    manager.create_session("Stats Session 2".to_string(), "user1".to_string()).unwrap();
    manager.create_session("Stats Session 3".to_string(), "user2".to_string()).unwrap();

    let stats = manager.get_statistics().unwrap();

    assert_eq!(stats.total_sessions, 3);
    assert_eq!(stats.sessions_by_owner.get("user1"), Some(&2));
    assert_eq!(stats.sessions_by_owner.get("user2"), Some(&1));
}

#[test]
fn test_session_export_import() {
    let temp_dir = TempDir::new().unwrap();
    let export_dir = TempDir::new().unwrap();

    let manager = SessionManager::new(temp_dir.path().to_path_buf());

    let session = manager.create_session("Export Test".to_string(), "test_user".to_string()).unwrap();
    manager.save_session(&session).unwrap();

    // Export session
    let export_path = export_dir.path().join("exported_session.json");
    manager.export_session(&session.id, &export_path).unwrap();

    // Verify export file exists
    assert!(export_path.exists());

    // Import session
    let imported_session = manager.import_session(&export_path).unwrap();
    assert_eq!(imported_session.name, "Export Test");
    assert_eq!(imported_session.owner, "test_user");
}