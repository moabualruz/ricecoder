//! Rollback capabilities, staged releases, and version management

use crate::error::{Result, UpdateError};
use crate::models::{ReleaseChannel, RollbackInfo, UpdateOperation, UpdateStatus};
use chrono::{DateTime, Duration, TimeZone, Utc};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Rollback manager for version management
#[derive(Clone)]
pub struct RollbackManager {
    backup_dir: PathBuf,
    install_dir: PathBuf,
    max_backups: usize,
}

impl RollbackManager {
    /// Create a new rollback manager
    pub fn new(backup_dir: PathBuf, install_dir: PathBuf, max_backups: usize) -> Self {
        Self {
            backup_dir,
            install_dir,
            max_backups,
        }
    }

    /// Create a backup before update
    pub async fn create_backup(&self, version: &semver::Version) -> Result<PathBuf> {
        // Ensure backup directory exists
        fs::create_dir_all(&self.backup_dir).await?;

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!(
            "backup_{}_v{}_{}",
            self.get_platform_key(),
            version,
            timestamp
        );
        let backup_path = self.backup_dir.join(backup_name);

        info!("Creating backup: {}", backup_path.display());

        // Create backup directory
        fs::create_dir_all(&backup_path).await?;

        // Copy installation files
        self.copy_installation_to_backup(&backup_path).await?;

        // Cleanup old backups if we exceed the limit
        self.cleanup_old_backups().await?;

        info!("Backup created successfully: {}", backup_path.display());
        Ok(backup_path)
    }

    /// Rollback to a specific version
    pub async fn rollback_to_version(
        &self,
        target_version: &semver::Version,
    ) -> Result<RollbackInfo> {
        info!("Rolling back to version {}", target_version);

        // Find the most recent backup for the target version
        let backup_path = self
            .find_backup_for_version(target_version)
            .await?
            .ok_or_else(|| {
                UpdateError::rollback(format!("No backup found for version {}", target_version))
            })?;

        // Perform the rollback
        self.perform_rollback(&backup_path).await?;

        let rollback_info = RollbackInfo {
            previous_version: self.get_current_version().await?,
            backup_path: backup_path.to_string_lossy().to_string(),
            reason: format!("Manual rollback to version {}", target_version),
            rolled_back_at: Utc::now(),
        };

        info!("Rollback completed successfully");
        Ok(rollback_info)
    }

    /// Rollback to a specific backup
    pub async fn rollback_to_backup(&self, backup_path: &Path) -> Result<RollbackInfo> {
        info!("Rolling back to backup: {}", backup_path.display());

        if !backup_path.exists() {
            return Err(UpdateError::rollback("Backup path does not exist"));
        }

        // Perform the rollback
        self.perform_rollback(backup_path).await?;

        let rollback_info = RollbackInfo {
            previous_version: self.get_current_version().await?,
            backup_path: backup_path.to_string_lossy().to_string(),
            reason: "Rollback to specific backup".to_string(),
            rolled_back_at: Utc::now(),
        };

        info!("Rollback to backup completed successfully");
        Ok(rollback_info)
    }

    /// List available backups
    pub async fn list_backups(&self) -> Result<Vec<BackupInfo>> {
        if !self.backup_dir.exists() {
            return Ok(vec![]);
        }

        let mut backups = vec![];
        let mut entries = fs::read_dir(&self.backup_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_dir() {
                if let Some(backup_info) = self.parse_backup_info(&entry.path()).await {
                    backups.push(backup_info);
                }
            }
        }

        // Sort by creation time (newest first)
        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(backups)
    }

