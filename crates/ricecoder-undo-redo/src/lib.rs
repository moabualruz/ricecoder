#![warn(missing_docs)]

//! Undo/Redo System for ricecoder
//!
//! Provides comprehensive change tracking, history management, and rollback capabilities
//! for all file operations and code generation in ricecoder.

pub mod change;
pub mod checkpoint;
pub mod di;
pub mod error;
pub mod history;
pub mod models;
pub mod persistence;

// Re-export public API
pub use change::{Change, ChangeTracker, ChangeType};
pub use checkpoint::{Checkpoint, CheckpointManager};
pub use error::UndoRedoError;
pub use history::{FileChangeInfo, HistoryConfig, HistoryEntry, HistoryManager};
pub use models::ChangeValidator;
pub use persistence::{HistorySnapshot, HistoryStore, StorageManager, StorageStats};
