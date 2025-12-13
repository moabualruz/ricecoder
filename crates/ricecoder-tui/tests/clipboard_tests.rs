use ricecoder_tui::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_operation_creation() {
        let op = CopyOperation::new("test content");
        assert_eq!(op.content, "test content");
        assert_eq!(op.feedback, None);
        assert_eq!(op.feedback_frame, 0);
    }

    #[test]
    fn test_copy_operation_feedback_progress() {
        let op = CopyOperation::new("test");
        assert_eq!(op.feedback_progress(), 0.0);

        let mut op = CopyOperation::new("test");
        op.feedback = Some(CopyFeedback::Success);
        op.feedback_frame = 30;
        op.feedback_duration = 60;
        assert_eq!(op.feedback_progress(), 0.5);

        op.feedback_frame = 60;
        assert_eq!(op.feedback_progress(), 1.0);
    }

    #[test]
    fn test_copy_operation_feedback_animation() {
        let mut op = CopyOperation::new("test");
        op.feedback = Some(CopyFeedback::Success);
        op.feedback_duration = 3;

        assert!(op.is_feedback_visible());
        assert_eq!(op.get_feedback(), Some(CopyFeedback::Success));

        op.update_feedback();
        assert!(op.is_feedback_visible());

        op.update_feedback();
        assert!(op.is_feedback_visible());

        op.update_feedback();
        assert!(!op.is_feedback_visible());
    }

    #[test]
    fn test_copy_feedback_display() {
        assert_eq!(
            CopyFeedback::Success.display_text(),
            "✓ Copied to clipboard"
        );
        assert_eq!(CopyFeedback::Failed.display_text(), "✗ Failed to copy");
        assert_eq!(CopyFeedback::Cancelled.display_text(), "⊘ Copy cancelled");
    }

    #[test]
    fn test_clipboard_manager_copy_text() {
        // This test will only work if clipboard is available
        let result = ClipboardManager::copy_text_static("test content");
        // We don't assert here because clipboard might not be available in test environment
        // Just verify the function doesn't panic
        let _ = result;
    }

    #[test]
    fn test_clipboard_manager_copy_command_block() {
        let manager = ClipboardManager::new();
        let result = manager.copy_command_block(
            "cargo build",
            "Compiling...\nFinished",
            "success",
        );
        // Just verify the function doesn't panic
        let _ = result;
    }

    #[test]
    fn test_clipboard_manager_copy_command_output() {
        let manager = ClipboardManager::new();
        let result = manager.copy_command_output("output text");
        let _ = result;
    }

    #[test]
    fn test_clipboard_manager_copy_command_text() {
        let manager = ClipboardManager::new();
        let result = manager.copy_command_text("cargo test");
        let _ = result;
    }

    #[test]
    fn test_copy_operation_size_limit() {
        let manager = ClipboardManager::new();
        // Create content larger than limit
        let large_content = "x".repeat(MAX_CLIPBOARD_SIZE + 1);
        let result = manager.copy_text(&large_content);
        assert!(result.is_err());
        match result {
            Err(ClipboardError::ContentTooLarge(size)) => {
                assert_eq!(size, MAX_CLIPBOARD_SIZE + 1);
            }
            _ => panic!("Expected ContentTooLarge error"),
        }
    }

    #[test]
    fn test_clipboard_manager_creation() {
        let manager = ClipboardManager::new();
        // Backend detection should not panic
        let _backend = manager.backend();
        assert!(manager.supports_operation());
    }

    #[test]
    fn test_clipboard_manager_with_backend() {
        let manager = ClipboardManager::with_backend(ClipboardBackend::System);
        assert_eq!(manager.backend(), ClipboardBackend::System);
    }

    #[test]
    fn test_copy_code_block() {
        let manager = ClipboardManager::new();
        let result = manager.copy_code_block("fn main() {}", Some("rust"));
        // Just verify the function doesn't panic
        let _ = result;
    }

    #[test]
    fn test_copy_chat_message() {
        let manager = ClipboardManager::new();
        let result = manager.copy_chat_message("user", "Hello world", Some("10:30"));
        // Just verify the function doesn't panic
        let _ = result;
    }

    #[test]
    fn test_copy_conversation_transcript() {
        let manager = ClipboardManager::new();
        let messages = vec![
            ("user", "Hello"),
            ("assistant", "Hi there!"),
        ];
        let result = manager.copy_conversation_transcript(&messages);
        // Just verify the function doesn't panic
        let _ = result;
    }

    #[test]
    fn test_copy_formatted_data() {
        let manager = ClipboardManager::new();

        // Test plain format
        let result = manager.copy_formatted_data("plain text", CopyFormat::Plain);
        let _ = result;

        // Test markdown format
        let result = manager.copy_formatted_data("code", CopyFormat::Markdown);
        let _ = result;
    }

    #[test]
    fn test_osc52_clipboard_support_detection() {
        // Test with various TERM values
        std::env::set_var("TERM", "xterm-256color");
        assert!(Osc52Clipboard::is_supported());

        std::env::set_var("TERM", "unknown-terminal");
        assert!(!Osc52Clipboard::is_supported());

        std::env::remove_var("TERM");
        assert!(!Osc52Clipboard::is_supported());
    }

    #[test]
    fn test_tmux_clipboard_detection() {
        // Test TMUX detection
        let original_tmux = std::env::var("TMUX");

        std::env::set_var("TMUX", "/tmp/tmux-1234/default");
        assert!(TmuxClipboard::is_in_tmux());

        if original_tmux.is_ok() {
            std::env::set_var("TMUX", original_tmux.unwrap());
        } else {
            std::env::remove_var("TMUX");
        }
        // Note: We can't easily test TMUX version detection without mocking
    }
}