    /// Get rollback plan for a version
    pub async fn get_rollback_plan(
        &self,
        target_version: &semver::Version,
    ) -> Result<RollbackPlan> {
        let current_version = self.get_current_version().await?;
        let backup = self.find_backup_for_version(target_version).await?;

        let (backup_path, steps) = if let Some(bp) = backup {
            let steps = vec![
                RollbackStep::BackupCurrentState,
                RollbackStep::StopServices,
                RollbackStep::RestoreFromBackup { path: bp.clone() },
                RollbackStep::UpdateVersionFile {
                    version: target_version.clone(),
                },
                RollbackStep::StartServices,
                RollbackStep::ValidateInstallation,
            ];
            (Some(bp), steps)
        } else {
            return Err(UpdateError::rollback(format!(
                "No backup available for version {}",
                target_version
            )));
        };

        Ok(RollbackPlan {
            current_version: current_version.clone(),
            target_version: target_version.clone(),
            backup_path,
            steps,
            estimated_duration: Duration::seconds(30),
            risk_level: if target_version < &current_version {
                "low"
            } else {
                "medium"
            }
            .to_string(),
        })
    }

    /// Validate rollback feasibility
    pub async fn validate_rollback(
        &self,
        target_version: &semver::Version,
    ) -> Result<ValidationResult> {
        let mut issues = vec![];

        // Check if backup exists
        if self
            .find_backup_for_version(target_version)
            .await?
            .is_none()
        {
            issues.push("No backup found for target version".to_string());
        }

        // Check disk space
        if let Err(e) = self.check_disk_space().await {
            issues.push(format!("Insufficient disk space: {}", e));
        }

        // Check file permissions
        if let Err(e) = self.check_permissions().await {
            issues.push(format!("Permission issues: {}", e));
        }

        let can_rollback = issues.is_empty();

        Ok(ValidationResult {
            can_rollback,
            issues,
            warnings: vec![], // Could add warnings for version compatibility
        })
    }

    /// Perform the actual rollback operation
    async fn perform_rollback(&self, backup_path: &Path) -> Result<()> {
        info!("Performing rollback from backup: {}", backup_path.display());

        // Stop services (in a real implementation, this would stop the running application)
        self.stop_services().await?;

        // Backup current state (just in case)
        let emergency_backup = self.create_emergency_backup().await?;

        // Clear current installation
        self.clear_installation().await?;

        // Restore from backup
        self.restore_from_backup(backup_path).await?;

        // Start services
        if let Err(e) = self.start_services().await {
            error!("Failed to start services after rollback: {}", e);
            // Attempt to restore emergency backup
            if let Err(restore_err) = self.restore_from_backup(&emergency_backup).await {
                error!("Emergency restore also failed: {}", restore_err);
            }
            return Err(e);
        }

        // Validate installation
        self.validate_installation().await?;

        info!("Rollback completed successfully");
        Ok(())
    }

    /// Find backup for a specific version
    async fn find_backup_for_version(&self, version: &semver::Version) -> Result<Option<PathBuf>> {
        let backups = self.list_backups().await?;

        // Find the most recent backup for this version
        for backup in backups {
            if backup.version == *version {
                return Ok(Some(backup.path));
            }
        }

        Ok(None)
    }

    /// Parse backup information from directory name
    async fn parse_backup_info(&self, path: &Path) -> Option<BackupInfo> {
        let dir_name = path.file_name()?.to_str()?;

        // Expected format: backup_{platform}_v{version}_{timestamp}
        let parts: Vec<&str> = dir_name.split('_').collect();
        if parts.len() < 4 || parts[0] != "backup" {
            return None;
        }

        let platform = parts[1].to_string();
        let version_str = parts[2].strip_prefix('v')?;
        let timestamp_str = parts[3];

        let version = semver::Version::parse(version_str).ok()?;
        let created_at = self.parse_timestamp(timestamp_str)?;

        let metadata = fs::metadata(path).await.ok()?;
        let size_bytes = self.calculate_directory_size(path).await.ok()?;

        Some(BackupInfo {
            path: path.to_path_buf(),
            version,
            platform,
            created_at,
            size_bytes,
        })
    }

