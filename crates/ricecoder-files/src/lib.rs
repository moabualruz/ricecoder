#![warn(missing_docs)]

//! File management module for ricecoder
//!
//! Provides safe file operations with backups, rollback support, atomic writes,
//! and comprehensive audit logging for all file changes.

pub mod audit;
pub mod backup;
pub mod conflict;
pub mod di;
pub mod diff;
pub mod error;
pub mod file_repository;
pub mod git;
pub mod gitignore;
pub mod manager;
pub mod models;
pub mod ripgrep;
pub mod session_tracking;
pub mod transaction;
pub mod verifier;
pub mod watcher;
pub mod writer;

// Re-export public API
pub use audit::AuditLogger;
pub use backup::BackupManager;
pub use file_repository::FileSystemRepository;
pub use conflict::ConflictResolver;
pub use diff::DiffEngine;
pub use error::FileError;
pub use git::GitIntegration;
pub use gitignore::GitignoreFilter;
pub use manager::FileManager;
pub use models::{
    AuditEntry, BackupMetadata, ConflictInfo, ConflictResolution, DiffHunk, DiffLine, DiffStats,
    FileDiff, FileOperation, GitStatus, OperationType, TransactionStatus,
};
pub use ripgrep::{Ripgrep, RipgrepError, SearchMatch};
pub use session_tracking::{FileReadRecord, SessionFileTracker};
pub use transaction::TransactionManager;
pub use verifier::ContentVerifier;
pub use watcher::{FileChangeBatch, FileChangeEvent, FileWatcher, WatcherConfig};
pub use writer::SafeWriter;
