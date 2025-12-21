//! Cross-crate integration tests for database integration with session persistence and encryption
//!
//! Tests the integration between:
//! - ricecoder-sessions (session management)
//! - ricecoder-storage (persistence layer)
//! - ricecoder-security (encryption)
//! - ricecoder-domain (session entities)

use ricecoder_domain::entities::Session;
use ricecoder_security::encryption::KeyManager;
use ricecoder_sessions::{SessionManager, SessionStatus, SessionStore};
use ricecoder_storage::{
    session::SessionManager as StorageSessionManager, StorageManager, StorageMode,
};
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::tempdir;

/// Mock storage manager for testing
struct MockStorageManager {
    global_path: PathBuf,
    project_path: Option<PathBuf>,
}

impl StorageManager for MockStorageManager {
    fn global_path(&self) -> &PathBuf {
        &self.global_path
    }

    fn project_path(&self) -> Option<&PathBuf> {
        self.project_path.as_ref()
    }

    fn mode(&self) -> StorageMode {
        StorageMode::Merged
    }

    fn global_resource_path(
        &self,
        _resource_type: ricecoder_storage::types::ResourceType,
    ) -> PathBuf {
        self.global_path.join("resources")
    }

    fn project_resource_path(
        &self,
        _resource_type: ricecoder_storage::types::ResourceType,
    ) -> Option<PathBuf> {
        self.project_path.as_ref().map(|p| p.join("resources"))
    }

    fn is_first_run(&self) -> bool {
        false
    }
}

#[tokio::test]
async fn test_session_persistence_with_encryption() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let storage_path = temp_dir.path().to_path_buf();

    // Initialize components
    let key_manager = KeyManager::new()?;
    let storage = Arc::new(MockStorageManager {
        global_path: storage_path.clone(),
        project_path: Some(storage_path.clone()),
    });

    let session_store = SessionStore::new(storage.clone(), Some(key_manager.clone()));
    let session_manager = SessionManager::new(session_store);

    // Create and configure a session
    let session_id = session_manager.create_session("openai", "gpt-4").await?;
    session_manager
        .set_session_name(session_id.clone(), "encrypted-test-session")
        .await?;

    // Add some messages to the session
    let message1 = ricecoder_sessions::Message {
        role: ricecoder_sessions::MessageRole::User,
        content: "Hello, world!".to_string(),
        metadata: Default::default(),
    };
    let message2 = ricecoder_sessions::Message {
        role: ricecoder_sessions::MessageRole::Assistant,
        content: "Hello! How can I help you?".to_string(),
        metadata: Default::default(),
    };

    session_manager
        .add_message(session_id.clone(), message1)
        .await?;
    session_manager
        .add_message(session_id.clone(), message2)
        .await?;

    // Persist the session
    session_manager.save_session(&session_id).await?;

    // Verify session was encrypted and stored
    let session_file = storage_path
        .join("sessions")
        .join(format!("{}.enc", session_id));
    assert!(session_file.exists(), "Encrypted session file should exist");

    // Load the session back
    let loaded_session = session_manager.load_session(&session_id).await?;
    assert!(loaded_session.is_some(), "Session should be loaded");

    let loaded = loaded_session.unwrap();
    assert_eq!(loaded.name, Some("encrypted-test-session".to_string()));
    assert_eq!(loaded.messages.len(), 2);

    Ok(())
}

#[tokio::test]
async fn test_session_store_encryption_integration() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let storage_path = temp_dir.path().to_path_buf();

    // Initialize encrypted session store
    let key_manager = KeyManager::new()?;
    let storage = Arc::new(MockStorageManager {
        global_path: storage_path.clone(),
        project_path: Some(storage_path.clone()),
    });

    let session_store = SessionStore::new(storage.clone(), Some(key_manager.clone()));

    // Create a session with sensitive data
    let mut session = ricecoder_sessions::Session::new("openai", "gpt-4");
    session.name = Some("sensitive-session".to_string());
    session.metadata.insert(
        "api_key".to_string(),
        serde_json::Value::String("sk-1234567890abcdef".to_string()),
    );

    // Store encrypted session
    session_store.save(&session).await?;

    // Verify file exists and is encrypted
    let session_file = storage_path
        .join("sessions")
        .join(format!("{}.enc", session.id));
    assert!(session_file.exists());

    let encrypted_content = std::fs::read(&session_file)?;
    // Verify it's not plaintext (basic check)
    let content_str = String::from_utf8_lossy(&encrypted_content);
    assert!(
        !content_str.contains("sk-1234567890abcdef"),
        "API key should be encrypted"
    );

    // Load and verify decryption
    let loaded_session = session_store.load(&session.id).await?;
    assert!(loaded_session.is_some());

    let loaded = loaded_session.unwrap();
    assert_eq!(loaded.name, session.name);
    assert_eq!(
        loaded.metadata.get("api_key"),
        session.metadata.get("api_key")
    );

    Ok(())
}

