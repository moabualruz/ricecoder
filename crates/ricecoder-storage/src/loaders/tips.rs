//! Tips loader for RiceCoder
//!
//! Loads user tips from `config/tips.txt` file.
//! Each line is a separate tip displayed in the UI.

use std::fs;
use std::path::PathBuf;

use crate::error::{StorageError, StorageResult};

/// Loader for tips displayed in the UI
pub struct TipsLoader {
    config_path: PathBuf,
}

impl TipsLoader {
    /// Create a new tips loader with the given config path
    pub fn new(config_path: PathBuf) -> Self {
        Self { config_path }
    }

    /// Create a tips loader using the default config path
    ///
    /// Priority: Project (.rice/tips.txt) > Global (~/Documents/.ricecoder/config/tips.txt)
    pub fn with_default_path() -> Self {
        use crate::manager::PathResolver;
        use crate::types::StorageDirectory;

        // First try project-local .rice/tips.txt
        let project_path = PathResolver::resolve_project_path().join("tips.txt");
        if project_path.exists() {
            return Self::new(project_path);
        }

        // Fall back to global ~/Documents/.ricecoder/config/tips.txt
        if let Ok(global_path) = PathResolver::resolve_global_path() {
            let global_tips = global_path
                .join(StorageDirectory::Config.dir_name())
                .join("tips.txt");
            if global_tips.exists() {
                return Self::new(global_tips);
            }
        }

        // Last resort: current directory's config/tips.txt (for backward compat)
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self::new(cwd.join("config").join("tips.txt"))
    }

    /// Load all tips from the file
    pub fn load_all(&self) -> StorageResult<Vec<String>> {
        if !self.config_path.exists() {
            return Ok(Self::default_tips());
        }

        let content = fs::read_to_string(&self.config_path).map_err(|e| {
            StorageError::io_error(
                self.config_path.clone(),
                crate::error::IoOperation::Read,
                e,
            )
        })?;

        let tips: Vec<String> = content
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|line| line.to_string())
            .collect();

        if tips.is_empty() {
            Ok(Self::default_tips())
        } else {
            Ok(tips)
        }
    }

    /// Get a random tip using simple randomization
    pub fn random_tip(&self) -> StorageResult<String> {
        let tips = self.load_all()?;
        if tips.is_empty() {
            return Err(StorageError::Internal("No tips available".to_string()));
        }
        // Simple random index using time-based seed
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        let index = (now.as_nanos() as usize) % tips.len();
        Ok(tips[index].clone())
    }

    /// Get tip by index (wraps around)
    pub fn get_tip(&self, index: usize) -> StorageResult<String> {
        let tips = self.load_all()?;
        if tips.is_empty() {
            return Err(StorageError::Internal("No tips available".to_string()));
        }
        Ok(tips[index % tips.len()].clone())
    }

    /// Default tips when no config file exists
    fn default_tips() -> Vec<String> {
        vec![
            "Use Tab to switch between agents".to_string(),
            "Press Ctrl+C to cancel a running operation".to_string(),
            "Use /help to see all available commands".to_string(),
            "Press Ctrl+L to clear the screen".to_string(),
            "Use @file to reference a file in your message".to_string(),
            "Use arrow keys to navigate message history".to_string(),
            "Press Esc to cancel input".to_string(),
            "Use /theme to change the color theme".to_string(),
            "Enable vim mode with --vim-mode flag".to_string(),
            "Use /session to manage your sessions".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_tips() {
        let tips = TipsLoader::default_tips();
        assert!(!tips.is_empty());
        assert!(tips.iter().any(|t| t.contains("Tab")));
    }
}
