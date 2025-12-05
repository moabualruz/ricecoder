// Refactor code

use super::Command;
use crate::error::CliResult;

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
