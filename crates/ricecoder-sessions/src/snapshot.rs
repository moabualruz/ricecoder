//! Git-based snapshot system for session workspace state tracking
//!
//! This module implements OpenCode-compatible snapshot functionality using git tree hashes
//! for content-addressed storage and efficient delta computation.

use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, error, info, warn};

use crate::error::SessionError;

/// Git-based snapshot manager for session workspace state
#[derive(Debug, Clone)]
pub struct SnapshotManager {
    /// Path to the git directory for snapshots
    git_dir: PathBuf,
    /// Working tree path
    work_tree: PathBuf,
    /// Whether snapshots are enabled
    enabled: bool,
}

/// Snapshot patch containing changed files
#[derive(Debug, Clone)]
pub struct SnapshotPatch {
    /// Snapshot hash
    pub hash: String,
    /// List of changed file paths
    pub files: Vec<PathBuf>,
}

/// Full diff with before/after content
#[derive(Debug, Clone)]
pub struct FileDiff {
    /// File path
    pub file: PathBuf,
    /// Content before change
    pub before: String,
    /// Content after change
    pub after: String,
    /// Number of additions
    pub additions: usize,
    /// Number of deletions
    pub deletions: usize,
}

impl SnapshotManager {
    /// Creates a new snapshot manager
    ///
    /// # Arguments
    ///
    /// * `data_dir` - Global data directory for snapshots
    /// * `project_id` - Unique project identifier
    /// * `work_tree` - Path to the working tree
    /// * `enabled` - Whether snapshots are enabled
    pub fn new(
        data_dir: impl AsRef<Path>,
        project_id: &str,
        work_tree: impl AsRef<Path>,
        enabled: bool,
    ) -> Self {
        let git_dir = data_dir.as_ref().join("snapshot").join(project_id);
        Self {
            git_dir,
            work_tree: work_tree.as_ref().to_path_buf(),
            enabled,
        }
    }

    /// Create a disabled snapshot manager (no-op)
    pub fn disabled() -> Self {
        Self {
            git_dir: PathBuf::new(),
            work_tree: PathBuf::new(),
            enabled: false,
        }
    }

    /// Tracks current workspace state and returns snapshot hash
    ///
    /// # Returns
    ///
    /// Git tree hash representing the snapshot, or None if snapshots disabled
    pub async fn track(&self) -> Result<Option<String>, SessionError> {
        if !self.enabled {
            debug!("snapshots disabled, skipping track");
            return Ok(None);
        }

        // Initialize git repo if needed
        if !self.git_dir.exists() {
            self.init_git_repo().await?;
        }

        // Stage all files
        self.git_add_all().await?;

        // Write tree and get hash
        let hash = self.git_write_tree().await?;

        info!(hash = %hash, "workspace snapshot tracked");
        Ok(Some(hash))
    }

    /// Computes patch (delta) between snapshot and current state
    ///
    /// # Arguments
    ///
    /// * `hash` - Snapshot hash to compare against
    ///
    /// # Returns
    ///
    /// Patch containing hash and list of changed files
    pub async fn patch(&self, hash: &str) -> Result<SnapshotPatch, SessionError> {
        if !self.enabled {
            return Ok(SnapshotPatch {
                hash: hash.to_string(),
                files: Vec::new(),
            });
        }

        // Stage current state
        self.git_add_all().await?;

        // Get changed files
        let files = self.git_diff_files(hash).await?;

        Ok(SnapshotPatch {
            hash: hash.to_string(),
            files,
        })
    }

