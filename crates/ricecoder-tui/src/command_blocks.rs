//! Command blocks widget for grouped command display

use std::time::{SystemTime, UNIX_EPOCH};

/// Command execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandStatus {
    /// Command is pending execution
    Pending,
    /// Command is currently executing
    Running,
    /// Command completed successfully
    Success,
    /// Command failed with error
    Failed,
    /// Command was cancelled
    Cancelled,
}

impl CommandStatus {
    /// Get display text for the status
    pub fn display_text(&self) -> &'static str {
        match self {
            CommandStatus::Pending => "⏳ Pending",
            CommandStatus::Running => "⚙️  Running",
            CommandStatus::Success => "✓ Success",
            CommandStatus::Failed => "✗ Failed",
            CommandStatus::Cancelled => "⊘ Cancelled",
        }
    }

    /// Get short display text
    pub fn short_text(&self) -> &'static str {
        match self {
            CommandStatus::Pending => "pending",
            CommandStatus::Running => "running",
            CommandStatus::Success => "success",
            CommandStatus::Failed => "failed",
            CommandStatus::Cancelled => "cancelled",
        }
    }
}

/// A single command in a block
#[derive(Debug, Clone)]
pub struct Command {
    /// Command text
    pub text: String,
    /// Command status
    pub status: CommandStatus,
    /// Command output
    pub output: String,
    /// Exit code (if completed)
    pub exit_code: Option<i32>,
    /// Timestamp when command was created
    pub created_at: u64,
    /// Timestamp when command started
    pub started_at: Option<u64>,
    /// Timestamp when command finished
    pub finished_at: Option<u64>,
}

impl Command {
    /// Create a new command
    pub fn new(text: impl Into<String>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            text: text.into(),
            status: CommandStatus::Pending,
            output: String::new(),
            exit_code: None,
            created_at: now,
            started_at: None,
            finished_at: None,
        }
    }

    /// Start executing the command
    pub fn start(&mut self) {
        self.status = CommandStatus::Running;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.started_at = Some(now);
    }

    /// Append output to the command
    pub fn append_output(&mut self, output: &str) {
        self.output.push_str(output);
    }

    /// Mark command as completed
    pub fn complete(&mut self, exit_code: i32) {
        self.status = if exit_code == 0 {
            CommandStatus::Success
        } else {
            CommandStatus::Failed
        };
        self.exit_code = Some(exit_code);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.finished_at = Some(now);
    }

    /// Mark command as cancelled
    pub fn cancel(&mut self) {
        self.status = CommandStatus::Cancelled;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.finished_at = Some(now);
    }

    /// Get the duration of command execution in seconds
    pub fn duration_secs(&self) -> Option<u64> {
        match (self.started_at, self.finished_at) {
            (Some(start), Some(end)) => Some(end - start),
            (Some(start), None) => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                Some(now - start)
            }
            _ => None,
        }
    }

    /// Check if command is complete
    pub fn is_complete(&self) -> bool {
        matches!(
            self.status,
            CommandStatus::Success | CommandStatus::Failed | CommandStatus::Cancelled
        )
    }
}

/// A block of related commands
#[derive(Debug, Clone)]
pub struct CommandBlock {
    /// Block ID
    pub id: String,
    /// Block title/description
    pub title: String,
    /// Commands in the block
    pub commands: Vec<Command>,
    /// Whether block is collapsed
    pub collapsed: bool,
    /// Selected command index
    pub selected_command: Option<usize>,
    /// Block creation timestamp
    pub created_at: u64,
}

