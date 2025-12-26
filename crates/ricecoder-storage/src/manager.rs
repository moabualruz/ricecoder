//! Storage manager trait and path resolution
//!
//! Provides unified path resolution for the RiceCoder storage structure:
//! - Global user config: `~/Documents/.ricecoder/` (or `~/.ricecoder/` fallback)
//! - Project config: `.rice/` in project root

use std::path::{Path, PathBuf};

use crate::{
    error::{StorageError, StorageResult},
    types::{ConfigSubdirectory, ResourceType, RuntimeStorageType, StorageDirectory, StorageMode},
};

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
///
/// Unified storage structure:
/// ```text
/// ~/Documents/.ricecoder/          # Global user config
/// ├── config/                      # User-editable config files
/// │   ├── config.yaml              # Main config
/// │   ├── agents/                  # Agent definitions
/// │   ├── commands/                # Slash commands
/// │   ├── themes/                  # Custom themes
/// │   └── prompts/                 # Prompt templates
/// ├── auth/                        # Credentials
/// │   └── providers.yaml           # API keys
/// ├── storage/                     # Runtime data
/// │   ├── sessions/
/// │   ├── messages/
/// │   └── ...
/// ├── logs/                        # Log files
/// ├── cache/                       # Cached data
/// └── templates/                   # User templates
///
/// .rice/                           # Project-specific config
/// └── config.yaml                  # Project overrides
/// ```
pub struct PathResolver;

impl PathResolver {
    /// Project folder name (since global is .ricecoder/)
    pub const PROJECT_DIR: &'static str = ".rice";

    /// Global folder name
    pub const GLOBAL_DIR: &'static str = ".ricecoder";

    /// Resolve the global storage path based on OS and environment
    ///
    /// Priority:
    /// 1. RICECODER_HOME environment variable
    /// 2. ~/Documents/.ricecoder/ (primary - OS-appropriate Documents)
    /// 3. ~/.ricecoder/ (fallback if Documents doesn't exist)
    pub fn resolve_global_path() -> StorageResult<PathBuf> {
        // Check for RICECODER_HOME environment variable
        if let Ok(home_override) = std::env::var("RICECODER_HOME") {
            let path = PathBuf::from(home_override);
            return Ok(path);
        }

        // Try Documents folder first (OS-appropriate)
        if let Some(docs_dir) = dirs::document_dir() {
            let ricecoder_path = docs_dir.join(Self::GLOBAL_DIR);
            return Ok(ricecoder_path);
        }

        // Fallback to home directory
        if let Some(home_dir) = dirs::home_dir() {
            let ricecoder_path = home_dir.join(Self::GLOBAL_DIR);
            return Ok(ricecoder_path);
        }

        Err(StorageError::path_resolution_error(
            "Could not determine home directory",
        ))
    }

    /// Resolve the project storage path (.rice/ in current directory)
    pub fn resolve_project_path() -> PathBuf {
        PathBuf::from(Self::PROJECT_DIR)
    }

    /// Get the path for a top-level storage directory
    pub fn storage_dir(base: &Path, dir: StorageDirectory) -> PathBuf {
        base.join(dir.dir_name())
    }

    /// Get the path for a config subdirectory
    pub fn config_subdir(base: &Path, subdir: ConfigSubdirectory) -> PathBuf {
        base.join(StorageDirectory::Config.dir_name())
            .join(subdir.dir_name())
    }

    /// Get the path for a runtime storage subdirectory
    pub fn runtime_storage_path(base: &Path, storage_type: RuntimeStorageType) -> PathBuf {
        base.join(StorageDirectory::Storage.dir_name())
            .join(storage_type.dir_name())
    }

    /// Get the path for the main config file (config/config.yaml or similar)
    pub fn main_config_path(base: &Path) -> PathBuf {
        base.join(StorageDirectory::Config.dir_name())
            .join("config.yaml")
    }

    /// Get the path for the auth providers file
    pub fn auth_providers_path(base: &Path) -> PathBuf {
        base.join(StorageDirectory::Auth.dir_name())
            .join("providers.yaml")
    }

    /// Get the path for the tips file
    pub fn tips_path(base: &Path) -> PathBuf {
        base.join(StorageDirectory::Config.dir_name())
            .join("tips.txt")
    }

    /// Get the path for log files
    pub fn log_file(base: &Path, name: &str) -> PathBuf {
        base.join(StorageDirectory::Logs.dir_name()).join(name)
    }

    /// Expand ~ in paths to home directory
    pub fn expand_home(path: &Path) -> StorageResult<PathBuf> {
        let path_str = path
            .to_str()
            .ok_or_else(|| StorageError::path_resolution_error("Invalid path encoding"))?;

        if path_str.starts_with('~') {
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

    /// Check if we're in a project directory (has .rice/ folder)
    pub fn is_project_dir(path: &Path) -> bool {
        path.join(Self::PROJECT_DIR).exists()
    }

    /// Find the project root by walking up directories
    pub fn find_project_root(start: &Path) -> Option<PathBuf> {
        let mut current = start.to_path_buf();
        loop {
            if Self::is_project_dir(&current) {
                return Some(current);
            }
            if !current.pop() {
                return None;
            }
        }
    }
}