    /// Restores workspace to snapshot state
    ///
    /// # Arguments
    ///
    /// * `hash` - Snapshot hash to restore
    pub async fn restore(&self, hash: &str) -> Result<(), SessionError> {
        if !self.enabled {
            warn!("snapshots disabled, cannot restore");
            return Err(SessionError::SnapshotDisabled);
        }

        info!(hash = %hash, "restoring workspace snapshot");

        // read-tree + checkout-index
        let output = Command::new("git")
            .args(&[
                "--git-dir",
                &self.git_dir.display().to_string(),
                "--work-tree",
                &self.work_tree.display().to_string(),
                "read-tree",
                hash,
            ])
            .output()
            .await
            .map_err(|e| {
                SessionError::SnapshotFailed(format!("failed to execute read-tree: {}", e))
            })?;

        if !output.status.success() {
            error!(
                stderr = %String::from_utf8_lossy(&output.stderr),
                "read-tree failed"
            );
            return Err(SessionError::SnapshotFailed(format!(
                "read-tree failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let output = Command::new("git")
            .args(&[
                "--git-dir",
                &self.git_dir.display().to_string(),
                "--work-tree",
                &self.work_tree.display().to_string(),
                "checkout-index",
                "-a",
                "-f",
            ])
            .output()
            .await
            .map_err(|e| {
                SessionError::SnapshotFailed(format!("failed to execute checkout-index: {}", e))
            })?;

        if !output.status.success() {
            error!(
                stderr = %String::from_utf8_lossy(&output.stderr),
                "checkout-index failed"
            );
            return Err(SessionError::SnapshotFailed(format!(
                "checkout-index failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        info!(hash = %hash, "workspace restored");
        Ok(())
    }

    /// Reverts files to their state in the given snapshots
    ///
    /// # Arguments
    ///
    /// * `patches` - Patches containing files to revert
    pub async fn revert(&self, patches: &[SnapshotPatch]) -> Result<(), SessionError> {
        if !self.enabled {
            warn!("snapshots disabled, cannot revert");
            return Err(SessionError::SnapshotDisabled);
        }

        let mut reverted_files = std::collections::HashSet::new();

        for patch in patches {
            for file in &patch.files {
                if reverted_files.contains(file) {
                    continue;
                }

                info!(file = %file.display(), hash = %patch.hash, "reverting file");

                let output = Command::new("git")
                    .args(&[
                        "--git-dir",
                        &self.git_dir.display().to_string(),
                        "--work-tree",
                        &self.work_tree.display().to_string(),
                        "checkout",
                        &patch.hash,
                        "--",
                        &file.display().to_string(),
                    ])
                    .output()
                    .await
                    .map_err(|e| {
                        SessionError::SnapshotFailed(format!("failed to checkout file: {}", e))
                    })?;

                if !output.status.success() {
                    // Check if file existed in snapshot
                    let relative_path = file
                        .strip_prefix(&self.work_tree)
                        .unwrap_or(file.as_path());
                    let ls_tree_output = Command::new("git")
                        .args(&[
                            "--git-dir",
                            &self.git_dir.display().to_string(),
                            "--work-tree",
                            &self.work_tree.display().to_string(),
                            "ls-tree",
                            &patch.hash,
                            "--",
                            &relative_path.display().to_string(),
                        ])
                        .output()
                        .await
                        .map_err(|e| {
                            SessionError::SnapshotFailed(format!("failed to ls-tree: {}", e))
                        })?;

                    if ls_tree_output.status.success()
                        && !ls_tree_output.stdout.is_empty()
                    {
                        info!(file = %file.display(), "file existed in snapshot but checkout failed, keeping");
                    } else {
                        info!(file = %file.display(), "file did not exist in snapshot, deleting");
                        let _ = tokio::fs::remove_file(file).await;
                    }
                }

                reverted_files.insert(file.clone());
            }
        }

        Ok(())
    }

    /// Gets git diff text between snapshot and current state
    ///
    /// # Arguments
    ///
    /// * `hash` - Snapshot hash to diff against
    ///
    /// # Returns
    ///
    /// Unified diff text
    pub async fn diff(&self, hash: &str) -> Result<String, SessionError> {
        if !self.enabled {
            return Ok(String::new());
        }

        // Stage current state
        self.git_add_all().await?;

        let output = Command::new("git")
            .args(&[
                "-c",
                "core.autocrlf=false",
                "--git-dir",
                &self.git_dir.display().to_string(),
                "--work-tree",
                &self.work_tree.display().to_string(),
                "diff",
                "--no-ext-diff",
                hash,
                "--",
                ".",
            ])
            .current_dir(&self.work_tree)
            .output()
            .await
            .map_err(|e| SessionError::SnapshotFailed(format!("failed to execute diff: {}", e)))?;

        if !output.status.success() {
            warn!(
                stderr = %String::from_utf8_lossy(&output.stderr),
                "diff failed"
            );
            return Ok(String::new());
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Gets full diff with before/after content for each file
    ///
    /// # Arguments
    ///
    /// * `from_hash` - Starting snapshot hash
    /// * `to_hash` - Ending snapshot hash
    ///
    /// # Returns
    ///
    /// Vector of file diffs with content and line counts
    pub async fn diff_full(
        &self,
        from_hash: &str,
        to_hash: &str,
    ) -> Result<Vec<FileDiff>, SessionError> {
        if !self.enabled {
            return Ok(Vec::new());
        }

        let mut result = Vec::new();

        // Get numstat output
        let output = Command::new("git")
            .args(&[
                "-c",
                "core.autocrlf=false",
                "--git-dir",
                &self.git_dir.display().to_string(),
                "--work-tree",
                &self.work_tree.display().to_string(),
                "diff",
                "--no-ext-diff",
                "--no-renames",
                "--numstat",
                from_hash,
                to_hash,
                "--",
                ".",
            ])
            .current_dir(&self.work_tree)
            .output()
            .await
            .map_err(|e| {
                SessionError::SnapshotFailed(format!("failed to execute diff numstat: {}", e))
            })?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let numstat = String::from_utf8_lossy(&output.stdout);
        for line in numstat.lines() {
            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() < 3 {
                continue;
            }

            let additions = parts[0];
            let deletions = parts[1];
            let file = PathBuf::from(parts[2]);

            // Check if binary file
            let is_binary = additions == "-" && deletions == "-";

            let before = if is_binary {
                String::new()
            } else {
                self.git_show_file(from_hash, &file).await.unwrap_or_default()
            };

            let after = if is_binary {
                String::new()
            } else {
                self.git_show_file(to_hash, &file).await.unwrap_or_default()
            };

            result.push(FileDiff {
                file,
                before,
                after,
                additions: additions.parse().unwrap_or(0),
                deletions: deletions.parse().unwrap_or(0),
            });
        }

        Ok(result)
    }

    // --- Private helper methods ---

    async fn init_git_repo(&self) -> Result<(), SessionError> {
        tokio::fs::create_dir_all(&self.git_dir)
            .await
            .map_err(|e| {
                SessionError::SnapshotFailed(format!("failed to create git dir: {}", e))
            })?;

        let output = Command::new("git")
            .arg("init")
            .env("GIT_DIR", &self.git_dir)
            .env("GIT_WORK_TREE", &self.work_tree)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .await
            .map_err(|e| SessionError::SnapshotFailed(format!("failed to git init: {}", e)))?;

        if !output.status.success() {
            return Err(SessionError::SnapshotFailed("git init failed".to_string()));
        }

        // Configure git to not convert line endings on Windows
        let _ = Command::new("git")
            .args(&[
                "--git-dir",
                &self.git_dir.display().to_string(),
                "config",
                "core.autocrlf",
                "false",
            ])
            .output()
            .await;

        info!("snapshot git repository initialized");
        Ok(())
    }

    async fn git_add_all(&self) -> Result<(), SessionError> {
        let output = Command::new("git")
            .args(&[
                "--git-dir",
                &self.git_dir.display().to_string(),
                "--work-tree",
                &self.work_tree.display().to_string(),
                "add",
                ".",
            ])
            .current_dir(&self.work_tree)
            .stdout(Stdio::null())
            .output()
            .await
            .map_err(|e| SessionError::SnapshotFailed(format!("failed to git add: {}", e)))?;

        if !output.status.success() {
            warn!(
                stderr = %String::from_utf8_lossy(&output.stderr),
                "git add failed (non-critical)"
            );
        }

        Ok(())
    }

    async fn git_write_tree(&self) -> Result<String, SessionError> {
        let output = Command::new("git")
            .args(&[
                "--git-dir",
                &self.git_dir.display().to_string(),
                "--work-tree",
                &self.work_tree.display().to_string(),
                "write-tree",
            ])
            .current_dir(&self.work_tree)
            .output()
            .await
            .map_err(|e| {
                SessionError::SnapshotFailed(format!("failed to execute write-tree: {}", e))
            })?;

        if !output.status.success() {
            error!(
                stderr = %String::from_utf8_lossy(&output.stderr),
                "write-tree failed"
            );
            return Err(SessionError::SnapshotFailed(
                "write-tree failed".to_string(),
            ));
        }

        let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(hash)
    }

    async fn git_diff_files(&self, hash: &str) -> Result<Vec<PathBuf>, SessionError> {
        let output = Command::new("git")
            .args(&[
                "-c",
                "core.autocrlf=false",
                "--git-dir",
                &self.git_dir.display().to_string(),
                "--work-tree",
                &self.work_tree.display().to_string(),
                "diff",
                "--no-ext-diff",
                "--name-only",
                hash,
                "--",
                ".",
            ])
            .current_dir(&self.work_tree)
            .output()
            .await
            .map_err(|e| {
                SessionError::SnapshotFailed(format!("failed to execute diff: {}", e))
            })?;

        if !output.status.success() {
            warn!(
                stderr = %String::from_utf8_lossy(&output.stderr),
                "diff failed"
            );
            return Ok(Vec::new());
        }

        let files_text = String::from_utf8_lossy(&output.stdout);
        let files = files_text
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| self.work_tree.join(line.trim()))
            .collect();

        Ok(files)
    }

    async fn git_show_file(&self, hash: &str, file: &Path) -> Result<String, SessionError> {
        let output = Command::new("git")
            .args(&[
                "-c",
                "core.autocrlf=false",
                "--git-dir",
                &self.git_dir.display().to_string(),
                "--work-tree",
                &self.work_tree.display().to_string(),
                "show",
                &format!("{}:{}", hash, file.display()),
            ])
            .output()
            .await
            .map_err(|e| SessionError::SnapshotFailed(format!("failed to show file: {}", e)))?;

        if !output.status.success() {
            return Ok(String::new());
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_track_snapshot() {
        let temp = TempDir::new().unwrap();
        let data_dir = temp.path().join("data");
        let work_tree = temp.path().join("project");
        tokio::fs::create_dir_all(&work_tree).await.unwrap();

        // Create test file
        tokio::fs::write(work_tree.join("test.txt"), "content")
            .await
            .unwrap();

        let manager = SnapshotManager::new(&data_dir, "test-project", &work_tree, true);

        let hash = manager.track().await.unwrap();
        assert!(hash.is_some());
        assert_eq!(hash.as_ref().unwrap().len(), 40); // Git SHA-1
    }

    #[tokio::test]
    async fn test_snapshot_disabled() {
        let temp = TempDir::new().unwrap();
        let data_dir = temp.path().join("data");
        let work_tree = temp.path().join("project");

        let manager = SnapshotManager::new(&data_dir, "test-project", &work_tree, false);

        let hash = manager.track().await.unwrap();
        assert!(hash.is_none());
    }

    #[tokio::test]
    async fn test_patch_computation() {
        let temp = TempDir::new().unwrap();
        let data_dir = temp.path().join("data");
        let work_tree = temp.path().join("project");
        tokio::fs::create_dir_all(&work_tree).await.unwrap();

        // Create initial file
        tokio::fs::write(work_tree.join("test.txt"), "original")
            .await
            .unwrap();

        let manager = SnapshotManager::new(&data_dir, "test-project", &work_tree, true);

        let hash = manager.track().await.unwrap().unwrap();

        // Modify file
        tokio::fs::write(work_tree.join("test.txt"), "modified")
            .await
            .unwrap();

        let patch = manager.patch(&hash).await.unwrap();
        assert_eq!(patch.hash, hash);
        assert_eq!(patch.files.len(), 1);
    }
}
