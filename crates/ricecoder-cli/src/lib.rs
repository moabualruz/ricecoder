// RiceCoder CLI Library

pub mod error;
pub mod commands;
pub mod router;
pub mod output;
pub mod completion;
pub mod progress;
pub mod chat;
pub mod logging;
pub mod branding;

pub use error::{CliError, CliResult};
pub use router::{Cli, CommandRouter, Commands};
pub use logging::{VerbosityLevel, init_logging};
pub use branding::{BrandingManager, TerminalCapabilities};
