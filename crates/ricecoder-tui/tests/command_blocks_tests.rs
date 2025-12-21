use ricecoder_tui::*;

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
