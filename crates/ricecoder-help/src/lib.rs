//! RiceCoder Help System
//!
//! This crate provides a comprehensive help system for RiceCoder TUI, including:
//! - Help dialog widget with scrollable content
//! - Search functionality (Ctrl+F)
//! - Organized help by categories (Getting Started, Commands, etc.)
//! - Keyboard navigation and escape to close
//!
//! # Examples
//!
//! ```ignore
//! use ricecoder_help::{HelpDialog, HelpContent, HelpCategory};
//!
//! // Create help content
//! let content = HelpContent::new()
//!     .add_category(HelpCategory::new("Getting Started")
//!         .add_item("Welcome", "Welcome to RiceCoder!")
//!         .add_item("Basic Usage", "Type your message and press Enter"))
//!     .add_category(HelpCategory::new("Commands")
//!         .add_item("/help", "Show this help dialog")
//!         .add_item("/exit", "Exit RiceCoder"));
//!
//! // Create help dialog
//! let mut dialog = HelpDialog::new(content);
//!
//! // Render in TUI
//! dialog.render(area, buf);
//! ```

pub mod content;
pub mod dialog;
pub mod error;
pub mod search;

pub use content::{HelpCategory, HelpContent, HelpItem, HelpSystem};
pub use dialog::HelpDialog;
pub use error::{HelpError, Result};
pub use search::HelpSearch;
