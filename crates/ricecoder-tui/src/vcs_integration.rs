//! VCS integration for TUI status bar
//!
//! This module provides integration between the ricecoder-vcs crate and the TUI status bar,
//! enabling display of comprehensive VCS information including repository status, branch info,
//! and modification indicators.

use crate::status_bar::StatusBarWidget;
use ricecoder_vcs::{GitRepository, RepositoryStatus, Result as VcsResult, Repository};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::watch;
use tokio::time;

/// VCS status information for display in status bar
#[derive(Debug, Clone, PartialEq)]
pub struct VcsStatus {
    /// Current branch name
    pub branch: Option<String>,
    /// Repository status summary (e.g., "1S 2M 1U")
    pub status_summary: Option<String>,
    /// Whether repository has uncommitted changes
    pub has_changes: bool,
    /// Number of staged files
    pub staged_count: usize,
    /// Number of modified files
    pub modified_count: usize,
    /// Number of untracked files
    pub untracked_count: usize,
    /// Whether there are merge conflicts
    pub has_conflicts: bool,
    /// Ahead/behind counts relative to remote
    pub ahead_behind: Option<(usize, usize)>,
}

impl Default for VcsStatus {
    fn default() -> Self {
        Self {
            branch: None,
            status_summary: None,
            has_changes: false,
            staged_count: 0,
            modified_count: 0,
            untracked_count: 0,
            has_conflicts: false,
            ahead_behind: None,
        }
    }
}

impl VcsStatus {
    /// Create VCS status from repository status
    pub fn from_repository_status(status: &RepositoryStatus) -> Self {
        Self {
            branch: Some(status.current_branch.name.clone()),
            status_summary: Some(status.summary()),
            has_changes: status.uncommitted_changes > 0,
            staged_count: status.staged_count,
            modified_count: status.modified_count,
            untracked_count: status.untracked_count,
            has_conflicts: status.has_conflicts,
            ahead_behind: status.ahead_behind,
        }
    }

    /// Check if we're in a git repository
    pub fn is_in_repo(&self) -> bool {
        self.branch.is_some()
    }
}

/// VCS integration manager for status bar
pub struct VcsIntegration {
    /// Current VCS status
    status: Arc<Mutex<VcsStatus>>,
    /// Status change notifier
    status_tx: watch::Sender<VcsStatus>,
    /// Status change receiver
    status_rx: watch::Receiver<VcsStatus>,
    /// Current working directory
    current_dir: PathBuf,
    /// Monitoring task handle
    monitoring_handle: Option<tokio::task::JoinHandle<()>>,
    /// Whether monitoring is active
    monitoring_active: Arc<Mutex<bool>>,
}

impl VcsIntegration {
    /// Create a new VCS integration
    pub fn new() -> Self {
        let status = Arc::new(Mutex::new(VcsStatus::default()));
        let (status_tx, status_rx) = watch::channel(status.lock().unwrap().clone());

        Self {
            status,
            status_tx,
            status_rx,
            current_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            monitoring_handle: None,
            monitoring_active: Arc::new(Mutex::new(false)),
        }
    }

    /// Update the current working directory and refresh VCS status
    pub async fn update_directory(&mut self, dir: PathBuf) -> VcsResult<()> {
        self.current_dir = dir;
        self.refresh_status().await
    }

    /// Refresh VCS status for current directory
    pub async fn refresh_status(&self) -> VcsResult<()> {
        let status = if let Ok(repo) = GitRepository::discover(&self.current_dir) {
            match repo.get_status() {
                Ok(repo_status) => VcsStatus::from_repository_status(&repo_status),
                Err(_) => VcsStatus::default(),
            }
        } else {
            VcsStatus::default()
        };

        // Update stored status
        *self.status.lock().unwrap() = status.clone();

        // Notify listeners
        let _ = self.status_tx.send(status);

        Ok(())
    }

    /// Get current VCS status
    pub fn get_status(&self) -> VcsStatus {
        self.status.lock().unwrap().clone()
    }

    /// Get status change receiver for reactive updates
    pub fn status_receiver(&self) -> watch::Receiver<VcsStatus> {
        self.status_rx.clone()
    }

    /// Apply VCS status to status bar widget
    pub fn apply_to_status_bar(&self, mut status_bar: StatusBarWidget) -> StatusBarWidget {
        let vcs_status = self.get_status();

        // Set basic branch info
        status_bar.git_branch = vcs_status.branch;

        // Add VCS status indicators if there are changes
        if vcs_status.is_in_repo() {
            // Add status summary to recording status (reuse existing field)
            if vcs_status.has_changes {
                status_bar.recording_status = vcs_status.status_summary;
            }

            // Add ahead/behind info to search status
            if let Some((ahead, behind)) = vcs_status.ahead_behind {
                if ahead > 0 || behind > 0 {
                    status_bar.search_status = Some(format!("↑{} ↓{}", ahead, behind));
                }
            }

            // Add conflict indicator to selection status
            if vcs_status.has_conflicts {
                status_bar.selection_status = Some("CONFLICTS".to_string());
            }
        }

        status_bar
    }

