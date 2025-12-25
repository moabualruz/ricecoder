//! FileSystemRepository Implementation
//!
//! Implements the `FileRepository` port from ricecoder-domain using the
//! standard library's file system operations.
//!
//! File System Repository

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use async_trait::async_trait;
use ricecoder_domain::{
    DomainError, DomainResult, FileManager, FileMetadata, FileReader, FileWriter, WriteResult,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::backup::BackupManager;
use crate::error::FileError;

/// Filesystem-based implementation of `FileRepository`
///
/// Provides safe file operations with optional backup support.
///
/// # Example
///
/// ```ignore
/// use ricecoder_files::FileSystemRepository;
/// use ricecoder_domain::FileRepository;
///
/// let repo = FileSystemRepository::new();
/// let content = repo.read_string(&PathBuf::from("file.txt")).await?;
/// ```
#[derive(Debug, Clone)]
pub struct FileSystemRepository {
    /// Directory for storing backups
    backup_dir: Option<PathBuf>,
}

impl FileSystemRepository {
    /// Create a new FileSystemRepository without backup support
    pub fn new() -> Self {
        Self { backup_dir: None }
    }

    /// Create a new FileSystemRepository with backup support
    pub fn with_backup_dir(backup_dir: PathBuf) -> Self {
        Self {
            backup_dir: Some(backup_dir),
        }
    }

    /// Convert FileError to DomainError
    fn to_domain_error(err: FileError) -> DomainError {
        DomainError::FileOperationError {
            operation: "file".to_string(),
            reason: err.to_string(),
        }
    }

    /// Convert std::io::Error to DomainError
    fn io_to_domain_error(err: std::io::Error, context: &str) -> DomainError {
        DomainError::IoError {
            reason: format!("{}: {}", context, err),
        }
    }

    /// Create a backup of a file if backup_dir is set
    async fn create_backup(&self, path: &PathBuf) -> Option<PathBuf> {
        if let Some(ref backup_dir) = self.backup_dir {
            if path.exists() {
                let backup_manager = BackupManager::new(backup_dir.clone(), 10);
                match backup_manager.create_backup(path).await {
                    Ok(info) => Some(info.backup_path),
                    Err(_) => None,
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl Default for FileSystemRepository {
    fn default() -> Self {
        Self::new()
    }
}

/// ISP-compliant: FileReader implementation (read-only operations)
#[async_trait]
impl FileReader for FileSystemRepository {
    async fn read(&self, path: &PathBuf) -> DomainResult<Vec<u8>> {
        let mut file = tokio::fs::File::open(path)
            .await
            .map_err(|e| Self::io_to_domain_error(e, "Failed to open file"))?;

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .await
            .map_err(|e| Self::io_to_domain_error(e, "Failed to read file"))?;

        Ok(buffer)
    }

    async fn read_string(&self, path: &PathBuf) -> DomainResult<String> {
        let bytes = self.read(path).await?;
        String::from_utf8(bytes).map_err(|e| DomainError::ValidationError {
            field: "file_content".to_string(),
            reason: format!("File is not valid UTF-8: {}", e),
        })
    }

    async fn exists(&self, path: &PathBuf) -> DomainResult<bool> {
        Ok(tokio::fs::try_exists(path)
            .await
            .map_err(|e| Self::io_to_domain_error(e, "Failed to check file existence"))?)
    }

    async fn metadata(&self, path: &PathBuf) -> DomainResult<FileMetadata> {
        let meta = tokio::fs::metadata(path)
            .await
            .map_err(|e| Self::io_to_domain_error(e, "Failed to get metadata"))?;

        let modified = meta
            .modified()
            .ok()
            .map(|t| chrono::DateTime::<chrono::Utc>::from(t));

        let created = meta
            .created()
            .ok()
            .map(|t| chrono::DateTime::<chrono::Utc>::from(t));

        Ok(FileMetadata {
            path: path.clone(),
            size: meta.len(),
            is_directory: meta.is_dir(),
            modified,
            created,
            is_readonly: meta.permissions().readonly(),
        })
    }

    async fn list_directory(&self, path: &PathBuf) -> DomainResult<Vec<FileMetadata>> {
        self.list_directory_with_ignore(path, &[]).await
    }
}

impl FileSystemRepository {
    /// List directory contents with ignore pattern support
    ///
    /// # Arguments
    /// * `path` - Directory path to list
    /// * `ignore_patterns` - Glob patterns to ignore (e.g., ["node_modules", "*.log", ".git"])
    ///
    /// # Returns
    /// Vector of FileMetadata for non-ignored entries
    pub async fn list_directory_with_ignore(
        &self,
        path: &PathBuf,
        ignore_patterns: &[&str],
    ) -> DomainResult<Vec<FileMetadata>> {
        let mut entries = Vec::new();
        let mut dir = tokio::fs::read_dir(path)
            .await
            .map_err(|e| Self::io_to_domain_error(e, "Failed to read directory"))?;

        // Compile glob patterns
        let patterns: Vec<glob::Pattern> = ignore_patterns
            .iter()
            .filter_map(|p| glob::Pattern::new(p).ok())
            .collect();

        while let Some(entry) = dir
            .next_entry()
            .await
            .map_err(|e| Self::io_to_domain_error(e, "Failed to read directory entry"))?
        {
            let entry_path = entry.path();
            let file_name = entry_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            // Check if entry matches any ignore pattern
            let should_ignore = patterns.iter().any(|pattern| pattern.matches(file_name));

            if should_ignore {
                continue;
            }

            match self.metadata(&entry_path).await {
                Ok(meta) => entries.push(meta),
                Err(_) => continue, // Skip entries we can't read
            }
        }

        Ok(entries)
    }

    /// List directory with OpenCode-compatible tree rendering
    ///
    /// Matches OpenCode ls.ts L8-L109:
    /// - Built-in ignore patterns (node_modules, .git, etc.)
    /// - Recursive enumeration
    /// - 100 file limit with truncated flag
    /// - Tree structure output (dirs first, sorted)
    /// - Title relative to worktree
    ///
    /// # Arguments
    /// * `path` - Directory to list (absolute or relative)
    /// * `ignore` - Additional ignore patterns (beyond built-in)
    /// * `worktree` - Worktree root for relative title
    ///
    /// # Returns
    /// `ListResult` with tree output, count, truncated flag, and title
    pub async fn list_directory_tree(
        &self,
        path: &PathBuf,
        ignore: &[String],
        worktree: Option<&PathBuf>,
    ) -> DomainResult<ListResult> {
        // Built-in ignore patterns from OpenCode ls.ts L8-L33
        const IGNORE_PATTERNS: &[&str] = &[
            "node_modules/",
            "__pycache__/",
            ".git/",
            "dist/",
            "build/",
            "target/",
            "vendor/",
            "bin/",
            "obj/",
            ".idea/",
            ".vscode/",
            ".zig-cache/",
            "zig-out",
            ".coverage",
            "coverage/",
            "tmp/",
            "temp/",
            ".cache/",
            "cache/",
            "logs/",
            ".venv/",
            "venv/",
            "env/",
        ];

        const LIMIT: usize = 100;

        // Resolve search path (make absolute)
        let search_path = if path.is_absolute() {
            path.clone()
        } else {
            std::env::current_dir()
                .map_err(|e| Self::io_to_domain_error(e, "Failed to get current directory"))?
                .join(path)
        };

        // Build combined ignore patterns
        let mut ignore_globs: Vec<String> = IGNORE_PATTERNS
            .iter()
            .map(|p| format!("!{}*", p))
            .collect();
        ignore_globs.extend(ignore.iter().map(|p| format!("!{}", p)));

        // Recursively enumerate files (OpenCode uses Ripgrep.files)
        // For simplicity, use WalkDir or similar
        let files = self
            .walk_directory_recursive(&search_path, &ignore_globs, LIMIT)
            .await?;

        let truncated = files.len() >= LIMIT;

        // Build directory structure (OpenCode ls.ts L53-L70)
        let tree_output = self.build_tree_structure(&files, &search_path)?;

        // Compute title relative to worktree (OpenCode ls.ts L102)
        let title = if let Some(worktree_path) = worktree {
            search_path
                .strip_prefix(worktree_path)
                .unwrap_or(&search_path)
                .display()
                .to_string()
        } else {
            search_path.display().to_string()
        };

        Ok(ListResult {
            title,
            count: files.len(),
            truncated,
            output: tree_output,
        })
    }

    /// Recursively walk directory and collect files (respecting ignore patterns and limit)
    async fn walk_directory_recursive(
        &self,
        root: &PathBuf,
        ignore_globs: &[String],
        limit: usize,
    ) -> DomainResult<Vec<PathBuf>> {
        use ignore::WalkBuilder;

        let mut files = Vec::new();
        let walker = WalkBuilder::new(root)
            .hidden(false)
            .git_ignore(true)
            .build();

        for entry in walker {
            if files.len() >= limit {
                break;
            }

            let entry = entry.map_err(|e| DomainError::IoError {
                reason: format!("Walk error: {}", e),
            })?;

            if entry.path().is_file() {
                // Check if file matches ignore patterns
                let relative_path = entry
                    .path()
                    .strip_prefix(root)
                    .unwrap_or(entry.path())
                    .to_path_buf();

                // Skip if matches ignore globs (simplified check)
                let path_str = relative_path.display().to_string();
                let should_ignore = ignore_globs.iter().any(|glob| {
                    // Simple prefix check for now
                    glob.trim_start_matches('!').split('*').next().map_or(false, |prefix| path_str.starts_with(prefix))
                });

                if !should_ignore {
                    files.push(relative_path);
                }
            }
        }

        Ok(files)
    }

    /// Build tree structure from file list (OpenCode ls.ts L72-L97)
    fn build_tree_structure(
        &self,
        files: &[PathBuf],
        root: &PathBuf,
    ) -> DomainResult<String> {
        use std::collections::BTreeMap;
        use std::collections::BTreeSet;

        // Build dirs set and filesByDir map
        let mut dirs = BTreeSet::new();
        let mut files_by_dir: BTreeMap<PathBuf, Vec<String>> = BTreeMap::new();
        let default_parent = PathBuf::from(".");

        for file in files {
            let parent = file.parent().unwrap_or(&default_parent);
            
            // Add all parent directories
            let components: Vec<_> = file.components().collect();
            for i in 0..components.len() {
                let dir_path: PathBuf = components.iter().take(i).collect();
                dirs.insert(dir_path.clone());
            }

            // Add file to its directory
            files_by_dir
                .entry(parent.to_path_buf())
                .or_insert_with(Vec::new)
                .push(file.file_name().unwrap().to_string_lossy().to_string());
        }

        // Render tree
        let mut output = String::new();
        output.push_str(&format!("{}/\n", root.display()));
        output.push_str(&self.render_dir(
            &PathBuf::from("."),
            0,
            &dirs,
            &files_by_dir,
        ));

        Ok(output)
    }

    /// Recursively render directory tree (OpenCode ls.ts L72-L97)
    fn render_dir(
        &self,
        dir_path: &PathBuf,
        depth: usize,
        dirs: &std::collections::BTreeSet<PathBuf>,
        files_by_dir: &std::collections::BTreeMap<PathBuf, Vec<String>>,
    ) -> String {
        let indent = "  ".repeat(depth);
        let mut output = String::new();

        // Render directory name (skip for root)
        if depth > 0 {
            let dir_name = dir_path.file_name().map_or("", |n| n.to_str().unwrap_or(""));
            output.push_str(&format!("{}{}/\n", indent, dir_name));
        }

        let child_indent = "  ".repeat(depth + 1);

        // Render subdirectories first (sorted)
        let children: Vec<_> = dirs
            .iter()
            .filter(|d| {
                d.parent().map_or(false, |p| p == dir_path) && *d != dir_path
            })
            .collect();

        for child in children {
            output.push_str(&self.render_dir(child, depth + 1, dirs, files_by_dir));
        }

        // Render files (sorted)
        if let Some(file_names) = files_by_dir.get(dir_path) {
            let mut sorted_files = file_names.clone();
            sorted_files.sort();
            for file in sorted_files {
                output.push_str(&format!("{}{}\n", child_indent, file));
            }
        }

        output
    }
}

/// OpenCode-compatible list result (ls.ts L101-L108)
#[derive(Debug, Clone)]
pub struct ListResult {
    /// Title (path relative to worktree)
    pub title: String,
    /// Number of files found
    pub count: usize,
    /// Whether results were truncated at limit
    pub truncated: bool,
    /// Tree-structured output
    pub output: String,
}

/// ISP-compliant: FileWriter implementation (write operations)
#[async_trait]
impl FileWriter for FileSystemRepository {
    async fn write(
        &self,
        path: &PathBuf,
        content: &[u8],
        create_backup: bool,
    ) -> DomainResult<WriteResult> {
        // Create backup if requested
        let backup_path = if create_backup {
            self.create_backup(path).await
        } else {
            None
        };

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| Self::io_to_domain_error(e, "Failed to create parent directory"))?;
        }

        // Write file atomically (write to temp, then rename)
        let temp_path = path.with_extension("tmp");

        let mut file = tokio::fs::File::create(&temp_path)
            .await
            .map_err(|e| Self::io_to_domain_error(e, "Failed to create temp file"))?;

        file.write_all(content)
            .await
            .map_err(|e| Self::io_to_domain_error(e, "Failed to write content"))?;

        file.sync_all()
            .await
            .map_err(|e| Self::io_to_domain_error(e, "Failed to sync file"))?;

        // Rename temp file to target (atomic on most systems)
        tokio::fs::rename(&temp_path, path)
            .await
            .map_err(|e| Self::io_to_domain_error(e, "Failed to rename temp file"))?;

        Ok(WriteResult {
            path: path.clone(),
            bytes_written: content.len() as u64,
            backup_created: backup_path.is_some(),
            backup_path,
        })
    }

    async fn write_string(
        &self,
        path: &PathBuf,
        content: &str,
        create_backup: bool,
    ) -> DomainResult<WriteResult> {
        self.write(path, content.as_bytes(), create_backup).await
    }

    async fn delete(&self, path: &PathBuf) -> DomainResult<()> {
        let metadata = tokio::fs::metadata(path)
            .await
            .map_err(|e| Self::io_to_domain_error(e, "Failed to get file metadata"))?;

        if metadata.is_dir() {
            tokio::fs::remove_dir(path)
                .await
                .map_err(|e| Self::io_to_domain_error(e, "Failed to remove directory"))?;
        } else {
            tokio::fs::remove_file(path)
                .await
                .map_err(|e| Self::io_to_domain_error(e, "Failed to remove file"))?;
        }

        Ok(())
    }
}

/// ISP-compliant: FileManager implementation (file management operations)
#[async_trait]
impl FileManager for FileSystemRepository {
    async fn create_directory(&self, path: &PathBuf) -> DomainResult<()> {
        tokio::fs::create_dir_all(path)
            .await
            .map_err(|e| Self::io_to_domain_error(e, "Failed to create directory"))?;
        Ok(())
    }

    async fn copy(&self, from: &PathBuf, to: &PathBuf) -> DomainResult<u64> {
        tokio::fs::copy(from, to)
            .await
            .map_err(|e| Self::io_to_domain_error(e, "Failed to copy file"))
    }

    async fn rename(&self, from: &PathBuf, to: &PathBuf) -> DomainResult<()> {
        tokio::fs::rename(from, to)
            .await
            .map_err(|e| Self::io_to_domain_error(e, "Failed to rename file"))?;
        Ok(())
    }
}

// FileRepository is automatically implemented via blanket impl in ricecoder-domain
// for any T: FileReader + FileWriter + FileManager

// Tests for FileSystemRepository
// Note: Full integration tests are in tests/ directory. Unit tests here focus on isolated behavior.
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    #[tokio::test]
    async fn test_file_system_repository_read_write() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileSystemRepository::new();

        let file_path = temp_dir.path().join("test.txt");
        let content = "Hello, World!";

        // Write file
        let result = repo.write_string(&file_path, content, false).await;
        assert!(result.is_ok());
        let write_result = result.unwrap();
        assert_eq!(write_result.bytes_written, content.len() as u64);
        assert!(!write_result.backup_created);

        // Read file
        let read_content = repo.read_string(&file_path).await.unwrap();
        assert_eq!(read_content, content);
    }

    #[tokio::test]
    async fn test_file_system_repository_exists() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileSystemRepository::new();

        let file_path = temp_dir.path().join("test.txt");

        // File should not exist initially
        assert!(!repo.exists(&file_path).await.unwrap());

        // Create file
        fs::write(&file_path, "content").await.unwrap();

        // File should now exist
        assert!(repo.exists(&file_path).await.unwrap());
    }

    #[tokio::test]
    async fn test_file_system_repository_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileSystemRepository::new();

        let file_path = temp_dir.path().join("test.txt");
        let content = "Hello, World!";
        fs::write(&file_path, content).await.unwrap();

        let metadata = repo.metadata(&file_path).await.unwrap();
        assert_eq!(metadata.size, content.len() as u64);
        assert!(!metadata.is_directory);
        assert!(!metadata.is_readonly);
    }

    #[tokio::test]
    async fn test_file_system_repository_list_directory() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileSystemRepository::new();

        // Create test files
        fs::write(temp_dir.path().join("file1.txt"), "content1")
            .await
            .unwrap();
        fs::write(temp_dir.path().join("file2.txt"), "content2")
            .await
            .unwrap();

        let entries = repo.list_directory(&temp_dir.path().to_path_buf()).await.unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[tokio::test]
    async fn test_file_system_repository_delete() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileSystemRepository::new();

        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "content").await.unwrap();

        // File should exist
        assert!(repo.exists(&file_path).await.unwrap());

        // Delete file
        repo.delete(&file_path).await.unwrap();

        // File should not exist
        assert!(!repo.exists(&file_path).await.unwrap());
    }

    #[tokio::test]
    async fn test_file_system_repository_with_backup() {
        let temp_dir = TempDir::new().unwrap();
        let backup_dir = temp_dir.path().join("backups");
        let repo = FileSystemRepository::with_backup_dir(backup_dir);

        let file_path = temp_dir.path().join("test.txt");
        let original_content = "Original content";
        fs::write(&file_path, original_content).await.unwrap();

        // Write with backup
        let result = repo.write_string(&file_path, "New content", true).await;
        assert!(result.is_ok());
        let write_result = result.unwrap();
        assert!(write_result.backup_created);
        assert!(write_result.backup_path.is_some());
    }

    #[tokio::test]
    async fn test_file_system_repository_create_directory() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileSystemRepository::new();

        let dir_path = temp_dir.path().join("subdir").join("nested");
        repo.create_directory(&dir_path).await.unwrap();

        assert!(dir_path.exists());
        assert!(dir_path.is_dir());
    }

    #[tokio::test]
    async fn test_file_system_repository_copy() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileSystemRepository::new();

        let source = temp_dir.path().join("source.txt");
        let dest = temp_dir.path().join("dest.txt");
        let content = "Copy me!";
        fs::write(&source, content).await.unwrap();

        let bytes_copied = repo.copy(&source, &dest).await.unwrap();
        assert_eq!(bytes_copied, content.len() as u64);

        let dest_content = fs::read_to_string(&dest).await.unwrap();
        assert_eq!(dest_content, content);
    }

    #[tokio::test]
    async fn test_file_system_repository_rename() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileSystemRepository::new();

        let source = temp_dir.path().join("source.txt");
        let dest = temp_dir.path().join("dest.txt");
        let content = "Rename me!";
        fs::write(&source, content).await.unwrap();

        repo.rename(&source, &dest).await.unwrap();

        assert!(!source.exists());
        assert!(dest.exists());

        let dest_content = fs::read_to_string(&dest).await.unwrap();
        assert_eq!(dest_content, content);
    }
}
