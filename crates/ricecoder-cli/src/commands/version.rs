// Display version information

use crate::error::CliResult;
use crate::output::OutputStyle;
use super::Command;

/// Display version information
pub struct VersionCommand;

impl VersionCommand {
    pub fn new() -> Self {
        Self
    }

    /// Get version information
    fn get_version_info() -> String {
        format!(
            "RiceCoder v{}\n\nBuild Information:\n  Edition: 2021\n  Profile: {}\n  Rust: {}",
            env!("CARGO_PKG_VERSION"),
            if cfg!(debug_assertions) { "debug" } else { "release" },
            env!("CARGO_PKG_RUST_VERSION")
        )
    }
}

impl Command for VersionCommand {
    fn execute(&self) -> CliResult<()> {
        let style = OutputStyle::default();
        println!("{}", style.header(&Self::get_version_info()));
        Ok(())
    }
}
