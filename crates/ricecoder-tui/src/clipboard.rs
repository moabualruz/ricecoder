//! Clipboard operations for copying content
//!
//! This module provides comprehensive clipboard support including:
//! - System clipboard via arboard
//! - Terminal OSC 52 sequences for remote sessions
//! - TMUX compatibility for wrapped sessions
//! - Special content formatting for code, messages, and transcripts

use std::io::{self, Write};
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

    /// OSC 52 not supported in terminal
    #[error("OSC 52 sequences not supported in this terminal")]
    Osc52NotSupported,

    /// TMUX version too old
    #[error("TMUX version {0} does not support OSC 52 passthrough")]
    TmuxVersionTooOld(String),

    /// Terminal detection failed
    #[error("Failed to detect terminal capabilities: {0}")]
    TerminalDetectionError(String),

    /// Invalid base64 encoding
    #[error("Invalid base64 encoding for OSC 52: {0}")]
    InvalidBase64(String),
}

/// Maximum clipboard content size (100 MB)
const MAX_CLIPBOARD_SIZE: usize = 100 * 1024 * 1024;

/// OSC 52 clipboard operations for terminal environments
pub struct Osc52Clipboard;

impl Osc52Clipboard {
    /// Copy text using OSC 52 escape sequences
    pub fn copy_text(text: &str) -> Result<(), ClipboardError> {
        // Check size limit
        if text.len() > MAX_CLIPBOARD_SIZE {
            return Err(ClipboardError::ContentTooLarge(text.len()));
        }

        // Encode as base64
        let encoded = base64::encode(text);

        // Create OSC 52 sequence: \e]52;c;<base64>\e\\
        let sequence = format!("\x1b]52;c;{}\x1b\\", encoded);

        // Write to stdout
        let mut stdout = io::stdout();
        stdout.write_all(sequence.as_bytes())
            .map_err(|e| ClipboardError::CopyError(format!("Failed to write OSC 52 sequence: {}", e)))?;
        stdout.flush()
            .map_err(|e| ClipboardError::CopyError(format!("Failed to flush stdout: {}", e)))?;

        Ok(())
    }

    /// Check if OSC 52 is supported in current terminal
    pub fn is_supported() -> bool {
        // Check for common terminals that support OSC 52
        if let Ok(term) = std::env::var("TERM") {
            matches!(term.as_str(),
                "xterm" | "xterm-256color" | "screen" | "screen-256color" |
                "tmux" | "tmux-256color" | "rxvt" | "rxvt-256color" |
                "alacritty" | "kitty" | "wezterm" | "foot" | "ghostty"
            )
        } else {
            false
        }
    }
}

/// TMUX clipboard wrapper for OSC 52 operations
pub struct TmuxClipboard;

impl TmuxClipboard {
    /// Copy text using TMUX passthrough for OSC 52
    pub fn copy_text(text: &str) -> Result<(), ClipboardError> {
        // Check TMUX version for OSC 52 support (requires TMUX >= 3.2)
        if let Ok(version) = Self::get_tmux_version() {
            if !Self::supports_osc52(&version) {
                return Err(ClipboardError::TmuxVersionTooOld(version));
            }
        }

        // Wrap OSC 52 sequence for TMUX passthrough
        let encoded = base64::encode(text);
        let osc52 = format!("\x1b]52;c;{}\x1b\\", encoded);

        // TMUX passthrough sequence: \ePtmux;\e<osc52>\e\\
        let tmux_sequence = format!("\x1bPtmux;{}\x1b\\", osc52);

        let mut stdout = io::stdout();
        stdout.write_all(tmux_sequence.as_bytes())
            .map_err(|e| ClipboardError::CopyError(format!("Failed to write TMUX sequence: {}", e)))?;
        stdout.flush()
            .map_err(|e| ClipboardError::CopyError(format!("Failed to flush stdout: {}", e)))?;

        Ok(())
    }

    /// Check if running inside TMUX
    pub fn is_in_tmux() -> bool {
        std::env::var("TMUX").is_ok()
    }

    /// Get TMUX version
    fn get_tmux_version() -> Result<String, ClipboardError> {
        use std::process::Command;

        let output = Command::new("tmux")
            .arg("-V")
            .output()
            .map_err(|e| ClipboardError::TerminalDetectionError(format!("Failed to run tmux -V: {}", e)))?;

        let version_output = String::from_utf8(output.stdout)
            .map_err(|e| ClipboardError::TerminalDetectionError(format!("Invalid TMUX version output: {}", e)))?;

        // Parse version from "tmux X.Y.Z"
        let version = version_output.trim()
            .strip_prefix("tmux ")
            .ok_or_else(|| ClipboardError::TerminalDetectionError("Unexpected TMUX version format".to_string()))?;

        Ok(version.to_string())
    }

    /// Check if TMUX version supports OSC 52
    fn supports_osc52(version: &str) -> bool {
        // TMUX 3.2+ supports OSC 52 passthrough
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() >= 2 {
            if let (Ok(major), Ok(minor)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                return major > 3 || (major == 3 && minor >= 2);
            }
        }
        false
    }
}

/// Clipboard backend types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardBackend {
    /// System clipboard (arboard)
    System,
    /// Terminal OSC 52 sequences
    Osc52,
    /// TMUX passthrough
    Tmux,
}

/// Clipboard manager for copy operations with multiple backends
pub struct ClipboardManager {
    backend: ClipboardBackend,
}

