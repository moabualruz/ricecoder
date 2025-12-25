//! Keybind customization with conflict detection and profile management
//!
//! This crate provides a comprehensive keybind system for ricecoder with:
//! - JSON and Markdown configuration parsing
//! - Keybind registry with fast lookup
//! - Conflict detection with resolution suggestions
//! - Profile management for switching between configurations
//! - Help system for displaying available keybinds
//! - Persistence layer for saving profiles across sessions

pub mod conflict;
pub mod di;
pub mod engine;
pub mod error;
pub mod help;
pub mod merge;
pub mod models;
pub mod parser;
pub mod persistence;
pub mod profile;
pub mod registry;

// Re-export public types
pub use conflict::{Conflict, ConflictDetector, Resolution};
pub use engine::{KeybindEngine, ValidationResult};
pub use error::{EngineError, ParseError, PersistenceError, ProfileError, RegistryError};
pub use help::{KeybindHelp, Page};
pub use merge::{KeybindMerger, MergeConflict, MergeResult};
pub use models::{Context, Key, KeyCombo, Keybind, KeybindManager, Modifier};
pub use parser::{JsonKeybindParser, KeybindParser, MarkdownKeybindParser, ParserRegistry};
pub use persistence::{FileSystemPersistence, KeybindPersistence};
pub use profile::{Profile, ProfileManager};
pub use registry::KeybindRegistry;
