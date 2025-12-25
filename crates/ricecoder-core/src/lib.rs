//! RiceCoder Core - Shared logging and utilities
//!
//! Provides unified logging system matching OpenCode behavior.

pub mod logging;

// Re-export commonly used types
pub use logging::{LogLevel, LogOptions, Logger, create as create_logger, format_error, init as init_logging};
