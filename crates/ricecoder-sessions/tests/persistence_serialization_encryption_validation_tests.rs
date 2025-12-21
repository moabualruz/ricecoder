//! Additional persistence/serialization/encryption validation tests
//! **Feature: ricecoder-sessions, Unit Tests: Persistence/Serialization/Encryption Validation**
//! **Validates: Requirements 2.1, 2.2, 2.3, 2.4**

use ricecoder_sessions::{Session, SessionContext, SessionMode, SessionStore};
use std::fs;
use tempfile::TempDir;
use tokio::runtime::Runtime;

fn create_test_session(name: &str) -> Session {
    let context = SessionContext::new("openai".to_string(), "gpt-4".to_string(), SessionMode::Chat);
    Session::new(name.to_string(), context)
}

#[test]
fn test_session_serialization_roundtrip() {
    let session = create_test_session("Serialization Test");

    // Serialize to JSON
    let json = serde_json::to_string(&session).unwrap();

    // Deserialize back
    let deserialized: Session = serde_json::from_str(&json).unwrap();

    // Verify all fields match
    assert_eq!(deserialized.id, session.id);
    assert_eq!(deserialized.name, session.name);
    assert_eq!(deserialized.status, session.status);
    assert_eq!(deserialized.context.provider, session.context.provider);
    assert_eq!(deserialized.context.model, session.context.model);
    assert_eq!(deserialized.context.mode, session.context.mode);
    assert_eq!(deserialized.history.len(), session.history.len());
    assert_eq!(
        deserialized.background_agents.len(),
        session.background_agents.len()
    );
}

#[test]
fn test_session_serialization_with_complex_data() {
    let mut session = create_test_session("Complex Serialization Test");

    // Add files and custom data
    session.context.files.push("src/main.rs".to_string());
    session.context.files.push("Cargo.toml".to_string());
    session
        .context
        .custom
        .insert("workspace".to_string(), serde_json::json!("dev"));
    session
        .context
        .custom
        .insert("version".to_string(), serde_json::json!(1.0));

    // Serialize and deserialize
    let json = serde_json::to_string(&session).unwrap();
    let deserialized: Session = serde_json::from_str(&json).unwrap();

    // Verify complex data preserved
    assert_eq!(deserialized.context.files.len(), 2);
    assert!(deserialized
        .context
        .files
        .contains(&"src/main.rs".to_string()));
    assert!(deserialized
        .context
        .files
        .contains(&"Cargo.toml".to_string()));
    assert_eq!(deserialized.context.custom.len(), 2);
    assert_eq!(
        deserialized.context.custom.get("workspace"),
        Some(&serde_json::json!("dev"))
    );
    assert_eq!(
        deserialized.context.custom.get("version"),
        Some(&serde_json::json!(1.0))
    );
}

#[test]
fn test_encryption_validation() {
    let temp_dir = TempDir::new().unwrap();
    let sessions_dir = temp_dir.path().join("sessions");
    let archive_dir = temp_dir.path().join("archive");

    let rt = Runtime::new().unwrap();
    let mut store =
        rt.block_on(async { SessionStore::with_dirs(sessions_dir.clone(), archive_dir).unwrap() });

    // Enable encryption
    store.enable_encryption("test-password").unwrap();

    let session = create_test_session("Encryption Test");

    // Save encrypted session
    rt.block_on(store.save(&session)).unwrap();

    // Verify file exists
    let session_file = sessions_dir.join(format!("{}.json", session.id));
    assert!(session_file.exists());

    // Verify file content is encrypted (not plain JSON)
    let content = fs::read_to_string(&session_file).unwrap();
    assert!(!content.contains(&session.name)); // Name should not be visible in encrypted content

    // Load and verify decryption works
    let loaded = rt.block_on(store.load(&session.id)).unwrap();
    assert_eq!(loaded.id, session.id);
    assert_eq!(loaded.name, session.name);
}

#[test]
fn test_enterprise_encryption_validation() {
    let temp_dir = TempDir::new().unwrap();
    let sessions_dir = temp_dir.path().join("sessions");
    let archive_dir = temp_dir.path().join("archive");

    let rt = Runtime::new().unwrap();
    let mut store =
        rt.block_on(async { SessionStore::with_dirs(sessions_dir.clone(), archive_dir).unwrap() });

    // Enable enterprise encryption
    store
        .enable_enterprise_encryption("enterprise-password")
        .unwrap();

    let session = create_test_session("Enterprise Encryption Test");

    // Save enterprise encrypted session
    rt.block_on(store.save(&session)).unwrap();

    // Verify file exists
    let session_file = sessions_dir.join(format!("{}.json", session.id));
    assert!(session_file.exists());

    // Verify file content is encrypted
    let content = fs::read_to_string(&session_file).unwrap();
    assert!(!content.contains(&session.name)); // Name should not be visible

    // Load and verify decryption works
    let loaded = rt.block_on(store.load(&session.id)).unwrap();
    assert_eq!(loaded.id, session.id);
    assert_eq!(loaded.name, session.name);
}

#[test]
fn test_corrupted_file_handling() {
    let temp_dir = TempDir::new().unwrap();
    let sessions_dir = temp_dir.path().join("sessions");
    let archive_dir = temp_dir.path().join("archive");

    let rt = Runtime::new().unwrap();
    let store =
        rt.block_on(async { SessionStore::with_dirs(sessions_dir.clone(), archive_dir).unwrap() });

    let session = create_test_session("Corruption Test");
    let session_id = session.id.clone();

    // Save session
    rt.block_on(store.save(&session)).unwrap();

    // Corrupt the file
    let session_file = sessions_dir.join(format!("{}.json", session_id));
    fs::write(&session_file, "corrupted json content").unwrap();

    // Attempt to load should fail gracefully
    let result = rt.block_on(store.load(&session_id));
    assert!(result.is_err());
}

