// Command handlers for ricecoder CLI

pub mod chat;
pub mod config;
pub mod custom;
pub mod custom_storage;
pub mod gen;
pub mod help;
pub mod hooks;
pub mod init;
pub mod lsp;
pub mod refactor;
pub mod review;
pub mod sessions;
pub mod tui;
pub mod version;

pub use chat::ChatCommand;
pub use config::ConfigCommand;
pub use custom::{CustomAction, CustomCommandHandler};
pub use gen::GenCommand;
pub use help::HelpCommand;
pub use hooks::{HooksAction, HooksCommand};
pub use init::InitCommand;
pub use lsp::LspCommand;
pub use refactor::RefactorCommand;
pub use review::ReviewCommand;
pub use sessions::{SessionsAction, SessionsCommand};
pub use tui::TuiCommand;
pub use version::VersionCommand;

use crate::error::CliResult;

/// Trait for command handlers
pub trait Command: Send + Sync {
    /// Execute the command
    fn execute(&self) -> CliResult<()>;
}
