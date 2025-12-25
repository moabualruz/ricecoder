//! File operations port interfaces and value objects
//!
//! File System Repository
//!
//! This module contains the contracts for file system operations.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::errors::*;

// ============================================================================
// File Operations Value Objects
// ============================================================================

/// File metadata returned from file operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    /// File path
    pub path: PathBuf,
    /// File size in bytes
    pub size: u64,
    /// Whether the file is a directory
    pub is_directory: bool,
    /// Last modification time
    pub modified: Option<chrono::DateTime<chrono::Utc>>,
    /// Creation time
    pub created: Option<chrono::DateTime<chrono::Utc>>,
    /// Whether the file is read-only
    pub is_readonly: bool,
}

/// Result of a file write operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteResult {
    /// Path where the file was written
    pub path: PathBuf,
    /// Number of bytes written
    pub bytes_written: u64,
    /// Whether a backup was created
    pub backup_created: bool,
    /// Path to backup file (if created)
    pub backup_path: Option<PathBuf>,
}

// ============================================================================
// File Repository Ports (ISP-Compliant Split)
// ============================================================================

/// Read-only file operations (ISP: 5 methods max)
///
///  File System Repository
#[async_trait]
pub trait FileReader: Send + Sync {
    /// Read file contents as bytes
    async fn read(&self, path: &PathBuf) -> DomainResult<Vec<u8>>;

    /// Read file contents as UTF-8 string
    async fn read_string(&self, path: &PathBuf) -> DomainResult<String>;

    /// Check if a file or directory exists
    async fn exists(&self, path: &PathBuf) -> DomainResult<bool>;

    /// Get file metadata
    async fn metadata(&self, path: &PathBuf) -> DomainResult<FileMetadata>;

    /// List files in a directory
    async fn list_directory(&self, path: &PathBuf) -> DomainResult<Vec<FileMetadata>>;
}

/// Write file operations (ISP: 3 methods)
///
///  File System Repository
#[async_trait]
pub trait FileWriter: Send + Sync {
    /// Write content to a file
    ///
    /// # Arguments
    /// * `path` - Target file path
    /// * `content` - Content to write
    /// * `create_backup` - Whether to create a backup before overwriting
    async fn write(&self, path: &PathBuf, content: &[u8], create_backup: bool) -> DomainResult<WriteResult>;

    /// Write string content to a file
    async fn write_string(&self, path: &PathBuf, content: &str, create_backup: bool) -> DomainResult<WriteResult>;

    /// Delete a file or empty directory
    async fn delete(&self, path: &PathBuf) -> DomainResult<()>;
}

/// File management operations (ISP: 3 methods)
///
///  File System Repository
#[async_trait]
pub trait FileManager: Send + Sync {
    /// Create a directory (including parents)
    async fn create_directory(&self, path: &PathBuf) -> DomainResult<()>;

    /// Copy a file
    async fn copy(&self, from: &PathBuf, to: &PathBuf) -> DomainResult<u64>;

    /// Move/rename a file
    async fn rename(&self, from: &PathBuf, to: &PathBuf) -> DomainResult<()>;
}

/// Combined file repository (Reader + Writer + Manager)
///
/// Clients that need full file operations can depend on this trait.
/// Clients with more focused needs should depend on role-specific traits:
/// - Read-only: `FileReader`
/// - Write-only: `FileWriter`
/// - Management: `FileManager`
///
///  File System Repository
pub trait FileRepository: FileReader + FileWriter + FileManager {}

/// Blanket implementation: Any type implementing all sub-traits gets FileRepository
impl<T: FileReader + FileWriter + FileManager> FileRepository for T {}