impl ClipboardManager {
    /// Create a new clipboard manager with auto-detected backend
    pub fn new() -> Self {
        let backend = Self::detect_backend();
        Self { backend }
    }

    /// Create clipboard manager with specific backend
    pub fn with_backend(backend: ClipboardBackend) -> Self {
        Self { backend }
    }

    /// Detect the best available clipboard backend
    pub fn detect_backend() -> ClipboardBackend {
        // Priority: System > TMUX > OSC 52
        if TmuxClipboard::is_in_tmux() {
            if let Ok(version) = TmuxClipboard::get_tmux_version() {
                if TmuxClipboard::supports_osc52(&version) {
                    return ClipboardBackend::Tmux;
                }
            }
        }

        if Osc52Clipboard::is_supported() {
            return ClipboardBackend::Osc52;
        }

        ClipboardBackend::System
    }

    /// Copy text to clipboard using the configured backend
    pub fn copy_text(&self, text: &str) -> Result<(), ClipboardError> {
        // Check size limit
        if text.len() > MAX_CLIPBOARD_SIZE {
            return Err(ClipboardError::ContentTooLarge(text.len()));
        }

        match self.backend {
            ClipboardBackend::System => Self::copy_text_system(text),
            ClipboardBackend::Osc52 => Osc52Clipboard::copy_text(text),
            ClipboardBackend::Tmux => TmuxClipboard::copy_text(text),
        }
    }

    /// Copy text using system clipboard (arboard)
    fn copy_text_system(text: &str) -> Result<(), ClipboardError> {
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

    /// Legacy static method for backward compatibility
    pub fn copy_text(text: &str) -> Result<(), ClipboardError> {
        Self::new().copy_text(text)
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
        &self,
        command: &str,
        output: &str,
        status: &str,
    ) -> Result<(), ClipboardError> {
        let content = format!(
            "Command: {}\nStatus: {}\nOutput:\n{}",
            command, status, output
        );
        self.copy_text(&content)
    }

    /// Copy command output only
    pub fn copy_command_output(&self, output: &str) -> Result<(), ClipboardError> {
        self.copy_text(output)
    }

    /// Copy command text only
    pub fn copy_command_text(&self, command: &str) -> Result<(), ClipboardError> {
        self.copy_text(command)
    }

    /// Copy code block with syntax highlighting hints
    pub fn copy_code_block(&self, code: &str, language: Option<&str>) -> Result<(), ClipboardError> {
        let formatted = if let Some(lang) = language {
            format!("```{}\n{}\n```", lang, code)
        } else {
            format!("```\n{}\n```", code)
        };
        self.copy_text(&formatted)
    }

    /// Copy chat message with role and content
    pub fn copy_chat_message(&self, role: &str, content: &str, timestamp: Option<&str>) -> Result<(), ClipboardError> {
        let formatted = if let Some(ts) = timestamp {
            format!("[{}] **{}**: {}", ts, role, content)
        } else {
            format!("**{}**: {}", role, content)
        };
        self.copy_text(&formatted)
    }

    /// Copy conversation transcript
    pub fn copy_conversation_transcript(&self, messages: &[(&str, &str)]) -> Result<(), ClipboardError> {
        let mut transcript = String::new();
        for (role, content) in messages {
            transcript.push_str(&format!("**{}**: {}\n\n", role, content));
        }
        self.copy_text(&transcript.trim_end())
    }

    /// Copy formatted data with custom formatting
    pub fn copy_formatted_data(&self, data: &str, format: CopyFormat) -> Result<(), ClipboardError> {
        let formatted = match format {
            CopyFormat::Plain => data.to_string(),
            CopyFormat::Markdown => format!("```\n{}\n```", data),
            CopyFormat::Json => {
                if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(data) {
                    serde_json::to_string_pretty(&json_value)
                        .unwrap_or_else(|_| data.to_string())
                } else {
                    data.to_string()
                }
            }
            CopyFormat::Yaml => data.to_string(), // Assume already in YAML format
            CopyFormat::Csv => data.to_string(),  // Assume already in CSV format
        };
        self.copy_text(&formatted)
    }

    /// Get current backend
    pub fn backend(&self) -> ClipboardBackend {
        self.backend
    }

    /// Check if current backend supports the operation
    pub fn supports_operation(&self) -> bool {
        match self.backend {
            ClipboardBackend::System => {
                // System clipboard is always available (may fail at runtime)
                true
            }
            ClipboardBackend::Osc52 => Osc52Clipboard::is_supported(),
            ClipboardBackend::Tmux => {
                TmuxClipboard::is_in_tmux() &&
                TmuxClipboard::get_tmux_version()
                    .map(|v| TmuxClipboard::supports_osc52(&v))
                    .unwrap_or(false)
            }
        }
    }

    /// Legacy static methods for backward compatibility
    pub fn copy_command_block(command: &str, output: &str, status: &str) -> Result<(), ClipboardError> {
        Self::new().copy_command_block(command, output, status)
    }

    pub fn copy_command_output(output: &str) -> Result<(), ClipboardError> {
        Self::new().copy_command_output(output)
    }

    pub fn copy_command_text(command: &str) -> Result<(), ClipboardError> {
        Self::new().copy_command_text(command)
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

/// Content formatting options for clipboard operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopyFormat {
    /// Plain text
    Plain,
    /// Markdown code block
    Markdown,
    /// Pretty-printed JSON
    Json,
    /// YAML format
    Yaml,
    /// CSV format
    Csv,
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