impl CommandBlock {
    /// Create a new command block
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            id: id.into(),
            title: title.into(),
            commands: Vec::new(),
            collapsed: false,
            selected_command: None,
            created_at: now,
        }
    }

    /// Add a command to the block
    pub fn add_command(&mut self, command: Command) {
        self.commands.push(command);
    }

    /// Get the overall status of the block
    pub fn overall_status(&self) -> CommandStatus {
        if self.commands.is_empty() {
            return CommandStatus::Pending;
        }

        // If any command is running, block is running
        if self.commands.iter().any(|c| c.status == CommandStatus::Running) {
            return CommandStatus::Running;
        }

        // If any command failed, block failed
        if self.commands.iter().any(|c| c.status == CommandStatus::Failed) {
            return CommandStatus::Failed;
        }

        // If any command is pending, block is pending
        if self.commands.iter().any(|c| c.status == CommandStatus::Pending) {
            return CommandStatus::Pending;
        }

        // If any command is cancelled, block is cancelled
        if self.commands.iter().any(|c| c.status == CommandStatus::Cancelled) {
            return CommandStatus::Cancelled;
        }

        // All commands succeeded
        CommandStatus::Success
    }

    /// Toggle collapsed state
    pub fn toggle_collapsed(&mut self) {
        self.collapsed = !self.collapsed;
    }

    /// Get visible commands
    pub fn visible_commands(&self) -> Vec<&Command> {
        if self.collapsed {
            Vec::new()
        } else {
            self.commands.iter().collect()
        }
    }

    /// Select next command
    pub fn select_next_command(&mut self) {
        match self.selected_command {
            None => {
                if !self.commands.is_empty() {
                    self.selected_command = Some(0);
                }
            }
            Some(idx) if idx < self.commands.len() - 1 => {
                self.selected_command = Some(idx + 1);
            }
            _ => {}
        }
    }

    /// Select previous command
    pub fn select_prev_command(&mut self) {
        match self.selected_command {
            Some(idx) if idx > 0 => {
                self.selected_command = Some(idx - 1);
            }
            Some(0) => {
                self.selected_command = None;
            }
            _ => {}
        }
    }

    /// Get the selected command
    pub fn get_selected_command(&self) -> Option<&Command> {
        self.selected_command.and_then(|idx| self.commands.get(idx))
    }

    /// Get the selected command (mutable)
    pub fn get_selected_command_mut(&mut self) -> Option<&mut Command> {
        let idx = self.selected_command?;
        self.commands.get_mut(idx)
    }

    /// Get total duration of all commands
    pub fn total_duration_secs(&self) -> u64 {
        self.commands
            .iter()
            .filter_map(|c| c.duration_secs())
            .sum()
    }

    /// Get the number of successful commands
    pub fn success_count(&self) -> usize {
        self.commands
            .iter()
            .filter(|c| c.status == CommandStatus::Success)
            .count()
    }

    /// Get the number of failed commands
    pub fn failed_count(&self) -> usize {
        self.commands
            .iter()
            .filter(|c| c.status == CommandStatus::Failed)
            .count()
    }

    /// Get the number of running commands
    pub fn running_count(&self) -> usize {
        self.commands
            .iter()
            .filter(|c| c.status == CommandStatus::Running)
            .count()
    }
}

/// Copy action type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopyActionType {
    /// Copy entire block (command + output + status)
    Block,
    /// Copy command text only
    Command,
    /// Copy output only
    Output,
}

/// Command blocks widget
pub struct CommandBlocksWidget {
    /// Blocks in the widget
    pub blocks: Vec<CommandBlock>,
    /// Selected block index
    pub selected_block: Option<usize>,
    /// Scroll offset
    pub scroll: usize,
    /// Last copy action result
    pub last_copy_result: Option<Result<CopyActionType, String>>,
}

impl CommandBlocksWidget {
    /// Create a new command blocks widget
    pub fn new() -> Self {
        Self {
            blocks: Vec::new(),
            selected_block: None,
            scroll: 0,
            last_copy_result: None,
        }
    }

    /// Add a block
    pub fn add_block(&mut self, block: CommandBlock) {
        self.blocks.push(block);
    }

    /// Select next block
    pub fn select_next_block(&mut self) {
        match self.selected_block {
            None => {
                if !self.blocks.is_empty() {
                    self.selected_block = Some(0);
                }
            }
            Some(idx) if idx < self.blocks.len() - 1 => {
                self.selected_block = Some(idx + 1);
            }
            _ => {}
        }
    }

    /// Select previous block
    pub fn select_prev_block(&mut self) {
        match self.selected_block {
            Some(idx) if idx > 0 => {
                self.selected_block = Some(idx - 1);
            }
            Some(0) => {
                self.selected_block = None;
            }
            _ => {}
        }
    }

    /// Get the selected block
    pub fn get_selected_block(&self) -> Option<&CommandBlock> {
        self.selected_block.and_then(|idx| self.blocks.get(idx))
    }

    /// Get the selected block (mutable)
    pub fn get_selected_block_mut(&mut self) -> Option<&mut CommandBlock> {
        let idx = self.selected_block?;
        self.blocks.get_mut(idx)
    }

    /// Toggle selected block collapsed state
    pub fn toggle_selected_block_collapsed(&mut self) {
        if let Some(block) = self.get_selected_block_mut() {
            block.toggle_collapsed();
        }
    }

    /// Clear all blocks
    pub fn clear(&mut self) {
        self.blocks.clear();
        self.selected_block = None;
        self.scroll = 0;
    }

    /// Get total number of commands across all blocks
    pub fn total_commands(&self) -> usize {
        self.blocks.iter().map(|b| b.commands.len()).sum()
    }

