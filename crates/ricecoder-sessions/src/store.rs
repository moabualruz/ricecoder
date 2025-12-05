//! Session persistence to disk

use crate::error::{SessionError, SessionResult};
use crate::models::Session;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, error, info};

/// Manages session persistence to disk
#[derive(Debug, Clone)]
pub struct SessionStore {
    /// Base directory for storing sessions
    sessions_dir: PathBuf,
    /// Archive directory for deleted sessions
    archive_dir: PathBuf,
}

impl SessionStore {
    /// Create a new session store with default directories
    pub fn new() -> SessionResult<Self> {
        let sessions_dir = Self::get_sessions_dir()?;
        let archive_dir = Self::get_archive_dir()?;

        // Ensure directories exist
        fs::create_dir_all(&sessions_dir)?;
        fs::create_dir_all(&archive_dir)?;

        debug!(
            "SessionStore initialized with sessions_dir: {:?}",
            sessions_dir
        );

        Ok(Self {
            sessions_dir,
            archive_dir,
        })
    }

    /// Create a session store with custom directories (for testing)
    pub fn with_dirs(sessions_dir: PathBuf, archive_dir: PathBuf) -> SessionResult<Self> {
        fs::create_dir_all(&sessions_dir)?;
        fs::create_dir_all(&archive_dir)?;

        Ok(Self {
            sessions_dir,
            archive_dir,
        })
    }

    /// Get the default sessions directory (~/.ricecoder/sessions/)
    fn get_sessions_dir() -> SessionResult<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| {
            SessionError::ConfigError("Could not determine home directory".to_string())
        })?;
        Ok(home.join(".ricecoder").join("sessions"))
    }

    /// Get the default archive directory (~/.ricecoder/sessions/archive/)
    fn get_archive_dir() -> SessionResult<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| {
            SessionError::ConfigError("Could not determine home directory".to_string())
        })?;
        Ok(home.join(".ricecoder").join("sessions").join("archive"))
    }

    /// Get the path for a session file
    fn session_path(&self, session_id: &str) -> PathBuf {
        self.sessions_dir.join(format!("{}.json", session_id))
    }

    /// Get the path for an archived session file
    fn archive_path(&self, session_id: &str) -> PathBuf {
        self.archive_dir.join(format!("{}.json", session_id))
    }

    /// Save a session to disk
    pub async fn save(&self, session: &Session) -> SessionResult<()> {
        let path = self.session_path(&session.id);

        // Serialize session to JSON
        let json_data = serde_json::to_string_pretty(session)?;

        // Write to file
        fs::write(&path, json_data)?;

        info!("Session saved: {} at {:?}", session.id, path);

        Ok(())
    }

    /// Load a session from disk
    pub async fn load(&self, session_id: &str) -> SessionResult<Session> {
        let path = self.session_path(session_id);

        if !path.exists() {
            return Err(SessionError::NotFound(format!(
                "Session file not found: {}",
                session_id
            )));
        }

        // Read file
        let json_data = fs::read_to_string(&path)?;

        // Deserialize from JSON
        let session: Session = serde_json::from_str(&json_data)?;

        debug!("Session loaded: {} from {:?}", session_id, path);

        Ok(session)
    }

    /// List all persisted sessions
    pub async fn list(&self) -> SessionResult<Vec<Session>> {
        let mut sessions = Vec::new();

        // Read all JSON files in sessions directory
        let entries = fs::read_dir(&self.sessions_dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            // Skip directories and non-JSON files
            if !path.is_file() {
                continue;
            }

            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }

            // Try to load the session
            match fs::read_to_string(&path) {
                Ok(json_data) => match serde_json::from_str::<Session>(&json_data) {
                    Ok(session) => sessions.push(session),
                    Err(e) => {
                        error!("Failed to deserialize session from {:?}: {}", path, e);
                    }
                },
                Err(e) => {
                    error!("Failed to read session file {:?}: {}", path, e);
                }
            }
        }

        debug!("Listed {} sessions", sessions.len());

        Ok(sessions)
    }

    /// Export a session to a user-specified file
    pub async fn export(&self, session_id: &str, export_path: &Path) -> SessionResult<()> {
        // Load the session
        let session = self.load(session_id).await?;

        // Serialize to JSON
        let json_data = serde_json::to_string_pretty(&session)?;

        // Write to export path
        fs::write(export_path, json_data)?;

        info!("Session exported: {} to {:?}", session_id, export_path);

        Ok(())
    }

    /// Delete a session and archive it
    pub async fn delete(&self, session_id: &str) -> SessionResult<()> {
        let session_path = self.session_path(session_id);
        let archive_path = self.archive_path(session_id);

        if !session_path.exists() {
            return Err(SessionError::NotFound(format!(
                "Session not found: {}",
                session_id
            )));
        }

        // Read the session file
        let json_data = fs::read_to_string(&session_path)?;

        // Write to archive
        fs::write(&archive_path, json_data)?;

        // Delete the original
        fs::remove_file(&session_path)?;

        info!("Session deleted and archived: {}", session_id);

        Ok(())
    }

    /// Check if a session exists
    pub fn exists(&self, session_id: &str) -> bool {
        self.session_path(session_id).exists()
    }

    /// Get the sessions directory path
    pub fn sessions_dir(&self) -> &Path {
        &self.sessions_dir
    }

    /// Get the archive directory path
    pub fn archive_dir(&self) -> &Path {
        &self.archive_dir
    }
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new().expect("Failed to create default SessionStore")
    }
}