    /// Parse timestamp string
    fn parse_timestamp(&self, timestamp_str: &str) -> Option<DateTime<Utc>> {
        // Expected format: YYYYMMDD_HHMMSS
        if timestamp_str.len() != 15 {
            return None;
        }

        let year: i32 = timestamp_str[0..4].parse().ok()?;
        let month: u32 = timestamp_str[4..6].parse().ok()?;
        let day: u32 = timestamp_str[6..8].parse().ok()?;
        let hour: u32 = timestamp_str[9..11].parse().ok()?;
        let minute: u32 = timestamp_str[11..13].parse().ok()?;
        let second: u32 = timestamp_str[13..15].parse().ok()?;

        Some(
            Utc.with_ymd_and_hms(year, month, day, hour, minute, second)
                .single()?
                .into(),
        )
    }

    /// Copy installation to backup
    async fn copy_installation_to_backup(&self, backup_path: &Path) -> Result<()> {
        self.copy_directory_recursive(&self.install_dir, backup_path)
            .await
    }

    /// Restore from backup
    async fn restore_from_backup(&self, backup_path: &Path) -> Result<()> {
        self.copy_directory_recursive(backup_path, &self.install_dir)
            .await
    }

    /// Clear current installation
    async fn clear_installation(&self) -> Result<()> {
        // Be very careful here - only remove files we expect to be part of the installation
        let keep_files = ["backups", "config", "logs"]; // Files/directories to preserve

        let mut entries = fs::read_dir(&self.install_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            if !keep_files.contains(&file_name) {
                if path.is_dir() {
                    fs::remove_dir_all(&path).await?;
                } else {
                    fs::remove_file(&path).await?;
                }
            }
        }

        Ok(())
    }

    /// Cleanup old backups
    async fn cleanup_old_backups(&self) -> Result<()> {
        let backups = self.list_backups().await?;

        if backups.len() <= self.max_backups {
            return Ok(());
        }

        // Remove oldest backups
        let to_remove = backups.len() - self.max_backups;
        for backup in backups.iter().rev().take(to_remove) {
            info!("Removing old backup: {}", backup.path.display());
            if let Err(e) = fs::remove_dir_all(&backup.path).await {
                warn!(
                    "Failed to remove old backup {}: {}",
                    backup.path.display(),
                    e
                );
            }
        }

        Ok(())
    }

    /// Get current version
    async fn get_current_version(&self) -> Result<semver::Version> {
        let version_file = self.install_dir.join("version.txt");
        if version_file.exists() {
            let content = fs::read_to_string(&version_file).await?;
            semver::Version::parse(content.trim())
                .map_err(|e| UpdateError::generic(format!("Invalid version file: {}", e)))
        } else {
            // Try to get from binary
            self.get_version_from_binary().await
        }
    }

    /// Get version from binary
    async fn get_version_from_binary(&self) -> Result<semver::Version> {
        // Simplified implementation
        Ok(semver::Version::new(0, 1, 0))
    }

    /// Get platform key
    fn get_platform_key(&self) -> String {
        format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
    }

    /// Calculate directory size
    async fn calculate_directory_size(&self, path: &Path) -> Result<u64> {
        let mut size = 0u64;
        let mut entries = fs::read_dir(path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let metadata = entry.metadata().await?;
            if metadata.is_dir() {
                size += Box::pin(self.calculate_directory_size(&entry.path())).await?;
            } else {
                size += metadata.len();
            }
        }

        Ok(size)
    }

