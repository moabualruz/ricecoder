//! File change detection for cache invalidation

use crate::error::ResearchError;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Tracks file modifications for cache invalidation
#[derive(Debug, Clone)]
pub struct ChangeDetector {
    /// Map of file paths to their last known modification times
    file_mtimes: HashMap<PathBuf, SystemTime>,
}

impl ChangeDetector {
    /// Create a new change detector
    pub fn new() -> Self {
        Self {
            file_mtimes: HashMap::new(),
        }
    }

    /// Record the current modification times of files in a directory
    pub fn record_mtimes(&mut self, root: &Path) -> Result<(), ResearchError> {
        self.file_mtimes.clear();
        self.scan_directory(root)?;
        Ok(())
    }

    /// Recursively scan a directory and record file modification times
    fn scan_directory(&mut self, path: &Path) -> Result<(), ResearchError> {
        if !path.exists() {
            return Err(ResearchError::ProjectNotFound {
                path: path.to_path_buf(),
                reason: "Directory does not exist".to_string(),
            });
        }

        for entry in std::fs::read_dir(path).map_err(|e| {
            ResearchError::IoError {
                reason: format!("Failed to read directory {}: {}", path.display(), e),
            }
        })? {
            let entry = entry.map_err(|e| {
                ResearchError::IoError {
                    reason: format!("Failed to read directory entry: {}", e),
                }
            })?;

            let path = entry.path();

            // Skip hidden files and common ignore patterns
            if self.should_skip(&path) {
                continue;
            }

            if path.is_dir() {
                self.scan_directory(&path)?;
            } else {
                // Record file modification time
                if let Ok(metadata) = std::fs::metadata(&path) {
                    if let Ok(modified) = metadata.modified() {
                        self.file_mtimes.insert(path, modified);
                    }
                }
            }
        }

        Ok(())
    }

    /// Check if a path should be skipped during scanning
    pub fn should_skip(&self, path: &Path) -> bool {
        // Skip hidden files and directories
        if let Some(file_name) = path.file_name() {
            if let Some(name_str) = file_name.to_str() {
                if name_str.starts_with('.') {
                    return true;
                }
            }
        }

        // Skip common directories
        if let Some(file_name) = path.file_name() {
            if let Some(_name_str @ ("node_modules" | "target" | ".git" | ".venv" | "venv" | "__pycache__"
                    | ".pytest_cache" | "dist" | "build")) = file_name.to_str() {
                return true;
            }
        }

        false
    }

    /// Detect changes since the last recording
    pub fn detect_changes(&self, root: &Path) -> Result<ChangeDetection, ResearchError> {
        let mut current_mtimes = HashMap::new();
        self.collect_mtimes(root, &mut current_mtimes)?;

        let mut added = Vec::new();
        let mut modified = Vec::new();
        let mut deleted = Vec::new();

        // Find added and modified files
        for (path, current_mtime) in &current_mtimes {
            match self.file_mtimes.get(path) {
                None => added.push(path.clone()),
                Some(recorded_mtime) if current_mtime > recorded_mtime => {
                    modified.push(path.clone());
                }
                _ => {}
            }
        }

        // Find deleted files
        for path in self.file_mtimes.keys() {
            if !current_mtimes.contains_key(path) {
                deleted.push(path.clone());
            }
        }

        let has_changes = !added.is_empty() || !modified.is_empty() || !deleted.is_empty();

        Ok(ChangeDetection {
            added,
            modified,
            deleted,
            has_changes,
        })
    }

    /// Collect modification times from a directory
    fn collect_mtimes(
        &self,
        path: &Path,
        mtimes: &mut HashMap<PathBuf, SystemTime>,
    ) -> Result<(), ResearchError> {
        if !path.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(path).map_err(|e| {
            ResearchError::IoError {
                reason: format!("Failed to read directory {}: {}", path.display(), e),
            }
        })? {
            let entry = entry.map_err(|e| {
                ResearchError::IoError {
                    reason: format!("Failed to read directory entry: {}", e),
                }
            })?;

            let path = entry.path();

            if self.should_skip(&path) {
                continue;
            }

            if path.is_dir() {
                self.collect_mtimes(&path, mtimes)?;
            } else if let Ok(metadata) = std::fs::metadata(&path) {
                if let Ok(modified) = metadata.modified() {
                    mtimes.insert(path, modified);
                }
            }
        }

        Ok(())
    }

    /// Get the recorded modification times
    pub fn recorded_mtimes(&self) -> &HashMap<PathBuf, SystemTime> {
        &self.file_mtimes
    }

