//! Gitignore and .ignore file parsing and filtering
//!
//! Provides gitignore pattern matching for directory listing and file operations
//! matching OpenCode's ignore.ts functionality.

use std::path::{Path, PathBuf};
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use tracing::{debug, warn};

use crate::error::FileError;

/// Gitignore-based file filtering
pub struct GitignoreFilter {
    gitignore: Option<Gitignore>,
}

impl GitignoreFilter {
    /// Create a new GitignoreFilter
    pub fn new() -> Self {
        Self { gitignore: None }
    }

    /// Load gitignore patterns from a directory
    ///
    /// Matches OpenCode File.list() behavior - loads both .gitignore and .ignore
    pub fn load_from_directory(worktree: &Path) -> Result<Self, FileError> {
        let mut builder = GitignoreBuilder::new(worktree);

        // Load .gitignore
        let gitignore_path = worktree.join(".gitignore");
        if gitignore_path.exists() {
            if let Some(e) = builder.add(&gitignore_path) {
                warn!("Failed to load .gitignore: {}", e);
            } else {
                debug!("Loaded .gitignore from {}", gitignore_path.display());
            }
        }

        // Load .ignore
        let ignore_path = worktree.join(".ignore");
        if ignore_path.exists() {
            if let Some(e) = builder.add(&ignore_path) {
                warn!("Failed to load .ignore: {}", e);
            } else {
                debug!("Loaded .ignore from {}", ignore_path.display());
            }
        }

        let gitignore = builder.build().ok();

        Ok(Self { gitignore })
    }

    /// Check if a path should be ignored
    ///
    /// Returns true if the path matches ignore patterns
    pub fn should_ignore(&self, path: &Path, is_dir: bool) -> bool {
        if let Some(ref gi) = self.gitignore {
            gi.matched(path, is_dir).is_ignore()
        } else {
            false
        }
    }

    /// Check if filter is loaded
    pub fn is_loaded(&self) -> bool {
        self.gitignore.is_some()
    }
}

impl Default for GitignoreFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_gitignore_filter_new() {
        let filter = GitignoreFilter::new();
        assert!(!filter.is_loaded());
    }

    #[test]
    fn test_gitignore_filter_load() {
        let temp_dir = TempDir::new().unwrap();

        // Create .gitignore file
        let gitignore_path = temp_dir.path().join(".gitignore");
        fs::write(&gitignore_path, "node_modules/\n*.log\n").unwrap();

        let filter = GitignoreFilter::load_from_directory(temp_dir.path()).unwrap();
        assert!(filter.is_loaded());

        // Test ignored paths
        assert!(filter.should_ignore(Path::new("node_modules"), true));
        assert!(filter.should_ignore(Path::new("test.log"), false));
        assert!(!filter.should_ignore(Path::new("test.txt"), false));
    }

    #[test]
    fn test_gitignore_filter_no_files() {
        let temp_dir = TempDir::new().unwrap();
        let filter = GitignoreFilter::load_from_directory(temp_dir.path()).unwrap();
        
        // Without .gitignore/.ignore files, nothing should be ignored
        // (except what the builder adds by default)
        // For this test, we just verify it doesn't crash
        let _ = filter.should_ignore(Path::new("test.txt"), false);
    }
}