    /// Copy directory recursively
    async fn copy_directory_recursive(&self, from: &Path, to: &Path) -> Result<()> {
        use std::future::Future;
        use std::pin::Pin;

        fn copy_recursive(
            from: PathBuf,
            to: PathBuf,
        ) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> {
            Box::pin(async move {
                let from_path = from.as_path();
                let to_path = to.as_path();
                let metadata = fs::metadata(from_path).await?;
                if metadata.is_dir() {
                    fs::create_dir_all(to_path).await?;
                    let mut entries = fs::read_dir(from_path).await?;
                    while let Some(entry) = entries.next_entry().await? {
                        let entry_path = entry.path();
                        let file_name = entry_path.file_name().unwrap();
                        let dest_path = to.join(file_name);
                        copy_recursive(entry_path, dest_path).await?;
                    }
                } else {
                    fs::copy(from_path, to_path).await?;
                }
                Ok(())
            })
        }

        copy_recursive(from.to_path_buf(), to.to_path_buf()).await
    }

    /// Service management (simplified)
    async fn stop_services(&self) -> Result<()> {
        Ok(())
    }
    async fn start_services(&self) -> Result<()> {
        Ok(())
    }
    async fn create_emergency_backup(&self) -> Result<PathBuf> {
        // Simplified - would create a quick backup
        Ok(self.backup_dir.join("emergency_backup"))
    }
    async fn validate_installation(&self) -> Result<()> {
        Ok(())
    }
    async fn check_disk_space(&self) -> Result<()> {
        Ok(())
    }
    async fn check_permissions(&self) -> Result<()> {
        Ok(())
    }
}

/// Backup information
#[derive(Debug, Clone)]
pub struct BackupInfo {
    /// Path to the backup
    pub path: PathBuf,
    /// Version of the backup
    pub version: semver::Version,
    /// Platform of the backup
    pub platform: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Size in bytes
    pub size_bytes: u64,
}

/// Rollback plan
#[derive(Debug, Clone)]
pub struct RollbackPlan {
    /// Current version
    pub current_version: semver::Version,
    /// Target version
    pub target_version: semver::Version,
    /// Backup path to use
    pub backup_path: Option<PathBuf>,
    /// Rollback steps
    pub steps: Vec<RollbackStep>,
    /// Estimated duration
    pub estimated_duration: Duration,
    /// Risk level
    pub risk_level: String,
}

/// Rollback step
#[derive(Debug, Clone)]
pub enum RollbackStep {
    /// Backup current state
    BackupCurrentState,
    /// Stop running services
    StopServices,
    /// Restore from backup
    RestoreFromBackup { path: PathBuf },
    /// Update version file
    UpdateVersionFile { version: semver::Version },
    /// Start services
    StartServices,
    /// Validate installation
    ValidateInstallation,
}

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether rollback can proceed
    pub can_rollback: bool,
    /// List of issues preventing rollback
    pub issues: Vec<String>,
    /// List of warnings
    pub warnings: Vec<String>,
}

/// Staged release manager
#[derive(Clone)]
pub struct StagedReleaseManager {
    staging_dir: PathBuf,
    release_channels: HashMap<String, ReleaseChannel>,
}

impl StagedReleaseManager {
    /// Create a new staged release manager
    pub fn new(staging_dir: PathBuf) -> Self {
        let mut release_channels = HashMap::new();
        release_channels.insert("stable".to_string(), ReleaseChannel::Stable);
        release_channels.insert("beta".to_string(), ReleaseChannel::Beta);
        release_channels.insert("nightly".to_string(), ReleaseChannel::Nightly);

        Self {
            staging_dir,
            release_channels,
        }
    }

    /// Stage a release for gradual rollout
    pub async fn stage_release(
        &self,
        release_info: &crate::models::ReleaseInfo,
        channel: &str,
    ) -> Result<()> {
        let channel_dir = self.staging_dir.join(channel);
        fs::create_dir_all(&channel_dir).await?;

        // Create staged release metadata
        let staged_release = StagedRelease {
            release_info: release_info.clone(),
            channel: channel.to_string(),
            staged_at: Utc::now(),
            rollout_percentage: 0,
            status: StagedReleaseStatus::Staged,
        };

        let metadata_path = channel_dir.join("metadata.json");
        let metadata = serde_json::to_string_pretty(&staged_release)?;
        fs::write(&metadata_path, metadata).await?;

        info!(
            "Release {} staged for channel {}",
            release_info.version, channel
        );
        Ok(())
    }