    /// Get the number of tracked files
    pub fn tracked_file_count(&self) -> usize {
        self.file_mtimes.len()
    }

    /// Clear all recorded modification times
    pub fn clear(&mut self) {
        self.file_mtimes.clear();
    }
}

impl Default for ChangeDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of change detection
#[derive(Debug, Clone)]
pub struct ChangeDetection {
    /// Files that were added
    pub added: Vec<PathBuf>,
    /// Files that were modified
    pub modified: Vec<PathBuf>,
    /// Files that were deleted
    pub deleted: Vec<PathBuf>,
    /// Whether any changes were detected
    pub has_changes: bool,
}

impl ChangeDetection {
    /// Get all changed files (added + modified + deleted)
    pub fn all_changed(&self) -> Vec<PathBuf> {
        let mut all = self.added.clone();
        all.extend(self.modified.clone());
        all.extend(self.deleted.clone());
        all
    }

    /// Get the total number of changes
    pub fn change_count(&self) -> usize {
        self.added.len() + self.modified.len() + self.deleted.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_change_detector_creation() {
        let detector = ChangeDetector::new();
        assert_eq!(detector.tracked_file_count(), 0);
    }

    #[test]
    fn test_change_detector_skip_hidden_files() {
        let detector = ChangeDetector::new();
        assert!(detector.should_skip(Path::new(".hidden")));
        assert!(detector.should_skip(Path::new(".git")));
        assert!(detector.should_skip(Path::new("node_modules")));
        assert!(detector.should_skip(Path::new("target")));
        assert!(!detector.should_skip(Path::new("src")));
    }

    #[test]
    fn test_change_detection_no_changes() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "content")?;

        let mut detector = ChangeDetector::new();
        detector.record_mtimes(temp_dir.path())?;

        let changes = detector.detect_changes(temp_dir.path())?;
        assert!(!changes.has_changes);
        assert_eq!(changes.change_count(), 0);

        Ok(())
    }

    #[test]
    fn test_change_detection_added_file() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let test_file1 = temp_dir.path().join("test1.txt");
        fs::write(&test_file1, "content1")?;

        let mut detector = ChangeDetector::new();
        detector.record_mtimes(temp_dir.path())?;

        // Add a new file
        let test_file2 = temp_dir.path().join("test2.txt");
        fs::write(&test_file2, "content2")?;

        let changes = detector.detect_changes(temp_dir.path())?;
        assert!(changes.has_changes);
        assert_eq!(changes.added.len(), 1);
        assert_eq!(changes.modified.len(), 0);
        assert_eq!(changes.deleted.len(), 0);

        Ok(())
    }

    #[test]
    fn test_change_detection_modified_file() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "content")?;

        let mut detector = ChangeDetector::new();
        detector.record_mtimes(temp_dir.path())?;

        // Modify the file (with a small delay to ensure mtime changes)
        std::thread::sleep(std::time::Duration::from_millis(10));
        fs::write(&test_file, "modified content")?;

        let changes = detector.detect_changes(temp_dir.path())?;
        assert!(changes.has_changes);
        assert_eq!(changes.added.len(), 0);
        assert_eq!(changes.modified.len(), 1);
        assert_eq!(changes.deleted.len(), 0);

        Ok(())
    }

    #[test]
    fn test_change_detection_deleted_file() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "content")?;

        let mut detector = ChangeDetector::new();
        detector.record_mtimes(temp_dir.path())?;

        // Delete the file
        fs::remove_file(&test_file)?;

        let changes = detector.detect_changes(temp_dir.path())?;
        assert!(changes.has_changes);
        assert_eq!(changes.added.len(), 0);
        assert_eq!(changes.modified.len(), 0);
        assert_eq!(changes.deleted.len(), 1);

        Ok(())
    }

    #[test]
    fn test_change_detection_all_changed() {
        let change_detection = ChangeDetection {
            added: vec![PathBuf::from("added.txt")],
            modified: vec![PathBuf::from("modified.txt")],
            deleted: vec![PathBuf::from("deleted.txt")],
            has_changes: true,
        };

        let all_changed = change_detection.all_changed();
        assert_eq!(all_changed.len(), 3);
        assert_eq!(change_detection.change_count(), 3);
    }

    #[test]
    fn test_change_detector_clear() {
        let mut detector = ChangeDetector::new();
        detector.file_mtimes.insert(PathBuf::from("test.txt"), SystemTime::now());
        assert_eq!(detector.tracked_file_count(), 1);

        detector.clear();
        assert_eq!(detector.tracked_file_count(), 0);
    }
}
