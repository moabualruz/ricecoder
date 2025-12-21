// RiceCoder CLI Library

pub mod accessibility;
pub mod async_optimization;
pub mod branding;
pub mod chat;
pub mod commands;
pub mod completion;
pub mod di;
pub mod error;
pub mod lifecycle;
pub mod logging;
pub mod output;
pub mod progress;
pub mod router;
pub mod sync_utils;

pub use accessibility::{AccessibilityFeatures, AccessibilitySettings, KeyboardShortcuts};
pub use branding::{BrandingManager, TerminalCapabilities};
pub use error::{CliError, CliResult};
pub use logging::{init_logging, VerbosityLevel};
pub use router::{Cli, CommandRouter, Commands};
pub use sync_utils::{
    safe_lock, safe_lock_optional, safe_lock_or_default, SafeLockable, SyncError, SyncResult,
};
