//! Offline mode handling for storage
//!
//! Provides functionality to detect when storage is unavailable and operate
//! in read-only mode with cached data.

use crate::error::{StorageError, StorageResult};
use crate::types::StorageState;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, warn};

/// Offline mode handler
pub struct OfflineModeHandler;

impl OfflineModeHandler {
    /// Check if storage is available
    ///
    /// # Arguments
    ///
    /// * `storage_path` - Path to storage directory
    ///
    /// # Returns
    ///
    /// Returns the storage state (Available, Unavailable, or ReadOnly)
    pub fn check_storage_availability(storage_path: &Path) -> StorageState {
        // Check if path exists
        if !storage_path.exists() {
            warn!(
                "Storage unavailable: path does not exist: {}",
                storage_path.display()
            );
            return StorageState::Unavailable {
                reason: "Storage path does not exist".to_string(),
            };
        }

        // Check if path is accessible (try to read directory)
        match std::fs::read_dir(storage_path) {
            Ok(_) => {
                debug!("Storage is available: {}", storage_path.display());
                StorageState::Available
            }
            Err(e) => {
                warn!(
                    "Storage unavailable: cannot read directory {}: {}",
                    storage_path.display(),
                    e
                );
                StorageState::Unavailable {
                    reason: format!("Cannot read directory: {}", e),
                }
            }
        }
    }

    /// Check if storage is on external or network drive
    ///
    /// # Arguments
    ///
    /// * `storage_path` - Path to storage directory
    ///
    /// # Returns
    ///
    /// Returns true if storage appears to be on external/network storage
    pub fn is_external_storage(storage_path: &Path) -> bool {
        let path_str = storage_path.to_string_lossy();

        // Check for common network/external indicators
        #[cfg(target_os = "windows")]
        {
            // Check for UNC paths (network drives)
            if path_str.starts_with("\\\\") {
                return true;
            }
            // Check for mapped drives (typically Z:, Y:, etc.)
            if let Some(drive) = path_str.chars().next() {
                if drive.is_alphabetic() {
                    let drive_letter = drive.to_ascii_uppercase();
                    // Assume drives beyond D: might be external/network
                    if drive_letter > 'D' {
                        return true;
                    }
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            // Check for mounted volumes
            if path_str.starts_with("/Volumes/") {
                return true;
            }
        }

        #[cfg(target_os = "linux")]
        {
            // Check for mounted filesystems
            if path_str.starts_with("/mnt/") || path_str.starts_with("/media/") {
                return true;
            }
        }

        false
    }

    /// Transition to offline mode
    ///
    /// # Arguments
    ///
    /// * `storage_path` - Path to storage directory
    /// * `cache_available` - Whether cached data is available
    ///
    /// # Returns
    ///
    /// Returns the new storage state
    pub fn enter_offline_mode(storage_path: &Path, cache_available: bool) -> StorageState {
        let cached_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .to_string();

        if cache_available {
            warn!(
                "Entering offline mode for storage: {}. Using cached data.",
                storage_path.display()
            );
            StorageState::ReadOnly { cached_at }
        } else {
            warn!(
                "Entering offline mode for storage: {}. No cached data available.",
                storage_path.display()
            );
            StorageState::Unavailable {
                reason: "Storage unavailable and no cached data available".to_string(),
            }
        }
    }

    /// Check if we should retry storage access
    ///
    /// # Arguments
    ///
    /// * `storage_path` - Path to storage directory
    ///
    /// # Returns
    ///
    /// Returns true if storage is now available
    pub fn retry_storage_access(storage_path: &Path) -> bool {
        match Self::check_storage_availability(storage_path) {
            StorageState::Available => {
                debug!("Storage is now available: {}", storage_path.display());
                true
            }
            _ => {
                debug!("Storage is still unavailable: {}", storage_path.display());
                false
            }
        }
    }

    /// Log offline mode warning
    ///
    /// # Arguments
    ///
    /// * `storage_path` - Path to storage directory
    /// * `reason` - Reason for offline mode
    pub fn log_offline_warning(storage_path: &Path, reason: &str) {
        warn!(
            "Storage offline mode activated for {}: {}",
            storage_path.display(),
            reason
        );
    }

    /// Validate that we can operate in offline mode
    ///
    /// # Arguments
    ///
    /// * `cache_available` - Whether cached data is available
    ///
    /// # Returns
    ///
    /// Returns error if offline mode cannot be used
    pub fn validate_offline_mode(cache_available: bool) -> StorageResult<()> {
        if !cache_available {
            return Err(StorageError::internal(
                "Cannot enter offline mode: no cached data available",
            ));
        }

        debug!("Offline mode validation passed");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_check_storage_availability_exists() {
        let temp_dir = TempDir::new().unwrap();
        let state = OfflineModeHandler::check_storage_availability(temp_dir.path());

        assert_eq!(state, StorageState::Available);
    }

    #[test]
    fn test_check_storage_availability_not_exists() {
        let path = std::path::PathBuf::from("/nonexistent/path/that/does/not/exist");
        let state = OfflineModeHandler::check_storage_availability(&path);

        match state {
            StorageState::Unavailable { .. } => {
                // Expected
            }
            _ => panic!("Expected Unavailable state"),
        }
    }

    #[test]
    fn test_enter_offline_mode_with_cache() {
        let temp_dir = TempDir::new().unwrap();
        let state = OfflineModeHandler::enter_offline_mode(temp_dir.path(), true);

        match state {
            StorageState::ReadOnly { .. } => {
                // Expected
            }
            _ => panic!("Expected ReadOnly state"),
        }
    }

    #[test]
    fn test_enter_offline_mode_without_cache() {
        let temp_dir = TempDir::new().unwrap();
        let state = OfflineModeHandler::enter_offline_mode(temp_dir.path(), false);

        match state {
            StorageState::Unavailable { .. } => {
                // Expected
            }
            _ => panic!("Expected Unavailable state"),
        }
    }

    #[test]
    fn test_retry_storage_access_available() {
        let temp_dir = TempDir::new().unwrap();
        let result = OfflineModeHandler::retry_storage_access(temp_dir.path());

        assert!(result);
    }

    #[test]
    fn test_retry_storage_access_unavailable() {
        let path = std::path::PathBuf::from("/nonexistent/path");
        let result = OfflineModeHandler::retry_storage_access(&path);

        assert!(!result);
    }

    #[test]
    fn test_validate_offline_mode_with_cache() {
        let result = OfflineModeHandler::validate_offline_mode(true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_offline_mode_without_cache() {
        let result = OfflineModeHandler::validate_offline_mode(false);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_external_storage() {
        // This test is platform-specific
        #[cfg(target_os = "windows")]
        {
            // UNC paths are external
            let unc_path = std::path::PathBuf::from("\\\\server\\share");
            assert!(OfflineModeHandler::is_external_storage(&unc_path));
        }

        #[cfg(target_os = "macos")]
        {
            // /Volumes paths are external
            let volume_path = std::path::PathBuf::from("/Volumes/ExternalDrive");
            assert!(OfflineModeHandler::is_external_storage(&volume_path));
        }

        #[cfg(target_os = "linux")]
        {
            // /mnt and /media paths are external
            let mnt_path = std::path::PathBuf::from("/mnt/external");
            assert!(OfflineModeHandler::is_external_storage(&mnt_path));

            let media_path = std::path::PathBuf::from("/media/user/external");
            assert!(OfflineModeHandler::is_external_storage(&media_path));
        }
    }
}
