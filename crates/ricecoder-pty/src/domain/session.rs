//! PTY session domain entities

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// PTY session status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    /// Session is running
    Running,
    /// Session has exited
    Exited,
}

/// PTY session information (public API)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    /// Unique session ID
    pub id: String,
    /// Session title/label
    pub title: String,
    /// Command being executed
    pub command: String,
    /// Command arguments
    pub args: Vec<String>,
    /// Working directory
    pub cwd: PathBuf,
    /// Session status
    pub status: SessionStatus,
    /// Process ID (if running)
    pub pid: Option<u32>,
}

/// Internal PTY session state (full entity)
#[derive(Debug)]
pub struct PtySession {
    /// Session info
    pub info: SessionInfo,
    /// Output buffer (when no subscribers)
    pub buffer: String,
}

impl PtySession {
    /// Create new PTY session
    pub fn new(
        id: String,
        title: String,
        command: String,
        args: Vec<String>,
        cwd: PathBuf,
        pid: Option<u32>,
    ) -> Self {
        Self {
            info: SessionInfo {
                id,
                title,
                command,
                args,
                cwd,
                status: SessionStatus::Running,
                pid,
            },
            buffer: String::new(),
        }
    }

    /// Mark session as exited
    pub fn mark_exited(&mut self) {
        self.info.status = SessionStatus::Exited;
        self.info.pid = None;
    }

    /// Update session title
    pub fn update_title(&mut self, title: String) {
        self.info.title = title;
    }

    /// Append to output buffer
    pub fn buffer_output(&mut self, data: &str) {
        self.buffer.push_str(data);
    }

    /// Flush buffer and return contents
    pub fn flush_buffer(&mut self) -> String {
        std::mem::take(&mut self.buffer)
    }

    /// Check if session is running
    pub fn is_running(&self) -> bool {
        self.info.status == SessionStatus::Running
    }
}