    /// Get total number of successful commands
    pub fn total_success(&self) -> usize {
        self.blocks.iter().map(|b| b.success_count()).sum()
    }

    /// Get total number of failed commands
    pub fn total_failed(&self) -> usize {
        self.blocks.iter().map(|b| b.failed_count()).sum()
    }

    /// Get total number of running commands
    pub fn total_running(&self) -> usize {
        self.blocks.iter().map(|b| b.running_count()).sum()
    }

    /// Copy selected block content
    pub fn copy_selected_block(&mut self, action_type: CopyActionType) -> Result<(), String> {
        let block = self
            .get_selected_block()
            .ok_or_else(|| "No block selected".to_string())?;

        let cmd = block
            .get_selected_command()
            .ok_or_else(|| "No command selected in block".to_string())?;

        match action_type {
            CopyActionType::Block => {
                let content = format!(
                    "Command: {}\nStatus: {}\nOutput:\n{}",
                    cmd.text,
                    cmd.status.display_text(),
                    cmd.output
                );
                self.copy_to_clipboard(&content)?;
            }
            CopyActionType::Command => {
                self.copy_to_clipboard(&cmd.text)?;
            }
            CopyActionType::Output => {
                self.copy_to_clipboard(&cmd.output)?;
            }
        }

        self.last_copy_result = Some(Ok(action_type));
        Ok(())
    }

    /// Copy all blocks content
    pub fn copy_all_blocks(&mut self) -> Result<(), String> {
        let mut content = String::new();

        for (block_idx, block) in self.blocks.iter().enumerate() {
            if block_idx > 0 {
                content.push_str("\n\n");
            }

            content.push_str(&format!("=== Block: {} ===\n", block.title));

            for cmd in &block.commands {
                content.push_str(&format!(
                    "Command: {}\nStatus: {}\nOutput:\n{}\n\n",
                    cmd.text,
                    cmd.status.display_text(),
                    cmd.output
                ));
            }
        }

        self.copy_to_clipboard(&content)?;
        self.last_copy_result = Some(Ok(CopyActionType::Block));
        Ok(())
    }

    /// Copy block by index
    pub fn copy_block_by_index(
        &mut self,
        block_idx: usize,
        action_type: CopyActionType,
    ) -> Result<(), String> {
        let block = self
            .blocks
            .get(block_idx)
            .ok_or_else(|| format!("Block index {} out of range", block_idx))?;

        match action_type {
            CopyActionType::Block => {
                let mut content = String::new();
                for cmd in &block.commands {
                    content.push_str(&format!(
                        "Command: {}\nStatus: {}\nOutput:\n{}\n\n",
                        cmd.text,
                        cmd.status.display_text(),
                        cmd.output
                    ));
                }
                self.copy_to_clipboard(&content)?;
            }
            CopyActionType::Command => {
                if let Some(cmd) = block.get_selected_command() {
                    self.copy_to_clipboard(&cmd.text)?;
                } else {
                    return Err("No command selected in block".to_string());
                }
            }
            CopyActionType::Output => {
                if let Some(cmd) = block.get_selected_command() {
                    self.copy_to_clipboard(&cmd.output)?;
                } else {
                    return Err("No command selected in block".to_string());
                }
            }
        }

        self.last_copy_result = Some(Ok(action_type));
        Ok(())
    }

    /// Internal method to copy text to clipboard
    fn copy_to_clipboard(&self, text: &str) -> Result<(), String> {
        // Check size limit (100 MB)
        const MAX_SIZE: usize = 100 * 1024 * 1024;
        if text.len() > MAX_SIZE {
            return Err(format!(
                "Content too large to copy: {} bytes (max: {} bytes)",
                text.len(),
                MAX_SIZE
            ));
        }

        // In a real implementation, this would use the clipboard module
        // For now, we just validate the content
        Ok(())
    }

    /// Get the last copy result
    pub fn get_last_copy_result(&self) -> Option<&Result<CopyActionType, String>> {
        self.last_copy_result.as_ref()
    }

    /// Clear the last copy result
    pub fn clear_copy_result(&mut self) {
        self.last_copy_result = None;
    }
}

impl Default for CommandBlocksWidget {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_creation() {
        let cmd = Command::new("echo hello");
        assert_eq!(cmd.text, "echo hello");
        assert_eq!(cmd.status, CommandStatus::Pending);
        assert!(cmd.output.is_empty());
        assert_eq!(cmd.exit_code, None);
    }

