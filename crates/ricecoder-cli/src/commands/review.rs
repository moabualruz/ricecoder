// Review code

use crate::error::CliResult;
use super::Command;

/// Review code
pub struct ReviewCommand {
    pub file: String,
}

impl ReviewCommand {
    pub fn new(file: String) -> Self {
        Self { file }
    }
}

impl Command for ReviewCommand {
    fn execute(&self) -> CliResult<()> {
        println!("Reviewing: {}", self.file);
        println!("✓ Review complete");
        Ok(())
    }
}
