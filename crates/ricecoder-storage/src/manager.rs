//! Storage manager trait and path resolution

use crate::error::{StorageError, StorageResult};
use crate::types::{ResourceType, StorageMode};
use std::path::{Path, PathBuf};

/// Storage manager trait for managing storage operations
pub trait StorageManager: Send + Sync {
    /// Get the global storage path
    fn global_path(&self) -> &PathBuf;

    /// Get the project storage path (if in a project)
    fn project_path(&self) -> Option<&PathBuf>;

    /// Get the current storage mode
    fn mode(&self) -> StorageMode;

    /// Get the path for a resource type in global storage
    fn global_resource_path(&self, resource_type: ResourceType) -> PathBuf;

    /// Get the path for a resource type in project storage
    fn project_resource_path(&self, resource_type: ResourceType) -> Option<PathBuf>;

    /// Check if this is the first run
    fn is_first_run(&self) -> bool;
}

/// Path resolver for cross-platform storage paths
pub struct PathResolver;

impl PathResolver {
    /// Resolve the global storage path based on OS and environment
    ///
    /// Priority:
    /// 1. RICECODER_HOME environment variable
    /// 2. ~/Documents/.ricecoder/ (primary)
    /// 3. ~/.ricecoder/ (fallback if Documents doesn't exist)
    pub fn resolve_global_path() -> StorageResult<PathBuf> {
        // Check for RICECODER_HOME environment variable
        if let Ok(home_override) = std::env::var("RICECODER_HOME") {
            let path = PathBuf::from(home_override);
            return Ok(path);
        }

        // Try Documents folder first
        if let Some(docs_dir) = dirs::document_dir() {
            let ricecoder_path = docs_dir.join(".ricecoder");
            return Ok(ricecoder_path);
        }

        // Fallback to home directory
        if let Some(home_dir) = dirs::home_dir() {
            let ricecoder_path = home_dir.join(".ricecoder");
            return Ok(ricecoder_path);
        }

        Err(StorageError::path_resolution_error(
            "Could not determine home directory",
        ))
    }

    /// Resolve the user storage path (~/.ricecoder/)
    pub fn resolve_user_path() -> StorageResult<PathBuf> {
        if let Some(home_dir) = dirs::home_dir() {
            let ricecoder_path = home_dir.join(".ricecoder");
            Ok(ricecoder_path)
        } else {
            Err(StorageError::path_resolution_error(
                "Could not determine home directory",
            ))
        }
    }

    /// Resolve the project storage path (./.agent/)
    pub fn resolve_project_path() -> PathBuf {
        PathBuf::from(".agent")
    }

    /// Expand ~ in paths to home directory
    pub fn expand_home(path: &Path) -> StorageResult<PathBuf> {
        let path_str = path
            .to_str()
            .ok_or_else(|| StorageError::path_resolution_error("Invalid path encoding"))?;

        if path_str.starts_with("~") {
            if let Some(home_dir) = dirs::home_dir() {
                let expanded = if path_str == "~" {
                    home_dir
                } else {
                    home_dir.join(&path_str[2..])
                };
                return Ok(expanded);
            }
        }

        Ok(path.to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_global_path_with_env_override() {
        // Set RICECODER_HOME environment variable
        std::env::set_var("RICECODER_HOME", "/tmp/ricecoder-test");
        let path = PathResolver::resolve_global_path().expect("Should resolve path");
        assert_eq!(path, PathBuf::from("/tmp/ricecoder-test"));
        std::env::remove_var("RICECODER_HOME");
    }

    #[test]
    fn test_resolve_global_path_without_env() {
        // Ensure RICECODER_HOME is not set
        std::env::remove_var("RICECODER_HOME");
        let path = PathResolver::resolve_global_path().expect("Should resolve path");
        // Should be either Documents/.ricecoder or ~/.ricecoder
        assert!(path.to_string_lossy().contains(".ricecoder"));
    }

    #[test]
    fn test_resolve_project_path() {
        let path = PathResolver::resolve_project_path();
        assert_eq!(path, PathBuf::from(".agent"));
    }

    #[test]
    fn test_expand_home_with_tilde() {
        let path = PathBuf::from("~/.ricecoder");
        let expanded = PathResolver::expand_home(&path).expect("Should expand");
        assert!(!expanded.to_string_lossy().contains("~"));
    }

    #[test]
    fn test_expand_home_without_tilde() {
        let path = PathBuf::from("/tmp/ricecoder");
        let expanded = PathResolver::expand_home(&path).expect("Should expand");
        assert_eq!(expanded, path);
    }
}