#[tokio::test]
async fn test_session_manager_storage_integration() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let storage_path = temp_dir.path().to_path_buf();

    // Initialize storage session manager
    let storage_session_manager = StorageSessionManager::new(storage_path.clone());

    // Create and save a session
    let session_data = ricecoder_storage::session::SessionData {
        id: "test-session-123".to_string(),
        name: Some("storage-test".to_string()),
        provider: "openai".to_string(),
        model: "gpt-4".to_string(),
        status: ricecoder_storage::session::SessionState::Active,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        metadata: serde_json::json!({"test": "data"}),
    };

    storage_session_manager.save_session(&session_data).await?;

    // Load and verify
    let loaded = storage_session_manager
        .load_session("test-session-123")
        .await?;
    assert!(loaded.is_some());

    let loaded_session = loaded.unwrap();
    assert_eq!(loaded_session.id, session_data.id);
    assert_eq!(loaded_session.name, session_data.name);
    assert_eq!(loaded_session.provider, session_data.provider);

    // Test session listing
    let sessions = storage_session_manager.list_sessions().await?;
    assert!(!sessions.is_empty());
    assert!(sessions.contains(&"test-session-123".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_session_domain_storage_bridge() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let storage_path = temp_dir.path().to_path_buf();

    // Create domain session
    let mut domain_session = Session::new("anthropic".to_string(), "claude-3".to_string());
    domain_session.set_name("domain-bridge-test".to_string());
    domain_session.metadata.insert(
        "environment".to_string(),
        serde_json::Value::String("testing".to_string()),
    );

    // Initialize storage
    let storage = Arc::new(MockStorageManager {
        global_path: storage_path.clone(),
        project_path: Some(storage_path.clone()),
    });

    let storage_session_manager = StorageSessionManager::new(storage_path.clone());

    // Convert domain session to storage session
    let storage_session = ricecoder_storage::session::SessionData {
        id: domain_session.id.to_string(),
        name: domain_session.name.clone(),
        provider: domain_session.provider_id.clone(),
        model: domain_session.model_id.clone(),
        status: match domain_session.status {
            ricecoder_domain::entities::SessionStatus::Active => {
                ricecoder_storage::session::SessionState::Active
            }
            ricecoder_domain::entities::SessionStatus::Paused => {
                ricecoder_storage::session::SessionState::Paused
            }
            ricecoder_domain::entities::SessionStatus::Ended => {
                ricecoder_storage::session::SessionState::Ended
            }
        },
        created_at: domain_session.created_at,
        updated_at: domain_session.updated_at,
        metadata: serde_json::json!(domain_session.metadata),
    };

    // Save to storage
    storage_session_manager
        .save_session(&storage_session)
        .await?;

    // Load back and convert to domain session
    let loaded_storage = storage_session_manager
        .load_session(&domain_session.id.to_string())
        .await?;
    assert!(loaded_storage.is_some());

    let loaded = loaded_storage.unwrap();
    let mut restored_domain = Session::new(loaded.provider.clone(), loaded.model.clone());
    restored_domain.id = ricecoder_domain::value_objects::SessionId::from_string(loaded.id.clone());
    restored_domain.name = loaded.name.clone();
    restored_domain.status = match loaded.status {
        ricecoder_storage::session::SessionState::Active => {
            ricecoder_domain::entities::SessionStatus::Active
        }
        ricecoder_storage::session::SessionState::Paused => {
            ricecoder_domain::entities::SessionStatus::Paused
        }
        ricecoder_storage::session::SessionState::Ended => {
            ricecoder_domain::entities::SessionStatus::Ended
        }
    };
    restored_domain.created_at = loaded.created_at;
    restored_domain.updated_at = loaded.updated_at;
    restored_domain.metadata = serde_json::from_value(loaded.metadata).unwrap_or_default();

    // Verify round-trip integrity
    assert_eq!(restored_domain.id, domain_session.id);
    assert_eq!(restored_domain.name, domain_session.name);
    assert_eq!(restored_domain.provider_id, domain_session.provider_id);
    assert_eq!(restored_domain.model_id, domain_session.model_id);
    assert_eq!(restored_domain.status, domain_session.status);

    Ok(())
}

#[tokio::test]
async fn test_encrypted_session_backup_recovery() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let storage_path = temp_dir.path().to_path_buf();
    let backup_path = temp_dir.path().join("backup");

    // Initialize components
    let key_manager = KeyManager::new()?;
    let storage = Arc::new(MockStorageManager {
        global_path: storage_path.clone(),
        project_path: Some(storage_path.clone()),
    });

    let session_store = SessionStore::new(storage.clone(), Some(key_manager.clone()));

    // Create multiple sessions with sensitive data
    let sessions = vec![
        (
            "session1",
            "openai",
            "gpt-4",
            "Production API key: sk-prod-123",
        ),
        (
            "session2",
            "anthropic",
            "claude-3",
            "Dev API key: sk-dev-456",
        ),
        ("session3", "google", "gemini", "Test API key: sk-test-789"),
    ];

    for (id, provider, model, sensitive_data) in &sessions {
        let mut session = ricecoder_sessions::Session::new(*provider, *model);
        session.id = id.to_string();
        session.name = Some(format!("{}-session", id));
        session.metadata.insert(
            "api_key".to_string(),
            serde_json::Value::String(sensitive_data.to_string()),
        );
        session_store.save(&session).await?;
    }

    // Perform encrypted backup
    session_store.backup_to(&backup_path).await?;

    // Verify backup files exist and are encrypted
    for (id, _, _, sensitive_data) in &sessions {
        let backup_file = backup_path.join(format!("{}.enc", id));
        assert!(backup_file.exists(), "Backup file should exist for {}", id);

        let encrypted_content = std::fs::read(&backup_file)?;
        let content_str = String::from_utf8_lossy(&encrypted_content);
        assert!(
            !content_str.contains(sensitive_data),
            "Sensitive data should be encrypted in backup"
        );
    }

    // Simulate disaster - delete original sessions
    let sessions_dir = storage_path.join("sessions");
    if sessions_dir.exists() {
        std::fs::remove_dir_all(&sessions_dir)?;
    }

    // Restore from encrypted backup
    session_store.restore_from(&backup_path).await?;

    // Verify all sessions restored correctly
    for (id, provider, model, sensitive_data) in &sessions {
        let restored = session_store.load(id).await?;
        assert!(restored.is_some(), "Session {} should be restored", id);

        let session = restored.unwrap();
        assert_eq!(session.provider, *provider);
        assert_eq!(session.model, *model);
        assert_eq!(
            session.metadata.get("api_key").and_then(|v| v.as_str()),
            Some(sensitive_data)
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_session_garbage_collection_with_encryption() -> Result<(), Box<dyn std::error::Error>>
{
    let temp_dir = tempdir()?;
    let storage_path = temp_dir.path().to_path_buf();

    // Initialize components
    let key_manager = KeyManager::new()?;
    let storage = Arc::new(MockStorageManager {
        global_path: storage_path.clone(),
        project_path: Some(storage_path.clone()),
    });

    let session_store = SessionStore::new(storage.clone(), Some(key_manager.clone()));

    // Create sessions with different ages
    let now = chrono::Utc::now();
    let old_date = now - chrono::Duration::days(90); // Older than retention period
    let recent_date = now - chrono::Duration::days(30); // Within retention period

    // Create old session
    let mut old_session = ricecoder_sessions::Session::new("openai", "gpt-4");
    old_session.id = "old-session".to_string();
    old_session.created_at = old_date;
    old_session.updated_at = old_date;
    session_store.save(&old_session).await?;

    // Create recent session
    let mut recent_session = ricecoder_sessions::Session::new("anthropic", "claude-3");
    recent_session.id = "recent-session".to_string();
    recent_session.created_at = recent_date;
    recent_session.updated_at = recent_date;
    session_store.save(&recent_session).await?;

    // Run garbage collection (simulate 60-day retention)
    let retention_days = 60;
    let deleted_count = session_store.garbage_collect(retention_days).await?;

    assert_eq!(deleted_count, 1, "Should delete 1 old session");

    // Verify old session is gone
    let old_loaded = session_store.load("old-session").await?;
    assert!(old_loaded.is_none(), "Old session should be deleted");

    // Verify recent session remains
    let recent_loaded = session_store.load("recent-session").await?;
    assert!(recent_loaded.is_some(), "Recent session should remain");

    Ok(())
}
