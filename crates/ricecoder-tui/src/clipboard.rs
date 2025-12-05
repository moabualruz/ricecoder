//! Clipboard operations for copying content

use thiserror::Error;

/// Clipboard error types
#[derive(Debug, Error)]
pub enum ClipboardError {
    /// Failed to access clipboard
    #[error("Failed to access clipboard: {0}")]
    AccessError(String),

    /// Failed to copy content
    #[error("Failed to copy content: {0}")]
    CopyError(String),

    /// Failed to read from clipboard
    #[error("Failed to read from clipboard: {0}")]
    ReadError(String),

    /// Content is too large
    #[error("Content is too large to copy: {0} bytes")]
    ContentTooLarge(usize),
}

/// Maximum clipboard content size (100 MB)
const MAX_CLIPBOARD_SIZE: usize = 100 * 1024 * 1024;

/// Clipboard manager for copy operations
pub struct ClipboardManager;

impl ClipboardManager {
    /// Copy text to clipboard
    pub fn copy_text(text: &str) -> Result<(), ClipboardError> {
        // Check size limit
        if text.len() > MAX_CLIPBOARD_SIZE {
            return Err(ClipboardError::ContentTooLarge(text.len()));
        }

        match arboard::Clipboard::new() {
            Ok(mut clipboard) => {
                clipboard
                    .set_text(text)
                    .map_err(|e| ClipboardError::CopyError(e.to_string()))?;
                Ok(())
            }
            Err(e) => Err(ClipboardError::AccessError(e.to_string())),
        }
    }

    /// Read text from clipboard
    pub fn read_text() -> Result<String, ClipboardError> {
        match arboard::Clipboard::new() {
            Ok(mut clipboard) => clipboard
                .get_text()
                .map_err(|e| ClipboardError::ReadError(e.to_string())),
            Err(e) => Err(ClipboardError::AccessError(e.to_string())),
        }
    }

    /// Copy command block content
    pub fn copy_command_block(
        command: &str,
        output: &str,
        status: &str,
    ) -> Result<(), ClipboardError> {
        let content = format!(
            "Command: {}\nStatus: {}\nOutput:\n{}",
            command, status, output
        );
        Self::copy_text(&content)
    }

    /// Copy command output only
    pub fn copy_command_output(output: &str) -> Result<(), ClipboardError> {
        Self::copy_text(output)
    }

    /// Copy command text only
    pub fn copy_command_text(command: &str) -> Result<(), ClipboardError> {
        Self::copy_text(command)
    }
}

/// Copy action feedback
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopyFeedback {
    /// Copy was successful
    Success,
    /// Copy failed
    Failed,
    /// Copy was cancelled
    Cancelled,
}

impl CopyFeedback {
    /// Get display text for the feedback
    pub fn display_text(&self) -> &'static str {
        match self {
            CopyFeedback::Success => "✓ Copied to clipboard",
            CopyFeedback::Failed => "✗ Failed to copy",
            CopyFeedback::Cancelled => "⊘ Copy cancelled",
        }
    }
}

/// Copy operation state
#[derive(Debug, Clone)]
pub struct CopyOperation {
    /// Content being copied
    pub content: String,
    /// Copy feedback
    pub feedback: Option<CopyFeedback>,
    /// Feedback display duration in frames
    pub feedback_duration: u32,
    /// Current feedback frame counter
    pub feedback_frame: u32,
}

impl CopyOperation {
    /// Create a new copy operation
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            feedback: None,
            feedback_duration: 60, // ~1 second at 60 FPS
            feedback_frame: 0,
        }
    }

    /// Execute the copy operation
    pub fn execute(&mut self) -> Result<(), ClipboardError> {
        match ClipboardManager::copy_text(&self.content) {
            Ok(()) => {
                self.feedback = Some(CopyFeedback::Success);
                self.feedback_frame = 0;
                Ok(())
            }
            Err(e) => {
                self.feedback = Some(CopyFeedback::Failed);
                self.feedback_frame = 0;
                Err(e)
            }
        }
    }

    /// Update feedback animation
    pub fn update_feedback(&mut self) {
        if self.feedback.is_some() {
            self.feedback_frame += 1;
            if self.feedback_frame >= self.feedback_duration {
                self.feedback = None;
                self.feedback_frame = 0;
            }
        }
    }

    /// Check if feedback is visible
    pub fn is_feedback_visible(&self) -> bool {
        self.feedback.is_some()
    }

    /// Get current feedback
    pub fn get_feedback(&self) -> Option<CopyFeedback> {
        self.feedback
    }

    /// Get feedback progress (0.0 to 1.0)
    pub fn feedback_progress(&self) -> f32 {
        if self.feedback_duration == 0 {
            1.0
        } else {
            self.feedback_frame as f32 / self.feedback_duration as f32
        }
    }
}

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
        let result = ClipboardManager::copy_text("test content");
        // We don't assert here because clipboard might not be available in test environment
        // Just verify the function doesn't panic
        let _ = result;
    }

    #[test]
    fn test_clipboard_manager_copy_command_block() {
        let result = ClipboardManager::copy_command_block(
            "cargo build",
            "Compiling...\nFinished",
            "success",
        );
        // Just verify the function doesn't panic
        let _ = result;
    }

    #[test]
    fn test_clipboard_manager_copy_command_output() {
        let result = ClipboardManager::copy_command_output("output text");
        let _ = result;
    }

    #[test]
    fn test_clipboard_manager_copy_command_text() {
        let result = ClipboardManager::copy_command_text("cargo test");
        let _ = result;
    }

    #[test]
    fn test_copy_operation_size_limit() {
        // Create content larger than limit
        let large_content = "x".repeat(MAX_CLIPBOARD_SIZE + 1);
        let result = ClipboardManager::copy_text(&large_content);
        assert!(result.is_err());
        match result {
            Err(ClipboardError::ContentTooLarge(size)) => {
                assert_eq!(size, MAX_CLIPBOARD_SIZE + 1);
            }
            _ => panic!("Expected ContentTooLarge error"),
        }
    }
}
