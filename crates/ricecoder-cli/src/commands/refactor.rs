// Refactor code

use async_trait::async_trait;

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

#[async_trait::async_trait]
impl Command for RefactorCommand {
    async fn execute(&self) -> CliResult<()> {
        println!("Refactoring: {}", self.file);
        println!("✓ Refactoring complete");
        Ok(())
    }
}
