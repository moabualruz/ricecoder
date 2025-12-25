//! Ricecoder PTY - Pseudo-terminal support for RiceCoder
//!
//! This crate provides cross-platform PTY (pseudo-terminal) support for RiceCoder,
//! enabling interactive shell sessions and terminal management.

pub mod domain;
pub mod error;

// Re-export commonly used types
pub use domain::{PtyConfig, PtySession};
pub use error::PtyError;
