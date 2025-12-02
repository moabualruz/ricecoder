#![warn(missing_docs)]

//! File management module for ricecoder
//!
//! Provides safe file operations with backups, rollback support, atomic writes,
//! and comprehensive audit logging for all file changes.

pub mod error;
pub mod models;
pub mod manager;
pub mod verifier;
pub mod conflict;
pub mod writer;
pub mod backup;
pub mod audit;
pub mod transaction;
pub mod diff;
pub mod git;

// Re-export public API
pub use error::FileError;
pub use models::{
    FileOperation, OperationType, ConflictInfo, ConflictResolution,
    FileTransaction, TransactionStatus,
    AuditEntry, BackupMetadata,
    FileDiff, DiffHunk, DiffLine, DiffStats,
    GitStatus,
};
pub use manager::FileManager;
pub use verifier::ContentVerifier;
pub use conflict::ConflictResolver;
pub use writer::SafeWriter;
pub use backup::BackupManager;
pub use audit::AuditLogger;
pub use transaction::TransactionManager;
pub use diff::DiffEngine;
pub use git::GitIntegration;