    #[test]
    fn test_command_lifecycle() {
        let mut cmd = Command::new("echo hello");
        assert_eq!(cmd.status, CommandStatus::Pending);

        cmd.start();
        assert_eq!(cmd.status, CommandStatus::Running);
        assert!(cmd.started_at.is_some());

        cmd.append_output("hello\n");
        assert_eq!(cmd.output, "hello\n");

        cmd.complete(0);
        assert_eq!(cmd.status, CommandStatus::Success);
        assert_eq!(cmd.exit_code, Some(0));
        assert!(cmd.finished_at.is_some());
    }

    #[test]
    fn test_command_failure() {
        let mut cmd = Command::new("false");
        cmd.start();
        cmd.complete(1);
        assert_eq!(cmd.status, CommandStatus::Failed);
        assert_eq!(cmd.exit_code, Some(1));
    }

    #[test]
    fn test_command_cancellation() {
        let mut cmd = Command::new("sleep 10");
        cmd.start();
        cmd.cancel();
        assert_eq!(cmd.status, CommandStatus::Cancelled);
        assert!(cmd.finished_at.is_some());
    }

    #[test]
    fn test_command_block_creation() {
        let block = CommandBlock::new("block1", "Build commands");
        assert_eq!(block.id, "block1");
        assert_eq!(block.title, "Build commands");
        assert!(block.commands.is_empty());
        assert!(!block.collapsed);
    }

    #[test]
    fn test_command_block_add_commands() {
        let mut block = CommandBlock::new("block1", "Build");
        let cmd1 = Command::new("cargo build");
        let cmd2 = Command::new("cargo test");

        block.add_command(cmd1);
        block.add_command(cmd2);

        assert_eq!(block.commands.len(), 2);
    }

    #[test]
    fn test_command_block_overall_status() {
        let mut block = CommandBlock::new("block1", "Build");

        // Empty block is pending
        assert_eq!(block.overall_status(), CommandStatus::Pending);

        // Add pending command
        let cmd1 = Command::new("cargo build");
        block.add_command(cmd1);
        assert_eq!(block.overall_status(), CommandStatus::Pending);

        // Start command
        if let Some(cmd) = block.commands.get_mut(0) {
            cmd.start();
        }
        assert_eq!(block.overall_status(), CommandStatus::Running);

        // Complete successfully
        if let Some(cmd) = block.commands.get_mut(0) {
            cmd.complete(0);
        }
        assert_eq!(block.overall_status(), CommandStatus::Success);
    }

    #[test]
    fn test_command_block_collapsed() {
        let mut block = CommandBlock::new("block1", "Build");
        let cmd = Command::new("cargo build");
        block.add_command(cmd);

        assert_eq!(block.visible_commands().len(), 1);

        block.toggle_collapsed();
        assert!(block.collapsed);
        assert_eq!(block.visible_commands().len(), 0);

        block.toggle_collapsed();
        assert!(!block.collapsed);
        assert_eq!(block.visible_commands().len(), 1);
    }

    #[test]
    fn test_command_block_selection() {
        let mut block = CommandBlock::new("block1", "Build");
        block.add_command(Command::new("cmd1"));
        block.add_command(Command::new("cmd2"));

        assert_eq!(block.selected_command, None);

        block.select_next_command();
        assert_eq!(block.selected_command, Some(0));

        block.select_next_command();
        assert_eq!(block.selected_command, Some(1));

        block.select_prev_command();
        assert_eq!(block.selected_command, Some(0));

        block.select_prev_command();
        assert_eq!(block.selected_command, None);
    }

    #[test]
    fn test_command_blocks_widget() {
        let mut widget = CommandBlocksWidget::new();
        assert!(widget.blocks.is_empty());

        let block1 = CommandBlock::new("block1", "Build");
        let block2 = CommandBlock::new("block2", "Test");

        widget.add_block(block1);
        widget.add_block(block2);

        assert_eq!(widget.blocks.len(), 2);
    }

    #[test]
    fn test_command_blocks_widget_selection() {
        let mut widget = CommandBlocksWidget::new();
        widget.add_block(CommandBlock::new("block1", "Build"));
        widget.add_block(CommandBlock::new("block2", "Test"));

        assert_eq!(widget.selected_block, None);

        widget.select_next_block();
        assert_eq!(widget.selected_block, Some(0));

        widget.select_next_block();
        assert_eq!(widget.selected_block, Some(1));

        widget.select_prev_block();
        assert_eq!(widget.selected_block, Some(0));
    }

    #[test]
    fn test_command_blocks_widget_statistics() {
        let mut widget = CommandBlocksWidget::new();
        let mut block = CommandBlock::new("block1", "Build");

        let mut cmd1 = Command::new("cmd1");
        cmd1.complete(0);
        block.add_command(cmd1);

        let mut cmd2 = Command::new("cmd2");
        cmd2.complete(1);
        block.add_command(cmd2);

        widget.add_block(block);

        assert_eq!(widget.total_commands(), 2);
        assert_eq!(widget.total_success(), 1);
        assert_eq!(widget.total_failed(), 1);
    }

