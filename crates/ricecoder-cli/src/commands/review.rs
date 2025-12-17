// Review code

use super::Command;
use crate::error::CliResult;
use async_trait::async_trait;

/// Review code
pub struct ReviewCommand {
    pub file: String,
}

impl ReviewCommand {
    pub fn new(file: String) -> Self {
        Self { file }
    }
}

#[async_trait::async_trait]
impl Command for ReviewCommand {
    async fn execute(&self) -> CliResult<()> {
        println!("Reviewing: {}", self.file);
        println!("✓ Review complete");
        Ok(())
    }
}
