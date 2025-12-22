// Display version information

use async_trait::async_trait;

use super::Command;
use crate::{error::CliResult, output::OutputStyle};

/// Display version information
#[derive(Default)]
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
            if cfg!(debug_assertions) {
                "debug"
            } else {
                "release"
            },
            env!("CARGO_PKG_RUST_VERSION")
        )
    }
}
#[async_trait::async_trait]
impl Command for VersionCommand {
    async fn execute(&self) -> CliResult<()> {
        let style = OutputStyle::default();
        println!("{}", style.header(&Self::get_version_info()));
        Ok(())
    }
}
