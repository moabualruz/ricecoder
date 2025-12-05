// RiceCoder CLI Library

pub mod accessibility;
pub mod branding;
pub mod chat;
pub mod commands;
pub mod completion;
pub mod error;
pub mod logging;
pub mod output;
pub mod progress;
pub mod router;

pub use accessibility::{AccessibilityFeatures, AccessibilitySettings, KeyboardShortcuts};
pub use branding::{BrandingManager, TerminalCapabilities};
pub use error::{CliError, CliResult};
pub use logging::{init_logging, VerbosityLevel};
pub use router::{Cli, CommandRouter, Commands};
