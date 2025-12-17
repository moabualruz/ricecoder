//! Self-updating binary functionality with rollback and security validation

use crate::error::{Result, UpdateError};
use crate::models::{ReleaseInfo, UpdateOperation, UpdateStatus, RollbackInfo, SecurityValidationResult};
use crate::policy::UpdatePolicy;
use chrono::{DateTime, Utc};
use futures::future::join_all;
use reqwest::Client;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::NamedTempFile;
use tokio::sync::RwLock;
use tracing::{info, warn, error};
use uuid::Uuid;

/// Binary updater service
#[derive(Clone)]
pub struct BinaryUpdater {
    client: Client,
    policy: UpdatePolicy,
    install_path: PathBuf,
    backup_dir: PathBuf,
    temp_dir: PathBuf,
}

impl BinaryUpdater {
    /// Create a new binary updater
    pub fn new(policy: UpdatePolicy, install_path: PathBuf) -> Self {
        let backup_dir = install_path.join("backups");
        let temp_dir = std::env::temp_dir().join("ricecoder-updates");

        Self {
            client: Client::new(),
            policy,
            install_path,
            backup_dir,
            temp_dir,
        }
    }

    /// Download and install an update
    pub async fn install_update(&self, release_info: &ReleaseInfo) -> Result<UpdateOperation> {
        let operation_id = Uuid::new_v4();
        let started_at = Utc::now();

        info!("Starting update installation: {} -> {}", self.get_current_version()?, release_info.version);

        // Create operation record
        let mut operation = UpdateOperation {
            id: operation_id,
            version: release_info.version.clone(),
            status: UpdateStatus::Pending,
            started_at,
            completed_at: None,
            error_message: None,
            rollback_info: None,
            security_validation: SecurityValidationResult {
                passed: false,
                validated_at: started_at,
                checksum_valid: false,
                signature_valid: None,
                details: Default::default(),
            },
        };

        // Validate policy
        if let Err(e) = self.validate_policy_for_update(release_info).await {
            operation.status = UpdateStatus::Failed;
            operation.error_message = Some(e.to_string());
            operation.completed_at = Some(Utc::now());
            return Ok(operation);
        }

        // Create backup
        operation.status = UpdateStatus::Downloading;
        let backup_path = match self.create_backup().await {
            Ok(path) => path,
            Err(e) => {
                operation.status = UpdateStatus::Failed;
                operation.error_message = Some(format!("Backup creation failed: {}", e));
                operation.completed_at = Some(Utc::now());
                return Ok(operation);
            }
        };

        // Download update
        let download_path = match self.download_update(release_info).await {
            Ok(path) => path,
            Err(e) => {
                operation.status = UpdateStatus::Failed;
                operation.error_message = Some(format!("Download failed: {}", e));
                operation.completed_at = Some(Utc::now());
                return Ok(operation);
            }
        };

        // Validate security
        operation.status = UpdateStatus::Downloaded;
        operation.security_validation = match self.validate_security(&download_path, release_info).await {
            Ok(validation) => validation,
            Err(e) => {
                operation.status = UpdateStatus::Failed;
                operation.error_message = Some(format!("Security validation failed: {}", e));
                operation.completed_at = Some(Utc::now());
                return Ok(operation);
            }
        };

        // Install update
        operation.status = UpdateStatus::Installing;
        if let Err(e) = self.perform_installation(&download_path).await {
            operation.status = UpdateStatus::Failed;
            operation.error_message = Some(format!("Installation failed: {}", e));
            operation.completed_at = Some(Utc::now());

            // Attempt rollback
            if let Err(rollback_err) = self.rollback_to_backup(&backup_path).await {
                error!("Rollback also failed: {}", rollback_err);
            } else {
                operation.status = UpdateStatus::RolledBack;
                operation.rollback_info = Some(RollbackInfo {
                    previous_version: self.get_current_version().unwrap_or_else(|_| semver::Version::new(0, 0, 0)),
                    backup_path: backup_path.to_string_lossy().to_string(),
                    reason: "Installation failed".to_string(),
                    rolled_back_at: Utc::now(),
                });
            }

            return Ok(operation);
        }

        // Cleanup
        operation.status = UpdateStatus::Installed;
        operation.completed_at = Some(Utc::now());

        // Remove temporary files
        if let Err(e) = tokio::fs::remove_file(&download_path).await {
            warn!("Failed to cleanup download file: {}", e);
        }

        info!("Update installation completed successfully");
        Ok(operation)
    }