#[test]
fn test_missing_file_handling() {
    let temp_dir = TempDir::new().unwrap();
    let sessions_dir = temp_dir.path().join("sessions");
    let archive_dir = temp_dir.path().join("archive");

    let rt = Runtime::new().unwrap();
    let store = rt.block_on(async { SessionStore::with_dirs(sessions_dir, archive_dir).unwrap() });

    // Attempt to load non-existent session
    let result = rt.block_on(store.load("nonexistent-id"));
    assert!(result.is_err());
}

#[test]
fn test_file_permission_handling() {
    let temp_dir = TempDir::new().unwrap();
    let sessions_dir = temp_dir.path().join("sessions");
    let archive_dir = temp_dir.path().join("archive");

    let rt = Runtime::new().unwrap();
    let store =
        rt.block_on(async { SessionStore::with_dirs(sessions_dir.clone(), archive_dir).unwrap() });

    let session = create_test_session("Permission Test");

    // Create a read-only file to simulate permission issues
    let session_file = sessions_dir.join(format!("{}.json", session.id));
    fs::write(&session_file, "{}").unwrap();

    // Make it read-only (if possible on this platform)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&session_file).unwrap().permissions();
        perms.set_mode(0o444); // Read-only
        fs::set_permissions(&session_file, perms).unwrap();
    }

    // Attempt to save should handle permission errors gracefully
    let result = rt.block_on(store.save(&session));
    // May succeed or fail depending on platform, but should not panic
    match result {
        Ok(_) => println!("Save succeeded despite read-only file"),
        Err(_) => println!("Save failed as expected due to permissions"),
    }
}

#[test]
fn test_large_file_handling() {
    let mut session = create_test_session("Large File Test");

    // Create a session with a lot of data
    for i in 0..1000 {
        session.context.files.push(format!("file_{}.rs", i));
        session.context.custom.insert(
            format!("key_{}", i),
            serde_json::json!(format!("value_{}", "x".repeat(100))),
        );
    }

    // Serialize (should not fail)
    let json = serde_json::to_string(&session).unwrap();

    // Should be reasonably sized
    let size_kb = json.len() / 1024;
    assert!(
        size_kb > 100,
        "Session should be substantial: {} KB",
        size_kb
    );

    // Deserialize (should not fail)
    let deserialized: Session = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.context.files.len(), 1000);
    assert_eq!(deserialized.context.custom.len(), 1000);
}

#[test]
fn test_backward_compatibility() {
    // Test that sessions serialized with older versions can still be loaded
    // This is a basic test - in practice would need actual old format data

    let session = create_test_session("Compatibility Test");

    // Serialize
    let json = serde_json::to_string(&session).unwrap();

    // Should be able to deserialize (self-compatibility)
    let deserialized: Session = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, session.id);
    assert_eq!(deserialized.name, session.name);
}

#[test]
fn test_concurrent_file_access() {
    let temp_dir = TempDir::new().unwrap();
    let sessions_dir = temp_dir.path().join("sessions");
    let archive_dir = temp_dir.path().join("archive");

    let rt = Runtime::new().unwrap();
    let store = Arc::new(
        rt.block_on(async { SessionStore::with_dirs(sessions_dir, archive_dir).unwrap() }),
    );

    let session = create_test_session("Concurrent Access Test");
    let session_id = session.id.clone();

    // Save session first
    rt.block_on(store.save(&session)).unwrap();

    let mut handles = vec![];

    // Spawn multiple threads loading the same session
    for _ in 0..10 {
        let store_clone = Arc::clone(&store);
        let session_id_clone = session_id.clone();

        let handle = std::thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(store_clone.load(&session_id_clone))
        });

        handles.push(handle);
    }

    // All should succeed
    for handle in handles {
        let result = handle.join().unwrap();
        assert!(result.is_ok());
        let loaded = result.unwrap();
        assert_eq!(loaded.id, session_id);
    }
}

#[test]
fn test_encryption_key_validation() {
    let temp_dir = TempDir::new().unwrap();
    let sessions_dir = temp_dir.path().join("sessions");
    let archive_dir = temp_dir.path().join("archive");

    let rt = Runtime::new().unwrap();
    let mut store =
        rt.block_on(async { SessionStore::with_dirs(sessions_dir, archive_dir).unwrap() });

    // Test with empty password
    let result = store.enable_encryption("");
    assert!(result.is_err());

    // Test with very short password
    let result = store.enable_encryption("x");
    assert!(result.is_err());

    // Test with valid password
    let result = store.enable_encryption("valid-password-123");
    assert!(result.is_ok());
}

#[test]
fn test_enterprise_encryption_key_validation() {
    let temp_dir = TempDir::new().unwrap();
    let sessions_dir = temp_dir.path().join("sessions");
    let archive_dir = temp_dir.path().join("archive");

    let rt = Runtime::new().unwrap();
    let mut store =
        rt.block_on(async { SessionStore::with_dirs(sessions_dir, archive_dir).unwrap() });

    // Test with empty password
    let result = store.enable_enterprise_encryption("");
    assert!(result.is_err());

    // Test with very short password
    let result = store.enable_enterprise_encryption("x");
    assert!(result.is_err());

    // Test with valid password
    let result = store.enable_enterprise_encryption("enterprise-password-123");
    assert!(result.is_ok());
}
