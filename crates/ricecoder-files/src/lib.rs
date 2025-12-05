#![warn(missing_docs)]

//! File management module for ricecoder
//!
//! Provides safe file operations with backups, rollback support, atomic writes,
//! and comprehensive audit logging for all file changes.

pub mod audit;
pub mod backup;
pub mod conflict;
pub mod diff;
pub mod error;
pub mod git;
pub mod manager;
pub mod models;
pub mod transaction;
pub mod verifier;
pub mod writer;

// Re-export public API
pub use audit::AuditLogger;
pub use backup::BackupManager;
pub use conflict::ConflictResolver;
pub use diff::DiffEngine;
pub use error::FileError;
pub use git::GitIntegration;
pub use manager::FileManager;
pub use models::{
    AuditEntry, BackupMetadata, ConflictInfo, ConflictResolution, DiffHunk, DiffLine, DiffStats,
    FileDiff, FileOperation, FileTransaction, GitStatus, OperationType, TransactionStatus,
};
pub use transaction::TransactionManager;
pub use verifier::ContentVerifier;
pub use writer::SafeWriter;