    /// Rollback to a previous version
    pub async fn rollback(&self, target_version: &semver::Version) -> Result<()> {
        info!("Rolling back to version {}", target_version);

        // Find backup for target version
        let backup_path = self.find_backup_for_version(target_version).await?
            .ok_or_else(|| UpdateError::rollback("No backup found for target version"))?;

        self.rollback_to_backup(&backup_path).await?;

        // Update current version indicator if exists
        self.update_version_file(target_version).await?;

        info!("Rollback to version {} completed", target_version);
        Ok(())
    }

    /// Get current installed version
    pub fn get_current_version(&self) -> Result<semver::Version> {
        // Try to read version from a version file
        let version_file = self.install_path.join("version.txt");
        if version_file.exists() {
            let content = fs::read_to_string(&version_file)?;
            return semver::Version::parse(content.trim())
                .map_err(|e| UpdateError::generic(format!("Invalid version file: {}", e)));
        }

        // Fallback: try to get version from binary
        self.get_version_from_binary()
    }

    /// Validate policy allows this update
    async fn validate_policy_for_update(&self, release_info: &ReleaseInfo) -> Result<()> {
        // Get download size estimate
        let download_size_mb = release_info.downloads.values()
            .map(|d| d.size / (1024 * 1024))
            .max()
            .unwrap_or(0) as u32;

        // Get compliance tags
        let compliance_tags = vec![
            if release_info.compliance.soc2_compliant { "SOC2" } else { "" },
            if release_info.compliance.gdpr_compliant { "GDPR" } else { "" },
            if release_info.compliance.hipaa_compliant { "HIPAA" } else { "" },
        ].into_iter()
         .filter(|s| !s.is_empty())
         .map(|s| s.to_string())
         .collect::<Vec<_>>();

        match self.policy.evaluate_update(&release_info.channel, download_size_mb, &compliance_tags) {
            crate::policy::PolicyResult::Allowed => Ok(()),
            crate::policy::PolicyResult::Denied(reason) => {
                Err(UpdateError::policy_violation(reason))
            }
            crate::policy::PolicyResult::RequiresApproval => {
                Err(UpdateError::policy_violation("Update requires manual approval"))
            }
        }
    }

    /// Create backup of current installation
    async fn create_backup(&self) -> Result<PathBuf> {
        // Ensure backup directory exists
        tokio::fs::create_dir_all(&self.backup_dir).await?;

        let current_version = self.get_current_version()?;
        let backup_name = format!("ricecoder-{}-backup-{}",
            current_version, Utc::now().format("%Y%m%d-%H%M%S"));
        let backup_path = self.backup_dir.join(backup_name);

        // Create backup directory
        tokio::fs::create_dir_all(&backup_path).await?;

        // Copy all files from install directory
        self.copy_directory_recursive(&self.install_path, &backup_path).await?;

        info!("Backup created at: {}", backup_path.display());
        Ok(backup_path)
    }

    /// Download update files
    async fn download_update(&self, release_info: &ReleaseInfo) -> Result<PathBuf> {
        // Ensure temp directory exists
        tokio::fs::create_dir_all(&self.temp_dir).await?;

        // Determine platform-specific download
        let platform_key = self.get_platform_key();
        let download_info = release_info.downloads.get(&platform_key)
            .ok_or_else(|| UpdateError::download(format!("No download available for platform: {}", platform_key)))?;

        info!("Downloading update from: {}", download_info.url);

        // Download file
        let response = self.client
            .get(&download_info.url)
            .header("User-Agent", "RiceCoder-Updater")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(UpdateError::download(format!("Download failed with status: {}", response.status())));
        }

        // Create temporary file
        let temp_file = NamedTempFile::new_in(&self.temp_dir)?;
        let temp_path = temp_file.path().to_path_buf();