    /// Get VCS status summary for display
    pub fn get_status_summary(&self) -> Option<String> {
        let status = self.get_status();
        if status.is_in_repo() && status.has_changes {
            status.status_summary
        } else {
            None
        }
    }

    /// Get branch display string with status indicators
    pub fn get_branch_display(&self) -> Option<String> {
        let status = self.get_status();
        status.branch.map(|branch| {
            if status.has_changes {
                format!("{}*", branch)
            } else {
                branch
            }
        })
    }

    /// Check if repository has uncommitted changes
    pub fn has_uncommitted_changes(&self) -> bool {
        self.get_status().has_changes
    }

    /// Get counts of different file states
    pub fn get_file_counts(&self) -> (usize, usize, usize) {
        let status = self.get_status();
        (status.staged_count, status.modified_count, status.untracked_count)
    }

    /// Start monitoring VCS status changes
    pub fn start_monitoring(&mut self, interval: Duration) {
        let status = Arc::clone(&self.status);
        let status_tx = self.status_tx.clone();
        let current_dir = self.current_dir.clone();
        let monitoring_active = Arc::clone(&self.monitoring_active);

        *monitoring_active.lock().unwrap() = true;

        let handle = tokio::spawn(async move {
            let mut interval = time::interval(interval);

            while *monitoring_active.lock().unwrap() {
                interval.tick().await;

                // Check if still active before doing work
                if !*monitoring_active.lock().unwrap() {
                    break;
                }

                let new_status = if let Ok(repo) = GitRepository::discover(&current_dir) {
                    match repo.get_status() {
                        Ok(repo_status) => VcsStatus::from_repository_status(&repo_status),
                        Err(_) => VcsStatus::default(),
                    }
                } else {
                    VcsStatus::default()
                };

                // Update status if it changed
                let mut current_status = status.lock().unwrap();
                if *current_status != new_status {
                    *current_status = new_status.clone();
                    drop(current_status); // Release lock before sending
                    let _ = status_tx.send(new_status);
                }
            }
        });

        self.monitoring_handle = Some(handle);
    }

    /// Stop monitoring VCS status changes
    pub async fn stop_monitoring(&mut self) {
        if let Some(handle) = self.monitoring_handle.take() {
            *self.monitoring_active.lock().unwrap() = false;
            let _ = handle.await;
        }
    }

    /// Check if monitoring is active
    pub fn is_monitoring(&self) -> bool {
        *self.monitoring_active.lock().unwrap()
    }

    /// Force a status refresh (useful for manual updates)
    pub async fn force_refresh(&self) -> VcsResult<()> {
        self.refresh_status().await
    }
}

impl Default for VcsIntegration {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension trait for StatusBarWidget to add VCS integration
pub trait StatusBarVcsExt {
    /// Set VCS integration
    fn with_vcs_integration(self, vcs: &VcsIntegration) -> Self;
}

impl StatusBarVcsExt for StatusBarWidget {
    fn with_vcs_integration(self, vcs: &VcsIntegration) -> Self {
        vcs.apply_to_status_bar(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_vcs_status_creation() {
        let status = VcsStatus {
            branch: Some("main".to_string()),
            status_summary: Some("1M 2U".to_string()),
            has_changes: true,
            staged_count: 0,
            modified_count: 1,
            untracked_count: 2,
            has_conflicts: false,
            ahead_behind: Some((1, 0)),
        };

        assert_eq!(status.branch, Some("main".to_string()));
        assert!(status.has_changes);
        assert!(status.is_in_repo());
    }

    #[test]
    fn test_vcs_status_default() {
        let status = VcsStatus::default();
        assert!(!status.is_in_repo());
        assert!(!status.has_changes);
    }

    #[test]
    fn test_vcs_integration_creation() {
        let integration = VcsIntegration::new();
        let status = integration.get_status();
        assert!(!status.is_in_repo());
    }

    #[test]
    fn test_branch_display_with_changes() {
        let mut status = VcsStatus::default();
        status.branch = Some("main".to_string());
        status.has_changes = true;

        let integration = VcsIntegration::new();
        *integration.status.lock().unwrap() = status;

        assert_eq!(integration.get_branch_display(), Some("main*".to_string()));
    }

    #[test]
    fn test_file_counts() {
        let mut status = VcsStatus::default();
        status.staged_count = 1;
        status.modified_count = 2;
        status.untracked_count = 3;

        let integration = VcsIntegration::new();
        *integration.status.lock().unwrap() = status;

        assert_eq!(integration.get_file_counts(), (1, 2, 3));
    }

    #[tokio::test]
    async fn test_directory_update() {
        let mut integration = VcsIntegration::new();
        let new_dir = PathBuf::from("/tmp/test");

        // This should not panic even if directory doesn't exist
        let result = integration.update_directory(new_dir.clone()).await;
        assert!(result.is_ok());
        assert_eq!(integration.current_dir, new_dir);
    }
}