    /// Update rollout percentage for staged release
    pub async fn update_rollout(&self, channel: &str, percentage: u8) -> Result<()> {
        let metadata_path = self.staging_dir.join(channel).join("metadata.json");

        let mut staged_release: StagedRelease = {
            let content = fs::read_to_string(&metadata_path).await?;
            serde_json::from_str(&content)?
        };

        staged_release.rollout_percentage = percentage;
        if percentage >= 100 {
            staged_release.status = StagedReleaseStatus::FullyRolledOut;
        } else if percentage > 0 {
            staged_release.status = StagedReleaseStatus::RollingOut;
        }

        let metadata = serde_json::to_string_pretty(&staged_release)?;
        fs::write(&metadata_path, metadata).await?;

        info!("Updated rollout for channel {} to {}%", channel, percentage);
        Ok(())
    }

    /// Get staged release info
    pub async fn get_staged_release(&self, channel: &str) -> Result<Option<StagedRelease>> {
        let metadata_path = self.staging_dir.join(channel).join("metadata.json");

        if !metadata_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&metadata_path).await?;
        let staged_release: StagedRelease = serde_json::from_str(&content)?;
        Ok(Some(staged_release))
    }
}

/// Staged release information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StagedRelease {
    /// Release information
    pub release_info: crate::models::ReleaseInfo,
    /// Release channel
    pub channel: String,
    /// When it was staged
    pub staged_at: DateTime<Utc>,
    /// Current rollout percentage
    pub rollout_percentage: u8,
    /// Status
    pub status: StagedReleaseStatus,
}

/// Staged release status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum StagedReleaseStatus {
    /// Release is staged but not rolling out
    Staged,
    /// Release is actively rolling out
    RollingOut,
    /// Release has been fully rolled out
    FullyRolledOut,
    /// Release rollout has been paused
    Paused,
    /// Release has been rolled back
    RolledBack,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_rollback_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = RollbackManager::new(
            temp_dir.path().join("backups"),
            temp_dir.path().join("install"),
            5,
        );

        assert_eq!(manager.max_backups, 5);
    }

    #[tokio::test]
    async fn test_backup_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = RollbackManager::new(
            temp_dir.path().join("backups"),
            temp_dir.path().join("install"),
            5,
        );

        // Create a test file in install directory
        let install_dir = temp_dir.path().join("install");
        fs::create_dir_all(&install_dir).await.unwrap();
        let test_file = install_dir.join("test.txt");
        fs::write(&test_file, "test content").await.unwrap();

        let version = semver::Version::from_str("1.0.0").unwrap();
        let backup_path = manager.create_backup(&version).await.unwrap();

        assert!(backup_path.exists());
        assert!(backup_path.join("test.txt").exists());
    }

    #[tokio::test]
    async fn test_staged_release_manager() {
        let temp_dir = TempDir::new().unwrap();
        let manager = StagedReleaseManager::new(temp_dir.path().to_path_buf());

        let release_info = crate::models::ReleaseInfo {
            version: semver::Version::from_str("1.1.0").unwrap(),
            channel: crate::models::ReleaseChannel::Beta,
            release_date: Utc::now(),
            minimum_version: None,
            notes: "Test release".to_string(),
            downloads: Default::default(),
            security_advisories: vec![],
            compliance: crate::models::ComplianceInfo {
                soc2_compliant: true,
                gdpr_compliant: true,
                hipaa_compliant: false,
                security_audited: true,
                last_review: Utc::now(),
            },
        };

        manager.stage_release(&release_info, "beta").await.unwrap();

        let staged = manager.get_staged_release("beta").await.unwrap().unwrap();
        assert_eq!(staged.release_info.version, release_info.version);
        assert_eq!(staged.rollout_percentage, 0);
    }
}