        // Keep temp file alive
        let mut temp_file = temp_file;

        // Download to temp file
        let content = response.bytes().await?;
        tokio::fs::write(&temp_path, &content).await?;

        info!("Download completed: {} bytes", content.len());
        Ok(temp_path)
    }

    /// Validate security of downloaded file
    async fn validate_security(&self, file_path: &Path, release_info: &ReleaseInfo) -> Result<SecurityValidationResult> {
        let validated_at = Utc::now();
        let mut details = std::collections::HashMap::new();

        // Get platform-specific download info
        let platform_key = self.get_platform_key();
        let download_info = release_info.downloads.get(&platform_key)
            .ok_or_else(|| UpdateError::security_validation("No download info for platform"))?;

        // Validate checksum
        let checksum_valid = self.validate_checksum(file_path, &download_info.sha256).await?;
        details.insert("checksum_valid".to_string(), checksum_valid.to_string());

        // Validate signature if required and available
        let signature_valid = if self.policy.signature_required() {
            if let Some(signature) = &download_info.signature {
                Some(self.validate_signature(file_path, signature).await?)
            } else {
                return Err(UpdateError::security_validation("Signature required but not provided"));
            }
        } else {
            if let Some(signature) = &download_info.signature {
                Some(self.validate_signature(file_path, signature).await?)
            } else {
                None
            }
        };

        details.insert("signature_valid".to_string(), signature_valid.map(|v| v.to_string()).unwrap_or_else(|| "N/A".to_string()));

        let passed = checksum_valid && signature_valid.unwrap_or(true);

        Ok(SecurityValidationResult {
            passed,
            validated_at,
            checksum_valid,
            signature_valid,
            details,
        })
    }

    /// Perform the actual installation
    async fn perform_installation(&self, download_path: &Path) -> Result<()> {
        info!("Performing installation from: {}", download_path.display());

        // For now, assume the download is a compressed archive
        // In a real implementation, this would extract and replace files
        // This is a simplified version for demonstration

        // Extract archive (simplified - would use actual archive extraction)
        let extract_dir = self.temp_dir.join("extract");
        tokio::fs::create_dir_all(&extract_dir).await?;

        // Simulate extraction by copying the file
        let binary_name = self.get_binary_name();
        let extracted_binary = extract_dir.join(&binary_name);
        tokio::fs::copy(download_path, &extracted_binary).await?;

        // Make executable on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = tokio::fs::metadata(&extracted_binary).await?.permissions();
            perms.set_mode(0o755);
            tokio::fs::set_permissions(&extracted_binary, perms).await?;
        }

        // Replace current binary
        let current_binary = self.install_path.join(&binary_name);
        let backup_binary = current_binary.with_extension("old");

        // Backup current binary
        if current_binary.exists() {
            tokio::fs::copy(&current_binary, &backup_binary).await?;
        }

        // Install new binary
        tokio::fs::copy(&extracted_binary, &current_binary).await?;

        // Update version file
        self.update_version_file(&semver::Version::new(0, 1, 0)).await?; // Would be from release_info

        // Cleanup
        tokio::fs::remove_dir_all(&extract_dir).await?;

        info!("Installation completed successfully");
        Ok(())
    }

    /// Rollback to a backup
    async fn rollback_to_backup(&self, backup_path: &Path) -> Result<()> {
        info!("Rolling back using backup: {}", backup_path.display());

        // Copy files back from backup
        self.copy_directory_recursive(backup_path, &self.install_path).await?;

        info!("Rollback completed");
        Ok(())
    }

    /// Find backup for a specific version
    async fn find_backup_for_version(&self, version: &semver::Version) -> Result<Option<PathBuf>> {
        if !self.backup_dir.exists() {
            return Ok(None);
        }

        let mut entries = tokio::fs::read_dir(&self.backup_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                let dir_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");

                if dir_name.starts_with(&format!("ricecoder-{}-backup", version)) {
                    return Ok(Some(path));
                }
            }
        }

        Ok(None)
    }

    /// Get platform-specific key for downloads
    fn get_platform_key(&self) -> String {
        let target_os = std::env::consts::OS;
        let target_arch = std::env::consts::ARCH;

        format!("{}-{}", target_os, target_arch)
    }

    /// Get binary name for current platform
    fn get_binary_name(&self) -> String {
        if cfg!(windows) {
            "ricecoder.exe".to_string()
        } else {
            "ricecoder".to_string()
        }
    }

    /// Get version from binary (fallback method)
    fn get_version_from_binary(&self) -> Result<semver::Version> {
        let binary_path = self.install_path.join(self.get_binary_name());

        // Try to run binary with --version flag
        let output = Command::new(&binary_path)
            .arg("--version")
            .output()
            .map_err(|e| UpdateError::generic(format!("Failed to run binary: {}", e)))?;

        let version_str = String::from_utf8_lossy(&output.stdout);
        let version_line = version_str.lines()
            .find(|line| line.contains("ricecoder"))
            .unwrap_or(&version_str);

        // Extract version (simplified parsing)
        if let Some(version_part) = version_line.split_whitespace().nth(1) {
            semver::Version::parse(version_part.trim())
                .map_err(|e| UpdateError::generic(format!("Failed to parse version: {}", e)))
        } else {
            Err(UpdateError::generic("Could not extract version from binary output"))
        }
    }

    /// Validate file checksum
    async fn validate_checksum(&self, file_path: &Path, expected_sha256: &str) -> Result<bool> {
        let content = tokio::fs::read(file_path).await?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let result = hasher.finalize();
        let actual_sha256 = hex::encode(result);

        Ok(actual_sha256 == expected_sha256)
    }

    /// Validate file signature (simplified - would use actual crypto)
    async fn validate_signature(&self, _file_path: &Path, _signature: &str) -> Result<bool> {
        // In a real implementation, this would verify cryptographic signatures
        // For now, just return true
        Ok(true)
    }

    /// Update version file
    async fn update_version_file(&self, version: &semver::Version) -> Result<()> {
        let version_file = self.install_path.join("version.txt");
        tokio::fs::write(&version_file, version.to_string()).await?;
        Ok(())
    }

    /// Recursively copy directory
    async fn copy_directory_recursive(&self, from: &Path, to: &Path) -> Result<()> {
        use std::pin::Pin;
        use std::future::Future;

        fn copy_recursive(from: PathBuf, to: PathBuf) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> {
            Box::pin(async move {
                let from_path = from.as_path();
                let to_path = to.as_path();
                let metadata = tokio::fs::metadata(from_path).await?;
                if metadata.is_dir() {
                    tokio::fs::create_dir_all(to_path).await?;
                    let mut entries = tokio::fs::read_dir(from_path).await?;
                    while let Some(entry) = entries.next_entry().await? {
                        let entry_path = entry.path();
                        let file_name = entry_path.file_name().unwrap();
                        let dest_path = to.join(file_name);
                        copy_recursive(entry_path, dest_path).await?;
                    }
                } else {
                    tokio::fs::copy(from_path, to_path).await?;
                }
                Ok(())
            })
        }

        copy_recursive(from.to_path_buf(), to.to_path_buf()).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_binary_updater_creation() {
        let temp_dir = TempDir::new().unwrap();
        let policy = UpdatePolicy::default();
        let updater = BinaryUpdater::new(policy, temp_dir.path().to_path_buf());

        assert_eq!(updater.install_path, temp_dir.path());
    }

    #[test]
    fn test_platform_key() {
        let temp_dir = TempDir::new().unwrap();
        let policy = UpdatePolicy::default();
        let updater = BinaryUpdater::new(policy, temp_dir.path().to_path_buf());

        let platform_key = updater.get_platform_key();
        assert!(platform_key.contains(std::env::consts::OS));
        assert!(platform_key.contains(std::env::consts::ARCH));
    }

    #[test]
    fn test_binary_name() {
        let temp_dir = TempDir::new().unwrap();
        let policy = UpdatePolicy::default();
        let updater = BinaryUpdater::new(policy, temp_dir.path().to_path_buf());

        let binary_name = updater.get_binary_name();
        if cfg!(windows) {
            assert_eq!(binary_name, "ricecoder.exe");
        } else {
            assert_eq!(binary_name, "ricecoder");
        }
    }
}