    #[test]
    fn test_command_status_display() {
        assert_eq!(CommandStatus::Pending.short_text(), "pending");
        assert_eq!(CommandStatus::Running.short_text(), "running");
        assert_eq!(CommandStatus::Success.short_text(), "success");
        assert_eq!(CommandStatus::Failed.short_text(), "failed");
        assert_eq!(CommandStatus::Cancelled.short_text(), "cancelled");
    }

    #[test]
    fn test_copy_action_type() {
        assert_eq!(CopyActionType::Block, CopyActionType::Block);
        assert_eq!(CopyActionType::Command, CopyActionType::Command);
        assert_eq!(CopyActionType::Output, CopyActionType::Output);
        assert_ne!(CopyActionType::Block, CopyActionType::Command);
    }

    #[test]
    fn test_command_blocks_widget_copy_selected_block() {
        let mut widget = CommandBlocksWidget::new();
        let mut block = CommandBlock::new("block1", "Build");
        let mut cmd = Command::new("cargo build");
        cmd.append_output("Compiling...\nFinished");
        block.add_command(cmd);
        widget.add_block(block);

        widget.select_next_block();
        if let Some(block) = widget.get_selected_block_mut() {
            block.select_next_command();
        }

        let result = widget.copy_selected_block(CopyActionType::Command);
        assert!(result.is_ok());
        assert_eq!(
            widget.get_last_copy_result(),
            Some(&Ok(CopyActionType::Command))
        );
    }

    #[test]
    fn test_command_blocks_widget_copy_all_blocks() {
        let mut widget = CommandBlocksWidget::new();

        let mut block1 = CommandBlock::new("block1", "Build");
        let mut cmd1 = Command::new("cargo build");
        cmd1.append_output("Success");
        block1.add_command(cmd1);
        widget.add_block(block1);

        let mut block2 = CommandBlock::new("block2", "Test");
        let mut cmd2 = Command::new("cargo test");
        cmd2.append_output("All tests passed");
        block2.add_command(cmd2);
        widget.add_block(block2);

        let result = widget.copy_all_blocks();
        assert!(result.is_ok());
        assert_eq!(
            widget.get_last_copy_result(),
            Some(&Ok(CopyActionType::Block))
        );
    }

    #[test]
    fn test_command_blocks_widget_copy_block_by_index() {
        let mut widget = CommandBlocksWidget::new();
        let mut block = CommandBlock::new("block1", "Build");
        let mut cmd = Command::new("cargo build");
        cmd.append_output("Success");
        block.add_command(cmd);
        widget.add_block(block);

        // Select the command in the block first
        if let Some(block) = widget.blocks.get_mut(0) {
            block.select_next_command();
        }

        let result = widget.copy_block_by_index(0, CopyActionType::Output);
        assert!(result.is_ok());
    }

    #[test]
    fn test_command_blocks_widget_copy_no_block_selected() {
        let mut widget = CommandBlocksWidget::new();
        let result = widget.copy_selected_block(CopyActionType::Command);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No block selected");
    }

    #[test]
    fn test_command_blocks_widget_copy_result_tracking() {
        let mut widget = CommandBlocksWidget::new();
        assert_eq!(widget.get_last_copy_result(), None);

        let mut block = CommandBlock::new("block1", "Build");
        let cmd = Command::new("cargo build");
        block.add_command(cmd);
        widget.add_block(block);

        widget.select_next_block();
        if let Some(block) = widget.get_selected_block_mut() {
            block.select_next_command();
        }

        let _ = widget.copy_selected_block(CopyActionType::Command);
        assert!(widget.get_last_copy_result().is_some());

        widget.clear_copy_result();
        assert_eq!(widget.get_last_copy_result(), None);
    }

    #[test]
    fn test_command_blocks_widget_copy_size_limit() {
        let mut widget = CommandBlocksWidget::new();
        let mut block = CommandBlock::new("block1", "Build");
        let mut cmd = Command::new("cargo build");

        // Create output larger than limit
        let large_output = "x".repeat(101 * 1024 * 1024);
        cmd.append_output(&large_output);
        block.add_command(cmd);
        widget.add_block(block);

        widget.select_next_block();
        if let Some(block) = widget.get_selected_block_mut() {
            block.select_next_command();
        }

        let result = widget.copy_selected_block(CopyActionType::Output);
        assert!(result.is_err());
    }
}
