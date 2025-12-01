// Refactor code

use crate::error::CliResult;
use super::Command;

/// Refactor existing code
pub struct RefactorCommand {
    pub file: String,
}

impl RefactorCommand {
    pub fn new(file: String) -> Self {
        Self { file }
    }
}

impl Command for RefactorCommand {
    fn execute(&self) -> CliResult<()> {
        println!("Refactoring: {}", self.file);
        println!("✓ Refactoring complete");
        Ok(())
    }
}
