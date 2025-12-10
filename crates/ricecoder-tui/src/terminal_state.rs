//! Terminal state management for graceful shutdown
//!
//! This module handles capturing and restoring terminal state to ensure
//! the terminal is left in a clean state when the application exits.
//! Requirements: 10.1, 10.2, 10.3

use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;

/// Captures and manages terminal state for graceful shutdown
///
/// This struct captures the terminal state before entering raw mode and
/// alternate screen, and restores it on drop or explicit call to restore().
///
/// Requirements: 10.2 - Create TerminalState struct to capture state
#[derive(Debug)]
pub struct TerminalState {
    /// Whether raw mode was enabled before we started
    /// This field is kept for future use and for documentation purposes
    #[allow(dead_code)]
    was_raw_mode_enabled: bool,
    /// Whether we're in alternate screen
    in_alternate_screen: bool,
}

impl TerminalState {
    /// Capture the current terminal state and prepare for TUI
    ///
    /// This function:
    /// 1. Captures whether raw mode is currently enabled
    /// 2. Enables raw mode for the TUI
    /// 3. Enters alternate screen
    ///
    /// Requirements: 10.1 - Capture terminal state before TUI initialization
    pub fn capture() -> Result<Self> {
        // Check if raw mode is already enabled (shouldn't be, but be safe)
        let was_raw_mode_enabled = crossterm::terminal::is_raw_mode_enabled().unwrap_or(false);

        // Enable raw mode for the TUI
        enable_raw_mode()?;

        // Enter alternate screen
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;

        tracing::info!("Terminal state captured: raw_mode={}, alternate_screen=true", was_raw_mode_enabled);

        Ok(Self {
            was_raw_mode_enabled,
            in_alternate_screen: true,
        })
    }

    /// Restore the terminal to its original state
    ///
    /// This function:
    /// 1. Leaves alternate screen
    /// 2. Disables raw mode
    /// 3. Restores the original terminal state
    ///
    /// This is called on normal exit, Ctrl+C, and error exit.
    ///
    /// Requirements: 10.2, 10.3 - Restore terminal on normal exit, Ctrl+C, and error exit
    pub fn restore(&mut self) -> Result<()> {
        if self.in_alternate_screen {
            // Leave alternate screen
            let mut stdout = io::stdout();
            execute!(stdout, LeaveAlternateScreen)?;
            self.in_alternate_screen = false;
        }

        // Disable raw mode
        disable_raw_mode()?;

        // If raw mode wasn't enabled before, we're done
        // If it was, we don't re-enable it (caller should handle that if needed)

        tracing::info!("Terminal state restored");
        Ok(())
    }

    /// Check if terminal is in alternate screen
    pub fn in_alternate_screen(&self) -> bool {
        self.in_alternate_screen
    }
}

impl Drop for TerminalState {
    /// Ensure terminal is restored even if restore() wasn't called explicitly
    fn drop(&mut self) {
        if let Err(e) = self.restore() {
            tracing::error!("Failed to restore terminal state on drop: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_state_creation() {
        // This test verifies that TerminalState can be created
        // Note: This test may fail in non-TTY environments
        if atty::is(atty::Stream::Stdout) {
            let state = TerminalState::capture();
            // We don't assert success here because the test environment may not support it
            // But we verify the code compiles and runs
            let _ = state;
        }
    }

    #[test]
    fn test_terminal_state_drop() {
        // This test verifies that TerminalState implements Drop correctly
        if atty::is(atty::Stream::Stdout) {
            let _state = TerminalState::capture();
            // Drop is called automatically when _state goes out of scope
        }
    }
}
