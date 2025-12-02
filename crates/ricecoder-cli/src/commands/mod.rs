// Command handlers for ricecoder CLI

pub mod init;
pub mod gen;
pub mod chat;
pub mod config;
pub mod refactor;
pub mod review;
pub mod version;
pub mod custom;
pub mod custom_storage;
pub mod tui;
pub mod sessions;

pub use init::InitCommand;
pub use gen::GenCommand;
pub use chat::ChatCommand;
pub use config::ConfigCommand;
pub use refactor::RefactorCommand;
pub use review::ReviewCommand;
pub use version::VersionCommand;
pub use custom::{
    CustomCommandHandler, CustomAction,
};
pub use tui::TuiCommand;
pub use sessions::{SessionsCommand, SessionsAction};

use crate::error::CliResult;

/// Trait for command handlers
pub trait Command: Send + Sync {
    /// Execute the command
    fn execute(&self) -> CliResult<()>;
}
